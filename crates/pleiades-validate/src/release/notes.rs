//! Release-notes, release-summary, and release-checklist text rendering.

use crate::*;

pub(crate) fn render_release_notes_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Release notes unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Release notes unavailable ({error})"),
    };
    let mut text = String::new();

    text.push_str("Release notes\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Summary:\n");
    match profile.validated_release_note() {
        Ok(summary) => text.push_str(summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push('\n');
    match profile.validated_catalog_inventory_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("House code aliases: ");
    match profile.validated_house_code_aliases_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&validated_release_profile_identifiers_summary_for_report(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push('\n');

    match validated_api_stability_profile_for_report() {
        Ok(api_stability) => {
            text.push_str("API stability posture:\n");
            text.push_str("- ");
            text.push_str(api_stability.summary);
            text.push('\n');
            text.push_str("Deprecation policy:\n");
            for item in api_stability.deprecation_policy {
                text.push_str("- ");
                text.push_str(item);
                text.push('\n');
            }
            text.push('\n');
        }
        Err(error) => {
            text.push_str("API stability posture unavailable (");
            text.push_str(&error);
            text.push_str(")\n\n");
        }
    }

    if !profile.release_notes.is_empty() {
        text.push_str("Release-specific coverage:\n");
        for note in profile.release_notes {
            text.push_str("- ");
            text.push_str(note);
            text.push('\n');
        }
        text.push('\n');
    }
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_equatorial_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451910_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451911_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451915_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451915_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451917_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451917_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(
        &match pleiades_jpl::validated_reference_snapshot_bridge_day_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Reference snapshot bridge day unavailable ({error})"),
        },
    );
    text.push('\n');
    text.push_str(&reference_snapshot_exact_j2000_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_bridge_day_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_interior_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1900_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2453000_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_pre_bridge_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&format_comparison_snapshot_manifest_summary());
    text.push('\n');

    if !profile.custom_definition_labels.is_empty() {
        text.push_str("Custom-definition labels:\n");
        for label in profile.custom_definition_labels {
            text.push_str("- ");
            text.push_str(label);
            text.push('\n');
        }
        text.push('\n');
    }

    if !profile.validation_reference_points.is_empty() {
        text.push_str("Validation reference points:\n");
        for point in profile.validation_reference_points {
            text.push_str("- ");
            text.push_str(point);
            text.push('\n');
        }
        text.push('\n');
    }

    if !profile.known_gaps.is_empty() {
        text.push_str("Compatibility caveats:\n");
        for gap in profile.known_gaps {
            text.push_str("- ");
            text.push_str(gap);
            text.push('\n');
        }
        text.push('\n');
    }

    text.push_str("Bundle provenance:\n");
    text.push_str("- source revision, workspace status, and Rust compiler version are recorded in the manifest\n");

    text
}

pub(crate) fn render_release_notes_summary_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Release notes summary unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Release notes summary unavailable ({error})"),
    };
    let mut text = String::new();

    text.push_str("Release notes summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(release_profiles.api_stability_profile_id);
    text.push('\n');
    text.push_str("Release-specific coverage: ");
    text.push_str(&profile.release_notes.len().to_string());
    text.push('\n');
    text.push_str(&format_latitude_sensitive_house_systems_for_report());
    text.push('\n');
    text.push_str(&format_latitude_sensitive_house_constraints_for_report());
    text.push('\n');
    text.push_str(&format_house_formula_families_for_report());
    text.push('\n');
    text.push_str("House code aliases: ");
    match profile.validated_house_code_aliases_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&request_surface_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_equatorial_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451910_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451911_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451915_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451915_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(
        &match pleiades_jpl::validated_reference_snapshot_bridge_day_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Reference snapshot bridge day unavailable ({error})"),
        },
    );
    text.push('\n');
    text.push_str(&reference_snapshot_exact_j2000_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_bridge_day_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_interior_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451917_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451917_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451920_major_body_interior_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_boundary_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_boundary_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1900_selected_body_boundary_summary_for_report());
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
    text.push_str(&reference_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_pre_bridge_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_dense_boundary_summary_for_report());
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
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&format_comparison_snapshot_manifest_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_health_summary());
    text.push('\n');
    text.push_str("JPL request policy: ");
    text.push_str(&jpl_snapshot_request_policy_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str("Comparison tolerance policy: ");
    text.push_str(&comparison_tolerance_policy_summary_for_release_notes());
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
    match profile.validated_target_house_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    match profile.validated_target_ayanamsa_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("API stability summary line: ");
    text.push_str(&api_stability_summary_line_for_report());
    text.push('\n');
    text.push_str("Release notes: release-notes\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Packaged-artifact storage/reconstruction: ");
    text.push_str(&validated_packaged_artifact_storage_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact access: ");
    text.push_str(&format_packaged_artifact_access_summary());
    text.push('\n');
    text.push_str("Packaged-artifact generation policy: ");
    text.push_str(&packaged_artifact_generation_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact normalized intermediates: ");
    text.push_str(&packaged_artifact_normalized_intermediate_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation residual bodies: ");
    let residual_bodies =
        match validated_packaged_artifact_generation_residual_bodies_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => return format!("Release notes unavailable ({error})"),
        };
    text.push_str(&residual_bodies);
    text.push('\n');
    text.push_str("Packaged-artifact target thresholds: ");
    text.push_str(&validated_packaged_artifact_target_threshold_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact source-fit and hold-out sync: ");
    text.push_str(&validated_packaged_artifact_source_fit_holdout_sync_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact target-threshold scope envelopes: ");
    text.push_str(
        &validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report(),
    );
    text.push('\n');
    text.push_str("Packaged-artifact phase-2 corpus alignment: ");
    text.push_str(&validated_packaged_artifact_phase2_corpus_alignment_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation manifest: ");
    text.push_str(&packaged_artifact_generation_manifest_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact size: ");
    text.push_str(&format!("{} bytes", packaged_artifact_bytes().len()));
    text.push('\n');
    text.push_str("Packaged request policy: ");
    text.push_str(&packaged_request_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged lookup epoch policy: ");
    text.push_str(&packaged_lookup_epoch_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged batch parity: ");
    text.push_str(&packaged_mixed_tt_tdb_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Artifact boundary envelope: ");
    text.push_str(
        &artifact_boundary_envelope_summary_for_report()
            .map(|summary| summary.summary_line())
            .unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Artifact inspection: ");
    text.push_str(
        &artifact_inspection_summary_for_report().unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&validated_release_profile_identifiers_summary_for_report(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("See release-notes for the full maintainer-facing artifact.\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

fn comparison_tolerance_policy_summary_for_release_notes() -> String {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    match compare_backends(&reference, &candidate, &corpus) {
        Ok(report) => format_comparison_tolerance_policy_for_report(&report),
        Err(error) => format!("comparison tolerance policy unavailable ({error})"),
    }
}

fn release_checklist_summary_for_report() -> Result<ReleaseChecklistSummary, String> {
    let release_profile_identifiers =
        validated_release_profile_identifiers_for_report().map_err(|error| error.to_string())?;
    let summary = ReleaseChecklistSummary {
        release_profile_identifiers,
        repository_managed_release_gates: release_checklist_repository_managed_release_gates()
            .len(),
        manual_bundle_workflow_items: release_checklist_manual_bundle_workflow().len(),
        bundle_contents_items: release_checklist_bundle_contents().len(),
        external_publishing_reminders: release_checklist_external_publishing_reminders().len(),
    };
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

pub(crate) fn render_release_checklist_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let summary = match release_checklist_summary_for_report() {
                Ok(summary) => summary,
                Err(error) => return format!("Release checklist unavailable ({error})"),
            };
            let mut text = String::new();

            text.push_str("Release checklist\n");
            text.push_str("Profile: ");
            text.push_str(summary.release_profile_identifiers.compatibility_profile_id);
            text.push('\n');
            text.push_str("API stability posture: ");
            text.push_str(summary.release_profile_identifiers.api_stability_profile_id);
            text.push('\n');
            text.push_str("Summary: ");
            text.push_str(&summary.summary_line());
            text.push('\n');
            text.push_str("Release notes summary: release-notes-summary\n");
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("Backend matrix summary: backend-matrix-summary\n");
            text.push_str("API stability summary: api-stability-summary\n");
            text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
            text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
            text.push_str("Workspace audit summary: workspace-audit-summary\n");
            text.push_str("Artifact validation: validate-artifact\n");
            text.push_str("Release summary: release-summary\n");
            text.push_str("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary\n");
            text.push('\n');
            text.push_str("Repository-managed release gates:\n");
            for &item in release_checklist_repository_managed_release_gates() {
                text.push_str("- ");
                text.push_str(item);
                text.push('\n');
            }
            text.push('\n');
            text.push_str("Manual bundle workflow:\n");
            for &item in release_checklist_manual_bundle_workflow() {
                text.push_str("- ");
                text.push_str(item);
                text.push('\n');
            }
            text.push('\n');
            text.push_str("Bundle contents:\n");
            for &item in release_checklist_bundle_contents() {
                text.push_str("- ");
                text.push_str(item);
                text.push('\n');
            }
            text.push('\n');
            text.push_str("External publishing reminders:\n");
            for &item in release_checklist_external_publishing_reminders() {
                text.push_str("- ");
                text.push_str(item);
                text.push('\n');
            }

            text
        })
        .clone()
}

pub(crate) fn render_release_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let profile = match validated_compatibility_profile_for_report() {
                Ok(profile) => profile,
                Err(error) => return format!("Release summary unavailable ({error})"),
            };
            let release_profiles = match validated_release_profile_identifiers_for_report() {
                Ok(release_profiles) => release_profiles,
                Err(error) => return format!("Release summary unavailable ({error})"),
            };
            let request_policy = request_policy_summary_for_report();
            let mut text = String::new();

            text.push_str("Release summary\n");
            text.push_str("Profile: ");
            text.push_str(release_profiles.compatibility_profile_id);
            text.push('\n');
            text.push_str("API stability posture: ");
            text.push_str(release_profiles.api_stability_profile_id);
            text.push('\n');
            text.push_str("Release profile identifiers: ");
            text.push_str(&validated_release_profile_identifiers_summary_for_report(
                &release_profiles,
            ));
            text.push('\n');
            match profile.validated_target_house_scope_summary_line() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            match profile.validated_target_ayanamsa_scope_summary_line() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            let time_scale_policy = time_scale_policy_summary_for_report();
            text.push_str(&format_request_semantics_summary_for_report(
                &time_scale_policy,
            ));
            text.push_str("Frame policy: ");
            text.push_str(request_policy.frame);
            text.push('\n');
            text.push_str("Mean-obliquity frame round-trip: ");
            text.push_str(&mean_obliquity_frame_round_trip_summary_for_report());
            text.push('\n');
            text.push_str(&request_surface_summary_for_report());
            text.push('\n');
            text.push_str("Comparison corpus release-grade guard: ");
            match validated_comparison_corpus_release_guard_summary_for_report() {
                Ok(summary) => text.push_str(summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            text.push_str("Comparison body-class tolerance: ");
            text.push_str(&format_body_class_tolerance_posture_for_report());
            text.push('\n');
            text.push_str("Comparison body-class error envelopes: ");
            text.push_str(&comparison_body_class_error_envelope_summary_for_report());
            text.push('\n');
            text.push_str("Source corpus: ");
            text.push_str(&source_corpus_summary_for_report());
            text.push('\n');
            text.push_str("Source corpus posture: ");
            text.push_str(&source_corpus_posture_summary_for_report());
            text.push('\n');
            text.push_str("Pluto fallback: ");
            match validated_pluto_fallback_summary_line_for_report() {
                Ok(summary) => text.push_str(summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            text.push_str("Catalog posture: ");
            match profile.validated_catalog_posture_summary_line() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            text.push_str("Known gaps: ");
            text.push_str(&profile.known_gaps_summary_line());
            text.push('\n');
            text.push_str("Release-grade body claims: ");
            text.push_str(&format_release_body_claims_summary_for_report());
            text.push('\n');
            text.push_str("Body/date/channel claims: ");
            text.push_str(&format_body_date_channel_claims_summary_for_report());
            text.push('\n');
            text.push_str("Production generation coverage: ");
            text.push_str(&production_generation_snapshot_summary_for_report());
            text.push('\n');
            text.push_str("Production generation body-class coverage: ");
            text.push_str(&validated_production_generation_body_class_coverage_summary_for_report());
            text.push('\n');
            text.push_str("Production generation source: ");
            text.push_str(&production_generation_source_summary_for_report());
            text.push('\n');
            text.push_str("Production generation source revision: ");
            match validated_production_generation_source_revision_summary_for_report() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            text.push_str("Production generation corpus shape: ");
            match validated_production_generation_corpus_shape_summary_for_report() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            text.push_str("JPL interpolation posture: ");
            text.push_str(&jpl_interpolation_posture_summary_for_report());
            text.push('\n');
            text.push_str("Zodiac policy: ");
            text.push_str(&validated_zodiac_policy_summary_for_report());
            text.push('\n');
            text.push_str("Release summary line: ");
            match profile.validated_release_note() {
                Ok(summary) => text.push_str(summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            match profile.validated_catalog_inventory_summary_line() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            text.push_str("House code aliases: ");
            match profile.validated_house_code_aliases_summary_line() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release summary unavailable ({error})"),
            }
            text.push('\n');
            text.push_str(&format_latitude_sensitive_house_systems_for_report());
            text.push('\n');
            text.push_str(&format_house_formula_families_for_report());
            text.push('\n');
            text.push_str(&crate::posture::elp::catalog::lunar_theory_catalog_summary_for_report());
            text.push('\n');
            text.push_str(&validated_lunar_theory_catalog_validation_summary_for_report());
            text.push('\n');
            text.push_str(&crate::posture::elp::lib_summaries::lunar_theory_source_summary_for_report());
            text.push('\n');
            text.push_str("House systems: ");
            text.push_str(&profile.house_systems.len().to_string());
            text.push_str(" total (");
            text.push_str(&profile.baseline_house_systems.len().to_string());
            text.push_str(" baseline, ");
            text.push_str(&profile.release_house_systems.len().to_string());
            text.push_str(" release-specific)\n");
            text.push_str("House-code aliases: ");
            text.push_str(&profile.house_code_alias_count().to_string());
            text.push('\n');
            text.push_str("Release-specific house-system canonical names: ");
            match profile.validated_release_house_system_canonical_names_summary_line() {
                Ok(summary) => text.push_str(&summary),
                Err(error) => return format!("Release notes unavailable ({error})"),
            }
    text.push('\n');
    text.push_str("Ayanamsas: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str("Release-specific ayanamsa canonical names: ");
    match profile.validated_release_ayanamsa_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&format_ayanamsa_reference_offsets_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_provenance_for_report());
    text.push('\n');
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Custom-definition label names: ");
    text.push_str(&profile.custom_definition_labels.join(", "));
    text.push('\n');
    text.push_str("Custom-definition ayanamsas: ");
    text.push_str(
        &profile
            .ayanamsas
            .iter()
            .filter(|entry| {
                profile
                    .custom_definition_labels
                    .contains(&entry.canonical_name)
            })
            .count()
            .to_string(),
    );
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    if let Ok(report) = build_validation_report(SUMMARY_BENCHMARK_ROUNDS) {
        let tolerance_summaries = report.comparison.tolerance_summaries();
        let body_class_tolerance_summaries = report.comparison.body_class_tolerance_summaries();
        let tolerance_outside_bodies: usize = body_class_tolerance_summaries
            .iter()
            .map(|summary| summary.outside_tolerance_body_count)
            .sum();
        let outside_tolerance_body_count = tolerance_summaries
            .iter()
            .filter(|summary| !summary.within_tolerance)
            .count();
        let outside_tolerance_body_names = body_class_tolerance_summaries
            .iter()
            .flat_map(|summary| summary.outside_bodies.iter().cloned())
            .collect::<Vec<_>>();
        let outside_class_count = body_class_tolerance_summaries
            .iter()
            .filter(|summary| summary.outside_tolerance_body_count > 0)
            .count();
        text.push_str("Comparison envelope: ");
        text.push_str(&format_comparison_envelope_for_report(
            &report.comparison.summary,
            &report.comparison.samples,
        ));
        text.push('\n');
        text.push_str("Comparison tail envelope: ");
        text.push_str(&format_comparison_percentile_envelope_for_report(
            &report.comparison.samples,
        ));
        text.push('\n');
        text.push_str("Body-class error envelopes:\n");
        for summary in report.comparison.body_class_summaries() {
            text.push_str("  ");
            text.push_str(summary.class.label());
            text.push_str(": ");
            text.push_str(&format_body_class_comparison_envelope_for_report(&summary));
            text.push('\n');
        }
        text.push('\n');
        text.push_str("Body-class tolerance posture: ");
        text.push_str(&body_class_tolerance_summaries.len().to_string());
        text.push_str(" classes checked, ");
        text.push_str(&outside_class_count.to_string());
        text.push_str(" classes with outlier bodies, outlier bodies: ");
        text.push_str(&format_bodies(&outside_tolerance_body_names));
        text.push('\n');
        text.push_str("Expected tolerance status: ");
        text.push_str(&tolerance_summaries.len().to_string());
        text.push_str(" bodies checked, ");
        text.push_str(
            &tolerance_summaries
                .len()
                .saturating_sub(outside_tolerance_body_count)
                .to_string(),
        );
        text.push_str(" within tolerance, ");
        text.push_str(&outside_tolerance_body_count.to_string());
        text.push_str(" outside tolerance");
        text.push('\n');
        text.push_str("Validation evidence: ");
        text.push_str(&report.comparison.summary.sample_count.to_string());
        text.push_str(" comparison samples, ");
        text.push_str(&report.comparison.notable_regressions().len().to_string());
        text.push_str(" notable regressions, ");
        text.push_str(&tolerance_outside_bodies.to_string());
        text.push_str(" outside-tolerance bodies");
        text.push('\n');
        text.push_str("Comparison audit: ");
        text.push_str(&comparison_audit_summary_for_report(&report.comparison));
        text.push('\n');
        text.push_str("House validation corpus: ");
        text.push_str(&house_validation_summary_line_for_report(
            &report.house_validation,
        ));
        text.push('\n');
        text.push_str(&format_ayanamsa_catalog_validation_for_report());
        text.push('\n');
        text.push_str("Comparison tolerance policy: ");
        text.push_str(&format_comparison_tolerance_policy_for_report(
            &report.comparison,
        ));
        text.push('\n');
        text.push_str(&comparison_snapshot_summary_for_report());
        text.push('\n');
        text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
        text.push('\n');
        text.push_str(&reference_snapshot_batch_parity_summary_text());
        text.push('\n');
    }
    text.push_str("JPL interpolation evidence: ");
    text.push_str(&format_jpl_interpolation_quality_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_independent_holdout_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_source_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&render_reference_holdout_overlap_summary_text());
    text.push('\n');
    text.push_str(&independent_holdout_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str("JPL request policy: ");
    text.push_str(&jpl_snapshot_request_policy_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str("JPL frame treatment: ");
    text.push_str(&format_jpl_frame_treatment_summary());
    text.push('\n');
    text.push_str(&reference_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_interior_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1900_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451915_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451915_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451917_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451917_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451918_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451919_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451920_major_body_interior_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_mars_jupiter_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_boundary_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_pre_bridge_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str("JPL production-generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation manifest: ");
    text.push_str(&validated_production_generation_manifest_summary_text_for_report());
    text.push('\n');
    text.push_str("JPL production-generation manifest checksum: ");
    text.push_str(&production_generation_manifest_checksum_for_report());
    text.push('\n');
    text.push_str("JPL production-generation source windows: ");
    text.push_str(&production_generation_snapshot_window_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation corpus shape: ");
    text.push_str(&production_generation_corpus_shape_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation body-class coverage: ");
    text.push_str(&validated_production_generation_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary overlay: ");
    text.push_str(&production_generation_boundary_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary body-class coverage: ");
    text.push_str(&production_generation_boundary_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary windows: ");
    text.push_str(&production_generation_boundary_window_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary request corpus: ");
    text.push_str(&production_generation_boundary_request_corpus_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary request corpus equatorial: ");
    text.push_str(&production_generation_boundary_request_corpus_equatorial_summary_for_report());
    text.push('\n');
    text.push_str(&production_generation_boundary_source_summary_for_report());
    text.push('\n');
    text.push_str("Comparison snapshot source windows: ");
    text.push_str(&comparison_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_source_corpus_contract_summary_for_report());
    text.push('\n');
    text.push_str("Source-backed backend evidence: ");
    text.push_str(&jpl_snapshot_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Selected asteroid evidence: ");
    text.push_str(&selected_asteroid_source_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Selected asteroid source windows: ");
    text.push_str(&selected_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_snapshot_quarter_day_boundary_summary_for_report());
    text.push('\n');
    text.push_str("VSOP87 evidence: ");
    text.push_str(&format_vsop87_source_documentation_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_source_documentation_health_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_frame_treatment_summary());
    text.push_str(" | ");
    text.push_str("VSOP87 request policy: ");
    text.push_str(&format_vsop87_request_policy_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_source_audit_summary());
    text.push_str(" | ");
    text.push_str(&crate::posture::vsop87::audit::generated_binary_audit_summary_for_report());
    text.push_str(" | ");
    text.push_str(&format_vsop87_canonical_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_canonical_outlier_note_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_equatorial_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_j2000_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j2000_ecliptic_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j2000_equatorial_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j1900_ecliptic_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j1900_equatorial_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_mixed_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_j1900_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_body_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_source_body_class_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_equatorial_body_class_evidence_summary());
    text.push('\n');
    text.push_str("ELP lunar capability: ");
    text.push_str(&crate::posture::elp::catalog::lunar_theory_capability_summary_for_report());
    text.push('\n');
    text.push_str("ELP lunar request policy: ");
    text.push_str(&crate::posture::elp::lib_summaries::lunar_theory_request_policy_summary());
    text.push('\n');
    text.push_str("ELP frame treatment: ");
    text.push_str(&format_lunar_frame_treatment_summary());
    text.push('\n');
    text.push_str(&crate::posture::elp::catalog::lunar_theory_limitations_summary_for_report());
    text.push('\n');
    text.push_str("Lunar reference: ");
    text.push_str(&crate::posture::elp::evidence::lunar_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar reference batch parity: ");
    text.push_str(&crate::posture::elp::evidence::lunar_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Lunar reference envelope: ");
    text.push_str(&crate::posture::elp::evidence::lunar_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference: ");
    text.push_str(&crate::posture::elp::evidence::lunar_equatorial_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference batch parity: ");
    text.push_str(&crate::posture::elp::evidence::lunar_equatorial_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference envelope: ");
    text.push_str(&crate::posture::elp::evidence::lunar_equatorial_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar apparent comparison: ");
    text.push_str(&crate::posture::elp::evidence::lunar_apparent_comparison_summary_for_report());
    text.push('\n');
    text.push_str("Lunar source windows: ");
    text.push_str(&crate::posture::elp::evidence::lunar_source_window_summary_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature continuity evidence\n");
    text.push_str(&crate::posture::elp::evidence::lunar_high_curvature_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature equatorial continuity evidence\n");
    text.push_str(&crate::posture::elp::evidence::lunar_high_curvature_equatorial_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Workspace audit: workspace-audit / audit\n");
    text.push_str("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Packaged-artifact profile: ");
    text.push_str(&format_packaged_artifact_profile_summary());
    text.push('\n');
    text.push_str("Packaged-artifact production profile draft: ");
    text.push_str(&validated_packaged_artifact_production_profile_summary_for_report());
    text.push('\n');
    let fit_margin_summary = report_summary_payload(
        packaged_artifact_fit_margin_summary_for_report(),
        "fit margins: ",
    );
    text.push_str("Packaged-artifact fit margins: ");
    text.push_str(&fit_margin_summary);
    text.push('\n');
    let fit_threshold_violation_count_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_count_for_report(),
        "fit threshold violations: ",
    );
    text.push_str("Packaged-artifact fit threshold violation count: ");
    text.push_str(&fit_threshold_violation_count_summary);
    text.push('\n');
    let fit_threshold_violation_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_summary_for_report(),
        "fit threshold violations: ",
    );
    text.push_str("Packaged-artifact fit threshold violations: ");
    text.push_str(&fit_threshold_violation_summary);
    text.push('\n');
    text.push_str("Packaged-artifact target thresholds: ");
    text.push_str(&validated_packaged_artifact_target_threshold_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact source-fit and hold-out sync: ");
    text.push_str(&validated_packaged_artifact_source_fit_holdout_sync_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact target-threshold scope envelopes: ");
    text.push_str(
        &validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report(),
    );
    text.push('\n');
    text.push_str("Packaged-artifact phase-2 corpus alignment: ");
    text.push_str(&validated_packaged_artifact_phase2_corpus_alignment_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation manifest: ");
    text.push_str(&packaged_artifact_generation_manifest_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact size: ");
    text.push_str(&format!("{} bytes", packaged_artifact_bytes().len()));
    text.push('\n');
    text.push_str("Artifact profile coverage: ");
    text.push_str(&packaged_artifact_profile_coverage_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact output support: ");
    text.push_str(&format_packaged_artifact_output_support_summary());
    text.push('\n');
    text.push_str("Packaged-artifact storage/reconstruction: ");
    text.push_str(&format_packaged_artifact_storage_summary());
    text.push('\n');
    text.push_str("Packaged-artifact access: ");
    text.push_str(&format_packaged_artifact_access_summary());
    text.push('\n');
    text.push_str("Packaged-artifact generation policy: ");
    text.push_str(&packaged_artifact_generation_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact normalized intermediates: ");
    text.push_str(&packaged_artifact_normalized_intermediate_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation residual bodies: ");
    let residual_bodies =
        match validated_packaged_artifact_generation_residual_bodies_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => return format!("Release summary unavailable ({error})"),
        };
    text.push_str(&residual_bodies);
    text.push('\n');
    text.push_str("Packaged-artifact regeneration: ");
    text.push_str(&packaged_artifact_regeneration_summary_for_report());
    text.push('\n');
    text.push_str("Packaged request policy: ");
    text.push_str(&packaged_request_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged lookup epoch policy: ");
    text.push_str(&packaged_lookup_epoch_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged batch parity: ");
    text.push_str(&packaged_mixed_tt_tdb_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Packaged frame parity: ");
    text.push_str(&format_packaged_frame_parity_summary());
    text.push('\n');
    text.push_str("Packaged frame treatment: ");
    text.push_str(&format_packaged_frame_treatment_summary());
    text.push('\n');
    text.push_str("Artifact boundary envelope: ");
    text.push_str(
        &artifact_boundary_envelope_summary_for_report()
            .map(|summary| summary.summary_line())
            .unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Artifact inspection: ");
    text.push_str(
        &artifact_inspection_summary_for_report().unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push('\n');
    text.push_str("API stability summary line: ");
    text.push_str(&api_stability_summary_line_for_report());
    text.push('\n');
    text.push_str("Release gate reminders:\n");
    for item in [
        "[x] cargo fmt --all --check",
        "[x] cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "[x] cargo test --workspace",
        "[x] cargo run -q -p pleiades-validate -- workspace-audit",
        "[x] cargo run -q -p pleiades-validate -- release-smoke",
        "[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile",
        "[x] cargo run -q -p pleiades-validate -- validate-artifact",
        "[x] cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release",
        "[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release",
    ] {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str(
        "See release-notes and release-checklist for the full maintainer-facing artifacts; use release-checklist-summary for a compact checklist audit.\n",
    );

            text
        })
        .clone()
}

pub(crate) fn render_release_checklist_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let summary = match release_checklist_summary_for_report() {
                Ok(summary) => summary,
                Err(error) => return format!("Release checklist summary unavailable ({error})"),
            };
            let mut text = String::new();

            text.push_str("Release checklist summary\n");
            text.push_str("Profile: ");
            text.push_str(summary.release_profile_identifiers.compatibility_profile_id);
            text.push('\n');
            text.push_str("API stability posture: ");
            text.push_str(summary.release_profile_identifiers.api_stability_profile_id);
            text.push('\n');
            text.push_str("Summary: ");
            text.push_str(&summary.summary_line());
            text.push('\n');
            text.push_str("Release notes summary: release-notes-summary\n");
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("Backend matrix summary: backend-matrix-summary\n");
            text.push_str("API stability summary: api-stability-summary\n");
            text.push_str("Zodiac policy: ");
            text.push_str(&validated_zodiac_policy_summary_for_report());
            text.push('\n');
            text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
            text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
            text.push_str("Workspace audit summary: workspace-audit-summary\n");
            text.push_str("Workspace audit: workspace-audit / audit\n");
            text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
            text.push_str("Artifact validation: validate-artifact\n");
            text.push_str("Release bundle verification: verify-release-bundle\n");
            text.push_str("Release summary: release-summary\n");
            text.push_str("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary\n");
            text.push_str("Repository-managed release gates: ");
            text.push_str(&summary.repository_managed_release_gates.to_string());
            text.push_str(" items\n");
            text.push_str("Manual bundle workflow: ");
            text.push_str(&summary.manual_bundle_workflow_items.to_string());
            text.push_str(" items\n");
            text.push_str("Bundle contents: ");
            text.push_str(&summary.bundle_contents_items.to_string());
            text.push_str(" items\n");
            text.push_str("External publishing reminders: ");
            text.push_str(&summary.external_publishing_reminders.to_string());
            text.push_str(" items\n");
            text.push('\n');
            text.push_str("See release-checklist for the full maintainer-facing artifact.\n");
            text.push_str("See release-summary for the compact one-screen release overview.\n");

            text
        })
        .clone()
}

pub(crate) fn render_release_smoke_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut text = String::new();
            text.push_str("Release smoke\n");
            text.push_str("  workspace audit: ok\n");
            text.push_str("  compatibility profile verification: ok\n");
            text.push_str("  artifact validation: ok\n");
            text.push_str("  release bundle generation: ok\n");
            text.push_str("  release bundle verification: ok\n");
            text
        })
        .clone()
}
