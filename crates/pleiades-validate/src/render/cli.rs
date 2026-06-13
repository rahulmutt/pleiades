//! CLI command dispatch and argument parsing for the validation tool.

use super::text::*;
use crate::*;

/// Returns the CLI banner.
pub fn banner() -> &'static str {
    BANNER
}

fn validate_release_smoke_at(output_dir: impl AsRef<Path>) -> Result<(), String> {
    struct Cleanup(PathBuf);

    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).map_err(|error| {
        format!(
            "release smoke working directory {} could not be created: {error}",
            output_dir.display()
        )
    })?;
    let _cleanup = Cleanup(output_dir.to_path_buf());

    let report = workspace_audit_report().map_err(|error| error.to_string())?;
    if !report.is_clean() {
        return Err(format!("release smoke failed:\n{report}"));
    }

    verify_compatibility_profile().map_err(render_error)?;
    let _ = render_artifact_report().map_err(render_artifact_error)?;
    let _ = render_release_bundle(1, output_dir).map_err(render_release_bundle_error)?;
    let _ = verify_release_bundle(output_dir).map_err(render_release_bundle_error)?;
    Ok(())
}

fn validate_release_smoke() -> Result<(), String> {
    let unique = format!(
        "pleiades-release-smoke-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos()
    );
    let output_dir = std::env::temp_dir().join(unique);
    validate_release_smoke_at(output_dir)
}

pub(crate) fn validate_release_gate_at(output_dir: impl AsRef<Path>) -> Result<(), String> {
    validate_release_smoke_at(output_dir).map_err(|error| format!("release gate failed: {error}"))
}

fn validate_release_gate() -> Result<(), String> {
    let unique = format!(
        "pleiades-release-gate-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos()
    );
    let output_dir = std::env::temp_dir().join(unique);
    validate_release_gate_at(output_dir)
}

