//! Validation, comparison, and benchmarking helpers for the workspace.
//!
//! The validation crate compares the algorithmic chart backends against the
//! checked-in JPL Horizons snapshot corpus and renders reproducible reports for
//! stage-4 work.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
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
pub use report::{BenchmarkReport, ValidationReport};
use compatibility::{has_surrounding_whitespace, summarize_validation_reference_points};
pub use compatibility::{
    compatibility_profile_verification_summary, verify_compatibility_profile,
    CompatibilityProfileVerificationSummary,
};
use comparison::validate_comparison_tolerance;
use comparison::{
    body_class, body_class_summaries, body_class_tolerance_summaries, comparison_audit_summary,
    comparison_audit_summary_for_report, comparison_audit_totals, comparison_tolerance_for_body,
    comparison_tolerance_policy_entries, comparison_tolerance_scope_for_body,
    format_regression_bodies, format_summary_body, BodyClass, BodyClassSummary,
    BodyClassToleranceSummary,
};
pub use comparison::{
    comparison_tolerance_catalog_entries, compare_backends, default_candidate_backend,
    default_reference_backend, BodyComparisonSummary,
    ComparisonAuditSummary, ComparisonReport, ComparisonSample, ComparisonSummary,
    ComparisonTolerance, ComparisonToleranceEntry, ComparisonTolerancePolicySummary,
    ComparisonToleranceScope, ComparisonToleranceScopeCoverageSummary, RegressionArchive,
    RegressionFinding,
};
use corpus::benchmark_timing_corpus;
pub use corpus::{
    benchmark_corpus, default_corpus, release_grade_corpus, CorpusSummary, ValidationCorpus,
};
pub use provenance::{
    benchmark_provenance_text, workspace_provenance, workspace_provenance_summary_for_report,
    WorkspaceProvenance, WorkspaceProvenanceValidationError,
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

fn comparison_tolerance_policy_coverage(
    comparison: &ComparisonReport,
) -> Vec<ComparisonToleranceScopeCoverageSummary> {
    let entries = comparison_tolerance_policy_entries(&comparison.candidate_backend.family);
    let tolerance_summaries = comparison.tolerance_summaries();

    entries
        .into_iter()
        .map(|entry| {
            let mut bodies = Vec::new();
            let mut sample_count = 0;

            for summary in &tolerance_summaries {
                if comparison_tolerance_scope_for_body(&summary.body) == entry.scope {
                    bodies.push(summary.body.clone());
                    sample_count += summary.sample_count;
                }
            }

            ComparisonToleranceScopeCoverageSummary {
                entry,
                body_count: bodies.len(),
                bodies,
                sample_count,
            }
        })
        .collect()
}

fn write_tolerance_policy(
    f: &mut fmt::Formatter<'_>,
    comparison: &ComparisonReport,
) -> fmt::Result {
    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let summary = match validated_comparison_tolerance_policy_summary_for_report(comparison) {
        Ok(summary) => summary,
        Err(error) => {
            writeln!(f, "Tolerance policy catalog")?;
            writeln!(f, "  unavailable ({error})")?;
            return Ok(());
        }
    };
    let coordinate_frames = format_frames(&summary.coordinate_frames);
    writeln!(f, "Tolerance policy catalog")?;
    writeln!(f, "  candidate backend family: {}", family_label)?;
    writeln!(
        f,
        "  comparison evidence: {} bodies, {} samples",
        summary.comparison_body_count, summary.comparison_sample_count
    )?;
    writeln!(
        f,
        "  comparison window: {}",
        summary.comparison_window.summary_line()
    )?;
    writeln!(f, "  coordinate frames: {}", coordinate_frames)?;
    for scope_coverage in summary.coverage {
        writeln!(f, "  {}", scope_coverage.summary_line())?;
    }
    Ok(())
}

fn write_tolerance_policy_text(text: &mut String, comparison: &ComparisonReport) {
    use std::fmt::Write as _;

    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let summary = match validated_comparison_tolerance_policy_summary_for_report(comparison) {
        Ok(summary) => summary,
        Err(error) => {
            let _ = writeln!(text, "Tolerance policy catalog");
            let _ = writeln!(text, "  unavailable ({error})");
            return;
        }
    };
    let coordinate_frames = format_frames(&summary.coordinate_frames);
    let _ = writeln!(text, "Tolerance policy catalog");
    let _ = writeln!(text, "  candidate backend family: {}", family_label);
    let _ = writeln!(
        text,
        "  comparison evidence: {} bodies, {} samples",
        summary.comparison_body_count, summary.comparison_sample_count
    );
    let _ = writeln!(
        text,
        "  comparison window: {}",
        summary.comparison_window.summary_line()
    );
    let _ = writeln!(text, "  coordinate frames: {}", coordinate_frames);
    for scope_coverage in summary.coverage {
        let _ = writeln!(text, "  {}", scope_coverage.summary_line());
    }
}

/// Per-body comparison status against the expected tolerance table.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyToleranceSummary {
    /// Body queried for this tolerance summary.
    pub body: CelestialBody,
    /// Expected tolerance for the body.
    pub tolerance: ComparisonTolerance,
    /// Number of samples compared for this body.
    pub sample_count: usize,
    /// Whether all measured deltas are within the expected tolerance.
    pub within_tolerance: bool,
    /// Maximum absolute longitude delta measured for this body.
    pub max_longitude_delta_deg: f64,
    /// Signed margin between the longitude limit and measured maximum.
    pub longitude_margin_deg: f64,
    /// Maximum absolute latitude delta measured for this body.
    pub max_latitude_delta_deg: f64,
    /// Signed margin between the latitude limit and measured maximum.
    pub latitude_margin_deg: f64,
    /// Maximum absolute distance delta measured for this body.
    pub max_distance_delta_au: Option<f64>,
    /// Signed margin between the distance limit and measured maximum.
    pub distance_margin_au: Option<f64>,
}

impl BodyToleranceSummary {
    /// Returns `Ok(())` when the tolerance status is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        validate_comparison_tolerance(&self.tolerance)?;

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} has no samples to compare",
                    self.body
                ),
            ));
        }

        for (label, value) in [
            ("longitude", self.max_longitude_delta_deg),
            ("latitude", self.max_latitude_delta_deg),
            ("longitude margin", self.longitude_margin_deg),
            ("latitude margin", self.latitude_margin_deg),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid {} {}",
                        self.body, label, value
                    ),
                ));
            }
        }

        if let Some(value) = self.max_distance_delta_au {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance delta {}",
                        self.body, value
                    ),
                ));
            }
        }

        if let Some(value) = self.distance_margin_au {
            if !value.is_finite() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance margin {}",
                        self.body, value
                    ),
                ));
            }
        }

        let tolerance = &self.tolerance;
        let distance_margin = self.distance_margin_au;
        let has_distance_limit = tolerance.max_distance_delta_au.is_some();
        let has_distance_measurement = self.max_distance_delta_au.is_some();
        if distance_margin.is_some() != (has_distance_limit && has_distance_measurement) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} distance-margin presence does not match the measured values and tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_longitude_margin =
            tolerance.max_longitude_delta_deg - self.max_longitude_delta_deg;
        if self.longitude_margin_deg != expected_longitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} longitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_latitude_margin =
            tolerance.max_latitude_delta_deg - self.max_latitude_delta_deg;
        if self.latitude_margin_deg != expected_latitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} latitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        if let (Some(measured), Some(limit), Some(margin)) = (
            self.max_distance_delta_au,
            tolerance.max_distance_delta_au,
            self.distance_margin_au,
        ) {
            if margin != limit - measured {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} distance margin drifted from the declared tolerance limit",
                        self.body
                    ),
                ));
            }
        }

        let within_tolerance = self.longitude_margin_deg >= 0.0
            && self.latitude_margin_deg >= 0.0
            && self
                .distance_margin_au
                .map(|value| value >= 0.0)
                .unwrap_or(true);
        if self.within_tolerance != within_tolerance {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} status disagrees with the measured margins",
                    self.body
                ),
            ));
        }

        Ok(())
    }

    /// Renders the compact report wording after validating the summary fields.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Renders the compact report wording for this tolerance status.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: backend family={}, profile={}, samples={}, status={}, limit Δlon≤{:.6}°, margin Δlon={:+.12}°, limit Δlat≤{:.6}°, margin Δlat={:+.12}°, limit Δdist={}, margin Δdist={}",
            self.body,
            tolerance_backend_family_label(&self.tolerance.backend_family),
            self.tolerance.profile,
            self.sample_count,
            if self.within_tolerance { "within" } else { "exceeded" },
            self.tolerance.max_longitude_delta_deg,
            self.longitude_margin_deg,
            self.tolerance.max_latitude_delta_deg,
            self.latitude_margin_deg,
            self.tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.distance_margin_au
                .map(|value| format!("{value:+.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }
}

impl fmt::Display for BodyToleranceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}


/// Renders a compact workspace audit summary used by the CLI and release bundle.
pub fn render_workspace_audit_summary() -> Result<String, std::io::Error> {
    let report = workspace_audit_report()?;
    Ok(render_workspace_audit_summary_text(&report))
}

/// Renders the compact native-dependency audit summary used by release bundling.
///
/// This stays explicit even though it currently shares the same underlying report,
/// so release-bundle bookkeeping can keep the native-dependency path separate.
pub fn render_native_dependency_audit_summary() -> Result<String, std::io::Error> {
    render_workspace_audit_summary()
}

/// Benchmarks a backend against a validation corpus.
pub fn benchmark_backend(
    backend: &dyn EphemerisBackend,
    corpus: &ValidationCorpus,
    rounds: usize,
) -> Result<BenchmarkReport, EphemerisError> {
    let single_start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(backend.position(request)?);
        }
    }
    let elapsed = single_start.elapsed();

    let batch_start = StdInstant::now();
    for _ in 0..rounds {
        std::hint::black_box(backend.positions(&corpus.requests)?);
    }
    let batch_elapsed = batch_start.elapsed();

    let report = BenchmarkReport {
        backend: backend.metadata(),
        corpus_name: corpus.name.clone(),
        apparentness: corpus.apparentness,
        rounds,
        sample_count: corpus.requests.len(),
        elapsed,
        batch_elapsed,
        estimated_corpus_heap_bytes: corpus.estimated_heap_bytes(),
    };
    report.validate()?;
    Ok(report)
}

/// Computes a deterministic 64-bit checksum for bundle text.
fn checksum64(text: &str) -> u64 {
    checksum64_bytes(text.as_bytes())
}

/// Computes a deterministic 64-bit checksum for arbitrary bytes.
fn checksum64_bytes(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Renders the compact compatibility-profile summary used by release tooling.
pub fn render_compatibility_profile_summary() -> String {
    render_compatibility_profile_summary_text()
}

/// Renders the compact compatibility-caveats summary used by release tooling.
pub fn render_compatibility_caveats_summary() -> String {
    render_compatibility_caveats_summary_text()
}

/// Renders the compact latitude-sensitive house failure modes summary used by release tooling.
pub fn render_house_latitude_sensitive_failure_modes_summary() -> String {
    format_latitude_sensitive_house_failure_modes_for_report()
}

/// Renders the compact known-gaps summary used by release tooling.
pub fn render_known_gaps_summary() -> String {
    render_known_gaps_summary_text()
}

/// Renders the compact compatibility catalog inventory summary used by release tooling.
pub fn render_catalog_inventory_summary() -> String {
    render_catalog_inventory_summary_text()
}

fn render_catalog_inventory_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| match validated_catalog_inventory_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Compatibility catalog inventory unavailable ({error})"),
        })
        .clone()
}

fn render_known_gaps_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| match validated_known_gaps_summary_for_report() {
            Ok(summary) => format!("Known gaps: {summary}"),
            Err(error) => format!("Known gaps unavailable ({error})"),
        })
        .clone()
}

/// Renders the compact compatibility catalog posture summary used by release tooling.
pub fn render_catalog_posture_summary() -> String {
    render_catalog_posture_summary_text()
}

fn render_catalog_posture_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(
            || match core_validated_catalog_posture_summary_for_report() {
                Ok(summary) => summary,
                Err(error) => format!("Compatibility catalog posture unavailable ({error})"),
            },
        )
        .clone()
}

/// Renders the compact custom-definition ayanamsa label summary used by release tooling.
pub fn render_custom_definition_ayanamsa_labels_summary() -> String {
    format_custom_definition_ayanamsa_labels_for_report()
}

/// Renders the compact release-specific house-system canonical-name summary used by release tooling.
pub fn render_release_house_system_canonical_names_summary() -> String {
    format_release_house_system_canonical_names_for_report()
}

/// Renders the compact release-specific ayanamsa canonical-name summary used by release tooling.
pub fn render_release_ayanamsa_canonical_names_summary() -> String {
    format_release_ayanamsa_canonical_names_for_report()
}

/// Renders the compact ayanamsa audit summary used by release tooling.
pub fn render_ayanamsa_audit_summary() -> String {
    format_ayanamsa_audit_for_report()
}

/// Renders the compact target house-system scope summary used by release tooling.
pub fn render_target_house_scope_summary() -> String {
    match core_validated_target_house_scope_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Compatibility profile target house scope unavailable ({error})"),
    }
}

/// Renders the compact target ayanamsa scope summary used by release tooling.
pub fn render_target_ayanamsa_scope_summary() -> String {
    match core_validated_target_ayanamsa_scope_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Compatibility profile target ayanamsa scope unavailable ({error})"),
    }
}

/// Renders the release notes used by release tooling.
pub fn render_release_notes() -> String {
    render_release_notes_text()
}

/// Renders the compact release notes summary used by release tooling.
pub fn render_release_notes_summary() -> String {
    render_release_notes_summary_text()
}

/// Renders the release checklist used by release tooling.
pub fn render_release_checklist() -> String {
    render_release_checklist_text()
}

/// Renders the compact release checklist summary used by release tooling.
pub fn render_release_checklist_summary() -> String {
    render_release_checklist_summary_text()
}

/// Renders the compact release summary used by release tooling.
pub fn render_release_summary() -> String {
    render_release_summary_text()
}

/// Renders the compact Delta T policy summary used by validation and release tooling.
pub fn render_delta_t_policy_summary() -> String {
    render_delta_t_policy_summary_text()
}

/// Renders the compact request-policy summary used by validation and release tooling.
pub fn render_request_policy_summary() -> String {
    render_request_policy_summary_text()
}

/// Renders the compact request-surface inventory used by validation and release tooling.
pub fn render_request_surface_summary() -> String {
    render_request_surface_summary_text()
}

#[derive(Clone, Debug, PartialEq)]
struct AyanamsaReferenceOffsetExample {
    canonical_name: &'static str,
    epoch: JulianDay,
    offset_degrees: Angle,
}

#[derive(Clone, Debug, PartialEq)]
struct AyanamsaReferenceOffsetsSummary {
    examples: Vec<AyanamsaReferenceOffsetExample>,
}

#[derive(Clone, Debug, PartialEq)]
struct AyanamsaProvenanceExample {
    canonical_name: &'static str,
    provenance_note: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct AyanamsaProvenanceSummary {
    examples: Vec<AyanamsaProvenanceExample>,
}

impl AyanamsaReferenceOffsetsSummary {
    fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "ayanamsa reference offsets",
            self.examples.iter().map(|example| example.canonical_name),
        )?;

        Ok(())
    }

    fn summary_line(&self) -> String {
        match self.examples.as_slice() {
            [] => "representative zero-point examples: 0 (none)".to_string(),
            [single] => format!(
                "representative zero-point examples: 1 ({}: epoch={}; offset={})",
                single.canonical_name, single.epoch, single.offset_degrees
            ),
            _ => format!(
                "representative zero-point examples: {}",
                self.examples
                    .iter()
                    .map(|example| format!(
                        "{}: epoch={}; offset={}",
                        example.canonical_name, example.epoch, example.offset_degrees
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

impl fmt::Display for AyanamsaReferenceOffsetsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl AyanamsaProvenanceSummary {
    fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "ayanamsa provenance examples",
            self.examples.iter().map(|example| example.canonical_name),
        )?;

        for example in &self.examples {
            if example.provenance_note.trim().is_empty()
                || example.provenance_note.contains('\n')
                || example.provenance_note.contains('\r')
                || has_surrounding_whitespace(example.provenance_note)
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "ayanamsa provenance example `{}` has an unnormalized provenance note",
                        example.canonical_name
                    ),
                ));
            }
        }

        Ok(())
    }

    fn summary_line(&self) -> String {
        match self.examples.as_slice() {
            [] => "representative provenance examples: 0 (none)".to_string(),
            [single] => format!(
                "representative provenance examples: 1 ({} — {})",
                single.canonical_name, single.provenance_note
            ),
            _ => format!(
                "representative provenance examples: {}",
                self.examples
                    .iter()
                    .map(|example| format!(
                        "{} — {}",
                        example.canonical_name, example.provenance_note
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }

    fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for AyanamsaProvenanceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn summarize_ayanamsa_reference_offsets() -> Result<AyanamsaReferenceOffsetsSummary, EphemerisError>
{
    let samples = pleiades_ayanamsa::reference_offset_sample_ayanamsas();

    let mut examples = Vec::with_capacity(samples.len());
    for sample in samples {
        let descriptor = descriptor(sample).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("ayanamsa reference offsets sample `{sample}` is unavailable"),
            )
        })?;
        let epoch = descriptor.epoch.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference epoch",
                    descriptor.canonical_name
                ),
            )
        })?;
        let offset_degrees = descriptor.offset_degrees.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference offset",
                    descriptor.canonical_name
                ),
            )
        })?;

        examples.push(AyanamsaReferenceOffsetExample {
            canonical_name: descriptor.canonical_name,
            epoch,
            offset_degrees,
        });
    }

    let summary = AyanamsaReferenceOffsetsSummary { examples };
    summary.validate()?;
    Ok(summary)
}

fn validated_ayanamsa_reference_offsets_summary_for_report(
    summary: &AyanamsaReferenceOffsetsSummary,
) -> Result<String, EphemerisError> {
    summary.validate()?;
    Ok(summary.to_string())
}

fn format_ayanamsa_reference_offsets_for_report() -> String {
    match summarize_ayanamsa_reference_offsets() {
        Ok(summary) => match validated_ayanamsa_reference_offsets_summary_for_report(&summary) {
            Ok(summary) => format!("Ayanamsa reference offsets: {summary}"),
            Err(error) => format!("Ayanamsa reference offsets: unavailable ({error})"),
        },
        Err(error) => format!("Ayanamsa reference offsets: unavailable ({error})"),
    }
}

fn summarize_ayanamsa_provenance() -> Result<AyanamsaProvenanceSummary, EphemerisError> {
    let samples = pleiades_ayanamsa::provenance_sample_ayanamsas();

    let mut examples = Vec::with_capacity(samples.len());
    for sample in samples {
        let descriptor = descriptor(sample).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("ayanamsa provenance sample `{sample}` is unavailable"),
            )
        })?;

        examples.push(AyanamsaProvenanceExample {
            canonical_name: descriptor.canonical_name,
            provenance_note: descriptor.notes,
        });
    }

    let summary = AyanamsaProvenanceSummary { examples };
    summary.validate()?;
    Ok(summary)
}

fn format_ayanamsa_catalog_validation_for_report() -> String {
    match ayanamsa_catalog_validation_summary().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("ayanamsa catalog validation: unavailable ({error})"),
    }
}

fn format_ayanamsa_metadata_coverage_for_report() -> String {
    match metadata_coverage().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("ayanamsa sidereal metadata: unavailable ({error})"),
    }
}

fn format_ayanamsa_provenance_for_report() -> String {
    match summarize_ayanamsa_provenance() {
        Ok(summary) => match summary.validated_summary_line() {
            Ok(summary) => format!("Ayanamsa provenance: {summary}"),
            Err(error) => format!("Ayanamsa provenance: unavailable ({error})"),
        },
        Err(error) => format!("Ayanamsa provenance: unavailable ({error})"),
    }
}

fn format_ayanamsa_audit_for_report() -> String {
    format!(
        "Ayanamsa audit: {}; {}; {}; {}",
        format_ayanamsa_catalog_validation_for_report(),
        format_ayanamsa_metadata_coverage_for_report(),
        format_ayanamsa_reference_offsets_for_report(),
        format_ayanamsa_provenance_for_report(),
    )
}

fn format_house_code_aliases_for_report() -> String {
    match pleiades_houses::validated_house_system_code_aliases_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("house-code aliases unavailable ({error})"),
    }
}

fn format_house_formula_families_for_report() -> String {
    match validated_house_formula_families_summary_for_report() {
        Ok(summary) => format!("House formula families: {summary}"),
        Err(error) => format!("house formula families unavailable ({error})"),
    }
}

fn format_latitude_sensitive_house_systems_for_report() -> String {
    match validated_latitude_sensitive_house_systems_summary_for_report() {
        Ok(summary) => format!("Latitude-sensitive house systems: {summary}"),
        Err(error) => format!("Latitude-sensitive house systems unavailable ({error})"),
    }
}

fn format_latitude_sensitive_house_constraints_for_report() -> String {
    match validated_latitude_sensitive_house_constraints_summary_for_report() {
        Ok(summary) => format!("Latitude-sensitive house constraints: {summary}"),
        Err(error) => format!("Latitude-sensitive house constraints unavailable ({error})"),
    }
}

fn format_latitude_sensitive_house_failure_modes_for_report() -> String {
    match validated_latitude_sensitive_house_failure_modes_summary_for_report() {
        Ok(summary) => format!("Latitude-sensitive house failure modes: {summary}"),
        Err(error) => format!("Latitude-sensitive house failure modes unavailable ({error})"),
    }
}

fn format_custom_definition_ayanamsa_labels_for_report() -> String {
    match validated_custom_definition_ayanamsa_labels_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("custom-definition ayanamsa labels unavailable ({error})"),
    }
}

fn validated_release_house_system_canonical_names_for_report() -> Result<String, String> {
    core_validated_release_house_system_canonical_names_summary_for_report()
        .map_err(|error| error.to_string())
}

fn validated_release_ayanamsa_canonical_names_for_report() -> Result<String, String> {
    core_validated_release_ayanamsa_canonical_names_summary_for_report()
        .map_err(|error| error.to_string())
}

