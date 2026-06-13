//! packaged-artifact and production-generation rendering tests (white-box; moved verbatim from the former `tests.rs`).

use super::test_support::*;
use super::*;

#[test]
fn packaged_artifact_fit_posture_validation_is_enforced_before_validation_reports_are_built() {
    validate_packaged_artifact_fit_posture()
        .expect("packaged-artifact fit posture should validate before report assembly");
}

#[test]
fn packaged_artifact_fit_posture_validation_reports_threshold_context() {
    let fit_envelope = packaged_artifact_fit_envelope_summary_details();
    let mut thresholds = packaged_artifact_fit_threshold_summary_details();
    thresholds.max_mean_longitude_delta_degrees = fit_envelope.mean_longitude_delta_degrees - 1.0;
    let target_threshold = packaged_artifact_target_threshold_summary_details();

    let error =
        validate_packaged_artifact_fit_posture_with(&fit_envelope, &thresholds, &target_threshold)
            .expect_err("fit threshold drift should be rejected");

    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains(
        "validation report packaged-artifact fit envelope exceeds calibrated thresholds"
    ));
    assert!(error.message.contains("1 violation"));
    assert!(error.message.contains("mean_longitude_delta_degrees"));
    assert!(error.message.contains("measured="));
    assert!(error.message.contains("threshold="));
    assert!(error.message.contains("mean Δlon≤"));
}

#[test]
fn packaged_artifact_fit_threshold_violation_count_summary_reflects_the_current_posture() {
    assert_eq!(
        packaged_artifact_fit_threshold_violation_count_for_report(),
        "fit threshold violations: 0"
    );
    let rendered = render_cli(&["packaged-artifact-fit-threshold-violation-count-summary"])
        .expect("fit threshold violation count summary should render");
    assert!(rendered.contains("Packaged-artifact fit threshold violation count: 0"));
    let alias_rendered = render_cli(&["packaged-artifact-fit-threshold-violation-count"])
        .expect("fit threshold violation count alias should render");
    assert!(alias_rendered.contains("Packaged-artifact fit threshold violation count: 0"));
}

#[test]
fn packaged_artifact_fit_threshold_violations_summary_reflects_the_current_posture() {
    assert_eq!(
        packaged_artifact_fit_threshold_violation_summary_for_report(),
        "fit threshold violations: 0; details: none"
    );
    let rendered = render_cli(&["packaged-artifact-fit-threshold-violations-summary"])
        .expect("fit threshold violations summary should render");
    assert!(rendered.contains("Packaged-artifact fit threshold violations: 0; details: none"));
    let alias_rendered = render_cli(&["packaged-artifact-fit-threshold-violations"])
        .expect("fit threshold violations alias should render");
    assert!(alias_rendered.contains("Packaged-artifact fit threshold violations: 0; details: none"));
}

#[test]
fn packaged_artifact_summary_commands_reject_extra_arguments() {
    for (args, expected) in [
            (
                &["packaged-artifact-production-profile-summary", "extra"][..],
                "packaged-artifact-production-profile-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-target-threshold-summary", "extra"][..],
                "packaged-artifact-target-threshold-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-target-threshold", "extra"][..],
                "packaged-artifact-target-threshold-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-target-threshold-scope-envelopes-summary", "extra"][..],
                "packaged-artifact-target-threshold-scope-envelopes-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-target-threshold-scope-envelopes", "extra"][..],
                "packaged-artifact-target-threshold-scope-envelopes-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-generation-manifest-summary", "extra"][..],
                "packaged-artifact-generation-manifest-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-generation-manifest", "extra"][..],
                "packaged-artifact-generation-manifest-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-generation-policy-summary", "extra"][..],
                "packaged-artifact-generation-policy-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-generation-policy", "extra"][..],
                "packaged-artifact-generation-policy-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-normalized-intermediate-summary", "extra"][..],
                "packaged-artifact-normalized-intermediate-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-regeneration-summary", "extra"][..],
                "packaged-artifact-regeneration-summary does not accept extra arguments",
            ),
            (
                &["packaged-artifact-regeneration", "extra"][..],
                "packaged-artifact-regeneration-summary does not accept extra arguments",
            ),
        ] {
            assert_eq!(
                render_cli(args).expect_err("packaged-artifact summary should reject extra arguments"),
                expected
            );
        }
}

