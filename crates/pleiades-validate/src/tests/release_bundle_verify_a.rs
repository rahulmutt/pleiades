//! release bundle write/verify tests (part 1) (white-box; moved verbatim from the former `tests.rs`).

use super::test_support::*;
use super::*;
use pleiades_core::current_release_profile_identifiers;

#[test]
fn release_bundle_commands_accept_output_aliases_in_the_validation_front_end() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-output-alias-validation");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();

    render_cli(&[
        "bundle-release",
        "--output",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle-release should accept --output alias in the validation front end");
    let verified = render_cli(&["verify-release-bundle", "--output", &bundle_dir_string])
        .expect("verify-release-bundle should accept --output alias in the validation front end");

    assert!(verified.contains("Release bundle"));
    assert!(verified.contains("bundle-manifest.checksum.txt"));

    let mixed_bundle_output_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--output",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect_err("bundle-release should reject mixed output aliases in the validation front end");
    assert!(mixed_bundle_output_error.contains("duplicate value for --out <dir> argument"));

    let mixed_verify_output_error = render_cli(&[
        "verify-release-bundle",
        "--out",
        &bundle_dir_string,
        "--output",
        &bundle_dir_string,
    ])
    .expect_err(
        "verify-release-bundle should reject mixed output aliases in the validation front end",
    );
    assert!(mixed_verify_output_error.contains("duplicate value for --out <dir> argument"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_tampered_release_house_validation_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-release-house-validation-semantic",
            "release-house-validation-summary.txt",
            "release-house-validation summary checksum (fnv1a-64):",
            "House validation corpus:",
            "Tampered house validation corpus:",
            "release house validation summary no longer matches the current release-house-validation posture",
        );
}

#[test]
fn release_bundle_writes_expected_artifacts() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    let rendered = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    assert!(rendered.contains("Release bundle"));
    assert!(rendered.contains("compatibility-profile.txt"));
    assert!(rendered.contains("compatibility-profile-summary.txt"));
    assert!(rendered.contains("validation rounds: 1"));
    assert!(rendered.contains("release-notes.txt"));
    assert!(rendered.contains("cargo version:"));
    assert!(rendered.contains("release-notes-summary.txt"));
    assert!(rendered.contains("release-checklist.txt"));
    assert!(rendered.contains("backend-matrix.txt"));
    assert!(rendered.contains("API stability posture:"));
    assert!(rendered.contains("api-stability.txt"));
    assert!(rendered.contains("comparison-corpus-summary.txt"));
    assert!(rendered.contains("source-corpus-summary.txt"));
    assert!(rendered.contains("comparison-snapshot-summary.txt"));
    assert!(rendered.contains("comparison-snapshot-source-summary.txt"));
    assert!(rendered.contains("comparison-snapshot-body-class-coverage-summary.txt"));
    assert!(rendered.contains("comparison-snapshot-manifest-summary.txt"));
    assert!(rendered.contains("reference-snapshot-source-summary.txt"));
    assert!(rendered.contains("reference-snapshot-source-window-summary.txt"));
    assert!(rendered.contains("reference-snapshot-2451917-major-body-boundary-summary.txt"));
    assert!(rendered.contains("reference-snapshot-2451918-major-body-boundary-summary.txt"));
    assert!(rendered.contains("reference-snapshot-2451919-major-body-boundary-summary.txt"));
    assert!(rendered.contains("reference-snapshot-2451916-major-body-dense-boundary-summary.txt"));
    assert!(rendered.contains("reference-snapshot-manifest-summary.txt"));
    assert!(rendered.contains("reference-snapshot-body-class-coverage-summary.txt"));
    assert!(rendered.contains("reference-asteroid-source-window-summary.txt"));
    assert!(rendered.contains("reference-asteroid-equatorial-evidence-summary.txt"));
    assert!(rendered.contains("production-generation-source-window-summary.txt"));
    assert!(rendered.contains("production-generation-boundary-window-summary.txt"));
    assert!(rendered.contains("production-generation-quarter-day-boundary-summary.txt"));
    assert!(rendered.contains("reference-snapshot-summary.txt"));
    assert!(rendered.contains("validation-report-summary.txt"));
    assert!(rendered.contains("workspace-audit-summary.txt"));
    assert!(rendered.contains("benchmark-corpus-summary.txt"));
    assert!(bundle_dir
        .join("chart-benchmark-corpus-summary.txt")
        .exists());
    assert!(bundle_dir.join("source-corpus-summary.txt").exists());
    assert!(bundle_dir.join("jpl-source-posture-summary.txt").exists());
    assert!(bundle_dir
        .join("reference-snapshot-manifest-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-snapshot-2451917-major-body-boundary-summary.txt")
        .exists());
    assert!(bundle_dir.join("jpl-provenance-only-summary.txt").exists());
    assert!(bundle_dir
        .join("comparison-snapshot-manifest-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-snapshot-2451918-major-body-boundary-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-snapshot-2451919-major-body-boundary-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-snapshot-2451916-major-body-dense-boundary-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("production-generation-boundary-request-corpus-equatorial-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("production-generation-quarter-day-boundary-summary.txt")
        .exists());
    assert!(rendered.contains("request-policy-summary.txt"));
    assert!(rendered.contains("observer-policy-summary.txt"));
    assert!(rendered.contains("apparentness-policy-summary.txt"));
    assert!(rendered.contains("time-scale-policy-summary.txt"));
    assert!(rendered.contains("delta-t-policy-summary.txt"));
    assert!(rendered.contains("native-sidereal-policy-summary.txt"));
    assert!(rendered.contains("lunar-theory-limitations-summary.txt"));
    assert!(rendered.contains("lunar-theory-source-selection-summary.txt"));
    assert!(rendered.contains("lunar-theory-source-family-summary.txt"));
    assert!(rendered.contains("lunar-source-window-summary.txt"));
    assert!(rendered.contains("lunar-theory-catalog-validation-summary.txt"));
    assert!(rendered.contains("release-house-validation-summary.txt"));
    assert!(bundle_dir
        .join("compatibility-caveats-summary.txt")
        .exists());
    assert!(bundle_dir.join("catalog-inventory-summary.txt").exists());
    assert!(bundle_dir.join("catalog-posture-summary.txt").exists());
    assert!(bundle_dir
        .join("comparison-corpus-guard-summary.txt")
        .exists());
    assert!(bundle_dir.join("comparison-snapshot-summary.txt").exists());
    assert!(bundle_dir
        .join("comparison-snapshot-source-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("comparison-snapshot-source-window-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("custom-definition-ayanamsa-labels-summary.txt")
        .exists());
    assert!(bundle_dir.join("ayanamsa-provenance-summary.txt").exists());
    assert!(bundle_dir
        .join("reference-snapshot-source-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-snapshot-body-class-coverage-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-snapshot-exact-j2000-evidence-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-asteroid-source-window-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("reference-asteroid-equatorial-evidence-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("production-generation-boundary-source-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("production-generation-boundary-window-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("production-generation-body-class-coverage-summary.txt")
        .exists());
    assert!(bundle_dir.join("reference-snapshot-summary.txt").exists());
    assert!(bundle_dir.join("request-policy-summary.txt").exists());
    assert!(bundle_dir.join("observer-policy-summary.txt").exists());
    assert!(bundle_dir.join("apparentness-policy-summary.txt").exists());
    assert!(bundle_dir.join("request-semantics-summary.txt").exists());
    assert!(bundle_dir.join("unsupported-modes-summary.txt").exists());
    assert!(bundle_dir.join("release-body-claims-summary.txt").exists());
    assert!(bundle_dir
        .join("body-date-channel-claims-summary.txt")
        .exists());
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
        .join("lunar-theory-limitations-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("lunar-theory-source-selection-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("lunar-theory-source-family-summary.txt")
        .exists());
    assert!(bundle_dir.join("lunar-source-window-summary.txt").exists());
    assert!(bundle_dir
        .join("lunar-theory-catalog-validation-summary.txt")
        .exists());
    assert!(bundle_dir.join("request-surface-summary.txt").exists());
    assert!(bundle_dir
        .join("release-house-system-canonical-names-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("release-ayanamsa-canonical-names-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("release-house-validation-summary.txt")
        .exists());
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
        .join("packaged-artifact-output-support-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-speed-policy-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-storage-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-production-profile-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-target-threshold-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-source-fit-holdout-sync-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-body-cadence-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-body-class-span-cap-summary.txt")
        .exists());
    assert!(rendered.contains("artifact-summary.txt"));
    assert!(rendered.contains("packaged-artifact-profile-coverage-summary.txt"));
    assert!(bundle_dir
        .join("packaged-artifact-generation-policy-summary.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-generation-manifest.txt")
        .exists());
    assert!(bundle_dir
        .join("packaged-artifact-generation-manifest.checksum.txt")
        .exists());
    assert!(rendered.contains("benchmark-report.txt"));
    assert!(rendered.contains("validation-report.txt"));
    assert!(rendered.contains("workspace-provenance-summary.txt"));
    assert!(rendered.contains("release-profile-identifiers.txt"));
    assert!(rendered.contains("release-profile-identifiers-summary.txt"));
    assert!(rendered.contains("bundle-manifest.checksum.txt"));
    assert!(rendered.contains("source revision:"));
    assert!(rendered.contains("workspace status:"));
    assert!(rendered.contains("rustc version:"));
    assert!(rendered.contains("checksum: 0x"));

    let profile = std::fs::read_to_string(bundle_dir.join("compatibility-profile.txt"))
        .expect("compatibility profile should be written");
    let profile_summary =
        std::fs::read_to_string(bundle_dir.join("compatibility-profile-summary.txt"))
            .expect("compatibility profile summary should be written");
    let release_notes = std::fs::read_to_string(bundle_dir.join("release-notes.txt"))
        .expect("release notes should be written");
    let release_notes_summary =
        std::fs::read_to_string(bundle_dir.join("release-notes-summary.txt"))
            .expect("release notes summary should be written");
    let release_summary = std::fs::read_to_string(bundle_dir.join("release-summary.txt"))
        .expect("release summary should be written");
    let release_profile_identifiers =
        std::fs::read_to_string(bundle_dir.join("release-profile-identifiers.txt"))
            .expect("release-profile identifiers should be written");
    let release_profile_identifiers_summary =
        std::fs::read_to_string(bundle_dir.join("release-profile-identifiers-summary.txt"))
            .expect("release-profile identifiers summary should be written");
    let request_semantics_summary =
        std::fs::read_to_string(bundle_dir.join("request-semantics-summary.txt"))
            .expect("request semantics summary should be written");
    let release_checklist = std::fs::read_to_string(bundle_dir.join("release-checklist.txt"))
        .expect("release checklist should be written");
    let release_checklist_summary =
        std::fs::read_to_string(bundle_dir.join("release-checklist-summary.txt"))
            .expect("release checklist summary should be written");
    let backend_matrix = std::fs::read_to_string(bundle_dir.join("backend-matrix.txt"))
        .expect("backend matrix should be written");
    let backend_matrix_summary =
        std::fs::read_to_string(bundle_dir.join("backend-matrix-summary.txt"))
            .expect("backend matrix summary should be written");
    let api_stability = std::fs::read_to_string(bundle_dir.join("api-stability.txt"))
        .expect("API stability posture should be written");
    let api_stability_summary =
        std::fs::read_to_string(bundle_dir.join("api-stability-summary.txt"))
            .expect("API stability summary should be written");
    let comparison_corpus_summary =
        std::fs::read_to_string(bundle_dir.join("comparison-corpus-summary.txt"))
            .expect("comparison corpus summary should be written");
    let comparison_snapshot_summary =
        std::fs::read_to_string(bundle_dir.join("comparison-snapshot-summary.txt"))
            .expect("comparison snapshot summary should be written");
    let benchmark_corpus_summary =
        std::fs::read_to_string(bundle_dir.join("benchmark-corpus-summary.txt"))
            .expect("benchmark corpus summary should be written");
    let chart_benchmark_corpus_summary =
        std::fs::read_to_string(bundle_dir.join("chart-benchmark-corpus-summary.txt"))
            .expect("chart benchmark corpus summary should be written");
    assert_eq!(
        chart_benchmark_corpus_summary,
        render_chart_benchmark_corpus_summary_text()
    );
    let workspace_provenance_summary_text =
        std::fs::read_to_string(bundle_dir.join("workspace-provenance-summary.txt"))
            .expect("workspace provenance summary should be written");
    assert_eq!(
        workspace_provenance_summary_text,
        workspace_provenance_summary_for_report()
    );
    let reference_snapshot_summary =
        std::fs::read_to_string(bundle_dir.join("reference-snapshot-summary.txt"))
            .expect("reference snapshot summary should be written");
    let production_generation_body_class_coverage_summary = std::fs::read_to_string(
        bundle_dir.join("production-generation-body-class-coverage-summary.txt"),
    )
    .expect("production generation body-class coverage summary should be written");
    let lunar_theory_limitations_summary =
        std::fs::read_to_string(bundle_dir.join("lunar-theory-limitations-summary.txt"))
            .expect("lunar theory limitations summary should be written");
    let validation_report_summary =
        std::fs::read_to_string(bundle_dir.join("validation-report-summary.txt"))
            .expect("validation report summary should be written");
    let workspace_audit_summary =
        std::fs::read_to_string(bundle_dir.join("workspace-audit-summary.txt"))
            .expect("workspace audit summary should be written");
    let artifact_summary = std::fs::read_to_string(bundle_dir.join("artifact-summary.txt"))
        .expect("artifact summary should be written");
    let packaged_artifact_bytes = std::fs::read(bundle_dir.join("packaged-artifact.bin"))
        .expect("packaged-artifact binary should be written");
    let packaged_artifact_checksum_sidecar =
        std::fs::read_to_string(bundle_dir.join("packaged-artifact.checksum.txt"))
            .expect("packaged-artifact checksum sidecar should be written");
    assert_eq!(
        packaged_artifact_bytes,
        pleiades_data::packaged_artifact_bytes()
    );
    assert_eq!(
        packaged_artifact_checksum_sidecar.trim(),
        format!("0x{:016x}", packaged_artifact().checksum)
    );
    let packaged_artifact_profile_coverage_summary =
        std::fs::read_to_string(bundle_dir.join("packaged-artifact-profile-coverage-summary.txt"))
            .expect("packaged-artifact profile coverage summary should be written");
    assert_eq!(
        packaged_artifact_profile_coverage_summary,
        packaged_artifact_profile_coverage_summary_for_report()
    );
    let packaged_artifact_generation_manifest =
        std::fs::read_to_string(bundle_dir.join("packaged-artifact-generation-manifest.txt"))
            .expect("packaged artifact generation manifest should be written");
    let packaged_artifact_generation_manifest_checksum_sidecar = std::fs::read_to_string(
        bundle_dir.join("packaged-artifact-generation-manifest.checksum.txt"),
    )
    .expect("packaged artifact generation manifest checksum sidecar should be written");
    assert_eq!(
        packaged_artifact_generation_manifest,
        packaged_artifact_generation_manifest_for_report()
    );
    assert_eq!(
        packaged_artifact_generation_manifest_checksum_sidecar.trim(),
        format!(
            "0x{:016x}",
            checksum64(&packaged_artifact_generation_manifest)
        )
    );
    let compatibility_profile = current_compatibility_profile();
    let house_code_aliases_summary = compatibility_profile.house_code_aliases_summary_line();
    let benchmark_report = std::fs::read_to_string(bundle_dir.join("benchmark-report.txt"))
        .expect("benchmark report should be written");
    let report = std::fs::read_to_string(bundle_dir.join("validation-report.txt"))
        .expect("validation report should be written");
    assert!(benchmark_report.contains("Benchmark report"));
    assert_eq!(
        lunar_theory_limitations_summary,
        lunar_theory_limitations_summary_for_report()
    );
    assert_eq!(
        comparison_snapshot_summary,
        comparison_snapshot_summary_for_report()
    );
    assert_eq!(
        reference_snapshot_summary,
        reference_snapshot_summary_for_report()
    );
    assert_eq!(
        production_generation_body_class_coverage_summary,
        production_generation_snapshot_body_class_coverage_summary_for_report()
    );
    assert_eq!(
        request_semantics_summary,
        render_request_semantics_summary_text()
    );
    let manifest = std::fs::read_to_string(bundle_dir.join("bundle-manifest.txt"))
        .expect("manifest should be written");
    let manifest_checksum =
        std::fs::read_to_string(bundle_dir.join("bundle-manifest.checksum.txt"))
            .expect("manifest checksum sidecar should be written");

    let release_profiles = current_release_profile_identifiers();
    assert!(profile.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(profile_summary.contains(&format!(
        "Compatibility profile summary\nProfile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(profile_summary.contains(
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
    assert!(release_notes.contains("Release notes"));
    assert!(release_notes.contains("Release notes summary: release-notes-summary"));
    assert!(release_notes.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(release_notes.contains("Artifact validation: validate-artifact"));
    assert!(
        release_notes.contains("Compatibility profile verification: verify-compatibility-profile")
    );
    assert!(release_notes.contains("Release summary: release-summary"));
    assert!(release_notes.contains("API stability posture:"));
    assert!(release_notes.contains("Deprecation policy:"));
    assert!(release_notes.contains("Release-specific coverage:"));
    assert!(release_notes.contains("selected asteroid coverage"));
    assert!(release_notes.contains("Selected asteroid evidence: 6 exact J2000 samples"));
    assert!(release_notes.contains("Selected asteroid batch parity: 6 requests across 6 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); frame mix: 3 ecliptic, 3 equatorial; batch/single parity preserved"));
    assert!(release_notes.contains("Selected asteroid equatorial evidence: 6 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) using a mean-obliquity equatorial transform"));
    assert!(release_notes.contains("asteroid:433-Eros"));
    assert!(release_notes.contains("Validation reference points:"));
    assert!(release_notes.contains("Compatibility caveats:"));
    assert!(release_notes.contains("Bundle provenance:"));
    assert!(release_notes.contains("Rust compiler version"));
    assert!(release_notes_summary.contains("Release notes summary"));
    assert!(release_notes_summary.contains(
            "Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="
        ));
    assert!(release_notes_summary.contains("Release-specific coverage:"));
    assert!(release_notes_summary
        .contains(&reference_snapshot_2268932_selected_body_boundary_summary_for_report()));
    assert!(release_notes_summary
        .contains(&reference_snapshot_2305457_selected_body_boundary_summary_for_report()));
    assert!(release_notes_summary
        .contains(&reference_snapshot_2360233_major_body_boundary_summary_for_report()));
    assert!(release_notes_summary
        .contains(&reference_snapshot_2378499_major_body_boundary_summary_for_report()));
    assert!(release_notes_summary
        .contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(release_notes_summary
        .contains(&reference_snapshot_2451916_major_body_boundary_summary_for_report()));
    assert!(release_notes_summary.lines().any(|line| {
        line == reference_snapshot_2451916_major_body_interior_summary_for_report()
    }));
    assert!(release_notes_summary.lines().any(|line| {
        line == reference_snapshot_2451916_major_body_boundary_summary_for_report()
    }));
    assert!(release_notes_summary.contains(&reference_snapshot_bridge_day_summary_for_report()));
    assert!(release_notes_summary
        .contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report()));
    assert!(release_notes_summary.contains(
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
    assert!(release_notes_summary.contains("API stability summary line: API stability posture:"));
    assert!(release_notes_summary.contains("Artifact validation: validate-artifact"));
    assert!(release_notes_summary.lines().any(|line| {
            line == "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
        }));
    assert!(release_notes_summary.lines().any(|line| {
            line == "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        }));
    assert!(release_notes_summary.lines().any(|line| {
            line == "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        }));
    assert!(release_notes_summary.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(release_notes_summary.contains("Release notes: release-notes"));
    assert!(release_notes_summary.contains("Compatibility caveats: 2"));
    assert!(release_notes_summary
        .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
    assert!(release_notes_summary
        .contains("Artifact summary: artifact-summary / artifact-posture-summary"));
    assert!(release_notes_summary.contains("Packaged-artifact storage/reconstruction: Quantized linear segments stored in pleiades-compression artifact format; body-indexed segment tables support random access by body and lookup time across the advertised range; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, sidereal, and motion outputs remain unsupported"));
    assert_report_contains_exact_line(
        &release_notes_summary,
        &format!(
            "Packaged-artifact generation policy: {}",
            packaged_artifact_generation_policy_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &release_notes_summary,
        &format!(
            "Packaged-artifact normalized intermediates: {}",
            packaged_artifact_normalized_intermediate_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &release_notes_summary,
        &format!(
            "Packaged-artifact generation residual bodies: {}",
            validated_packaged_artifact_generation_residual_bodies_summary_for_report()
                .expect("packaged artifact residual bodies summary should validate")
        ),
    );
    assert!(release_notes_summary
        .contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(release_notes_summary.contains("Workspace audit summary: workspace-audit-summary"));
    assert!(release_notes_summary.contains(&format!(
        "House code aliases: {}",
        house_code_aliases_summary
    )));
    assert!(release_summary.contains("Release summary"));
    assert!(release_summary.contains("API stability summary line: API stability posture:"));
    assert!(release_summary
        .contains("Production generation source: strategy=documented hybrid fixture corpus"));
    assert!(release_summary.contains(
        "Production generation source revision: source revision=reference_snapshot.csv checksum=0x"
    ));
    assert!(release_summary.contains(
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
    assert!(release_summary.contains(&format!(
        "House-code aliases: {}",
        current_compatibility_profile().house_code_alias_count()
    )));
    assert!(release_summary.contains(&format!(
        "House code aliases: {}",
        house_code_aliases_summary
    )));
    assert!(
        release_summary.contains("Compatibility profile summary: compatibility-profile-summary")
    );
    assert!(release_summary.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(release_summary.contains("JPL interpolation posture: source="));
    assert!(release_summary
        .contains(&reference_snapshot_2360233_major_body_boundary_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_2378499_major_body_boundary_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_2451916_major_body_boundary_summary_for_report()));
    assert!(release_summary.contains(
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        ));
    assert!(release_summary.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        }));
    assert!(release_summary.contains(
            "Validation report summary: validation-report-summary / validation-summary / report-summary"
        ));
    assert!(release_summary
        .contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(release_summary.contains("Workspace audit summary: workspace-audit-summary"));
    assert!(release_summary.contains("Artifact validation: validate-artifact"));
    assert!(release_summary.contains(
            "Packaged-artifact profile: byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]"
        ));
    assert!(release_summary.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported; unlisted outputs: []; support counts: stored=0, derived=2, approximated=0, unsupported=4, unlisted=0"
        ));
    assert!(release_summary.contains(
            "Packaged-artifact storage/reconstruction: Quantized linear segments stored in pleiades-compression artifact format; body-indexed segment tables support random access by body and lookup time across the advertised range; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, sidereal, and motion outputs remain unsupported"
        ));
    assert!(release_summary.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-draft; target thresholds: production thresholds recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
    assert!(release_summary.contains("Packaged-artifact fit margins: mean Δlon="));
    assert!(release_summary.contains("Packaged-artifact fit threshold violation count: 0"));
    assert!(
        release_summary.contains("Packaged-artifact fit threshold violations: 0; details: none")
    );
    assert!(release_summary.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
    assert!(release_summary.contains("Packaged-artifact source-fit and hold-out sync: "));
    assert!(release_summary.contains(
        "selected asteroid source request corpus=Selected asteroid source request corpus:"
    ));
    assert!(release_summary
        .contains("Packaged-artifact generation manifest: Packaged artifact generation manifest:"));
    assert!(release_summary.contains("Packaged-artifact size: "));
    assert!(release_summary.contains(
            "Artifact profile coverage: stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"
        ));
    assert!(release_summary.contains(&format!(
        "Packaged-artifact access: {}",
        format_packaged_artifact_access_summary()
    )));
    assert_report_contains_exact_line(
        &release_summary,
        &format!(
            "Packaged-artifact generation policy: {}",
            packaged_artifact_generation_policy_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &release_summary,
        &format!(
            "Packaged-artifact normalized intermediates: {}",
            packaged_artifact_normalized_intermediate_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &release_summary,
        &format!(
            "Packaged-artifact generation residual bodies: {}",
            validated_packaged_artifact_generation_residual_bodies_summary_for_report()
                .expect("packaged artifact residual bodies summary should validate")
        ),
    );
    assert_report_contains_exact_line(
            &release_summary,
            "Packaged lookup epoch policy: TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
        );
    assert!(release_summary.contains(
            "Packaged-artifact regeneration: Packaged artifact regeneration source: label=stage-5 packaged-data draft"
        ));
    assert!(release_summary
        .contains("quantization scales: stored=Longitude=9, Latitude=9, DistanceAu=10"));
    assert!(release_summary.contains("fit envelope:"));
    assert!(release_summary.contains("segment samples across"));
    assert!(release_summary.contains("checksum=0x"));
    assert!(release_summary.contains(
        &pleiades_data::packaged_artifact_regeneration_summary_details().generation_policy_line()
    ));
    assert!(release_summary.contains(&format!(
        "artifact version={}",
        pleiades_data::packaged_artifact_regeneration_summary_details().artifact_version
    )));
    assert!(release_profile_identifiers.contains(&format!(
        "Release profile identifiers: {}",
        release_profiles
    )));
    assert!(release_profile_identifiers_summary.contains("Release profile identifiers summary"));
    assert!(release_profile_identifiers_summary.contains(&format!(
        "Summary line: {}",
        release_profiles.summary_line()
    )));
    assert!(manifest.contains("release-profile-identifiers.txt"));
    assert!(manifest.contains("release-profile-identifiers-summary.txt"));
    assert!(manifest.contains("release-house-system-canonical-names-summary.txt"));
    assert!(manifest.contains("release-ayanamsa-canonical-names-summary.txt"));
    assert!(manifest.contains("release-house-validation-summary.txt"));
    assert!(manifest.contains("house-code-aliases-summary.txt"));
    assert!(manifest.contains("house-formula-families-summary.txt"));
    assert!(manifest.contains("house-latitude-sensitive-summary.txt"));
    assert!(manifest.contains("house-latitude-sensitive-constraints-summary.txt"));
    assert!(manifest.contains("house-latitude-sensitive-failure-modes-summary.txt"));
    assert!(release_summary.contains("Comparison envelope: samples:"));
    assert!(release_summary
        .contains("Comparison corpus release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(release_summary.contains("Release-grade body claims: Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies"));
    assert!(release_summary.contains("Body/date/channel claims:"));
    let comparison_report = compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &release_grade_corpus(),
    )
    .expect("comparison should build");
    assert_report_contains_exact_line(
        &release_summary,
        &format!(
            "Comparison tail envelope: {}",
            comparison_tail_envelope(&comparison_report.samples)
                .expect("comparison tail envelope should exist")
        ),
    );
    assert!(release_summary.contains("mean longitude delta:"));
    assert!(release_summary.contains("median longitude delta:"));
    assert!(release_summary.contains("mean latitude delta:"));
    assert!(release_summary.contains("median latitude delta:"));
    assert!(release_summary.contains("mean distance delta:"));
    assert!(release_summary.contains("median distance delta:"));
    assert!(release_summary.contains("ELP lunar capability: lunar capability summary:"));
    assert!(release_summary.contains(
            "ELP lunar request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        ));
    assert!(release_summary.contains(
            "ELP frame treatment: Geocentric ecliptic coordinates are produced directly from the truncated lunar series; equatorial coordinates are derived with a mean-obliquity transform"
        ));
    assert!(release_summary.contains(
            "lunar theory catalog: 1 entry, 1 selected entry; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
    assert!(release_summary.contains(
            "lunar source selection: Compact Meeus-style truncated lunar baseline [selected key: source identifier=meeus-style-truncated-lunar-baseline; family key: source family=Meeus-style truncated analytical baseline; family: Meeus-style truncated analytical baseline]; aliases: Meeus-style truncated lunar baseline"
        ));
    assert!(release_summary.contains(
            "lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies, TT requests=5, TDB requests=4, order=preserved, single-query parity=preserved"
        ));
    assert!(release_summary.contains(
            "lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"
        ));
    assert!(release_summary.contains("JPL independent hold-out:"));
    assert!(release_summary.contains("Reference/hold-out overlap:"));
    assert!(release_summary.contains(&independent_holdout_source_summary_for_report()));
    assert!(release_summary.contains(&independent_holdout_high_curvature_summary_for_report()));
    assert_report_contains_exact_line(
        &release_summary,
        &independent_holdout_snapshot_source_window_summary_for_report(),
    );
    assert!(release_summary.contains(&independent_holdout_manifest_summary_for_report()));
    assert!(release_summary.contains("JPL independent hold-out equatorial parity:"));
    assert!(release_summary.contains("JPL independent hold-out batch parity:"));
    assert_report_contains_exact_line(
            &release_summary,
            "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false",
        );
    assert_report_contains_exact_line(
            &release_summary,
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant",
        );
    assert!(release_summary.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
    assert!(release_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
    assert!(release_summary.contains("JPL frame treatment: checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"));
    assert!(release_summary.contains(
            "JPL reference snapshot equatorial parity: 357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
    assert!(release_summary.contains(
            "JPL reference snapshot batch parity: 357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
    assert!(release_summary.contains("Production generation coverage:"));
    assert!(release_summary.contains("JPL production-generation coverage:"));
    assert!(release_summary.contains("JPL production-generation manifest:"));
    assert!(release_summary.contains("JPL production-generation manifest checksum:"));
    assert!(release_summary.contains("JPL production-generation source windows:"));
    assert!(release_summary.contains("JPL production-generation body-class coverage:"));
    assert!(release_summary.contains("JPL production-generation boundary overlay:"));
    assert!(release_summary.contains("JPL production-generation boundary body-class coverage:"));
    assert!(release_summary.contains("JPL production-generation boundary windows:"));
    assert!(release_summary.contains("JPL production-generation boundary request corpus:"));
    assert!(release_summary.contains("Production generation boundary overlay source:"));
    assert!(
        release_summary.contains("Source corpus posture: comparison corpus release-grade guard:")
    );
    assert!(release_summary.contains("JPL source corpus contract:"));
    assert!(release_summary.contains("Source-backed backend evidence:"));
    assert!(release_summary.contains(&pleiades_jpl::jpl_provenance_only_summary_for_report()));
    assert!(release_summary.contains("JPL evidence classification: release-tolerance=reference/comparison/production-generation validation summaries; hold-out=independent hold-out rows and interpolation-quality summaries; fixture exactness=reference snapshot exact J2000 evidence; provenance-only=source and manifest summaries"));
    assert!(
        release_summary.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report())
    );
    assert!(release_summary.contains(&reference_snapshot_sparse_boundary_summary_for_report()));
    assert!(release_summary.contains(&reference_snapshot_pre_bridge_boundary_summary_for_report()));
    assert!(release_summary.contains(&reference_snapshot_dense_boundary_summary_for_report()));
    assert!(release_summary.contains("Selected asteroid evidence:"));
    assert!(release_summary.contains("Selected asteroid batch parity:"));
    assert!(release_summary.contains("Selected asteroid source windows:"));
    assert!(release_summary.contains("Reference snapshot coverage:"));
    assert!(release_summary.contains("Reference snapshot body-class coverage: major bodies: 262 rows across 10 bodies and 31 epochs; major windows: "));
    assert!(release_summary.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
    assert!(release_summary.contains(&reference_snapshot_mars_outer_boundary_summary_for_report()));
    assert!(
        release_summary.contains(&reference_snapshot_high_curvature_window_summary_for_report())
    );
    assert!(release_summary
        .contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report()));
    assert!(release_summary.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(release_summary
        .contains("selected asteroids: 95 rows across 6 bodies and 17 epochs; asteroid windows: "));
    assert!(release_summary.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_1500_selected_body_boundary_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_1600_selected_body_boundary_summary_for_report()));
    assert!(release_summary.contains(&reference_snapshot_source_summary_for_report()));
    assert!(release_summary.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(release_summary
        .contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(release_summary.contains("Selected asteroid evidence:"));
    assert!(release_summary.contains("Selected asteroid equatorial evidence:"));
    assert!(release_summary.contains("Comparison snapshot coverage:"));
    assert!(release_summary.contains("VSOP87 evidence:"));
    assert!(release_summary.contains("VSOP87 source-backed body-class envelopes:"));
    assert!(release_summary.contains("VSOP87 canonical J2000 equatorial body-class envelopes:"));
    assert!(release_summary.contains("Luminary: samples=1, bodies: Sun"));
    assert!(release_summary.contains("median Δlon="));
    assert!(release_summary.contains("p95 Δlon="));
    assert!(release_summary.contains("median Δlat="));
    assert!(release_summary.contains("p95 Δlat="));
    assert!(release_summary.contains("median Δdist="));
    assert!(release_summary.contains("p95 Δdist="));
    assert!(release_summary.contains(
        "Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
    ));
    assert!(release_summary.contains("VSOP87 source documentation:"));
    assert!(release_summary.contains("VSOP87 frame treatment:"));
    assert!(release_summary.contains("VSOP87 request policy:"));
    assert!(release_summary.contains("VSOP87 source audit:"));
    assert!(release_summary.contains("VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"));
    assert!(release_summary.contains("VSOP87 generated binary audit:"));
    assert!(release_summary.contains("VSOP87 canonical J2000 source-backed evidence:"));
    assert!(release_summary.contains("VSOP87 canonical J2000 interim outliers: none"));
    assert!(release_summary.contains("VSOP87 canonical J2000 equatorial companion evidence:"));
    assert!(release_summary.contains("VSOP87 canonical J2000 batch parity:"));
    assert!(release_summary.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
    assert!(release_summary.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
    assert!(release_summary.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
    assert!(release_summary.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
    assert!(release_summary.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
    assert!(release_summary.contains("VSOP87 canonical J1900 batch parity:"));
    assert!(release_summary.contains("VSOP87 source-backed body evidence:"));
    assert!(release_summary.contains("Lunar reference: lunar reference evidence:"));
    assert!(release_summary.contains("Lunar reference envelope:"));
    assert!(release_summary
        .contains("Lunar equatorial reference: lunar equatorial reference evidence:"));
    assert!(release_summary.contains("Lunar equatorial reference envelope:"));
    assert!(release_summary.contains("Lunar source windows: lunar source windows: 7 exact Moon samples across 1 bodies in 2 exact windows; 4 reference-only apparent Moon samples across 1 bodies in 4 apparent windows"));
    assert!(
        release_summary.contains("Lunar apparent comparison: lunar apparent comparison evidence:")
    );
    assert!(release_summary.contains("Lunar high-curvature continuity evidence"));
    assert!(release_summary.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(release_summary.contains("|Δlon| mean/median/p95="));
    assert!(release_summary.contains("|ΔDec| mean/median/p95="));
    assert!(release_summary.contains(&packaged_request_policy_summary_for_report()));
    assert!(release_summary.contains(&format!(
        "Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    )));
    assert!(release_summary.contains(&packaged_mixed_tt_tdb_batch_parity_summary_for_report()));
    assert!(release_summary.contains(&format!(
        "Packaged frame treatment: {}",
        packaged_frame_treatment_summary_for_report()
    )));
    assert!(release_summary.contains("Packaged batch parity:"));
    assert!(release_summary.contains("Packaged frame parity:"));
    assert!(release_summary.contains("Artifact boundary envelope:"));
    assert!(release_summary.contains(
        &artifact_inspection_summary_for_report()
            .expect("artifact inspection summary should build")
    ));
    assert!(release_summary.contains("residual-bearing bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"));
    assert!(release_summary.contains("applies to 11 bundled bodies"));
    assert!(release_summary.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(release_summary
        .lines()
        .any(|line| line == "Release notes summary: release-notes-summary"));
    assert!(artifact_summary.contains("Artifact summary"));
    assert!(
        packaged_artifact_generation_manifest.contains("Packaged artifact generation manifest:")
    );
    assert!(artifact_summary.contains("residual-bearing segments:"));
    assert!(artifact_summary.contains("residual-bearing bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"));
    assert!(artifact_summary.contains("Body classes: luminaries=2; major planets=8; lunar points=0; built-in asteroids=0; custom bodies=1; other bodies=0"));
    assert!(artifact_summary.contains(
            "Artifact profile: byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"
        ));
    assert!(artifact_summary.contains("Generation manifest:"));
    assert!(artifact_summary.contains("Packaged-artifact phase-2 corpus alignment: "));
    assert!(artifact_summary.contains(
        "selected asteroid source request corpus=Selected asteroid source request corpus:"
    ));
    assert!(artifact_summary.contains("Packaged-artifact source-fit and hold-out sync: "));
    assert!(artifact_summary.contains("Packaged artifact generation manifest:"));
    assert!(artifact_summary.contains("Artifact request policy"));
    assert!(artifact_summary.contains(
            "Artifact storage: Quantized linear segments stored in pleiades-compression artifact format; body-indexed segment tables support random access by body and lookup time across the advertised range; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, sidereal, and motion outputs remain unsupported"
        ));
    assert!(artifact_summary.contains(
            "regeneration provenance: Packaged artifact regeneration source: label=stage-5 packaged-data draft"
        ));
    assert!(artifact_summary
        .contains("quantization scales: stored=Longitude=9, Latitude=9, DistanceAu=10"));
    assert!(artifact_summary.contains("fit envelope:"));
    assert!(artifact_summary.contains("segment samples across"));
    assert!(artifact_summary.contains("checksum=0x"));
    assert!(artifact_summary.contains(
        &pleiades_data::packaged_artifact_regeneration_summary_details().generation_policy_line()
    ));
    assert!(artifact_summary.contains("residual bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros; applies to 11 bundled bodies"));
    assert!(artifact_summary.contains(&format!(
        "artifact version={}",
        pleiades_data::packaged_artifact_regeneration_summary_details().artifact_version
    )));
    assert!(artifact_summary.lines().any(|line| {
        line == format!(
            "  Packaged frame treatment: {}",
            packaged_frame_treatment_summary_for_report()
        )
    }));
    assert!(artifact_summary.contains("applies to 11 bundled bodies"));
    assert!(artifact_summary.contains("Model error envelope"));
    assert!(artifact_summary.contains("mean longitude delta:"));
    assert!(artifact_summary.contains("median longitude delta:"));
    assert!(artifact_summary.contains("95th percentile longitude delta:"));
    assert!(artifact_summary.contains("rms longitude delta:"));
    assert!(artifact_summary.contains("mean latitude delta:"));
    assert!(artifact_summary.contains("median latitude delta:"));
    assert!(artifact_summary.contains("95th percentile latitude delta:"));
    assert!(artifact_summary.contains("rms latitude delta:"));
    assert!(artifact_summary.contains("mean distance delta:"));
    assert!(artifact_summary.contains("median distance delta:"));
    assert!(artifact_summary.contains("95th percentile distance delta:"));
    assert!(artifact_summary.contains("rms distance delta:"));
    assert!(artifact_summary.contains("Expected tolerance status"));
    assert!(artifact_summary.contains("Comparison tolerance audit"));
    assert!(artifact_summary.contains("bodies checked:"));
    assert!(artifact_summary.contains("Artifact lookup benchmark"));
    assert!(artifact_summary.contains("Artifact batch lookup benchmark"));
    assert!(artifact_summary.contains("ns/lookup="));
    assert!(artifact_summary.contains("lookups/s="));
    assert!(artifact_summary.contains("Artifact decode benchmark"));
    assert!(artifact_summary.contains("ns/decode="));
    assert!(artifact_summary.contains("decodes/s="));
    assert!(artifact_summary.contains("Release summary: release-summary"));
    assert!(artifact_summary.contains("Release notes summary: release-notes-summary"));
    assert!(artifact_summary.contains("Workspace audit: workspace-audit / audit"));
    assert!(
        artifact_summary.contains("Compatibility profile summary: compatibility-profile-summary")
    );
    assert!(artifact_summary
        .contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(artifact_summary.contains("Release checklist summary: release-checklist-summary"));
    assert!(artifact_summary.contains("Release bundle verification: verify-release-bundle"));
    assert!(artifact_summary.contains(
            "custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus"
        ));
    assert!(release_checklist.contains("Release checklist"));
    assert!(release_checklist.contains("Repository-managed release gates:"));
    assert!(release_checklist
        .contains("[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile"));
    assert!(
        release_checklist.contains("[x] cargo run -q -p pleiades-validate -- validate-artifact")
    );
    assert!(release_checklist.contains("Manual bundle workflow:"));
    assert!(release_checklist.contains("bundle-release --out /tmp/pleiades-release"));
    assert!(release_checklist.contains("verify-release-bundle --out /tmp/pleiades-release"));
    assert!(release_checklist.contains("docs/release-reproducibility.md"));
    assert!(release_checklist
        .contains("docs/release-reproducibility.md (broader source-corpus provenance contract)"));
    assert!(release_checklist.contains("Bundle contents:"));
    assert!(release_checklist.contains("compatibility-profile-summary.txt"));
    assert!(release_checklist.contains("release-notes-summary.txt"));
    assert!(release_checklist.contains("workspace-audit-summary.txt"));
    assert!(release_checklist.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(release_checklist.contains("API stability summary: api-stability-summary"));
    assert!(release_checklist.contains("release-summary.txt"));
    assert!(release_checklist.contains("release-checklist-summary.txt"));
    assert!(release_checklist.contains("bundle-manifest.checksum.txt"));
    assert!(release_checklist_summary.contains("Release checklist summary"));
    assert!(release_checklist_summary
        .contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(release_checklist_summary.contains("Artifact validation: validate-artifact"));
    assert!(release_checklist_summary.contains("Workspace audit summary: workspace-audit-summary"));
    assert!(release_checklist_summary.contains("Workspace audit: workspace-audit / audit"));
    assert!(release_checklist_summary.contains("Repository-managed release gates: 10 items"));
    assert!(release_checklist_summary.contains("Manual bundle workflow: 4 items"));
    assert!(release_checklist_summary.contains("Bundle contents: 25 items"));
    assert!(release_checklist_summary.contains("External publishing reminders: 3 items"));
    assert!(backend_matrix.contains("Implemented backend matrices"));
    assert!(backend_matrix.contains("JPL snapshot reference backend"));
    assert!(backend_matrix.contains(&format!(
        "House code aliases: {}",
        compatibility_profile.house_code_aliases_summary_line()
    )));
    assert!(backend_matrix_summary.contains("Backend matrix summary"));
    assert!(backend_matrix_summary.contains(&format!(
        "House code aliases: {}",
        compatibility_profile.house_code_aliases_summary_line()
    )));
    assert!(backend_matrix_summary.contains(
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        ));
    assert!(
            backend_matrix_summary.lines().any(|line| {
                line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
            })
        );
    assert!(backend_matrix_summary.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(backend_matrix_summary
        .contains("Production generation source: strategy=documented hybrid fixture corpus"));
    assert!(backend_matrix_summary.contains(
        "Production generation source revision: source revision=reference_snapshot.csv checksum=0x"
    ));
    assert!(backend_matrix_summary.contains("Production generation coverage:"));
    assert!(backend_matrix_summary.contains("Backends: 5"));
    assert!(backend_matrix_summary.contains("Algorithmic: 2"));
    assert!(backend_matrix_summary.contains("Composite: 1"));
    assert!(backend_matrix_summary.contains(
            "JPL reference snapshot equatorial parity: 357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
    assert!(
        backend_matrix_summary.contains(&reference_snapshot_major_body_bridge_summary_for_report())
    );
    assert!(
        backend_matrix_summary.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples")
    );
    assert!(backend_matrix_summary.contains("VSOP87 canonical J2000 interim outliers: none"));
    assert!(backend_matrix_summary
        .contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
    assert!(backend_matrix_summary.contains("Selected asteroid evidence: 6 exact J2000 samples"));
    assert!(backend_matrix_summary.contains("Selected asteroid batch parity:"));
    assert!(backend_matrix_summary
        .contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(backend_matrix.contains(
            "selected asteroid coverage: 6 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        ));
    assert!(backend_matrix.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(backend_matrix.contains("exact J2000 evidence: 6 bodies at JD 2451545.0"));
    assert!(backend_matrix.contains(
        "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0"
    ));
    assert!(api_stability.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(api_stability_summary.contains("API stability summary"));
    assert!(api_stability_summary.contains(&format!(
        "Profile: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(api_stability_summary.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(comparison_corpus_summary.contains("Comparison corpus summary"));
    assert!(benchmark_corpus_summary.contains("Benchmark corpus summary"));
    assert!(validation_report_summary.contains("Validation report summary"));
    assert!(validation_report_summary.contains("Comparison corpus"));
    assert!(validation_report_summary.contains("Body-class tolerance posture"));
    assert!(validation_report_summary.contains("Tolerance policy catalog"));
    assert!(validation_report_summary.contains("Expected tolerance status"));
    assert!(validation_report_summary.contains("VSOP87 source-backed evidence"));
    assert!(validation_report_summary.contains("VSOP87 source-backed body-class envelopes:"));
    assert!(validation_report_summary
        .contains("VSOP87 canonical J2000 equatorial body-class envelopes:"));
    assert!(workspace_audit_summary.contains("Workspace audit summary"));
    assert!(workspace_audit_summary.contains("Result: no workspace policy violations detected"));
    assert!(validation_report_summary.contains("VSOP87 source documentation: 8 source specs, 8 source-backed body profiles, 1 approximate fallback mean-element body profile (Pluto); source-backed bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
    assert!(validation_report_summary.contains(
            "source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"
        ));
    assert!(validation_report_summary.contains(
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        ));
    assert!(validation_report_summary.contains("VSOP87 request policy:"));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
    assert!(validation_report_summary
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
    assert!(validation_report_summary
        .contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files"));
    assert!(validation_report_summary
        .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
    assert!(validation_report_summary.contains("VSOP87 canonical J2000 batch parity:"));
    assert!(
        validation_report_summary.contains("VSOP87 supported-body J2000 ecliptic batch parity:")
    );
    assert!(
        validation_report_summary.contains("VSOP87 supported-body J2000 equatorial batch parity:")
    );
    assert!(
        validation_report_summary.contains("VSOP87 supported-body J1900 ecliptic batch parity:")
    );
    assert!(
        validation_report_summary.contains("VSOP87 supported-body J1900 equatorial batch parity:")
    );
    assert!(validation_report_summary.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
    assert!(validation_report_summary
        .contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
    assert!(validation_report_summary.contains("VSOP87 canonical J1900 batch parity:"));
    assert!(validation_report_summary.contains("generated binary VSOP87B"));
    assert!(validation_report_summary.contains("VSOP87 source-backed evidence"));
    assert!(validation_report_summary.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
    assert!(validation_report_summary
        .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
    assert!(validation_report_summary.contains(
            "ELP lunar theory specification: Compact Meeus-style truncated lunar baseline [meeus-style-truncated-lunar-baseline; family: Meeus-style truncated analytical baseline; selected key: source identifier=meeus-style-truncated-lunar-baseline]"
        ));
    assert!(validation_report_summary.contains(
            "lunar source selection: Compact Meeus-style truncated lunar baseline [selected key: source identifier=meeus-style-truncated-lunar-baseline; family key: source family=Meeus-style truncated analytical baseline; family: Meeus-style truncated analytical baseline]; aliases: Meeus-style truncated lunar baseline"
        ));
    assert!(validation_report_summary.contains(
            "lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies, TT requests=5, TDB requests=4, order=preserved, single-query parity=preserved"
        ));
    assert!(validation_report_summary.contains(
            "lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"
        ));
    assert!(validation_report_summary.contains("ELP lunar capability: lunar capability summary:"));
    assert!(validation_report_summary.contains(
            "ELP lunar request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        ));
    assert!(validation_report_summary.contains(
            "ELP frame treatment: Geocentric ecliptic coordinates are produced directly from the truncated lunar series; equatorial coordinates are derived with a mean-obliquity transform"
        ));
    assert!(validation_report_summary.contains("ELP lunar theory limitations:"));
    assert!(validation_report_summary.contains(
            "lunar theory catalog: 1 entry, 1 selected entry; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
    assert!(validation_report_summary.contains(
            "lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; selected key: source identifier=meeus-style-truncated-lunar-baseline; selected family key: source family=Meeus-style truncated analytical baseline; aliases=1; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)"
        ));
    assert!(manifest.contains("production-generation-summary.txt"));
    assert!(manifest.contains("production generation summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-body-class-coverage-summary.txt"));
    assert!(manifest
        .contains("production generation body-class coverage summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-source-summary.txt"));
    assert!(manifest.contains("production-generation-source-window-summary.txt"));
    assert!(manifest.contains("production-generation-quarter-day-boundary-summary.txt"));
    assert!(manifest.contains("production-generation-corpus-shape-summary.txt"));
    assert!(manifest.contains("production generation source summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production generation corpus shape summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-manifest-summary.txt"));
    assert!(manifest.contains("production generation manifest summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-manifest-checksum-summary.txt"));
    assert!(manifest
        .contains("production generation manifest checksum summary checksum (fnv1a-64): 0x"));
    assert!(validation_report_summary
        .contains("lunar reference error envelope: 9 samples across 5 bodies"));
    assert!(validation_report_summary.contains("max Δlon="));
    assert!(validation_report_summary.contains("mean Δlon="));
    assert!(validation_report_summary.contains("median Δlon="));
    assert!(validation_report_summary.contains("p95 Δlon="));
    assert!(validation_report_summary.contains("max Δlat="));
    assert!(validation_report_summary.contains("mean Δlat="));
    assert!(validation_report_summary.contains("median Δlat="));
    assert!(validation_report_summary.contains("p95 Δlat="));
    assert!(validation_report_summary.contains("limits: Δlon≤1e-4°"));
    assert!(validation_report_summary.contains("request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"));
    assert!(validation_report_summary
        .contains("lookup epoch policy=TT-grid retag without relativistic correction"));
    assert!(validation_report_summary
        .contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
    assert!(validation_report_summary.contains("date-range note: Validated against the published 1992-04-12 geocentric Moon example, the published 1992-04-12 geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform, the reference-only published 1968-12-24 apparent geocentric Moon comparison datum, the reference-only published 2004-04-01 NASA RP 1349 apparent Moon table row, the reference-only published 2006-09-07 EclipseWise apparent Moon coordinate row, J2000 lunar-point anchors including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example"));
    assert!(validation_report_summary
        .contains("lunar equatorial reference error envelope: 3 samples across 1 bodies"));
    assert!(validation_report_summary.contains("mean ΔRA="));
    assert!(validation_report_summary.contains("median ΔRA="));
    assert!(validation_report_summary.contains("p95 ΔRA="));
    assert!(validation_report_summary.contains("limits: ΔRA≤1e-2°"));
    assert!(validation_report_summary.contains("citation: Jean Meeus"));
    assert!(validation_report_summary
        .contains("provenance: Published lunar position, node, and mean-point formulas"));
    assert!(validation_report_summary
        .contains("redistribution: No external coefficient-file redistribution constraints apply"));
    assert!(validation_report_summary
        .contains("license: The current baseline is handwritten pure Rust"));
    assert!(validation_report_summary.contains("2 unsupported bodies: True Apogee, True Perigee"));
    assert!(report.contains("Validation report"));
    assert!(report.contains("Expected tolerance status"));
    assert!(report.contains("margin Δlon="));
    assert!(report.contains("margin Δdist="));
    assert!(manifest.contains("Release bundle manifest"));
    assert!(manifest.contains("validation rounds: 1"));
    assert!(manifest.contains("compatibility-profile.txt"));
    assert!(manifest.contains("compatibility-profile-summary.txt"));
    assert!(manifest.contains("release-notes.txt"));
    assert!(manifest.contains("release-notes-summary.txt"));
    assert!(manifest.contains("backend-matrix.txt"));
    assert!(manifest.contains("backend-matrix-summary.txt"));
    assert!(manifest.contains("api-stability.txt"));
    assert!(manifest.contains("api-stability-summary.txt"));
    assert!(manifest.contains("comparison-corpus-summary.txt"));
    assert!(manifest.contains("source-corpus-summary.txt"));
    assert!(manifest.contains("jpl-source-posture-summary.txt"));
    assert!(manifest.contains("jpl-provenance-only-summary.txt"));
    assert!(manifest.contains("comparison-snapshot-summary.txt"));
    assert!(manifest.contains("comparison-snapshot-source-summary.txt"));
    assert!(manifest.contains("comparison-snapshot-source-window-summary.txt"));
    assert!(manifest.contains("comparison-snapshot-body-class-coverage-summary.txt"));
    assert!(manifest.contains("comparison-snapshot-manifest-summary.txt"));
    assert!(manifest.contains("reference-snapshot-2451917-major-body-boundary-summary.txt"));
    assert!(manifest.contains(
        "reference snapshot 2451917 major-body boundary summary checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains("reference-snapshot-2451916-major-body-dense-boundary-summary.txt"));
    assert!(manifest.contains(
        "reference snapshot 2451916 major-body dense boundary summary checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains("comparison-snapshot manifest summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("comparison-envelope-summary.txt"));
    assert!(manifest.contains("catalog-inventory-summary.txt"));
    assert!(manifest.contains("catalog-posture-summary.txt"));
    assert!(manifest.contains("catalog inventory summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("catalog posture summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("custom-definition-ayanamsa-labels-summary.txt"));
    assert!(manifest.contains("ayanamsa-provenance-summary.txt"));
    assert!(manifest.contains("ayanamsa provenance summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("validation-report-summary.txt"));
    assert!(manifest.contains("release-body-claims-summary.txt"));
    assert!(manifest.contains("body-date-channel-claims-summary.txt"));
    assert!(manifest.contains("request-policy-summary.txt"));
    assert!(manifest.contains("observer-policy-summary.txt"));
    assert!(manifest.contains("apparentness-policy-summary.txt"));
    assert!(manifest.contains("request-semantics-summary.txt"));
    assert!(manifest.contains("unsupported-modes-summary.txt"));
    assert!(manifest.contains("reference-snapshot-bridge-day-summary.txt"));
    assert!(manifest.contains("reference-snapshot-major-body-boundary-window-summary.txt"));
    assert!(manifest.contains("reference-snapshot-boundary-epoch-coverage-summary.txt"));
    assert!(manifest.contains("reference-snapshot-pre-bridge-boundary-summary.txt"));
    assert!(manifest.contains("reference-snapshot-2451918-major-body-boundary-summary.txt"));
    assert!(manifest.contains("reference-snapshot-2451919-major-body-boundary-summary.txt"));
    assert!(manifest.contains("reference-snapshot-2451916-major-body-dense-boundary-summary.txt"));
    assert!(manifest.contains("reference-snapshot-sparse-boundary-summary.txt"));
    assert!(manifest.contains("reference snapshot bridge day summary checksum (fnv1a-64): 0x"));
    assert!(manifest
        .contains("reference snapshot major-body boundary window summary checksum (fnv1a-64): 0x"));
    assert!(manifest
        .contains("reference snapshot boundary epoch coverage summary checksum (fnv1a-64): 0x"));
    assert!(
        manifest.contains("reference snapshot pre-bridge boundary summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains(
        "reference snapshot 2451918 major-body boundary summary checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains(
        "reference snapshot 2451919 major-body boundary summary checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains("reference snapshot sparse boundary summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("reference-snapshot-source-summary.txt"));
    assert!(manifest.contains("reference snapshot source summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("reference-snapshot-source-window-summary.txt"));
    assert!(manifest.contains("reference-snapshot-manifest-summary.txt"));
    assert!(manifest.contains("reference-snapshot-body-class-coverage-summary.txt"));
    assert!(manifest.contains("reference-snapshot-equatorial-parity-summary.txt"));
    assert!(manifest.contains("reference snapshot source window summary checksum (fnv1a-64): 0x"));
    assert!(
        manifest.contains("reference snapshot body-class coverage summary checksum (fnv1a-64): 0x")
    );
    assert!(
        manifest.contains("reference snapshot equatorial parity summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains("reference-asteroid-source-window-summary.txt"));
    assert!(manifest.contains("reference-asteroid-equatorial-evidence-summary.txt"));
    assert!(manifest.contains("reference asteroid source window summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("independent-holdout-source-window-summary.txt"));
    assert!(manifest.contains("independent-holdout-equatorial-parity-summary.txt"));
    assert!(manifest.contains("independent-holdout source window summary checksum (fnv1a-64): 0x"));
    assert!(
        manifest.contains("independent-holdout equatorial parity summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains("independent-holdout-body-class-coverage-summary.txt"));
    assert!(manifest
        .contains("independent-holdout body-class coverage summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-boundary-source-summary.txt"));
    assert!(manifest.contains("production-generation-boundary-window-summary.txt"));
    assert!(
        manifest.contains("production generation boundary source summary checksum (fnv1a-64): 0x")
    );
    assert!(
        manifest.contains("production generation boundary window summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains("production-generation-boundary-request-corpus-summary.txt"));
    assert!(
        manifest.contains("production-generation-boundary-request-corpus-equatorial-summary.txt")
    );
    assert!(manifest
        .contains("production generation boundary request corpus summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
        "production generation boundary request corpus equatorial summary checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains("reference-snapshot-summary.txt"));
    assert!(manifest.contains("reference snapshot summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-summary.txt"));
    assert!(manifest.contains("production generation summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-body-class-coverage-summary.txt"));
    assert!(manifest
        .contains("production generation body-class coverage summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-source-summary.txt"));
    assert!(manifest.contains("production-generation-source-window-summary.txt"));
    assert!(manifest.contains("production-generation-quarter-day-boundary-summary.txt"));
    assert!(manifest.contains("production-generation-corpus-shape-summary.txt"));
    assert!(manifest.contains("production generation source summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production generation corpus shape summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-manifest-summary.txt"));
    assert!(manifest.contains("production generation manifest summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("production-generation-manifest-checksum-summary.txt"));
    assert!(manifest
        .contains("production generation manifest checksum summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("time-scale-policy-summary.txt"));
    assert!(manifest.contains("delta-t-policy-summary.txt"));
    assert!(manifest.contains("zodiac-policy-summary.txt"));
    assert!(manifest.contains("native-sidereal-policy-summary.txt"));
    assert!(manifest.contains("native sidereal policy summary checksum (fnv1a-64): 0x"));
    assert!(
        manifest.contains("lunar theory limitations summary: lunar-theory-limitations-summary.txt")
    );
    assert!(manifest.contains("lunar theory limitations summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
        "lunar theory source selection summary: lunar-theory-source-selection-summary.txt"
    ));
    assert!(manifest.contains("lunar theory source selection summary checksum (fnv1a-64): 0x"));
    assert!(manifest
        .contains("lunar theory source family summary: lunar-theory-source-family-summary.txt"));
    assert!(manifest.contains("lunar theory source family summary checksum (fnv1a-64): 0x"));
    assert!(
        manifest.contains("lunar theory source window summary: lunar-source-window-summary.txt")
    );
    assert!(manifest.contains("lunar theory source window summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
        "lunar theory catalog validation summary: lunar-theory-catalog-validation-summary.txt"
    ));
    assert!(manifest.contains("lunar theory catalog validation summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("compatibility-caveats-summary.txt"));
    assert!(manifest.contains("native-dependency-audit-summary.txt"));
    assert!(manifest.contains("artifact-summary.txt"));
    assert!(manifest.contains("packaged-artifact-profile-coverage-summary.txt"));
    assert!(manifest.contains("packaged-artifact-access-summary.txt"));
    assert!(manifest.contains("packaged-artifact profile coverage summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("packaged-artifact access summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
        "packaged-artifact output support summary: packaged-artifact-output-support-summary.txt"
    ));
    assert!(manifest.contains("packaged-artifact output support summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
            "packaged-artifact fit sample classes summary: packaged-artifact-fit-sample-classes-summary.txt"
        ));
    assert!(
        manifest.contains("packaged-artifact fit sample classes summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains(
            "packaged-artifact fit threshold violation count summary: packaged-artifact-fit-threshold-violation-count-summary.txt"
        ));
    assert!(manifest.contains(
        "packaged-artifact fit threshold violation count summary checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains(
            "packaged-artifact fit threshold violations summary: packaged-artifact-fit-threshold-violations-summary.txt"
        ));
    assert!(manifest
        .contains("packaged-artifact fit threshold violations summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
        "packaged-artifact body cadence summary: packaged-artifact-body-cadence-summary.txt"
    ));
    assert!(manifest.contains("packaged-artifact body cadence summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
            "packaged-artifact body-class span cap summary: packaged-artifact-body-class-span-cap-summary.txt"
        ));
    assert!(
        manifest.contains("packaged-artifact body-class span cap summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains(
            "packaged-artifact normalized intermediate summary: packaged-artifact-normalized-intermediate-summary.txt"
        ));
    assert!(manifest
        .contains("packaged-artifact normalized intermediate summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains(
        "packaged-artifact speed policy summary: packaged-artifact-speed-policy-summary.txt"
    ));
    assert!(manifest.contains("packaged-artifact speed policy summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("packaged-artifact storage summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("packaged-artifact-production-profile-summary.txt"));
    assert!(manifest.contains("packaged-frame-treatment-summary.txt"));
    assert!(manifest.contains("packaged-artifact-target-threshold-summary.txt"));
    assert!(manifest.contains("packaged-artifact-target-threshold-state-summary.txt"));
    assert!(manifest.contains("packaged-artifact-source-fit-holdout-sync-summary.txt"));
    assert!(manifest.contains(
        "packaged-artifact source-fit and hold-out sync summary checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains("packaged-artifact-generation-policy-summary.txt"));
    assert!(manifest.contains("packaged-artifact generation policy summary: packaged-artifact-generation-policy-summary.txt"));
    assert!(
        manifest.contains("packaged-artifact generation policy summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains("packaged-artifact-regeneration-summary.txt"));
    assert!(manifest.contains(
        "packaged-artifact regeneration summary: packaged-artifact-regeneration-summary.txt"
    ));
    assert!(manifest.contains("packaged-artifact regeneration summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("packaged-artifact-generation-manifest.txt"));
    assert!(manifest.contains("packaged-artifact generation manifest checksum sidecar: packaged-artifact-generation-manifest.checksum.txt"));
    assert!(manifest.contains(
        "packaged-artifact generation manifest checksum sidecar checksum (fnv1a-64): 0x"
    ));
    assert!(manifest.contains("packaged-artifact-generation-manifest.checksum.txt"));
    assert!(manifest.contains("benchmark-corpus-summary.txt"));
    assert!(manifest.contains("chart-benchmark-corpus-summary.txt"));
    assert!(manifest.contains("selected-asteroid-source-request-corpus-summary.txt"));
    assert!(manifest.contains("selected-asteroid-source-request-corpus-equatorial-summary.txt"));
    assert!(manifest.contains("selected-asteroid-source-window-summary.txt"));
    assert!(manifest.contains("selected asteroid source window summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("interpolation-quality-request-corpus-summary.txt"));
    assert!(manifest.contains("benchmark-report.txt"));
    assert!(manifest.contains("validation-report.txt"));
    assert!(!manifest.contains("bundle-manifest.checksum.txt"));
    assert!(manifest.contains("source revision:"));
    assert!(manifest.contains("workspace status:"));
    assert!(manifest.contains("rustc version:"));
    assert!(manifest.contains("cargo version:"));
    assert!(manifest.contains("profile checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("profile summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("release notes checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("release notes summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("release summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("release-profile identifiers: release-profile-identifiers.txt"));
    assert!(manifest
        .contains("release-profile identifiers summary: release-profile-identifiers-summary.txt"));
    assert!(manifest.contains("release-profile identifiers checksum (fnv1a-64): 0x"));
    assert!(
        manifest.contains("release-house-system-canonical-names summary checksum (fnv1a-64): 0x")
    );
    assert!(manifest.contains("release-ayanamsa-canonical-names summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("release checklist checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("release checklist summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("backend matrix checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("backend matrix summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("api stability checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("comparison-envelope summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("comparison-corpus release-guard summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("catalog inventory summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("ayanamsa-provenance-summary.txt"));
    assert!(manifest.contains("ayanamsa provenance summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("validation report summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("workspace provenance summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("request policy summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("request surface summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("compatibility caveats summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("native-dependency audit summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("workspace audit summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("artifact summary checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("benchmark report checksum (fnv1a-64): 0x"));
    assert!(manifest.contains("validation report checksum (fnv1a-64): 0x"));
    assert!(manifest_checksum.trim().starts_with("0x"));

    let verified = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect("bundle verification should render");
    assert!(verified.contains("Release bundle"));
    assert!(verified.contains("bundle-manifest.txt"));
    assert!(verified.contains("bundle-manifest.checksum.txt"));
    assert!(verified.contains("compatibility-profile-summary.txt"));
    assert!(verified.contains("release-notes-summary.txt"));
    assert!(verified.contains("release-summary.txt"));
    assert!(verified.contains("release-checklist-summary.txt"));
    assert!(verified.contains("release-house-system-canonical-names-summary.txt"));
    assert!(verified.contains("release-ayanamsa-canonical-names-summary.txt"));
    assert!(verified.contains("release-house-validation-summary.txt"));
    assert!(verified.contains("reference-snapshot-2451916-major-body-dense-boundary-summary.txt"));
    assert!(verified.contains("house-code-aliases-summary.txt"));
    assert!(verified.contains("house-formula-families-summary.txt"));
    assert!(verified.contains("house-latitude-sensitive-summary.txt"));
    assert!(verified.contains("house-latitude-sensitive-constraints-summary.txt"));
    assert!(verified.contains("house-latitude-sensitive-failure-modes-summary.txt"));
    assert!(verified.contains("comparison-corpus-summary.txt"));
    assert!(verified.contains("source-corpus-summary.txt"));
    assert!(verified.contains("comparison-snapshot-summary.txt"));
    assert!(verified.contains("comparison-snapshot-source-summary.txt"));
    assert!(verified.contains("comparison-snapshot-body-class-coverage-summary.txt"));
    assert!(verified.contains("comparison-snapshot-manifest-summary.txt"));
    assert!(verified.contains("comparison-envelope-summary.txt"));
    assert!(verified.contains("comparison-corpus-release-guard-summary.txt"));
    assert!(verified.contains("reference-snapshot-bridge-day-summary.txt"));
    assert!(verified.contains("reference-snapshot-2451918-major-body-boundary-summary.txt"));
    assert!(verified.contains("reference-snapshot-2451919-major-body-boundary-summary.txt"));
    assert!(verified.contains("reference-snapshot-2451917-major-body-boundary-summary.txt"));
    assert!(verified.contains("reference-snapshot-2451916-major-body-dense-boundary-summary.txt"));
    assert!(verified.contains("reference-snapshot-sparse-boundary-summary.txt"));
    assert!(verified.contains("reference-snapshot-source-summary.txt"));
    assert!(verified.contains("reference-snapshot-source-window-summary.txt"));
    assert!(verified.contains("reference-snapshot-manifest-summary.txt"));
    assert!(verified.contains("reference-snapshot-body-class-coverage-summary.txt"));
    assert!(verified.contains("reference-snapshot-equatorial-parity-summary.txt"));
    assert!(verified.contains("reference-asteroid-source-window-summary.txt"));
    assert!(verified.contains("reference-asteroid-equatorial-evidence-summary.txt"));
    assert!(verified.contains("independent-holdout-source-window-summary.txt"));
    assert!(verified.contains("catalog-inventory-summary.txt"));
    assert!(verified.contains("custom-definition-ayanamsa-labels-summary.txt"));
    assert!(verified.contains("request-policy-summary.txt"));
    assert!(verified.contains("observer-policy-summary.txt"));
    assert!(verified.contains("apparentness-policy-summary.txt"));
    assert!(verified.contains("request-semantics-summary.txt"));
    assert!(verified.contains("unsupported-modes-summary.txt"));
    assert!(verified.contains("time-scale-policy-summary.txt"));
    assert!(verified.contains("delta-t-policy-summary.txt"));
    assert!(verified.contains("zodiac-policy-summary.txt"));
    assert!(verified.contains("native-sidereal-policy-summary.txt"));
    assert!(verified.contains("reference-snapshot-summary.txt"));
    assert!(verified.contains("production-generation-summary.txt"));
    assert!(verified.contains("production-generation-source-summary.txt"));
    assert!(verified.contains("production-generation-source-window-summary.txt"));
    assert!(verified.contains("production-generation-boundary-window-summary.txt"));
    assert!(verified.contains("production-generation-quarter-day-boundary-summary.txt"));
    assert!(verified.contains("production-generation-boundary-request-corpus-summary.txt"));
    assert!(verified.contains("lunar-theory-limitations-summary.txt"));
    assert!(verified.contains("lunar-theory-source-selection-summary.txt"));
    assert!(verified.contains("lunar-theory-source-family-summary.txt"));
    assert!(verified.contains("lunar-theory-catalog-validation-summary.txt"));
    assert!(verified.contains("request-surface-summary.txt"));
    assert!(verified.contains("compatibility-caveats-summary.txt"));
    assert!(verified.contains("workspace-provenance-summary.txt"));
    assert!(verified.contains("native-dependency-audit-summary.txt"));
    assert!(verified.contains("validation-report-summary.txt"));
    assert!(verified.contains("artifact-summary.txt"));
    assert!(verified.contains("benchmark-corpus-summary.txt"));
    assert!(verified.contains("selected-asteroid-source-request-corpus-summary.txt"));
    assert!(verified.contains("selected-asteroid-source-request-corpus-equatorial-summary.txt"));
    assert!(verified.contains("selected-asteroid-source-window-summary.txt"));
    assert!(verified.contains("interpolation-quality-request-corpus-summary.txt"));
    assert!(verified.contains("benchmark-report.txt"));
    assert!(verified.contains("source revision:"));
    assert!(verified.contains("workspace status:"));
    assert!(verified.contains("rustc version:"));
    assert!(verified.contains("cargo version:"));
    assert!(verified.contains("validation rounds: 1"));
    assert!(verified.contains("release notes checksum: 0x"));
    assert!(verified.contains("release notes summary checksum: 0x"));
    assert!(verified.contains("release-profile identifiers checksum: 0x"));
    assert!(verified.contains("release-house-system canonical names summary checksum: 0x"));
    assert!(verified.contains("release-ayanamsa canonical names summary checksum: 0x"));
    assert!(verified.contains("release checklist checksum: 0x"));
    assert!(verified.contains("release checklist summary checksum: 0x"));
    assert!(verified.contains("backend matrix checksum: 0x"));
    assert!(verified.contains("backend matrix summary checksum: 0x"));
    assert!(verified.contains("comparison-envelope summary checksum: 0x"));
    assert!(verified.contains("comparison-corpus release-guard summary checksum: 0x"));
    assert!(verified.contains("validation report summary checksum: 0x"));
    assert!(verified.contains("workspace audit summary checksum: 0x"));
    assert!(verified.contains("artifact summary checksum: 0x"));
    assert!(verified.contains("validation report checksum: 0x"));
    assert!(verified.contains("manifest checksum bytes:"));
    assert!(verified.contains("manifest checksum: 0x"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_missing_source_revision_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-source-revision",
        "source revision:",
        &[
            "missing manifest entry: source revision:",
            "missing source revision entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_workspace_status_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-workspace-status",
        "workspace status:",
        &[
            "missing manifest entry: workspace status:",
            "missing workspace status entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_request_surface_summary_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-request-surface-summary",
        "request surface summary:",
        &[
            "missing manifest entry: request surface summary:",
            "missing request surface summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-workspace-provenance-summary",
        "workspace provenance summary:",
        &[
            "missing manifest entry: workspace provenance summary:",
            "missing workspace provenance summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-interpolation-quality-request-corpus-summary",
        "interpolation-quality sample request corpus summary:",
        &[
            "missing manifest entry: interpolation-quality sample request corpus summary:",
            "missing interpolation-quality sample request corpus summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-snapshot-bridge-day-summary",
        "reference snapshot bridge day summary:",
        &[
            "missing manifest entry: reference snapshot bridge day summary:",
            "missing reference snapshot bridge day summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-snapshot-sparse-boundary-summary",
        "reference snapshot sparse boundary summary:",
        &[
            "missing manifest entry: reference snapshot sparse boundary summary:",
            "missing reference snapshot sparse boundary summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-asteroid-equatorial-evidence-summary",
        "reference asteroid equatorial evidence summary:",
        &[
            "missing manifest entry: reference asteroid equatorial evidence summary:",
            "missing reference asteroid equatorial evidence summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-target-house-scope-summary",
        "target-house-scope summary:",
        &[
            "missing manifest entry: target-house-scope summary:",
            "missing target-house-scope summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-target-ayanamsa-scope-summary",
        "target-ayanamsa-scope summary:",
        &[
            "missing manifest entry: target-ayanamsa-scope summary:",
            "missing target-ayanamsa-scope summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_production_generation_summary_entries() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-summary",
        "production generation summary:",
        &[
            "missing manifest entry: production generation summary:",
            "missing production generation summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-body-class-coverage-summary",
        "production generation body-class coverage summary:",
        &[
            "missing manifest entry: production generation body-class coverage summary:",
            "missing production generation body-class coverage summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-catalog-posture-summary",
        "catalog posture summary:",
        &[
            "missing manifest entry: catalog posture summary:",
            "missing catalog posture summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-source-summary",
        "production generation source summary:",
        &[
            "missing manifest entry: production generation source summary:",
            "missing production generation source summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-source-revision-summary",
        "production generation source revision summary:",
        &[
            "missing manifest entry: production generation source revision summary:",
            "missing production generation source revision summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-source-window-summary",
        "production generation source window summary:",
        &[
            "missing manifest entry: production generation source window summary:",
            "missing production generation source window summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-corpus-shape-summary",
        "production generation corpus shape summary:",
        &[
            "missing manifest entry: production generation corpus shape summary:",
            "missing production generation corpus shape summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-boundary-request-corpus-summary",
        "production generation boundary request corpus summary:",
        &[
            "missing manifest entry: production generation boundary request corpus summary:",
            "missing production generation boundary request corpus summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
            "pleiades-release-bundle-missing-production-generation-boundary-request-corpus-equatorial-summary",
            "production generation boundary request corpus equatorial summary:",
            &[
                "missing manifest entry: production generation boundary request corpus equatorial summary:",
                "missing production generation boundary request corpus equatorial summary entry",
            ],
        );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-manifest-summary",
        "production generation manifest summary:",
        &[
            "missing manifest entry: production generation manifest summary:",
            "missing production generation manifest summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-production-generation-manifest-checksum-summary",
        "production generation manifest checksum summary:",
        &[
            "missing manifest entry: production generation manifest checksum summary:",
            "missing production generation manifest checksum summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_release_catalog_summary_entries() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-release-house-system-canonical-names-summary",
        "release-house-system-canonical-names summary:",
        &[
            "missing manifest entry: release-house-system-canonical-names summary:",
            "missing release-house-system-canonical-names summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-release-ayanamsa-canonical-names-summary",
        "release-ayanamsa-canonical-names summary:",
        &[
            "missing manifest entry: release-ayanamsa-canonical-names summary:",
            "missing release-ayanamsa-canonical-names summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-release-house-validation-summary",
        "release-house-validation summary:",
        &[
            "missing manifest entry: release-house-validation summary:",
            "missing release-house-validation summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_reference_holdout_overlap_summary_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-holdout-overlap-summary",
        "reference-holdout overlap summary:",
        &[
            "missing manifest entry: reference-holdout overlap summary:",
            "missing reference-holdout overlap summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_reference_snapshot_source_window_summary_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-snapshot-source-window-summary",
        "reference snapshot source window summary:",
        &[
            "missing manifest entry: reference snapshot source window summary:",
            "missing reference snapshot source window summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_reference_snapshot_boundary_summary_entries() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-snapshot-major-body-boundary-window-summary",
        "reference snapshot major-body boundary window summary:",
        &[
            "missing manifest entry: reference snapshot major-body boundary window summary:",
            "missing reference snapshot major-body boundary window summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-snapshot-boundary-epoch-coverage-summary",
        "reference snapshot boundary epoch coverage summary:",
        &[
            "missing manifest entry: reference snapshot boundary epoch coverage summary:",
            "missing reference snapshot boundary epoch coverage summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-snapshot-pre-bridge-boundary-summary",
        "reference snapshot pre-bridge boundary summary:",
        &[
            "missing manifest entry: reference snapshot pre-bridge boundary summary:",
            "missing reference snapshot pre-bridge boundary summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_reference_asteroid_source_window_summary_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-reference-asteroid-source-window-summary",
        "reference asteroid source window summary:",
        &[
            "missing manifest entry: reference asteroid source window summary:",
            "missing reference asteroid source window summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_packaged_artifact_normalized_intermediate_summary_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-packaged-artifact-normalized-intermediate-summary",
        "packaged-artifact normalized intermediate summary:",
        &[
            "missing manifest entry: packaged-artifact normalized intermediate summary:",
            "missing packaged-artifact normalized intermediate summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_packaged_artifact_phase2_corpus_alignment_summary_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-packaged-artifact-phase2-corpus-alignment-summary",
        "packaged-artifact phase-2 corpus alignment summary:",
        &[
            "missing manifest entry: packaged-artifact phase-2 corpus alignment summary:",
            "missing packaged-artifact phase-2 corpus alignment summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_packaged_artifact_source_fit_holdout_sync_summary_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-packaged-artifact-source-fit-holdout-sync-summary",
        "packaged-artifact source-fit and hold-out sync summary:",
        &[
            "missing manifest entry: packaged-artifact source-fit and hold-out sync summary:",
            "missing packaged-artifact source-fit and hold-out sync summary entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_packaged_artifact_target_threshold_summary_entries() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-packaged-artifact-target-threshold-summary",
        "packaged-artifact target-threshold summary:",
        &[
            "missing manifest entry: packaged-artifact target-threshold summary:",
            "missing packaged-artifact target-threshold summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-packaged-artifact-target-threshold-state-summary",
        "packaged-artifact target-threshold state summary:",
        &[
            "missing manifest entry: packaged-artifact target-threshold state summary:",
            "missing packaged-artifact target-threshold state summary entry",
        ],
    );
    assert_release_bundle_rejects_missing_manifest_entry(
            "pleiades-release-bundle-missing-packaged-artifact-target-threshold-scope-envelopes-summary",
            "packaged-artifact target-threshold scope envelopes summary:",
            &[
                "missing manifest entry: packaged-artifact target-threshold scope envelopes summary:",
                "missing packaged-artifact target-threshold scope envelopes summary entry",
            ],
        );
}

#[test]
fn verify_release_bundle_rejects_missing_packaged_artifact_bundle_entries() {
    for (bundle_dir_prefix, manifest_line_prefix, expected_fragments) in [
            (
                "pleiades-release-bundle-missing-packaged-artifact-binary",
                "packaged-artifact:",
                ["missing manifest entry: packaged-artifact:", "missing packaged-artifact entry"],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-checksum-sidecar",
                "packaged-artifact checksum sidecar:",
                [
                    "missing manifest entry: packaged-artifact checksum sidecar:",
                    "missing packaged-artifact checksum sidecar entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-generation-manifest",
                "packaged-artifact generation manifest:",
                [
                    "missing manifest entry: packaged-artifact generation manifest:",
                    "missing packaged-artifact generation manifest entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-generation-manifest-checksum-sidecar",
                "packaged-artifact generation manifest checksum sidecar:",
                [
                    "missing manifest entry: packaged-artifact generation manifest checksum sidecar:",
                    "missing packaged-artifact generation manifest checksum sidecar entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-profile-coverage-summary",
                "packaged-artifact profile coverage summary:",
                [
                    "missing manifest entry: packaged-artifact profile coverage summary:",
                    "missing packaged-artifact profile coverage summary entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-access-summary",
                "packaged-artifact access summary:",
                [
                    "missing manifest entry: packaged-artifact access summary:",
                    "missing packaged-artifact access summary entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-output-support-summary",
                "packaged-artifact output support summary:",
                [
                    "missing manifest entry: packaged-artifact output support summary:",
                    "missing packaged-artifact output support summary entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-speed-policy-summary",
                "packaged-artifact speed policy summary:",
                [
                    "missing manifest entry: packaged-artifact speed policy summary:",
                    "missing packaged-artifact speed policy summary entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-storage-summary",
                "packaged-artifact storage summary:",
                [
                    "missing manifest entry: packaged-artifact storage summary:",
                    "missing packaged-artifact storage summary entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-artifact-production-profile-summary",
                "packaged-artifact production-profile summary:",
                [
                    "missing manifest entry: packaged-artifact production-profile summary:",
                    "missing packaged-artifact production-profile summary entry",
                ],
            ),
            (
                "pleiades-release-bundle-missing-packaged-frame-treatment-summary",
                "packaged-frame-treatment summary:",
                [
                    "missing manifest entry: packaged-frame-treatment summary:",
                    "missing packaged-frame-treatment summary entry",
                ],
            ),
        ] {
            assert_release_bundle_rejects_missing_manifest_entry(
                bundle_dir_prefix,
                manifest_line_prefix,
                &expected_fragments,
            );
        }
}

#[test]
fn verify_release_bundle_rejects_missing_rustc_version_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-rustc",
        "rustc version:",
        &[
            "missing manifest entry: rustc version:",
            "missing rustc version entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_missing_cargo_version_entry() {
    assert_release_bundle_rejects_missing_manifest_entry(
        "pleiades-release-bundle-missing-cargo",
        "cargo version:",
        &[
            "missing manifest entry: cargo version:",
            "missing cargo version entry",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_blank_source_revision_entry() {
    assert_release_bundle_rejects_blank_manifest_value(
        "pleiades-release-bundle-blank-source-revision",
        "source revision:",
        &["missing source revision entry"],
    );
}

#[test]
fn verify_release_bundle_rejects_blank_workspace_status_entry() {
    assert_release_bundle_rejects_blank_manifest_value(
        "pleiades-release-bundle-blank-workspace-status",
        "workspace status:",
        &["missing workspace status entry"],
    );
}

#[test]
fn verify_release_bundle_rejects_blank_rustc_version_entry() {
    assert_release_bundle_rejects_blank_manifest_value(
        "pleiades-release-bundle-blank-rustc",
        "rustc version:",
        &["missing rustc version entry"],
    );
}

#[test]
fn verify_release_bundle_rejects_blank_cargo_version_entry() {
    assert_release_bundle_rejects_blank_manifest_value(
        "pleiades-release-bundle-blank-cargo",
        "cargo version:",
        &["missing cargo version entry"],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_source_revision_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-source-revision",
        "source revision:",
        &[
            "duplicate entry: source revision:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_workspace_status_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-workspace-status",
        "workspace status:",
        &[
            "duplicate entry: workspace status:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_rustc_version_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-rustc",
        "rustc version:",
        &[
            "duplicate entry: rustc version:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_cargo_version_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-cargo",
        "cargo version:",
        &[
            "duplicate entry: cargo version:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_blank_profile_id_entry() {
    assert_release_bundle_rejects_blank_manifest_value(
        "pleiades-release-bundle-blank-profile-id",
        "profile id:",
        &["missing profile id entry"],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_profile_id_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-profile-id",
        "profile id:",
        &[
            "duplicate entry: profile id:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_api_stability_posture_id_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-api-stability-posture-id",
        "api stability posture id:",
        &[
            "duplicate entry: api stability posture id:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_validation_rounds_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-validation-rounds",
        "validation rounds:",
        &[
            "duplicate entry: validation rounds:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_release_summary_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-release-summary",
        "release summary:",
        &[
            "duplicate entry: release summary:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_duplicate_release_notes_summary_entry() {
    assert_release_bundle_rejects_duplicate_manifest_entry(
        "pleiades-release-bundle-duplicate-release-notes-summary",
        "release notes summary:",
        &[
            "duplicate entry: release notes summary:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_whitespace_source_revision_entry() {
    assert_release_bundle_rejects_whitespace_manifest_entry(
        "pleiades-release-bundle-whitespace-source-revision",
        "source revision:",
        &[
            "unexpected leading or trailing whitespace in manifest entry: source revision:",
            "release bundle verification failed",
        ],
    );
}

#[test]
fn verify_release_bundle_rejects_blank_api_stability_posture_id_entry() {
    assert_release_bundle_rejects_blank_manifest_value(
        "pleiades-release-bundle-blank-api-stability-posture-id",
        "api stability posture id:",
        &["missing API stability posture id entry"],
    );
}

#[test]
fn verify_release_bundle_rejects_checksum_mismatches() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt");
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
        "profile checksum (fnv1a-64):",
        "profile checksum (fnv1a-64): 0x0000000000000000 #",
    );
    std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a corrupted manifest");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("invalid profile checksum")
            || error.contains("missing 0x prefix")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_tampered_manifest_file() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-manifest");
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
    let tampered = manifest.replace("validation rounds: 1", "validation rounds: 2");
    std::fs::write(&manifest_path, tampered).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered manifest file");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("bundle manifest checksum mismatch")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_invalid_validation_rounds() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-invalid-validation-rounds");
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
    let tampered_manifest = manifest.replace("validation rounds: 1", "validation rounds: nope");
    std::fs::write(&manifest_path, &tampered_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&tampered_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for invalid validation rounds");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("invalid validation rounds entry"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn release_bundle_validate_accepts_rendered_bundle() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-validate-accepts");
    let bundle = render_release_bundle(1, &bundle_dir).expect("release bundle should render");
    bundle
        .validate()
        .expect("rendered release bundle should validate");

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn release_bundle_validate_rejects_whitespace_padded_provenance() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-provenance-padding");
    let mut bundle = render_release_bundle(1, &bundle_dir).expect("release bundle should render");
    let source_revision = bundle.source_revision.clone();
    bundle.source_revision = format!(" {source_revision} ");

    let error = bundle
        .validate()
        .expect_err("padded provenance should be rejected by bundle validation");
    let error = error.to_string();
    assert!(error.contains("invalid source revision entry"));
    assert!(error.contains("unexpected leading or trailing whitespace"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn release_bundle_validate_rejects_placeholder_provenance() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-provenance-placeholder");
    let mut bundle = render_release_bundle(1, &bundle_dir).expect("release bundle should render");
    bundle.rustc_version = "unknown".to_string();

    let error = bundle
        .validate()
        .expect_err("placeholder provenance should be rejected by bundle validation");
    let error = error.to_string();
    assert!(error.contains("invalid rustc version entry"));
    assert!(error.contains("placeholder values are not allowed"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn release_bundle_validate_rejects_multiline_provenance() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-provenance-multiline");
    let mut bundle = render_release_bundle(1, &bundle_dir).expect("release bundle should render");
    bundle.workspace_status = "clean\nmodified".to_string();

    let error = bundle
        .validate()
        .expect_err("multiline provenance should be rejected by bundle validation");
    let error = error.to_string();
    assert!(error.contains("invalid workspace status entry"));
    assert!(error.contains("unexpected line break"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn release_bundle_validate_rejects_manifest_path_drift() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-manifest-path-drift");
    let mut bundle = render_release_bundle(1, &bundle_dir).expect("release bundle should render");
    bundle.manifest_path = bundle.output_dir.join("bundle-manifest-drift.txt");

    let error = bundle
        .validate()
        .expect_err("path drift should be rejected by bundle validation");
    let error = error.to_string();
    assert!(error.contains("unexpected bundle manifest file path"));
    assert!(error.contains("bundle-manifest.txt"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_tampered_manifest_checksum_sidecar() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-manifest-checksum");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    let tampered = "0x0000000000000000\n";
    std::fs::write(&checksum_path, tampered).expect("manifest checksum should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered manifest checksum sidecar");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("bundle manifest checksum mismatch")
            || error.contains("invalid bundle manifest checksum sidecar value")
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_missing_manifest_checksum_sidecar() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-missing-manifest-checksum");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::remove_file(&checksum_path).expect("manifest checksum sidecar should be removable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a missing manifest checksum sidecar");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("missing bundle manifest checksum sidecar file"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_malformed_manifest_checksum_sidecar() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-malformed-manifest-checksum");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(&checksum_path, " 0x0000000000000000 \n")
        .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a malformed manifest checksum sidecar");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("invalid bundle manifest checksum sidecar value"));
    assert!(error.contains("unexpected leading or trailing whitespace"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_noncanonical_manifest_checksum_sidecar() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-noncanonical-manifest-checksum");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(&checksum_path, "0x1\n").expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a noncanonical manifest checksum sidecar");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("invalid bundle manifest checksum sidecar value"));
    assert!(error.contains("expected exactly 16 lowercase hex digits"));
    assert!(error.contains("found \"0x1\""));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_noncanonical_manifest_checksum_entry() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-noncanonical-manifest-entry");
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
    let rewritten = manifest
        .lines()
        .map(|line| {
            line.strip_prefix("profile checksum (fnv1a-64): ")
                .map(|value| {
                    let digits = value.strip_prefix("0x").unwrap_or(value);
                    format!(
                        "profile checksum (fnv1a-64): 0x{}",
                        digits.to_ascii_uppercase()
                    )
                })
                .unwrap_or_else(|| line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&manifest_path, format!("{rewritten}\n")).expect("manifest should be writable");

    let checksum = checksum64(
        &std::fs::read_to_string(&manifest_path).expect("manifest should exist after rewrite"),
    );
    std::fs::write(
        bundle_dir.join("bundle-manifest.checksum.txt"),
        format!("0x{checksum:016x}\n"),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a noncanonical manifest checksum entry");
    assert!(error.contains("release bundle verification failed"));
    assert!(error.contains("invalid profile checksum (fnv1a-64): value"));
    assert!(error.contains("expected exactly 16 lowercase hex digits"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn verify_release_bundle_rejects_unexpected_bundle_entries() {
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-extra-entry");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    std::fs::write(bundle_dir.join("unexpected.txt"), "spurious bundle content")
        .expect("unexpected file should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for unexpected bundle contents");
    assert!(
        error.contains("release bundle verification failed")
            || error.contains("unexpected release bundle directory contents")
    );
    assert!(error.contains("unexpected.txt"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
#[cfg(unix)]
fn verify_release_bundle_rejects_symlinked_release_summary_file() {
    assert_release_bundle_rejects_symlinked_text_file(
        "pleiades-release-bundle-symlinked-release-summary",
        "release-summary.txt",
        "release-notes.txt",
        "unexpected non-regular release bundle file",
    );
}