/// Renders the command-line interface output.
pub fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compare-backends") => {
            ensure_no_extra_args(&args[1..], "compare-backends")?;
            render_comparison_report().map_err(render_error)
        }
        Some("comparison-report") => {
            ensure_no_extra_args(&args[1..], "comparison-report")?;
            render_comparison_report().map_err(render_error)
        }
        Some("compare-backends-audit") => {
            ensure_no_extra_args(&args[1..], "compare-backends-audit")?;
            render_comparison_audit_report()
        }
        Some("comparison-audit-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-audit-summary")?;
            render_comparison_audit_summary()
        }
        Some("comparison-audit") => {
            ensure_no_extra_args(&args[1..], "comparison-audit")?;
            render_comparison_audit_summary()
        }
        Some("backend-matrix") | Some("capability-matrix") => {
            ensure_no_extra_args(&args[1..], "backend-matrix")?;
            render_backend_matrix_report().map_err(render_error)
        }
        Some("backend-matrix-summary") | Some("matrix-summary") => {
            ensure_no_extra_args(&args[1..], "backend-matrix-summary")?;
            Ok(render_backend_matrix_summary())
        }
        Some("compatibility-profile") | Some("profile") => {
            ensure_no_extra_args(&args[1..], "compatibility-profile")?;
            validated_compatibility_profile_for_report().map(|profile| profile.to_string())
        }
        Some("benchmark") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_benchmark_report(rounds).map_err(render_error)
        }
        Some("benchmark-matrix") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_benchmark_matrix_summary(rounds).map_err(render_error)
        }
        Some("benchmark-matrix-summary") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_benchmark_matrix_summary(rounds).map_err(render_error)
        }
        Some("comparison-corpus-summary") | Some("comparison-corpus") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-summary")?;
            Ok(render_comparison_corpus_summary_text())
        }
        Some("comparison-corpus-release-guard-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-release-guard-summary")?;
            Ok(render_comparison_corpus_release_guard_summary_text())
        }
        Some("comparison-corpus-release-guard") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-release-guard")?;
            Ok(render_comparison_corpus_release_guard_summary_text())
        }
        Some("comparison-corpus-guard-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-guard-summary")?;
            Ok(render_comparison_corpus_release_guard_summary_text())
        }
        Some("comparison-corpus-guard") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-guard")?;
            Ok(render_comparison_corpus_release_guard_summary_text())
        }
        Some("benchmark-corpus-summary") => {
            ensure_no_extra_args(&args[1..], "benchmark-corpus-summary")?;
            Ok(render_benchmark_corpus_summary_text())
        }
        Some("chart-benchmark-corpus-summary") => {
            ensure_no_extra_args(&args[1..], "chart-benchmark-corpus-summary")?;
            Ok(render_chart_benchmark_corpus_summary_text())
        }
        Some("chart-benchmark-corpus") => {
            ensure_no_extra_args(&args[1..], "chart-benchmark-corpus")?;
            Ok(render_chart_benchmark_corpus_summary_text())
        }
        Some("report") | Some("generate-report") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_validation_report(rounds).map_err(render_error)
        }
        Some("validation-report-summary") | Some("report-summary") | Some("validation-summary") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_validation_report_summary(rounds).map_err(render_error)
        }
        Some("validate-artifact") => {
            ensure_no_extra_args(&args[1..], "validate-artifact")?;
            render_artifact_report().map_err(render_artifact_error)
        }
        Some("generate-packaged-artifact") | Some("regenerate-packaged-artifact") => {
            match parse_packaged_artifact_command(&args[1..])? {
                PackagedArtifactCommand::Write {
                    output_path,
                    manifest_path,
                    manifest_summary_path,
                    manifest_checksum_path,
                    artifact_checksum_path,
                    normalized_intermediate_path,
                } => render_packaged_artifact_regeneration(
                    output_path,
                    manifest_path,
                    manifest_summary_path,
                    manifest_checksum_path,
                    artifact_checksum_path,
                    normalized_intermediate_path,
                ),
                PackagedArtifactCommand::Check => render_packaged_artifact_regeneration_check(),
            }
        }
        Some("artifact-summary") | Some("artifact-posture-summary") => {
            ensure_no_extra_args(&args[1..], "artifact-summary")?;
            render_artifact_summary().map_err(render_artifact_error)
        }
        Some("artifact-boundary-envelope-summary") => {
            ensure_no_extra_args(&args[1..], "artifact-boundary-envelope-summary")?;
            artifact_boundary_envelope_summary_for_report()
                .map(|summary| summary.summary_line())
                .map_err(render_artifact_error)
        }
        Some("artifact-profile-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "artifact-profile-coverage-summary")?;
            Ok(format!(
                "Artifact profile coverage: {}",
                packaged_artifact_profile_coverage_summary_for_report()
            ))
        }
        Some("packaged-artifact-output-support-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-output-support-summary")?;
            Ok(format!(
                "Packaged-artifact output support: {}",
                packaged_artifact_output_support_summary_for_report()
            ))
        }
        Some("packaged-artifact-speed-policy-summary") | Some("packaged-artifact-speed-policy") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-speed-policy-summary")?;
            Ok(format!(
                "Packaged-artifact speed policy: {}",
                packaged_artifact_speed_policy_summary_for_report()
            ))
        }
        Some("motion-policy-summary") | Some("motion-policy") => {
            ensure_no_extra_args(&args[1..], "motion-policy-summary")?;
            Ok(validated_motion_policy_summary_for_report())
        }
        Some("packaged-artifact-output-support") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-output-support")?;
            Ok(format!(
                "Packaged-artifact output support: {}",
                packaged_artifact_output_support_summary_for_report()
            ))
        }
        Some("packaged-artifact-body-class-span-cap-summary")
        | Some("packaged-artifact-body-class-span-cap") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-body-class-span-cap-summary")?;
            Ok(format!(
                "Packaged-artifact body-class span caps: {}",
                packaged_artifact_body_class_span_cap_entries_for_report()
            ))
        }
        Some("packaged-artifact-access-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-access-summary")?;
            Ok(format!(
                "Packaged-artifact access: {}",
                packaged_artifact_access_summary_for_report()
            ))
        }
        Some("packaged-artifact-access") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-access")?;
            Ok(format!(
                "Packaged-artifact access: {}",
                packaged_artifact_access_summary_for_report()
            ))
        }
        Some("packaged-artifact-path-policy-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-path-policy-summary")?;
            Ok(format!(
                "Packaged-artifact access: {}",
                packaged_artifact_access_summary_for_report()
            ))
        }
        Some("packaged-artifact-path-policy") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-path-policy")?;
            Ok(format!(
                "Packaged-artifact access: {}",
                packaged_artifact_access_summary_for_report()
            ))
        }
        Some("packaged-artifact-storage-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-storage-summary")?;
            Ok(format!(
                "Packaged-artifact storage/reconstruction: {}",
                validated_packaged_artifact_storage_summary_for_report()
            ))
        }
        Some("packaged-artifact-storage") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-storage")?;
            Ok(format!(
                "Packaged-artifact storage/reconstruction: {}",
                validated_packaged_artifact_storage_summary_for_report()
            ))
        }
        Some("packaged-artifact-production-profile-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-production-profile-summary")?;
            Ok(validated_packaged_artifact_production_profile_summary_for_report())
        }
        Some("packaged-artifact-production-profile") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-production-profile")?;
            Ok(validated_packaged_artifact_production_profile_summary_for_report())
        }
        Some("packaged-artifact-target-threshold-summary")
        | Some("packaged-artifact-target-threshold") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-target-threshold-summary")?;
            Ok(format!(
                "Packaged-artifact target thresholds: {}",
                validated_packaged_artifact_target_threshold_summary_for_report()
            ))
        }
        Some("packaged-artifact-target-threshold-state-summary")
        | Some("packaged-artifact-target-threshold-state") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-target-threshold-state-summary",
            )?;
            Ok(format!(
                "Packaged-artifact target-threshold state: {}",
                validated_packaged_artifact_target_threshold_state_for_report()
            ))
        }
        Some("packaged-artifact-target-threshold-scope-envelopes-summary")
        | Some("packaged-artifact-target-threshold-scope-envelopes") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-target-threshold-scope-envelopes-summary",
            )?;
            Ok(format!(
                "Packaged-artifact target-threshold scope envelopes: {}",
                validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report()
            ))
        }
        Some("packaged-artifact-source-fit-holdout-sync-summary")
        | Some("packaged-artifact-source-fit-holdout-sync") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-source-fit-holdout-sync-summary",
            )?;
            Ok(format!(
                "Packaged-artifact source-fit and hold-out sync: {}",
                validated_packaged_artifact_source_fit_holdout_sync_summary_for_report()
            ))
        }
        Some("packaged-artifact-phase2-corpus-alignment-summary")
        | Some("packaged-artifact-phase2-corpus-alignment") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-phase2-corpus-alignment-summary",
            )?;
            Ok(format!(
                "Packaged-artifact phase-2 corpus alignment: {}",
                validated_packaged_artifact_phase2_corpus_alignment_summary_for_report()
            ))
        }
        Some("packaged-artifact-fit-envelope-summary") | Some("packaged-artifact-fit-envelope") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-fit-envelope-summary")?;
            Ok(format!(
                "Packaged-artifact fit envelope: {}",
                packaged_artifact_fit_envelope_summary_for_report()
            ))
        }
        Some("packaged-artifact-fit-margins-summary") | Some("packaged-artifact-fit-margins") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-fit-margins-summary")?;
            Ok(format!(
                "Packaged-artifact fit margins: {}",
                packaged_artifact_fit_margin_summary_for_report()
            ))
        }
        Some("packaged-artifact-fit-sample-classes-summary")
        | Some("packaged-artifact-fit-sample-classes") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-fit-sample-classes-summary")?;
            Ok(format!(
                "Packaged-artifact fit sample classes: {}",
                packaged_artifact_fit_sample_classes_summary_for_report()
            ))
        }
        Some("packaged-artifact-body-cadence-summary") | Some("packaged-artifact-body-cadence") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-body-cadence-summary")?;
            Ok(format!(
                "Packaged-artifact body cadence: {}",
                validated_packaged_artifact_body_cadence_summary_for_report()
            ))
        }
        Some("packaged-artifact-fit-outliers-summary") | Some("packaged-artifact-fit-outliers") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-fit-outliers-summary")?;
            Ok(format!(
                "Packaged-artifact fit outliers: {}",
                packaged_artifact_fit_outlier_summary_for_report()
            ))
        }
        Some("packaged-artifact-fit-threshold-violation-count-summary")
        | Some("packaged-artifact-fit-threshold-violation-count") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-fit-threshold-violation-count-summary",
            )?;
            Ok(format!(
                "Packaged-artifact fit threshold violation count: {}",
                report_summary_payload(
                    packaged_artifact_fit_threshold_violation_count_for_report(),
                    "fit threshold violations: ",
                )
            ))
        }
        Some("packaged-artifact-fit-threshold-violations-summary")
        | Some("packaged-artifact-fit-threshold-violations") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-fit-threshold-violations-summary",
            )?;
            Ok(format!(
                "Packaged-artifact fit threshold violations: {}",
                report_summary_payload(
                    packaged_artifact_fit_threshold_violation_summary_for_report(),
                    "fit threshold violations: ",
                )
            ))
        }
        Some("packaged-artifact-generation-manifest-summary")
        | Some("packaged-artifact-generation-manifest") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-generation-manifest-summary")?;
            Ok(packaged_artifact_generation_manifest_for_report())
        }
        Some("packaged-artifact-generation-manifest-checksum-summary")
        | Some("packaged-artifact-generation-manifest-checksum") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-generation-manifest-checksum-summary",
            )?;
            Ok(packaged_artifact_generation_manifest_checksum_for_report())
        }
        Some("packaged-artifact-generation-policy-summary")
        | Some("packaged-artifact-generation-policy") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-generation-policy-summary")?;
            Ok(packaged_artifact_generation_policy_summary_for_report())
        }
        Some("packaged-artifact-generation-residual-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-generation-residual-summary")?;
            let summary =
                validated_packaged_artifact_generation_residual_bodies_summary_for_report()?;
            Ok(format!(
                "Packaged-artifact generation residual bodies: {}",
                summary
            ))
        }
        Some("packaged-artifact-generation-residual-bodies-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-generation-residual-bodies-summary",
            )?;
            let summary =
                validated_packaged_artifact_generation_residual_bodies_summary_for_report()?;
            Ok(format!(
                "Packaged-artifact generation residual bodies: {}",
                summary
            ))
        }
        Some("packaged-artifact-regeneration-summary") | Some("packaged-artifact-regeneration") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-regeneration-summary")?;
            Ok(format!(
                "Packaged-artifact regeneration: {}",
                packaged_artifact_regeneration_summary_for_report()
            ))
        }
        Some("packaged-frame-parity-summary") | Some("packaged-frame-parity") => {
            ensure_no_extra_args(&args[1..], "packaged-frame-parity-summary")?;
            Ok(format!(
                "Packaged frame parity: {}",
                packaged_frame_parity_summary_for_report()
            ))
        }
        Some("packaged-frame-treatment-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-frame-treatment-summary")?;
            Ok(format!(
                "Packaged frame treatment: {}",
                format_packaged_frame_treatment_summary()
            ))
        }
        Some("packaged-lookup-epoch-policy-summary")
        | Some("packaged-lookup-epoch-policy")
        | Some("packaged-artifact-lookup-epoch-policy-summary")
        | Some("packaged-artifact-lookup-epoch-policy") => {
            ensure_no_extra_args(&args[1..], "packaged-lookup-epoch-policy-summary")?;
            Ok(packaged_lookup_epoch_policy_summary_for_report())
        }
        Some("workspace-audit") => {
            ensure_no_extra_args(&args[1..], "workspace-audit")?;
            let report = workspace_audit_report().map_err(|error| error.to_string())?;
            if report.is_clean() {
                Ok(report.to_string())
            } else {
                Err(format!("workspace audit failed:\n{report}"))
            }
        }
        Some("audit") => {
            ensure_no_extra_args(&args[1..], "audit")?;
            let report = workspace_audit_report().map_err(|error| error.to_string())?;
            if report.is_clean() {
                Ok(report.to_string())
            } else {
                Err(format!("workspace audit failed:\n{report}"))
            }
        }
        Some("native-dependency-audit") => {
            ensure_no_extra_args(&args[1..], "native-dependency-audit")?;
            let report = workspace_audit_report().map_err(|error| error.to_string())?;
            if report.is_clean() {
                Ok(report.to_string())
            } else {
                Err(format!("workspace audit failed:\n{report}"))
            }
        }
        Some("workspace-audit-summary") => {
            ensure_no_extra_args(&args[1..], "workspace-audit-summary")?;
            render_workspace_audit_summary().map_err(|error| error.to_string())
        }
        Some("native-dependency-audit-summary") => {
            ensure_no_extra_args(&args[1..], "native-dependency-audit-summary")?;
            render_native_dependency_audit_summary().map_err(|error| error.to_string())
        }
        Some("workspace-provenance-summary") => {
            ensure_no_extra_args(&args[1..], "workspace-provenance-summary")?;
            Ok(workspace_provenance_summary_for_report())
        }
        Some("workspace-provenance") => {
            ensure_no_extra_args(&args[1..], "workspace-provenance")?;
            Ok(workspace_provenance_summary_for_report())
        }
        Some("api-stability-summary") | Some("api-posture-summary") => {
            ensure_no_extra_args(&args[1..], "api-stability-summary")?;
            Ok(render_api_stability_summary())
        }
        Some("api-stability") | Some("api-posture") => {
            ensure_no_extra_args(&args[1..], "api-stability")?;
            Ok(current_api_stability_profile().to_string())
        }
        Some("compatibility-profile-summary") | Some("profile-summary") => {
            ensure_no_extra_args(&args[1..], "compatibility-profile-summary")?;
            Ok(render_compatibility_profile_summary())
        }
        Some("compatibility-caveats-summary") => {
            ensure_no_extra_args(&args[1..], "compatibility-caveats-summary")?;
            Ok(render_compatibility_caveats_summary())
        }
        Some("compatibility-caveats") => {
            ensure_no_extra_args(&args[1..], "compatibility-caveats")?;
            Ok(render_compatibility_caveats_summary())
        }
        Some("known-gaps-summary") => {
            ensure_no_extra_args(&args[1..], "known-gaps-summary")?;
            Ok(render_known_gaps_summary())
        }
        Some("known-gaps") => {
            ensure_no_extra_args(&args[1..], "known-gaps")?;
            Ok(render_known_gaps_summary())
        }
        Some("catalog-inventory-summary") => {
            ensure_no_extra_args(&args[1..], "catalog-inventory-summary")?;
            Ok(render_catalog_inventory_summary())
        }
        Some("catalog-inventory") => {
            ensure_no_extra_args(&args[1..], "catalog-inventory")?;
            Ok(render_catalog_inventory_summary())
        }
        Some("catalog-posture-summary") => {
            ensure_no_extra_args(&args[1..], "catalog-posture-summary")?;
            Ok(render_catalog_posture_summary())
        }
        Some("catalog-posture") => {
            ensure_no_extra_args(&args[1..], "catalog-posture")?;
            Ok(render_catalog_posture_summary())
        }
        Some("custom-definition-ayanamsa-labels-summary")
        | Some("custom-definition-ayanamsa-labels") => {
            ensure_no_extra_args(&args[1..], "custom-definition-ayanamsa-labels-summary")?;
            Ok(render_custom_definition_ayanamsa_labels_summary())
        }
        Some("release-house-system-canonical-names-summary")
        | Some("release-house-system-canonical-names") => {
            ensure_no_extra_args(&args[1..], "release-house-system-canonical-names-summary")?;
            Ok(render_release_house_system_canonical_names_summary())
        }
        Some("release-ayanamsa-canonical-names-summary")
        | Some("release-ayanamsa-canonical-names") => {
            ensure_no_extra_args(&args[1..], "release-ayanamsa-canonical-names-summary")?;
            Ok(render_release_ayanamsa_canonical_names_summary())
        }
        Some("ayanamsa-audit-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-audit-summary")?;
            Ok(render_ayanamsa_audit_summary())
        }
        Some("ayanamsa-audit") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-audit")?;
            Ok(render_ayanamsa_audit_summary())
        }
        Some("target-house-scope-summary") => {
            ensure_no_extra_args(&args[1..], "target-house-scope-summary")?;
            Ok(render_target_house_scope_summary())
        }
        Some("target-house-scope") => {
            ensure_no_extra_args(&args[1..], "target-house-scope")?;
            Ok(render_target_house_scope_summary())
        }
        Some("target-ayanamsa-scope-summary") => {
            ensure_no_extra_args(&args[1..], "target-ayanamsa-scope-summary")?;
            Ok(render_target_ayanamsa_scope_summary())
        }
        Some("target-ayanamsa-scope") => {
            ensure_no_extra_args(&args[1..], "target-ayanamsa-scope")?;
            Ok(render_target_ayanamsa_scope_summary())
        }
        Some("verify-compatibility-profile") => {
            ensure_no_extra_args(&args[1..], "verify-compatibility-profile")?;
            verify_compatibility_profile().map_err(render_error)
        }
        Some("release-notes") => {
            ensure_no_extra_args(&args[1..], "release-notes")?;
            Ok(render_release_notes_text())
        }
        Some("release-notes-summary") => {
            ensure_no_extra_args(&args[1..], "release-notes-summary")?;
            Ok(render_release_notes_summary_text())
        }
        Some("release-checklist") => {
            ensure_no_extra_args(&args[1..], "release-checklist")?;
            Ok(render_release_checklist_text())
        }
        Some("release-smoke") => {
            ensure_no_extra_args(&args[1..], "release-smoke")?;
            validate_release_smoke()?;
            Ok(render_release_smoke_text())
        }
        Some("release-gate") => {
            ensure_no_extra_args(&args[1..], "release-gate")?;
            validate_release_gate()?;
            Ok(render_release_checklist_text())
        }
        Some("release-checklist-summary") | Some("checklist-summary") => {
            ensure_no_extra_args(&args[1..], "release-checklist-summary")?;
            Ok(render_release_checklist_summary_text())
        }
        Some("release-gate-summary") => {
            ensure_no_extra_args(&args[1..], "release-gate-summary")?;
            validate_release_gate()?;
            Ok(render_release_checklist_summary_text())
        }
        Some("release-summary") => {
            ensure_no_extra_args(&args[1..], "release-summary")?;
            Ok(render_release_summary_text())
        }
        Some("source-corpus-summary") => {
            ensure_no_extra_args(&args[1..], "source-corpus-summary")?;
            Ok(source_corpus_summary_for_report())
        }
        Some("source-corpus") => {
            ensure_no_extra_args(&args[1..], "source-corpus")?;
            Ok(source_corpus_summary_for_report())
        }
        Some("source-corpus-posture-summary") => {
            ensure_no_extra_args(&args[1..], "source-corpus-posture-summary")?;
            Ok(source_corpus_summary_for_report())
        }
        Some("source-corpus-posture") => {
            ensure_no_extra_args(&args[1..], "source-corpus-posture")?;
            Ok(source_corpus_summary_for_report())
        }
        Some("jpl-batch-error-taxonomy-summary") => {
            ensure_no_extra_args(&args[1..], "jpl-batch-error-taxonomy-summary")?;
            Ok(jpl_snapshot_batch_error_taxonomy_summary_for_report())
        }
        Some("jpl-snapshot-evidence-summary") => {
            ensure_no_extra_args(&args[1..], "jpl-snapshot-evidence-summary")?;
            Ok(jpl_snapshot_evidence_summary_for_report())
        }
        Some("jpl-source-corpus-contract-summary") => {
            ensure_no_extra_args(&args[1..], "jpl-source-corpus-contract-summary")?;
            Ok(jpl_source_corpus_contract_summary_for_report())
        }
        Some("jpl-source-corpus-contract") => {
            ensure_no_extra_args(&args[1..], "jpl-source-corpus-contract")?;
            Ok(jpl_source_corpus_contract_summary_for_report())
        }
        Some("jpl-source-posture-summary") | Some("jpl-source-posture") => {
            ensure_no_extra_args(&args[1..], "jpl-source-posture-summary")?;
            Ok(jpl_source_posture_summary_for_report())
        }
        Some("jpl-provenance-only-summary") => {
            ensure_no_extra_args(&args[1..], "jpl-provenance-only-summary")?;
            Ok(jpl_provenance_only_summary_for_report())
        }
        Some("jpl-provenance-only") => {
            ensure_no_extra_args(&args[1..], "jpl-provenance-only")?;
            Ok(jpl_provenance_only_summary_for_report())
        }
        Some("production-generation-boundary-summary") | Some("production-generation-boundary") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-summary")?;
            Ok(production_generation_boundary_summary_for_report())
        }
        Some("production-generation-boundary-request-corpus-summary")
        | Some("production-generation-boundary-request-corpus") => {
            ensure_no_extra_args(
                &args[1..],
                "production-generation-boundary-request-corpus-summary",
            )?;
            Ok(production_generation_boundary_request_corpus_summary_for_report())
        }
        Some("production-generation-boundary-request-corpus-equatorial-summary")
        | Some("production-generation-boundary-request-corpus-equatorial") => {
            ensure_no_extra_args(
                &args[1..],
                "production-generation-boundary-request-corpus-equatorial-summary",
            )?;
            Ok(production_generation_boundary_request_corpus_equatorial_summary_for_report())
        }
        Some("production-generation-body-class-coverage-summary")
        | Some("production-body-class-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "production-generation-body-class-coverage-summary",
            )?;
            Ok(validated_production_generation_body_class_coverage_summary_for_report())
        }
        Some("production-generation-source-window-summary")
        | Some("production-generation-source-window") => {
            ensure_no_extra_args(&args[1..], "production-generation-source-window-summary")?;
            Ok(production_generation_snapshot_window_summary_for_report())
        }
        Some("production-generation-manifest-summary") | Some("production-generation-manifest") => {
            ensure_no_extra_args(&args[1..], "production-generation-manifest-summary")?;
            Ok(validated_production_generation_manifest_summary_text_for_report())
        }
        Some("production-generation-manifest-checksum-summary")
        | Some("production-generation-manifest-checksum") => {
            ensure_no_extra_args(
                &args[1..],
                "production-generation-manifest-checksum-summary",
            )?;
            Ok(production_generation_manifest_checksum_for_report())
        }
        Some("production-generation") => {
            ensure_no_extra_args(&args[1..], "production-generation")?;
            Ok(production_generation_snapshot_summary_for_report())
        }
        Some("production-generation-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-summary")?;
            Ok(production_generation_snapshot_summary_for_report())
        }
        Some("production-generation-quarter-day-boundary-summary")
        | Some("production-generation-quarter-day-boundary") => {
            ensure_no_extra_args(
                &args[1..],
                "production-generation-quarter-day-boundary-summary",
            )?;
            Ok(pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report())
        }
        Some("production-generation-boundary-source-summary")
        | Some("production-generation-boundary-source") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-source-summary")?;
            Ok(production_generation_boundary_source_summary_for_report())
        }
        Some("production-generation-boundary-window-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-window-summary")?;
            Ok(production_generation_boundary_window_summary_for_report())
        }
        Some("production-generation-boundary-window") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-window")?;
            Ok(production_generation_boundary_window_summary_for_report())
        }
        Some("production-generation-corpus-shape-summary")
        | Some("production-generation-corpus-shape") => {
            ensure_no_extra_args(&args[1..], "production-generation-corpus-shape-summary")?;
            validated_production_generation_corpus_shape_summary_for_report()
        }
        Some("production-generation-source-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-source-summary")?;
            validated_production_generation_source_summary_for_report()
        }
        Some("production-generation-source-revision-summary")
        | Some("production-generation-source-revision") => {
            ensure_no_extra_args(&args[1..], "production-generation-source-revision-summary")?;
            validated_production_generation_source_revision_summary_for_report()
                .map_err(|error| error.to_string())
        }
        Some("production-generation-source") => {
            ensure_no_extra_args(&args[1..], "production-generation-source")?;
            validated_production_generation_source_summary_for_report()
        }
        Some("packaged-artifact-normalized-intermediate-summary")
        | Some("packaged-artifact-normalized-intermediate") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-normalized-intermediate-summary",
            )?;
            Ok(format!(
                "Packaged-artifact normalized intermediates: {}",
                packaged_artifact_normalized_intermediate_summary_for_report()
            ))
        }
        Some("comparison-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source-summary")?;
            validated_comparison_snapshot_source_summary_for_report()
        }
        Some("comparison-snapshot-source") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source")?;
            validated_comparison_snapshot_source_summary_for_report()
        }
        Some("comparison-snapshot-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source-window-summary")?;
            validated_comparison_snapshot_source_window_summary_for_report()
        }
        Some("comparison-snapshot-source-window") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source-window")?;
            validated_comparison_snapshot_source_window_summary_for_report()
        }
        Some("comparison-snapshot-body-class-coverage-summary")
        | Some("comparison-body-class-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "comparison-snapshot-body-class-coverage-summary",
            )?;
            validated_comparison_snapshot_body_class_coverage_summary_for_report()
        }
        Some("comparison-snapshot-manifest-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-manifest-summary")?;
            Ok(format_comparison_snapshot_manifest_summary())
        }
        Some("comparison-snapshot-manifest") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-manifest")?;
            Ok(format_comparison_snapshot_manifest_summary())
        }
        Some("comparison-snapshot") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot")?;
            Ok(render_comparison_snapshot_summary_text())
        }
        Some("j2000-snapshot") => {
            ensure_no_extra_args(&args[1..], "j2000-snapshot")?;
            Ok(render_comparison_snapshot_summary_text())
        }
        Some("comparison-snapshot-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-summary")?;
            Ok(render_comparison_snapshot_summary_text())
        }
        Some("comparison-snapshot-batch-parity-summary")
        | Some("comparison-snapshot-batch-parity") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-batch-parity-summary")?;
            Ok(comparison_snapshot_batch_parity_summary_text())
        }
        Some("reference-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source-summary")?;
            Ok(reference_snapshot_source_summary_for_report())
        }
        Some("reference-snapshot-source") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source")?;
            Ok(reference_snapshot_source_summary_for_report())
        }
        Some("reference-snapshot-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source-window-summary")?;
            Ok(reference_snapshot_source_window_summary_for_report())
        }
        Some("reference-snapshot-source-window") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source-window")?;
            Ok(reference_snapshot_source_window_summary_for_report())
        }
        Some("reference-snapshot-lunar-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-lunar-boundary-summary")?;
            Ok(reference_snapshot_lunar_boundary_summary_for_report())
        }
        Some("lunar-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-lunar-boundary-summary")?;
            Ok(reference_snapshot_lunar_boundary_summary_for_report())
        }
        Some("reference-snapshot-1500-selected-body-boundary-summary")
        | Some("1500-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1500-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1500_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2268932-selected-body-boundary-summary")
        | Some("2268932-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2268932-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2268932_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-1600-selected-body-boundary-summary")
        | Some("1600-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1600-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1600_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2305457-selected-body-boundary-summary")
        | Some("2305457-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2305457-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2305457_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-1750-selected-body-boundary-summary")
        | Some("1750-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1750-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1750_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-1750-major-body-interior-summary")
        | Some("1750-major-body-interior-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1750-major-body-interior-summary",
            )?;
            Ok(reference_snapshot_1750_major_body_interior_summary_for_report())
        }
        Some("reference-snapshot-2200-selected-body-boundary-summary")
        | Some("2200-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2200-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2200_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2524593-selected-body-boundary-summary")
        | Some("2524593-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2524593-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2524593_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-1900-selected-body-boundary-summary")
        | Some("1900-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1900-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1900_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2415020-selected-body-boundary-summary")
        | Some("2415020-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2415020-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2415020_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2500-selected-body-boundary-summary")
        | Some("2500-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2500-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2500_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2634167-selected-body-boundary-summary")
        | Some("2634167-selected-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2634167-selected-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2634167_selected_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2400000-major-body-boundary-summary")
        | Some("2400000-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2400000-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2400000_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451545-major-body-boundary-summary")
        | Some("2451545-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451545-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451545_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-1749-major-body-boundary-summary")
        | Some("1749-major-body-boundary-summary")
        | Some("2360233-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1749-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1749_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-early-major-body-boundary-summary")
        | Some("early-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-early-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_early_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2378498-major-body-boundary-summary")
        | Some("2378498-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2378498-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2378498_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-1800-major-body-boundary-summary")
        | Some("1800-major-body-boundary-summary")
        | Some("2378499-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1800-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1800_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2500-major-body-boundary-summary")
        | Some("2500-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2500-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2500_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2453000-major-body-boundary-summary")
        | Some("2453000-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2453000-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2453000_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2500000-major-body-boundary-summary")
        | Some("2500000-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2500000-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2500000_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2600000-major-body-boundary-summary")
        | Some("2600000-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2600000-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2600000_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451910-major-body-boundary-summary")
        | Some("2451910-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451910-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451910_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451911-major-body-boundary-summary")
        | Some("2451911-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451911-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451911_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451912-major-body-boundary-summary")
        | Some("2451912-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451912-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451912_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451913-major-body-boundary-summary")
        | Some("2451913-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451913-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451913_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451914-major-body-boundary-summary")
        | Some("2451914-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451914-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451914_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451915-major-body-boundary-summary")
        | Some("2451915-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451915-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451915_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451917-major-body-boundary-summary")
        | Some("2451917-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451917-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451917_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451918-major-body-boundary-summary")
        | Some("2451918-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451918-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451918_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451919-major-body-boundary-summary")
        | Some("2451919-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451919-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451919_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2360234-major-body-interior-summary")
        | Some("2360234-major-body-interior-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2360234-major-body-interior-summary",
            )?;
            Ok(reference_snapshot_2360234_major_body_interior_summary_for_report())
        }
        Some("reference-snapshot-2451916-major-body-interior-summary")
        | Some("2451916-major-body-interior-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451916-major-body-interior-summary",
            )?;
            Ok(reference_snapshot_2451916_major_body_interior_summary_for_report())
        }
        Some("reference-snapshot-2451920-major-body-interior-summary")
        | Some("2451920-major-body-interior-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451920-major-body-interior-summary",
            )?;
            Ok(reference_snapshot_2451920_major_body_interior_summary_for_report())
        }
        Some("reference-snapshot-body-class-coverage-summary")
        | Some("reference-body-class-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-body-class-coverage-summary")?;
            Ok(reference_snapshot_body_class_coverage_summary_for_report())
        }
        Some("reference-snapshot-manifest-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-manifest-summary")?;
            Ok(reference_snapshot_manifest_summary_for_report())
        }
        Some("reference-snapshot-manifest") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-manifest")?;
            Ok(reference_snapshot_manifest_summary_for_report())
        }
        Some("reference-snapshot") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot")?;
            Ok(render_reference_snapshot_summary_text())
        }
        Some("reference-snapshot-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-summary")?;
            Ok(render_reference_snapshot_summary_text())
        }
        Some("reference-snapshot-exact-j2000-evidence-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-exact-j2000-evidence-summary",
            )?;
            Ok(render_reference_snapshot_exact_j2000_evidence_text())
        }
        Some("reference-snapshot-exact-j2000-evidence") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-exact-j2000-evidence")?;
            Ok(render_reference_snapshot_exact_j2000_evidence_text())
        }
        Some("exact-j2000-evidence") => {
            ensure_no_extra_args(&args[1..], "exact-j2000-evidence")?;
            Ok(render_reference_snapshot_exact_j2000_evidence_text())
        }
        Some("reference-snapshot-batch-parity-summary")
        | Some("reference-snapshot-batch-parity") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-batch-parity-summary")?;
            Ok(reference_snapshot_batch_parity_summary_text())
        }
        Some("reference-snapshot-mixed-time-scale-batch-parity-summary")
        | Some("reference-snapshot-mixed-tt-tdb-batch-parity-summary")
        | Some("reference-snapshot-mixed-tt-tdb-batch-parity") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-mixed-time-scale-batch-parity-summary",
            )?;
            Ok(reference_snapshot_mixed_time_scale_batch_parity_summary_text())
        }
        Some("reference-snapshot-equatorial-parity-summary")
        | Some("reference-snapshot-equatorial-parity") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-equatorial-parity-summary")?;
            Ok(reference_snapshot_equatorial_parity_summary_for_report())
        }
        Some("reference-high-curvature-summary")
        | Some("high-curvature-summary")
        | Some("reference-snapshot-major-body-high-curvature-summary")
        | Some("major-body-high-curvature-summary") => {
            ensure_no_extra_args(&args[1..], "reference-high-curvature-summary")?;
            Ok(reference_snapshot_high_curvature_summary_for_report())
        }
        Some("reference-snapshot-major-body-boundary-summary")
        | Some("major-body-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-major-body-boundary-summary")?;
            Ok(reference_snapshot_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-major-body-bridge-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-major-body-bridge-summary")?;
            Ok(reference_snapshot_major_body_bridge_summary_for_report())
        }
        Some("bridge-summary") => {
            ensure_no_extra_args(&args[1..], "bridge-summary")?;
            Ok(reference_snapshot_major_body_bridge_summary_for_report())
        }
        Some("reference-snapshot-2451915-major-body-bridge-summary")
        | Some("2451915-major-body-bridge-summary")
        | Some("2451915-major-body-bridge") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451915-major-body-bridge-summary",
            )?;
            Ok(pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary_for_report())
        }
        Some("reference-snapshot-2451917-major-body-bridge-summary")
        | Some("2451917-major-body-bridge-summary")
        | Some("2451917-major-body-bridge") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451917-major-body-bridge-summary",
            )?;
            Ok(reference_snapshot_2451917_major_body_bridge_summary_for_report())
        }
        Some("reference-snapshot-bridge-day-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-bridge-day-summary")?;
            Ok(reference_snapshot_bridge_day_summary_for_report())
        }
        Some("reference-snapshot-2451914-bridge-day-summary")
        | Some("2451914-bridge-day-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-2451914-bridge-day-summary")?;
            Ok(reference_snapshot_2451914_bridge_day_summary_for_report())
        }
        Some("reference-snapshot-2451914-major-body-bridge-day-summary")
        | Some("2451914-major-body-bridge-day-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451914-major-body-bridge-day-summary",
            )?;
            Ok(reference_snapshot_2451914_major_body_bridge_day_summary_for_report())
        }
        Some("reference-snapshot-2451914-major-body-bridge-summary")
        | Some("2451914-major-body-bridge-summary")
        | Some("2451914-major-body-bridge") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451914-major-body-bridge-summary",
            )?;
            Ok(reference_snapshot_bridge_day_summary_for_report())
        }
        Some("bridge-day-summary") => {
            ensure_no_extra_args(&args[1..], "bridge-day-summary")?;
            Ok(reference_snapshot_bridge_day_summary_for_report())
        }
        Some("major-body-bridge-summary") => {
            ensure_no_extra_args(&args[1..], "major-body-bridge-summary")?;
            Ok(reference_snapshot_major_body_bridge_summary_for_report())
        }
        Some("reference-snapshot-mars-jupiter-boundary-summary")
        | Some("mars-jupiter-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-mars-jupiter-boundary-summary",
            )?;
            Ok(reference_snapshot_mars_jupiter_boundary_summary_for_report())
        }
        Some("reference-snapshot-mars-outer-boundary-summary")
        | Some("mars-outer-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-mars-outer-boundary-summary")?;
            Ok(reference_snapshot_mars_outer_boundary_summary_for_report())
        }
        Some("reference-snapshot-major-body-boundary-window-summary")
        | Some("major-body-boundary-window-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-major-body-boundary-window-summary",
            )?;
            Ok(reference_snapshot_major_body_boundary_window_summary_for_report())
        }
        Some("reference-high-curvature-window-summary")
        | Some("high-curvature-window-summary")
        | Some("reference-snapshot-major-body-high-curvature-window-summary")
        | Some("major-body-high-curvature-window-summary") => {
            ensure_no_extra_args(&args[1..], "reference-high-curvature-window-summary")?;
            Ok(reference_snapshot_high_curvature_window_summary_for_report())
        }
        Some("reference-high-curvature-epoch-coverage-summary")
        | Some("high-curvature-epoch-coverage-summary")
        | Some("reference-snapshot-major-body-high-curvature-epoch-coverage-summary")
        | Some("major-body-high-curvature-epoch-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-high-curvature-epoch-coverage-summary",
            )?;
            Ok(reference_snapshot_high_curvature_epoch_coverage_summary_for_report())
        }
        Some("reference-snapshot-boundary-epoch-coverage-summary")
        | Some("reference-snapshot-boundary-epoch-coverage")
        | Some("boundary-epoch-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-boundary-epoch-coverage-summary",
            )?;
            Ok(reference_snapshot_boundary_epoch_coverage_summary_for_report())
        }
        Some("reference-snapshot-sparse-boundary-summary") | Some("sparse-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-sparse-boundary-summary")?;
            Ok(reference_snapshot_sparse_boundary_summary_for_report())
        }
        Some("reference-snapshot-boundary-day-summary")
        | Some("reference-snapshot-boundary-day")
        | Some("boundary-day-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-boundary-day-summary")?;
            Ok(reference_snapshot_sparse_boundary_summary_for_report())
        }
        Some("reference-snapshot-dense-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-dense-boundary-summary")?;
            Ok(reference_snapshot_dense_boundary_summary_for_report())
        }
        Some("dense-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "dense-boundary-summary")?;
            Ok(reference_snapshot_dense_boundary_summary_for_report())
        }
        Some("reference-snapshot-pre-bridge-boundary-summary")
        | Some("reference-snapshot-pre-bridge-boundary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-pre-bridge-boundary-summary")?;
            Ok(reference_snapshot_pre_bridge_boundary_summary_for_report())
        }
        Some("pre-bridge-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "pre-bridge-boundary-summary")?;
            Ok(reference_snapshot_pre_bridge_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451914-major-body-pre-bridge-summary")
        | Some("2451914-major-body-pre-bridge-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451914-major-body-pre-bridge-summary",
            )?;
            Ok(reference_snapshot_pre_bridge_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451916-major-body-boundary-summary")
        | Some("2451916-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451916-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2451916_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2451916-major-body-dense-boundary-summary")
        | Some("2451916-major-body-dense-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451916-major-body-dense-boundary-summary",
            )?;
            Ok(reference_snapshot_2451916_major_body_dense_boundary_summary_for_report())
        }
        Some("source-documentation-summary") => {
            ensure_no_extra_args(&args[1..], "source-documentation-summary")?;
            Ok(format_vsop87_source_documentation_summary())
        }
        Some("source-documentation") => {
            ensure_no_extra_args(&args[1..], "source-documentation")?;
            Ok(format_vsop87_source_documentation_summary())
        }
        Some("source-documentation-health-summary") => {
            ensure_no_extra_args(&args[1..], "source-documentation-health-summary")?;
            Ok(format_vsop87_source_documentation_health_summary())
        }
        Some("source-documentation-health") => {
            ensure_no_extra_args(&args[1..], "source-documentation-health")?;
            Ok(format_vsop87_source_documentation_health_summary())
        }
        Some("source-audit-summary") => {
            ensure_no_extra_args(&args[1..], "source-audit-summary")?;
            Ok(source_audit_summary_for_report())
        }
        Some("source-audit") => {
            ensure_no_extra_args(&args[1..], "source-audit")?;
            Ok(source_audit_summary_for_report())
        }
        Some("generated-binary-audit-summary") => {
            ensure_no_extra_args(&args[1..], "generated-binary-audit-summary")?;
            Ok(generated_binary_audit_summary_for_report())
        }
        Some("generated-binary-audit") => {
            ensure_no_extra_args(&args[1..], "generated-binary-audit")?;
            Ok(generated_binary_audit_summary_for_report())
        }
        Some("time-scale-policy-summary") => {
            ensure_no_extra_args(&args[1..], "time-scale-policy-summary")?;
            Ok(render_time_scale_policy_summary_text())
        }
        Some("time-scale-policy") => {
            ensure_no_extra_args(&args[1..], "time-scale-policy")?;
            Ok(render_time_scale_policy_summary_text())
        }
        Some("utc-convenience-policy-summary") => {
            ensure_no_extra_args(&args[1..], "utc-convenience-policy-summary")?;
            Ok(render_utc_convenience_policy_summary_text())
        }
        Some("utc-convenience-policy") => {
            ensure_no_extra_args(&args[1..], "utc-convenience-policy")?;
            Ok(render_utc_convenience_policy_summary_text())
        }
        Some("delta-t-policy-summary") => {
            ensure_no_extra_args(&args[1..], "delta-t-policy-summary")?;
            Ok(render_delta_t_policy_summary_text())
        }
        Some("delta-t-policy") => {
            ensure_no_extra_args(&args[1..], "delta-t-policy")?;
            Ok(render_delta_t_policy_summary_text())
        }
        Some("observer-policy-summary") => {
            ensure_no_extra_args(&args[1..], "observer-policy-summary")?;
            Ok(render_observer_policy_summary_text())
        }
        Some("observer-policy") => {
            ensure_no_extra_args(&args[1..], "observer-policy")?;
            Ok(render_observer_policy_summary_text())
        }
        Some("apparentness-policy-summary") => {
            ensure_no_extra_args(&args[1..], "apparentness-policy-summary")?;
            Ok(render_apparentness_policy_summary_text())
        }
        Some("apparentness-policy") => {
            ensure_no_extra_args(&args[1..], "apparentness-policy")?;
            Ok(render_apparentness_policy_summary_text())
        }
        Some("native-sidereal-policy-summary") => {
            ensure_no_extra_args(&args[1..], "native-sidereal-policy-summary")?;
            Ok(render_native_sidereal_policy_summary_text())
        }
        Some("native-sidereal-policy") => {
            ensure_no_extra_args(&args[1..], "native-sidereal-policy")?;
            Ok(render_native_sidereal_policy_summary_text())
        }
        Some("zodiac-policy-summary") => {
            ensure_no_extra_args(&args[1..], "zodiac-policy-summary")?;
            Ok(render_zodiac_policy_summary_text())
        }
        Some("zodiac-policy") => {
            ensure_no_extra_args(&args[1..], "zodiac-policy")?;
            Ok(render_zodiac_policy_summary_text())
        }
        Some("interpolation-posture-summary") => {
            ensure_no_extra_args(&args[1..], "interpolation-posture-summary")?;
            Ok(render_interpolation_posture_summary_text())
        }
        Some("interpolation-posture") => {
            ensure_no_extra_args(&args[1..], "interpolation-posture")?;
            Ok(render_interpolation_posture_summary_text())
        }
        Some("interpolation-quality-summary") => {
            ensure_no_extra_args(&args[1..], "interpolation-quality-summary")?;
            Ok(render_interpolation_quality_summary_text())
        }
        Some("interpolation-quality-kind-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "interpolation-quality-kind-coverage-summary")?;
            Ok(jpl_interpolation_quality_kind_coverage_for_report())
        }
        Some("interpolation-quality-request-corpus-summary") => {
            ensure_no_extra_args(&args[1..], "interpolation-quality-request-corpus-summary")?;
            Ok(interpolation_quality_sample_request_corpus_summary_for_report())
        }
        Some("interpolation-quality-request-corpus") => {
            ensure_no_extra_args(&args[1..], "interpolation-quality-request-corpus")?;
            Ok(interpolation_quality_sample_request_corpus_summary_for_report())
        }
        Some("lunar-reference-error-envelope-summary") | Some("lunar-reference-error-envelope") => {
            ensure_no_extra_args(&args[1..], "lunar-reference-error-envelope-summary")?;
            Ok(render_lunar_reference_error_envelope_summary_text())
        }
        Some("lunar-reference-evidence-summary") | Some("lunar-reference-evidence") => {
            ensure_no_extra_args(&args[1..], "lunar-reference-evidence-summary")?;
            Ok(render_lunar_reference_evidence_summary_text())
        }
        Some("lunar-equatorial-reference-error-envelope-summary")
        | Some("lunar-equatorial-reference-error-envelope") => {
            ensure_no_extra_args(
                &args[1..],
                "lunar-equatorial-reference-error-envelope-summary",
            )?;
            Ok(render_lunar_equatorial_reference_error_envelope_summary_text())
        }
        Some("lunar-apparent-comparison-summary") | Some("lunar-apparent-comparison") => {
            ensure_no_extra_args(&args[1..], "lunar-apparent-comparison-summary")?;
            Ok(render_lunar_apparent_comparison_summary_text())
        }
        Some("reference-snapshot-lunar-source-window-summary")
        | Some("lunar-source-window-summary")
        | Some("lunar-source-window") => {
            ensure_no_extra_args(&args[1..], "lunar-source-window-summary")?;
            validated_lunar_source_window_summary_for_report()
        }
        Some("lunar-reference-mixed-time-scale-batch-parity-summary")
        | Some("lunar-reference-mixed-tt-tdb-batch-parity-summary")
        | Some("lunar-reference-mixed-tt-tdb-batch-parity") => {
            ensure_no_extra_args(
                &args[1..],
                "lunar-reference-mixed-time-scale-batch-parity-summary",
            )?;
            Ok(lunar_reference_batch_parity_summary_for_report())
        }
        Some("lunar-theory-request-policy-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-request-policy-summary")?;
            Ok(lunar_theory_request_policy_summary())
        }
        Some("lunar-theory-request-policy") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-request-policy")?;
            Ok(lunar_theory_request_policy_summary())
        }
        Some("lunar-theory-frame-treatment-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-frame-treatment-summary")?;
            Ok(lunar_theory_frame_treatment_summary_for_report())
        }
        Some("lunar-theory-frame-treatment") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-frame-treatment")?;
            Ok(lunar_theory_frame_treatment_summary_for_report())
        }
        Some("lunar-theory-limitations-summary") | Some("lunar-theory-limitations") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-limitations-summary")?;
            Ok(lunar_theory_limitations_summary_for_report())
        }
        Some("lunar-theory-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-summary")?;
            Ok(lunar_theory_summary_for_report())
        }
        Some("lunar-theory-capability-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-capability-summary")?;
            Ok(lunar_theory_capability_summary_for_report())
        }
        Some("lunar-theory-source-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-source-summary")?;
            Ok(lunar_theory_source_summary_for_report())
        }
        Some("lunar-theory-source-selection-summary") | Some("lunar-theory-source-selection") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-source-selection-summary")?;
            Ok(pleiades_elp::lunar_theory_source_selection_summary_for_report())
        }
        Some("lunar-theory-source-family-summary") | Some("lunar-theory-source-family") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-source-family-summary")?;
            Ok(pleiades_elp::lunar_theory_source_family_summary_for_report())
        }
        Some("lunar-theory-catalog-summary") | Some("lunar-theory-catalog") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-catalog-summary")?;
            Ok(lunar_theory_catalog_summary_for_report())
        }
        Some("lunar-theory-catalog-validation-summary")
        | Some("lunar-theory-catalog-validation") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-catalog-validation-summary")?;
            Ok(validated_lunar_theory_catalog_validation_summary_for_report())
        }
        Some("selected-asteroid-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-boundary-summary")?;
            Ok(selected_asteroid_boundary_summary_for_report())
        }
        Some("reference-snapshot-selected-asteroid-bridge-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-selected-asteroid-bridge-summary",
            )?;
            Ok(selected_asteroid_bridge_summary_for_report())
        }
        Some("selected-asteroid-bridge-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-bridge-summary")?;
            Ok(selected_asteroid_bridge_summary_for_report())
        }
        Some("reference-snapshot-selected-asteroid-dense-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-selected-asteroid-dense-boundary-summary",
            )?;
            Ok(selected_asteroid_dense_boundary_summary_for_report())
        }
        Some("selected-asteroid-dense-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-dense-boundary-summary")?;
            Ok(selected_asteroid_dense_boundary_summary_for_report())
        }
        Some("reference-snapshot-selected-asteroid-terminal-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-selected-asteroid-terminal-boundary-summary",
            )?;
            Ok(selected_asteroid_terminal_boundary_summary_for_report())
        }
        Some("selected-asteroid-terminal-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-terminal-boundary-summary")?;
            Ok(selected_asteroid_terminal_boundary_summary_for_report())
        }
        Some("selected-asteroid-source-evidence-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-evidence-summary")?;
            Ok(selected_asteroid_source_evidence_summary_for_report())
        }
        Some("reference-snapshot-2378498-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2378498-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2378498_summary_for_report())
        }
        Some("2378498-selected-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "2378498-selected-asteroid-source-summary")?;
            Ok(selected_asteroid_source_2378498_summary_for_report())
        }
        Some("reference-snapshot-2451917-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451917-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2451917_summary_for_report())
        }
        Some("2451917-selected-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "2451917-selected-asteroid-source-summary")?;
            Ok(selected_asteroid_source_2451917_summary_for_report())
        }
        Some("reference-snapshot-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_evidence_summary_for_report())
        }
        Some("selected-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-summary")?;
            Ok(selected_asteroid_source_evidence_summary_for_report())
        }
        Some("selected-asteroid-source-request-corpus-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "selected-asteroid-source-request-corpus-summary",
            )?;
            Ok(selected_asteroid_source_request_corpus_summary_for_report())
        }
        Some("selected-asteroid-source-request-corpus") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-request-corpus")?;
            Ok(selected_asteroid_source_request_corpus_summary_for_report())
        }
        Some("selected-asteroid-source-request-corpus-equatorial-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "selected-asteroid-source-request-corpus-equatorial-summary",
            )?;
            validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report()
        }
        Some("selected-asteroid-source-request-corpus-equatorial") => {
            ensure_no_extra_args(
                &args[1..],
                "selected-asteroid-source-request-corpus-equatorial",
            )?;
            validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report()
        }
        Some("reference-snapshot-selected-asteroid-source-window-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-selected-asteroid-source-window-summary",
            )?;
            Ok(selected_asteroid_source_window_summary_for_report())
        }
        Some("reference-snapshot-selected-asteroid-source-window") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-selected-asteroid-source-window",
            )?;
            Ok(selected_asteroid_source_window_summary_for_report())
        }
        Some("reference-snapshot-2453000-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2453000-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2453000_summary_for_report())
        }
        Some("2453000-selected-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "2453000-selected-asteroid-source-summary")?;
            Ok(selected_asteroid_source_2453000_summary_for_report())
        }
        Some("reference-snapshot-2500000-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2500000-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2500000_summary_for_report())
        }
        Some("2500000-selected-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "2500000-selected-asteroid-source-summary")?;
            Ok(selected_asteroid_source_2500000_summary_for_report())
        }
        Some("reference-snapshot-2634167-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2634167-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2634167_summary_for_report())
        }
        Some("2634167-selected-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "2634167-selected-asteroid-source-summary")?;
            Ok(selected_asteroid_source_2634167_summary_for_report())
        }
        Some("selected-asteroid-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-window-summary")?;
            validated_selected_asteroid_source_window_summary_for_report()
        }
        Some("selected-asteroid-source-window") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-window")?;
            validated_selected_asteroid_source_window_summary_for_report()
        }
        Some("selected-asteroid-batch-parity-summary") | Some("selected-asteroid-batch-parity") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-batch-parity-summary")?;
            Ok(selected_asteroid_batch_parity_summary_for_report())
        }
        Some("reference-asteroid-evidence-summary") => {
            ensure_no_extra_args(&args[1..], "reference-asteroid-evidence-summary")?;
            Ok(reference_asteroid_evidence_summary_for_report())
        }
        Some("reference-asteroid-equatorial-evidence-summary")
        | Some("reference-asteroid-equatorial-evidence") => {
            ensure_no_extra_args(&args[1..], "reference-asteroid-equatorial-evidence-summary")?;
            Ok(reference_asteroid_equatorial_evidence_summary_for_report())
        }
        Some("reference-asteroid-source-window-summary")
        | Some("reference-asteroid-source-window") => {
            ensure_no_extra_args(&args[1..], "reference-asteroid-source-window-summary")?;
            validated_reference_asteroid_source_window_summary_for_report()
        }
        Some("reference-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "reference-asteroid-source-summary")?;
            validated_reference_asteroid_source_window_summary_for_report()
        }
        Some("independent-holdout-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-source-window-summary")?;
            Ok(independent_holdout_snapshot_source_window_summary_for_report())
        }
        Some("independent-holdout-manifest-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-manifest-summary")?;
            Ok(independent_holdout_manifest_summary_for_report())
        }
        Some("independent-holdout-manifest") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-manifest")?;
            Ok(independent_holdout_manifest_summary_for_report())
        }
        Some("independent-holdout-quarter-day-boundary-summary")
        | Some("independent-holdout-quarter-day-boundary") => {
            ensure_no_extra_args(
                &args[1..],
                "independent-holdout-quarter-day-boundary-summary",
            )?;
            Ok(independent_holdout_snapshot_quarter_day_boundary_summary_for_report())
        }
        Some("independent-holdout-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-summary")?;
            Ok(jpl_independent_holdout_summary_for_report())
        }
        Some("independent-holdout-source-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-source-summary")?;
            Ok(independent_holdout_source_summary_for_report())
        }
        Some("independent-holdout-high-curvature-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-high-curvature-summary")?;
            Ok(independent_holdout_high_curvature_summary_for_report())
        }
        Some("holdout-high-curvature-summary") => {
            ensure_no_extra_args(&args[1..], "holdout-high-curvature-summary")?;
            Ok(independent_holdout_high_curvature_summary_for_report())
        }
        Some("independent-holdout-body-class-coverage-summary")
        | Some("holdout-body-class-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "independent-holdout-body-class-coverage-summary",
            )?;
            Ok(independent_holdout_snapshot_body_class_coverage_summary_for_report())
        }
        Some("independent-holdout-batch-parity-summary")
        | Some("independent-holdout-batch-parity") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-batch-parity-summary")?;
            Ok(independent_holdout_snapshot_batch_parity_summary_text())
        }
        Some("independent-holdout-equatorial-parity-summary")
        | Some("independent-holdout-equatorial-parity") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-equatorial-parity-summary")?;
            Ok(jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report())
        }
        Some("reference-holdout-overlap-summary") => {
            ensure_no_extra_args(&args[1..], "reference-holdout-overlap-summary")?;
            Ok(render_reference_holdout_overlap_summary_text())
        }
        Some("holdout-overlap-summary") => {
            ensure_no_extra_args(&args[1..], "holdout-overlap-summary")?;
            Ok(render_reference_holdout_overlap_summary_text())
        }
        Some("house-validation-summary") => {
            ensure_no_extra_args(&args[1..], "house-validation-summary")?;
            Ok(house_validation_summary_for_report())
        }
        Some("house-validation") => {
            ensure_no_extra_args(&args[1..], "house-validation")?;
            Ok(house_validation_summary_for_report())
        }
        Some("house-latitude-sensitive-summary") => {
            ensure_no_extra_args(&args[1..], "house-latitude-sensitive-summary")?;
            Ok(format_latitude_sensitive_house_systems_for_report())
        }
        Some("house-latitude-sensitive-constraints-summary") => {
            ensure_no_extra_args(&args[1..], "house-latitude-sensitive-constraints-summary")?;
            Ok(format_latitude_sensitive_house_constraints_for_report())
        }
        Some("house-latitude-sensitive-failure-modes-summary") => {
            ensure_no_extra_args(&args[1..], "house-latitude-sensitive-failure-modes-summary")?;
            Ok(format_latitude_sensitive_house_failure_modes_for_report())
        }
        Some("house-latitude-sensitive-failure-modes") => {
            ensure_no_extra_args(&args[1..], "house-latitude-sensitive-failure-modes")?;
            Ok(format_latitude_sensitive_house_failure_modes_for_report())
        }
        Some("house-latitude-sensitive-constraints") => {
            ensure_no_extra_args(&args[1..], "house-latitude-sensitive-constraints")?;
            Ok(format_latitude_sensitive_house_constraints_for_report())
        }
        Some("house-latitude-sensitive") => {
            ensure_no_extra_args(&args[1..], "house-latitude-sensitive")?;
            Ok(format_latitude_sensitive_house_systems_for_report())
        }
        Some("release-house-validation-summary") => {
            ensure_no_extra_args(&args[1..], "release-house-validation-summary")?;
            Ok(release_house_validation_summary_for_report())
        }
        Some("release-house-validation") => {
            ensure_no_extra_args(&args[1..], "release-house-validation")?;
            Ok(release_house_validation_summary_for_report())
        }
        Some("house-formula-families-summary") => {
            ensure_no_extra_args(&args[1..], "house-formula-families-summary")?;
            Ok(format_house_formula_families_for_report())
        }
        Some("house-formula-families") => {
            ensure_no_extra_args(&args[1..], "house-formula-families")?;
            Ok(format_house_formula_families_for_report())
        }
        Some("house-code-aliases-summary") => {
            ensure_no_extra_args(&args[1..], "house-code-aliases-summary")?;
            Ok(format_house_code_aliases_for_report())
        }
        Some("house-code-alias-summary") => {
            ensure_no_extra_args(&args[1..], "house-code-alias-summary")?;
            Ok(format_house_code_aliases_for_report())
        }
        Some("ayanamsa-catalog-validation-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-catalog-validation-summary")?;
            Ok(format_ayanamsa_catalog_validation_for_report())
        }
        Some("ayanamsa-catalog-validation") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-catalog-validation")?;
            Ok(format_ayanamsa_catalog_validation_for_report())
        }
        Some("ayanamsa-metadata-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-metadata-coverage-summary")?;
            Ok(format_ayanamsa_metadata_coverage_for_report())
        }
        Some("ayanamsa-metadata-coverage") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-metadata-coverage")?;
            Ok(format_ayanamsa_metadata_coverage_for_report())
        }
        Some("ayanamsa-reference-offsets-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-reference-offsets-summary")?;
            Ok(format_ayanamsa_reference_offsets_for_report())
        }
        Some("ayanamsa-reference-offsets") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-reference-offsets")?;
            Ok(format_ayanamsa_reference_offsets_for_report())
        }
        Some("ayanamsa-provenance-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-provenance-summary")?;
            Ok(format_ayanamsa_provenance_for_report())
        }
        Some("ayanamsa-provenance") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-provenance")?;
            Ok(format_ayanamsa_provenance_for_report())
        }
        Some("frame-policy-summary") => {
            ensure_no_extra_args(&args[1..], "frame-policy-summary")?;
            Ok(render_frame_policy_summary_text())
        }
        Some("frame-policy") => {
            ensure_no_extra_args(&args[1..], "frame-policy")?;
            Ok(render_frame_policy_summary_text())
        }
        Some("mean-obliquity-frame-round-trip-summary") => {
            ensure_no_extra_args(&args[1..], "mean-obliquity-frame-round-trip-summary")?;
            Ok(mean_obliquity_frame_round_trip_summary_for_report())
        }
        Some("mean-obliquity-frame-round-trip") => {
            ensure_no_extra_args(&args[1..], "mean-obliquity-frame-round-trip")?;
            Ok(mean_obliquity_frame_round_trip_summary_for_report())
        }
        Some("release-profile-identifiers-summary") => {
            ensure_no_extra_args(&args[1..], "release-profile-identifiers-summary")?;
            Ok(render_release_profile_identifiers_summary())
        }
        Some("release-profile-identifiers") => {
            ensure_no_extra_args(&args[1..], "release-profile-identifiers")?;
            Ok(render_release_profile_identifiers_summary())
        }
        Some("request-surface-summary") => {
            ensure_no_extra_args(&args[1..], "request-surface-summary")?;
            Ok(render_request_surface_summary_text())
        }
        Some("request-surface") => {
            ensure_no_extra_args(&args[1..], "request-surface")?;
            Ok(render_request_surface_summary_text())
        }
        Some("request-policy-summary") => {
            ensure_no_extra_args(&args[1..], "request-policy-summary")?;
            Ok(render_request_policy_summary_text())
        }
        Some("request-policy") => {
            ensure_no_extra_args(&args[1..], "request-policy")?;
            Ok(render_request_policy_summary_text())
        }
        Some("request-semantics-summary") => {
            ensure_no_extra_args(&args[1..], "request-semantics-summary")?;
            Ok(render_request_semantics_summary_text())
        }
        Some("request-semantics") => {
            ensure_no_extra_args(&args[1..], "request-semantics")?;
            Ok(render_request_semantics_summary_text())
        }
        Some("unsupported-modes-summary") => {
            ensure_no_extra_args(&args[1..], "unsupported-modes-summary")?;
            Ok(render_unsupported_modes_summary_text())
        }
        Some("unsupported-modes") => {
            ensure_no_extra_args(&args[1..], "unsupported-modes")?;
            Ok(render_unsupported_modes_summary_text())
        }
        Some("comparison-tolerance-policy-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-tolerance-policy-summary")?;
            Ok(render_comparison_tolerance_policy_summary_text())
        }
        Some("comparison-tolerance-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-tolerance-summary")?;
            Ok(render_comparison_tolerance_policy_summary_text())
        }

        Some("comparison-tolerance-scope-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-tolerance-scope-coverage-summary")?;
            Ok(render_comparison_tolerance_scope_coverage_summary_text())
        }
        Some("comparison-tolerance-scope-coverage") => {
            ensure_no_extra_args(&args[1..], "comparison-tolerance-scope-coverage")?;
            Ok(render_comparison_tolerance_scope_coverage_summary_text())
        }
        Some("comparison-body-class-tolerance-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-body-class-tolerance-summary")?;
            Ok(render_comparison_body_class_tolerance_summary_text())
        }
        Some("comparison-body-class-error-envelope-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-body-class-error-envelope-summary")?;
            Ok(render_comparison_body_class_error_envelope_summary_text())
        }
        Some("comparison-body-class-error-envelope") => {
            ensure_no_extra_args(&args[1..], "comparison-body-class-error-envelope")?;
            Ok(render_comparison_body_class_error_envelope_summary_text())
        }
        Some("comparison-body-class-tolerance-posture-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "comparison-body-class-tolerance-posture-summary",
            )?;
            Ok(render_comparison_body_class_tolerance_posture_summary_text())
        }
        Some("comparison-body-class-tolerance-posture") => {
            ensure_no_extra_args(&args[1..], "comparison-body-class-tolerance-posture")?;
            Ok(render_comparison_body_class_tolerance_posture_summary_text())
        }
        Some("comparison-body-class-tolerance") => {
            ensure_no_extra_args(&args[1..], "comparison-body-class-tolerance")?;
            Ok(render_comparison_body_class_tolerance_summary_text())
        }
        Some("comparison-envelope-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-envelope-summary")?;
            Ok(render_comparison_envelope_summary_text())
        }
        Some("comparison-envelope") => {
            ensure_no_extra_args(&args[1..], "comparison-envelope")?;
            Ok(render_comparison_envelope_summary_text())
        }
        Some("release-body-claims-summary") => {
            ensure_no_extra_args(&args[1..], "release-body-claims-summary")?;
            Ok(render_release_body_claims_summary_text())
        }
        Some("body-claims-summary") => {
            ensure_no_extra_args(&args[1..], "body-claims-summary")?;
            Ok(render_release_body_claims_summary_text())
        }
        Some("body-date-channel-claims-summary") => {
            ensure_no_extra_args(&args[1..], "body-date-channel-claims-summary")?;
            Ok(render_body_date_channel_claims_summary_text())
        }
        Some("body-date-channel-claims") => {
            ensure_no_extra_args(&args[1..], "body-date-channel-claims")?;
            Ok(render_body_date_channel_claims_summary_text())
        }
        Some("pluto-fallback-summary") => {
            ensure_no_extra_args(&args[1..], "pluto-fallback-summary")?;
            Ok(render_pluto_fallback_summary_text())
        }
        Some("pluto-fallback") => {
            ensure_no_extra_args(&args[1..], "pluto-fallback")?;
            Ok(render_pluto_fallback_summary_text())
        }
        Some("bundle-release") => {
            let (output_dir, rounds) =
                parse_release_bundle_args(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_release_bundle(rounds, output_dir)
                .map(|bundle| bundle.to_string())
                .map_err(render_release_bundle_error)
        }
        Some("verify-release-bundle") => {
            let (output_dir, _) = parse_release_bundle_args(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            verify_release_bundle(output_dir)
                .map(|bundle| bundle.to_string())
                .map_err(render_release_bundle_error)
        }
        Some("help") | Some("--help") | Some("-h") | None => Ok(help_text()),
        Some(other) => Err(format!("unknown command: {other}\n\n{}", help_text())),
    }
}