fn format_release_house_system_canonical_names_for_report() -> String {
    match validated_release_house_system_canonical_names_for_report() {
        Ok(summary) => format!("Release-specific house-system canonical names: {summary}"),
        Err(error) => {
            format!("Release-specific house-system canonical names unavailable ({error})")
        }
    }
}

fn format_release_ayanamsa_canonical_names_for_report() -> String {
    match validated_release_ayanamsa_canonical_names_for_report() {
        Ok(summary) => format!("Release-specific ayanamsa canonical names: {summary}"),
        Err(error) => {
            format!("Release-specific ayanamsa canonical names unavailable ({error})")
        }
    }
}

fn validate_name_sequence<'a, I>(
    section_label: &'static str,
    names: I,
) -> Result<(), EphemerisError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen_names = BTreeSet::new();
    let mut seen_names_case_insensitive = BTreeMap::new();

    for name in names {
        if name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a blank name"),
            ));
        }

        if has_surrounding_whitespace(name) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} entry '{name}' contains surrounding whitespace"),
            ));
        }

        let normalized_name = name.trim().to_string();
        if !seen_names.insert(normalized_name.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a duplicate name '{name}'"),
            ));
        }

        let normalized_name_case_insensitive = normalized_name.to_ascii_lowercase();
        if let Some(existing_name) =
            seen_names_case_insensitive.get(&normalized_name_case_insensitive)
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "{section_label} contains a case-insensitive duplicate name '{name}' that conflicts with '{existing_name}'"
                ),
            ));
        }
        seen_names_case_insensitive.insert(normalized_name_case_insensitive, normalized_name);
    }

    Ok(())
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DescriptorNamesSummary {
    names: Vec<&'static str>,
}

#[cfg(test)]
impl DescriptorNamesSummary {
    fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence("descriptor-name summary", self.names.iter().copied())
    }

    fn summary_line(&self) -> String {
        match self.names.as_slice() {
            [] => "0 (none)".to_string(),
            [single] => format!("1 ({single})"),
            _ => format!("{} ({})", self.names.len(), self.names.join(", ")),
        }
    }
}

#[cfg(test)]
impl fmt::Display for DescriptorNamesSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
fn summarize_descriptor_names<T>(
    entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
) -> DescriptorNamesSummary {
    DescriptorNamesSummary {
        names: entries.iter().map(canonical_name).collect::<Vec<_>>(),
    }
}

fn validate_compatibility_profile_summary_text(
    text: &str,
    profile: &CompatibilityProfile,
    release_profiles: &ReleaseProfileIdentifiers,
) -> Result<(), String> {
    let expected_house_line = format!(
        "House systems: {} total ({} baseline, {} release-specific)",
        profile.house_systems.len(),
        profile.baseline_house_systems.len(),
        profile.release_house_systems.len()
    );
    if !text.contains(&expected_house_line) {
        return Err(format!(
            "compatibility profile summary house-system baseline/release split mismatch: expected `{expected_house_line}`"
        ));
    }

    let expected_constraints_line = format!(
        "Latitude-sensitive house constraints: {}",
        profile.latitude_sensitive_house_constraints_summary_line()
    );
    if !text.contains(&expected_constraints_line) {
        return Err(format!(
            "compatibility profile summary latitude-sensitive house constraints mismatch: expected `{expected_constraints_line}`"
        ));
    }

    let expected_ayanamsa_line = format!(
        "Ayanamsas: {} total ({} baseline, {} release-specific)",
        profile.ayanamsas.len(),
        profile.baseline_ayanamsas.len(),
        profile.release_ayanamsas.len()
    );
    if !text.contains(&expected_ayanamsa_line) {
        return Err(format!(
            "compatibility profile summary ayanamsa baseline/release split mismatch: expected `{expected_ayanamsa_line}`"
        ));
    }

    let expected_profile_line = format!("Profile: {}", release_profiles.compatibility_profile_id);
    if !text.contains(&expected_profile_line) {
        return Err(format!(
            "compatibility profile summary profile id mismatch: expected `{expected_profile_line}`"
        ));
    }

    let expected_unsupported_modes_line = format!(
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    );
    if !text.contains(&expected_unsupported_modes_line) {
        return Err(format!(
            "compatibility profile summary unsupported-modes mismatch: expected `{expected_unsupported_modes_line}`"
        ));
    }

    Ok(())
}

fn render_compatibility_profile_summary_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    };
    let coverage = metadata_coverage();
    let mut text = String::new();

    text.push_str("Compatibility profile summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("House systems: ");
    text.push_str(&profile.house_systems.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_house_systems.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_house_systems.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str(&format_latitude_sensitive_house_systems_for_report());
    text.push('\n');
    text.push_str(&format_latitude_sensitive_house_constraints_for_report());
    text.push('\n');
    text.push_str(&format_house_formula_families_for_report());
    text.push('\n');
    text.push_str("House code aliases: ");
    match profile.validated_house_code_aliases_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Ayanamsas: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" release-specific)\n");
    match profile.validated_target_house_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    match profile.validated_target_ayanamsa_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    match coverage.validated_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&format_ayanamsa_catalog_validation_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_metadata_coverage_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_reference_offsets_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_provenance_for_report());
    text.push('\n');
    text.push_str("Release-specific house-system canonical names: ");
    match profile.validated_release_house_system_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Release-specific ayanamsa canonical names: ");
    match profile.validated_release_ayanamsa_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Custom-definition label names: ");
    if profile.custom_definition_labels.is_empty() {
        text.push_str("none");
    } else {
        text.push_str(&profile.custom_definition_labels.join(", "));
    }
    text.push('\n');
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    text.push_str("Compatibility caveats documented: ");
    if profile.known_gaps.is_empty() {
        text.push_str("none");
    } else {
        text.push_str(&profile.known_gaps.join("; "));
    }
    text.push('\n');
    text.push_str("Unsupported modes: ");
    text.push_str(unsupported_modes_summary_for_report());
    text.push('\n');
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    if let Err(error) =
        validate_compatibility_profile_summary_text(&text, &profile, &release_profiles)
    {
        return format!("Compatibility profile summary unavailable ({error})");
    }

    text
}

fn render_compatibility_caveats_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let profile = match validated_compatibility_profile_for_report() {
                Ok(profile) => profile,
                Err(error) => {
                    return format!("Compatibility caveats summary unavailable ({error})")
                }
            };
            let release_profiles = match validated_release_profile_identifiers_for_report() {
                Ok(release_profiles) => release_profiles,
                Err(error) => {
                    return format!("Compatibility caveats summary unavailable ({error})")
                }
            };
            core_compatibility_caveats_summary_for_report(&profile, &release_profiles)
        })
        .clone()
}



fn validate_packaged_artifact_fit_posture() -> Result<(), EphemerisError> {
    let fit_envelope = packaged_artifact_fit_envelope_summary_details();
    let thresholds = packaged_artifact_fit_threshold_summary_details();
    let target_threshold = packaged_artifact_target_threshold_summary_details();
    validate_packaged_artifact_fit_posture_with(&fit_envelope, &thresholds, &target_threshold)
}

fn validate_packaged_artifact_fit_posture_with(
    fit_envelope: &pleiades_data::PackagedArtifactFitEnvelopeSummary,
    thresholds: &pleiades_data::PackagedArtifactFitThresholdSummary,
    target_threshold: &pleiades_data::PackagedArtifactTargetThresholdSummary,
) -> Result<(), EphemerisError> {
    fit_envelope.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("validation report packaged-artifact fit envelope is invalid: {error}"),
        )
    })?;
    fit_envelope
        .validate_against_thresholds(thresholds)
        .map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "validation report packaged-artifact fit envelope exceeds calibrated thresholds: {error}; measured fit envelope: {}; fit thresholds: {}",
                    fit_envelope.summary_line(),
                    thresholds.summary_line(),
                ),
            )
        })?;
    target_threshold.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "validation report packaged-artifact target-threshold summary is invalid: {error}"
            ),
        )
    })?;

    Ok(())
}

fn build_validation_report(rounds: usize) -> Result<ValidationReport, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, ValidationReport>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("validation report cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = build_validation_report_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

fn build_validation_report_uncached(rounds: usize) -> Result<ValidationReport, EphemerisError> {
    validate_packaged_artifact_fit_posture()?;
    let comparison_corpus = release_grade_corpus();
    let benchmark_corpus = benchmark_timing_corpus();
    let packaged_benchmark_corpus = artifact::packaged_artifact_corpus();
    let chart_benchmark_corpus = chart_benchmark_corpus_summary();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let packaged = PackagedDataBackend::new();
    let comparison = compare_backends(&reference, &candidate, &comparison_corpus)?;
    let reference_benchmark = benchmark_backend(&reference, &comparison_corpus, rounds)?;
    let candidate_benchmark = benchmark_backend(&candidate, &benchmark_corpus, rounds)?;
    let packaged_benchmark = benchmark_backend(&packaged, &packaged_benchmark_corpus, rounds)?;
    let artifact_decode_benchmark =
        artifact::benchmark_packaged_artifact_decode(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let chart_benchmark = benchmark_chart_backend(default_candidate_backend(), rounds)?;
    let archived_regressions = comparison.regression_archive();

    let report = ValidationReport {
        comparison_corpus: comparison_corpus.summary(),
        benchmark_corpus: benchmark_corpus.summary(),
        packaged_benchmark_corpus: packaged_benchmark_corpus.summary(),
        chart_benchmark_corpus,
        artifact_decode_benchmark,
        house_validation: house_validation_report(),
        comparison,
        archived_regressions,
        reference_benchmark,
        candidate_benchmark,
        packaged_benchmark,
        chart_benchmark,
    };
    report.validate()?;
    Ok(report)
}

/// Renders the validation report used by the CLI.
pub fn render_validation_report(rounds: usize) -> Result<String, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("validation report cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = render_validation_report_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

fn render_validation_report_uncached(rounds: usize) -> Result<String, EphemerisError> {
    Ok(build_validation_report(rounds)?.to_string())
}

/// Renders a compact validation-report summary used by the CLI.
pub fn render_validation_report_summary(rounds: usize) -> Result<String, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("validation report summary cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = render_validation_report_summary_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

fn render_validation_report_summary_uncached(rounds: usize) -> Result<String, EphemerisError> {
    let report = build_validation_report(rounds)?;
    Ok(render_validation_report_summary_text(&report))
}

fn validated_packaged_artifact_fit_sample_classes_summary_for_report(
    boundary: &ArtifactBoundaryEnvelopeSummary,
) -> Result<String, String> {
    let boundary = boundary
        .validated_summary_line()
        .map_err(|error| error.to_string())?;
    let interior = packaged_artifact_fit_envelope_summary_for_report();

    Ok(format!(
        "fit sample classes: boundary continuity={}; interior fit={}",
        boundary, interior,
    ))
}

/// Returns the combined packaged-artifact boundary and interior fit sample summary for reports.
pub fn packaged_artifact_fit_sample_classes_summary_for_report() -> String {
    let boundary = match artifact_boundary_envelope_summary_for_report() {
        Ok(boundary) => boundary,
        Err(error) => return format!("fit sample classes: unavailable ({error})"),
    };

    match validated_packaged_artifact_fit_sample_classes_summary_for_report(&boundary) {
        Ok(summary) => summary,
        Err(error) => format!("fit sample classes: unavailable ({error})"),
    }
}

/// Renders the comparison report used by the CLI.
pub fn render_comparison_report() -> Result<String, EphemerisError> {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    Ok(compare_backends(&reference, &candidate, &corpus)?.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonMedianEnvelope {
    longitude_delta_deg: f64,
    latitude_delta_deg: f64,
    distance_delta_au: Option<f64>,
}

impl ComparisonMedianEnvelope {
    /// Validates the stored median comparison envelope.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison median envelope field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = self.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison median envelope field `distance_delta_au` must be a finite non-negative value",
                ));
            }
        }

        Ok(())
    }

    /// Returns the compact median comparison envelope line.
    pub fn summary_line(&self) -> String {
        let distance = self
            .distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "median longitude delta: {:.12}°, median latitude delta: {:.12}°, median distance delta: {}",
            self.longitude_delta_deg, self.latitude_delta_deg, distance,
        )
    }
}

impl fmt::Display for ComparisonMedianEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the median comparison envelope used by the compact report.
pub fn comparison_median_envelope(
    samples: &[ComparisonSample],
) -> Result<ComparisonMedianEnvelope, EphemerisError> {
    validate_comparison_samples_for_report(samples)?;

    let envelope = comparison_median_envelope_for_samples(samples);
    envelope.validate()?;
    Ok(envelope)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonPercentileEnvelope {
    longitude_delta_deg: f64,
    latitude_delta_deg: f64,
    distance_delta_au: Option<f64>,
}

impl ComparisonPercentileEnvelope {
    /// Validates the stored 95th-percentile comparison envelope.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison percentile envelope field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = self.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison percentile envelope field `distance_delta_au` must be a finite non-negative value",
                ));
            }
        }

        Ok(())
    }

    /// Returns the compact 95th-percentile comparison envelope line.
    pub fn summary_line(&self) -> String {
        let distance = self
            .distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "95th percentile absolute deltas: longitude {:.12}°, latitude {:.12}°, distance {}",
            self.longitude_delta_deg, self.latitude_delta_deg, distance,
        )
    }
}

impl fmt::Display for ComparisonPercentileEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the 95th-percentile comparison envelope used by the compact tail report.
pub fn comparison_tail_envelope(
    samples: &[ComparisonSample],
) -> Result<ComparisonPercentileEnvelope, EphemerisError> {
    validate_comparison_samples_for_report(samples)?;

    let envelope = comparison_percentile_envelope(samples, 0.95);
    envelope.validate()?;
    Ok(envelope)
}

/// Combined comparison envelope summary used by the compact report.
///
/// The summary keeps the aggregate comparison record, the median deltas, and
/// the 95th-percentile tail together so downstream tooling can reuse the same
/// validated envelope that the report formatter renders.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonEnvelopeSummary {
    summary: ComparisonSummary,
    median: ComparisonMedianEnvelope,
    percentile: ComparisonPercentileEnvelope,
}

impl ComparisonEnvelopeSummary {
    /// Returns the compact comparison summary line with the median envelope.
    pub fn summary_line(&self) -> String {
        let summary = self
            .summary
            .validated_summary_line()
            .unwrap_or_else(|error| format!("comparison summary unavailable ({error})"));
        format!("{}; {}", summary, self.median)
    }

    /// Returns the compact comparison summary line after validating against samples.
    pub fn validated_summary_line(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<String, EphemerisError> {
        self.validate_against_samples(samples)?;
        Ok(self.summary_line())
    }

    /// Returns the compact 95th-percentile tail line.
    pub fn percentile_line(&self) -> String {
        self.percentile.summary_line()
    }

    /// Returns the compact 95th-percentile tail line after validating against samples.
    pub fn validated_percentile_line(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<String, EphemerisError> {
        self.validate_against_samples(samples)?;
        Ok(self.percentile_line())
    }

    /// Validates the stored envelope against the provided comparison samples.
    pub fn validate_against_samples(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<(), EphemerisError> {
        self.summary.validate()?;

        if self.summary.sample_count != samples.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison envelope summary sample-count mismatch: expected {}, found {}",
                    self.summary.sample_count,
                    samples.len()
                ),
            ));
        }

        if samples.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary has no samples",
            ));
        }

        for (index, sample) in samples.iter().enumerate() {
            for (label, value) in [
                ("longitude_delta_deg", sample.longitude_delta_deg),
                ("latitude_delta_deg", sample.latitude_delta_deg),
            ] {
                if !value.is_finite() || value.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison sample {} field `{label}` must be a finite non-negative value",
                            index + 1
                        ),
                    ));
                }
            }

            if let Some(distance_delta_au) = sample.distance_delta_au {
                if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison sample {} field `distance_delta_au` must be a finite non-negative value",
                            index + 1
                        ),
                    ));
                }
            }
        }

        validate_comparison_sample_distance_channels(samples)?;
        self.median.validate()?;
        self.percentile.validate()?;

        let expected_median = comparison_median_envelope_for_samples(samples);
        if self.median != expected_median {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary median drifted from the sampled comparison values",
            ));
        }

        let expected_percentile = comparison_percentile_envelope(samples, 0.95);
        if self.percentile != expected_percentile {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary percentile drifted from the sampled comparison values",
            ));
        }

        Ok(())
    }
}

impl fmt::Display for ComparisonEnvelopeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the combined comparison envelope summary used by the compact report.
pub fn comparison_envelope_summary(
    summary: &ComparisonSummary,
    samples: &[ComparisonSample],
) -> ComparisonEnvelopeSummary {
    ComparisonEnvelopeSummary {
        summary: summary.clone(),
        median: comparison_median_envelope_for_samples(samples),
        percentile: comparison_percentile_envelope(samples, 0.95),
    }
}

fn median_value(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.total_cmp(right));
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[middle - 1] + values[middle]) / 2.0)
    } else {
        Some(values[middle])
    }
}

fn percentile_value(values: &mut [f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.total_cmp(right));
    let percentile = percentile.clamp(0.0, 1.0);
    let position = percentile * (values.len().saturating_sub(1)) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        Some(values[lower_index])
    } else {
        let weight = position - lower_index as f64;
        Some(values[lower_index] + (values[upper_index] - values[lower_index]) * weight)
    }
}

fn comparison_median_envelope_for_samples(
    samples: &[ComparisonSample],
) -> ComparisonMedianEnvelope {
    let mut longitude_values = samples
        .iter()
        .map(|sample| sample.longitude_delta_deg)
        .collect::<Vec<_>>();
    let mut latitude_values = samples
        .iter()
        .map(|sample| sample.latitude_delta_deg)
        .collect::<Vec<_>>();
    let mut distance_values = samples
        .iter()
        .filter_map(|sample| sample.distance_delta_au)
        .collect::<Vec<_>>();

    ComparisonMedianEnvelope {
        longitude_delta_deg: median_value(&mut longitude_values).unwrap_or_default(),
        latitude_delta_deg: median_value(&mut latitude_values).unwrap_or_default(),
        distance_delta_au: median_value(&mut distance_values),
    }
}

fn comparison_percentile_envelope(
    samples: &[ComparisonSample],
    percentile: f64,
) -> ComparisonPercentileEnvelope {
    let mut longitude_values = samples
        .iter()
        .map(|sample| sample.longitude_delta_deg)
        .collect::<Vec<_>>();
    let mut latitude_values = samples
        .iter()
        .map(|sample| sample.latitude_delta_deg)
        .collect::<Vec<_>>();
    let mut distance_values = samples
        .iter()
        .filter_map(|sample| sample.distance_delta_au)
        .collect::<Vec<_>>();

    ComparisonPercentileEnvelope {
        longitude_delta_deg: percentile_value(&mut longitude_values, percentile)
            .unwrap_or_default(),
        latitude_delta_deg: percentile_value(&mut latitude_values, percentile).unwrap_or_default(),
        distance_delta_au: percentile_value(&mut distance_values, percentile),
    }
}

fn format_comparison_percentile_envelope_for_report(samples: &[ComparisonSample]) -> String {
    match comparison_tail_envelope(samples) {
        Ok(envelope) => envelope.summary_line(),
        Err(error) => format!("comparison percentile envelope unavailable ({error})"),
    }
}

fn format_comparison_envelope_for_report(
    summary: &ComparisonSummary,
    samples: &[ComparisonSample],
) -> String {
    let envelope = comparison_envelope_summary(summary, samples);
    match envelope.validated_summary_line(samples) {
        Ok(rendered) => rendered,
        Err(error) => format!("comparison envelope unavailable ({error})"),
    }
}

fn format_body_class_comparison_envelope_for_report(summary: &BodyClassSummary) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body-class error envelope unavailable ({error})"),
    }
}

fn comparison_body_class_error_envelope_summaries_for_report(
) -> Result<Vec<BodyClassSummary>, String> {
    let report = comparison_report_for_default_render()?;
    let summaries = report.body_class_summaries();

    if summaries.is_empty() {
        return Err("comparison report did not produce any body-class error envelopes".to_string());
    }

    for summary in &summaries {
        summary.validate().map_err(|error| error.to_string())?;
    }

    Ok(summaries)
}

fn comparison_body_class_error_envelope_summary_for_report() -> String {
    match comparison_body_class_error_envelope_summaries_for_report() {
        Ok(summaries) => format!("{} classes checked", summaries.len()),
        Err(error) => format!("body-class error envelopes unavailable ({error})"),
    }
}

fn render_comparison_body_class_error_envelope_summary_text_from_summaries(
    summaries: Result<Vec<BodyClassSummary>, String>,
) -> String {
    use std::fmt::Write as _;

    let summaries = match summaries {
        Ok(summaries) => summaries,
        Err(error) => {
            return format!(
                "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable ({error})\n"
            );
        }
    };

    if summaries.is_empty() {
        return "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable (comparison report did not produce any body-class error envelopes)\n".to_string();
    }

    for summary in &summaries {
        if let Err(error) = summary.validate() {
            return format!(
                "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable ({error})\n"
            );
        }
    }

    let mut text = String::from("Comparison body-class error envelope summary\n");
    let _ = writeln!(text, "Body-class error envelopes: {}", summaries.len());
    for summary in summaries {
        let _ = writeln!(
            text,
            "  {}: {}",
            summary.class.label(),
            summary.summary_line()
        );
    }
    text
}

fn render_comparison_body_class_error_envelope_summary_text() -> String {
    render_comparison_body_class_error_envelope_summary_text_from_summaries(
        comparison_body_class_error_envelope_summaries_for_report(),
    )
}

fn format_body_class_tolerance_envelope_for_report(summary: &BodyClassToleranceSummary) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("body-class tolerance envelope unavailable ({error})"),
    }
}

fn comparison_report_for_default_render() -> Result<ComparisonReport, String> {
    compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &default_corpus(),
    )
    .map_err(|error| error.to_string())
}