#[test]
fn packaged_artifact_generation_policy_summary_and_alias_commands_render_the_policy() {
    let summary = render_cli(&["packaged-artifact-generation-policy-summary"])
        .expect("packaged artifact generation policy summary should render");
    assert_eq!(
        summary,
        packaged_artifact_generation_policy_summary_for_report()
    );
    assert_eq!(
        render_cli(&["packaged-artifact-generation-policy"])
            .expect("packaged artifact generation policy alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["packaged-artifact-generation-policy", "extra"])
            .expect_err("packaged artifact generation policy alias should reject extra arguments"),
        "packaged-artifact-generation-policy-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_generation_residual_bodies_validation_rejects_artifact_drift() {
    let artifact = packaged_artifact();
    let mut summary = pleiades_data::packaged_artifact_generation_residual_bodies_summary_details();
    *summary
        .bodies
        .first_mut()
        .expect("the residual body list should not be empty") = CelestialBody::Custom(
        pleiades_backend::CustomBodyId::new("test", "residual-drift"),
    );

    let error = validate_packaged_artifact_generation_residual_bodies_summary(&summary, artifact)
        .expect_err("residual body drift should fail validation");

    assert!(error.contains("does not match the current artifact"));
}

#[test]
fn packaged_artifact_normalized_intermediate_summary_and_alias_commands_render_the_summary() {
    let summary = render_cli(&["packaged-artifact-normalized-intermediate-summary"])
        .expect("packaged artifact normalized intermediate summary should render");
    assert_eq!(
        summary,
        format!(
            "Packaged-artifact normalized intermediates: {}",
            validated_packaged_artifact_normalized_intermediate_summary_for_report()
        )
    );
    assert!(summary
        .contains("Packaged artifact normalized intermediates: label=stage-5 packaged-data draft"));
    assert_eq!(
        render_cli(&["packaged-artifact-normalized-intermediate"])
            .expect("packaged artifact normalized intermediate alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["packaged-artifact-normalized-intermediate", "extra"]).expect_err(
            "packaged artifact normalized intermediate alias should reject extra arguments"
        ),
        "packaged-artifact-normalized-intermediate-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_body_cadence_summary_and_alias_commands_render_the_summary() {
    let summary = render_cli(&["packaged-artifact-body-cadence-summary"])
        .expect("packaged artifact body cadence summary should render");
    assert_eq!(
        summary,
        format!(
            "Packaged-artifact body cadence: {}",
            validated_packaged_artifact_body_cadence_summary_for_report()
        )
    );
    assert!(summary.contains("Packaged-artifact body cadence: body cadence:"));
    assert_eq!(
        render_cli(&["packaged-artifact-body-cadence"])
            .expect("packaged artifact body cadence alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["packaged-artifact-body-cadence", "extra"])
            .expect_err("packaged artifact body cadence alias should reject extra arguments"),
        "packaged-artifact-body-cadence-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_lookup_epoch_policy_summary_and_alias_commands_render_the_policy() {
    let summary = render_cli(&["packaged-artifact-lookup-epoch-policy-summary"])
        .expect("packaged artifact lookup epoch policy summary should render");
    assert_eq!(summary, packaged_lookup_epoch_policy_summary_for_report());
    assert_eq!(
        render_cli(&["packaged-artifact-lookup-epoch-policy"])
            .expect("packaged artifact lookup epoch policy alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["packaged-artifact-lookup-epoch-policy", "extra"]).expect_err(
            "packaged artifact lookup epoch policy alias should reject extra arguments"
        ),
        "packaged-lookup-epoch-policy-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_regeneration_summary_and_alias_commands_render_the_summary() {
    let summary = render_cli(&["packaged-artifact-regeneration-summary"])
        .expect("packaged artifact regeneration summary should render");
    assert!(summary.contains("Packaged-artifact regeneration: "));
    assert!(summary.contains("normalized intermediates: label=stage-5 packaged-data draft"));
    assert_eq!(
        render_cli(&["packaged-artifact-regeneration"])
            .expect("packaged artifact regeneration alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["packaged-artifact-regeneration", "extra"])
            .expect_err("packaged artifact regeneration alias should reject extra arguments"),
        "packaged-artifact-regeneration-summary does not accept extra arguments"
    );
}

#[test]
fn regenerate_packaged_artifact_check_command_reports_success() {
    let check = render_cli(&["regenerate-packaged-artifact", "--check"])
        .expect("packaged artifact regeneration check should render");
    assert!(check.contains("Packaged artifact regeneration check passed"));
    assert!(check.contains("checksum: 0x"));
    assert!(check.contains(&packaged_artifact_regeneration_summary_for_report()));

    let generated_check = render_cli(&["generate-packaged-artifact", "--check"])
        .expect("packaged artifact generation alias should render");
    assert_eq!(generated_check, check);
}

#[test]
fn regenerate_packaged_artifact_out_command_writes_bytes() {
    let output_dir = unique_temp_dir("pleiades-packaged-artifact-regeneration");
    let output_path = output_dir.join("packaged-artifact.bin");
    let output_path_string = output_path.to_string_lossy().to_string();
    let rendered = render_cli(&["regenerate-packaged-artifact", "--out", &output_path_string])
        .expect("packaged artifact regeneration should write bytes");
    assert!(rendered.contains("Packaged artifact regenerated"));
    assert!(rendered.contains("checksum: 0x"));
    let regenerated_bytes = std::fs::read(&output_path).expect("regenerated artifact should exist");
    assert!(!regenerated_bytes.is_empty());
}

#[test]
fn regenerate_packaged_artifact_output_alias_writes_bytes() {
    let output_alias_dir = unique_temp_dir("pleiades-packaged-artifact-regeneration-output");
    let output_alias_path = output_alias_dir.join("packaged-artifact.bin");
    let output_alias_path_string = output_alias_path.to_string_lossy().to_string();
    let rendered_alias = render_cli(&[
        "regenerate-packaged-artifact",
        "--output",
        &output_alias_path_string,
    ])
    .expect("packaged artifact regeneration should accept --output");
    assert!(rendered_alias.contains("Packaged artifact regenerated"));
    assert!(rendered_alias.contains("checksum: 0x"));
    let regenerated_alias_bytes =
        std::fs::read(&output_alias_path).expect("regenerated artifact alias should exist");
    assert!(!regenerated_alias_bytes.is_empty());
}

#[test]
fn regenerate_packaged_artifact_command_writes_all_sidecars() {
    let output_alias_dir = unique_temp_dir("pleiades-packaged-artifact-regeneration-sidecars");
    let output_alias_path = output_alias_dir.join("packaged-artifact.bin");
    let output_alias_path_string = output_alias_path.to_string_lossy().to_string();
    let manifest_path = output_alias_dir.join("packaged-artifact.manifest.txt");
    let manifest_path_string = manifest_path.to_string_lossy().to_string();
    let manifest_summary_path = output_alias_dir.join("packaged-artifact.manifest.summary.txt");
    let manifest_summary_path_string = manifest_summary_path.to_string_lossy().to_string();
    let manifest_checksum_path = output_alias_dir.join("packaged-artifact.manifest.checksum.txt");
    let manifest_checksum_path_string = manifest_checksum_path.to_string_lossy().to_string();
    let artifact_checksum_path = output_alias_dir.join("packaged-artifact.checksum.txt");
    let artifact_checksum_path_string = artifact_checksum_path.to_string_lossy().to_string();
    let normalized_intermediate_path =
        output_alias_dir.join("packaged-artifact.normalized-intermediate-summary.txt");
    let normalized_intermediate_path_string =
        normalized_intermediate_path.to_string_lossy().to_string();
    let rendered_with_sidecars = render_cli(&[
        "generate-packaged-artifact",
        "--out",
        &output_alias_path_string,
        "--manifest-out",
        &manifest_path_string,
        "--manifest-summary-out",
        &manifest_summary_path_string,
        "--manifest-checksum-out",
        &manifest_checksum_path_string,
        "--artifact-checksum-out",
        &artifact_checksum_path_string,
        "--normalized-intermediate-summary-out",
        &normalized_intermediate_path_string,
    ])
    .expect("packaged artifact regeneration should write all sidecars");
    assert!(rendered_with_sidecars.contains("manifest:"));
    assert!(rendered_with_sidecars.contains(&manifest_path_string));
    assert!(rendered_with_sidecars.contains("manifest summary sidecar:"));
    assert!(rendered_with_sidecars.contains(&manifest_summary_path_string));
    assert!(rendered_with_sidecars.contains("manifest checksum sidecar:"));
    assert!(rendered_with_sidecars.contains(&manifest_checksum_path_string));
    assert!(rendered_with_sidecars.contains("artifact checksum sidecar:"));
    assert!(rendered_with_sidecars.contains(&artifact_checksum_path_string));
    assert!(rendered_with_sidecars.contains("normalized intermediate sidecar:"));
    assert!(rendered_with_sidecars.contains(&normalized_intermediate_path_string));
    for path in [
        &manifest_path,
        &manifest_summary_path,
        &manifest_checksum_path,
        &artifact_checksum_path,
        &normalized_intermediate_path,
    ] {
        let metadata = std::fs::metadata(path).unwrap_or_else(|_| {
            panic!("packaged artifact sidecar should exist: {}", path.display())
        });
        assert!(
            metadata.len() > 0,
            "packaged artifact sidecar should not be empty: {}",
            path.display()
        );
    }
}

#[test]
fn packaged_artifact_phase2_alignment_matches_source_fit_holdout_sync_payload() {
    let sync_summary = validated_packaged_artifact_source_fit_holdout_sync_summary_for_report();
    let phase2_summary = validated_packaged_artifact_phase2_corpus_alignment_summary_for_report();

    ensure_packaged_artifact_phase2_alignment_matches_source_fit_holdout_sync(
        &format!(
            "Packaged-artifact source-fit and hold-out sync: {}",
            sync_summary
        ),
        &format!(
            "Packaged-artifact phase-2 corpus alignment: {}",
            phase2_summary
        ),
    )
    .expect("phase-2 alignment payload should match the source-fit sync payload");
}

#[test]
fn packaged_artifact_target_threshold_phase2_alignment_matches_source_fit_holdout_sync_payload() {
    let target_threshold_summary =
        validated_packaged_artifact_target_threshold_summary_for_report();
    let sync_summary = validated_packaged_artifact_source_fit_holdout_sync_summary_for_report();

    ensure_packaged_artifact_target_threshold_phase2_alignment_matches_source_fit_holdout_sync(
        &format!(
            "Packaged-artifact target thresholds: {}",
            target_threshold_summary
        ),
        &format!(
            "Packaged-artifact source-fit and hold-out sync: {}",
            sync_summary
        ),
    )
    .expect("target-threshold and source-fit summaries should share the phase-2 payload");
}

#[test]
fn packaged_artifact_target_threshold_phase2_alignment_rejects_payload_drift() {
    let target_threshold_summary =
        validated_packaged_artifact_target_threshold_summary_for_report();
    let sync_summary = validated_packaged_artifact_source_fit_holdout_sync_summary_for_report();
    let drifted_target_threshold_summary = target_threshold_summary.replace(
        "production generation source=Production generation source:",
        "production generation source=Production generation source: drifted",
    );

    let error =
        ensure_packaged_artifact_target_threshold_phase2_alignment_matches_source_fit_holdout_sync(
            &format!(
                "Packaged-artifact target thresholds: {}",
                drifted_target_threshold_summary
            ),
            &format!(
                "Packaged-artifact source-fit and hold-out sync: {}",
                sync_summary
            ),
        )
        .expect_err("drifted target-threshold payload should be rejected");
    assert!(error
        .to_string()
        .contains("target-threshold summary phase-2 corpus alignment payload does not match"));
}

#[test]
fn packaged_artifact_target_threshold_summary_matches_current_rendering() {
    let summary = validated_packaged_artifact_target_threshold_summary_for_report();

    ensure_packaged_artifact_target_threshold_summary_matches_current_rendering(&summary)
        .expect("packaged-artifact target-threshold summary should match the current rendering");
}

#[test]
fn packaged_artifact_normalized_intermediate_summary_matches_current_rendering() {
    let summary = validated_packaged_artifact_normalized_intermediate_summary_for_report();

    ensure_packaged_artifact_normalized_intermediate_summary_matches_current_rendering(&summary)
        .expect(
            "packaged-artifact normalized intermediate summary should match the current rendering",
        );
}

#[test]
fn packaged_artifact_normalized_intermediate_summary_validation_rejects_drift() {
    let summary = validated_packaged_artifact_normalized_intermediate_summary_for_report();
    let drifted_summary = summary.replace(
        "label=stage-5 packaged-data draft",
        "label=stage-5 drifted packaged-data draft",
    );

    let error = ensure_packaged_artifact_normalized_intermediate_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted normalized intermediate summary should be rejected");
    assert!(error
        .to_string()
        .contains("normalized intermediate summary no longer matches"));
}

#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_drift() {
    let summary = validated_packaged_artifact_target_threshold_summary_for_report();
    let drifted_summary = summary.replace(
        "production thresholds recorded",
        "production thresholds drifting",
    );

    let error = ensure_packaged_artifact_target_threshold_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted target-threshold summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current packaged-artifact target-threshold posture"));
}

#[test]
fn packaged_artifact_target_threshold_state_summary_validation_rejects_drift() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-packaged-artifact-target-threshold-state-semantic",
            "packaged-artifact-target-threshold-state-summary.txt",
            "packaged-artifact target-threshold state summary checksum (fnv1a-64):",
            "production thresholds recorded",
            "production thresholds drifting",
            "packaged-artifact target-threshold state summary no longer matches the current packaged-artifact target-threshold posture",
        );
}

#[test]
fn packaged_artifact_source_fit_holdout_sync_summary_validation_rejects_drift() {
    let summary = validated_packaged_artifact_source_fit_holdout_sync_summary_for_report();
    let drifted_summary = summary.replace("fit thresholds=", "fit thresholds=drifted-");

    let error = ensure_packaged_artifact_source_fit_holdout_sync_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted source-fit and hold-out sync summary should be rejected");
    assert!(error.to_string().contains(
        "no longer matches the current packaged-artifact source-fit and hold-out sync posture"
    ));
}

#[test]
fn production_generation_source_summary_embeds_the_source_window_payload() {
    let source_summary = production_generation_source_summary_for_report();
    let source_window_summary = production_generation_snapshot_window_summary_for_report();

    ensure_production_generation_source_summary_matches_source_windows(
        &source_summary,
        &source_window_summary,
    )
    .expect("production-generation source summary should match the source-window payload");
}

#[test]
fn production_generation_source_window_summary_matches_current_rendering() {
    let summary = production_generation_snapshot_window_summary_for_report();

    ensure_production_generation_source_window_summary_matches_current_rendering(&summary)
        .expect("production-generation source window summary should match the current rendering");
}

#[test]
fn production_generation_source_window_summary_validation_rejects_drift() {
    let summary = production_generation_snapshot_window_summary_for_report();
    let drifted_summary = summary.replace("357 source-backed samples", "358 source-backed samples");

    let error = ensure_production_generation_source_window_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted production-generation source window summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current production-generation source-window posture"));
}

#[test]
fn production_generation_source_summary_source_window_payload_validation_rejects_drift() {
    let source_summary = production_generation_source_summary_for_report();
    let drifted_source_windows = production_generation_snapshot_window_summary_for_report()
        .replace("357 source-backed samples", "358 source-backed samples");

    let error = ensure_production_generation_source_summary_matches_source_windows(
        &source_summary,
        &format!(
            "Production generation source windows: {}",
            drifted_source_windows
        ),
    )
    .expect_err("drifted source-window payload should be rejected");
    assert!(error
        .to_string()
        .contains("source windows payload does not match"));
}

#[test]
fn packaged_artifact_phase2_alignment_payload_validation_rejects_drift() {
    let sync_summary = validated_packaged_artifact_source_fit_holdout_sync_summary_for_report();
    let phase2_summary = "reference source=drifted; reference snapshot=drifted";

    let error = ensure_packaged_artifact_phase2_alignment_matches_source_fit_holdout_sync(
        &format!(
            "Packaged-artifact source-fit and hold-out sync: {}",
            sync_summary
        ),
        &format!(
            "Packaged-artifact phase-2 corpus alignment: {}",
            phase2_summary
        ),
    )
    .expect_err("drifted phase-2 alignment payload should be rejected");
    assert!(error
        .to_string()
        .contains("phase-2 corpus alignment payload does not match"));
}

#[test]
fn packaged_artifact_phase2_corpus_alignment_summary_matches_current_rendering() {
    let summary = validated_packaged_artifact_phase2_corpus_alignment_summary_for_report();

    ensure_packaged_artifact_phase2_corpus_alignment_summary_matches_current_rendering(&summary)
        .expect(
            "packaged-artifact phase-2 corpus alignment summary should match the current rendering",
        );
}

#[test]
fn packaged_artifact_phase2_corpus_alignment_summary_validation_rejects_drift() {
    let summary = validated_packaged_artifact_phase2_corpus_alignment_summary_for_report();
    let drifted_summary = summary.replace(
        "reference source=Reference snapshot source:",
        "reference source=Drifted snapshot source:",
    );

    let error = ensure_packaged_artifact_phase2_corpus_alignment_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted phase-2 corpus alignment summary should be rejected");
    assert!(error
        .to_string()
        .contains("phase-2 corpus alignment summary no longer matches"));
}

#[test]
fn production_generation_boundary_summary_command_renders_the_overlay_summary() {
    let rendered = render_cli(&["production-generation-boundary-summary"])
        .expect("production generation boundary summary should render");

    assert!(rendered.contains("Production generation boundary overlay:"));
    assert!(rendered.contains("boundary overlay"));
    assert_eq!(
        rendered,
        production_generation_boundary_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_alias_command_renders_the_overlay_summary() {
    let rendered = render_cli(&["production-generation-boundary"])
        .expect("production generation boundary alias should render");

    assert_eq!(
        rendered,
        production_generation_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary", "extra"]).unwrap_err(),
        "production-generation-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn production_generation_boundary_request_corpus_summary_command_renders_the_request_corpus_block()
{
    let rendered = render_cli(&["production-generation-boundary-request-corpus-summary"])
        .expect("production generation boundary request corpus summary should render");

    assert!(rendered.contains("Production generation boundary request corpus:"));
    assert!(rendered.contains("request corpus"));
    assert_eq!(
        rendered,
        production_generation_boundary_request_corpus_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_request_corpus_alias_command_renders_the_request_corpus_block() {
    let rendered = render_cli(&["production-generation-boundary-request-corpus"])
        .expect("production generation boundary request corpus alias should render");

    assert_eq!(
        rendered,
        production_generation_boundary_request_corpus_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_request_corpus_equatorial_command_renders_the_request_corpus_block(
) {
    let rendered =
        render_cli(&["production-generation-boundary-request-corpus-equatorial-summary"]).expect(
            "production generation boundary request corpus equatorial summary should render",
        );

    assert!(rendered.contains("Production generation boundary request corpus:"));
    assert!(rendered.contains("frame=Equatorial"));
    assert_eq!(
        rendered,
        production_generation_boundary_request_corpus_equatorial_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_request_corpus_equatorial_alias_command_renders_the_request_corpus_block(
) {
    let rendered = render_cli(&["production-generation-boundary-request-corpus-equatorial"])
        .expect("production generation boundary request corpus equatorial alias should render");

    assert_eq!(
        rendered,
        production_generation_boundary_request_corpus_equatorial_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_request_corpus_alias_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-boundary-request-corpus", "extra"]).expect_err(
        "production generation boundary request corpus alias should reject extra arguments",
    );

    assert!(error.contains(
        "production-generation-boundary-request-corpus-summary does not accept extra arguments"
    ));
}

#[test]
fn production_generation_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["production-generation-source-window-summary"])
        .expect("production generation source window summary should render");

    assert!(rendered.contains("Production generation source windows:"));
    assert!(rendered.contains("357 source-backed samples"));
    assert_eq!(
        rendered,
        production_generation_snapshot_window_summary_for_report()
    );
}

#[test]
fn production_generation_source_window_alias_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["production-generation-source-window"])
        .expect("production generation source window alias should render");

    assert_eq!(
        rendered,
        production_generation_snapshot_window_summary_for_report()
    );
}

#[test]
fn production_generation_source_window_alias_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-source-window", "extra"])
        .expect_err("production generation source window alias should reject extra arguments");

    assert!(error
        .contains("production-generation-source-window-summary does not accept extra arguments"));
}

#[test]
fn production_generation_summary_command_renders_the_overall_block() {
    let rendered = render_cli(&["production-generation-summary"])
        .expect("production generation summary should render");

    assert!(rendered.contains("Production generation coverage:"));
    assert!(rendered.contains("357 rows across 16 bodies and 31 epochs"));
    assert_eq!(
        rendered,
        production_generation_snapshot_summary_for_report()
    );
}

#[test]
fn production_generation_quarter_day_boundary_summary_command_renders_the_quarter_day_block() {
    let rendered = render_cli(&["production-generation-quarter-day-boundary-summary"])
        .expect("production generation quarter-day boundary summary should render");

    assert!(rendered.contains("Production generation quarter-day boundary samples:"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus"));
    assert_eq!(
        rendered,
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report()
    );
}

#[test]
fn production_generation_quarter_day_boundary_alias_command_renders_the_quarter_day_block() {
    let rendered = render_cli(&["production-generation-quarter-day-boundary"])
        .expect("production generation quarter-day boundary alias should render");

    assert_eq!(
        rendered,
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report()
    );
}

#[test]
fn production_generation_alias_command_renders_the_overall_block() {
    let rendered =
        render_cli(&["production-generation"]).expect("production generation alias should render");

    assert_eq!(
        rendered,
        production_generation_snapshot_summary_for_report()
    );
}

#[test]
fn production_generation_alias_rejects_extra_arguments() {
    let error = render_cli(&["production-generation", "extra"])
        .expect_err("production generation alias should reject extra arguments");

    assert!(error.contains("production-generation does not accept extra arguments"));
}

#[test]
fn production_generation_boundary_summary_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-boundary-summary", "extra"])
        .expect_err("production generation boundary summary should reject extra arguments");

    assert!(
        error.contains("production-generation-boundary-summary does not accept extra arguments")
    );
}

#[test]
fn production_generation_summary_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-summary", "extra"])
        .expect_err("production generation summary should reject extra arguments");

    assert!(error.contains("production-generation-summary does not accept extra arguments"));
}

#[test]
fn production_generation_corpus_shape_summary_command_renders_the_shape_block() {
    let rendered = render_cli(&["production-generation-corpus-shape-summary"])
        .expect("production generation corpus shape summary should render");

    assert!(rendered.contains("Production generation corpus shape:"));
    assert!(rendered.contains("boundary request corpora: ecliptic="));
    assert!(rendered.contains("equatorial="));
    assert!(rendered.contains(
        "validated fields=body order, epochs, frame, time scale, columns, apparentness, checksums"
    ));
    assert_eq!(
        rendered,
        production_generation_corpus_shape_summary_for_report()
    );
}

#[test]
fn production_generation_corpus_shape_alias_command_renders_the_shape_block() {
    let rendered = render_cli(&["production-generation-corpus-shape"])
        .expect("production generation corpus shape alias should render");

    assert_eq!(
        rendered,
        production_generation_corpus_shape_summary_for_report()
    );
}

#[test]
fn production_generation_corpus_shape_alias_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-corpus-shape", "extra"])
        .expect_err("production generation corpus shape alias should reject extra arguments");

    assert!(error
        .contains("production-generation-corpus-shape-summary does not accept extra arguments"));
}

#[test]
fn production_generation_source_summary_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-source-summary", "extra"])
        .expect_err("production generation source summary should reject extra arguments");

    assert!(error.contains("production-generation-source-summary does not accept extra arguments"));
}

#[test]
fn production_generation_boundary_source_summary_command_renders_the_source_block() {
    let rendered = render_cli(&["production-generation-boundary-source-summary"])
        .expect("production generation boundary source summary should render");

    assert!(rendered.contains("Production generation boundary overlay source:"));
    assert!(rendered.contains("boundary overlay source"));
    assert_eq!(
        rendered,
        production_generation_boundary_source_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_source_alias_command_renders_the_source_block() {
    let rendered = render_cli(&["production-generation-boundary-source"])
        .expect("production generation boundary source alias should render");

    assert_eq!(
        rendered,
        production_generation_boundary_source_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_source_alias_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-boundary-source", "extra"])
        .expect_err("production generation boundary source alias should reject extra arguments");

    assert!(error
        .contains("production-generation-boundary-source-summary does not accept extra arguments"));
}

#[test]
fn production_generation_boundary_window_summary_command_renders_the_window_block() {
    let rendered = render_cli(&["production-generation-boundary-window-summary"])
        .expect("production generation boundary window summary should render");

    assert!(rendered.contains("Production generation boundary windows:"));
    assert!(rendered.contains("source-backed samples"));
    assert_eq!(
        rendered,
        production_generation_boundary_window_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_window_alias_command_renders_the_window_block() {
    let rendered = render_cli(&["production-generation-boundary-window"])
        .expect("production generation boundary window alias should render");

    assert_eq!(
        rendered,
        production_generation_boundary_window_summary_for_report()
    );
}

#[test]
fn production_generation_boundary_window_alias_rejects_extra_arguments() {
    let error = render_cli(&["production-generation-boundary-window", "extra"])
        .expect_err("production generation boundary window alias should reject extra arguments");

    assert!(error.contains("production-generation-boundary-window does not accept extra arguments"));
}

#[test]
fn production_generation_source_summary_command_renders_the_merged_source_block() {
    let rendered = render_cli(&["production-generation-source-summary"])
        .expect("production generation source summary should render");

    let source_index = rendered
        .find("Production generation source:")
        .expect("production generation source heading should render");
    let reference_index = rendered
        .find("Reference snapshot source:")
        .expect("reference snapshot source heading should render");
    let boundary_index = rendered
        .find("Production generation boundary overlay source:")
        .expect("production generation boundary overlay heading should render");
    let window_index = rendered
        .find("source windows=")
        .expect("source-window evidence should render");
    let input_path_index = rendered
        .find("input path=")
        .expect("generation input path should render");

    assert!(source_index < reference_index);
    assert!(reference_index < boundary_index);
    assert!(boundary_index < window_index);
    assert!(window_index < input_path_index);
    assert_eq!(rendered, production_generation_source_summary_for_report());
}

#[test]
fn production_generation_source_alias_command_renders_the_merged_source_block() {
    let rendered = render_cli(&["production-generation-source"])
        .expect("production generation source alias should render");

    assert_eq!(rendered, production_generation_source_summary_for_report());
    assert_eq!(
        render_cli(&["production-generation-source", "extra"])
            .expect_err("production generation source alias should reject extra arguments"),
        "production-generation-source does not accept extra arguments"
    );
}

#[test]
fn production_generation_source_revision_summary_command_renders_the_revision_block() {
    let rendered = render_cli(&["production-generation-source-revision-summary"])
        .expect("production generation source revision summary should render");

    assert!(rendered.contains("source revision=reference_snapshot.csv checksum=0x"));
    assert!(rendered.contains("independent_holdout_snapshot.csv checksum=0x"));
    assert_eq!(
        rendered,
        production_generation_source_revision_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-source-revision"])
            .expect("production generation source revision alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["production-generation-source-revision", "extra"]).expect_err(
            "production generation source revision alias should reject extra arguments"
        ),
        "production-generation-source-revision-summary does not accept extra arguments"
    );
}

#[test]
fn production_generation_manifest_summary_command_renders_the_manifest_block() {
    let rendered = render_cli(&["production-generation-manifest-summary"])
        .expect("production generation manifest summary should render");

    assert!(rendered.contains("Production generation manifest:"));
    assert!(rendered.contains("coverage="));
    assert_eq!(
        rendered,
        production_generation_manifest_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-manifest", "extra"])
            .expect_err("production generation manifest alias should reject extra arguments"),
        "production-generation-manifest-summary does not accept extra arguments"
    );
}

#[test]
fn production_generation_manifest_checksum_summary_command_renders_the_checksum() {
    let rendered = render_cli(&["production-generation-manifest-checksum-summary"])
        .expect("production generation manifest checksum summary should render");

    assert!(rendered.contains("Production generation manifest checksum: 0x"));
    assert_eq!(
        rendered,
        production_generation_manifest_checksum_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-manifest-checksum", "extra"]).expect_err(
            "production generation manifest checksum alias should reject extra arguments",
        ),
        "production-generation-manifest-checksum-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_generation_manifest_summary_command_renders_the_manifest_block() {
    let rendered = render_cli(&["packaged-artifact-generation-manifest-summary"])
        .expect("packaged artifact generation manifest summary should render");

    assert!(rendered.contains("Packaged artifact generation manifest:"));
    assert!(rendered.contains("coverage="));
    assert_eq!(rendered, packaged_artifact_generation_manifest_for_report());
    assert_eq!(
        render_cli(&["packaged-artifact-generation-manifest", "extra"]).expect_err(
            "packaged artifact generation manifest alias should reject extra arguments"
        ),
        "packaged-artifact-generation-manifest-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_generation_manifest_summary_validation_rejects_drift() {
    let summary = packaged_artifact_generation_manifest_for_report();
    let drifted_summary = summary.replace("coverage=", "coverage=drifted-");

    let error = ensure_packaged_artifact_generation_manifest_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted packaged artifact generation manifest summary should be rejected");
    assert!(error
        .to_string()
        .contains("generation manifest summary no longer matches"));
}

#[test]
fn packaged_artifact_generation_manifest_checksum_summary_command_renders_the_checksum() {
    let rendered = render_cli(&["packaged-artifact-generation-manifest-checksum-summary"])
        .expect("packaged artifact generation manifest checksum summary should render");

    assert!(rendered.contains("Packaged artifact generation manifest checksum: 0x"));
    assert_eq!(
        rendered,
        packaged_artifact_generation_manifest_checksum_for_report()
    );
    assert_eq!(
        render_cli(&["packaged-artifact-generation-manifest-checksum", "extra"]).expect_err(
            "packaged artifact generation manifest checksum alias should reject extra arguments"
        ),
        "packaged-artifact-generation-manifest-checksum-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_access_summary_and_alias_commands_render_the_summary() {
    let access = render_cli(&["packaged-artifact-access-summary"])
        .expect("packaged artifact access summary should render");
    assert_eq!(
        access,
        format!(
            "Packaged-artifact access: {}",
            packaged_artifact_access_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-access"])
            .expect("packaged artifact access alias should render"),
        access
    );
    assert_eq!(
        render_cli(&["packaged-artifact-path-policy-summary"])
            .expect("packaged artifact path policy summary should render"),
        access
    );
    assert_eq!(
        render_cli(&["packaged-artifact-path-policy"])
            .expect("packaged artifact path policy alias should render"),
        access
    );
    assert_eq!(
        render_cli(&["packaged-artifact-path-policy-summary", "extra"])
            .expect_err("packaged artifact path policy summary should reject extra arguments"),
        "packaged-artifact-path-policy-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-path-policy", "extra"])
            .expect_err("packaged artifact path policy alias should reject extra arguments"),
        "packaged-artifact-path-policy does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-access-summary", "extra"])
            .expect_err("packaged artifact access summary should reject extra arguments"),
        "packaged-artifact-access-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-access", "extra"])
            .expect_err("packaged artifact access alias should reject extra arguments"),
        "packaged-artifact-access does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_output_support_and_storage_aliases_render_the_summary() {
    let output_support = render_cli(&["packaged-artifact-output-support-summary"])
        .expect("packaged artifact output support summary should render");
    assert_eq!(
        output_support,
        format!(
            "Packaged-artifact output support: {}",
            packaged_artifact_output_support_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-output-support"])
            .expect("packaged artifact output support alias should render"),
        output_support
    );
    assert_eq!(
        render_cli(&["packaged-artifact-output-support", "extra"])
            .expect_err("packaged artifact output support alias should reject extra arguments"),
        "packaged-artifact-output-support does not accept extra arguments"
    );

    let speed_policy = render_cli(&["packaged-artifact-speed-policy-summary"])
        .expect("packaged artifact speed policy summary should render");
    assert_eq!(
        speed_policy,
        format!(
            "Packaged-artifact speed policy: {}",
            packaged_artifact_speed_policy_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-speed-policy"])
            .expect("packaged artifact speed policy alias should render"),
        speed_policy
    );
    assert_eq!(
        render_cli(&["packaged-artifact-speed-policy", "extra"])
            .expect_err("packaged artifact speed policy alias should reject extra arguments"),
        "packaged-artifact-speed-policy-summary does not accept extra arguments"
    );

    let motion_policy =
        render_cli(&["motion-policy-summary"]).expect("motion policy summary should render");
    assert_eq!(
        motion_policy,
        format!(
            "Motion policy: {}",
            packaged_artifact_speed_policy_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["motion-policy"]).expect("motion policy alias should render"),
        motion_policy
    );
    assert_eq!(
        render_cli(&["motion-policy", "extra"])
            .expect_err("motion policy alias should reject extra arguments"),
        "motion-policy-summary does not accept extra arguments"
    );

    let storage = render_cli(&["packaged-artifact-storage-summary"])
        .expect("packaged artifact storage summary should render");
    assert_eq!(
        storage,
        format!(
            "Packaged-artifact storage/reconstruction: {}",
            validated_packaged_artifact_storage_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-storage"])
            .expect("packaged artifact storage alias should render"),
        storage
    );
    assert_eq!(
        render_cli(&["packaged-artifact-storage", "extra"])
            .expect_err("packaged artifact storage alias should reject extra arguments"),
        "packaged-artifact-storage does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_fit_sample_classes_summary_and_alias_commands_render_the_combined_fit_summary()
{
    let fit_sample_classes = render_cli(&["packaged-artifact-fit-sample-classes-summary"])
        .expect("packaged artifact fit sample classes summary should render");
    assert!(fit_sample_classes.contains("Packaged-artifact fit sample classes: "));
    assert_eq!(
        fit_sample_classes,
        format!(
            "Packaged-artifact fit sample classes: {}",
            packaged_artifact_fit_sample_classes_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-sample-classes"])
            .expect("packaged artifact fit sample classes alias should render"),
        fit_sample_classes
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-sample-classes-summary", "extra"]).expect_err(
            "packaged artifact fit sample classes summary should reject extra arguments"
        ),
        "packaged-artifact-fit-sample-classes-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-sample-classes", "extra"])
            .expect_err("packaged artifact fit sample classes alias should reject extra arguments"),
        "packaged-artifact-fit-sample-classes-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_fit_sample_classes_summary_validation_rejects_invalid_boundary_envelope() {
    let invalid_boundary = ArtifactBoundaryEnvelopeSummary {
        body_count: 1,
        boundary_check_count: 1,
        sum_boundary_longitude_delta_deg: f64::NAN,
        sum_boundary_longitude_delta_deg_sq: 0.0,
        sum_boundary_latitude_delta_deg: 0.0,
        sum_boundary_latitude_delta_deg_sq: 0.0,
        sum_boundary_distance_delta_au: None,
        sum_boundary_distance_delta_au_sq: None,
        boundary_distance_check_count: 0,
        max_boundary_longitude_delta_body: Some(CelestialBody::Sun),
        max_boundary_longitude_delta_deg: 0.0,
        max_boundary_latitude_delta_body: Some(CelestialBody::Sun),
        max_boundary_latitude_delta_deg: 0.0,
        max_boundary_distance_delta_body: None,
        max_boundary_distance_delta_au: None,
    };

    let error =
        validated_packaged_artifact_fit_sample_classes_summary_for_report(&invalid_boundary)
            .expect_err("invalid boundary envelope should fail validation before rendering");
    assert!(error.contains("must be finite"));
}

#[test]
fn packaged_artifact_fit_margins_summary_and_alias_commands_render_the_fit_margins() {
    let fit_margins = render_cli(&["packaged-artifact-fit-margins-summary"])
        .expect("packaged artifact fit margins summary should render");
    assert!(fit_margins.contains("Packaged-artifact fit margins: "));
    assert_eq!(
        fit_margins,
        format!(
            "Packaged-artifact fit margins: {}",
            pleiades_data::packaged_artifact_fit_margin_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-margins"])
            .expect("packaged artifact fit margins alias should render"),
        fit_margins
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-margins-summary", "extra"])
            .expect_err("packaged artifact fit margins summary should reject extra arguments"),
        "packaged-artifact-fit-margins-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-margins", "extra"])
            .expect_err("packaged artifact fit margins alias should reject extra arguments"),
        "packaged-artifact-fit-margins-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_fit_outliers_summary_and_alias_commands_render_the_body_channel_outliers() {
    let fit_outliers = render_cli(&["packaged-artifact-fit-outliers-summary"])
        .expect("packaged artifact fit outliers summary should render");
    assert!(fit_outliers.contains("Packaged-artifact fit outliers: "));
    assert_eq!(
        fit_outliers,
        format!(
            "Packaged-artifact fit outliers: {}",
            pleiades_data::packaged_artifact_fit_outlier_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-outliers"])
            .expect("packaged artifact fit outliers alias should render"),
        fit_outliers
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-outliers-summary", "extra"])
            .expect_err("packaged artifact fit outliers summary should reject extra arguments"),
        "packaged-artifact-fit-outliers-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-outliers", "extra"])
            .expect_err("packaged artifact fit outliers alias should reject extra arguments"),
        "packaged-artifact-fit-outliers-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_body_class_span_cap_summary_and_alias_commands_render_the_summary() {
    let span_caps = render_cli(&["packaged-artifact-body-class-span-cap-summary"])
        .expect("packaged artifact body-class span cap summary should render");
    assert_eq!(
        span_caps,
        format!(
            "Packaged-artifact body-class span caps: {}",
            pleiades_data::packaged_artifact_body_class_span_cap_entries_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-body-class-span-cap"])
            .expect("packaged artifact body-class span cap alias should render"),
        span_caps
    );
    assert_eq!(
        render_cli(&["packaged-artifact-body-class-span-cap-summary", "extra"]).expect_err(
            "packaged artifact body-class span cap summary should reject extra arguments"
        ),
        "packaged-artifact-body-class-span-cap-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-body-class-span-cap", "extra"]).expect_err(
            "packaged artifact body-class span cap alias should reject extra arguments"
        ),
        "packaged-artifact-body-class-span-cap-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_production_profile_summary_and_alias_commands_render_the_profile_skeleton() {
    let production_profile = render_cli(&["packaged-artifact-production-profile-summary"])
        .expect("packaged artifact production profile summary should render");
    assert!(production_profile.contains("Packaged artifact production profile draft:"));
    assert_eq!(
        production_profile,
        validated_packaged_artifact_production_profile_summary_for_report()
    );
    assert_eq!(
        render_cli(&["packaged-artifact-production-profile"])
            .expect("packaged artifact production profile alias should render"),
        production_profile
    );
    assert_eq!(
        render_cli(&["packaged-artifact-production-profile-summary", "extra"]).expect_err(
            "packaged artifact production profile summary should reject extra arguments"
        ),
        "packaged-artifact-production-profile-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-production-profile", "extra"])
            .expect_err("packaged artifact production profile alias should reject extra arguments"),
        "packaged-artifact-production-profile does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_target_threshold_summary_and_alias_commands_render_the_summary() {
    let target_threshold = render_cli(&["packaged-artifact-target-threshold-summary"])
        .expect("packaged artifact target threshold summary should render");
    assert!(target_threshold.contains("Packaged-artifact target thresholds: "));
    assert_eq!(
        target_threshold,
        format!(
            "Packaged-artifact target thresholds: {}",
            validated_packaged_artifact_target_threshold_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold"])
            .expect("packaged artifact target threshold alias should render"),
        target_threshold
    );
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold-summary", "extra"])
            .expect_err("packaged artifact target threshold summary should reject extra arguments"),
        "packaged-artifact-target-threshold-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold", "extra"])
            .expect_err("packaged artifact target threshold alias should reject extra arguments"),
        "packaged-artifact-target-threshold-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_target_threshold_state_summary_and_alias_commands_render_the_summary() {
    let target_threshold_state = render_cli(&["packaged-artifact-target-threshold-state-summary"])
        .expect("packaged artifact target threshold state summary should render");
    assert!(target_threshold_state.contains("Packaged-artifact target-threshold state: "));
    assert_eq!(
        target_threshold_state,
        format!(
            "Packaged-artifact target-threshold state: {}",
            validated_packaged_artifact_target_threshold_state_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold-state"])
            .expect("packaged artifact target threshold state alias should render"),
        target_threshold_state
    );
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold-state-summary", "extra"]).expect_err(
            "packaged artifact target threshold state summary should reject extra arguments"
        ),
        "packaged-artifact-target-threshold-state-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold-state", "extra"]).expect_err(
            "packaged artifact target threshold state alias should reject extra arguments"
        ),
        "packaged-artifact-target-threshold-state-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_target_threshold_scope_envelopes_summary_and_alias_commands_render_the_summary(
) {
    let scope_envelopes =
        render_cli(&["packaged-artifact-target-threshold-scope-envelopes-summary"])
            .expect("packaged artifact target-threshold scope envelopes summary should render");
    assert!(scope_envelopes.contains("Packaged-artifact target-threshold scope envelopes: "));
    assert_eq!(
        scope_envelopes,
        format!(
            "Packaged-artifact target-threshold scope envelopes: {}",
            validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold-scope-envelopes"])
            .expect("packaged artifact target-threshold scope envelopes alias should render"),
        scope_envelopes
    );
    assert_eq!(
            render_cli(&["packaged-artifact-target-threshold-scope-envelopes-summary", "extra"])
                .expect_err(
                    "packaged artifact target-threshold scope envelopes summary should reject extra arguments"
                ),
            "packaged-artifact-target-threshold-scope-envelopes-summary does not accept extra arguments"
        );
    assert_eq!(
            render_cli(&["packaged-artifact-target-threshold-scope-envelopes", "extra"])
                .expect_err(
                    "packaged artifact target-threshold scope envelopes alias should reject extra arguments"
                ),
            "packaged-artifact-target-threshold-scope-envelopes-summary does not accept extra arguments"
        );
}

#[test]
fn packaged_artifact_phase2_corpus_alignment_summary_and_alias_commands_render_the_summary() {
    let phase2_alignment = render_cli(&["packaged-artifact-phase2-corpus-alignment-summary"])
        .expect("packaged artifact phase-2 corpus alignment summary should render");
    assert!(phase2_alignment.contains("Packaged-artifact phase-2 corpus alignment: "));
    assert_eq!(
        phase2_alignment,
        format!(
            "Packaged-artifact phase-2 corpus alignment: {}",
            validated_packaged_artifact_phase2_corpus_alignment_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-phase2-corpus-alignment"])
            .expect("packaged artifact phase-2 corpus alignment alias should render"),
        phase2_alignment
    );
    assert_eq!(
        render_cli(&["packaged-artifact-phase2-corpus-alignment-summary", "extra"]).expect_err(
            "packaged artifact phase-2 corpus alignment summary should reject extra arguments"
        ),
        "packaged-artifact-phase2-corpus-alignment-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-phase2-corpus-alignment", "extra"]).expect_err(
            "packaged artifact phase-2 corpus alignment alias should reject extra arguments"
        ),
        "packaged-artifact-phase2-corpus-alignment-summary does not accept extra arguments"
    );
}

#[test]
fn packaged_artifact_source_fit_holdout_sync_summary_and_alias_commands_render_the_summary() {
    let sync = render_cli(&["packaged-artifact-source-fit-holdout-sync-summary"])
        .expect("packaged artifact source-fit and hold-out sync summary should render");
    assert!(sync.contains("Packaged-artifact source-fit and hold-out sync: "));
    assert_eq!(
        sync,
        format!(
            "Packaged-artifact source-fit and hold-out sync: {}",
            validated_packaged_artifact_source_fit_holdout_sync_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-source-fit-holdout-sync"])
            .expect("packaged artifact source-fit and hold-out sync alias should render"),
        sync
    );
    assert_eq!(
        render_cli(&["packaged-artifact-source-fit-holdout-sync-summary", "extra"]).expect_err(
            "packaged artifact source-fit and hold-out sync summary should reject extra arguments"
        ),
        "packaged-artifact-source-fit-holdout-sync-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-source-fit-holdout-sync", "extra"]).expect_err(
            "packaged artifact source-fit and hold-out sync alias should reject extra arguments"
        ),
        "packaged-artifact-source-fit-holdout-sync-summary does not accept extra arguments"
    );
}