fn ensure_no_extra_args(args: &[&str], command: &str) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("{command} does not accept extra arguments"))
    }
}

fn parse_rounds(args: &[&str], default: usize) -> Result<usize, String> {
    let mut rounds = default;
    let mut saw_rounds = false;
    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--rounds" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value for --rounds".to_string())?;
                if saw_rounds {
                    return Err("duplicate value for --rounds argument".to_string());
                }
                saw_rounds = true;
                rounds = value
                    .parse::<usize>()
                    .map_err(|error| format!("invalid value for --rounds: {error}"))?;
                if rounds == 0 {
                    return Err(
                        "invalid value for --rounds: expected a positive integer".to_string()
                    );
                }
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(rounds)
}

fn help_text() -> String {
    let corpus_size = default_corpus().requests.len();
    format!(
        "{banner}\n\nCommands:\n  compare-backends          Compare the JPL snapshot against the algorithmic composite backend\n  comparison-report         Alias for compare-backends\n  compare-backends-audit    Compare the JPL snapshot against the algorithmic composite backend and fail if the tolerance audit reports regressions\n  comparison-audit-summary  Print the compact release-grade comparison-audit summary\n  comparison-audit         Alias for comparison-audit-summary\n  backend-matrix            Print the implemented backend capability matrices\n  capability-matrix         Alias for backend-matrix\n  backend-matrix-summary    Print the compact backend capability matrix summary\n  matrix-summary            Alias for backend-matrix-summary\n  compatibility-profile     Print the release compatibility profile\n  profile                   Alias for compatibility-profile\n  benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus and full chart assembly on representative house scenarios\n  benchmark-matrix-summary [--rounds N]  Print the compact benchmark matrix summary\n  benchmark-matrix [--rounds N]  Alias for benchmark-matrix-summary\n  comparison-corpus-summary  Print the compact release-grade comparison corpus summary\n  comparison-corpus         Alias for comparison-corpus-summary\n  comparison-corpus-release-guard-summary  Print the compact release-grade comparison corpus guard summary\n  comparison-corpus-release-guard  Alias for comparison-corpus-release-guard-summary\n  comparison-corpus-guard-summary  Alias for comparison-corpus-release-guard-summary\n  comparison-corpus-guard       Alias for comparison-corpus-guard-summary\n  comparison-envelope-summary  Print the compact comparison envelope summary\n  comparison-envelope       Alias for comparison-envelope-summary\n  comparison-tolerance-policy-summary  Print the compact comparison tolerance policy summary\n  comparison-tolerance-summary  Alias for comparison-tolerance-policy-summary\n  comparison-tolerance-scope-coverage-summary  Print the compact comparison tolerance scope coverage summary\n  comparison-tolerance-scope-coverage  Alias for comparison-tolerance-scope-coverage-summary\n  comparison-body-class-tolerance-summary  Print the compact comparison body-class tolerance summary\n  comparison-body-class-tolerance  Alias for comparison-body-class-tolerance-summary\n  comparison-body-class-error-envelope-summary  Print the compact comparison body-class error envelope summary\n  comparison-body-class-error-envelope  Alias for comparison-body-class-error-envelope-summary\n  comparison-body-class-tolerance-posture-summary  Print the compact comparison body-class tolerance posture summary\n  comparison-body-class-tolerance-posture  Alias for comparison-body-class-tolerance-posture-summary\n  release-body-claims-summary  Print the compact release-grade body claims summary\n  body-claims-summary          Alias for release-body-claims-summary\n  body-date-channel-claims-summary  Print the compact body/date/channel claims summary\n  body-date-channel-claims     Alias for body-date-channel-claims-summary\n  benchmark-corpus-summary  Print the compact representative benchmark corpus summary\n  chart-benchmark-corpus-summary  Print the compact chart benchmark corpus summary\n  chart-benchmark-corpus  Alias for chart-benchmark-corpus-summary\n  report [--rounds N]       Render the full validation report\n  generate-report           Alias for report\n  validation-report-summary [--rounds N]  Render a compact validation report summary\n  report-summary [--rounds N]  Alias for validation-report-summary\n  validation-summary        Alias for validation-report-summary\n  validate-artifact         Inspect and validate the bundled compressed artifact\n  generate-packaged-artifact  Generate or verify the packaged artifact fixture from the checked-in reference snapshot; pass a file path, --out FILE, --output FILE, --manifest-out FILE, --manifest-summary-out FILE, --manifest-checksum-out FILE, --artifact-checksum-out FILE, --normalized-intermediate-summary-out FILE, or --check\n  regenerate-packaged-artifact  Alias for generate-packaged-artifact\n  artifact-summary          Print the compact packaged-artifact summary\n  artifact-posture-summary  Alias for artifact-summary\n  artifact-boundary-envelope-summary  Print the compact packaged-artifact boundary envelope summary\n  artifact-profile-coverage-summary  Print the packaged-artifact profile coverage summary\n  packaged-artifact-output-support-summary  Print the packaged-artifact output support summary\n  packaged-artifact-output-support       Alias for packaged-artifact-output-support-summary\n  packaged-artifact-body-class-span-cap-summary  Print the packaged-artifact body-class span caps summary\n  packaged-artifact-body-class-span-cap  Alias for packaged-artifact-body-class-span-cap-summary\n  packaged-artifact-speed-policy-summary  Print the packaged-artifact speed policy summary\n  packaged-artifact-speed-policy       Alias for packaged-artifact-speed-policy-summary\n  motion-policy-summary         Print the compact motion policy summary\n  motion-policy               Alias for motion-policy-summary\n  packaged-artifact-access-summary  Print the packaged-artifact access summary\n  packaged-artifact-access  Alias for packaged-artifact-access-summary\n  packaged-artifact-path-policy-summary  Alias for packaged-artifact-access-summary\n  packaged-artifact-path-policy  Alias for packaged-artifact-path-policy-summary\n  packaged-artifact-storage-summary  Print the packaged-artifact storage/reconstruction summary\n  packaged-artifact-storage           Alias for packaged-artifact-storage-summary\n  packaged-artifact-production-profile-summary  Print the packaged-artifact production profile draft summary\n  packaged-artifact-production-profile  Alias for packaged-artifact-production-profile-summary\n  packaged-artifact-target-threshold-summary  Print the packaged-artifact target thresholds summary\n  packaged-artifact-target-threshold  Alias for packaged-artifact-target-threshold-summary\n  packaged-artifact-target-threshold-state-summary  Print the packaged-artifact target-threshold state summary\n  packaged-artifact-target-threshold-state  Alias for packaged-artifact-target-threshold-state-summary\n  packaged-artifact-target-threshold-scope-envelopes-summary  Print the packaged-artifact target-threshold scope envelopes summary\n  packaged-artifact-target-threshold-scope-envelopes  Alias for packaged-artifact-target-threshold-scope-envelopes-summary\n  packaged-artifact-phase2-corpus-alignment-summary  Print the packaged-artifact phase-2 corpus alignment summary\n  packaged-artifact-phase2-corpus-alignment  Alias for packaged-artifact-phase2-corpus-alignment-summary\n  packaged-artifact-source-fit-holdout-sync-summary  Print the packaged-artifact source-fit and hold-out sync summary\n  packaged-artifact-source-fit-holdout-sync  Alias for packaged-artifact-source-fit-holdout-sync-summary\n  packaged-artifact-fit-envelope-summary  Print the packaged-artifact fit envelope summary\n  packaged-artifact-fit-envelope  Alias for packaged-artifact-fit-envelope-summary\n  packaged-artifact-fit-margins-summary  Print the packaged-artifact fit margins summary\n  packaged-artifact-fit-margins      Alias for packaged-artifact-fit-margins-summary\n  packaged-artifact-fit-sample-classes-summary  Print the packaged-artifact fit sample classes summary\n  packaged-artifact-fit-sample-classes  Alias for packaged-artifact-fit-sample-classes-summary\n  packaged-artifact-body-cadence-summary  Print the packaged-artifact body cadence summary\n  packaged-artifact-body-cadence         Alias for packaged-artifact-body-cadence-summary\n  packaged-artifact-fit-outliers-summary  Print the packaged-artifact body/channel fit outlier summary\n  packaged-artifact-fit-outliers  Alias for packaged-artifact-fit-outliers-summary\n  packaged-artifact-fit-threshold-violation-count-summary  Print the packaged-artifact fit threshold violation count summary\n  packaged-artifact-fit-threshold-violation-count  Alias for packaged-artifact-fit-threshold-violation-count-summary\n  packaged-artifact-fit-threshold-violations-summary  Print the packaged-artifact fit threshold violations summary\n  packaged-artifact-fit-threshold-violations  Alias for packaged-artifact-fit-threshold-violations-summary\n  packaged-artifact-generation-manifest-summary  Print the packaged-artifact generation manifest summary\n  packaged-artifact-generation-manifest  Alias for packaged-artifact-generation-manifest-summary\n  packaged-artifact-generation-manifest-checksum-summary  Print the packaged-artifact generation manifest checksum summary\n  packaged-artifact-generation-manifest-checksum  Alias for packaged-artifact-generation-manifest-checksum-summary\n  packaged-artifact-generation-policy-summary  Print the packaged-artifact generation policy summary\n  packaged-artifact-generation-policy     Alias for packaged-artifact-generation-policy-summary\n  packaged-artifact-normalized-intermediate-summary  Print the packaged-artifact normalized intermediates summary\n  packaged-artifact-normalized-intermediate  Alias for packaged-artifact-normalized-intermediate-summary\n  packaged-artifact-generation-residual-summary  Alias for packaged-artifact-generation-residual-bodies-summary\n  packaged-artifact-generation-residual-bodies-summary  Print the packaged-artifact generation residual bodies summary\n  packaged-artifact-regeneration-summary  Print the packaged-artifact regeneration summary\n  packaged-artifact-regeneration      Alias for packaged-artifact-regeneration-summary\n  packaged-frame-parity-summary  Print the packaged frame parity summary\n  packaged-frame-parity         Alias for packaged-frame-parity-summary\n  packaged-frame-treatment-summary  Print the packaged frame treatment summary\n  packaged-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary\n  packaged-lookup-epoch-policy         Alias for packaged-lookup-epoch-policy-summary\n  packaged-artifact-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary\n  packaged-artifact-lookup-epoch-policy         Alias for packaged-artifact-lookup-epoch-policy-summary\n  workspace-audit           Check the workspace for mandatory native build hooks
  audit                     Alias for workspace-audit
  native-dependency-audit   Alias for workspace-audit
  workspace-audit-summary   Print the compact workspace audit summary
  native-dependency-audit-summary  Alias for workspace-audit-summary
  workspace-provenance-summary  Print the compact workspace provenance summary
  workspace-provenance     Alias for workspace-provenance-summary
  api-stability             Print the release API stability posture\n  api-posture               Alias for api-stability\n  api-stability-summary     Print the compact API stability summary\n  api-posture-summary       Alias for api-stability-summary\n  compatibility-profile-summary  Print the compact compatibility profile summary
  compatibility-caveats-summary  Print the compact compatibility caveats summary
  compatibility-caveats    Alias for compatibility-caveats-summary
  known-gaps-summary      Print the compact compatibility known-gaps summary
  known-gaps              Alias for known-gaps-summary
  catalog-inventory-summary  Print the compact compatibility catalog inventory summary
  catalog-inventory        Alias for catalog-inventory-summary
  catalog-posture-summary   Print the compact compatibility catalog posture summary
  catalog-posture         Alias for catalog-posture-summary
  custom-definition-ayanamsa-labels-summary  Print the compact custom-definition ayanamsa labels summary
  custom-definition-ayanamsa-labels  Alias for custom-definition-ayanamsa-labels-summary
  release-house-system-canonical-names-summary  Print the compact release-specific house-system canonical names summary
  release-house-system-canonical-names  Alias for release-house-system-canonical-names-summary
  release-ayanamsa-canonical-names-summary  Print the compact release-specific ayanamsa canonical names summary
  release-ayanamsa-canonical-names  Alias for release-ayanamsa-canonical-names-summary
  target-house-scope-summary  Print the compact compatibility target house-system scope summary
  target-house-scope         Alias for target-house-scope-summary
  target-ayanamsa-scope-summary  Print the compact compatibility target ayanamsa scope summary
  target-ayanamsa-scope      Alias for target-ayanamsa-scope-summary
  profile-summary           Alias for compatibility-profile-summary
  verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs\n  release-notes             Print the release compatibility notes\n  release-notes-summary     Print the compact release notes summary\n  release-checklist         Print the release maintainer checklist\n  release-checklist-summary Print the compact release checklist summary\n  release-smoke            Run the release smoke checks and render the short smoke report\n  release-gate              Run the release gate checks and render the release checklist\n  release-gate-summary      Run the release gate checks and render the compact release checklist summary\n  checklist-summary        Alias for release-checklist-summary\n  release-summary           Print the compact release summary\n  source-corpus-summary     Print the consolidated source corpus summary\n  source-corpus             Alias for source-corpus-summary\n  source-corpus-posture-summary  Alias for source-corpus-summary\n  source-corpus-posture     Alias for source-corpus-posture-summary\n  jpl-batch-error-taxonomy-summary  Print the compact JPL batch error taxonomy summary\n  jpl-snapshot-evidence-summary  Print the compact combined JPL evidence summary\n  jpl-source-corpus-contract-summary  Print the compact JPL source corpus contract summary\n  jpl-source-corpus-contract  Alias for jpl-source-corpus-contract-summary\n  jpl-source-posture-summary  Print the compact JPL source posture summary\n  jpl-source-posture         Alias for jpl-source-posture-summary\n  jpl-provenance-only-summary  Print the compact JPL provenance-only evidence summary\n  jpl-provenance-only  Alias for jpl-provenance-only-summary\n  production-generation-boundary-summary  Print the compact production-generation boundary overlay summary\n  production-generation-boundary         Alias for production-generation-boundary-summary\n  production-generation-boundary-request-corpus-summary  Print the compact production-generation boundary request corpus summary\n  production-generation-boundary-request-corpus  Alias for production-generation-boundary-request-corpus-summary\n  production-generation-boundary-request-corpus-equatorial-summary  Print the compact production-generation boundary request corpus summary in the equatorial frame\n  production-generation-boundary-request-corpus-equatorial  Alias for production-generation-boundary-request-corpus-equatorial-summary\n  production-generation-body-class-coverage-summary  Print the compact production-generation body-class coverage summary\n  production-body-class-coverage-summary  Alias for production-generation-body-class-coverage-summary\n  production-generation-source-window-summary  Print the compact production-generation source windows summary\n  production-generation-source-window  Alias for production-generation-source-window-summary\n  production-generation-corpus-shape-summary  Print the compact production-generation corpus shape summary\n  production-generation-corpus-shape  Alias for production-generation-corpus-shape-summary\n  production-generation-summary  Print the compact production-generation coverage summary
  production-generation           Alias for production-generation-summary
  production-generation-quarter-day-boundary-summary  Print the compact production-generation quarter-day boundary samples summary
  production-generation-quarter-day-boundary  Alias for production-generation-quarter-day-boundary-summary
  production-generation-boundary-source-summary  Print the compact production-generation boundary source summary
  production-generation-boundary-source  Alias for production-generation-boundary-source-summary
  production-generation-boundary-window-summary  Print the compact production-generation boundary windows summary
  production-generation-boundary-window  Alias for production-generation-boundary-window-summary\n  production-generation-manifest-summary  Print the compact production-generation manifest summary\n  production-generation-manifest  Alias for production-generation-manifest-summary\n  production-generation-manifest-checksum-summary  Print the compact production-generation manifest checksum summary\n  production-generation-manifest-checksum  Alias for production-generation-manifest-checksum-summary\n  production-generation-source      Alias for production-generation-source-summary\n  production-generation-source-summary  Print the compact production-generation source summary\n  production-generation-source-revision-summary  Print the compact production-generation source revision summary\n  production-generation-source-revision  Alias for production-generation-source-revision-summary\n  comparison-snapshot-source-window-summary  Print the compact comparison snapshot source windows summary\n  comparison-snapshot-source-window  Alias for comparison-snapshot-source-window-summary\n  comparison-snapshot-source-summary  Print the compact comparison snapshot source summary\n  comparison-snapshot-source        Alias for comparison-snapshot-source-summary\n  comparison-snapshot-body-class-coverage-summary  Print the compact comparison snapshot body-class coverage summary\n  comparison-body-class-coverage-summary  Alias for comparison-snapshot-body-class-coverage-summary\n  comparison-snapshot-manifest-summary  Print the compact comparison snapshot manifest summary\n  comparison-snapshot-manifest  Alias for comparison-snapshot-manifest-summary\n  comparison-snapshot-summary  Print the compact comparison snapshot summary\n  j2000-snapshot           Alias for comparison-snapshot-summary\n  comparison-snapshot         Alias for comparison-snapshot-summary\n  comparison-snapshot-batch-parity-summary  Print the compact comparison snapshot batch parity summary\n  comparison-snapshot-batch-parity  Alias for comparison-snapshot-batch-parity-summary\n  reference-snapshot-source-window-summary  Print the compact reference snapshot source windows summary\n  reference-snapshot-source-window  Alias for reference-snapshot-source-window-summary\n  reference-snapshot-source-summary  Print the compact reference snapshot source summary\n  reference-snapshot-source        Alias for reference-snapshot-source-summary\n  reference-asteroid-source-window-summary  Print the compact reference asteroid source windows summary
  reference-asteroid-source-window  Alias for reference-asteroid-source-window-summary
  reference-snapshot-manifest-summary  Print the compact reference snapshot manifest summary
  reference-snapshot-manifest  Alias for reference-snapshot-manifest-summary
  reference-snapshot-summary  Print the compact reference snapshot summary
  reference-snapshot         Alias for reference-snapshot-summary
  reference-snapshot-body-class-coverage-summary  Print the compact reference snapshot body-class coverage summary
  reference-body-class-coverage-summary  Alias for reference-snapshot-body-class-coverage-summary
  reference-snapshot-exact-j2000-evidence-summary  Print the compact reference snapshot exact J2000 evidence summary
  reference-snapshot-exact-j2000-evidence  Alias for reference-snapshot-exact-j2000-evidence-summary
  exact-j2000-evidence    Alias for reference-snapshot-exact-j2000-evidence-summary
  reference-snapshot-batch-parity-summary  Print the compact reference snapshot batch parity summary
  reference-snapshot-batch-parity          Alias for reference-snapshot-batch-parity-summary
  reference-snapshot-mixed-time-scale-batch-parity-summary  Print the compact reference snapshot mixed TT/TDB batch parity summary
  reference-snapshot-mixed-tt-tdb-batch-parity-summary  Alias for reference-snapshot-mixed-time-scale-batch-parity-summary
  reference-snapshot-mixed-tt-tdb-batch-parity  Alias for reference-snapshot-mixed-time-scale-batch-parity-summary
  reference-snapshot-equatorial-parity-summary  Print the compact reference snapshot equatorial parity summary\n  reference-snapshot-equatorial-parity     Alias for reference-snapshot-equatorial-parity-summary\n  reference-snapshot-lunar-boundary-summary  Print the compact reference lunar boundary evidence summary\n  lunar-boundary-summary   Alias for reference-snapshot-lunar-boundary-summary\n  reference-snapshot-1500-selected-body-boundary-summary  Print the compact reference 1500 selected-body boundary evidence summary\n  1500-selected-body-boundary-summary  Alias for reference-snapshot-1500-selected-body-boundary-summary\n  reference-snapshot-2268932-selected-body-boundary-summary  Print the compact reference 2268932 selected-body boundary evidence summary\n  2268932-selected-body-boundary-summary  Alias for reference-snapshot-2268932-selected-body-boundary-summary\n  reference-snapshot-1600-selected-body-boundary-summary  Print the compact reference 1600 selected-body boundary evidence summary\n  1600-selected-body-boundary-summary  Alias for reference-snapshot-1600-selected-body-boundary-summary\n  reference-snapshot-2305457-selected-body-boundary-summary  Print the compact reference 2305457 selected-body boundary evidence summary\n  2305457-selected-body-boundary-summary  Alias for reference-snapshot-2305457-selected-body-boundary-summary\n  reference-snapshot-1750-selected-body-boundary-summary  Print the compact reference 1750 selected-body boundary evidence summary\n  1750-selected-body-boundary-summary  Alias for reference-snapshot-1750-selected-body-boundary-summary\n  reference-snapshot-1750-major-body-interior-summary  Print the compact reference 1750 major-body interior comparison evidence summary\n  1750-major-body-interior-summary  Alias for reference-snapshot-1750-major-body-interior-summary\n  reference-snapshot-2360234-major-body-interior-summary  Print the compact reference 2360234 major-body interior comparison evidence summary\n  2360234-major-body-interior-summary  Alias for reference-snapshot-2360234-major-body-interior-summary\n  reference-snapshot-2451916-major-body-interior-summary  Print the compact reference 2451916 major-body interior evidence summary\n  2451916-major-body-interior-summary  Alias for reference-snapshot-2451916-major-body-interior-summary
  reference-snapshot-2451916-major-body-boundary-summary  Print the compact reference 2451916 major-body boundary evidence summary
  2451916-major-body-boundary-summary  Alias for reference-snapshot-2451916-major-body-boundary-summary
  reference-snapshot-2451916-major-body-dense-boundary-summary  Print the compact reference 2451916 major-body dense boundary evidence summary
  2451916-major-body-dense-boundary-summary  Alias for reference-snapshot-2451916-major-body-dense-boundary-summary\n  reference-snapshot-2451917-major-body-boundary-summary  Print the compact reference 2451917 major-body boundary evidence summary\n  2451917-major-body-boundary-summary  Alias for reference-snapshot-2451917-major-body-boundary-summary\n  reference-snapshot-2451910-major-body-boundary-summary  Print the compact reference 2451910 major-body boundary evidence summary\n  2451910-major-body-boundary-summary  Alias for reference-snapshot-2451910-major-body-boundary-summary\n  reference-snapshot-2451911-major-body-boundary-summary  Print the compact reference 2451911 major-body boundary evidence summary\n  2451911-major-body-boundary-summary  Alias for reference-snapshot-2451911-major-body-boundary-summary\n  reference-snapshot-2451912-major-body-boundary-summary  Print the compact reference 2451912 major-body boundary evidence summary\n  2451912-major-body-boundary-summary  Alias for reference-snapshot-2451912-major-body-boundary-summary\n  reference-snapshot-2200-selected-body-boundary-summary  Print the compact reference 2200 selected-body boundary evidence summary\n  2200-selected-body-boundary-summary  Alias for reference-snapshot-2200-selected-body-boundary-summary\n  reference-snapshot-2524593-selected-body-boundary-summary  Print the compact reference 2524593 selected-body boundary evidence summary\n  2524593-selected-body-boundary-summary  Alias for reference-snapshot-2524593-selected-body-boundary-summary
  reference-snapshot-2634167-selected-body-boundary-summary  Print the compact reference 2634167 selected-body boundary evidence summary
  2634167-selected-body-boundary-summary  Alias for reference-snapshot-2634167-selected-body-boundary-summary\n  reference-snapshot-1900-selected-body-boundary-summary  Print the compact reference 1900 selected-body boundary evidence summary\n  1900-selected-body-boundary-summary  Alias for reference-snapshot-1900-selected-body-boundary-summary\n  reference-snapshot-2415020-selected-body-boundary-summary  Print the compact reference 2415020 selected-body boundary evidence summary\n  2415020-selected-body-boundary-summary  Alias for reference-snapshot-2415020-selected-body-boundary-summary\n  reference-snapshot-2500-selected-body-boundary-summary  Print the compact reference 2500 selected-body boundary evidence summary\n  2500-selected-body-boundary-summary  Alias for reference-snapshot-2500-selected-body-boundary-summary\n  reference-snapshot-1749-major-body-boundary-summary  Print the compact reference 1749 major-body boundary evidence summary\n  1749-major-body-boundary-summary  Alias for reference-snapshot-1749-major-body-boundary-summary\n  2360233-major-body-boundary-summary  Alias for reference-snapshot-1749-major-body-boundary-summary\n  reference-snapshot-early-major-body-boundary-summary  Print the compact reference early major-body boundary evidence summary\n  early-major-body-boundary-summary  Alias for reference-snapshot-early-major-body-boundary-summary\n  reference-snapshot-2378498-major-body-boundary-summary  Print the compact reference 2378498 major-body boundary evidence summary\n  2378498-major-body-boundary-summary  Alias for reference-snapshot-2378498-major-body-boundary-summary\n  reference-snapshot-1800-major-body-boundary-summary  Print the compact reference 1800 major-body boundary evidence summary\n  1800-major-body-boundary-summary  Alias for reference-snapshot-1800-major-body-boundary-summary\n  2378499-major-body-boundary-summary  Alias for reference-snapshot-1800-major-body-boundary-summary\n  reference-snapshot-2400000-major-body-boundary-summary  Print the compact reference 2400000 major-body boundary evidence summary\n  2400000-major-body-boundary-summary  Alias for reference-snapshot-2400000-major-body-boundary-summary\n  reference-snapshot-2451545-major-body-boundary-summary  Print the compact reference 2451545 major-body boundary evidence summary\n  2451545-major-body-boundary-summary  Alias for reference-snapshot-2451545-major-body-boundary-summary\n  reference-snapshot-2500-major-body-boundary-summary  Print the compact reference 2500 major-body boundary evidence summary\n  2500-major-body-boundary-summary  Alias for reference-snapshot-2500-major-body-boundary-summary\n  reference-snapshot-2453000-major-body-boundary-summary  Print the compact reference 2453000 major-body boundary evidence summary\n  2453000-major-body-boundary-summary  Alias for reference-snapshot-2453000-major-body-boundary-summary\n  reference-snapshot-2500000-major-body-boundary-summary  Print the compact reference 2500000 major-body boundary evidence summary\n  2500000-major-body-boundary-summary  Alias for reference-snapshot-2500000-major-body-boundary-summary\n  reference-snapshot-2600000-major-body-boundary-summary  Print the compact reference 2600000 major-body boundary evidence summary
  2600000-major-body-boundary-summary  Alias for reference-snapshot-2600000-major-body-boundary-summary
  reference-snapshot-major-body-boundary-summary  Print the compact reference major-body boundary evidence summary
  major-body-boundary-summary  Alias for reference-snapshot-major-body-boundary-summary
  reference-snapshot-major-body-bridge-summary  Print the compact reference major-body bridge evidence summary
  major-body-bridge-summary  Alias for reference-snapshot-major-body-bridge-summary
  bridge-summary           Alias for reference-snapshot-major-body-bridge-summary
  reference-snapshot-major-body-boundary-window-summary  Print the compact reference major-body boundary windows summary
  major-body-boundary-window-summary  Alias for reference-snapshot-major-body-boundary-window-summary
  reference-snapshot-mars-jupiter-boundary-summary  Print the compact reference Mars/Jupiter boundary evidence summary
  mars-jupiter-boundary-summary  Alias for reference-snapshot-mars-jupiter-boundary-summary
  reference-snapshot-mars-outer-boundary-summary  Print the compact reference Mars outer-boundary evidence summary
  mars-outer-boundary-summary  Alias for reference-snapshot-mars-outer-boundary-summary
  reference-high-curvature-summary  Print the compact reference major-body high-curvature evidence summary
  high-curvature-summary  Alias for reference-high-curvature-summary
  reference-snapshot-major-body-high-curvature-summary  Print the compact reference major-body high-curvature evidence summary
  major-body-high-curvature-summary  Alias for reference-snapshot-major-body-high-curvature-summary
  reference-high-curvature-window-summary  Print the compact reference major-body high-curvature windows summary
  high-curvature-window-summary  Alias for reference-high-curvature-window-summary
  reference-snapshot-major-body-high-curvature-window-summary  Print the compact reference major-body high-curvature windows summary
  major-body-high-curvature-window-summary  Alias for reference-snapshot-major-body-high-curvature-window-summary
  reference-high-curvature-epoch-coverage-summary  Print the compact reference major-body high-curvature epoch coverage summary
  high-curvature-epoch-coverage-summary  Alias for reference-high-curvature-epoch-coverage-summary
  reference-snapshot-major-body-high-curvature-epoch-coverage-summary  Print the compact reference major-body high-curvature epoch coverage summary
  major-body-high-curvature-epoch-coverage-summary  Alias for reference-snapshot-major-body-high-curvature-epoch-coverage-summary
  reference-snapshot-sparse-boundary-summary  Print the compact reference sparse boundary summary
  sparse-boundary-summary  Alias for reference-snapshot-sparse-boundary-summary
  reference-snapshot-boundary-day-summary  Print the compact reference snapshot boundary day summary
  reference-snapshot-boundary-day  Alias for reference-snapshot-boundary-day-summary
  boundary-day-summary     Alias for reference-snapshot-boundary-day-summary
  reference-snapshot-bridge-day-summary  Print the compact reference snapshot bridge day summary
  bridge-day-summary       Alias for reference-snapshot-bridge-day-summary
  reference-snapshot-boundary-epoch-coverage-summary  Print the compact reference snapshot boundary epoch coverage summary
  reference-snapshot-boundary-epoch-coverage  Alias for reference-snapshot-boundary-epoch-coverage-summary
  boundary-epoch-coverage-summary  Alias for reference-snapshot-boundary-epoch-coverage-summary
  reference-snapshot-pre-bridge-boundary-summary  Print the compact reference pre-bridge boundary summary
  reference-snapshot-pre-bridge-boundary  Alias for reference-snapshot-pre-bridge-boundary-summary
  pre-bridge-boundary-summary  Alias for reference-snapshot-pre-bridge-boundary-summary
  reference-snapshot-dense-boundary-summary  Print the compact reference dense boundary summary
  dense-boundary-summary  Alias for reference-snapshot-dense-boundary-summary
  reference-snapshot-2451913-major-body-boundary-summary  Print the compact reference 2451913 major-body boundary evidence summary\n  2451913-major-body-boundary-summary  Alias for reference-snapshot-2451913-major-body-boundary-summary\n  reference-snapshot-2451914-major-body-boundary-summary  Print the compact reference 2451914 major-body boundary evidence summary
  2451914-major-body-boundary-summary  Alias for reference-snapshot-2451914-major-body-boundary-summary
  reference-snapshot-2451914-major-body-pre-bridge-summary  Print the compact reference 2451914 major-body pre-bridge boundary evidence summary
  2451914-major-body-pre-bridge-summary  Alias for reference-snapshot-2451914-major-body-pre-bridge-summary
  reference-snapshot-2451914-bridge-day-summary  Print the compact reference 2451914 bridge day summary
  2451914-bridge-day-summary  Alias for reference-snapshot-2451914-bridge-day-summary
  reference-snapshot-2451914-major-body-bridge-day-summary  Print the compact reference 2451914 major-body bridge-day summary
  2451914-major-body-bridge-day-summary  Alias for reference-snapshot-2451914-major-body-bridge-day-summary
  reference-snapshot-2451914-major-body-bridge-summary  Print the compact reference 2451914 major-body bridge evidence summary
  2451914-major-body-bridge-summary  Alias for reference-snapshot-2451914-major-body-bridge-summary
  2451914-major-body-bridge  Alias for reference-snapshot-2451914-major-body-bridge-summary
  reference-snapshot-2451915-major-body-boundary-summary  Print the compact reference 2451915 major-body boundary evidence summary\n  2451915-major-body-boundary-summary  Alias for reference-snapshot-2451915-major-body-boundary-summary\n  reference-snapshot-2451915-major-body-bridge-summary  Print the compact reference 2451915 major-body bridge evidence summary\n  2451915-major-body-bridge-summary  Alias for reference-snapshot-2451915-major-body-bridge-summary\n  2451915-major-body-bridge  Alias for reference-snapshot-2451915-major-body-bridge-summary\n  reference-snapshot-2451917-major-body-boundary-summary  Print the compact reference 2451917 major-body boundary evidence summary
  2451917-major-body-boundary-summary  Alias for reference-snapshot-2451917-major-body-boundary-summary
  reference-snapshot-2451917-major-body-bridge-summary  Print the compact reference 2451917 major-body bridge evidence summary
  2451917-major-body-bridge-summary  Alias for reference-snapshot-2451917-major-body-bridge-summary
  2451917-major-body-bridge  Alias for reference-snapshot-2451917-major-body-bridge-summary
  reference-snapshot-2451918-major-body-boundary-summary  Print the compact reference 2451918 major-body boundary evidence summary
  2451918-major-body-boundary-summary  Alias for reference-snapshot-2451918-major-body-boundary-summary
  reference-snapshot-2451919-major-body-boundary-summary  Print the compact reference 2451919 major-body boundary evidence summary
  2451919-major-body-boundary-summary  Alias for reference-snapshot-2451919-major-body-boundary-summary
  reference-snapshot-2600000-major-body-boundary-summary  Print the compact reference 2600000 major-body boundary evidence summary
  2600000-major-body-boundary-summary  Alias for reference-snapshot-2600000-major-body-boundary-summary
  reference-snapshot-2451920-major-body-interior-summary  Print the compact reference 2451920 major-body interior evidence summary
  2451920-major-body-interior-summary  Alias for reference-snapshot-2451920-major-body-interior-summary
  source-documentation-summary  Print the compact VSOP87 source-documentation summary\n  source-documentation         Alias for source-documentation-summary\n  source-documentation-health-summary  Print the compact VSOP87 source-documentation health summary\n  source-documentation-health  Alias for source-documentation-health-summary\n  source-audit-summary      Print the compact VSOP87 source audit summary\n  source-audit              Alias for source-audit-summary\n  generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary\n  generated-binary-audit    Alias for generated-binary-audit-summary\n  time-scale-policy-summary  Print the compact time-scale policy summary\n  time-scale-policy       Alias for time-scale-policy-summary\n  utc-convenience-policy-summary  Print the compact UTC convenience policy summary\n  utc-convenience-policy  Alias for utc-convenience-policy-summary\n  delta-t-policy-summary   Print the compact Delta T policy summary\n  delta-t-policy         Alias for delta-t-policy-summary\n  zodiac-policy-summary  Print the compact zodiac policy summary\n  zodiac-policy         Alias for zodiac-policy-summary\n  observer-policy-summary  Print the compact observer policy summary\n  observer-policy        Alias for observer-policy-summary\n  apparentness-policy-summary  Print the compact apparentness policy summary\n  apparentness-policy     Alias for apparentness-policy-summary\n  native-sidereal-policy-summary  Print the compact native sidereal policy summary\n  native-sidereal-policy   Alias for native-sidereal-policy-summary\n  interpolation-posture-summary  Print the compact JPL interpolation posture summary\n  interpolation-posture         Alias for interpolation-posture-summary\n  interpolation-quality-summary  Print the compact JPL interpolation quality summary\n  interpolation-quality-kind-coverage-summary  Print the compact JPL interpolation quality kind coverage summary\n  interpolation-quality-request-corpus-summary  Print the compact JPL interpolation-quality sample request corpus summary\n  interpolation-quality-request-corpus  Alias for interpolation-quality-request-corpus-summary\n  lunar-reference-error-envelope-summary  Print the compact lunar reference error envelope summary\n  lunar-reference-error-envelope  Alias for lunar-reference-error-envelope-summary\n  lunar-reference-evidence-summary  Print the compact lunar reference evidence summary\n  lunar-reference-evidence  Alias for lunar-reference-evidence-summary\n  lunar-equatorial-reference-error-envelope-summary  Print the compact lunar equatorial reference error envelope summary\n  lunar-equatorial-reference-error-envelope  Alias for lunar-equatorial-reference-error-envelope-summary\n  lunar-apparent-comparison-summary  Print the compact lunar apparent comparison summary\n  lunar-apparent-comparison  Alias for lunar-apparent-comparison-summary\n  lunar-source-window-summary  Print the compact lunar source windows summary\n  reference-snapshot-lunar-source-window-summary  Alias for lunar-source-window-summary\n  lunar-source-window  Alias for lunar-source-window-summary\n  lunar-reference-mixed-time-scale-batch-parity-summary  Print the compact lunar reference mixed TT/TDB batch parity summary\n  lunar-reference-mixed-tt-tdb-batch-parity-summary  Alias for lunar-reference-mixed-time-scale-batch-parity-summary\n  lunar-reference-mixed-tt-tdb-batch-parity  Alias for lunar-reference-mixed-time-scale-batch-parity-summary\n  lunar-theory-request-policy-summary  Print the compact ELP lunar request policy summary\n  lunar-theory-request-policy  Alias for lunar-theory-request-policy-summary\n  lunar-theory-frame-treatment-summary  Print the compact ELP lunar frame treatment summary\n  lunar-theory-frame-treatment  Alias for lunar-theory-frame-treatment-summary\n  lunar-theory-limitations-summary  Print the compact ELP lunar limitations summary\n  lunar-theory-limitations   Alias for lunar-theory-limitations-summary\n  lunar-theory-summary      Print the compact ELP lunar theory specification\n  lunar-theory-capability-summary  Print the compact ELP lunar capability summary\n  lunar-theory-source-summary  Print the compact ELP lunar source summary\n  lunar-theory-source-selection-summary  Print the compact ELP lunar source selection summary\n  lunar-theory-source-selection  Alias for lunar-theory-source-selection-summary\n  lunar-theory-source-family-summary  Print the compact ELP lunar source family summary\n  lunar-theory-source-family  Alias for lunar-theory-source-family-summary\n  lunar-theory-catalog-summary  Print the compact ELP lunar theory catalog summary\n  lunar-theory-catalog-validation-summary  Print the compact ELP lunar theory catalog validation summary\n  lunar-theory-catalog      Alias for lunar-theory-catalog-summary\n  lunar-theory-catalog-validation  Alias for lunar-theory-catalog-validation-summary\n  selected-asteroid-boundary-summary  Print the compact selected-asteroid boundary evidence summary\n  reference-snapshot-selected-asteroid-bridge-summary  Print the compact selected-asteroid bridge evidence summary\n  selected-asteroid-bridge-summary  Alias for reference-snapshot-selected-asteroid-bridge-summary\n  reference-snapshot-selected-asteroid-dense-boundary-summary  Print the compact selected-asteroid dense boundary evidence summary\n  selected-asteroid-dense-boundary-summary  Alias for reference-snapshot-selected-asteroid-dense-boundary-summary\n  reference-snapshot-selected-asteroid-terminal-boundary-summary  Print the compact selected-asteroid terminal boundary evidence summary\n  selected-asteroid-terminal-boundary-summary  Alias for reference-snapshot-selected-asteroid-terminal-boundary-summary\n  selected-asteroid-source-evidence-summary  Print the compact selected-asteroid source evidence summary\n  reference-snapshot-selected-asteroid-source-summary  Print the compact selected-asteroid source evidence summary\n  selected-asteroid-source-summary  Alias for selected-asteroid-source-evidence-summary\n  selected-asteroid-source-request-corpus-summary  Print the compact selected-asteroid source request corpus summary\n  selected-asteroid-source-request-corpus  Alias for selected-asteroid-source-request-corpus-summary\n  selected-asteroid-source-request-corpus-equatorial-summary  Print the compact selected-asteroid source request corpus summary in the equatorial frame\n  selected-asteroid-source-request-corpus-equatorial  Alias for selected-asteroid-source-request-corpus-equatorial-summary\n  selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary\n  reference-snapshot-selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary\n  reference-snapshot-selected-asteroid-source-window  Alias for reference-snapshot-selected-asteroid-source-window-summary\n  reference-snapshot-2378498-selected-asteroid-source-summary  Print the compact reference selected-asteroid 2378498.5 source evidence summary\n  2378498-selected-asteroid-source-summary  Alias for reference-snapshot-2378498-selected-asteroid-source-summary\n  reference-snapshot-2451917-selected-asteroid-source-summary  Print the compact reference selected-asteroid 2001-01-08 source evidence summary\n  2451917-selected-asteroid-source-summary  Alias for reference-snapshot-2451917-selected-asteroid-source-summary\n  reference-snapshot-2453000-selected-asteroid-source-summary  Print the compact reference 2003-12-27 selected-asteroid source evidence summary\n  2453000-selected-asteroid-source-summary  Alias for reference-snapshot-2453000-selected-asteroid-source-summary\n  reference-snapshot-2500000-selected-asteroid-source-summary  Print the compact reference selected-asteroid 2500000 source evidence summary\n  2500000-selected-asteroid-source-summary  Alias for reference-snapshot-2500000-selected-asteroid-source-summary\n  reference-snapshot-2634167-selected-asteroid-source-summary  Print the compact reference selected-asteroid 2634167 source evidence summary\n  2634167-selected-asteroid-source-summary  Alias for reference-snapshot-2634167-selected-asteroid-source-summary\n  selected-asteroid-source-window  Alias for selected-asteroid-source-window-summary\n  selected-asteroid-batch-parity-summary  Print the compact selected-asteroid batch-parity summary\n  selected-asteroid-batch-parity  Alias for selected-asteroid-batch-parity-summary\n  reference-asteroid-evidence-summary  Print the compact reference asteroid evidence summary\n  reference-asteroid-equatorial-evidence-summary  Print the compact reference asteroid equatorial evidence summary\n  reference-asteroid-equatorial-evidence  Alias for reference-asteroid-equatorial-evidence-summary\n  reference-asteroid-source-window-summary  Print the compact reference asteroid source windows summary\n  reference-asteroid-source-summary  Alias for reference-asteroid-source-window-summary\n  reference-holdout-overlap-summary  Print the compact reference/hold-out overlap summary\n  holdout-overlap-summary   Alias for reference-holdout-overlap-summary\n  independent-holdout-source-window-summary  Print the compact independent hold-out source windows summary\n  independent-holdout-manifest-summary  Print the compact independent hold-out manifest summary\n  independent-holdout-manifest            Alias for independent-holdout-manifest-summary\n  independent-holdout-quarter-day-boundary-summary  Print the compact independent hold-out quarter-day boundary samples summary\n  independent-holdout-quarter-day-boundary  Alias for independent-holdout-quarter-day-boundary-summary\n  independent-holdout-summary  Print the compact independent hold-out summary\n  independent-holdout-source-summary  Print the compact independent hold-out source summary\n  independent-holdout-high-curvature-summary  Print the compact independent hold-out high-curvature evidence summary\n  holdout-high-curvature-summary  Alias for independent-holdout-high-curvature-summary\n  independent-holdout-body-class-coverage-summary  Print the compact independent hold-out body-class coverage summary\n  holdout-body-class-coverage-summary  Alias for independent-holdout-body-class-coverage-summary\n  independent-holdout-batch-parity-summary  Print the compact independent hold-out batch parity summary\n  independent-holdout-batch-parity  Alias for independent-holdout-batch-parity-summary\n  independent-holdout-equatorial-parity-summary  Print the compact independent hold-out equatorial parity summary\n  independent-holdout-equatorial-parity  Alias for independent-holdout-equatorial-parity-summary\n  house-validation-summary   Print the compact house-validation corpus summary\n  house-validation            Alias for house-validation-summary\n  release-house-validation-summary  Print the compact release house-validation corpus summary\n  release-house-validation  Alias for release-house-validation-summary\n  house-formula-families-summary  Print the compact house formula families summary\n  house-formula-families    Alias for house-formula-families-summary\n  house-latitude-sensitive-summary  Print the compact latitude-sensitive house systems summary\n  house-latitude-sensitive-constraints-summary  Print the compact latitude-sensitive house constraints summary\n  house-latitude-sensitive-failure-modes-summary  Print the compact latitude-sensitive house failure modes summary\n  house-latitude-sensitive-failure-modes  Alias for house-latitude-sensitive-failure-modes-summary\n  house-latitude-sensitive-constraints  Alias for house-latitude-sensitive-constraints-summary\n  house-latitude-sensitive  Alias for house-latitude-sensitive-summary\n  house-code-aliases-summary  Print the compact house-code alias summary\n  house-code-alias-summary  Alias for house-code-aliases-summary\n  ayanamsa-catalog-validation-summary  Print the compact ayanamsa catalog validation summary\n  ayanamsa-catalog-validation  Alias for ayanamsa-catalog-validation-summary\n  ayanamsa-metadata-coverage-summary  Print the compact ayanamsa sidereal metadata coverage summary\n  ayanamsa-metadata-coverage  Alias for ayanamsa-metadata-coverage-summary\n  ayanamsa-reference-offsets-summary  Print the compact ayanamsa reference offsets summary\n  ayanamsa-reference-offsets  Alias for ayanamsa-reference-offsets-summary\n  ayanamsa-provenance-summary  Print the compact ayanamsa provenance summary\n  ayanamsa-provenance        Alias for ayanamsa-provenance-summary\n  ayanamsa-audit-summary    Print the compact ayanamsa audit summary\n  ayanamsa-audit            Alias for ayanamsa-audit-summary\n  frame-policy-summary      Print the compact frame-policy summary\n  frame-policy             Alias for frame-policy-summary\n  mean-obliquity-frame-round-trip-summary  Print the compact mean-obliquity frame round-trip summary\n  mean-obliquity-frame-round-trip  Alias for mean-obliquity-frame-round-trip-summary\n  release-profile-identifiers-summary  Print the compact release-profile identifiers summary\n  release-profile-identifiers  Alias for release-profile-identifiers-summary\n  request-surface-summary  Print the compact request-surface inventory summary\n  request-surface         Alias for request-surface-summary\n  request-policy-summary    Print the compact request-policy summary\n  request-policy           Alias for request-policy-summary\n  request-semantics-summary  Print the compact request-semantics summary\n  request-semantics        Alias for request-semantics-summary\n  unsupported-modes-summary  Print the compact unsupported modes summary\n  unsupported-modes        Alias for unsupported-modes-summary\n  comparison-tolerance-policy-summary  Print the compact comparison tolerance policy summary\n  comparison-tolerance-summary  Alias for comparison-tolerance-policy-summary\n  pluto-fallback-summary   Print the compact Pluto fallback summary\n  pluto-fallback           Alias for pluto-fallback-summary\n  bundle-release --out DIR  Write the release compatibility profile, profile summary, release notes, release notes summary, release summary, release-profile identifiers, release-profile identifiers summary, release-house-system-canonical-names summary, release-ayanamsa-canonical-names summary, release-house-validation summary, house-code-aliases summary, house-formula-families summary, house-latitude-sensitive summary, house-latitude-sensitive constraints summary, house-latitude-sensitive failure-modes summary, release checklist, release checklist summary, backend matrix, backend matrix summary, API posture, API stability summary, comparison-corpus summary, source-corpus summary, comparison-snapshot summary, comparison-snapshot source summary, comparison-snapshot body-class coverage summary, comparison-snapshot manifest summary, comparison-envelope summary, comparison-body-class-tolerance summary, comparison-body-class-error-envelope summary, comparison-corpus release-guard summary, comparison-corpus guard summary, request policy summary, observer policy summary, apparentness policy summary, request-semantics summary, unsupported modes summary, time-scale policy summary, UTC convenience policy summary, delta-t policy summary, zodiac policy summary, native sidereal policy summary, request surface summary, compatibility-caveats summary, workspace provenance summary, workspace audit summary, native-dependency audit summary, reference-holdout overlap summary, reference snapshot bridge day summary, reference snapshot 2451916 major-body dense boundary summary, reference snapshot sparse boundary summary, reference snapshot summary, reference snapshot source window summary, reference snapshot manifest summary, reference snapshot body-class coverage summary, reference snapshot equatorial parity summary, reference asteroid source window summary, reference asteroid equatorial evidence summary, production-generation summary, production-generation source summary, production-generation source revision summary, production-generation manifest summary, production-generation manifest checksum summary, catalog inventory summary, ayanamsa provenance summary, artifact summary, packaged-artifact binary, packaged-artifact checksum sidecar, packaged-artifact profile coverage summary, packaged-artifact access summary, packaged-artifact output support summary, packaged-artifact normalized intermediate summary, packaged-artifact speed policy summary, packaged-artifact storage summary, packaged-artifact production-profile summary, packaged-frame-treatment summary, packaged-artifact target-threshold summary, packaged-artifact target-threshold scope envelopes summary, packaged-artifact phase-2 corpus alignment summary, packaged-artifact source-fit and hold-out sync summary, packaged-artifact lookup-epoch policy summary, packaged-artifact generation policy summary, packaged-artifact generation manifest, packaged-artifact generation manifest summary, packaged-artifact generation manifest checksum summary, packaged-artifact generation manifest checksum sidecar, benchmark-corpus summary, chart-benchmark-corpus summary, selected asteroid source request corpus summary, interpolation-quality request corpus summary, benchmark report, validation report, release-body-claims summary, pluto fallback summary, manifest, and manifest checksum sidecar\n  bundle-release --output DIR  Alias for bundle-release --out DIR\n  verify-release-bundle     Read a staged release bundle back and verify its manifest checksums\n  verify-release-bundle --output DIR  Alias for verify-release-bundle --out DIR\n  help                      Show this help text\n\nDefault benchmark rounds: {DEFAULT_BENCHMARK_ROUNDS}\nDefault comparison corpus size: {corpus_size}",
        banner = banner(),
        corpus_size = corpus_size,
    )
}

