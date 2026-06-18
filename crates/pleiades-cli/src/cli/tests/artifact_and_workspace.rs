//! Tests for artifact, workspace, and packaged-artifact commands.

use pleiades_data::packaged_artifact_generation_manifest_for_report;
use pleiades_validate::render_cli as validate_render_cli;

use super::super::test_support::unique_temp_dir;
use crate::cli::render_cli;

#[test]
fn artifact_and_workspace_commands_render_compact_reports() {
    let artifact_summary =
        render_cli(&["artifact-summary"]).expect("artifact summary should render");
    assert!(artifact_summary.contains("Artifact summary"));
    assert!(artifact_summary.contains("Artifact output support:"));
    assert!(artifact_summary.contains("Artifact boundary envelope"));
    assert!(artifact_summary.contains("Artifact fit outliers by channel"));
    assert!(artifact_summary.contains("Model error envelope"));
    assert!(artifact_summary.lines().any(|line| {
        line == format!(
            "  Packaged frame treatment: {}",
            pleiades_data::packaged_frame_treatment_summary_for_report()
        )
    }));
    assert!(artifact_summary.contains("Release summary: release-summary"));
    assert!(artifact_summary.contains("Release notes summary: release-notes-summary"));
    assert!(artifact_summary
        .contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(artifact_summary.contains("Workspace audit: workspace-audit / audit"));

    let artifact_boundary_envelope = render_cli(&["artifact-boundary-envelope-summary"])
        .expect("artifact boundary envelope summary should render");
    assert_eq!(
        artifact_boundary_envelope,
        pleiades_validate::artifact_boundary_envelope_summary_for_report()
            .expect("boundary envelope summary should validate")
            .summary_line()
    );

    let artifact_profile_coverage = render_cli(&["artifact-profile-coverage-summary"])
        .expect("artifact profile coverage summary should render");
    assert!(artifact_profile_coverage.contains("Artifact profile coverage: "));
    assert!(artifact_profile_coverage.contains("asteroid:433-Eros"));
    assert!(artifact_profile_coverage.contains("TopocentricCoordinates"));

    let packaged_artifact_output_support =
        render_cli(&["packaged-artifact-output-support-summary"])
            .expect("packaged artifact output support summary should render");
    assert!(packaged_artifact_output_support.contains("Packaged-artifact output support: "));
    assert!(packaged_artifact_output_support.contains("ApparentCorrections=unsupported"));
    assert_eq!(
        packaged_artifact_output_support,
        format!(
            "Packaged-artifact output support: {}",
            pleiades_data::packaged_artifact_output_support_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-output-support"])
            .expect("packaged artifact output support alias should render"),
        packaged_artifact_output_support
    );
    assert_eq!(
        render_cli(&["packaged-artifact-output-support", "extra"])
            .expect_err("packaged artifact output support alias should reject extra arguments"),
        "packaged-artifact-output-support does not accept extra arguments"
    );

    let packaged_artifact_body_class_span_caps =
        render_cli(&["packaged-artifact-body-class-span-cap-summary"])
            .expect("packaged artifact body-class span cap summary should render");
    assert!(
        packaged_artifact_body_class_span_caps.contains("Packaged-artifact body-class span caps: ")
    );
    assert_eq!(
        packaged_artifact_body_class_span_caps,
        format!(
            "Packaged-artifact {}",
            pleiades_data::packaged_artifact_body_class_span_cap_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-body-class-span-cap"])
            .expect("packaged artifact body-class span cap alias should render"),
        packaged_artifact_body_class_span_caps
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

    let packaged_artifact_speed_policy = render_cli(&["packaged-artifact-speed-policy-summary"])
        .expect("packaged artifact speed policy summary should render");
    assert!(packaged_artifact_speed_policy.contains("Packaged-artifact speed policy: "));
    assert!(
        packaged_artifact_speed_policy.contains("Unsupported; motion output support=unsupported")
    );
    assert_eq!(
        packaged_artifact_speed_policy,
        format!(
            "Packaged-artifact speed policy: {}",
            pleiades_data::packaged_artifact_speed_policy_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-speed-policy"])
            .expect("packaged artifact speed policy alias should render"),
        packaged_artifact_speed_policy
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
            pleiades_data::packaged_artifact_speed_policy_summary_for_report()
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

    let packaged_artifact_access = render_cli(&["packaged-artifact-access-summary"])
        .expect("packaged artifact access summary should render");
    assert!(packaged_artifact_access.contains("Packaged-artifact access: "));
    assert_eq!(
        packaged_artifact_access,
        format!(
            "Packaged-artifact access: {}",
            pleiades_data::packaged_artifact_access_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-access"])
            .expect("packaged artifact access alias should render"),
        packaged_artifact_access
    );
    assert_eq!(
        render_cli(&["packaged-artifact-path-policy-summary"])
            .expect("packaged artifact path policy summary should render"),
        packaged_artifact_access
    );
    assert_eq!(
        render_cli(&["packaged-artifact-path-policy"])
            .expect("packaged artifact path policy alias should render"),
        packaged_artifact_access
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

    let packaged_artifact_storage = render_cli(&["packaged-artifact-storage-summary"])
        .expect("packaged artifact storage summary should render");
    assert!(packaged_artifact_storage.contains("Packaged-artifact storage/reconstruction: "));
    assert!(packaged_artifact_storage.contains("equatorial coordinates are reconstructed"));
    assert_eq!(
        packaged_artifact_storage,
        format!(
            "Packaged-artifact storage/reconstruction: {}",
            pleiades_data::packaged_artifact_storage_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-storage"])
            .expect("packaged artifact storage alias should render"),
        packaged_artifact_storage
    );
    assert_eq!(
        render_cli(&["packaged-artifact-storage", "extra"])
            .expect_err("packaged artifact storage alias should reject extra arguments"),
        "packaged-artifact-storage does not accept extra arguments"
    );

    let packaged_artifact_production_profile =
        render_cli(&["packaged-artifact-production-profile-summary"])
            .expect("packaged artifact production profile summary should render");
    assert!(packaged_artifact_production_profile
        .contains("Packaged artifact production profile draft:"));
    assert!(packaged_artifact_production_profile
        .contains("profile id=pleiades-packaged-artifact-profile/stage-5-draft"));
    assert_eq!(
        packaged_artifact_production_profile,
        pleiades_data::packaged_artifact_production_profile_summary_for_report()
    );
    assert_eq!(
        render_cli(&["packaged-artifact-production-profile"])
            .expect("packaged artifact production profile alias should render"),
        packaged_artifact_production_profile
    );

    let packaged_artifact_generation_manifest =
        render_cli(&["packaged-artifact-generation-manifest-summary"])
            .expect("packaged artifact generation manifest summary should render");
    assert!(
        packaged_artifact_generation_manifest.contains("Packaged artifact generation manifest:")
    );
    assert_eq!(
        packaged_artifact_generation_manifest,
        packaged_artifact_generation_manifest_for_report()
    );
    assert_eq!(
        render_cli(&["packaged-artifact-generation-manifest"])
            .expect("packaged artifact generation manifest alias should render"),
        packaged_artifact_generation_manifest
    );

    let packaged_artifact_fit_envelope = render_cli(&["packaged-artifact-fit-envelope-summary"])
        .expect("packaged artifact fit envelope summary should render");
    assert!(packaged_artifact_fit_envelope.contains("Packaged-artifact fit envelope: "));
    assert_eq!(
        render_cli(&["packaged-artifact-fit-envelope"])
            .expect("packaged artifact fit envelope alias should render"),
        packaged_artifact_fit_envelope
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-envelope-summary", "extra"])
            .expect_err("packaged artifact fit envelope summary should reject extra arguments"),
        "packaged-artifact-fit-envelope-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-envelope", "extra"])
            .expect_err("packaged artifact fit envelope alias should reject extra arguments"),
        "packaged-artifact-fit-envelope-summary does not accept extra arguments"
    );
    assert_eq!(
        packaged_artifact_fit_envelope,
        format!(
            "Packaged-artifact fit envelope: {}",
            pleiades_data::packaged_artifact_fit_envelope_summary_for_report()
        )
    );

    let packaged_artifact_fit_sample_classes =
        render_cli(&["packaged-artifact-fit-sample-classes-summary"])
            .expect("packaged artifact fit sample classes summary should render");
    assert!(packaged_artifact_fit_sample_classes.contains("Packaged-artifact fit sample classes: "));
    assert_eq!(
        packaged_artifact_fit_sample_classes,
        format!(
            "Packaged-artifact fit sample classes: {}",
            pleiades_validate::packaged_artifact_fit_sample_classes_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-sample-classes"])
            .expect("packaged artifact fit sample classes alias should render"),
        packaged_artifact_fit_sample_classes
    );

    let packaged_artifact_fit_outliers = render_cli(&["packaged-artifact-fit-outliers-summary"])
        .expect("packaged artifact fit outliers summary should render");
    assert!(packaged_artifact_fit_outliers.contains("Packaged-artifact fit outliers: "));
    assert_eq!(
        packaged_artifact_fit_outliers,
        format!(
            "Packaged-artifact fit outliers: {}",
            pleiades_data::packaged_artifact_fit_outlier_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-fit-outliers"])
            .expect("packaged artifact fit outliers alias should render"),
        packaged_artifact_fit_outliers
    );

    let packaged_artifact_fit_threshold_violation_count =
        render_cli(&["packaged-artifact-fit-threshold-violation-count-summary"])
            .expect("packaged artifact fit threshold violation count summary should render");
    assert!(packaged_artifact_fit_threshold_violation_count
        .contains("Packaged-artifact fit threshold violation count: 0"));
    assert_eq!(
        render_cli(&["packaged-artifact-fit-threshold-violation-count"])
            .expect("packaged artifact fit threshold violation count alias should render"),
        packaged_artifact_fit_threshold_violation_count
    );

    let packaged_artifact_fit_threshold_violations =
        render_cli(&["packaged-artifact-fit-threshold-violations-summary"])
            .expect("packaged artifact fit threshold violations summary should render");
    assert!(packaged_artifact_fit_threshold_violations
        .contains("Packaged-artifact fit threshold violations: 0; details: none"));
    assert_eq!(
        render_cli(&["packaged-artifact-fit-threshold-violations"])
            .expect("packaged artifact fit threshold violations alias should render"),
        packaged_artifact_fit_threshold_violations
    );

    let packaged_artifact_regeneration = render_cli(&["packaged-artifact-regeneration-summary"])
        .expect("packaged artifact regeneration summary should render");
    assert!(packaged_artifact_regeneration.contains("Packaged-artifact regeneration: "));
    assert!(packaged_artifact_regeneration.contains("profile id="));
    assert!(packaged_artifact_regeneration
        .contains("quantization scales: stored=Longitude=9, Latitude=9, DistanceAu=10"));
    assert_eq!(
        packaged_artifact_regeneration,
        format!(
            "Packaged-artifact regeneration: {}",
            pleiades_data::packaged_artifact_regeneration_summary_for_report()
        )
    );
    let packaged_frame_parity = render_cli(&["packaged-frame-parity-summary"])
        .expect("packaged frame parity summary should render");
    assert_eq!(
        packaged_frame_parity,
        format!(
            "Packaged frame parity: {}",
            pleiades_data::packaged_frame_parity_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-frame-parity"]).expect("packaged-frame-parity should render"),
        packaged_frame_parity
    );
    assert_eq!(
        validate_render_cli(&["packaged-frame-parity"])
            .expect("packaged-frame-parity should match validation output"),
        packaged_frame_parity
    );
    assert_eq!(
        render_cli(&["packaged-frame-parity", "extra"])
            .expect_err("packaged-frame-parity should reject extra arguments"),
        "packaged-frame-parity-summary does not accept extra arguments"
    );
    let packaged_frame_treatment = render_cli(&["packaged-frame-treatment-summary"])
        .expect("packaged frame treatment summary should render");
    assert_eq!(
        packaged_frame_treatment,
        format!(
            "Packaged frame treatment: {}",
            pleiades_data::packaged_frame_treatment_summary_for_report()
        )
    );

    let packaged_artifact_target_threshold =
        render_cli(&["packaged-artifact-target-threshold-summary"])
            .expect("packaged artifact target threshold summary should render");
    assert!(packaged_artifact_target_threshold.contains("Packaged-artifact target thresholds: "));
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold"])
            .expect("packaged artifact target threshold alias should render"),
        packaged_artifact_target_threshold
    );
    assert_eq!(
        packaged_artifact_target_threshold,
        format!(
            "Packaged-artifact target thresholds: {}",
            pleiades_data::packaged_artifact_target_threshold_summary_for_report()
        )
    );

    let packaged_artifact_target_threshold_scope_envelopes =
        render_cli(&["packaged-artifact-target-threshold-scope-envelopes-summary"])
            .expect("packaged artifact target threshold scope envelopes summary should render");
    assert!(packaged_artifact_target_threshold_scope_envelopes
        .contains("Packaged-artifact target-threshold scope envelopes: "));
    assert_eq!(
        render_cli(&["packaged-artifact-target-threshold-scope-envelopes"])
            .expect("packaged artifact target threshold scope envelopes alias should render"),
        packaged_artifact_target_threshold_scope_envelopes
    );
    assert_eq!(
        packaged_artifact_target_threshold_scope_envelopes,
        format!(
            "Packaged-artifact target-threshold scope envelopes: {}",
            pleiades_data::packaged_artifact_target_threshold_scope_envelopes_for_report()
        )
    );

    let packaged_artifact_generation_policy =
        render_cli(&["packaged-artifact-generation-policy-summary"])
            .expect("packaged artifact generation policy summary should render");
    assert_eq!(
        packaged_artifact_generation_policy,
        pleiades_data::packaged_artifact_generation_policy_summary_for_report()
    );
    assert_eq!(
        render_cli(&["packaged-artifact-generation-policy"])
            .expect("packaged artifact generation policy alias should render"),
        packaged_artifact_generation_policy
    );

    let packaged_artifact_regeneration = render_cli(&["packaged-artifact-regeneration-summary"])
        .expect("packaged artifact regeneration summary should render");
    assert!(packaged_artifact_regeneration.contains("Packaged-artifact regeneration: "));
    assert!(packaged_artifact_regeneration.contains("profile id="));
    assert!(packaged_artifact_regeneration
        .contains("quantization scales: stored=Longitude=9, Latitude=9, DistanceAu=10"));
    assert_eq!(
        packaged_artifact_regeneration,
        format!(
            "Packaged-artifact regeneration: {}",
            pleiades_data::packaged_artifact_regeneration_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-regeneration"])
            .expect("packaged artifact regeneration alias should render"),
        packaged_artifact_regeneration
    );

    for (args, expected) in [
        (
            &["packaged-artifact-production-profile-summary", "extra"][..],
            "packaged-artifact-production-profile-summary does not accept extra arguments",
        ),
        (
            &["packaged-artifact-production-profile", "extra"][..],
            "packaged-artifact-production-profile does not accept extra arguments",
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
            &["packaged-artifact-fit-sample-classes-summary", "extra"][..],
            "packaged-artifact-fit-sample-classes-summary does not accept extra arguments",
        ),
        (
            &["packaged-artifact-fit-sample-classes", "extra"][..],
            "packaged-artifact-fit-sample-classes-summary does not accept extra arguments",
        ),
        (
            &["packaged-artifact-fit-outliers-summary", "extra"][..],
            "packaged-artifact-fit-outliers-summary does not accept extra arguments",
        ),
        (
            &["packaged-artifact-fit-outliers", "extra"][..],
            "packaged-artifact-fit-outliers-summary does not accept extra arguments",
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

    let packaged_artifact_generation_residual =
        render_cli(&["packaged-artifact-generation-residual-summary"])
            .expect("packaged artifact generation residual summary should render");
    assert_eq!(
        packaged_artifact_generation_residual,
        format!(
            "Packaged-artifact generation residual bodies: {}",
            pleiades_data::packaged_artifact_generation_residual_bodies_summary_for_report()
        )
    );

    let packaged_artifact_generation_residual_bodies =
        render_cli(&["packaged-artifact-generation-residual-bodies-summary"])
            .expect("packaged artifact generation residual bodies summary should render");
    assert_eq!(
        packaged_artifact_generation_residual_bodies,
        format!(
            "Packaged-artifact generation residual bodies: {}",
            pleiades_data::packaged_artifact_generation_residual_bodies_summary_for_report()
        )
    );

    let artifact_fixture_dir = unique_temp_dir("pleiades-cli-packaged-artifact");
    let artifact_fixture_path = artifact_fixture_dir.join("packaged-artifact.bin");
    let artifact_fixture_path_string = artifact_fixture_path.display().to_string();
    // The WRITE path is kernel-gated; without PLEIADES_DE_KERNEL every output
    // form (positional, --out, --output, and sidecar combinations) fails closed
    // and writes nothing. Kernel-free callers use the committed artifact (decode
    // and --check) instead.
    for args in [
        vec![
            "regenerate-packaged-artifact",
            "--out",
            &artifact_fixture_path_string,
        ],
        vec![
            "generate-packaged-artifact",
            "--out",
            &artifact_fixture_path_string,
        ],
        vec![
            "regenerate-packaged-artifact",
            "--output",
            &artifact_fixture_path_string,
        ],
        vec![
            "regenerate-packaged-artifact",
            &artifact_fixture_path_string,
        ],
    ] {
        let error = render_cli(&args)
            .expect_err("packaged artifact write path should fail closed without a kernel");
        assert!(
            error.contains("generate-packaged-artifact requires PLEIADES_DE_KERNEL"),
            "unexpected error: {error}"
        );
    }
    assert!(
        !artifact_fixture_path.exists(),
        "no artifact bytes should be written when the kernel is unset"
    );

    let regeneration_check = render_cli(&["regenerate-packaged-artifact", "--check"])
        .expect("packaged artifact check mode should render");
    assert!(regeneration_check.contains("Packaged artifact regeneration check passed"));
    assert!(regeneration_check.contains("checksum=0x"));
    assert!(!regeneration_check.contains("path:"));
    assert!(regeneration_check.contains(
        "11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"
    ));

    let artifact_report =
        render_cli(&["validate-artifact"]).expect("validate-artifact should render");
    assert!(artifact_report.contains("Artifact validation report"));
    assert!(artifact_report.contains("Artifact output support:"));
    assert!(artifact_report.contains("Bodies"));
    assert!(artifact_report.contains("Artifact boundary envelope"));
    assert!(artifact_report.contains("Model error envelope"));

    let workspace_audit = render_cli(&["workspace-audit"])
        .expect("workspace-audit should render through the primary CLI");
    let native_dependency_audit = render_cli(&["native-dependency-audit"])
        .expect("native-dependency-audit should render through the CLI");
    assert_eq!(workspace_audit, native_dependency_audit);
    assert!(workspace_audit.contains("Workspace audit"));
    assert!(workspace_audit.contains("no workspace policy violations detected"));

    let audit = render_cli(&["audit"]).expect("audit alias should render through the CLI");
    assert!(audit.contains("Workspace audit"));
    assert!(audit.contains("no workspace policy violations detected"));

    assert_eq!(
        render_cli(&["workspace-audit", "extra"]).unwrap_err(),
        "workspace-audit does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["audit", "extra"]).unwrap_err(),
        "audit does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["native-dependency-audit", "extra"]).unwrap_err(),
        "native-dependency-audit does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["workspace-audit-summary", "extra"]).unwrap_err(),
        "workspace-audit-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["native-dependency-audit-summary", "extra"]).unwrap_err(),
        "native-dependency-audit-summary does not accept extra arguments"
    );
}

#[test]
fn workspace_provenance_summary_reports_workspace_tool_versions() {
    let summary = render_cli(&["workspace-provenance-summary"])
        .expect("workspace provenance summary should render through the CLI");
    let alias = render_cli(&["workspace-provenance"])
        .expect("workspace provenance alias should render through the CLI");
    assert_eq!(summary, alias);
    assert!(summary.contains("Workspace provenance"));
    assert!(summary.contains("source revision:"));
    assert!(summary.contains("workspace status:"));
    assert!(summary.contains("rustc version:"));
    assert!(summary.contains("cargo version:"));
    assert!(summary.contains("rustfmt version:"));
    assert!(summary.contains("clippy version:"));
    assert_eq!(
        render_cli(&["workspace-provenance-summary", "extra"]).unwrap_err(),
        "workspace-provenance-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["workspace-provenance", "extra"]).unwrap_err(),
        "workspace-provenance does not accept extra arguments"
    );
}

#[test]
fn regenerate_packaged_artifact_write_path_fails_closed_without_kernel() {
    // The WRITE path (with sidecars) is kernel-gated. Without PLEIADES_DE_KERNEL
    // it fails closed before writing any output, and the kernel-free committed
    // bytes remain available via the library decode path. This replaces the
    // former snapshot-regen stability test, which no longer exercises a
    // kernel-free WRITE path.
    let artifact_fixture_dir = unique_temp_dir("pleiades-packaged-artifact-regeneration-repeat");
    let artifact_fixture_path = artifact_fixture_dir.join("packaged-artifact.bin");
    let artifact_fixture_path_string = artifact_fixture_path.display().to_string();
    let manifest_fixture_path = artifact_fixture_dir.join("packaged-artifact.manifest.txt");
    let manifest_fixture_path_string = manifest_fixture_path.display().to_string();
    let artifact_checksum_fixture_path =
        artifact_fixture_dir.join("packaged-artifact.checksum.txt");
    let artifact_checksum_fixture_path_string =
        artifact_checksum_fixture_path.display().to_string();

    let error = render_cli(&[
        "generate-packaged-artifact",
        "--out",
        &artifact_fixture_path_string,
        "--manifest-out",
        &manifest_fixture_path_string,
        "--artifact-checksum-out",
        &artifact_checksum_fixture_path_string,
    ])
    .expect_err("packaged artifact write path should fail closed without a kernel");
    assert!(
        error.contains("generate-packaged-artifact requires PLEIADES_DE_KERNEL"),
        "unexpected error: {error}"
    );
    for path in [
        &artifact_fixture_path,
        &manifest_fixture_path,
        &artifact_checksum_fixture_path,
    ] {
        assert!(
            !path.exists(),
            "no output should be written when the kernel is unset: {}",
            path.display()
        );
    }

    // The committed bytes remain the kernel-free source of truth.
    assert!(!pleiades_data::regenerate_packaged_artifact_bytes().is_empty());
}