fn validated_comparison_body_class_tolerance_posture_line(
    report: &ComparisonReport,
) -> Result<String, String> {
    use std::fmt::Write as _;

    let summaries = report.body_class_tolerance_summaries();
    if summaries.is_empty() {
        return Err(
            "comparison report did not produce any body-class tolerance summaries".to_string(),
        );
    }

    let outlier_class_count = summaries
        .iter()
        .filter(|summary| summary.outside_tolerance_body_count > 0)
        .count();
    let outlier_bodies = summaries
        .iter()
        .flat_map(|summary| summary.outside_bodies.iter().cloned())
        .collect::<Vec<_>>();

    let mut text = String::new();
    let _ = write!(
        text,
        "body-class tolerance posture: {} classes checked, {} classes with outlier bodies, outlier bodies: {}",
        summaries.len(),
        outlier_class_count,
        if outlier_bodies.is_empty() {
            "none".to_string()
        } else {
            format_bodies(&outlier_bodies)
        }
    );
    Ok(text)
}

fn validated_comparison_body_class_tolerance_posture_for_report() -> Result<String, String> {
    let report = comparison_report_for_default_render()?;
    validated_comparison_body_class_tolerance_posture_line(&report)
}

fn format_body_class_tolerance_posture_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();

    SUMMARY
        .get_or_init(|| {
            validated_comparison_body_class_tolerance_posture_for_report().unwrap_or_else(|error| {
                format!("body-class tolerance posture unavailable ({error})")
            })
        })
        .clone()
}

fn validate_comparison_sample_distance_channels(
    samples: &[ComparisonSample],
) -> Result<(), EphemerisError> {
    let has_distance = samples
        .iter()
        .any(|sample| sample.distance_delta_au.is_some());
    let has_missing_distance = samples
        .iter()
        .any(|sample| sample.distance_delta_au.is_none());

    if has_distance && has_missing_distance {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison sample slice must either provide distance deltas for every sample or for none of them",
        ));
    }

    Ok(())
}

fn validate_comparison_samples_for_report(
    samples: &[ComparisonSample],
) -> Result<(), EphemerisError> {
    if samples.is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison sample slice is empty",
        ));
    }

    for (index, sample) in samples.iter().enumerate() {
        for (label, value) in [
            ("longitude_delta_deg", sample.longitude_delta_deg),
            ("latitude_delta_deg", sample.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison sample {} field `{label}` must be a finite non-negative value",
                        index + 1
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = sample.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison sample {} field `distance_delta_au` must be a finite non-negative value",
                        index + 1
                    ),
                ));
            }
        }
    }

    validate_comparison_sample_distance_channels(samples)
}

fn comparison_tolerance_policy_summary_details(
    comparison: &ComparisonReport,
) -> ComparisonTolerancePolicySummary {
    let entries = comparison_tolerance_policy_entries(&comparison.candidate_backend.family);
    let coverage = comparison_tolerance_policy_coverage(comparison);
    let comparison_window = TimeRange::new(
        comparison.corpus_summary.epochs.first().copied(),
        comparison.corpus_summary.epochs.last().copied(),
    );

    ComparisonTolerancePolicySummary {
        backend_family: comparison.candidate_backend.family.clone(),
        entries,
        coverage,
        comparison_body_count: comparison.body_summaries().len(),
        comparison_sample_count: comparison.summary.sample_count,
        comparison_window,
        coordinate_frames: comparison_coordinate_frames(comparison).to_vec(),
    }
}

fn validated_comparison_tolerance_policy_summary_for_report(
    comparison: &ComparisonReport,
) -> Result<ComparisonTolerancePolicySummary, String> {
    let summary = comparison_tolerance_policy_summary_details(comparison);
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

fn format_comparison_tolerance_policy_for_report(comparison: &ComparisonReport) -> String {
    let summary = comparison_tolerance_policy_summary_details(comparison);
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("comparison tolerance policy unavailable ({error})"),
    }
}

pub(crate) fn format_comparison_tolerance_limits_for_report(
    entries: &[ComparisonToleranceEntry],
) -> String {
    entries
        .iter()
        .map(format_comparison_tolerance_limit_for_report)
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_comparison_tolerance_limit_for_report(entry: &ComparisonToleranceEntry) -> String {
    match entry.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("{} unavailable ({error})", entry.scope.label()),
    }
}

fn comparison_coordinate_frames(comparison: &ComparisonReport) -> &[CoordinateFrame] {
    &comparison.candidate_backend.supported_frames
}

/// Renders a release-grade comparison tolerance audit used by the CLI.
pub fn render_comparison_audit_report() -> Result<String, String> {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let comparison =
        compare_backends(&reference, &candidate, &corpus).map_err(|error| error.to_string())?;
    let (_, _, _, regression_count) = comparison_audit_totals(&comparison);
    let rendered = render_comparison_audit_report_text(&comparison);

    if regression_count == 0 {
        Ok(rendered)
    } else {
        Err(format!("comparison audit failed:\n{rendered}"))
    }
}

/// Renders the compact release-grade comparison-audit summary used by the CLI.
pub fn render_comparison_audit_summary() -> Result<String, String> {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let comparison =
        compare_backends(&reference, &candidate, &corpus).map_err(|error| error.to_string())?;

    Ok(comparison_audit_summary_for_report(&comparison))
}

fn comparison_audit_result_label(regression_count: usize) -> &'static str {
    if regression_count == 0 {
        "clean"
    } else {
        "regressions found"
    }
}

fn render_comparison_audit_report_text(report: &ComparisonReport) -> String {
    use std::fmt::Write as _;

    let (body_count, within_tolerance_body_count, outside_tolerance_body_count, regression_count) =
        comparison_audit_totals(report);
    let mut text = String::new();

    let _ = writeln!(text, "Comparison tolerance audit");
    let _ = writeln!(text, "  corpus: {}", report.corpus_name);
    let _ = writeln!(
        text,
        "  reference backend: {} ({})",
        report.reference_backend.id,
        report
            .reference_backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    );
    let _ = writeln!(
        text,
        "  candidate backend: {} ({})",
        report.candidate_backend.id,
        report
            .candidate_backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    );
    let _ = writeln!(text, "  comparison corpus");
    write_corpus_summary_text(&mut text, &report.corpus_summary);
    let _ = writeln!(text, "  bodies checked: {}", body_count);
    let _ = writeln!(
        text,
        "  within tolerance bodies: {}",
        within_tolerance_body_count
    );
    let _ = writeln!(
        text,
        "  outside tolerance bodies: {}",
        outside_tolerance_body_count
    );
    let _ = writeln!(text, "  notable regressions: {}", regression_count);
    let _ = writeln!(
        text,
        "  regression bodies: {}",
        format_regression_bodies(&report.notable_regressions())
    );
    let body_class_tolerance_posture =
        match validated_comparison_body_class_tolerance_posture_line(report) {
            Ok(line) => line,
            Err(error) => format!("body-class tolerance posture unavailable ({error})"),
        };
    let _ = writeln!(text, "  {}", body_class_tolerance_posture);
    let _ = writeln!(
        text,
        "  result: {}",
        comparison_audit_result_label(regression_count)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(text, "  samples: {}", report.summary.sample_count);
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°{}",
        report.summary.max_longitude_delta_deg,
        format_summary_body(&report.summary.max_longitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°{}",
        report.summary.max_latitude_delta_deg,
        format_summary_body(&report.summary.max_latitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}{}",
        report
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string()),
        format_summary_body(&report.summary.max_distance_delta_body)
    );
    let _ = writeln!(
        text,
        "  {}",
        format_comparison_percentile_envelope_for_report(&report.samples)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class error envelopes");
    for summary in report.body_class_summaries() {
        let _ = writeln!(text, "  {}", summary.class.label());
        let _ = writeln!(text, "    samples: {}", summary.sample_count);
        let _ = writeln!(
            text,
            "    max longitude delta: {:.12}°{}",
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default()
        );
        let _ = writeln!(
            text,
            "    mean longitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                summary.sum_longitude_delta_deg / summary.sample_count as f64
            }
        );
        let _ = writeln!(
            text,
            "    median longitude delta: {:.12}°",
            summary.median_longitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms longitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                (summary.sum_longitude_delta_sq_deg / summary.sample_count as f64).sqrt()
            }
        );
        let _ = writeln!(
            text,
            "    max latitude delta: {:.12}°{}",
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default()
        );
        let _ = writeln!(
            text,
            "    mean latitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                summary.sum_latitude_delta_deg / summary.sample_count as f64
            }
        );
        let _ = writeln!(
            text,
            "    median latitude delta: {:.12}°",
            summary.median_latitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms latitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                (summary.sum_latitude_delta_sq_deg / summary.sample_count as f64).sqrt()
            }
        );
        if let Some(value) = summary.max_distance_delta_au {
            let _ = writeln!(text, "    max distance delta: {:.12} AU", value);
        }
        if summary.distance_count > 0 {
            let mean_distance = summary.sum_distance_delta_au / summary.distance_count as f64;
            let median_distance = summary.median_distance_delta_au.unwrap_or(mean_distance);
            let rms_distance =
                (summary.sum_distance_delta_sq_au / summary.distance_count as f64).sqrt();
            let _ = writeln!(text, "    mean distance delta: {:.12} AU", mean_distance);
            let _ = writeln!(
                text,
                "    median distance delta: {:.12} AU",
                median_distance
            );
            let _ = writeln!(text, "    rms distance delta: {:.12} AU", rms_distance);
        }
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class tolerance posture");
    for summary in report.body_class_tolerance_summaries() {
        let _ = writeln!(text, "  {}", summary.class.label());
        let _ = writeln!(text, "    bodies: {}", summary.body_count);
        let _ = writeln!(text, "    samples: {}", summary.sample_count);
        let _ = writeln!(
            text,
            "    within tolerance bodies: {}",
            summary.within_tolerance_body_count
        );
        let _ = writeln!(
            text,
            "    outside tolerance bodies: {}",
            summary.outside_tolerance_body_count
        );
        if !summary.outside_bodies.is_empty() {
            let _ = writeln!(
                text,
                "    outside bodies: {}",
                format_bodies(&summary.outside_bodies)
            );
        }
        let _ = writeln!(
            text,
            "    mean longitude delta: {:.12}°",
            summary.mean_longitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    median longitude delta: {:.12}°",
            summary.median_longitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms longitude delta: {:.12}°",
            summary.rms_longitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    mean latitude delta: {:.12}°",
            summary.mean_latitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    median latitude delta: {:.12}°",
            summary.median_latitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms latitude delta: {:.12}°",
            summary.rms_latitude_delta_deg()
        );
        if let Some(value) = summary.mean_distance_delta_au() {
            let _ = writeln!(text, "    mean distance delta: {:.12} AU", value);
        }
        if let Some(value) = summary.median_distance_delta_au {
            let _ = writeln!(text, "    median distance delta: {:.12} AU", value);
        }
        if let Some(value) = summary.rms_distance_delta_au() {
            let _ = writeln!(text, "    rms distance delta: {:.12} AU", value);
        }
        if let (Some(body), Some(value)) = (
            summary.max_longitude_delta_body.as_ref(),
            summary.max_longitude_delta_deg,
        ) {
            let _ = writeln!(text, "    max longitude delta: {:.12}° ({})", value, body);
        }
        if let (Some(body), Some(value)) = (
            summary.max_latitude_delta_body.as_ref(),
            summary.max_latitude_delta_deg,
        ) {
            let _ = writeln!(text, "    max latitude delta: {:.12}° ({})", value, body);
        }
        if let (Some(body), Some(value)) = (
            summary.max_distance_delta_body.as_ref(),
            summary.max_distance_delta_au,
        ) {
            let _ = writeln!(text, "    max distance delta: {:.12} AU ({})", value, body);
        }
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Tolerance policy");
    write_tolerance_policy_text(&mut text, report);
    let _ = writeln!(text);
    let _ = writeln!(text, "Notable regressions");
    let regressions = report.notable_regressions();
    if regressions.is_empty() {
        let _ = writeln!(text, "  none");
    } else {
        for finding in regressions {
            let _ = writeln!(
                text,
                "  {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
                finding.body,
                finding.longitude_delta_deg,
                finding.latitude_delta_deg,
                finding
                    .distance_delta_au
                    .map(|value| format!("{value:.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string()),
                finding.note
            );
        }
    }

    text
}

/// Renders a benchmark report used by the CLI.
pub fn render_benchmark_report(rounds: usize) -> Result<String, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("benchmark report cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = render_benchmark_report_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

fn render_benchmark_report_uncached(rounds: usize) -> Result<String, EphemerisError> {
    let corpus = benchmark_timing_corpus();
    let candidate = default_candidate_backend();
    let backend_report = benchmark_backend(&candidate, &corpus, rounds)?;
    let artifact_lookup_report =
        artifact::benchmark_packaged_artifact_lookup(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let artifact_decode_report =
        artifact::benchmark_packaged_artifact_decode(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let chart_report = benchmark_chart_backend(default_candidate_backend(), rounds)?;
    Ok(format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}",
        benchmark_provenance_text(),
        backend_report,
        artifact_lookup_report,
        artifact_decode_report,
        chart_report
    ))
}

/// Renders a compact benchmark matrix summary used by the CLI.
pub fn render_benchmark_matrix_summary(rounds: usize) -> Result<String, EphemerisError> {
    let report = build_validation_report(rounds)?;
    Ok(render_benchmark_matrix_summary_text(&report))
}

fn report_summary_payload(summary: String, prefix: &str) -> String {
    summary
        .strip_prefix(prefix)
        .unwrap_or(summary.as_str())
        .to_string()
}

fn render_benchmark_matrix_summary_text(report: &ValidationReport) -> String {
    use std::fmt::Write as _;

    let mut text = String::from("Benchmark matrix summary\n");
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text, "Benchmark corpora");
    let _ = writeln!(
        text,
        "  comparison corpus: {}",
        report.comparison_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  benchmark corpus: {}",
        report.benchmark_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-data benchmark corpus: {}",
        report.packaged_benchmark_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  chart benchmark corpus: {}",
        report.chart_benchmark_corpus.summary_line()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark rows");
    let _ = writeln!(
        text,
        "  reference benchmark: {}",
        report.reference_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  candidate benchmark: {}",
        report.candidate_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-data benchmark: {}",
        report.packaged_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  chart benchmark: {}",
        report.chart_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  artifact decode benchmark: {}",
        report.artifact_decode_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-artifact size: {} bytes",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let fit_envelope_summary = packaged_artifact_fit_envelope_summary_for_report();
    let fit_sample_classes_summary = packaged_artifact_fit_sample_classes_summary_for_report();
    let fit_outlier_summary = packaged_artifact_fit_outlier_summary_for_report();
    let fit_thresholds_summary = packaged_artifact_fit_threshold_summary_for_report();
    let target_threshold_summary =
        validated_packaged_artifact_target_threshold_summary_for_report();
    let target_threshold_state_summary =
        validated_packaged_artifact_target_threshold_state_for_report();
    let target_threshold_scope_envelopes_summary =
        validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report();
    let fit_margin_summary = report_summary_payload(
        packaged_artifact_fit_margin_summary_for_report(),
        "fit margins: ",
    );
    let fit_threshold_violation_count_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_count_for_report(),
        "fit threshold violations: ",
    );
    let fit_threshold_violation_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_summary_for_report(),
        "fit threshold violations: ",
    );
    let fit_envelope = fit_envelope_summary
        .strip_prefix("fit envelope: ")
        .unwrap_or(&fit_envelope_summary);
    let fit_sample_classes = fit_sample_classes_summary
        .strip_prefix("fit sample classes: ")
        .unwrap_or(&fit_sample_classes_summary);
    let fit_outliers = fit_outlier_summary
        .strip_prefix("fit outliers: ")
        .unwrap_or(&fit_outlier_summary);
    let fit_thresholds = fit_thresholds_summary
        .strip_prefix("fit thresholds: ")
        .unwrap_or(&fit_thresholds_summary);
    let target_threshold_scope_envelopes = target_threshold_scope_envelopes_summary
        .strip_prefix("scope envelopes: ")
        .unwrap_or(&target_threshold_scope_envelopes_summary);

    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-artifact fit posture");
    let _ = writeln!(text, "  fit envelope: {}", fit_envelope);
    let _ = writeln!(text, "  fit margins: {}", fit_margin_summary);
    let _ = writeln!(
        text,
        "  fit threshold violation count: {}",
        fit_threshold_violation_count_summary
    );
    let _ = writeln!(
        text,
        "  fit threshold violations: {}",
        fit_threshold_violation_summary
    );
    let _ = writeln!(text, "  fit sample classes: {}", fit_sample_classes);
    let _ = writeln!(text, "  fit outliers: {}", fit_outliers);
    let _ = writeln!(text, "  fit thresholds: {}", fit_thresholds);
    let _ = writeln!(text, "  target thresholds: {}", target_threshold_summary);
    let _ = writeln!(
        text,
        "  target-threshold state: {}",
        target_threshold_state_summary
    );
    let _ = writeln!(
        text,
        "  target-threshold scope envelopes: {}",
        target_threshold_scope_envelopes
    );
    text
}

fn vsop87_canonical_body_evidence() -> Option<Vec<pleiades_vsop87::Vsop87CanonicalBodyEvidence>> {
    pleiades_vsop87::canonical_epoch_body_evidence()
}

fn format_vsop87_canonical_evidence_summary() -> String {
    canonical_epoch_evidence_summary_for_report()
}

fn format_vsop87_equatorial_evidence_summary() -> String {
    canonical_epoch_equatorial_evidence_summary_for_report()
}

fn format_vsop87_j2000_batch_summary() -> String {
    canonical_j2000_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j2000_ecliptic_batch_summary() -> String {
    supported_body_j2000_ecliptic_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j2000_equatorial_batch_summary() -> String {
    supported_body_j2000_equatorial_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j1900_ecliptic_batch_summary() -> String {
    supported_body_j1900_ecliptic_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j1900_equatorial_batch_summary() -> String {
    supported_body_j1900_equatorial_batch_parity_summary_for_report()
}

fn format_vsop87_mixed_batch_summary() -> String {
    canonical_mixed_time_scale_batch_parity_summary_for_report()
}

fn format_vsop87_j1900_batch_summary() -> String {
    canonical_j1900_batch_parity_summary_for_report()
}

fn format_vsop87_body_evidence_summary() -> String {
    source_body_evidence_summary_for_report()
}

fn format_vsop87_source_body_class_evidence_summary() -> String {
    source_body_class_evidence_summary_for_report()
}

fn format_vsop87_equatorial_body_class_evidence_summary() -> String {
    canonical_epoch_equatorial_body_class_evidence_summary_for_report()
}

fn format_vsop87_canonical_outlier_note_summary() -> String {
    canonical_epoch_outlier_note_for_report()
}

fn format_validated_vsop87_source_documentation_summary_for_report(
    summary: &pleiades_vsop87::Vsop87SourceDocumentationSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("VSOP87 source documentation: unavailable ({error})"),
    }
}

fn format_validated_vsop87_source_documentation_health_summary_for_report(
    summary: &pleiades_vsop87::Vsop87SourceDocumentationHealthSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source documentation health: unavailable ({error})"),
    }
}

fn format_vsop87_source_documentation_summary() -> String {
    format_validated_vsop87_source_documentation_summary_for_report(&source_documentation_summary())
}

fn format_vsop87_source_documentation_health_summary() -> String {
    format_validated_vsop87_source_documentation_health_summary_for_report(
        &source_documentation_health_summary(),
    )
}

fn format_vsop87_frame_treatment_summary() -> String {
    frame_treatment_summary_for_report()
}

fn format_jpl_frame_treatment_summary() -> String {
    jpl_frame_treatment_summary_for_report()
}

/// Compact validation evidence for the shared mean-obliquity frame round-trip samples.
#[derive(Clone, Debug, PartialEq)]
pub struct MeanObliquityFrameRoundTripSummary {
    sample_count: usize,
    max_longitude_delta_deg: f64,
    max_latitude_delta_deg: f64,
    max_distance_delta_au: f64,
    mean_longitude_delta_deg: f64,
    mean_latitude_delta_deg: f64,
    mean_distance_delta_au: f64,
    percentile_longitude_delta_deg: f64,
    percentile_latitude_delta_deg: f64,
    percentile_distance_delta_au: f64,
}

impl MeanObliquityFrameRoundTripSummary {
    /// Validates the stored round-trip envelope.
    pub fn validate(&self) -> Result<(), String> {
        if self.sample_count == 0 {
            return Err("mean-obliquity frame round-trip summary has no samples".to_string());
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "mean-obliquity frame round-trip summary field `{label}` must be a finite non-negative value"
                ));
            }
        }

        let expected = expected_mean_obliquity_frame_round_trip_summary()?;
        if *self != expected {
            return Err(
                "mean-obliquity frame round-trip summary drifted from the canonical sample set"
                    .to_string(),
            );
        }

        Ok(())
    }

    fn summary_line(&self) -> String {
        format!(
            "{} samples, max |Δlon|={:.12}°, mean |Δlon|={:.12}°, p95 |Δlon|={:.12}°, max |Δlat|={:.12}°, mean |Δlat|={:.12}°, p95 |Δlat|={:.12}°, max |Δdist|={:.12} AU, mean |Δdist|={:.12} AU, p95 |Δdist|={:.12} AU",
            self.sample_count,
            self.max_longitude_delta_deg,
            self.mean_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.mean_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            self.max_distance_delta_au,
            self.mean_distance_delta_au,
            self.percentile_distance_delta_au,
        )
    }

    /// Returns the compact round-trip summary line after validating the canonical sample set.
    pub fn validated_summary_line(&self) -> Result<String, String> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for MeanObliquityFrameRoundTripSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the canonical sample corpus used to validate the shared mean-obliquity frame round-trip envelope.
///
/// Downstream tooling can reuse this exact input set instead of reconstructing it from report text.
/// The corpus intentionally covers a near-polar wraparound case so the report evidence exercises the
/// same precision edge that the frame regression tests pin.
pub fn mean_obliquity_frame_round_trip_sample_corpus() -> [(EclipticCoordinates, Instant); 7] {
    [
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(123.45),
                pleiades_core::Latitude::from_degrees(-6.75),
                Some(0.123),
            ),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(90.0),
                pleiades_core::Latitude::from_degrees(0.0),
                Some(1.0),
            ),
            Instant::new(JulianDay::from_days(2_459_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(27.5),
                pleiades_core::Latitude::from_degrees(-33.25),
                Some(2.5),
            ),
            Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(315.0),
                pleiades_core::Latitude::from_degrees(18.0),
                Some(4.25),
            ),
            Instant::new(JulianDay::from_days(2_440_587.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(5.0),
                pleiades_core::Latitude::from_degrees(66.0),
                Some(0.75),
            ),
            Instant::new(JulianDay::from_days(2_500_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(359.875),
                pleiades_core::Latitude::from_degrees(89.25),
                Some(0.5),
            ),
            Instant::new(JulianDay::from_days(2_450_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(180.0),
                pleiades_core::Latitude::from_degrees(-89.25),
                Some(0.5),
            ),
            Instant::new(JulianDay::from_days(2_450_000.5), TimeScale::Tt),
        ),
    ]
}

fn validate_mean_obliquity_frame_round_trip_sample_corpus(
    samples: &[(EclipticCoordinates, Instant)],
) -> Result<(), String> {
    if samples.len() != 7 {
        return Err(format!(
            "mean-obliquity frame round-trip sample corpus must contain 7 samples, found {}",
            samples.len()
        ));
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() > 80.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a northern polar sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() < -80.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a southern polar sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.longitude.degrees() > 350.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a wraparound longitude sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees().abs() < 1e-12)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include an equatorial sample"
                .to_string(),
        );
    }

    Ok(())
}