fn parse_release_bundle_args(
    args: &[&str],
    default_rounds: usize,
) -> Result<(PathBuf, usize), String> {
    let mut output_dir: Option<PathBuf> = None;
    let mut rounds = default_rounds;
    let mut iter = args.iter().copied();

    while let Some(arg) = iter.next() {
        match arg {
            "--out" | "--output" => {
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;
                if output_dir.is_some() {
                    return Err("duplicate value for --out <dir> argument".to_string());
                }
                output_dir = Some(PathBuf::from(value));
            }
            "--rounds" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value for --rounds".to_string())?;
                if rounds != default_rounds {
                    return Err("duplicate value for --rounds argument".to_string());
                }
                rounds = value
                    .parse::<usize>()
                    .map_err(|error| format!("invalid value for --rounds: {error}"))?;
                if rounds == 0 {
                    return Err(
                        "invalid value for --rounds: expected a positive integer".to_string()
                    );
                }
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let output_dir =
        output_dir.ok_or_else(|| "missing required --out <dir> argument".to_string())?;
    Ok((output_dir, rounds))
}

enum PackagedArtifactCommand {
    Write {
        output_path: String,
        manifest_path: Option<String>,
        manifest_summary_path: Option<String>,
        manifest_checksum_path: Option<String>,
        artifact_checksum_path: Option<String>,
        normalized_intermediate_path: Option<String>,
    },
    Check,
}

fn parse_packaged_artifact_command(args: &[&str]) -> Result<PackagedArtifactCommand, String> {
    if args.is_empty() {
        return Err(
            "missing required output path argument; pass a file path, --out <file>, --output <file>, --manifest-out <file>, --manifest-summary-out <file>, --manifest-checksum-out <file>, --artifact-checksum-out <file>, --normalized-intermediate-summary-out <file>, or --check"
                .to_string(),
        );
    }

    let mut output_path = None;
    let mut manifest_path = None;
    let mut manifest_summary_path = None;
    let mut manifest_checksum_path = None;
    let mut artifact_checksum_path = None;
    let mut normalized_intermediate_path = None;
    let mut check = false;
    let mut iter = args.iter().copied();

    while let Some(arg) = iter.next() {
        match arg {
            "--check" => {
                check = true;
            }
            "--out" | "--output" => {
                let path = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;
                if output_path.replace(path.to_string()).is_some() {
                    return Err(format!("duplicate output path argument: {arg}"));
                }
            }
            "--manifest-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --manifest-out".to_string())?;
                if manifest_path.replace(path.to_string()).is_some() {
                    return Err("duplicate manifest path argument: --manifest-out".to_string());
                }
            }
            "--manifest-summary-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --manifest-summary-out".to_string())?;
                if manifest_summary_path.replace(path.to_string()).is_some() {
                    return Err(
                        "duplicate manifest summary path argument: --manifest-summary-out"
                            .to_string(),
                    );
                }
            }
            "--manifest-checksum-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --manifest-checksum-out".to_string())?;
                if manifest_checksum_path.replace(path.to_string()).is_some() {
                    return Err(
                        "duplicate manifest checksum path argument: --manifest-checksum-out"
                            .to_string(),
                    );
                }
            }
            "--artifact-checksum-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --artifact-checksum-out".to_string())?;
                if artifact_checksum_path.replace(path.to_string()).is_some() {
                    return Err(
                        "duplicate artifact checksum path argument: --artifact-checksum-out"
                            .to_string(),
                    );
                }
            }
            "--normalized-intermediate-summary-out" => {
                let path = iter.next().ok_or_else(|| {
                    "missing value for --normalized-intermediate-summary-out".to_string()
                })?;
                if normalized_intermediate_path
                    .replace(path.to_string())
                    .is_some()
                {
                    return Err(
                        "duplicate normalized intermediate path argument: --normalized-intermediate-summary-out"
                            .to_string(),
                    );
                }
            }
            other if other.starts_with('-') => return Err(format!("unknown argument: {other}")),
            path => {
                if output_path.replace(path.to_string()).is_some() {
                    return Err(format!("unexpected positional output path: {path}"));
                }
            }
        }
    }

    if check {
        if output_path.is_some()
            || manifest_path.is_some()
            || manifest_summary_path.is_some()
            || manifest_checksum_path.is_some()
            || artifact_checksum_path.is_some()
            || normalized_intermediate_path.is_some()
        {
            return Err("the --check flag cannot be combined with output paths".to_string());
        }
        return Ok(PackagedArtifactCommand::Check);
    }

    let output_path = output_path.ok_or_else(|| {
        "missing required output path argument; pass a file path, --out <file>, --output <file>, --manifest-out <file>, --manifest-summary-out <file>, --manifest-checksum-out <file>, --artifact-checksum-out <file>, --normalized-intermediate-summary-out <file>, or --check"
            .to_string()
    })?;

    Ok(PackagedArtifactCommand::Write {
        output_path,
        manifest_path,
        manifest_summary_path,
        manifest_checksum_path,
        artifact_checksum_path,
        normalized_intermediate_path,
    })
}

