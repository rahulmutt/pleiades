//! release bundle write/verify tests (part 2) (white-box; moved verbatim from the former `tests.rs`).

use super::test_support::*;
use super::*;
use pleiades_core::current_release_profile_identifiers;

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_unexpected_manifest_lines() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-unexpected-manifest-line");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let tampered = format!("{manifest}unexpected manifest note: review required\n");
    std::fs::write(&manifest_path, tampered).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for an unexpected manifest line");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("unexpected release bundle manifest line count")
    );
    assert!(error.contains("unexpected release bundle manifest line count"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_backend_matrix_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-matrix");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let corrupted = manifest.replace(
        "backend matrix checksum (fnv1a-64):",
        "backend matrix checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted backend matrix checksum");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid backend matrix checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_api_stability_summary_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-api-summary");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let corrupted = manifest.replace(
        "api stability summary checksum (fnv1a-64):",
        "api stability summary checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted API stability summary checksum");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid api stability summary checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_api_stability_summary_even_with_updated_checksum() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-api-summary-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("api-stability-summary.txt");
    let summary =
        std::fs::read_to_string(&summary_path).expect("API stability summary should exist");
    let tampered_summary =
        summary.replace("API stability summary", "Tampered API stability summary");
    std::fs::write(&summary_path, &tampered_summary)
        .expect("API stability summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("api stability summary checksum (fnv1a-64):"))
        .expect("manifest should contain the API stability summary checksum line");
    let new_checksum_line = format!(
        "api stability summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic API stability summary drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("API stability summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_output_support_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-output-support-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-output-support-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact output support summary should exist");
    let mut tampered_summary = summary;
    tampered_summary.push_str("\nTampered packaged-artifact output support summary.\n");
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact output support summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| {
            line.starts_with("packaged-artifact output support summary checksum (fnv1a-64):")
        })
        .expect(
            "manifest should contain the packaged-artifact output support summary checksum line",
        );
    let new_checksum_line = format!(
        "packaged-artifact output support summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic packaged-artifact output support drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("packaged-artifact output support summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_profile_coverage_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-profile-coverage-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-profile-coverage-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact profile coverage summary should exist");
    let mut tampered_summary = summary;
    tampered_summary.push_str("\nTampered packaged-artifact profile coverage summary.\n");
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact profile coverage summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| {
            line.starts_with("packaged-artifact profile coverage summary checksum (fnv1a-64):")
        })
        .expect(
            "manifest should contain the packaged-artifact profile coverage summary checksum line",
        );
    let new_checksum_line = format!(
        "packaged-artifact profile coverage summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
        "verification should fail for semantic packaged-artifact profile coverage drift",
    );
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("packaged-artifact profile coverage summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_fit_sample_classes_summary_even_with_updated_checksum(
) {
    let bundle_dir =
        unique_temp_dir("pleiades-release-bundle-tampered-fit-sample-classes-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-fit-sample-classes-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact fit sample classes summary should exist");
    let mut tampered_summary = summary;
    tampered_summary.push_str("\nTampered packaged-artifact fit sample classes summary.\n");
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact fit sample classes summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
            .lines()
            .find(|line| line.starts_with("packaged-artifact fit sample classes summary checksum (fnv1a-64):"))
            .expect("manifest should contain the packaged-artifact fit sample classes summary checksum line");
    let new_checksum_line = format!(
        "packaged-artifact fit sample classes summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
        "verification should fail for semantic packaged-artifact fit sample classes drift",
    );
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("packaged-artifact fit sample classes summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_fit_threshold_violation_count_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-fit-threshold-violation-count-semantic",
            "packaged-artifact-fit-threshold-violation-count-summary.txt",
            "packaged-artifact fit threshold violation count summary checksum (fnv1a-64):",
            "fit threshold violations: 0",
            "fit threshold violations: 1",
            "packaged-artifact fit threshold violation count summary no longer matches the current packaged-artifact fit threshold violation count posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_fit_threshold_violations_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-fit-threshold-violations-semantic",
            "packaged-artifact-fit-threshold-violations-summary.txt",
            "packaged-artifact fit threshold violations summary checksum (fnv1a-64):",
            "fit threshold violations: 0; details: none",
            "fit threshold violations: 1; details: tampered",
            "packaged-artifact fit threshold violations summary no longer matches the current packaged-artifact fit threshold violations posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_storage_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-storage-semantic",
            "packaged-artifact-storage-summary.txt",
            "packaged-artifact storage summary checksum (fnv1a-64):",
            "reconstructed at runtime",
            "reconstructed at validation time",
            "packaged-artifact storage summary no longer matches the current packaged-artifact storage posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_frame_treatment_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-frame-treatment-semantic",
            "packaged-frame-treatment-summary.txt",
            "packaged-frame-treatment summary checksum (fnv1a-64):",
            "stores ecliptic coordinates directly",
            "stores ecliptic coordinates explicitly",
            "packaged frame treatment summary no longer matches the current packaged frame treatment posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_access_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-access-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-access-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact access summary should exist");
    let mut tampered_summary = summary;
    tampered_summary.push_str("\nTampered packaged-artifact access summary.\n");
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact access summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("packaged-artifact access summary checksum (fnv1a-64):"))
        .expect("manifest should contain the packaged-artifact access summary checksum line");
    let new_checksum_line = format!(
        "packaged-artifact access summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic packaged-artifact access drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("packaged-artifact access summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_speed_policy_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-speed-policy-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-speed-policy-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact speed policy summary should exist");
    let mut tampered_summary = summary;
    tampered_summary.push_str("\nTampered packaged-artifact speed policy summary.\n");
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact speed policy summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| {
            line.starts_with("packaged-artifact speed policy summary checksum (fnv1a-64):")
        })
        .expect("manifest should contain the packaged-artifact speed policy summary checksum line");
    let new_checksum_line = format!(
        "packaged-artifact speed policy summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic packaged-artifact speed policy drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("packaged-artifact speed policy summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_regeneration_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-packaged-artifact-regeneration-semantic",
            "packaged-artifact-regeneration-summary.txt",
            "packaged-artifact regeneration summary checksum (fnv1a-64):",
            "Packaged artifact regeneration source: ",
            "Packaged artifact regeneration source (drifted): ",
            "packaged-artifact regeneration summary no longer matches the current packaged-artifact regeneration posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_generation_policy_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-generation-policy-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-generation-policy-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact generation policy summary should exist");
    let mut tampered_summary = summary;
    tampered_summary.push_str("\nTampered packaged-artifact generation policy summary.\n");
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact generation policy summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| {
            line.starts_with("packaged-artifact generation policy summary checksum (fnv1a-64):")
        })
        .expect(
            "manifest should contain the packaged-artifact generation policy summary checksum line",
        );
    let new_checksum_line = format!(
        "packaged-artifact generation policy summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
        "verification should fail for semantic packaged-artifact generation policy drift",
    );
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("packaged-artifact generation policy summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_generation_residual_bodies_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-packaged-artifact-generation-residual-bodies-semantic",
            "packaged-artifact-generation-residual-bodies-summary.txt",
            "packaged-artifact generation residual bodies summary checksum (fnv1a-64):",
            "residual bodies: ",
            "residual bodies (drifted): ",
            "packaged-artifact generation residual bodies summary no longer matches the current packaged-artifact residual-body posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_production_profile_summary_even_with_updated_checksum(
) {
    let bundle_dir =
        unique_temp_dir("pleiades-release-bundle-tampered-production-profile-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-production-profile-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact production-profile summary should exist");
    let tampered_summary = summary.replace(
        "Packaged artifact production profile draft",
        "Tampered packaged artifact production profile draft",
    );
    assert_ne!(
        summary, tampered_summary,
        "summary should change under the regression edit"
    );
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact production-profile summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
            .lines()
            .find(|line| {
                line.starts_with("packaged-artifact production-profile summary checksum (fnv1a-64):")
            })
            .expect(
                "manifest should contain the packaged-artifact production-profile summary checksum line",
            );
    let new_checksum_line = format!(
        "packaged-artifact production-profile summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
        "verification should fail for semantic packaged-artifact production-profile drift",
    );
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("packaged-artifact production-profile summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_lookup_epoch_policy_summary_even_with_updated_checksum(
) {
    let bundle_dir =
        unique_temp_dir("pleiades-release-bundle-tampered-lookup-epoch-policy-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-lookup-epoch-policy-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged lookup epoch policy summary should exist");
    let tampered_summary = summary.replace(
        "TT-grid retag without relativistic correction",
        "drifted TT-grid retag without relativistic correction",
    );
    assert_ne!(
        summary, tampered_summary,
        "summary should change under the regression edit"
    );
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged lookup epoch policy summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
            .lines()
            .find(|line| {
                line.starts_with("packaged-artifact lookup-epoch policy summary checksum (fnv1a-64):")
            })
            .expect(
                "manifest should contain the packaged-artifact lookup-epoch policy summary checksum line",
            );
    let new_checksum_line = format!(
        "packaged-artifact lookup-epoch policy summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic packaged lookup epoch policy drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("lookup-epoch policy summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_release_notes_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-notes");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let corrupted = manifest.replace(
        "release notes checksum (fnv1a-64):",
        "release notes checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted release notes checksum");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid release notes checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_compatibility_profile_summary_file() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-summary");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("compatibility-profile-summary.txt");
    let summary =
        std::fs::read_to_string(&summary_path).expect("compatibility profile summary should exist");
    let tampered = summary.replace(
        "Compatibility profile summary",
        "Tampered compatibility profile summary",
    );
    std::fs::write(&summary_path, tampered).expect("summary should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered compatibility profile summary");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("compatibility profile summary checksum mismatch")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_release_notes_file() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-notes");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let notes_path = bundle_dir.join("release-notes.txt");
    let mut notes = std::fs::read_to_string(&notes_path).expect("release notes should exist");
    notes.push_str("\nTampered for regression coverage.\n");
    std::fs::write(&notes_path, notes).expect("release notes should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered release notes file");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("release notes checksum mismatch")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_backend_matrix_summary_file() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-matrix-summary");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("backend-matrix-summary.txt");
    let summary =
        std::fs::read_to_string(&summary_path).expect("backend matrix summary should exist");
    let tampered = summary.replace("Backend matrix summary", "Tampered backend matrix summary");
    std::fs::write(&summary_path, tampered).expect("backend matrix summary should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered backend matrix summary");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("backend matrix summary checksum mismatch")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_backend_matrix_file_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-backend-matrix-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("backend-matrix.txt");
    let summary = std::fs::read_to_string(&summary_path).expect("backend matrix should exist");
    let from = selected_asteroid_source_evidence_summary_for_report();
    let to = from.replacen(
        "Selected asteroid source evidence",
        "Tampered selected asteroid source evidence",
        1,
    );
    let tampered = summary.replace(&from, &to);
    assert_ne!(
        summary, tampered,
        "backend matrix should change under the regression edit"
    );
    std::fs::write(&summary_path, &tampered).expect("backend matrix should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("backend matrix checksum (fnv1a-64):"))
        .expect("manifest should contain the backend matrix checksum line");
    let new_checksum_line = format!(
        "backend matrix checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantically tampered backend matrix text");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("selected-asteroid source evidence/window posture")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_release_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-release-summary",
        "release-summary.txt",
        "release summary checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_compatibility_profile_summary_file() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-compatibility-profile-summary",
            "compatibility-profile-summary.txt",
            "profile summary checksum (fnv1a-64):",
            "Compatibility profile summary",
            "Tampered compatibility profile summary",
            "compatibility profile summary no longer matches the current compatibility profile summary posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_release_summary_file() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-release-summary",
        "release-summary.txt",
        "release summary checksum (fnv1a-64):",
        "Release summary",
        "Tampered release summary",
        "release summary no longer matches the current release summary posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_release_body_claims_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-release-body-claims-summary",
            "release-body-claims-summary.txt",
            "release body claims summary checksum (fnv1a-64):",
            "release-grade major-body claims",
            "release-grade major-body claims (validated)",
            "release body claims summary no longer matches the current release-grade body-claims posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_pluto_fallback_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-pluto-fallback-summary",
            "pluto-fallback-summary.txt",
            "pluto fallback summary checksum (fnv1a-64):",
            "Pluto remains an explicitly approximate fallback; release-grade major-body claims exclude Pluto",
            "Pluto remains an explicitly approximate fallback (drifted); release-grade major-body claims exclude Pluto",
            "Pluto fallback summary no longer matches the current Pluto fallback posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_release_notes_file() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-release-notes",
        "release-notes.txt",
        "release notes checksum (fnv1a-64):",
        "Release notes",
        "Tampered release notes",
        "release notes no longer matches the current release notes posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_source_corpus_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-source-corpus-summary",
            "source-corpus-summary.txt",
            "source-corpus summary checksum (fnv1a-64):",
            "coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1900-2100 CE window; coverage=",
            "coverage posture=drifted production-generation coverage and corpus shape remain aligned across the advertised 1900-2100 CE window; coverage=",
            "source corpus summary no longer matches the current source-corpus posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_comparison_snapshot_manifest_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-comparison-snapshot-manifest",
            "comparison-snapshot-manifest-summary.txt",
            "comparison-snapshot manifest summary checksum (fnv1a-64):",
            "redistribution=repository-checked regression fixtures, not a broad public corpus.",
            "redistribution=drifted repository-checked regression fixtures, not a broad public corpus.",
            "comparison snapshot manifest summary no longer matches the current comparison snapshot manifest posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_reference_snapshot_manifest_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-reference-snapshot-manifest",
            "reference-snapshot-manifest-summary.txt",
            "reference snapshot manifest summary checksum (fnv1a-64):",
            "source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
            "source=drifted NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
            "reference snapshot manifest summary no longer matches the current reference snapshot manifest posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_reference_snapshot_2451918_major_body_boundary_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-reference-snapshot-2451918-major-body-boundary",
            "reference-snapshot-2451918-major-body-boundary-summary.txt",
            "reference snapshot 2451918 major-body boundary summary checksum (fnv1a-64):",
            "Reference 2451918 major-body boundary evidence:",
            "Tampered 2451918 major-body boundary evidence:",
            "reference snapshot 2451918 major-body boundary summary no longer matches the current reference snapshot 2451918 major-body boundary posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_reference_snapshot_2451919_major_body_boundary_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-reference-snapshot-2451919-major-body-boundary",
            "reference-snapshot-2451919-major-body-boundary-summary.txt",
            "reference snapshot 2451919 major-body boundary summary checksum (fnv1a-64):",
            "Reference 2451919 major-body boundary evidence:",
            "Tampered 2451919 major-body boundary evidence:",
            "reference snapshot 2451919 major-body boundary summary no longer matches the current reference snapshot 2451919 major-body boundary posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_release_checklist_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-release-checklist-summary",
        "release-checklist-summary.txt",
        "release checklist summary checksum (fnv1a-64):",
        "Release checklist summary",
        "Tampered release checklist summary",
        "release checklist summary no longer matches the current release checklist summary posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_compatibility_caveats_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-compatibility-caveats-summary",
        "compatibility-caveats-summary.txt",
        "compatibility caveats summary checksum (fnv1a-64):",
        "Compatibility caveats summary",
        "Tampered compatibility caveats summary",
        "compatibility caveats summary no longer matches the current compatibility-caveats posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_release_profile_identifiers_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-release-profile-identifiers-summary",
        "release-profile-identifiers-summary.txt",
        "release-profile identifiers summary checksum (fnv1a-64):",
        "Summary line: v1 compatibility=",
        "Summary line: tampered compatibility=",
        "release-profile identifiers summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_custom_definition_ayanamsa_labels_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-custom-definition-ayanamsa-labels-summary",
        "custom-definition-ayanamsa-labels-summary.txt",
        "custom-definition ayanamsa labels summary checksum (fnv1a-64):",
        "Babylonian (House)",
        "Tampered (House)",
        "custom-definition ayanamsa labels summary mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_ayanamsa_provenance_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-ayanamsa-provenance-summary",
        "ayanamsa-provenance-summary.txt",
        "ayanamsa provenance summary checksum (fnv1a-64):",
        "representative provenance examples:",
        "tampered provenance examples:",
        "ayanamsa provenance summary no longer matches the current ayanamsa provenance posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_release_notes_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-release-notes-summary",
        "release-notes-summary.txt",
        "release notes summary checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_comparison_envelope_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-semantic-comparison-envelope",
        "comparison-envelope-summary.txt",
        "comparison-envelope summary checksum (fnv1a-64):",
        "Summary line:",
        "Summary line (tampered):",
        "comparison envelope summary no longer matches the current comparison envelope posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_semantically_tampered_comparison_body_class_tolerance_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-comparison-body-class-tolerance",
            "comparison-body-class-tolerance-summary.txt",
            "comparison-body-class-tolerance summary checksum (fnv1a-64):",
            "Body-class tolerance posture:",
            "Body-class tolerance posture (tampered):",
            "comparison body-class tolerance summary no longer matches the current comparison body-class tolerance posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_comparison_body_class_error_envelope_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-comparison-body-class-error-envelope-semantic",
            "comparison-body-class-error-envelope-summary.txt",
            "comparison-body-class-error-envelope summary checksum (fnv1a-64):",
            "2 classes checked",
            "3 classes checked",
            "comparison body-class error-envelope summary no longer matches the current comparison body-class error-envelope posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_target_threshold_scope_envelopes_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-target-threshold-scope-envelopes-semantic",
            "packaged-artifact-target-threshold-scope-envelopes-summary.txt",
            "packaged-artifact target-threshold scope envelopes summary checksum (fnv1a-64):",
            "scope=luminaries; bodies=2 (Sun, Moon); fit envelope:",
            "scope=luminaries; bodies=2 (Sun, Moon); drifted fit envelope:",
            "packaged-artifact target-threshold scope envelopes summary no longer matches the current packaged-artifact target-threshold scope envelopes posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_comparison_corpus_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-comparison-corpus-semantic",
        "comparison-corpus-summary.txt",
        "comparison-corpus summary checksum (fnv1a-64):",
        "Comparison corpus summary",
        "Tampered comparison corpus summary",
        "comparison corpus summary no longer matches the current comparison-corpus posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_comparison_corpus_release_guard_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-comparison-corpus-release-guard-semantic",
            "comparison-corpus-release-guard-summary.txt",
            "comparison-corpus release-guard summary checksum (fnv1a-64):",
            "Pluto excluded from tolerance evidence",
            "Pluto excluded from tolerance evidence (tampered)",
            "comparison-corpus release-guard summary no longer matches the current comparison-corpus release-guard posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_source_fit_holdout_sync_summary_even_with_updated_checksum(
) {
    let bundle_dir =
        unique_temp_dir("pleiades-release-bundle-tampered-source-fit-holdout-sync-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("packaged-artifact-source-fit-holdout-sync-summary.txt");
    let summary = std::fs::read_to_string(&summary_path)
        .expect("packaged-artifact source-fit and hold-out sync summary should exist");
    let tampered_summary = summary.replace(
        "source-fit and hold-out sync",
        "tampered source-fit and hold-out sync",
    );
    std::fs::write(&summary_path, &tampered_summary)
        .expect("packaged-artifact source-fit and hold-out sync summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
            .lines()
            .find(|line| {
                line.starts_with(
                    "packaged-artifact source-fit and hold-out sync summary checksum (fnv1a-64):",
                )
            })
            .expect("manifest should contain the packaged-artifact source-fit and hold-out sync summary checksum line");
    let new_checksum_line = format!(
        "packaged-artifact source-fit and hold-out sync summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_summary)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for semantic packaged-artifact source-fit and hold-out sync drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(
            error.contains("packaged-artifact source-fit and hold-out sync summary no longer matches the current packaged-artifact source-fit and hold-out sync posture")
                || error.contains("validation report summary no longer matches the current validation report posture")
                || error.contains("unexpected release bundle directory contents")
                || error.contains("unexpected release bundle manifest line count"),
            "{}",
            error
        );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_holdout_overlap_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-holdout-overlap-semantic",
            "reference-holdout-overlap-summary.txt",
            "reference-holdout overlap summary checksum (fnv1a-64):",
            "Reference/hold-out overlap:",
            "Reference/hold-out overlap (drifted):",
            "reference/hold-out overlap summary no longer matches the current reference/hold-out overlap posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_bridge_day_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-bridge-day-semantic",
            "reference-snapshot-bridge-day-summary.txt",
            "reference snapshot bridge day summary checksum (fnv1a-64):",
            "2451914.0",
            "2451914.1",
            "reference snapshot bridge day summary no longer matches the current reference snapshot bridge day posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_2451917_boundary_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-2451917-boundary-semantic",
            "reference-snapshot-2451917-major-body-boundary-summary.txt",
            "reference snapshot 2451917 major-body boundary summary checksum (fnv1a-64):",
            "JD 2451917.5 (TDB)",
            "JD 2451917.6 (TDB)",
            "reference snapshot 2451917 major-body boundary summary no longer matches the current reference snapshot 2451917 major-body boundary posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_2451916_dense_boundary_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-2451916-dense-boundary-semantic",
            "reference-snapshot-2451916-major-body-dense-boundary-summary.txt",
            "reference snapshot 2451916 major-body dense boundary summary checksum (fnv1a-64):",
            "JD 2451916.5 (TDB)",
            "JD 2451916.6 (TDB)",
            "reference snapshot 2451916 major-body dense boundary summary no longer matches the current reference snapshot 2451916 major-body dense boundary posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_boundary_summary_files_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-major-body-boundary-window-semantic",
            "reference-snapshot-major-body-boundary-window-summary.txt",
            "reference snapshot major-body boundary window summary checksum (fnv1a-64):",
            "Reference major-body boundary windows:",
            "Reference major-body boundary windows (drifted):",
            "reference snapshot major-body boundary window summary no longer matches the current reference snapshot major-body boundary window posture",
        );
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-boundary-epoch-coverage-semantic",
            "reference-snapshot-boundary-epoch-coverage-summary.txt",
            "reference snapshot boundary epoch coverage summary checksum (fnv1a-64):",
            "Reference snapshot boundary epoch coverage:",
            "Reference snapshot boundary epoch coverage (drifted):",
            "reference snapshot boundary epoch coverage summary no longer matches the current reference snapshot boundary epoch coverage posture",
        );
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-pre-bridge-boundary-semantic",
            "reference-snapshot-pre-bridge-boundary-summary.txt",
            "reference snapshot pre-bridge boundary summary checksum (fnv1a-64):",
            "Reference snapshot pre-bridge boundary day:",
            "Reference snapshot pre-bridge boundary day (drifted):",
            "reference snapshot pre-bridge boundary summary no longer matches the current reference snapshot pre-bridge boundary posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_sparse_boundary_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-sparse-boundary-semantic",
            "reference-snapshot-sparse-boundary-summary.txt",
            "reference snapshot sparse boundary summary checksum (fnv1a-64):",
            "Reference snapshot boundary day:",
            "Reference snapshot boundary day (drifted):",
            "reference snapshot sparse boundary summary no longer matches the current reference snapshot boundary day posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-reference-snapshot-summary-semantic",
        "reference-snapshot-summary.txt",
        "reference snapshot summary checksum (fnv1a-64):",
        "Reference snapshot coverage:",
        "Tampered reference snapshot coverage:",
        "reference snapshot summary no longer matches the current reference snapshot coverage",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_exact_j2000_evidence_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-exact-j2000-semantic",
            "reference-snapshot-exact-j2000-evidence-summary.txt",
            "reference snapshot exact J2000 evidence summary checksum (fnv1a-64):",
            "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0",
            "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.1",
            "reference snapshot exact J2000 evidence summary no longer matches the current reference snapshot exact J2000 evidence posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_snapshot_equatorial_parity_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-snapshot-equatorial-parity-semantic",
            "reference-snapshot-equatorial-parity-summary.txt",
            "reference snapshot equatorial parity summary checksum (fnv1a-64):",
            "JPL reference snapshot equatorial parity:",
            "JPL reference snapshot equatorial parity (drifted):",
            "reference snapshot equatorial parity summary no longer matches the current reference snapshot equatorial parity posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_boundary_source_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-boundary-source-semantic",
            "production-generation-boundary-source-summary.txt",
            "production generation boundary source summary checksum (fnv1a-64):",
            "Mars and Jupiter at 2001-01-01",
            "drifted Mars and Jupiter at 2001-01-01",
            "production generation boundary source summary no longer matches the current production-generation boundary source posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_boundary_window_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-boundary-window-semantic",
            "production-generation-boundary-window-summary.txt",
            "production generation boundary window summary checksum (fnv1a-64):",
            "source-backed samples",
            "drifted source-backed samples",
            "production generation boundary window summary no longer matches the current production-generation boundary window posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_boundary_request_corpus_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-boundary-request-corpus-semantic",
            "production-generation-boundary-request-corpus-summary.txt",
            "production generation boundary request corpus summary checksum (fnv1a-64):",
            "66 requests",
            "67 drifted requests",
            "production generation boundary request corpus summary no longer matches the current production-generation boundary request corpus posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_catalog_posture_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-catalog-posture-semantic",
        "catalog-posture-summary.txt",
        "catalog posture summary checksum (fnv1a-64):",
        current_compatibility_profile().known_gaps[0],
        "Tampered compatibility caveat",
        "catalog posture summary no longer matches the current catalog posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_jpl_provenance_only_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-jpl-provenance-only-semantic",
            "jpl-provenance-only-summary.txt",
            "jpl provenance-only evidence summary checksum (fnv1a-64):",
            "source and manifest summaries are provenance-only evidence",
            "source and manifest summaries are provenance-only evidence (drifted)",
            "JPL provenance-only evidence summary no longer matches the current JPL provenance-only posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_jpl_source_posture_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-jpl-source-posture-semantic",
            "jpl-source-posture-summary.txt",
            "jpl source posture summary checksum (fnv1a-64):",
            "JPL source posture: documented hybrid snapshot/hold-out fixture backend with a separate generation-input path; pure-Rust include_str! ingestion and reusable CSV parsing entry points; not a broad public reader/corpus provider",
            "JPL source posture: drifted",
            "JPL source posture summary no longer matches the current JPL source posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_boundary_request_corpus_equatorial_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-boundary-request-corpus-equatorial-semantic",
            "production-generation-boundary-request-corpus-equatorial-summary.txt",
            "production generation boundary request corpus equatorial summary checksum (fnv1a-64):",
            "66 requests",
            "67 drifted requests",
            "production generation boundary request corpus equatorial summary no longer matches the current production-generation boundary request corpus equatorial posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_interpolation_quality_request_corpus_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-interpolation-quality-request-corpus-semantic",
            "interpolation-quality-request-corpus-summary.txt",
            "interpolation-quality sample request corpus summary checksum (fnv1a-64):",
            "Interpolation-quality sample request corpus:",
            "Interpolation-quality sample request corpus (drifted):",
            "interpolation-quality sample request corpus summary no longer matches the current interpolation-quality sample request corpus posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_selected_asteroid_source_request_corpus_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-selected-asteroid-source-request-corpus-semantic",
            "selected-asteroid-source-request-corpus-summary.txt",
            "selected asteroid source request corpus summary checksum (fnv1a-64):",
            "Selected asteroid source request corpus:",
            "Tampered selected asteroid source request corpus:",
            "selected asteroid source request corpus summary no longer matches the current selected asteroid source request corpus posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_selected_asteroid_source_request_corpus_equatorial_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-selected-asteroid-source-request-corpus-equatorial-semantic",
            "selected-asteroid-source-request-corpus-equatorial-summary.txt",
            "selected asteroid source request corpus equatorial summary checksum (fnv1a-64):",
            "Selected asteroid source request corpus:",
            "Tampered selected asteroid source request corpus:",
            "selected asteroid source request corpus equatorial summary no longer matches the current selected asteroid source request corpus equatorial posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_selected_asteroid_source_window_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-selected-asteroid-source-window-semantic",
            "selected-asteroid-source-window-summary.txt",
            "selected asteroid source window summary checksum (fnv1a-64):",
            "Selected asteroid source windows:",
            "Tampered selected asteroid source windows:",
            "selected asteroid source window summary no longer matches the current selected asteroid source window posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_asteroid_source_window_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-asteroid-source-window-semantic",
            "reference-asteroid-source-window-summary.txt",
            "reference asteroid source window summary checksum (fnv1a-64):",
            "Reference asteroid source windows:",
            "Tampered reference asteroid source windows:",
            "reference asteroid source window summary no longer matches the current reference asteroid source-window posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_reference_asteroid_equatorial_evidence_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-reference-asteroid-equatorial-evidence-semantic",
            "reference-asteroid-equatorial-evidence-summary.txt",
            "reference asteroid equatorial evidence summary checksum (fnv1a-64):",
            "Selected asteroid equatorial evidence:",
            "Tampered selected asteroid equatorial evidence (drifted):",
            "reference asteroid equatorial evidence summary no longer matches the current reference asteroid equatorial evidence posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_independent_holdout_source_window_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-independent-holdout-source-window-semantic",
            "independent-holdout-source-window-summary.txt",
            "independent-holdout source window summary checksum (fnv1a-64):",
            "source-backed samples",
            "tampered source-backed samples",
            "independent-holdout source window summary no longer matches the current independent-holdout source-window posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_independent_holdout_body_class_coverage_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-independent-holdout-body-class-coverage-semantic",
            "independent-holdout-body-class-coverage-summary.txt",
            "independent-holdout body-class coverage summary checksum (fnv1a-64):",
            "Independent hold-out body-class coverage:",
            "Tampered independent hold-out body-class coverage:",
            "independent-holdout body-class coverage summary no longer matches the current independent-holdout body-class coverage posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_independent_holdout_equatorial_parity_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-independent-holdout-equatorial-parity-semantic",
            "independent-holdout-equatorial-parity-summary.txt",
            "independent-holdout equatorial parity summary checksum (fnv1a-64):",
            "JPL independent hold-out equatorial parity:",
            "JPL independent hold-out equatorial parity (drifted):",
            "independent-holdout equatorial parity summary no longer matches the current independent-holdout equatorial parity posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_benchmark_corpus_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-benchmark-corpus-semantic",
        "benchmark-corpus-summary.txt",
        "benchmark-corpus summary checksum (fnv1a-64):",
        "Benchmark corpus summary",
        "Benchmark corpus summary (drifted)",
        "benchmark corpus summary no longer matches the current benchmark-corpus posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_chart_benchmark_corpus_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-chart-benchmark-corpus-semantic",
            "chart-benchmark-corpus-summary.txt",
            "chart-benchmark-corpus summary checksum (fnv1a-64):",
            "Chart benchmark corpus summary",
            "Chart benchmark corpus summary (drifted)",
            "chart benchmark corpus summary no longer matches the current chart-benchmark corpus posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_phase2_corpus_alignment_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-packaged-artifact-phase2-corpus-alignment-semantic",
        "packaged-artifact-phase2-corpus-alignment-summary.txt",
        "packaged-artifact phase-2 corpus alignment summary checksum (fnv1a-64):",
        "reference source=Reference snapshot source:",
        "reference source=Drifted snapshot source:",
        "packaged-artifact phase-2 corpus alignment summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_summary_even_with_updated_checksum()
{
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-production-generation-semantic",
        "production-generation-summary.txt",
        "production generation summary checksum (fnv1a-64):",
        "Production generation coverage:",
        "Production generation coverage: drifted",
        "production generation summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_body_class_coverage_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-production-generation-body-class-coverage-semantic",
        "production-generation-body-class-coverage-summary.txt",
        "production generation body-class coverage summary checksum (fnv1a-64):",
        "Production generation body-class coverage:",
        "Production generation body-class coverage: drifted",
        "production generation body-class coverage summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_source_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-production-generation-source-semantic",
        "production-generation-source-summary.txt",
        "production generation source summary checksum (fnv1a-64):",
        "generation command=generate-packaged-artifact --check",
        "generation command=generate-packaged-artifact --check (drifted)",
        "production generation source summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_source_window_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-source-window-semantic",
            "production-generation-source-window-summary.txt",
            "production generation source window summary checksum (fnv1a-64):",
            "Production generation source windows:",
            "Production generation source windows (tampered):",
            "production generation source window summary no longer matches the current production-generation source-window posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_quarter_day_boundary_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-quarter-day-boundary-semantic",
            "production-generation-quarter-day-boundary-summary.txt",
            "production generation quarter-day boundary summary checksum (fnv1a-64):",
            "Production generation quarter-day boundary samples:",
            "Production generation quarter-day boundary samples (tampered):",
            "production generation quarter-day boundary summary no longer matches the current production-generation quarter-day boundary posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_manifest_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-production-generation-manifest-semantic",
        "production-generation-manifest-summary.txt",
        "production generation manifest summary checksum (fnv1a-64):",
        "Production generation manifest:",
        "Tampered production generation manifest:",
        "production generation manifest summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_production_generation_manifest_checksum_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-production-generation-manifest-checksum-semantic",
        "production-generation-manifest-checksum-summary.txt",
        "production generation manifest checksum summary checksum (fnv1a-64):",
        "Production generation manifest checksum:",
        "Tampered production generation manifest checksum:",
        "production generation manifest checksum summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_release_house_system_canonical_names_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-release-house-system-canonical-names-semantic",
        "release-house-system-canonical-names-summary.txt",
        "release-house-system-canonical-names summary checksum (fnv1a-64):",
        "Release-specific house-system canonical names: ",
        "Release-specific house-system canonical names (drifted): ",
        "release-house-system canonical names summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_release_ayanamsa_canonical_names_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-release-ayanamsa-canonical-names-semantic",
        "release-ayanamsa-canonical-names-summary.txt",
        "release-ayanamsa-canonical-names summary checksum (fnv1a-64):",
        "Release-specific ayanamsa canonical names: ",
        "Release-specific ayanamsa canonical names (drifted): ",
        "release-ayanamsa canonical names summary no longer matches",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_body_cadence_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-packaged-artifact-body-cadence-semantic",
            "packaged-artifact-body-cadence-summary.txt",
            "packaged-artifact body cadence summary checksum (fnv1a-64):",
            "body cadence: ",
            "body cadence: drifted ",
            "packaged-artifact body cadence summary no longer matches the current packaged-artifact body cadence posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_body_class_span_cap_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-packaged-artifact-body-class-span-cap-semantic",
            "packaged-artifact-body-class-span-cap-summary.txt",
            "packaged-artifact body-class span cap summary checksum (fnv1a-64):",
            "Packaged-artifact body-class span caps: ",
            "Packaged-artifact body-class span caps: drifted ",
            "packaged-artifact body-class span cap summary no longer matches the current packaged-artifact body-class span cap posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_lunar_theory_source_family_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-lunar-theory-source-family-semantic",
            "lunar-theory-source-family-summary.txt",
            "lunar theory source family summary checksum (fnv1a-64):",
            "selected model=",
            "selected model=drifted-",
            "lunar theory source family summary no longer matches the current lunar source-family posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_lunar_source_window_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-lunar-source-window-semantic",
        "lunar-source-window-summary.txt",
        "lunar theory source window summary checksum (fnv1a-64):",
        "exact Moon samples across",
        "drifted exact Moon samples across",
        "lunar source window summary no longer matches the current lunar source-window posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_lunar_theory_source_selection_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-lunar-theory-source-selection-semantic",
            "lunar-theory-source-selection-summary.txt",
            "lunar theory source selection summary checksum (fnv1a-64):",
            "selected key: source identifier=meeus-style-truncated-lunar-baseline",
            "selected key: source identifier=drifted-meeus-style-truncated-lunar-baseline",
            "lunar theory source selection summary no longer matches the current lunar source-selection posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_lunar_theory_limitations_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-lunar-theory-limitations-semantic",
            "lunar-theory-limitations-summary.txt",
            "lunar theory limitations summary checksum (fnv1a-64):",
            "Compact Meeus-style truncated lunar baseline",
            "Drifted Meeus-style truncated lunar baseline",
            "lunar theory limitations summary no longer matches the current lunar-theory limitations posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_lunar_theory_catalog_validation_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-lunar-theory-catalog-validation-semantic",
            "lunar-theory-catalog-validation-summary.txt",
            "lunar theory catalog validation summary checksum (fnv1a-64):",
            "aliases=1",
            "aliases=2",
            "lunar theory catalog validation summary no longer matches the current lunar theory catalog posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_native_sidereal_policy_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-native-sidereal-policy-semantic",
            "native-sidereal-policy-summary.txt",
            "native sidereal policy summary checksum (fnv1a-64):",
            "native sidereal backend output remains unsupported unless a backend explicitly advertises it",
            "native sidereal backend output is now advertised by default",
            "native sidereal policy summary no longer matches the current native-sidereal posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_time_scale_policy_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-time-scale-policy-semantic",
        "time-scale-policy-summary.txt",
        "time-scale policy summary checksum (fnv1a-64):",
        "Time-scale policy: ",
        "Time-scale policy: drifted ",
        "time-scale policy summary no longer matches the current time-scale posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_utc_convenience_policy_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-utc-convenience-policy-semantic",
        "utc-convenience-policy-summary.txt",
        "utc-convenience policy summary checksum (fnv1a-64):",
        "UTC convenience policy: ",
        "UTC convenience policy: drifted ",
        "UTC convenience policy summary no longer matches the current UTC-convenience posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_delta_t_policy_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-delta-t-policy-semantic",
        "delta-t-policy-summary.txt",
        "delta-t policy summary checksum (fnv1a-64):",
        "Delta T policy: ",
        "Delta T policy: drifted ",
        "delta-t policy summary no longer matches the current delta-t posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_native_dependency_audit_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-native-dependency-audit-semantic",
        "native-dependency-audit-summary.txt",
        "native-dependency audit summary checksum (fnv1a-64):",
        "Result: no workspace policy violations detected",
        "Result: drifted native build hooks detected",
        "native-dependency audit summary no longer matches the workspace audit summary",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_target_threshold_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-packaged-artifact-target-threshold-semantic",
            "packaged-artifact-target-threshold-summary.txt",
            "packaged-artifact target-threshold summary checksum (fnv1a-64):",
            "production thresholds recorded",
            "production thresholds drifting",
            "packaged-artifact target-threshold summary no longer matches the current packaged-artifact target-threshold posture",
        );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_profile_alignment_helpers_accept_matching_release_text() {
    let release_profiles = current_release_profile_identifiers();
    let profile_id = release_profiles.compatibility_profile_id;
    let api_stability_posture_id = release_profiles.api_stability_profile_id;
    let release_notes_summary = format!(
            "Release notes summary\nProfile: {profile_id}\nAPI stability posture: {api_stability_posture_id}\n"
        );
    let release_summary = format!(
            "Release summary\nProfile: {profile_id}\nAPI stability posture: {api_stability_posture_id}\n"
        );
    let release_checklist = format!(
            "Release checklist\nProfile: {profile_id}\nAPI stability posture: {api_stability_posture_id}\n"
        );
    let release_profile_identifiers = format!(
            "Release profile identifiers: v1 compatibility={profile_id}, api-stability={api_stability_posture_id}\n"
        );
    let catalog_inventory = current_compatibility_profile()
        .validated_catalog_inventory_summary_line()
        .expect("catalog inventory summary should validate");
    let custom_definition_ayanamsa_labels = current_compatibility_profile()
        .validated_custom_definition_ayanamsa_labels_summary_line()
        .expect("custom-definition ayanamsa labels summary should validate");

    ensure_release_profile_line_alignment(
        "release notes",
        &format!("Release notes\nProfile: {profile_id}\n"),
        profile_id,
    )
    .expect("matching release notes should verify");
    ensure_release_profile_summary_alignment(
        "release notes summary",
        &release_notes_summary,
        profile_id,
        api_stability_posture_id,
    )
    .expect("matching release notes summary should verify");
    ensure_release_notes_summary_matches_current_rendering(&render_release_notes_summary_text())
        .expect("matching release notes summary should match the current rendering");
    ensure_release_profile_summary_alignment(
        "release summary",
        &release_summary,
        profile_id,
        api_stability_posture_id,
    )
    .expect("matching release summary should verify");
    ensure_release_profile_summary_alignment(
        "release checklist",
        &release_checklist,
        profile_id,
        api_stability_posture_id,
    )
    .expect("matching release checklist should verify");
    ensure_release_profile_identifiers_alignment(
        &release_profile_identifiers,
        profile_id,
        api_stability_posture_id,
    )
    .expect("matching release profile identifiers should verify");
    ensure_release_profile_identifiers_summary_matches_current_rendering(
        &render_release_profile_identifiers_summary(),
    )
    .expect("matching release-profile identifiers summary should verify");
    ensure_catalog_inventory_alignment(&catalog_inventory)
        .expect("matching catalog inventory should verify");
    ensure_custom_definition_ayanamsa_labels_alignment(&custom_definition_ayanamsa_labels)
        .expect("matching custom-definition labels should verify");
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_profile_alignment_helpers_reject_mismatched_release_text() {
    let release_profiles = current_release_profile_identifiers();
    let profile_id = release_profiles.compatibility_profile_id;
    let api_stability_posture_id = release_profiles.api_stability_profile_id;

    let error = ensure_release_notes_summary_matches_current_rendering(
        "Release notes summary\nProfile: incorrect-profile\nAPI stability posture: incorrect-api\n",
    )
    .expect_err("mismatched release notes summary should fail");
    assert!(error
        .to_string()
        .contains("release notes summary no longer matches"));

    let error = ensure_release_profile_summary_alignment(
        "release summary",
        "Release summary\nProfile: incorrect-profile\nAPI stability posture: incorrect-api\n",
        profile_id,
        api_stability_posture_id,
    )
    .expect_err("mismatched release summary should fail");
    assert!(error
        .to_string()
        .contains("release bundle verification failed"));
    assert!(error
        .to_string()
        .contains("release summary profile id mismatch"));

    let error = ensure_release_profile_identifiers_alignment(
            "Release profile identifiers: v1 compatibility=incorrect-profile, api-stability=incorrect-api\n",
            profile_id,
            api_stability_posture_id,
        )
        .expect_err("mismatched release-profile identifiers should fail");
    assert!(error
        .to_string()
        .contains("release-profile identifiers mismatch"));

    let error = ensure_release_profile_identifiers_summary_matches_current_rendering(
        "Release profile identifiers summary\nRelease profile identifiers: tampered\n",
    )
    .expect_err("mismatched release-profile identifiers summary should fail");
    assert!(error
        .to_string()
        .contains("release-profile identifiers summary no longer matches"));

    let error = ensure_catalog_inventory_alignment("Compatibility catalog inventory: tampered")
        .expect_err("mismatched catalog inventory should fail");
    assert!(error
        .to_string()
        .contains("catalog inventory summary mismatch"));

    let error = ensure_custom_definition_ayanamsa_labels_alignment(
        "Babylonian (House), Babylonian (Sissy)",
    )
    .expect_err("mismatched custom-definition labels should fail");
    assert!(error
        .to_string()
        .contains("custom-definition ayanamsa labels summary mismatch"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_artifact_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-artifact-summary",
        "artifact-summary.txt",
        "artifact summary checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_api_stability_summary_file() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-api-summary");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let summary_path = bundle_dir.join("api-stability-summary.txt");
    let summary =
        std::fs::read_to_string(&summary_path).expect("API stability summary should exist");
    let tampered = summary.replace("API stability summary", "Tampered API stability summary");
    std::fs::write(&summary_path, tampered).expect("API stability summary should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered API stability summary");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("API stability summary checksum mismatch")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_catalog_inventory_summary_even_with_updated_checksum() {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-catalog-inventory-semantic",
        "catalog-inventory-summary.txt",
        "catalog inventory summary checksum (fnv1a-64):",
        "Compatibility catalog inventory:",
        "Tampered compatibility catalog inventory:",
        "catalog inventory summary no longer matches the current catalog inventory posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_compatibility_profile_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-profile",
        "compatibility-profile.txt",
        "compatibility profile checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_release_checklist_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-checklist",
        "release-checklist.txt",
        "release checklist checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_release_checklist_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-checklist-summary",
        "release-checklist-summary.txt",
        "release checklist summary checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_backend_matrix_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-matrix",
        "backend-matrix.txt",
        "backend matrix checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_api_stability_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-api-stability",
        "api-stability.txt",
        "API stability checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-validation-report-summary",
        "validation-report-summary.txt",
        "validation report summary checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-validation-report",
        "validation-report.txt",
        "validation report checksum mismatch",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_file_even_with_updated_checksum() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-validation-report-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let report_path = bundle_dir.join("validation-report.txt");
    let report = std::fs::read_to_string(&report_path).expect("validation report should exist");
    let tampered_report = report.replace(
        "Packaged artifact decode benchmark",
        "Tampered packaged artifact decode benchmark",
    );
    std::fs::write(&report_path, &tampered_report).expect("validation report should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("validation report checksum (fnv1a-64):"))
        .expect("manifest should contain the validation report checksum line");
    let new_checksum_line = format!(
        "validation report checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_report)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic validation-report drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(
        error.contains("validation report no longer matches the current validation report posture")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_summary_file_even_with_updated_checksum(
) {
    let bundle_dir =
        unique_temp_dir("pleiades-release-bundle-tampered-validation-report-summary-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let report_path = bundle_dir.join("validation-report-summary.txt");
    let report =
        std::fs::read_to_string(&report_path).expect("validation report summary should exist");
    let tampered_report = report.replace(
        "Packaged-artifact fit envelope:",
        "Tampered packaged-artifact fit envelope:",
    );
    std::fs::write(&report_path, &tampered_report)
        .expect("validation report summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("validation report summary checksum (fnv1a-64):"))
        .expect("manifest should contain the validation report summary checksum line");
    let new_checksum_line = format!(
        "validation report summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_report)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic validation-report-summary drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("validation report summary no longer matches"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_summary_header_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
        "pleiades-release-bundle-tampered-validation-report-summary-header-semantic",
        "validation-report-summary.txt",
        "validation report summary checksum (fnv1a-64):",
        "Validation report summary",
        "Tampered validation report summary",
        "validation report summary no longer matches the current validation report posture",
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_fit_margin_summary_even_with_updated_checksum(
) {
    let bundle_dir =
        unique_temp_dir("pleiades-release-bundle-tampered-validation-report-fit-margin-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let report_path = bundle_dir.join("validation-report-summary.txt");
    let report =
        std::fs::read_to_string(&report_path).expect("validation report summary should exist");
    let tampered_report = report.replace(
        "  Packaged-artifact fit margins: mean Δlon=",
        "  Packaged-artifact fit margins: drifted mean Δlon=",
    );
    std::fs::write(&report_path, &tampered_report)
        .expect("validation report summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("validation report summary checksum (fnv1a-64):"))
        .expect("manifest should contain the validation report summary checksum line");
    let new_checksum_line = format!(
        "validation report summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_report)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic validation-report fit-margin drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("validation report summary no longer matches the current packaged-artifact fit margins posture"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_fit_outlier_summary_even_with_updated_checksum(
) {
    let bundle_dir =
        unique_temp_dir("pleiades-release-bundle-tampered-validation-report-fit-outlier-semantic");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let report_path = bundle_dir.join("validation-report-summary.txt");
    let report =
        std::fs::read_to_string(&report_path).expect("validation report summary should exist");
    let tampered_report = report.replace(
        "  Packaged-artifact fit outliers: fit outliers:",
        "  Packaged-artifact fit outliers: drifted fit outliers:",
    );
    std::fs::write(&report_path, &tampered_report)
        .expect("validation report summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("validation report summary checksum (fnv1a-64):"))
        .expect("manifest should contain the validation report summary checksum line");
    let new_checksum_line = format!(
        "validation report summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_report)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic validation-report fit-outlier drift");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("validation report summary no longer matches the current packaged-artifact fit outliers posture"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_fit_sample_classes_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir(
        "pleiades-release-bundle-tampered-validation-report-fit-sample-classes-semantic",
    );
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let report_path = bundle_dir.join("validation-report-summary.txt");
    let report =
        std::fs::read_to_string(&report_path).expect("validation report summary should exist");
    let tampered_report = report.replace(
        "  Packaged-artifact fit sample classes: fit sample classes:",
        "  Packaged-artifact fit sample classes: drifted fit sample classes:",
    );
    std::fs::write(&report_path, &tampered_report)
        .expect("validation report summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("validation report summary checksum (fnv1a-64):"))
        .expect("manifest should contain the validation report summary checksum line");
    let new_checksum_line = format!(
        "validation report summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_report)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
        "verification should fail for semantic validation-report fit-sample-classes drift",
    );
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("validation report summary no longer matches the current packaged-artifact fit sample classes posture"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_fit_threshold_violation_count_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir(
        "pleiades-release-bundle-tampered-validation-report-fit-threshold-violation-count-semantic",
    );
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let report_path = bundle_dir.join("validation-report-summary.txt");
    let report =
        std::fs::read_to_string(&report_path).expect("validation report summary should exist");
    let tampered_report = report.replace(
        "  Packaged-artifact fit threshold violation count: 0",
        "  Packaged-artifact fit threshold violation count: 1",
    );
    std::fs::write(&report_path, &tampered_report)
        .expect("validation report summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("validation report summary checksum (fnv1a-64):"))
        .expect("manifest should contain the validation report summary checksum line");
    let new_checksum_line = format!(
        "validation report summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_report)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
            "verification should fail for semantic validation-report fit-threshold-violation-count drift",
        );
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("validation report summary no longer matches the current packaged-artifact fit threshold violation count posture"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_validation_report_fit_threshold_violations_summary_even_with_updated_checksum(
) {
    let bundle_dir = unique_temp_dir(
        "pleiades-release-bundle-tampered-validation-report-fit-threshold-violations-semantic",
    );
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let report_path = bundle_dir.join("validation-report-summary.txt");
    let report =
        std::fs::read_to_string(&report_path).expect("validation report summary should exist");
    let tampered_report = report.replace(
        "  Packaged-artifact fit threshold violations: 0; details: none",
        "  Packaged-artifact fit threshold violations: 1; details: tampered",
    );
    std::fs::write(&report_path, &tampered_report)
        .expect("validation report summary should be writable");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with("validation report summary checksum (fnv1a-64):"))
        .expect("manifest should contain the validation report summary checksum line");
    let new_checksum_line = format!(
        "validation report summary checksum (fnv1a-64): 0x{:016x}",
        checksum64(&tampered_report)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
        "verification should fail for semantic validation-report fit-threshold-violations drift",
    );
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("validation report summary no longer matches the current packaged-artifact fit threshold violations posture"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_release_checklist_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-checklist");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let corrupted = manifest.replace(
        "release checklist checksum (fnv1a-64):",
        "release checklist checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted release checklist checksum");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid release checklist checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_api_stability_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-api-stability");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let corrupted = manifest.replace(
        "api stability checksum (fnv1a-64):",
        "api stability checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted API stability checksum");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid api stability checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_validation_report_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-validation-report");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let corrupted = manifest.replace(
        "validation report checksum (fnv1a-64):",
        "validation report checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted validation report checksum");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid validation report checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_validation_report_summary_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-validation-report-summary");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let corrupted = manifest.replace(
        "validation report summary checksum (fnv1a-64):",
        "validation report summary checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted validation report summary checksum");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid validation report summary checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn verify_release_bundle_rejects_tampered_packaged_artifact_generation_manifest_checksum_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-packaged-artifact-generation-manifest-checksum-semantic",
            "packaged-artifact-generation-manifest-checksum-summary.txt",
            "packaged-artifact generation manifest checksum summary checksum (fnv1a-64):",
            "Packaged artifact generation manifest checksum:",
            "Tampered packaged artifact generation manifest checksum:",
            "packaged-artifact generation manifest checksum summary no longer matches the current packaged-artifact generation-manifest checksum posture",
        );
}
