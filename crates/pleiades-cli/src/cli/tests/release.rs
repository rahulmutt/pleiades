//! Tests for bundle-release and verify-release-bundle commands.

use super::super::test_support::unique_temp_dir;
use crate::cli::render_cli;

#[test]
fn bundle_release_command_writes_a_staged_bundle() {
    let bundle_dir = unique_temp_dir("pleiades-cli-release-bundle");
    let bundle_dir_string = bundle_dir.display().to_string();

    let rendered = render_cli(&["bundle-release", "--out", &bundle_dir_string])
        .expect("bundle generation should render");

    assert!(rendered.contains("Release bundle"));
    assert!(rendered.contains("compatibility-profile.txt"));
    assert!(rendered.contains("bundle-manifest.checksum.txt"));
    assert!(rendered.contains("native-sidereal-policy-summary.txt"));
    assert!(rendered.contains("zodiac-policy-summary.txt"));
    assert!(rendered.contains("packaged-artifact-profile-coverage-summary.txt"));
    assert!(bundle_dir.join("bundle-manifest.txt").exists());
    assert!(bundle_dir
        .join("comparison-snapshot-body-class-coverage-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-snapshot-bridge-day-summary.txt")
        .exists());
    assert!(bundle_dir.join("catalog-inventory-summary.txt").exists());
    assert!(bundle_dir
        .join("custom-definition-ayanamsa-labels-summary.txt")
        .exists());
    assert!(bundle_dir.join("ayanamsa-provenance-summary.txt").exists());
    assert!(bundle_dir
        .join("compatibility-caveats-summary.txt")
        .exists());
    assert!(bundle_dir.join("request-policy-summary.txt").exists());
    assert!(bundle_dir.join("request-semantics-summary.txt").exists());
    assert!(bundle_dir.join("unsupported-modes-summary.txt").exists());
    assert!(bundle_dir.join("release-body-claims-summary.txt").exists());
    assert!(bundle_dir.join("pluto-fallback-summary.txt").exists());
    assert!(bundle_dir.join("time-scale-policy-summary.txt").exists());
    assert!(bundle_dir
        .join("utc-convenience-policy-summary.txt")
        .exists());
    assert!(bundle_dir.join("delta-t-policy-summary.txt").exists());
    assert!(bundle_dir.join("zodiac-policy-summary.txt").exists());
    assert!(bundle_dir
        .join("native-sidereal-policy-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("lunar-theory-source-family-summary.txt")
        .exists());
    assert!(bundle_dir.join("request-surface-summary.txt").exists());
    assert!(bundle_dir
        .join("release-profile-identifiers-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("release-house-system-canonical-names-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("release-ayanamsa-canonical-names-summary.txt")
        .exists());
    assert!(bundle_dir.join("workspace-audit-summary.txt").exists());
    assert!(bundle_dir
        .join("native-dependency-audit-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-profile-coverage-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-access-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-fit-sample-classes-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-production-profile-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-frame-treatment-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-target-threshold-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-phase2-corpus-alignment-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-generation-manifest.txt")
        .exists());
    let manifest = std::fs::read_to_string(bundle_dir.join("bundle-manifest.txt"))
        .expect("bundle manifest should be written");
    assert!(manifest.contains("packaged-artifact-profile-coverage-summary.txt"));
    assert!(manifest.contains("comparison-snapshot-body-class-coverage-summary.txt"));
    assert!(manifest.contains("packaged-artifact-access-summary.txt"));
    assert!(manifest.contains("packaged-artifact-fit-sample-classes-summary.txt"));
    assert!(
        manifest.contains("packaged-artifact fit sample classes summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains("release-body-claims-summary.txt"));
    assert!(manifest.contains("pluto-fallback-summary.txt"));
    assert!(manifest.contains("reference-snapshot-bridge-day-summary.txt"));
    assert!(manifest.contains("reference snapshot bridge day summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("packaged-artifact profile coverage summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("packaged-artifact access summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("packaged-artifact-production-profile-summary.txt"));
    assert!(manifest.contains("packaged-frame-treatment-summary.txt"));
    assert!(manifest.contains("packaged-artifact-target-threshold-summary.txt"));
    assert!(manifest.contains("packaged-artifact-phase2-corpus-alignment-summary.txt"));
    assert!(manifest
        .contains("packaged-artifact phase-2 corpus alignment summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("lunar-theory-source-family-summary.txt"));
    assert!(manifest.contains("lunar theory source family summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("zodiac-policy-summary.txt"));
    assert!(manifest.contains("workspace-audit-summary.txt"));
}

#[test]
fn verify_release_bundle_command_verifies_a_staged_bundle() {
    let bundle_dir = unique_temp_dir("pleiades-cli-release-bundle");
    let bundle_dir_string = bundle_dir.display().to_string();

    render_cli(&["bundle-release", "--out", &bundle_dir_string])
        .expect("bundle generation should succeed");
    let verified = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect("bundle verification should render");

    assert!(verified.contains("Release bundle"));
    assert!(verified.contains("compatibility-profile.txt"));
    assert!(verified.contains("comparison-snapshot-body-class-coverage-summary.txt"));
    assert!(verified.contains("catalog-inventory-summary.txt"));
    assert!(verified.contains("reference-snapshot-bridge-day-summary.txt"));
    assert!(verified.contains("custom-definition-ayanamsa-labels-summary.txt"));
    assert!(verified.contains("workspace-audit-summary.txt"));
    assert!(verified.contains("request-semantics-summary.txt"));
    assert!(verified.contains("unsupported-modes-summary.txt"));
    assert!(verified.contains("time-scale-policy-summary.txt"));
    assert!(verified.contains("delta-t-policy-summary.txt"));
    assert!(verified.contains("zodiac-policy-summary.txt"));
    assert!(verified.contains("native-sidereal-policy-summary.txt"));
    assert!(verified.contains("packaged-artifact-phase2-corpus-alignment-summary.txt"));
    assert!(verified.contains("lunar-theory-source-family-summary.txt"));
    assert!(verified.contains("release-house-system-canonical-names-summary.txt"));
    assert!(verified.contains("release-ayanamsa-canonical-names-summary.txt"));
    assert!(verified.contains("bundle-manifest.checksum.txt"));
}

#[test]
fn bundle_release_commands_reject_duplicate_output_arguments() {
    let bundle_dir = unique_temp_dir("pleiades-cli-release-bundle-duplicate-out");
    let bundle_dir_string = bundle_dir.display().to_string();

    let bundle_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--out",
        &bundle_dir_string,
    ])
    .expect_err("bundle-release should reject duplicate output arguments");
    assert!(bundle_error.contains("duplicate value for --out <dir> argument"));

    let bundle_output_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--output",
        &bundle_dir_string,
    ])
    .expect_err("bundle-release should reject mixed output aliases");
    assert!(bundle_output_error.contains("duplicate value for --out <dir> argument"));

    let verify_error = render_cli(&[
        "verify-release-bundle",
        "--out",
        &bundle_dir_string,
        "--out",
        &bundle_dir_string,
    ])
    .expect_err("verify-release-bundle should reject duplicate output arguments");
    assert!(verify_error.contains("duplicate value for --out <dir> argument"));

    let verify_output_error = render_cli(&[
        "verify-release-bundle",
        "--out",
        &bundle_dir_string,
        "--output",
        &bundle_dir_string,
    ])
    .expect_err("verify-release-bundle should reject mixed output aliases");
    assert!(verify_output_error.contains("duplicate value for --out <dir> argument"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn bundle_release_commands_accept_output_alias() {
    let bundle_dir = unique_temp_dir("pleiades-cli-release-bundle-output-alias");
    let bundle_dir_string = bundle_dir.display().to_string();

    render_cli(&["bundle-release", "--output", &bundle_dir_string])
        .expect("bundle-release should accept --output alias");
    let verified = render_cli(&["verify-release-bundle", "--output", &bundle_dir_string])
        .expect("verify-release-bundle should accept --output alias");

    assert!(verified.contains("Release bundle"));
    assert!(verified.contains("bundle-manifest.checksum.txt"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}