fn mean_obliquity_frame_round_trip_summary_from_samples(
    samples: &[(EclipticCoordinates, Instant)],
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    validate_mean_obliquity_frame_round_trip_sample_corpus(samples)?;

    let mut sample_count = 0usize;
    let mut max_longitude_delta_deg: f64 = 0.0;
    let mut max_latitude_delta_deg: f64 = 0.0;
    let mut max_distance_delta_au: f64 = 0.0;
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut distance_deltas = Vec::with_capacity(samples.len());

    for (ecliptic, instant) in samples.iter().copied() {
        let obliquity = instant.mean_obliquity();
        let round_trip = ecliptic.to_equatorial(obliquity).to_ecliptic(obliquity);
        let longitude_delta_deg =
            (round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs();
        let latitude_delta_deg =
            (round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs();
        let distance_delta_au = (round_trip.distance_au.unwrap_or_default()
            - ecliptic.distance_au.unwrap_or_default())
        .abs();

        if !longitude_delta_deg.is_finite()
            || !latitude_delta_deg.is_finite()
            || !distance_delta_au.is_finite()
        {
            return Err("non-finite round-trip delta".to_string());
        }

        max_longitude_delta_deg = max_longitude_delta_deg.max(longitude_delta_deg);
        max_latitude_delta_deg = max_latitude_delta_deg.max(latitude_delta_deg);
        max_distance_delta_au = max_distance_delta_au.max(distance_delta_au);
        longitude_deltas.push(longitude_delta_deg);
        latitude_deltas.push(latitude_delta_deg);
        distance_deltas.push(distance_delta_au);
        sample_count += 1;
    }

    Ok(MeanObliquityFrameRoundTripSummary {
        sample_count,
        max_longitude_delta_deg,
        max_latitude_delta_deg,
        max_distance_delta_au,
        mean_longitude_delta_deg: arithmetic_mean(&longitude_deltas),
        mean_latitude_delta_deg: arithmetic_mean(&latitude_deltas),
        mean_distance_delta_au: arithmetic_mean(&distance_deltas),
        percentile_longitude_delta_deg: percentile_linear_interpolation(&longitude_deltas, 0.95),
        percentile_latitude_delta_deg: percentile_linear_interpolation(&latitude_deltas, 0.95),
        percentile_distance_delta_au: percentile_linear_interpolation(&distance_deltas, 0.95),
    })
}

fn expected_mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )
}

fn arithmetic_mean(values: &[f64]) -> f64 {
    values.iter().copied().sum::<f64>() / values.len() as f64
}

fn percentile_linear_interpolation(values: &[f64], percentile: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    if sorted.len() == 1 {
        return sorted[0];
    }

    let clamped = percentile.clamp(0.0, 1.0);
    let position = clamped * (sorted.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;

    if lower_index == upper_index {
        sorted[lower_index]
    } else {
        let lower_value = sorted[lower_index];
        let upper_value = sorted[upper_index];
        let fraction = position - lower_index as f64;
        lower_value + (upper_value - lower_value) * fraction
    }
}

/// Computes the shared mean-obliquity frame round-trip validation summary.
pub fn mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    let summary = mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )?;
    summary.validate()?;
    Ok(summary)
}

fn format_mean_obliquity_frame_round_trip_summary_for_report(
    summary: &MeanObliquityFrameRoundTripSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}

fn mean_obliquity_frame_round_trip_summary_for_report() -> String {
    match mean_obliquity_frame_round_trip_summary() {
        Ok(summary) => format_mean_obliquity_frame_round_trip_summary_for_report(&summary),
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}

fn format_time_scale_policy_summary_for_report(
    summary: &pleiades_backend::TimeScalePolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("time-scale policy unavailable ({error})"),
    }
}

fn format_delta_t_policy_summary_for_report(
    summary: &pleiades_backend::DeltaTPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("delta T policy unavailable ({error})"),
    }
}

fn format_observer_policy_summary_for_report(
    summary: &pleiades_backend::ObserverPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("observer policy unavailable ({error})"),
    }
}

fn format_apparentness_policy_summary_for_report(
    summary: &pleiades_backend::ApparentnessPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("apparentness policy unavailable ({error})"),
    }
}

fn format_request_policy_summary_for_report(
    summary: &pleiades_backend::RequestPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("request policy unavailable ({error})"),
    }
}

fn validated_request_policy_summary_for_report(
) -> Result<pleiades_backend::RequestPolicySummary, String> {
    let summary = request_policy_summary_for_report();
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

fn validated_production_generation_body_class_coverage_summary_for_report() -> String {
    match validated_production_generation_snapshot_body_class_coverage_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Production generation body-class coverage unavailable ({error})"),
    }
}

fn format_request_semantics_summary_for_report(
    time_scale_policy: &pleiades_backend::TimeScalePolicySummary,
) -> String {
    use std::fmt::Write as _;

    let mut text = String::new();
    let _ = writeln!(
        text,
        "Time-scale policy: {}",
        format_time_scale_policy_summary_for_report(time_scale_policy)
    );

    let utc_convenience_policy =
        pleiades_backend::validated_utc_convenience_policy_summary_for_report();
    let _ = writeln!(text, "UTC convenience policy: {}", utc_convenience_policy);

    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );

    let native_sidereal_policy =
        pleiades_backend::validated_native_sidereal_policy_summary_for_report();
    let _ = writeln!(text, "Native sidereal policy: {}", native_sidereal_policy);

    let request_policy = match validated_request_policy_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => {
            let _ = writeln!(text, "Observer policy unavailable ({error})");
            let _ = writeln!(text, "Apparentness policy unavailable ({error})");
            let _ = writeln!(text, "Request policy unavailable ({error})");
            return text;
        }
    };

    let observer_policy = pleiades_backend::observer_policy_summary_for_report();
    let apparentness_policy = pleiades_backend::apparentness_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Observer policy: {}",
        format_observer_policy_summary_for_report(&observer_policy)
    );
    let _ = writeln!(
        text,
        "Apparentness policy: {}",
        format_apparentness_policy_summary_for_report(&apparentness_policy)
    );
    let _ = writeln!(
        text,
        "Request policy: {}",
        format_request_policy_summary_for_report(&request_policy)
    );
    text
}

fn render_time_scale_policy_summary_text() -> String {
    match time_scale_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!(
            "Time-scale policy summary\nTime-scale policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Time-scale policy summary\nTime-scale policy unavailable ({error})\n")
        }
    }
}

fn render_delta_t_policy_summary_text() -> String {
    match delta_t_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!("Delta T policy summary\nDelta T policy: {}\n", summary),
        Err(error) => format!("Delta T policy summary\nDelta T policy unavailable ({error})\n"),
    }
}

fn render_zodiac_policy_summary_text() -> String {
    format!(
        "Zodiac policy summary\nZodiac policy: {}\n",
        pleiades_backend::validated_zodiac_policy_summary_for_report()
    )
}

fn render_utc_convenience_policy_summary_text() -> String {
    format!(
        "UTC convenience policy summary\nUTC convenience policy: {}\n",
        pleiades_backend::validated_utc_convenience_policy_summary_for_report()
    )
}

fn render_observer_policy_summary_text() -> String {
    match pleiades_backend::observer_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!("Observer policy summary\nObserver policy: {}\n", summary),
        Err(error) => format!("Observer policy summary\nObserver policy unavailable ({error})\n"),
    }
}

fn render_apparentness_policy_summary_text() -> String {
    match pleiades_backend::apparentness_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!(
            "Apparentness policy summary\nApparentness policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Apparentness policy summary\nApparentness policy unavailable ({error})\n")
        }
    }
}

fn render_native_sidereal_policy_summary_text() -> String {
    format!(
        "Native sidereal policy summary\nNative sidereal policy: {}\n",
        pleiades_backend::validated_native_sidereal_policy_summary_for_report()
    )
}

fn render_interpolation_posture_summary_text() -> String {
    match jpl_interpolation_posture_summary() {
        Some(summary) => {
            match summary.validated_summary_line() {
                Ok(summary) => format!(
                    "Interpolation posture summary\nInterpolation posture: {}\n",
                    summary
                ),
                Err(error) => {
                    format!("Interpolation posture summary\nInterpolation posture unavailable ({error})\n")
                }
            }
        }
        None => "Interpolation posture summary\nInterpolation posture unavailable\n".to_string(),
    }
}

fn render_interpolation_quality_summary_text() -> String {
    format!(
        "Interpolation quality summary\n{}\n",
        format_jpl_interpolation_quality_summary_for_report()
    )
}

fn render_comparison_snapshot_summary_text() -> String {
    format!(
        "Comparison snapshot summary\n{}\n",
        comparison_snapshot_summary_for_report()
    )
}

fn comparison_corpus_release_guard_summary() -> &'static str {
    "Pluto excluded from tolerance evidence"
}

fn validated_comparison_corpus_release_guard_summary_for_report() -> Result<&'static str, String> {
    const EXPECTED: &str = "Pluto excluded from tolerance evidence";
    let summary = comparison_corpus_release_guard_summary();

    if summary == EXPECTED {
        Ok(summary)
    } else {
        Err(format!(
            "comparison corpus release-grade guard mismatch: expected {EXPECTED}, found {summary}"
        ))
    }
}

fn render_comparison_corpus_summary_text() -> String {
    use std::fmt::Write as _;

    let corpus = release_grade_corpus();
    let summary = corpus.summary();
    let mut text = String::from("Comparison corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => return format!("Comparison corpus summary unavailable ({error})"),
    };
    let _ = writeln!(text, "  release-grade guard: {release_grade_guard}");
    text.push('\n');
    text
}

fn ensure_comparison_corpus_summary_matches_current_rendering(
    comparison_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_corpus_summary_text != render_comparison_corpus_summary_text() {
        return Err(ReleaseBundleError::Verification(
            "comparison corpus summary no longer matches the current comparison-corpus posture"
                .to_string(),
        ));
    }

    Ok(())
}

fn required_summary_payload(
    summary: String,
    prefix: &str,
    field: &'static str,
) -> Result<String, String> {
    summary
        .strip_prefix(prefix)
        .map(str::to_string)
        .ok_or_else(|| {
            format!("source corpus summary field `{field}` is out of sync with the current posture")
        })
}

fn required_labelled_summary_payload(
    summary: String,
    prefix: &str,
    field: &'static str,
) -> Result<String, String> {
    let payload = required_summary_payload(summary, prefix, field)?;
    if payload.starts_with(prefix) {
        return Err(format!(
            "source corpus summary field `{field}` is out of sync with the current posture"
        ));
    }

    Ok(payload)
}

#[derive(Clone, Debug, PartialEq)]
struct SourceCorpusSummary {
    comparison_corpus_release_grade_guard: String,
    jpl_source_corpus_contract: String,
    jpl_evidence_classification: String,
    jpl_provenance_only: String,
    lunar_source_window: String,
    shared_schema: String,
    generation_command: String,
    production_generation_source: String,
    production_generation_source_revision: String,
    production_generation_coverage: String,
    production_generation_source_windows: String,
    production_generation_body_class_coverage: String,
    production_generation_date_range: String,
    production_generation_quarter_day_boundary_samples: String,
    coverage_posture: String,
    production_generation_boundary_window: String,
    production_generation_boundary_source: String,
    production_generation_boundary_request_corpus: String,
    production_generation_boundary_request_corpus_equatorial: String,
    reference_snapshot_sparse_boundary: String,
    reference_snapshot_exact_j2000_evidence: String,
    reference_snapshot_exact_j2000_body_class_coverage: String,
    reference_snapshot_equatorial_parity: String,
    reference_snapshot_body_class_coverage: String,
    reference_snapshot_manifest: String,
    comparison_snapshot_manifest: String,
    independent_holdout_body_class_coverage: String,
    independent_holdout_source_window: String,
    pluto_fallback: String,
    release_grade_body_claims: String,
    body_date_channel_claims: String,
    phase2_corpus_alignment: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SourceCorpusSummaryValidationError {
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for SourceCorpusSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the source corpus summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for SourceCorpusSummaryValidationError {}

impl SourceCorpusSummary {
    fn summary_line(&self) -> String {
        format!(
            "comparison corpus release-grade guard: {}; JPL source corpus contract: {}; evidence classification={}; provenance-only={}; lunar source windows={}; shared schema={}; generation command={}; production generation source={}; production generation source revision={}; production generation coverage={}; production generation source windows={}; production generation body-class coverage={}; production generation date range={}; production generation quarter-day boundary samples={}; coverage posture={}; production generation boundary window={}; production generation boundary source={}; production generation boundary request corpus={}; production generation boundary request corpus equatorial={}; reference snapshot sparse boundary={}; reference snapshot exact J2000 evidence={}; reference snapshot exact J2000 body-class coverage={}; reference snapshot equatorial parity={}; reference snapshot body-class coverage={}; reference snapshot manifest={}; comparison snapshot manifest={}; independent-holdout body-class coverage={}; independent-holdout source window={}; pluto fallback={}; release-grade body claims={}; body-date-channel claims={}; phase-2 corpus alignment: {}",
            self.comparison_corpus_release_grade_guard,
            self.jpl_source_corpus_contract,
            self.jpl_evidence_classification,
            self.jpl_provenance_only,
            self.lunar_source_window,
            self.shared_schema,
            self.generation_command,
            self.production_generation_source,
            self.production_generation_source_revision,
            self.production_generation_coverage,
            self.production_generation_source_windows,
            self.production_generation_body_class_coverage,
            self.production_generation_date_range,
            self.production_generation_quarter_day_boundary_samples,
            self.coverage_posture,
            self.production_generation_boundary_window,
            self.production_generation_boundary_source,
            self.production_generation_boundary_request_corpus,
            self.production_generation_boundary_request_corpus_equatorial,
            self.reference_snapshot_sparse_boundary,
            self.reference_snapshot_exact_j2000_evidence,
            self.reference_snapshot_exact_j2000_body_class_coverage,
            self.reference_snapshot_equatorial_parity,
            self.reference_snapshot_body_class_coverage,
            self.reference_snapshot_manifest,
            self.comparison_snapshot_manifest,
            self.independent_holdout_body_class_coverage,
            self.independent_holdout_source_window,
            self.pluto_fallback,
            self.release_grade_body_claims,
            self.body_date_channel_claims,
            self.phase2_corpus_alignment,
        )
    }

    fn validate(&self) -> Result<(), SourceCorpusSummaryValidationError> {
        let expected = source_corpus_summary_details().ok_or(
            SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "source_corpus_summary",
            },
        )?;

        if self.comparison_corpus_release_grade_guard
            != expected.comparison_corpus_release_grade_guard
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "comparison_corpus_release_grade_guard",
            });
        }
        if self.jpl_source_corpus_contract != expected.jpl_source_corpus_contract {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_source_corpus_contract",
            });
        }
        if self.jpl_evidence_classification != expected.jpl_evidence_classification {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_evidence_classification",
            });
        }
        if self.jpl_provenance_only != expected.jpl_provenance_only {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_provenance_only",
            });
        }
        if self.lunar_source_window != expected.lunar_source_window {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "lunar_source_window",
            });
        }
        if self.shared_schema != expected.shared_schema {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "shared_schema",
            });
        }
        if self.generation_command != expected.generation_command {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "generation_command",
            });
        }
        if self.production_generation_source != expected.production_generation_source {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source",
            });
        }
        if self.production_generation_source_revision
            != expected.production_generation_source_revision
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source_revision",
            });
        }
        if self.production_generation_coverage != expected.production_generation_coverage {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_coverage",
            });
        }
        if self.production_generation_source_windows
            != expected.production_generation_source_windows
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source_windows",
            });
        }
        if self.production_generation_body_class_coverage
            != expected.production_generation_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_body_class_coverage",
            });
        }
        if self.production_generation_date_range != expected.production_generation_date_range {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_date_range",
            });
        }
        if self.production_generation_quarter_day_boundary_samples
            != expected.production_generation_quarter_day_boundary_samples
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_quarter_day_boundary_samples",
            });
        }
        if self.coverage_posture != expected.coverage_posture {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "coverage_posture",
            });
        }
        if self.production_generation_boundary_window
            != expected.production_generation_boundary_window
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_window",
            });
        }
        if self.production_generation_boundary_source
            != expected.production_generation_boundary_source
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_source",
            });
        }
        if self.production_generation_boundary_request_corpus
            != expected.production_generation_boundary_request_corpus
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_request_corpus",
            });
        }
        if self.production_generation_boundary_request_corpus_equatorial
            != expected.production_generation_boundary_request_corpus_equatorial
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_request_corpus_equatorial",
            });
        }
        if self.reference_snapshot_sparse_boundary != expected.reference_snapshot_sparse_boundary {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_sparse_boundary",
            });
        }
        if self.reference_snapshot_exact_j2000_evidence
            != expected.reference_snapshot_exact_j2000_evidence
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_exact_j2000_evidence",
            });
        }
        if self.reference_snapshot_exact_j2000_body_class_coverage
            != expected.reference_snapshot_exact_j2000_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_exact_j2000_body_class_coverage",
            });
        }
        if self.reference_snapshot_equatorial_parity
            != expected.reference_snapshot_equatorial_parity
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_equatorial_parity",
            });
        }
        if self.reference_snapshot_body_class_coverage
            != expected.reference_snapshot_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_body_class_coverage",
            });
        }
        if self.reference_snapshot_manifest != expected.reference_snapshot_manifest {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_manifest",
            });
        }
        if self.independent_holdout_body_class_coverage
            != expected.independent_holdout_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "independent_holdout_body_class_coverage",
            });
        }
        if self.independent_holdout_source_window != expected.independent_holdout_source_window {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "independent_holdout_source_window",
            });
        }
        if self.pluto_fallback != expected.pluto_fallback {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "pluto_fallback",
            });
        }
        if self.release_grade_body_claims != expected.release_grade_body_claims {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "release_grade_body_claims",
            });
        }
        if self.body_date_channel_claims != expected.body_date_channel_claims {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "body_date_channel_claims",
            });
        }
        if self.phase2_corpus_alignment != expected.phase2_corpus_alignment {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "phase2_corpus_alignment",
            });
        }

        Ok(())
    }

    fn validated_summary_line(&self) -> Result<String, SourceCorpusSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