fn write_text_file(path: &str, contents: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }
    fs::write(path, contents).map_err(|error| format!("failed to write {}: {error}", path))
}

fn render_packaged_artifact_regeneration(
    output_path: String,
    manifest_path: Option<String>,
    manifest_summary_path: Option<String>,
    manifest_checksum_path: Option<String>,
    artifact_checksum_path: Option<String>,
    normalized_intermediate_path: Option<String>,
) -> Result<String, String> {
    let artifact = packaged_artifact();
    let encoded = packaged_artifact_bytes();
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }
    fs::write(&output_path, encoded)
        .map_err(|error| format!("failed to write {}: {error}", output_path))?;

    let manifest = if manifest_path.is_some()
        || manifest_summary_path.is_some()
        || manifest_checksum_path.is_some()
    {
        Some(packaged_artifact_generation_manifest_for_report())
    } else {
        None
    };
    let normalized_intermediate = if normalized_intermediate_path.is_some() {
        Some(validated_packaged_artifact_normalized_intermediate_summary_for_report())
    } else {
        None
    };

    let manifest_line = if let Some(manifest_path) = manifest_path {
        let manifest_text = manifest
            .as_deref()
            .expect("manifest text should be available when a manifest path is requested");
        write_text_file(&manifest_path, manifest_text)?;
        format!(
            "
  manifest: {}",
            manifest_path
        )
    } else {
        String::new()
    };

    let manifest_summary_line = if let Some(manifest_summary_path) = manifest_summary_path {
        let manifest_text = manifest
            .as_deref()
            .expect("manifest text should be available when a manifest summary path is requested");
        write_text_file(&manifest_summary_path, manifest_text)?;
        format!(
            "
  manifest summary sidecar: {}",
            manifest_summary_path
        )
    } else {
        String::new()
    };

    let manifest_checksum_line = if let Some(manifest_checksum_path) = manifest_checksum_path {
        let manifest_text = manifest
            .as_deref()
            .expect("manifest text should be available when a manifest checksum path is requested");
        let checksum_text = format!(
            "0x{:016x}
",
            checksum64(manifest_text)
        );
        write_text_file(&manifest_checksum_path, &checksum_text)?;
        format!(
            "
  manifest checksum sidecar: {}",
            manifest_checksum_path
        )
    } else {
        String::new()
    };

    let artifact_checksum_line = if let Some(artifact_checksum_path) = artifact_checksum_path {
        let checksum_text = format!("0x{:016x}\n", artifact.checksum);
        write_text_file(&artifact_checksum_path, &checksum_text)?;
        format!("\n  artifact checksum sidecar: {}", artifact_checksum_path)
    } else {
        String::new()
    };

    let normalized_intermediate_line = if let Some(normalized_intermediate_path) =
        normalized_intermediate_path
    {
        let normalized_intermediate_text = normalized_intermediate
            .as_deref()
            .expect("normalized intermediate text should be available when a normalized intermediate path is requested");
        write_text_file(&normalized_intermediate_path, normalized_intermediate_text)?;
        format!(
            "
  normalized intermediate sidecar: {}",
            normalized_intermediate_path
        )
    } else {
        String::new()
    };

    Ok(format!(
        "Packaged artifact regenerated
  path: {}
  label: {}
  source: {}
  checksum: 0x{:016x}
  bytes: {}
  {}{}{}{}{}{}",
        output_path,
        artifact.header.generation_label,
        artifact.header.source,
        artifact.checksum,
        encoded.len(),
        packaged_artifact_regeneration_summary_for_report(),
        manifest_line,
        manifest_summary_line,
        manifest_checksum_line,
        artifact_checksum_line,
        normalized_intermediate_line,
    ))
}

fn render_packaged_artifact_regeneration_check() -> Result<String, String> {
    static CACHE: OnceLock<String> = OnceLock::new();

    Ok(CACHE
        .get_or_init(|| {
            let artifact = packaged_artifact();
            let committed = packaged_artifact_bytes();

            format!(
                "Packaged artifact regeneration check passed\n  label: {}\n  source: {}\n  checksum: 0x{:016x}\n  bytes: {}\n  {}",
                artifact.header.generation_label,
                artifact.header.source,
                artifact.checksum,
                committed.len(),
                packaged_artifact_regeneration_summary_for_report(),
            )
        })
        .clone())
}

fn render_error(error: EphemerisError) -> String {
    error.summary_line()
}

fn render_artifact_error(error: crate::artifact::ArtifactInspectionError) -> String {
    error.to_string()
}

fn render_release_bundle_error(error: ReleaseBundleError) -> String {
    error.to_string()
}