fn source_corpus_summary_details() -> Option<SourceCorpusSummary> {
    let comparison_corpus_release_grade_guard =
        validated_comparison_corpus_release_guard_summary_for_report()
            .ok()?
            .to_string();
    let jpl_source_corpus_contract = required_labelled_summary_payload(
        jpl_source_corpus_contract_summary_for_report(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    )
    .ok()?;
    let jpl_evidence_classification = required_labelled_summary_payload(
        jpl_snapshot_evidence_classification_summary_for_report(),
        "JPL evidence classification: ",
        "JPL evidence classification",
    )
    .ok()?;
    let jpl_provenance_only = required_labelled_summary_payload(
        jpl_provenance_only_summary_for_report(),
        "JPL provenance-only evidence: ",
        "JPL provenance-only evidence",
    )
    .ok()?;
    let release_grade_body_claims = validated_release_body_claims_summary_line_for_report()
        .ok()?
        .to_string();
    let lunar_source_window = required_summary_payload(
        lunar_source_window_summary_for_report(),
        "lunar source windows: ",
        "lunar source window",
    )
    .ok()?;
    let reference_snapshot_sparse_boundary = required_summary_payload(
        reference_snapshot_sparse_boundary_summary_for_report(),
        "Reference snapshot boundary day: ",
        "reference snapshot sparse boundary",
    )
    .ok()?;
    let reference_snapshot_exact_j2000_evidence = required_summary_payload(
        reference_snapshot_exact_j2000_evidence_summary_for_report(),
        "Reference snapshot exact J2000 evidence: ",
        "reference snapshot exact J2000 evidence",
    )
    .ok()?;
    let reference_snapshot_exact_j2000_body_class_coverage = required_summary_payload(
        pleiades_jpl::reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(),
        "Reference snapshot exact J2000 body-class coverage: ",
        "reference snapshot exact J2000 body-class coverage",
    )
    .ok()?;
    let reference_snapshot_equatorial_parity = required_summary_payload(
        reference_snapshot_equatorial_parity_summary_for_report(),
        "JPL reference snapshot equatorial parity: ",
        "reference snapshot equatorial parity",
    )
    .ok()?;
    let reference_snapshot_body_class_coverage = required_summary_payload(
        reference_snapshot_body_class_coverage_summary_for_report(),
        "Reference snapshot body-class coverage: ",
        "reference snapshot body-class coverage",
    )
    .ok()?;
    let reference_snapshot_manifest = required_summary_payload(
        reference_snapshot_manifest_summary_for_report(),
        "Reference snapshot manifest: ",
        "reference snapshot manifest",
    )
    .ok()?;
    let comparison_snapshot_manifest = required_summary_payload(
        validated_comparison_snapshot_manifest_summary_for_report().ok()?,
        "Comparison snapshot manifest: ",
        "comparison snapshot manifest",
    )
    .ok()?;
    let independent_holdout_body_class_coverage = required_summary_payload(
        independent_holdout_snapshot_body_class_coverage_summary_for_report(),
        "Independent hold-out body-class coverage: ",
        "independent-holdout body-class coverage",
    )
    .ok()?;
    let independent_holdout_source_window = required_summary_payload(
        independent_holdout_snapshot_source_window_summary_for_report(),
        "Independent hold-out source windows: ",
        "independent-holdout source window",
    )
    .ok()?;
    let phase2_corpus_alignment =
        validated_packaged_artifact_phase2_corpus_alignment_summary_for_report();
    let pluto_fallback = required_summary_payload(
        format!(
            "Pluto fallback: {}",
            validated_pluto_fallback_summary_line_for_report().ok()?
        ),
        "Pluto fallback: ",
        "pluto fallback",
    )
    .ok()?;
    let production_generation_date_range = production_generation_date_range_for_report()?;
    let production_generation_quarter_day_boundary_samples = required_summary_payload(
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report(),
        "Production generation quarter-day boundary samples: ",
        "production generation quarter-day boundary samples",
    )
    .ok()?;

    Some(SourceCorpusSummary {
        comparison_corpus_release_grade_guard,
        jpl_source_corpus_contract,
        jpl_evidence_classification,
        jpl_provenance_only,
        lunar_source_window,
        shared_schema: validated_checked_in_snapshot_schema_summary_for_report().ok()?,
        generation_command: "generate-packaged-artifact --check".to_string(),
        production_generation_source: required_summary_payload(
            validated_production_generation_source_summary_for_report().ok()?,
            "Production generation source: ",
            "production generation source",
        )
        .ok()?,
        production_generation_source_revision:
            validated_production_generation_source_revision_summary_for_report().ok()?,
        production_generation_coverage: required_summary_payload(
            production_generation_snapshot_summary_for_report(),
            "Production generation coverage: ",
            "production generation coverage",
        )
        .ok()?,
        production_generation_source_windows: required_summary_payload(
            production_generation_snapshot_window_summary_for_report(),
            "Production generation source windows: ",
            "production generation source windows",
        )
        .ok()?,
        production_generation_body_class_coverage: required_summary_payload(
            pleiades_jpl::production_generation_snapshot_body_class_coverage_summary_for_report(),
            "Production generation body-class coverage: ",
            "production generation body-class coverage",
        )
        .ok()?,
        production_generation_date_range,
        production_generation_quarter_day_boundary_samples,
        coverage_posture: production_generation_coverage_posture_for_report()?,
        production_generation_boundary_window: required_summary_payload(
            production_generation_boundary_window_summary_for_report(),
            "Production generation boundary windows: ",
            "production generation boundary window",
        )
        .ok()?,
        production_generation_boundary_source: required_summary_payload(
            production_generation_boundary_source_summary_for_report(),
            "Production generation boundary overlay source: ",
            "production generation boundary source",
        )
        .ok()?,
        production_generation_boundary_request_corpus: required_summary_payload(
            production_generation_boundary_request_corpus_summary_for_report(),
            "Production generation boundary request corpus: ",
            "production generation boundary request corpus",
        )
        .ok()?,
        production_generation_boundary_request_corpus_equatorial: required_summary_payload(
            production_generation_boundary_request_corpus_equatorial_summary_for_report(),
            "Production generation boundary request corpus: ",
            "production generation boundary request corpus equatorial",
        )
        .ok()?,
        reference_snapshot_sparse_boundary,
        reference_snapshot_exact_j2000_evidence,
        reference_snapshot_exact_j2000_body_class_coverage,
        reference_snapshot_equatorial_parity,
        reference_snapshot_body_class_coverage,
        reference_snapshot_manifest,
        comparison_snapshot_manifest,
        independent_holdout_body_class_coverage,
        independent_holdout_source_window,
        pluto_fallback,
        release_grade_body_claims,
        body_date_channel_claims: body_date_channel_claims_summary_details()?
            .validated_summary_line()
            .ok()?,
        phase2_corpus_alignment,
    })
}

fn validated_source_corpus_summary_for_report() -> Result<String, String> {
    let summary =
        source_corpus_summary_details().ok_or_else(|| "source corpus unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

fn source_corpus_summary_for_report() -> String {
    match validated_source_corpus_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Source corpus unavailable ({error})"),
    }
}

fn source_corpus_posture_summary_for_report() -> String {
    source_corpus_summary_for_report()
}

fn render_comparison_corpus_release_guard_summary_text() -> String {
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => {
            return format!("Comparison corpus release-grade guard summary unavailable ({error})")
        }
    };
    format!(
        "Comparison corpus release-grade guard summary\nRelease-grade guard: {release_grade_guard}\n",
    )
}

fn ensure_comparison_corpus_release_guard_summary_matches_current_rendering(
    comparison_corpus_release_guard_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_corpus_release_guard_summary_text
        != render_comparison_corpus_release_guard_summary_text()
    {
        return Err(ReleaseBundleError::Verification(
            "comparison-corpus release-guard summary no longer matches the current comparison-corpus release-guard posture"
                .to_string(),
        ));
    }

    Ok(())
}

fn validated_benchmark_corpus_summary_for_report() -> Result<String, String> {
    let corpus = benchmark_corpus();
    let summary = corpus.summary();
    summary.validate().map_err(|error| error.to_string())?;

    let mut text = String::from("Benchmark corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    text.push('\n');
    Ok(text)
}

fn validated_chart_benchmark_corpus_summary_for_report() -> Result<String, String> {
    let summary = chart_benchmark_corpus_summary();
    summary.validate().map_err(|error| error.to_string())?;

    let mut text = String::from("Chart benchmark corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    text.push('\n');
    Ok(text)
}

fn render_benchmark_corpus_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(|| match validated_benchmark_corpus_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Benchmark corpus summary unavailable ({error})\n"),
        })
        .clone()
}

fn render_chart_benchmark_corpus_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(
            || match validated_chart_benchmark_corpus_summary_for_report() {
                Ok(summary) => summary,
                Err(error) => format!("Chart benchmark corpus summary unavailable ({error})\n"),
            },
        )
        .clone()
}

fn render_reference_snapshot_summary_text() -> String {
    format!(
        "Reference snapshot summary\n{}\n",
        reference_snapshot_summary_for_report()
    )
}

fn render_reference_snapshot_exact_j2000_evidence_text() -> String {
    format!(
        "Reference snapshot exact J2000 evidence summary\n{}\n",
        reference_snapshot_exact_j2000_evidence_summary_for_report()
    )
}

fn render_lunar_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar reference error envelope summary\n{}\n",
        lunar_reference_evidence_envelope_for_report()
    )
}

fn render_lunar_reference_evidence_summary_text() -> String {
    format!(
        "Lunar reference evidence summary\n{}\n",
        lunar_reference_evidence_summary_for_report()
    )
}

fn render_lunar_equatorial_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar equatorial reference error envelope summary\n{}\n",
        lunar_equatorial_reference_evidence_envelope_for_report()
    )
}

fn render_lunar_apparent_comparison_summary_text() -> String {
    format!(
        "Lunar apparent comparison summary\n{}\n",
        lunar_apparent_comparison_summary_for_report()
    )
}

fn render_frame_policy_summary_text() -> String {
    match frame_policy_summary_details().validated_summary_line() {
        Ok(summary) => format!("Frame policy summary\nFrame policy: {}\n", summary),
        Err(error) => format!("Frame policy summary\nFrame policy unavailable ({error})\n"),
    }
}

fn render_reference_holdout_overlap_summary_text() -> String {
    match validated_reference_holdout_overlap_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference/hold-out overlap: unavailable ({error})"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RequestPolicyReportKind {
    Policy,
    Semantics,
}

impl RequestPolicyReportKind {
    const fn title(self) -> &'static str {
        match self {
            Self::Policy => "Request policy summary\n",
            Self::Semantics => "Request semantics summary\n",
        }
    }

    const fn unavailable_prefix(self) -> &'static str {
        match self {
            Self::Policy => "Request policy summary unavailable",
            Self::Semantics => "Request semantics summary unavailable",
        }
    }
}

fn validate_request_policy_report_title(
    kind: RequestPolicyReportKind,
    title: &str,
) -> Result<(), String> {
    let expected = kind.title();
    if title != expected {
        return Err(format!("{} ({title})", kind.unavailable_prefix()));
    }
    Ok(())
}

fn render_request_policy_like_summary_text(
    title: &'static str,
    kind: RequestPolicyReportKind,
) -> String {
    let time_scale_policy = time_scale_policy_summary_for_report();
    if let Err(error) = validate_request_policy_report_title(kind, title) {
        return error;
    }

    let mut text = String::from(title);
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text
}

fn render_request_policy_summary_text() -> String {
    render_request_policy_like_summary_text(
        "Request policy summary\n",
        RequestPolicyReportKind::Policy,
    )
}

fn render_request_semantics_summary_text() -> String {
    use std::fmt::Write as _;

    let mut text = render_request_policy_like_summary_text(
        "Request semantics summary\n",
        RequestPolicyReportKind::Semantics,
    );
    let _ = writeln!(
        text,
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    );
    text
}

fn render_unsupported_modes_summary_text() -> String {
    format!(
        "Unsupported modes summary\nUnsupported modes: {}\n",
        unsupported_modes_summary_for_report()
    )
}

fn render_request_surface_summary_text() -> String {
    format!(
        "Request surface summary\n{}\n",
        request_surface_summary_for_report()
    )
}

fn render_comparison_tolerance_policy_summary_text_from_report(
    report: Result<ComparisonReport, String>,
) -> String {
    match report {
        Ok(report) => format!(
            "Comparison tolerance policy summary\nComparison tolerance policy: {}\n",
            format_comparison_tolerance_policy_for_report(&report)
        ),
        Err(error) => format!(
            "Comparison tolerance policy summary\nComparison tolerance policy unavailable ({error})\n"
        ),
    }
}

fn render_comparison_tolerance_policy_summary_text() -> String {
    render_comparison_tolerance_policy_summary_text_from_report(
        comparison_report_for_default_render(),
    )
}
fn render_comparison_tolerance_scope_coverage_summary_text_from_summary(
    summary: Result<ComparisonTolerancePolicySummary, String>,
) -> String {
    use std::fmt::Write as _;

    let summary = match summary {
        Ok(summary) => match summary.validate() {
            Ok(()) => summary,
            Err(error) => {
                return format!("Comparison tolerance scope coverage summary\nComparison tolerance scope coverage unavailable ({error})\n");
            }
        },
        Err(error) => {
            return format!("Comparison tolerance scope coverage summary\nComparison tolerance scope coverage unavailable ({error})\n");
        }
    };

    let mut text = String::from("Comparison tolerance scope coverage summary\n");
    let _ = writeln!(
        text,
        "Scope coverage posture: {} rows",
        summary.coverage.len()
    );
    for coverage in &summary.coverage {
        let _ = writeln!(text, "  {}", coverage.summary_line());
    }
    text
}

fn render_comparison_tolerance_scope_coverage_summary_text() -> String {
    let summary = match comparison_report_for_default_render() {
        Ok(report) => validated_comparison_tolerance_policy_summary_for_report(&report),
        Err(error) => Err(error),
    };

    render_comparison_tolerance_scope_coverage_summary_text_from_summary(summary)
}

fn render_comparison_body_class_tolerance_summary_text_from_summaries(
    summaries: Result<Vec<BodyClassToleranceSummary>, String>,
) -> String {
    use std::fmt::Write as _;

    let summaries = match summaries {
        Ok(summaries) => summaries,
        Err(error) => {
            return format!("Comparison body-class tolerance summary\nComparison body-class tolerance unavailable ({error})\n");
        }
    };

    if summaries.is_empty() {
        return "Comparison body-class tolerance summary\nComparison body-class tolerance unavailable (comparison report did not produce any body-class tolerance summaries)\n".to_string();
    }

    for summary in &summaries {
        if let Err(error) = summary.validate() {
            return format!("Comparison body-class tolerance summary\nComparison body-class tolerance unavailable ({error})\n");
        }
    }

    let mut text = String::from("Comparison body-class tolerance summary\n");
    let _ = writeln!(text, "Body-class tolerance posture: {}", summaries.len());
    for summary in summaries {
        let _ = writeln!(
            text,
            "  {}",
            format_body_class_tolerance_envelope_for_report(&summary)
        );
    }
    text
}

fn render_comparison_body_class_tolerance_summary_text() -> String {
    let summaries = match comparison_report_for_default_render() {
        Ok(report) => Ok(report.body_class_tolerance_summaries()),
        Err(error) => Err(error),
    };

    render_comparison_body_class_tolerance_summary_text_from_summaries(summaries)
}

fn render_comparison_body_class_tolerance_posture_summary_text() -> String {
    match validated_comparison_body_class_tolerance_posture_for_report() {
        Ok(summary) => format!(
            "Comparison body-class tolerance posture summary\n{}\n",
            summary
        ),
        Err(error) => format!(
            "Comparison body-class tolerance posture summary\nComparison body-class tolerance unavailable ({error})\n"
        ),
    }
}

fn render_comparison_envelope_summary_text() -> String {
    let report = match comparison_report_for_default_render() {
        Ok(report) => report,
        Err(error) => {
            return format!(
                "Comparison envelope summary\nComparison envelope unavailable ({error})\n"
            );
        }
    };
    let envelope = comparison_envelope_summary(&report.summary, &report.samples);
    let summary_line = envelope
        .validated_summary_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison envelope unavailable ({error})"));
    let percentile_line = envelope
        .validated_percentile_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison percentile envelope unavailable ({error})"));

    format!(
        "Comparison envelope summary\nSummary line: {summary_line}\nPercentile line: {percentile_line}\n"
    )
}

fn ensure_comparison_envelope_summary_matches_current_rendering(
    comparison_envelope_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_envelope_summary_text == render_comparison_envelope_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison envelope summary no longer matches the current comparison envelope posture"
                .to_string(),
        ))
    }
}

fn ensure_comparison_body_class_tolerance_summary_matches_current_rendering(
    comparison_body_class_tolerance_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_body_class_tolerance_summary_text
        == render_comparison_body_class_tolerance_summary_text()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison body-class tolerance summary no longer matches the current comparison body-class tolerance posture"
                .to_string(),
        ))
    }
}

fn validate_release_body_claims_posture(
    release_body_claims_summary: &str,
    pluto_fallback_summary: &str,
) -> Result<(), String> {
    validate_release_body_claims_posture_backend(
        release_body_claims_summary,
        pluto_fallback_summary,
    )
    .map_err(|error| error.to_string())
}

#[derive(Clone, Debug, PartialEq)]
struct BodyDateChannelClaimsSummary {
    release_body_claims: String,
    frame_policy: String,
    production_generation_date_range: String,
    production_generation_coverage: String,
    corpus_shape: String,
    coverage_posture: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum BodyDateChannelClaimsSummaryValidationError {
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for BodyDateChannelClaimsSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the body/date/channel claims summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for BodyDateChannelClaimsSummaryValidationError {}

impl BodyDateChannelClaimsSummary {
    fn summary_line(&self) -> String {
        format!(
            "bodies={}; frame policy={}; date range={}; production generation coverage={}; corpus shape={}; coverage posture={}",
            self.release_body_claims,
            self.frame_policy,
            self.production_generation_date_range,
            self.production_generation_coverage,
            self.corpus_shape,
            self.coverage_posture
        )
    }

    fn validate(&self) -> Result<(), BodyDateChannelClaimsSummaryValidationError> {
        let expected = body_date_channel_claims_summary_details().ok_or(
            BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                field: "body_date_channel_claims_summary",
            },
        )?;

        if self.release_body_claims != expected.release_body_claims {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "release_body_claims",
                },
            );
        }
        if self.frame_policy != expected.frame_policy {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "frame_policy",
                },
            );
        }
        if self.production_generation_date_range != expected.production_generation_date_range {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "production_generation_date_range",
                },
            );
        }
        if self.production_generation_coverage != expected.production_generation_coverage {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "production_generation_coverage",
                },
            );
        }
        if self.corpus_shape != expected.corpus_shape {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "corpus_shape",
                },
            );
        }
        if self.coverage_posture != expected.coverage_posture {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "coverage_posture",
                },
            );
        }
        Ok(())
    }

    fn validated_summary_line(
        &self,
    ) -> Result<String, BodyDateChannelClaimsSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

#[allow(dead_code)]
fn strip_report_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

fn production_generation_coverage_posture_for_report() -> Option<String> {
    let production_generation_coverage = required_summary_payload(
        pleiades_jpl::production_generation_snapshot_summary_for_report(),
        "Production generation coverage: ",
        "production generation coverage",
    )
    .ok()?;
    let production_generation_body_class_coverage = required_summary_payload(
        pleiades_jpl::production_generation_snapshot_body_class_coverage_summary_for_report(),
        "Production generation body-class coverage: ",
        "production generation body-class coverage",
    )
    .ok()?;
    validated_production_generation_corpus_shape_summary_for_report().ok()?;

    Some(format!(
        "production-generation coverage and corpus shape remain aligned across the advertised 1500-2500 CE window; coverage={}; body-class coverage={}",
        production_generation_coverage,
        production_generation_body_class_coverage,
    ))
}

fn production_generation_date_range_for_report() -> Option<String> {
    let production_generation_window =
        pleiades_jpl::production_generation_snapshot_window_summary()?;

    Some(format!(
        "{}..{}",
        format_instant(production_generation_window.earliest_epoch),
        format_instant(production_generation_window.latest_epoch)
    ))
}

fn body_date_channel_claims_summary_details() -> Option<BodyDateChannelClaimsSummary> {
    let coverage_posture = production_generation_coverage_posture_for_report()?;
    Some(BodyDateChannelClaimsSummary {
        release_body_claims: validated_release_body_claims_summary_line_for_report()
            .ok()?
            .to_string(),
        frame_policy: validated_frame_policy_summary_for_report(),
        production_generation_date_range: production_generation_date_range_for_report()?,
        production_generation_coverage: production_generation_snapshot_summary_for_report(),
        corpus_shape: validated_production_generation_corpus_shape_summary_for_report().ok()?,
        coverage_posture,
    })
}

fn format_release_body_claims_summary_for_report() -> String {
    let summary_line = match validated_release_body_claims_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => return format!("release-grade body claims unavailable ({error})"),
    };
    let pluto_line = match validated_pluto_fallback_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => return format!("release-grade body claims unavailable ({error})"),
    };
    if let Err(error) = validate_release_body_claims_posture(summary_line, pluto_line) {
        return format!("release-grade body claims unavailable ({error})");
    }
    summary_line.to_string()
}

fn format_body_date_channel_claims_summary_for_report() -> String {
    let summary = match body_date_channel_claims_summary_details() {
        Some(summary) => summary,
        None => {
            return "body/date/channel claims unavailable (source corpus unavailable)".to_string()
        }
    };
    match summary.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("body/date/channel claims unavailable ({error})"),
    }
}

fn render_release_body_claims_summary_text() -> String {
    format!(
        "Release-grade body claims summary\nRelease-grade body claims: {}\n",
        format_release_body_claims_summary_for_report()
    )
}

fn render_body_date_channel_claims_summary_text() -> String {
    format!(
        "Body/date/channel claims summary\nBody/date/channel claims: {}\n",
        format_body_date_channel_claims_summary_for_report()
    )
}

fn render_pluto_fallback_summary_text_from_report(
    report: Result<ComparisonReport, String>,
) -> String {
    let policy_line = match validated_pluto_fallback_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => {
            return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
        }
    };

    let release_body_claims_line = match validated_release_body_claims_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => {
            return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
        }
    };
    if let Err(error) = validate_release_body_claims_posture(release_body_claims_line, policy_line)
    {
        return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
    }

    let report = match report {
        Ok(report) => report,
        Err(error) => {
            return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
        }
    };
    let summary = match comparison_tolerance_policy_summary_details(&report)
        .entries
        .into_iter()
        .find(|entry| entry.scope == ComparisonToleranceScope::Pluto)
    {
        Some(summary) => summary,
        None => {
            return "Pluto fallback summary\nPluto fallback unavailable (comparison report is missing a Pluto scope entry)\n".to_string();
        }
    };
    match summary.validated_summary_line() {
        Ok(line) => format!(
            "Pluto fallback summary\nRelease-grade body claims: {}\nPluto fallback policy: {policy_line}\nPluto fallback: {line}\n",
            format_release_body_claims_summary_for_report()
        ),
        Err(error) => format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n"),
    }
}

fn render_pluto_fallback_summary_text() -> String {
    render_pluto_fallback_summary_text_from_report(comparison_report_for_default_render())
}

fn validated_api_stability_profile_for_report() -> Result<pleiades_core::ApiStabilityProfile, String>
{
    let profile = current_api_stability_profile();
    profile.validate().map_err(|error| error.to_string())?;
    Ok(profile)
}

fn validated_compatibility_profile_for_report() -> Result<CompatibilityProfile, String> {
    let profile = current_compatibility_profile();
    profile.validate().map_err(|error| error.to_string())?;
    Ok(profile)
}

fn validated_release_profile_identifiers_for_report() -> Result<ReleaseProfileIdentifiers, String> {
    let release_profiles = current_release_profile_identifiers();
    release_profiles
        .validate()
        .map_err(|error| error.to_string())?;
    Ok(release_profiles)
}

fn validated_catalog_inventory_summary_for_report() -> Result<String, String> {
    validated_compatibility_profile_for_report()?;
    validated_release_profile_identifiers_for_report()?;
    core_validated_catalog_inventory_summary_for_report().map_err(|error| error.to_string())
}

#[cfg(test)]
fn validated_house_code_aliases_summary_for_profile(
    profile: &CompatibilityProfile,
) -> Result<String, String> {
    profile
        .validated_house_code_aliases_summary_line()
        .map_err(|error| error.to_string())
}

fn validated_house_code_aliases_summary_for_report() -> Result<String, String> {
    core_validated_house_code_aliases_summary_for_report().map_err(|error| error.to_string())
}

fn validated_release_profile_identifiers_summary_for_report(
    release_profiles: &ReleaseProfileIdentifiers,
) -> String {
    match core_validated_release_profile_identifiers_summary_for_report(release_profiles) {
        Ok(summary) => summary,
        Err(error) => format!("unavailable ({error})"),
    }
}

/// Renders the compact release-profile identifiers summary.
pub fn render_release_profile_identifiers_summary() -> String {
    render_release_profile_identifiers_summary_text()
}

fn render_release_profile_identifiers_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let release_profiles = match validated_release_profile_identifiers_for_report() {
                Ok(release_profiles) => release_profiles,
                Err(error) => {
                    return format!("Release profile identifiers summary unavailable ({error})");
                }
            };

            let mut text = String::new();
            text.push_str("Release profile identifiers summary\n");
            text.push_str("Summary line: ");
            text.push_str(&validated_release_profile_identifiers_summary_for_report(
                &release_profiles,
            ));
            text.push('\n');
            text.push_str("Compatibility profile: ");
            text.push_str(release_profiles.compatibility_profile_id);
            text.push('\n');
            text.push_str("API stability posture: ");
            text.push_str(release_profiles.api_stability_profile_id);
            text.push('\n');
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("API stability summary: api-stability-summary\n");
            text.push_str("Release summary: release-summary\n");

            text
        })
        .clone()
}

fn api_stability_summary_line_for_report() -> String {
    match validated_api_stability_profile_for_report() {
        Ok(profile) => profile.summary_line(),
        Err(error) => format!("API stability summary unavailable ({error})"),
    }
}

/// Compact inventory of the public request surfaces that are called out in the
/// time-observer policy and release-facing validation summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestSurfaceSummary {
    instant: &'static str,
    chart_request: &'static str,
    backend_request: &'static str,
    house_request: &'static str,
    request_policy: &'static str,
    cli_chart: &'static str,
}

impl RequestSurfaceSummary {
    /// Returns the current compact request-surface inventory.
    pub const fn current() -> Self {
        Self {
            instant: "pleiades-types::Instant (tagged instant plus caller-supplied retagging)",
            chart_request: "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)",
            backend_request:
                "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)",
            house_request: "pleiades-houses::HouseRequest (house-only observer calculations)",
            request_policy:
                "request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints)",
            cli_chart: "pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)",
        }
    }

    /// Validates that the cached inventory still matches the documented request surfaces.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        const EXPECTED_INSTANT: &str =
            "pleiades-types::Instant (tagged instant plus caller-supplied retagging)";
        const EXPECTED_CHART_REQUEST: &str =
            "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)";
        const EXPECTED_BACKEND_REQUEST: &str =
            "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)";
        const EXPECTED_HOUSE_REQUEST: &str =
            "pleiades-houses::HouseRequest (house-only observer calculations)";
        const EXPECTED_REQUEST_POLICY: &str =
            "request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints)";
        const EXPECTED_CLI_CHART: &str = "pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)";

        validate_request_surface_label("instant", self.instant, EXPECTED_INSTANT)?;
        validate_request_surface_label(
            "chart request",
            self.chart_request,
            EXPECTED_CHART_REQUEST,
        )?;
        validate_request_surface_label(
            "backend request",
            self.backend_request,
            EXPECTED_BACKEND_REQUEST,
        )?;
        validate_request_surface_label(
            "house request",
            self.house_request,
            EXPECTED_HOUSE_REQUEST,
        )?;
        validate_request_surface_label(
            "request policy",
            self.request_policy,
            EXPECTED_REQUEST_POLICY,
        )?;
        validate_request_surface_label("CLI chart", self.cli_chart, EXPECTED_CLI_CHART)?;

        Ok(())
    }

    /// Returns the chart-help clause that spells out the explicit UTC/UT1 and
    /// TT/TDB aliases used by the chart CLI.
    pub fn validated_chart_help_clause(self) -> Result<&'static str, EphemerisError> {
        self.validate()?;
        Ok(self.cli_chart)
    }

    /// Returns the chart-help clause that spells out the explicit UTC/UT1 and
    /// TT/TDB aliases used by the chart CLI.
    pub const fn chart_help_clause(self) -> &'static str {
        self.cli_chart
    }

    /// Returns the compact `Primary request surfaces:` line.
    pub fn summary_line(self) -> String {
        format!(
            "Primary request surfaces: {}; {}; {}; {}; {}; {}",
            self.instant,
            self.chart_request,
            self.backend_request,
            self.house_request,
            self.request_policy,
            self.cli_chart,
        )
    }

    /// Validates the summary and returns its compact report line.
    pub fn validated_summary_line(self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for RequestSurfaceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn validate_request_surface_label(
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), EphemerisError> {
    if actual == expected {
        return Ok(());
    }

    Err(EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("primary request surface {field} mismatch: expected {expected}, found {actual}"),
    ))
}

/// Returns the current compact request-surface inventory.
pub const fn current_request_surface_summary() -> RequestSurfaceSummary {
    RequestSurfaceSummary::current()
}

fn request_surface_summary_for_report() -> String {
    let summary = RequestSurfaceSummary::current();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("primary request surfaces unavailable ({error})"),
    }
}

fn format_vsop87_request_policy_summary() -> String {
    vsop87_request_policy_summary_for_report()
}

fn format_vsop87_source_audit_summary() -> String {
    source_audit_summary_for_report()
}

fn format_packaged_artifact_profile_summary() -> String {
    packaged_artifact_profile_summary_with_body_coverage()
}

fn validated_packaged_artifact_output_support_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_output_support_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact output support: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_speed_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_speed_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact speed policy: unavailable ({error})"),
    }
}

fn validated_motion_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_speed_policy_summary_details();
    match summary.validate() {
        Ok(()) => format!("Motion policy: {}", summary.summary_line()),
        Err(error) => format!("Motion policy: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_access_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_access_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact access: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_generation_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_generation_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_body_cadence_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_body_cadence_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body cadence: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_body_class_span_cap_summary_for_report() -> String {
    format!(
        "Packaged-artifact body-class span caps: {}",
        pleiades_data::packaged_artifact_body_class_span_cap_entries_for_report()
    )
}

fn validated_packaged_artifact_normalized_intermediate_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_normalized_intermediate_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact normalized intermediates: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_storage_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_storage_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact storage/reconstruction: unavailable ({error})"),
    }
}

fn validated_packaged_frame_treatment_summary_for_report() -> String {
    let summary = pleiades_data::packaged_frame_treatment_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged frame treatment: unavailable ({error})"),
    }
}

fn ensure_packaged_artifact_storage_summary_matches_current_rendering(
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

fn ensure_packaged_frame_treatment_summary_matches_current_rendering(
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

fn validated_packaged_artifact_target_threshold_state_for_report() -> String {
    pleiades_data::packaged_artifact_target_threshold_state_for_report()
}

fn validated_packaged_artifact_target_threshold_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_target_threshold_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged-artifact target thresholds: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report() -> String {
    let summary =
        pleiades_data::packaged_artifact_target_threshold_scope_envelopes_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("scope envelopes: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_source_fit_holdout_sync_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_source_fit_holdout_sync_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("source-fit and hold-out sync: unavailable ({error})"),
    }
}

fn validated_packaged_artifact_phase2_corpus_alignment_summary_for_report() -> String {
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

fn format_packaged_artifact_output_support_summary() -> String {
    validated_packaged_artifact_output_support_summary_for_report()
}

fn format_packaged_artifact_speed_policy_summary() -> String {
    validated_packaged_artifact_speed_policy_summary_for_report()
}

fn format_packaged_artifact_generation_policy_summary() -> String {
    validated_packaged_artifact_generation_policy_summary_for_report()
}

fn validate_packaged_artifact_generation_residual_bodies_summary(
    summary: &pleiades_compression::ArtifactResidualBodyCoverageSummary,
    artifact: &pleiades_compression::CompressedArtifact,
) -> Result<String, String> {
    summary
        .validated_summary_line_with_body_count(artifact)
        .map_err(|error| error.to_string())
}

fn validated_packaged_artifact_generation_residual_bodies_summary_for_report(
) -> Result<String, String> {
    validate_packaged_artifact_generation_residual_bodies_summary(
        &pleiades_data::packaged_artifact_generation_residual_bodies_summary_details(),
        packaged_artifact(),
    )
}

fn validated_packaged_artifact_production_profile_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_production_profile_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact production profile draft: unavailable ({error})"),
    }
}

fn format_packaged_artifact_storage_summary() -> String {
    validated_packaged_artifact_storage_summary_for_report()
}

fn format_packaged_artifact_access_summary() -> String {
    validated_packaged_artifact_access_summary_for_report()
}

fn format_packaged_frame_parity_summary() -> String {
    packaged_frame_parity_summary_for_report()
}

fn format_lunar_frame_treatment_summary() -> String {
    lunar_theory_frame_treatment_summary_for_report()
}

fn format_packaged_frame_treatment_summary() -> String {
    packaged_frame_treatment_summary_for_report()
}

fn format_comparison_snapshot_manifest_summary() -> String {
    match validated_comparison_snapshot_manifest_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Comparison snapshot manifest: unavailable ({error})"),
    }
}

fn render_validation_report_summary_text(report: &ValidationReport) -> String {
    use std::fmt::Write as _;

    if let Err(error) = report.validate() {
        return format!("Validation report summary unavailable ({error})");
    }

    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Validation report summary unavailable ({error})"),
    };
    let request_policy = request_policy_summary_for_report();
    let comparison_regressions = report.comparison.notable_regressions().len();
    let mut text = String::new();
    let _ = writeln!(text, "Validation report summary");
    let _ = writeln!(
        text,
        "Profile: {}",
        release_profiles.compatibility_profile_id
    );
    let _ = writeln!(
        text,
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    );
    let _ = writeln!(
        text,
        "Release profile identifiers: {}",
        validated_release_profile_identifiers_summary_for_report(&release_profiles)
    );
    let _ = writeln!(text, "Time-scale policy: {}", request_policy.time_scale);
    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );
    let utc_convenience_policy =
        pleiades_backend::validated_utc_convenience_policy_summary_for_report();
    let _ = writeln!(text, "UTC convenience policy: {}", utc_convenience_policy);
    let _ = writeln!(text, "Observer policy: {}", request_policy.observer);
    let _ = writeln!(text, "Apparentness policy: {}", request_policy.apparentness);
    let native_sidereal_policy =
        pleiades_backend::validated_native_sidereal_policy_summary_for_report();
    let _ = writeln!(text, "Native sidereal policy: {}", native_sidereal_policy);
    let _ = writeln!(text, "Frame policy: {}", request_policy.frame);
    let _ = writeln!(
        text,
        "Mean-obliquity frame round-trip: {}",
        mean_obliquity_frame_round_trip_summary_for_report()
    );
    let _ = writeln!(
        text,
        "Request policy: {}",
        format_request_policy_summary_for_report(&request_policy)
    );
    let _ = writeln!(text, "{}", request_surface_summary_for_report());
    let _ = writeln!(
        text,
        "Zodiac policy: {}",
        validated_zodiac_policy_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison corpus");
    let _ = writeln!(text, "  name: {}", report.comparison_corpus.name);
    let _ = writeln!(
        text,
        "  requests: {}",
        report.comparison_corpus.request_count
    );
    let _ = writeln!(text, "  epochs: {}", report.comparison_corpus.epoch_count);
    let _ = writeln!(
        text,
        "  epoch labels: {}",
        format_instant_list(&report.comparison_corpus.epochs)
    );
    let _ = writeln!(text, "  bodies: {}", report.comparison_corpus.body_count);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.comparison_corpus.apparentness
    );
    let _ = writeln!(text, "  {}", comparison_snapshot_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        comparison_snapshot_body_class_coverage_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        comparison_snapshot_source_summary_for_report()
    );
    let _ = writeln!(text, "  {}", format_comparison_snapshot_manifest_summary());
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => return format!("Comparison corpus summary unavailable ({error})"),
    };
    let _ = writeln!(text, "  release-grade guard: {release_grade_guard}");
    let _ = writeln!(
        text,
        "  Source corpus: {}",
        source_corpus_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Source corpus posture: {}",
        source_corpus_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Reference snapshot");
    let _ = writeln!(text, "  {}", reference_snapshot_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451911_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(text, "  {}", reference_snapshot_source_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_source_window_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_body_class_coverage_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_dense_boundary_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "House validation corpus");
    let _ = writeln!(
        text,
        "  {}",
        house_validation_summary_line_for_report(&report.house_validation)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(
        text,
        "  samples: {}",
        report.comparison.summary.sample_count
    );
    let median = comparison_median_envelope_for_samples(&report.comparison.samples);
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°{}",
        report.comparison.summary.max_longitude_delta_deg,
        format_summary_body(&report.comparison.summary.max_longitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°{}",
        report.comparison.summary.max_latitude_delta_deg,
        format_summary_body(&report.comparison.summary.max_latitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}{}",
        report
            .comparison
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string()),
        format_summary_body(&report.comparison.summary.max_distance_delta_body)
    );
    let _ = writeln!(
        text,
        "  mean longitude delta: {:.12}°",
        report.comparison.summary.mean_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  median longitude delta: {:.12}°",
        median.longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  rms longitude delta: {:.12}°",
        report.comparison.summary.rms_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  mean latitude delta: {:.12}°",
        report.comparison.summary.mean_latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  median latitude delta: {:.12}°",
        median.latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  rms latitude delta: {:.12}°",
        report.comparison.summary.rms_latitude_delta_deg
    );
    if let Some(value) = report.comparison.summary.mean_distance_delta_au {
        let _ = writeln!(text, "  mean distance delta: {:.12} AU", value);
    }
    if let Some(value) = median.distance_delta_au {
        let _ = writeln!(text, "  median distance delta: {:.12} AU", value);
    }
    if let Some(value) = report.comparison.summary.rms_distance_delta_au {
        let _ = writeln!(text, "  rms distance delta: {:.12} AU", value);
    }
    let _ = writeln!(
        text,
        "  {}",
        format_comparison_percentile_envelope_for_report(&report.comparison.samples)
    );
    let _ = writeln!(text, "  notable regressions: {}", comparison_regressions);
    let _ = writeln!(
        text,
        "  regression bodies: {}",
        format_regression_bodies(&report.comparison.notable_regressions())
    );
    let _ = writeln!(
        text,
        "Comparison tolerance policy: {}",
        format_comparison_tolerance_policy_for_report(&report.comparison)
    );
    let _ = writeln!(
        text,
        "Comparison audit: {}",
        comparison_audit_summary_for_report(&report.comparison)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "JPL interpolation quality");
    let _ = writeln!(
        text,
        "  {}",
        format_jpl_interpolation_quality_summary_for_report()
    );
    let _ = writeln!(text, "  {}", jpl_independent_holdout_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        render_reference_holdout_overlap_summary_text()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_bridge_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451916_major_body_interior_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451918_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451919_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451920_major_body_interior_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_mars_jupiter_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_mars_outer_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_window_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        independent_holdout_snapshot_batch_parity_summary_text()
    );
    let _ = writeln!(
        text,
        "  {}",
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "JPL request policy: {}",
        jpl_snapshot_request_policy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "{}",
        jpl_snapshot_batch_error_taxonomy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "JPL frame treatment: {}",
        format_jpl_frame_treatment_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark summaries");
    let _ = writeln!(text, "Reference benchmark");
    let _ = writeln!(text, "  corpus: {}", report.reference_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.reference_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.reference_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.reference_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.reference_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.reference_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.reference_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Candidate benchmark");
    let _ = writeln!(text, "  corpus: {}", report.candidate_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.candidate_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.candidate_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.candidate_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.candidate_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.candidate_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.candidate_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-data benchmark");
    let _ = writeln!(text, "  corpus: {}", report.packaged_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.packaged_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.packaged_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.packaged_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.packaged_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.packaged_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.packaged_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged artifact decode benchmark");
    let _ = writeln!(
        text,
        "  artifact: {}",
        report.artifact_decode_benchmark.artifact_label
    );
    let _ = writeln!(
        text,
        "  source: {}",
        report.artifact_decode_benchmark.source
    );
    let _ = writeln!(
        text,
        "  rounds: {}",
        report.artifact_decode_benchmark.rounds
    );
    let _ = writeln!(
        text,
        "  decodes per round: {}",
        report.artifact_decode_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  encoded bytes: {}",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(
        text,
        "  ns/decode: {}",
        format_ns(report.artifact_decode_benchmark.nanoseconds_per_decode())
    );
    let _ = writeln!(
        text,
        "  decodes per second: {:.2} decodes/s",
        report.artifact_decode_benchmark.decodes_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Chart benchmark");
    let _ = writeln!(text, "  corpus: {}", report.chart_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.chart_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.chart_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.chart_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/chart: {}",
        format_ns(report.chart_benchmark.nanoseconds_per_chart())
    );
    let _ = writeln!(
        text,
        "  charts per second: {:.2} charts/s",
        report.chart_benchmark.charts_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "ELP lunar capability: {}",
        lunar_theory_capability_summary_for_report()
    );
    let _ = writeln!(
        text,
        "ELP lunar request policy: {}",
        lunar_theory_request_policy_summary()
    );
    let _ = writeln!(
        text,
        "ELP frame treatment: {}",
        format_lunar_frame_treatment_summary()
    );
    let _ = writeln!(
        text,
        "ELP lunar theory limitations: {}",
        lunar_theory_limitations_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_theory_source_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar reference");
    let _ = writeln!(text, "  {}", lunar_reference_evidence_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        lunar_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_reference_evidence_envelope_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar equatorial reference");
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_evidence_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_evidence_envelope_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar apparent comparison");
    let _ = writeln!(text, "  {}", lunar_apparent_comparison_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar source windows");
    let _ = writeln!(text, "  {}", lunar_source_window_summary_for_report());
    let _ = writeln!(text, "Lunar high-curvature continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        lunar_high_curvature_continuity_evidence_for_report()
    );
    let _ = writeln!(text, "Lunar high-curvature equatorial continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        lunar_high_curvature_equatorial_continuity_evidence_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Body comparison summaries");
    for summary in report.comparison.body_summaries() {
        let _ = writeln!(
            text,
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, rms Δdist={}",
            summary.body,
            summary.sample_count,
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_longitude_delta_deg,
            summary.rms_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_latitude_delta_deg,
            summary.rms_latitude_delta_deg,
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary
                .mean_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class error envelopes");
    for summary in report.comparison.body_class_summaries() {
        let max_longitude_body = summary
            .max_longitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_latitude_body = summary
            .max_latitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_distance_body = summary
            .max_distance_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let _ = writeln!(
            text,
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, median Δdist={}, p95 Δdist={}, rms Δdist={}",
            summary.class.label(),
            summary.sample_count,
            summary.max_longitude_delta_deg,
            max_longitude_body,
            summary.mean_longitude_delta_deg(),
            summary.median_longitude_delta_deg,
            summary.percentile_longitude_delta_deg,
            summary.rms_longitude_delta_deg(),
            summary.max_latitude_delta_deg,
            max_latitude_body,
            summary.mean_latitude_delta_deg(),
            summary.median_latitude_delta_deg,
            summary.percentile_latitude_delta_deg,
            summary.rms_latitude_delta_deg(),
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_distance_body,
            summary
                .mean_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .median_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .percentile_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class tolerance posture");
    for summary in report.comparison.body_class_tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}",
            format_body_class_tolerance_envelope_for_report(&summary)
        );
        if !summary.outside_bodies.is_empty() {
            let _ = writeln!(
                text,
                "    outside bodies: {}",
                format_bodies(&summary.outside_bodies)
            );
        }
        let _ = writeln!(
            text,
            "    mean Δlon={:.12}°, median Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={}, median Δdist={}, rms Δdist={}",
            summary.mean_longitude_delta_deg(),
            summary.median_longitude_delta_deg,
            summary.rms_longitude_delta_deg(),
            summary.mean_latitude_delta_deg(),
            summary.median_latitude_delta_deg,
            summary.rms_latitude_delta_deg(),
            summary
                .mean_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .median_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Tolerance policy");
    write_tolerance_policy_text(&mut text, &report.comparison);
    let _ = writeln!(text);
    let _ = writeln!(text, "Expected tolerance status");
    for summary in report.comparison.tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}: profile={}, status={}, limit Δlon≤{:.6}°, margin Δlon={:+.12}°, limit Δlat≤{:.6}°, margin Δlat={:+.12}°, limit Δdist={}, margin Δdist={}, measured max Δlon={:.12}°, max Δlat={:.12}°, max Δdist={}",
            summary.body,
            summary.tolerance.profile,
            if summary.within_tolerance { "within" } else { "exceeded" },
            summary.tolerance.max_longitude_delta_deg,
            summary.longitude_margin_deg,
            summary.tolerance.max_latitude_delta_deg,
            summary.latitude_margin_deg,
            summary
                .tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .distance_margin_au
                .map(|value| format!("{value:+.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary.max_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison tolerance audit");
    let (audit_body_count, audit_within_count, audit_outside_count, audit_regression_count) =
        comparison_audit_totals(&report.comparison);
    let _ = writeln!(text, "  command: compare-backends-audit");
    let _ = writeln!(
        text,
        "  status: {}",
        if audit_regression_count == 0 {
            "clean"
        } else {
            "regressions found"
        }
    );
    let _ = writeln!(text, "  bodies checked: {}", audit_body_count);
    let _ = writeln!(text, "  within tolerance bodies: {}", audit_within_count);
    let _ = writeln!(text, "  outside tolerance bodies: {}", audit_outside_count);
    let _ = writeln!(text, "  notable regressions: {}", audit_regression_count);
    let _ = writeln!(text);
    let house_validation_summary =
        house_validation_summary_line_for_report(&report.house_validation);
    let house_validation_summary = house_validation_summary
        .strip_prefix("House validation corpus: ")
        .unwrap_or(&house_validation_summary);
    let _ = writeln!(
        text,
        "House validation corpus: {}",
        house_validation_summary
    );
    let _ = writeln!(text, "{}", format_ayanamsa_catalog_validation_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "VSOP87 source-backed evidence");
    let _ = writeln!(text, "  {}", format_vsop87_source_documentation_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_source_documentation_health_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_frame_treatment_summary());
    let _ = writeln!(
        text,
        "  VSOP87 request policy: {}",
        format_vsop87_request_policy_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_source_audit_summary());
    let _ = writeln!(text, "  {}", generated_binary_audit_summary_for_report());
    let _ = writeln!(text, "  {}", format_vsop87_canonical_evidence_summary());
    let _ = writeln!(text, "  {}", format_vsop87_canonical_outlier_note_summary());
    let _ = writeln!(text, "  {}", format_vsop87_equatorial_evidence_summary());
    let _ = writeln!(text, "  {}", format_vsop87_j2000_batch_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j2000_ecliptic_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j2000_equatorial_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j1900_ecliptic_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j1900_equatorial_batch_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_mixed_batch_summary());
    let _ = writeln!(text, "  {}", format_vsop87_j1900_batch_summary());
    let _ = writeln!(text, "  {}", format_vsop87_body_evidence_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_source_body_class_evidence_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_equatorial_body_class_evidence_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "ELP lunar theory specification");
    let _ = writeln!(text, "  {}", lunar_theory_catalog_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        validated_lunar_theory_catalog_validation_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_theory_source_summary_for_report());
    let _ = writeln!(text, "  {}", lunar_theory_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-artifact profile");
    let _ = writeln!(text, "  {}", format_packaged_artifact_profile_summary());
    let _ = writeln!(
        text,
        "  Packaged-artifact output support: {}",
        format_packaged_artifact_output_support_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact speed policy: {}",
        format_packaged_artifact_speed_policy_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact storage/reconstruction: {}",
        format_packaged_artifact_storage_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact access: {}",
        format_packaged_artifact_access_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation policy: {}",
        format_packaged_artifact_generation_policy_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact normalized intermediates: {}",
        packaged_artifact_normalized_intermediate_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation residual bodies: {}",
        match validated_packaged_artifact_generation_residual_bodies_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => return format!("Validation report summary unavailable ({error})"),
        }
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target thresholds: {}",
        validated_packaged_artifact_target_threshold_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target-threshold state: {}",
        validated_packaged_artifact_target_threshold_state_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit envelope: {}",
        packaged_artifact_fit_envelope_summary_for_report()
    );
    let fit_margin_summary = report_summary_payload(
        packaged_artifact_fit_margin_summary_for_report(),
        "fit margins: ",
    );
    let fit_threshold_violation_count_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_count_for_report(),
        "fit threshold violations: ",
    );
    let fit_threshold_violation_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_summary_for_report(),
        "fit threshold violations: ",
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit margins: {}",
        fit_margin_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit threshold violation count: {}",
        fit_threshold_violation_count_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit threshold violations: {}",
        fit_threshold_violation_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit sample classes: {}",
        packaged_artifact_fit_sample_classes_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit outliers: {}",
        packaged_artifact_fit_outlier_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target-threshold scope envelopes: {}",
        validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact source-fit and hold-out sync: {}",
        validated_packaged_artifact_source_fit_holdout_sync_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact phase-2 corpus alignment: {}",
        validated_packaged_artifact_phase2_corpus_alignment_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation manifest: {}",
        packaged_artifact_generation_manifest_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact size: {} bytes",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(text, "  {}", packaged_request_policy_summary_for_report());
    let _ = writeln!(
        text,
        "  Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged batch parity: {}",
        packaged_mixed_tt_tdb_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged frame parity: {}",
        format_packaged_frame_parity_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged frame treatment: {}",
        format_packaged_frame_treatment_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark summaries");
    let _ = writeln!(text, "Reference benchmark");
    let _ = writeln!(text, "  corpus: {}", report.reference_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.reference_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.reference_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.reference_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.reference_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.reference_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.reference_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Candidate benchmark");
    let _ = writeln!(text, "  corpus: {}", report.candidate_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.candidate_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.candidate_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.candidate_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.candidate_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.candidate_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.candidate_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-data benchmark");
    let _ = writeln!(text, "  corpus: {}", report.packaged_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.packaged_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.packaged_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.packaged_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.packaged_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.packaged_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.packaged_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged artifact decode benchmark");
    let _ = writeln!(
        text,
        "  artifact: {}",
        report.artifact_decode_benchmark.artifact_label
    );
    let _ = writeln!(
        text,
        "  source: {}",
        report.artifact_decode_benchmark.source
    );
    let _ = writeln!(
        text,
        "  rounds: {}",
        report.artifact_decode_benchmark.rounds
    );
    let _ = writeln!(
        text,
        "  decodes per round: {}",
        report.artifact_decode_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  encoded bytes: {}",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(
        text,
        "  ns/decode: {}",
        format_ns(report.artifact_decode_benchmark.nanoseconds_per_decode())
    );
    let _ = writeln!(
        text,
        "  decodes per second: {:.2} decodes/s",
        report.artifact_decode_benchmark.decodes_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Chart benchmark");
    let _ = writeln!(text, "  corpus: {}", report.chart_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.chart_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.chart_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.chart_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/chart: {}",
        format_ns(report.chart_benchmark.nanoseconds_per_chart())
    );
    let _ = writeln!(
        text,
        "  charts per second: {:.2} charts/s",
        report.chart_benchmark.charts_per_second()
    );
    let _ = writeln!(text, "Release bundle verification: verify-release-bundle");
    let _ = writeln!(text, "Workspace audit: workspace-audit / audit");
    let _ = writeln!(
        text,
        "Compatibility profile summary: compatibility-profile-summary"
    );
    let _ = writeln!(text, "Release notes summary: release-notes-summary");
    let _ = writeln!(text, "Release checklist summary: release-checklist-summary");
    let _ = writeln!(text, "Release summary: release-summary");

    text
}

/// Renders a compact summary of the implemented backend capability matrix catalog.
pub fn render_backend_matrix_summary() -> String {
    render_backend_matrix_summary_text()
}

fn native_sidereal_posture_line(native_sidereal_count: usize) -> String {
    match native_sidereal_count {
        0 => "Native sidereal posture: unsupported across first-party backends".to_string(),
        1 => "Native sidereal posture: supported natively by 1 backend".to_string(),
        count => format!("Native sidereal posture: supported natively by {count} backends"),
    }
}

fn render_backend_matrix_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    };
    if let Err(error) = validated_compatibility_profile_for_report() {
        return format!("Backend matrix summary unavailable ({error})");
    }
    let catalog = implemented_backend_catalog();
    let mut family_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut bodies: Vec<String> = Vec::new();
    let mut frames: Vec<String> = Vec::new();
    let mut time_scales: Vec<String> = Vec::new();
    let mut deterministic_count = 0usize;
    let mut offline_count = 0usize;
    let mut batch_count = 0usize;
    let mut native_sidereal_count = 0usize;
    let mut bounded_nominal_range_count = 0usize;
    let mut open_ended_nominal_range_count = 0usize;
    let mut exact_accuracy_count = 0usize;
    let mut high_accuracy_count = 0usize;
    let mut moderate_accuracy_count = 0usize;
    let mut approximate_accuracy_count = 0usize;
    let mut unknown_accuracy_count = 0usize;
    let mut selected_asteroid_count = 0usize;
    let mut data_source_count = 0usize;
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();

    for entry in &catalog {
        *status_counts
            .entry(entry.implementation_status.label().to_string())
            .or_insert(0) += 1;

        *family_counts
            .entry(backend_family_label(&entry.metadata.family))
            .or_insert(0) += 1;
        deterministic_count += usize::from(entry.metadata.deterministic);
        offline_count += usize::from(entry.metadata.offline);
        batch_count += usize::from(entry.metadata.capabilities.batch);
        native_sidereal_count += usize::from(entry.metadata.capabilities.native_sidereal);
        if entry.metadata.nominal_range.start.is_some()
            || entry.metadata.nominal_range.end.is_some()
        {
            bounded_nominal_range_count += 1;
        } else {
            open_ended_nominal_range_count += 1;
        }
        match entry.metadata.accuracy {
            AccuracyClass::Exact => exact_accuracy_count += 1,
            AccuracyClass::High => high_accuracy_count += 1,
            AccuracyClass::Moderate => moderate_accuracy_count += 1,
            AccuracyClass::Approximate => approximate_accuracy_count += 1,
            AccuracyClass::Unknown => unknown_accuracy_count += 1,
            _ => unknown_accuracy_count += 1,
        }
        if selected_asteroid_coverage(&entry.metadata.body_coverage).is_some() {
            selected_asteroid_count += 1;
        }
        if !entry.metadata.provenance.data_sources.is_empty() {
            data_source_count += 1;
        }
        for body in &entry.metadata.body_coverage {
            push_unique(&mut bodies, body.to_string());
        }
        for frame in &entry.metadata.supported_frames {
            push_unique(&mut frames, frame.to_string());
        }
        for scale in &entry.metadata.supported_time_scales {
            push_unique(&mut time_scales, scale.to_string());
        }
    }

    let mut family_entries = family_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    family_entries.sort();

    let mut status_entries = status_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    status_entries.sort();

    let mut text = String::new();
    text.push_str("Backend matrix summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Backends: ");
    text.push_str(&catalog.len().to_string());
    text.push('\n');
    text.push_str("Families: ");
    text.push_str(&family_entries.join(", "));
    text.push('\n');
    text.push_str("Implementation statuses: ");
    text.push_str(&status_entries.join(", "));
    text.push('\n');
    text.push_str("Deterministic backends: ");
    text.push_str(&deterministic_count.to_string());
    text.push('\n');
    text.push_str("Offline backends: ");
    text.push_str(&offline_count.to_string());
    text.push('\n');
    text.push_str("Batch-capable backends: ");
    text.push_str(&batch_count.to_string());
    text.push('\n');
    text.push_str("Native sidereal backends: ");
    text.push_str(&native_sidereal_count.to_string());
    text.push('\n');
    text.push_str(&native_sidereal_posture_line(native_sidereal_count));
    text.push('\n');
    text.push_str("Nominal ranges: bounded: ");
    text.push_str(&bounded_nominal_range_count.to_string());
    text.push_str(", open-ended: ");
    text.push_str(&open_ended_nominal_range_count.to_string());
    text.push('\n');
    text.push_str("Accuracy classes: Exact: ");
    text.push_str(&exact_accuracy_count.to_string());
    text.push_str(", High: ");
    text.push_str(&high_accuracy_count.to_string());
    text.push_str(", Moderate: ");
    text.push_str(&moderate_accuracy_count.to_string());
    text.push_str(", Approximate: ");
    text.push_str(&approximate_accuracy_count.to_string());
    text.push_str(", Unknown: ");
    text.push_str(&unknown_accuracy_count.to_string());
    text.push('\n');
    text.push_str("Backends with selected asteroid coverage: ");
    text.push_str(&selected_asteroid_count.to_string());
    text.push('\n');
    text.push_str(&selected_asteroid_source_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_terminal_boundary_summary_for_report());
    text.push('\n');
    text.push_str("Comparison corpus release-grade guard: ");
    match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(summary) => text.push_str(summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Reference/hold-out overlap: ");
    text.push_str(&render_reference_holdout_overlap_summary_text());
    text.push('\n');
    text.push_str("JPL independent hold-out: ");
    text.push_str(&jpl_independent_holdout_summary_for_report());
    text.push('\n');
    text.push_str("Release-grade body claims: ");
    text.push_str(&format_release_body_claims_summary_for_report());
    text.push('\n');
    text.push_str("Body/date/channel claims: ");
    text.push_str(&format_body_date_channel_claims_summary_for_report());
    text.push('\n');
    text.push_str("Source corpus: ");
    text.push_str(&source_corpus_summary_for_report());
    text.push('\n');
    text.push_str("Source corpus posture: ");
    text.push_str(&source_corpus_posture_summary_for_report());
    text.push('\n');
    text.push_str("JPL source corpus contract: ");
    match required_labelled_summary_payload(
        jpl_source_corpus_contract_summary_for_report(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    ) {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Catalog posture: ");
    match core_validated_catalog_posture_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Target house scope: ");
    match core_validated_target_house_scope_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Target ayanamsa scope: ");
    match core_validated_target_ayanamsa_scope_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Pluto fallback: ");
    match validated_pluto_fallback_summary_line_for_report() {
        Ok(summary) => text.push_str(summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("House code aliases: ");
    match validated_house_code_aliases_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_boundary_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_sparse_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_pre_bridge_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1500_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1600_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1750_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2360234_major_body_interior_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1900_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_early_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1800_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2500_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str(&validated_production_generation_manifest_summary_text_for_report());
    text.push('\n');
    text.push_str("Production generation source revision: ");
    match validated_production_generation_source_revision_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Production generation source: ");
    text.push_str(&production_generation_source_summary_for_report());
    text.push('\n');
    text.push_str("Production generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&production_generation_snapshot_window_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation body-class coverage: ");
    text.push_str(&validated_production_generation_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation corpus shape: ");
    text.push_str(&production_generation_corpus_shape_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary request corpus equatorial: ");
    text.push_str(&production_generation_boundary_request_corpus_equatorial_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_source_corpus_contract_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&format_comparison_snapshot_manifest_summary());
    text.push('\n');
    if let Ok(report) = build_validation_report(SUMMARY_BENCHMARK_ROUNDS) {
        text.push_str("Comparison audit: compare-backends-audit; ");
        text.push_str(&comparison_audit_summary_for_report(&report.comparison));
        text.push('\n');
    }
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text.push_str(&request_surface_summary_for_report());
    text.push('\n');
    text.push_str("Frame policy: ");
    text.push_str(&validated_frame_policy_summary_for_report());
    text.push('\n');
    text.push_str("Mean-obliquity frame round-trip: ");
    text.push_str(&mean_obliquity_frame_round_trip_summary_for_report());
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&validated_zodiac_policy_summary_for_report());
    text.push('\n');
    text.push_str("Backends with external data sources: ");
    text.push_str(&data_source_count.to_string());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_health_summary());
    text.push('\n');
    text.push_str(&format_vsop87_frame_treatment_summary());
    text.push('\n');
    text.push_str("VSOP87 request policy: ");
    text.push_str(&format_vsop87_request_policy_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_audit_summary());
    text.push('\n');
    text.push_str(&generated_binary_audit_summary_for_report());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_outlier_note_summary());
    text.push('\n');
    text.push_str(&format_vsop87_equatorial_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j2000_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_mixed_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j1900_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_body_evidence_summary());
    text.push('\n');
    text.push_str(&lunar_theory_catalog_summary_for_report());
    text.push('\n');
    text.push_str(&validated_lunar_theory_catalog_validation_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_source_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference\n");
    text.push_str(&lunar_equatorial_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference batch parity\n");
    text.push_str(&lunar_equatorial_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_equatorial_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar source windows: ");
    text.push_str(&lunar_source_window_summary_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature continuity evidence\n");
    text.push_str(&lunar_high_curvature_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature equatorial continuity evidence\n");
    text.push_str(&lunar_high_curvature_equatorial_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Distinct bodies covered: ");
    text.push_str(&bodies.len().to_string());
    text.push_str(" (");
    text.push_str(&bodies.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct coordinate frames: ");
    text.push_str(&frames.len().to_string());
    text.push_str(" (");
    text.push_str(&frames.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct time scales: ");
    text.push_str(&time_scales.len().to_string());
    text.push_str(" (");
    text.push_str(&time_scales.join(", "));
    text.push_str(")\n");
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str("Time-scale policy: ");
    text.push_str(&format_time_scale_policy_summary_for_report(
        &time_scale_policy,
    ));
    text.push('\n');
    text.push_str("Delta T policy: ");
    text.push_str(&format_delta_t_policy_summary_for_report(
        &delta_t_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Observer policy: ");
    text.push_str(&format_observer_policy_summary_for_report(
        &pleiades_backend::observer_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Apparentness policy: ");
    text.push_str(&format_apparentness_policy_summary_for_report(
        &pleiades_backend::apparentness_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Native sidereal policy: ");
    text.push_str(&pleiades_backend::validated_native_sidereal_policy_summary_for_report());
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&validated_zodiac_policy_summary_for_report());
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&validated_release_profile_identifiers_summary_for_report(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

/// Renders a compact summary of the API stability posture.
pub fn render_api_stability_summary() -> String {
    render_api_stability_summary_text()
}

fn render_api_stability_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("API stability summary unavailable ({error})"),
    };

    match validated_api_stability_profile_for_report() {
        Ok(profile) => {
            let mut text = String::new();

            text.push_str("API stability summary\n");
            text.push_str("Profile: ");
            text.push_str(profile.profile_id);
            text.push('\n');
            text.push_str("Summary line: ");
            text.push_str(&profile.summary_line());
            text.push('\n');
            text.push_str("Compatibility profile: ");
            text.push_str(release_profiles.compatibility_profile_id);
            text.push('\n');
            text.push_str("Release profile identifiers: ");
            text.push_str(&validated_release_profile_identifiers_summary_for_report(
                &release_profiles,
            ));
            text.push('\n');
            text.push_str("Stable surfaces: ");
            text.push_str(&profile.stable_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Experimental surfaces: ");
            text.push_str(&profile.experimental_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Deprecation policy items: ");
            text.push_str(&profile.deprecation_policy.len().to_string());
            text.push('\n');
            text.push_str("Intentional limits: ");
            text.push_str(&profile.intentional_limits.len().to_string());
            text.push('\n');
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("Backend matrix summary: backend-matrix-summary\n");
            text.push_str("Release notes summary: release-notes-summary\n");
            text.push_str("Release checklist summary: release-checklist-summary\n");
            text.push_str("Release bundle verification: verify-release-bundle\n");
            text.push_str("See release-summary for the compact one-screen release overview.\n");

            text
        }
        Err(error) => format!("API stability summary unavailable ({error})"),
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

pub(crate) fn backend_family_label(family: &BackendFamily) -> String {
    family.to_string()
}

/// Renders a backend capability matrix for the implemented backend catalog.
pub fn render_backend_matrix_report() -> Result<String, EphemerisError> {
    validated_compatibility_profile_for_report().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("backend capability matrix unavailable ({error})"),
        )
    })?;
    let mut rendered = String::new();
    fmt::write(
        &mut rendered,
        format_args!("Implemented backend matrices\n\n"),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    let house_code_aliases =
        validated_house_code_aliases_summary_for_report().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("backend capability matrix unavailable ({error})"),
            )
        })?;

    fmt::write(
        &mut rendered,
        format_args!("House code aliases: {}\n\n", house_code_aliases),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    fmt::write(
        &mut rendered,
        format_args!(
            "Body/date/channel claims: {}\n\n",
            format_body_date_channel_claims_summary_for_report()
        ),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    for entry in implemented_backend_catalog() {
        validate_backend_matrix_entry(&entry)?;
        fmt::write(&mut rendered, format_args!("{}\n", entry.label)).map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
        fmt::write(
            &mut rendered,
            format_args!("{}\n\n", BackendMatrixDisplay(&entry)),
        )
        .map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
    }

    Ok(rendered)
}

fn validate_backend_matrix_entry(entry: &BackendMatrixEntry) -> Result<(), EphemerisError> {
    entry.metadata.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "backend matrix entry `{}` has invalid metadata: {error}",
                entry.label
            ),
        )
    })
}

struct BackendMatrixDisplay<'a>(&'a BackendMatrixEntry);

impl fmt::Display for BackendMatrixDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_backend_catalog_entry(f, self.0)
    }
}

fn write_corpus_summary(f: &mut fmt::Formatter<'_>, corpus: &CorpusSummary) -> fmt::Result {
    if let Err(error) = corpus.validate() {
        writeln!(f, "  corpus summary unavailable ({error})")?;
        return Ok(());
    }

    writeln!(f, "  name: {}", corpus.name)?;
    writeln!(f, "  description: {}", corpus.description)?;
    writeln!(f, "  Apparentness: {}", corpus.apparentness)?;
    writeln!(f, "  requests: {}", corpus.request_count)?;
    writeln!(f, "  epochs: {}", corpus.epoch_count)?;
    writeln!(f, "  epoch labels: {}", format_instant_list(&corpus.epochs))?;
    writeln!(f, "  bodies: {}", corpus.body_count)?;
    writeln!(
        f,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    )
}

fn write_corpus_summary_text(text: &mut String, corpus: &CorpusSummary) {
    use std::fmt::Write as _;

    if let Err(error) = corpus.validate() {
        let _ = writeln!(text, "  corpus summary unavailable ({error})");
        return;
    }

    let _ = writeln!(text, "  name: {}", corpus.name);
    let _ = writeln!(text, "  description: {}", corpus.description);
    let _ = writeln!(text, "  Apparentness: {}", corpus.apparentness);
    let _ = writeln!(text, "  requests: {}", corpus.request_count);
    let _ = writeln!(text, "  epochs: {}", corpus.epoch_count);
    let _ = writeln!(
        text,
        "  epoch labels: {}",
        format_instant_list(&corpus.epochs)
    );
    let _ = writeln!(text, "  bodies: {}", corpus.body_count);
    let _ = writeln!(
        text,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    );
}

fn write_backend_matrix(f: &mut fmt::Formatter<'_>, backend: &BackendMetadata) -> fmt::Result {
    writeln!(
        f,
        "  summary: {}",
        backend
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    writeln!(f, "  id: {}", backend.id)?;
    writeln!(f, "  version: {}", backend.version)?;
    writeln!(f, "  family: {}", backend.family)?;
    writeln!(f, "  family posture: {}", backend.family.posture())?;
    writeln!(f, "  accuracy: {}", backend.accuracy)?;
    writeln!(f, "  deterministic: {}", backend.deterministic)?;
    writeln!(f, "  offline: {}", backend.offline)?;
    writeln!(f, "  nominal range: {}", backend.nominal_range)?;
    writeln!(
        f,
        "  time scales: {}",
        format_time_scales(&backend.supported_time_scales)
    )?;
    writeln!(f, "  bodies: {}", format_bodies(&backend.body_coverage))?;
    if let Some(asteroids) = selected_asteroid_coverage(&backend.body_coverage) {
        writeln!(
            f,
            "  {}",
            selected_asteroid_coverage_summary_for_report(&asteroids)
        )?;
        if backend.id.as_str() == "jpl-snapshot" {
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_evidence_summary_for_report()
            )?;
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_window_summary_for_report()
            )?;
            writeln!(f, "  {}", selected_asteroid_boundary_summary_for_report())?;
            writeln!(f, "  {}", selected_asteroid_bridge_summary_for_report())?;
            let evidence = reference_asteroid_evidence();
            if let Some(first) = evidence.first() {
                writeln!(
                    f,
                    "  exact J2000 evidence: {} bodies at JD {:.1}",
                    evidence.len(),
                    first.epoch.julian_day.days()
                )?;
                for sample in evidence {
                    writeln!(
                        f,
                        "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                        sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                    )?;
                }
            }
            writeln!(
                f,
                "  {}",
                reference_snapshot_exact_j2000_evidence_summary_for_report()
            )?;
            writeln!(
                f,
                "  {}",
                reference_snapshot_major_body_bridge_summary_for_report()
            )?;
        }
    }
    writeln!(f, "  frames: {}", format_frames(&backend.supported_frames))?;
    writeln!(
        f,
        "  capabilities: {}",
        format_capabilities(&backend.capabilities)
    )?;
    writeln!(
        f,
        "  provenance: {}",
        backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    if !backend.provenance.data_sources.is_empty() {
        writeln!(
            f,
            "  provenance sources: {}",
            backend.provenance.data_sources.join("; ")
        )?;
    }
    Ok(())
}

fn write_backend_catalog_entry(
    f: &mut fmt::Formatter<'_>,
    entry: &BackendMatrixEntry,
) -> fmt::Result {
    write_backend_matrix(f, &entry.metadata)?;
    writeln!(
        f,
        "  implementation status: {}",
        entry.implementation_status.label()
    )?;
    writeln!(f, "  implementation note: {}", entry.status_note)?;
    if entry.metadata.id.as_str() == "pleiades-vsop87" {
        writeln!(f, "  body source profiles:")?;
        for profile in body_source_profiles() {
            writeln!(f, "    {}", profile.summary_line())?;
        }

        writeln!(f, "  source documentation:")?;
        for spec in source_specifications() {
            writeln!(
                f,
                "    {}: {} {} | {} | {} | {} | {} | {} | {} | {}",
                spec.body,
                spec.variant,
                spec.source_file,
                spec.coordinate_family,
                spec.frame,
                spec.units,
                spec.reduction,
                spec.transform_note,
                spec.truncation_policy,
                spec.date_range
            )?;
        }

        writeln!(f, "  source audit:")?;
        for audit in source_audits() {
            writeln!(
                f,
                "    {}: {} bytes, {} lines, {} terms, 0x{:016x}",
                audit.body,
                audit.byte_length,
                audit.line_count,
                audit.term_count,
                audit.fingerprint
            )?;
        }

        writeln!(f, "  generated binary audit:")?;
        writeln!(f, "    {}", generated_binary_audit_summary_for_report())?;

        writeln!(f, "  canonical J2000 VSOP87B evidence:")?;
        match vsop87_canonical_body_evidence() {
            Some(body_evidence) => {
                for evidence in body_evidence {
                    writeln!(
                        f,
                        "    {}: kind={} from {} — {} — Δlon={:.12}° / limit {:.12}° / margin {:+.12}°, Δlat={:.12}° / limit {:.12}° / margin {:+.12}°, Δdist={:.12} AU / limit {:.12} AU / margin {:+.12} AU",
                        evidence.body,
                        evidence.source_kind,
                        evidence.source_file,
                        if evidence.within_interim_limits {
                            evidence.provenance
                        } else {
                            "outside interim limits"
                        },
                        evidence.longitude_delta_deg,
                        evidence.longitude_limit_deg,
                        evidence.longitude_limit_deg - evidence.longitude_delta_deg,
                        evidence.latitude_delta_deg,
                        evidence.latitude_limit_deg,
                        evidence.latitude_limit_deg - evidence.latitude_delta_deg,
                        evidence.distance_delta_au,
                        evidence.distance_limit_au,
                        evidence.distance_limit_au - evidence.distance_delta_au
                    )?;
                }
            }
            None => {
                writeln!(f, "    unavailable")?;
            }
        }
        writeln!(
            f,
            "  body profile evidence summary: {}",
            format_vsop87_body_evidence_summary()
        )?;
    } else if entry.metadata.id.as_str() == "pleiades-elp" {
        let theory = lunar_theory_specification();
        writeln!(f, "  lunar theory specification:")?;
        writeln!(
            f,
            "    catalog summary: {}",
            lunar_theory_catalog_summary_for_report()
        )?;
        writeln!(
            f,
            "    catalog validation: {}",
            validated_lunar_theory_catalog_validation_summary_for_report()
        )?;
        writeln!(f, "    model: {}", theory.model_name)?;
        writeln!(
            f,
            "    source family: {}",
            pleiades_elp::lunar_theory_source_family().label()
        )?;
        writeln!(
            f,
            "    capability summary: {}",
            lunar_theory_capability_summary_for_report()
        )?;
        writeln!(
            f,
            "    specification summary: {}",
            lunar_theory_summary_for_report()
        )?;
        writeln!(f, "    source identifier: {}", theory.source_identifier)?;
        writeln!(f, "    source citation: {}", theory.source_citation)?;
        writeln!(f, "    source material: {}", theory.source_material)?;
        writeln!(f, "    redistribution note: {}", theory.redistribution_note)?;
        writeln!(f, "    license note: {}", theory.license_note)?;
        writeln!(
            f,
            "    supported bodies: {}",
            format_bodies(theory.supported_bodies)
        )?;
        writeln!(
            f,
            "    unsupported bodies: {}",
            format_bodies(theory.unsupported_bodies)
        )?;
        writeln!(
            f,
            "    request policy: {}",
            lunar_theory_request_policy_summary()
        )?;
        writeln!(f, "    validation window: {}", theory.validation_window)?;
        writeln!(f, "    date-range note: {}", theory.date_range_note)?;
        writeln!(f, "    frame note: {}", theory.frame_note)?;
        write_lunar_reference_evidence(f)?;
        write_lunar_equatorial_reference_evidence(f)?;
        write_lunar_apparent_comparison_evidence(f)?;
        write_lunar_source_window_evidence(f)?;
        writeln!(f, "  Lunar high-curvature continuity evidence:")?;
        writeln!(
            f,
            "    {}",
            lunar_high_curvature_continuity_evidence_for_report()
        )?;
        write_lunar_high_curvature_equatorial_continuity_evidence(f)?;
    }
    if entry.metadata.id.as_str() == "jpl-snapshot" {
        write_jpl_interpolation_quality(f)?;
        writeln!(
            f,
            "    {}",
            jpl_snapshot_batch_error_taxonomy_summary_for_report()
        )?;
    }
    writeln!(
        f,
        "  expected error classes: {}",
        format_error_kinds(entry.expected_error_kinds)
    )?;
    if entry.required_data_files.is_empty() {
        writeln!(f, "  required external data files: none")?;
    } else {
        writeln!(
            f,
            "  required external data files: {}",
            format_data_files(entry.required_data_files)
        )?;
    }
    Ok(())
}

fn write_jpl_interpolation_quality(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  interpolation quality checks:")?;
    let Some(summary) = jpl_interpolation_quality_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        format_jpl_interpolation_quality_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        jpl_interpolation_quality_kind_coverage_for_report()
    )?;
    writeln!(f, "    {}", jpl_interpolation_posture_summary_for_report())?;
    writeln!(f, "    {}", jpl_independent_holdout_summary_for_report())?;
    writeln!(f, "    {}", render_reference_holdout_overlap_summary_text())?;
    writeln!(
        f,
        "    {}",
        independent_holdout_snapshot_body_class_coverage_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        independent_holdout_snapshot_batch_parity_summary_text()
    )?;
    writeln!(
        f,
        "    {}",
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    )?;
    for sample in interpolation_quality_samples() {
        writeln!(f, "    {}", sample.summary_line())?;
    }
    writeln!(
        f,
        "    {}",
        jpl_interpolation_body_class_error_envelopes_for_report()
    )?;
    Ok(())
}

fn jpl_interpolation_quality_summary() -> Option<pleiades_jpl::JplInterpolationQualitySummary> {
    pleiades_jpl::jpl_interpolation_quality_summary()
}

fn format_jpl_interpolation_quality_summary(
    summary: &pleiades_jpl::JplInterpolationQualitySummary,
) -> String {
    pleiades_jpl::format_jpl_interpolation_quality_summary(summary)
}

fn write_lunar_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar reference:")?;
    let Some(summary) = lunar_reference_evidence_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        pleiades_elp::format_lunar_reference_evidence_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        pleiades_elp::lunar_reference_batch_parity_summary_for_report()
    )?;
    writeln!(f, "    {}", lunar_reference_evidence_envelope_for_report())?;
    for sample in lunar_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

fn write_lunar_equatorial_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar equatorial reference:")?;
    if lunar_equatorial_reference_evidence_summary().is_none() {
        writeln!(f, "    none")?;
        return Ok(());
    }

    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_evidence_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_batch_parity_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_evidence_envelope_for_report()
    )?;
    for sample in lunar_equatorial_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

fn write_lunar_apparent_comparison_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar apparent comparison:")?;
    let Some(summary) = lunar_apparent_comparison_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(f, "    {}", summary.summary_line())?;
    for sample in lunar_apparent_comparison_evidence() {
        writeln!(
            f,
            "    {} at JD {:.1}: apparent lon={:.12}°, apparent lat={:.12}°, apparent dist={:.12} AU, apparent RA={:.12}°, apparent Dec={:.12}°, note={}",
            sample.body,
            sample.epoch.julian_day.days(),
            sample.apparent_longitude_deg,
            sample.apparent_latitude_deg,
            sample.apparent_distance_au,
            sample.apparent_right_ascension_deg,
            sample.apparent_declination_deg,
            sample.note
        )?;
    }
    Ok(())
}

fn write_lunar_source_window_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar source windows:")?;
    writeln!(f, "    {}", lunar_source_window_summary_for_report())?;
    Ok(())
}

fn write_lunar_high_curvature_equatorial_continuity_evidence(
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(f, "  Lunar high-curvature equatorial continuity evidence:")?;
    writeln!(
        f,
        "    {}",
        lunar_high_curvature_equatorial_continuity_evidence_for_report()
    )?;
    Ok(())
}

fn write_comparison_summary(f: &mut fmt::Formatter<'_>, report: &ComparisonReport) -> fmt::Result {
    let summary = &report.summary;
    let comparison_envelope = comparison_envelope_summary(summary, &report.samples);
    let median = comparison_envelope.median;

    writeln!(f, "  samples: {}", summary.sample_count)?;
    writeln!(
        f,
        "  max longitude delta: {:.12}°",
        summary.max_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean longitude delta: {:.12}°",
        summary.mean_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  median longitude delta: {:.12}°",
        median.longitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms longitude delta: {:.12}°",
        summary.rms_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  max latitude delta: {:.12}°",
        summary.max_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean latitude delta: {:.12}°",
        summary.mean_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  median latitude delta: {:.12}°",
        median.latitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms latitude delta: {:.12}°",
        summary.rms_latitude_delta_deg
    )?;
    if let Some(value) = summary.max_distance_delta_au {
        writeln!(f, "  max distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.mean_distance_delta_au {
        writeln!(f, "  mean distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = median.distance_delta_au {
        writeln!(f, "  median distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.rms_distance_delta_au {
        writeln!(f, "  rms distance delta: {:.12} AU", value)?;
    }
    match comparison_envelope.validated_percentile_line(&report.samples) {
        Ok(line) => writeln!(f, "  {line}")?,
        Err(error) => writeln!(f, "  comparison percentile envelope unavailable ({error})")?,
    }
    Ok(())
}

fn format_body_comparison_summary_for_report(summary: &BodyComparisonSummary) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!(
            "body comparison summary for {} unavailable ({error})",
            summary.body
        ),
    }
}

fn write_body_comparison_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyComparisonSummary],
) -> fmt::Result {
    writeln!(f, "Body comparison summaries")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        writeln!(
            f,
            "  {}",
            format_body_comparison_summary_for_report(summary)
        )?;
    }
    Ok(())
}

fn write_body_class_envelopes(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
) -> fmt::Result {
    writeln!(f, "Body-class error envelopes")?;
    let summaries = body_class_summaries(samples);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

fn write_body_class_tolerance_posture(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
    backend_family: &BackendFamily,
) -> fmt::Result {
    writeln!(f, "Body-class tolerance posture")?;
    let summaries = body_class_tolerance_summaries(samples, backend_family);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

pub(crate) fn tolerance_backend_family_label(family: &BackendFamily) -> String {
    match family {
        BackendFamily::Algorithmic => "algorithmic".to_string(),
        BackendFamily::ReferenceData => "reference data".to_string(),
        BackendFamily::CompressedData => "compressed data".to_string(),
        BackendFamily::Composite => "composite".to_string(),
        BackendFamily::Other(value) => format!("other ({value})"),
        _ => "other (unknown)".to_string(),
    }
}

fn write_tolerance_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyToleranceSummary],
) -> fmt::Result {
    writeln!(f, "Expected tolerance status")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        match summary.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(
                f,
                "  body tolerance summary for {} unavailable ({error})",
                summary.body
            ),
        }?;
    }
    Ok(())
}

fn write_regression_section(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    findings: &[RegressionFinding],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    if findings.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in findings {
        match finding.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(f, "  regression finding unavailable ({error})"),
        }?;
    }
    Ok(())
}

fn write_regression_archive_section(
    f: &mut fmt::Formatter<'_>,
    archive: &RegressionArchive,
) -> fmt::Result {
    writeln!(f, "Archived regression cases")?;
    writeln!(f, "  corpus: {}", archive.corpus_name)?;
    if archive.cases.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in &archive.cases {
        match finding.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(f, "  regression finding unavailable ({error})"),
        }?;
    }
    Ok(())
}

fn write_reference_asteroid_section(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Selected asteroid coverage")?;
    let asteroids = reference_asteroids();
    if asteroids.is_empty() {
        writeln!(f, "  none")?;
    } else {
        writeln!(
            f,
            "  {}",
            selected_asteroid_coverage_summary_for_report(asteroids)
        )?;
        let evidence = reference_asteroid_evidence();
        if evidence.is_empty() {
            writeln!(f, "  exact J2000 evidence: unavailable")?;
        } else {
            writeln!(
                f,
                "  exact J2000 evidence: {} bodies at JD {:.1}",
                evidence.len(),
                evidence[0].epoch.julian_day.days()
            )?;
            for sample in evidence {
                writeln!(
                    f,
                    "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                    sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                )?;
            }
        }
        writeln!(
            f,
            "  note: comparison reports stay on the planetary subset while the JPL snapshot preserves selected asteroid coverage."
        )?;
    }
    Ok(())
}

fn regression_finding(
    sample: &ComparisonSample,
    backend_family: &BackendFamily,
) -> Option<RegressionFinding> {
    let tolerance = comparison_tolerance_for_body(&sample.body, backend_family);
    let mut notes = Vec::new();
    if sample.longitude_delta_deg >= tolerance.max_longitude_delta_deg {
        notes.push(format!(
            "longitude delta exceeds {:.1}°",
            tolerance.max_longitude_delta_deg
        ));
    }
    if sample.latitude_delta_deg >= tolerance.max_latitude_delta_deg {
        notes.push(format!(
            "latitude delta exceeds {:.2}°",
            tolerance.max_latitude_delta_deg
        ));
    }
    if sample
        .distance_delta_au
        .is_some_and(|value| value >= tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY))
    {
        notes.push(format!(
            "distance delta exceeds {:.3} AU",
            tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY)
        ));
    }

    if notes.is_empty() {
        return None;
    }

    Some(RegressionFinding {
        body: sample.body.clone(),
        longitude_delta_deg: sample.longitude_delta_deg,
        latitude_delta_deg: sample.latitude_delta_deg,
        distance_delta_au: sample.distance_delta_au,
        note: notes.join(", "),
    })
}

const JPL_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
];
const JPL_REQUIRED_DATA_FILES: &[&str] = &[
    "crates/pleiades-jpl/data/reference_snapshot.csv",
    "crates/pleiades-jpl/data/j2000_snapshot.csv",
];
const VSOP87_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
const ELP_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
const PACKAGED_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];
const COMPOSITE_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];

fn implemented_backend_catalog() -> Vec<BackendMatrixEntry> {
    vec![
        BackendMatrixEntry {
            label: "JPL snapshot reference backend",
            metadata: default_reference_backend().metadata(),
            implementation_status: BackendImplementationStatus::FixtureReference,
            status_note: "checked-in public-input derivative fixture with exact lookup and cubic interpolation on four-sample windows when available, with quadratic and linear fallbacks for sparser bodies; reference corpus now spans 357 rows across 16 bodies and 31 epochs with expanded bridge and boundary coverage, while the broader production reader remains planned",
            expected_error_kinds: JPL_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
        BackendMatrixEntry {
            label: "VSOP87 planetary backend",
            metadata: Vsop87Backend::new().metadata(),
            implementation_status: BackendImplementationStatus::PartialSourceBacked,
            status_note: "Sun through Neptune now use generated binary VSOP87B source tables derived from the vendored full-file inputs, and Pluto remains the current approximate mean-element fallback special case until a Pluto-specific source path is selected",
            expected_error_kinds: VSOP87_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "ELP lunar backend (Moon and lunar nodes)",
            metadata: ElpBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
            status_note: "compact lunar and lunar-point formulas provide the current deterministic baseline while documented production lunar-theory ingestion remains open",
            expected_error_kinds: ELP_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Packaged data backend",
            metadata: PackagedDataBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::DraftArtifact,
            status_note: "sample packaged artifact exercises lookup and profile plumbing; generated 1500-2500 production artifacts are Phase 2 work",
            expected_error_kinds: PACKAGED_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Composite routed backend",
            metadata: default_candidate_backend().metadata(),
            implementation_status: BackendImplementationStatus::RoutingFacade,
            status_note: "routes current planetary and lunar implementations for chart-facing validation without increasing underlying backend accuracy claims",
            expected_error_kinds: COMPOSITE_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
    ]
}

struct BackendMatrixEntry {
    label: &'static str,
    metadata: BackendMetadata,
    implementation_status: BackendImplementationStatus,
    status_note: &'static str,
    expected_error_kinds: &'static [EphemerisErrorKind],
    required_data_files: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BackendImplementationStatus {
    FixtureReference,
    PartialSourceBacked,
    PreliminaryAlgorithm,
    DraftArtifact,
    RoutingFacade,
}

impl BackendImplementationStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::FixtureReference => "fixture-reference",
            Self::PartialSourceBacked => "partial-source-backed",
            Self::PreliminaryAlgorithm => "preliminary-algorithm",
            Self::DraftArtifact => "draft-artifact",
            Self::RoutingFacade => "routing-facade",
        }
    }
}

fn write_backend_catalog(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    catalog: &[BackendMatrixEntry],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for entry in catalog {
        writeln!(f, "{}", entry.label)?;
        write_backend_catalog_entry(f, entry)?;
        writeln!(f)?;
    }
    Ok(())
}

pub(crate) fn format_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(|body| body.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn selected_asteroid_coverage(bodies: &[CelestialBody]) -> Option<Vec<CelestialBody>> {
    let asteroids = bodies
        .iter()
        .filter(|body| is_selected_asteroid(body))
        .cloned()
        .collect::<Vec<_>>();

    if asteroids.is_empty() {
        None
    } else {
        Some(asteroids)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectedAsteroidCoverageSummary {
    body_count: usize,
    bodies: Vec<CelestialBody>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SelectedAsteroidCoverageSummaryValidationError {
    MissingBodies,
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    UnsupportedBody {
        index: usize,
        body: String,
    },
}

impl fmt::Display for SelectedAsteroidCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBodies => f.write_str("missing bodies"),
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match body list length {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(f, "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"),
            Self::UnsupportedBody { index, body } => write!(f, "body '{body}' at index {index} is not a selected asteroid"),
        }
    }
}

impl std::error::Error for SelectedAsteroidCoverageSummaryValidationError {}

impl SelectedAsteroidCoverageSummary {
    fn summary_line(&self) -> String {
        format!(
            "selected asteroid coverage: {} bodies ({})",
            self.body_count,
            format_bodies(&self.bodies)
        )
    }

    fn validate(&self) -> Result<(), SelectedAsteroidCoverageSummaryValidationError> {
        if self.body_count == 0 || self.bodies.is_empty() {
            return Err(SelectedAsteroidCoverageSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                SelectedAsteroidCoverageSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }
        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    SelectedAsteroidCoverageSummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .expect("duplicate body should have a first index"),
                        second_index: index,
                        body: body.to_string(),
                    },
                );
            }
            if !is_selected_asteroid(body) {
                return Err(
                    SelectedAsteroidCoverageSummaryValidationError::UnsupportedBody {
                        index,
                        body: body.to_string(),
                    },
                );
            }
        }

        Ok(())
    }

    fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

fn selected_asteroid_coverage_summary(
    bodies: &[CelestialBody],
) -> Option<SelectedAsteroidCoverageSummary> {
    selected_asteroid_coverage(bodies).map(|bodies| SelectedAsteroidCoverageSummary {
        body_count: bodies.len(),
        bodies,
    })
}

fn selected_asteroid_coverage_summary_for_report(bodies: &[CelestialBody]) -> String {
    match selected_asteroid_coverage_summary(bodies) {
        Some(summary) => summary
            .validated_summary_line()
            .unwrap_or_else(|error| format!("selected asteroid coverage: unavailable ({error})")),
        None => "selected asteroid coverage: unavailable".to_string(),
    }
}

fn is_selected_asteroid(body: &CelestialBody) -> bool {
    match body {
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => true,
        CelestialBody::Custom(custom) => custom.catalog == "asteroid",
        _ => false,
    }
}

pub(crate) fn format_frames(frames: &[CoordinateFrame]) -> String {
    frames
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_time_scales(scales: &[TimeScale]) -> String {
    scales
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_capabilities(capabilities: &BackendCapabilities) -> String {
    match capabilities.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("unavailable ({error})"),
    }
}

fn format_error_kinds(kinds: &[EphemerisErrorKind]) -> String {
    kinds
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_data_files(files: &[&str]) -> String {
    files.join("; ")
}

fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

fn format_instant_list(instants: &[Instant]) -> String {
    if instants.is_empty() {
        return "none".to_string();
    }

    instants
        .iter()
        .copied()
        .map(format_instant)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_ns(value: f64) -> String {
    format!("{value:.2}")
}

fn format_duration(duration: std::time::Duration) -> String {
    format!("{:.6}s", duration.as_secs_f64())
}

#[cfg(test)]
mod tests;
