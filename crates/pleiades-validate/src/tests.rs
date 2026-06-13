use pleiades_core::{
    current_release_profile_identifiers, Apparentness, Ayanamsa, CoordinateFrame, HouseSystem,
    JulianDay, Latitude, TimeScale,
};

use super::*;
use pleiades_jpl::comparison_bodies;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = format!(
        "{}-{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed),
    );
    let path = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temporary directory should be creatable");
    path
}

fn assert_report_contains_exact_line(report: &str, expected: &str) {
    let expected = expected.trim_start();
    assert!(
        report.lines().any(|line| line.trim_start() == expected),
        "expected report to contain line `{expected}`\nreport:\n{report}"
    );
}

fn assert_release_bundle_rejects_tampered_text_file(
    bundle_dir_prefix: &str,
    file_name: &str,
    expected_fragment: &str,
) {
    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let file_path = bundle_dir.join(file_name);
    let mut text = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|error| panic!("{file_name} should exist: {error}"));
    text.push_str("\nTampered for regression coverage.\n");
    std::fs::write(&file_path, text)
        .unwrap_or_else(|error| panic!("{file_name} should be writable: {error}"));

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered release bundle file");
    assert!(
        error.contains("release bundle verification failed") || error.contains(expected_fragment),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

fn assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
    bundle_dir_prefix: &str,
    file_name: &str,
    manifest_checksum_prefix: &str,
    from: &str,
    to: &str,
    expected_fragment: &str,
) {
    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let file_path = bundle_dir.join(file_name);
    let original = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|error| panic!("{file_name} should exist: {error}"));
    let tampered = original.replace(from, to);
    assert_ne!(
        original, tampered,
        "{file_name} should be changed by the regression edit"
    );
    std::fs::write(&file_path, &tampered)
        .unwrap_or_else(|error| panic!("{file_name} should be writable: {error}"));

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with(manifest_checksum_prefix))
        .unwrap_or_else(|| panic!("manifest should contain the {manifest_checksum_prefix} line"));
    let new_checksum_line = format!(
        "{manifest_checksum_prefix} 0x{:016x}",
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
        .expect_err("verification should fail for semantic release bundle drift");
    assert!(
        error.contains("release bundle verification failed") || error.contains(expected_fragment),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[cfg(unix)]
fn assert_release_bundle_rejects_symlinked_text_file(
    bundle_dir_prefix: &str,
    file_name: &str,
    link_target: &str,
    expected_fragment: &str,
) {
    use std::os::unix::fs::symlink;

    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");

    let file_path = bundle_dir.join(file_name);
    std::fs::remove_file(&file_path).expect("bundled text file should be removable");
    symlink(link_target, &file_path).expect("symlink should be creatable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a symlinked release bundle file");
    assert!(
        error.contains("release bundle verification failed") || error.contains(expected_fragment),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

fn assert_release_bundle_rejects_missing_manifest_entry(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
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
    let filtered = manifest
        .lines()
        .filter(|line| !line.starts_with(manifest_line_prefix))
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&manifest_path, format!("{filtered}\n")).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a manifest missing the requested entry");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment))
            || error.contains("unexpected release bundle manifest line count"),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

fn assert_release_bundle_rejects_blank_manifest_value(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
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
            if line.starts_with(manifest_line_prefix) {
                manifest_line_prefix.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&manifest_path, format!("{rewritten}\n")).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a manifest with a blank requested entry");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment)),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

fn assert_release_bundle_rejects_duplicate_manifest_entry(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
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
    let duplicate_line = manifest
        .lines()
        .find(|line| line.starts_with(manifest_line_prefix))
        .unwrap_or_else(|| panic!("{manifest_line_prefix} should exist"));
    let mut lines = manifest.lines().map(str::to_owned).collect::<Vec<_>>();
    lines.push(duplicate_line.to_string());
    std::fs::write(&manifest_path, format!("{}\n", lines.join("\n")))
        .expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a manifest with a duplicate requested entry");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment)),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

fn assert_release_bundle_rejects_whitespace_manifest_entry(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
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
            if line.starts_with(manifest_line_prefix) {
                format!("{line} ")
            } else {
                line.to_string()
            }
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
        .expect_err("verification should fail for a manifest with noncanonical whitespace");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment)),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn default_corpus_covers_the_comparison_snapshot() {
    let corpus = default_corpus();
    let summary = corpus.summary();
    assert_eq!(corpus.requests.len(), 232);
    assert_eq!(summary.epoch_count, 28);
    assert_eq!(summary.epochs.len(), 28);
    assert!(summary
        .epochs
        .iter()
        .all(|epoch| epoch.scale == TimeScale::Tt));
    assert_eq!(summary.epochs[0].julian_day.days(), 2_268_932.5);
    assert_eq!(summary.body_count, comparison_bodies().len());
    assert!(corpus
        .requests
        .iter()
        .all(|request| request.instant.scale == TimeScale::Tt));
    assert!(corpus.requests.iter().all(|request| matches!(
        request.body,
        CelestialBody::Sun
            | CelestialBody::Moon
            | CelestialBody::Mercury
            | CelestialBody::Venus
            | CelestialBody::Mars
            | CelestialBody::Jupiter
            | CelestialBody::Saturn
            | CelestialBody::Uranus
            | CelestialBody::Neptune
            | CelestialBody::Pluto
    )));
    assert!(corpus
        .requests
        .iter()
        .any(|request| request.instant.julian_day.days() == 2_360_233.5));
    assert!(corpus
        .requests
        .iter()
        .any(|request| request.instant.julian_day.days() == 2_451_545.0));
    assert!(corpus
        .requests
        .iter()
        .any(|request| request.instant.julian_day.days() == 2_634_167.0));
    assert_eq!(corpus.requests[0].frame, CoordinateFrame::Ecliptic);
    assert_eq!(corpus.apparentness, Apparentness::Mean);
    assert_eq!(corpus.requests[0].apparent, Apparentness::Mean);
}

#[test]
fn release_grade_corpus_excludes_pluto_from_tolerance_evidence() {
    let corpus = release_grade_corpus();
    let summary = corpus.summary();
    assert!(corpus
        .requests
        .iter()
        .all(|request| request.body != CelestialBody::Pluto));
    assert!(summary.body_count < default_corpus().summary().body_count);
    assert!(summary.request_count < default_corpus().summary().request_count);
    assert!(corpus.name.contains("release-grade comparison window"));
    assert!(corpus
        .description
        .contains("Pluto excluded from tolerance evidence"));
    assert!(!corpus
        .summary()
        .epochs
        .iter()
        .any(|epoch| epoch.julian_day.days() == 2_451_913.5));
}

#[test]
fn comparison_report_uses_the_snapshot_backend() {
    let report = render_comparison_report().expect("comparison should render");
    assert_eq!(report.lines().next(), Some("Comparison report"));
    assert!(report.lines().any(|line| line == "Comparison corpus"));
    assert!(report
        .lines()
        .any(|line| line == "  name: JPL Horizons comparison window"));
    assert!(report.lines().any(|line| {
            line == "  description: Source-backed comparison corpus built from the checked-in JPL Horizons snapshot across a small set of reference epochs, restricted to the bodies shared by the algorithmic comparison backend."
        }));
    assert!(report.lines().any(|line| line == "  Apparentness: Mean"));
    assert!(report.lines().any(|line| {
        line.starts_with("  epoch labels: JD 2268932.5 (TT)")
            && line.contains("JD 2451545.0 (TT)")
            && line.contains("JD 2634167.0 (TT)")
    }));
    assert!(report
        .lines()
        .any(|line| line == "  julian day span: 2268932.5 → 2634167.0"));
    assert!(report
        .lines()
        .any(|line| line == "Reference backend: jpl-snapshot"));
    assert!(report
        .lines()
        .any(|line| line == "Candidate backend: composite:pleiades-vsop87+pleiades-elp"));
}

#[test]
fn request_surface_summary_validation_matches_the_report_line() {
    let summary = RequestSurfaceSummary::current();
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_chart_help_clause(),
        Ok(summary.chart_help_clause())
    );
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), request_surface_summary_for_report());
    assert_eq!(
        render_cli(&["request-surface"]).unwrap(),
        render_request_surface_summary_text()
    );
    assert_eq!(
            summary.summary_line(),
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        );
    assert_eq!(summary.summary_line().lines().count(), 1);
}

#[test]
fn request_surface_summary_validated_summary_line_rejects_drift() {
    let summary = RequestSurfaceSummary {
        instant: "",
        chart_request: "",
        backend_request: "",
        house_request: "",
        request_policy: "",
        cli_chart: "",
    };

    let error = summary
        .validated_chart_help_clause()
        .expect_err("summary should reject blank request-surface labels");
    assert!(error
        .to_string()
        .contains("primary request surface instant mismatch"));

    let error = summary
        .validated_summary_line()
        .expect_err("summary should reject blank request-surface labels");
    assert!(error
        .to_string()
        .contains("primary request surface instant mismatch"));
}

#[test]
fn request_surface_summary_alias_rejects_extra_arguments_with_alias_specific_diagnostics() {
    let summary_error = render_cli(&["request-surface-summary", "extra"])
        .expect_err("request surface summary should reject extra arguments");
    assert_eq!(
        summary_error,
        "request-surface-summary does not accept extra arguments"
    );

    let error = render_cli(&["request-surface", "extra"])
        .expect_err("request surface alias should reject extra arguments");
    assert_eq!(error, "request-surface does not accept extra arguments");
}

#[test]
fn request_policy_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_request_policy_summary_text();
    stale.push_str(" stale");

    let error = ensure_request_policy_summary_matches_current_rendering(&stale)
        .expect_err("stale request policy summary should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("request policy summary no longer matches"));
}

#[test]
fn request_semantics_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_request_semantics_summary_text();
    stale.push_str(" stale");

    let error = ensure_request_semantics_summary_matches_current_rendering(&stale).expect_err(
        "stale request-semantics summary should fail the release-bundle semantic check",
    );
    assert!(error
        .to_string()
        .contains("request-semantics summary no longer matches"));
}

#[test]
fn request_surface_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_request_surface_summary_text();
    stale.push_str(" stale");

    let error = ensure_request_surface_summary_matches_current_rendering(&stale)
        .expect_err("stale request surface summary should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("request surface summary no longer matches"));
}

#[test]
fn backend_matrix_report_semantic_check_rejects_stale_rendering() {
    let mut stale = render_backend_matrix_report().expect("backend matrix should render");
    stale.push_str(" stale");

    let error = ensure_backend_matrix_report_matches_current_rendering(&stale)
        .expect_err("stale backend matrix report should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("backend matrix no longer matches"));
}

#[test]
fn backend_matrix_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_backend_matrix_summary();
    stale.push_str(" stale");

    let error = ensure_backend_matrix_summary_matches_current_rendering(&stale)
        .expect_err("stale backend matrix summary should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("backend matrix summary no longer matches"));
}

#[test]
fn comparison_tolerance_policy_summary_matches_the_rendered_line() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let summary = report.tolerance_policy_summary();

    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(
        summary.summary_line(),
        format_comparison_tolerance_policy_for_report(&report)
    );
    assert_eq!(
        summary
            .validated_summary_line()
            .expect("summary should validate"),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains("frames=Ecliptic"));
    assert_eq!(summary.coverage.len(), summary.entries.len());
    assert_eq!(summary.comparison_body_count, report.body_summaries().len());
    assert!(summary.coverage.iter().any(|coverage| coverage.entry.scope
        == ComparisonToleranceScope::Pluto
        && coverage.body_count == 0
        && coverage.sample_count == 0));
    assert!(summary.coverage.iter().all(|coverage| coverage.entry.scope
        != ComparisonToleranceScope::Pluto
        || coverage.bodies.is_empty()));
    assert_eq!(summary.comparison_sample_count, report.summary.sample_count);
    assert_eq!(
        summary.comparison_window.start,
        corpus.summary().epochs.first().copied()
    );
    assert_eq!(
        summary.comparison_window.end,
        corpus.summary().epochs.last().copied()
    );
}

#[test]
fn comparison_tolerance_policy_summary_validated_summary_line_rejects_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summary = report.tolerance_policy_summary();
    summary.comparison_sample_count += 1;

    let error = summary
        .validated_summary_line()
        .expect_err("summary should reject drifted counts");
    assert!(error.to_string().contains("sample-count mismatch"));
}

#[test]
fn comparison_tolerance_policy_summary_renderer_falls_back_when_the_report_fails() {
    let rendered = render_comparison_tolerance_policy_summary_text_from_report(Err(
        "comparison report construction failed".to_string(),
    ));

    assert_eq!(
            rendered,
            "Comparison tolerance policy summary\nComparison tolerance policy unavailable (comparison report construction failed)\n"
        );
}

#[test]
fn pluto_fallback_summary_renderer_falls_back_when_the_report_fails() {
    let rendered = render_pluto_fallback_summary_text_from_report(Err(
        "comparison report construction failed".to_string(),
    ));

    assert_eq!(
            rendered,
            "Pluto fallback summary\nPluto fallback unavailable (comparison report construction failed)\n"
        );
}

#[test]
fn comparison_tolerance_scope_coverage_summary_and_alias_commands_render_the_scope_coverage() {
    let summary = render_cli(&["comparison-tolerance-scope-coverage-summary"])
        .expect("comparison tolerance scope coverage summary should render");
    assert!(summary.contains("Comparison tolerance scope coverage summary"));
    assert!(summary.contains("Scope coverage posture:"));
    assert_eq!(
        summary,
        render_comparison_tolerance_scope_coverage_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage"])
            .expect("comparison tolerance scope coverage alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage-summary", "extra"]).expect_err(
            "comparison tolerance scope coverage summary should reject extra arguments"
        ),
        "comparison-tolerance-scope-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage", "extra"])
            .expect_err("comparison tolerance scope coverage alias should reject extra arguments"),
        "comparison-tolerance-scope-coverage does not accept extra arguments"
    );
}

#[test]
fn comparison_tolerance_scope_coverage_summary_renderer_fails_closed_on_invalid_rows() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summary = report.tolerance_policy_summary();
    summary.coverage[0].body_count = summary.coverage[0].bodies.len() + 1;

    let rendered =
        render_comparison_tolerance_scope_coverage_summary_text_from_summary(Ok(summary));

    assert!(rendered.contains("Comparison tolerance scope coverage summary"));
    assert!(rendered.contains("Comparison tolerance scope coverage unavailable"));
    assert!(rendered.contains("body-count mismatch"));
}

#[test]
fn comparison_body_class_tolerance_summary_and_alias_commands_render_the_posture() {
    let summary = render_cli(&["comparison-body-class-tolerance-summary"])
        .expect("comparison body-class tolerance summary should render");
    assert!(summary.contains("Comparison body-class tolerance summary"));
    assert_eq!(
        summary,
        render_comparison_body_class_tolerance_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance"])
            .expect("comparison body-class tolerance alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-summary", "extra"])
            .expect_err("comparison body-class tolerance summary should reject extra arguments"),
        "comparison-body-class-tolerance-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance", "extra"])
            .expect_err("comparison body-class tolerance alias should reject extra arguments"),
        "comparison-body-class-tolerance does not accept extra arguments"
    );

    let posture = render_cli(&["comparison-body-class-tolerance-posture-summary"])
        .expect("comparison body-class tolerance posture summary should render");
    assert!(posture.contains("Comparison body-class tolerance posture summary"));
    assert_eq!(
        posture,
        render_comparison_body_class_tolerance_posture_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-posture"])
            .expect("comparison body-class tolerance posture alias should render"),
        posture
    );
    assert_eq!(
        validated_comparison_body_class_tolerance_posture_for_report()
            .expect("comparison body-class tolerance posture helper should validate"),
        format_body_class_tolerance_posture_for_report()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-posture-summary", "extra"]).expect_err(
            "comparison body-class tolerance posture summary should reject extra arguments"
        ),
        "comparison-body-class-tolerance-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-posture", "extra"]).expect_err(
            "comparison body-class tolerance posture alias should reject extra arguments"
        ),
        "comparison-body-class-tolerance-posture does not accept extra arguments"
    );
}

#[test]
fn comparison_body_class_error_envelope_summary_and_alias_commands_render_the_envelopes() {
    let summary = render_cli(&["comparison-body-class-error-envelope-summary"])
        .expect("comparison body-class error envelope summary should render");
    assert!(summary.contains("Comparison body-class error envelope summary"));
    assert!(summary.contains("Body-class error envelopes:"));
    assert!(summary.contains("Luminaries"));
    assert_eq!(
        summary,
        render_comparison_body_class_error_envelope_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-error-envelope"])
            .expect("comparison body-class error envelope alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["comparison-body-class-error-envelope-summary", "extra"]).expect_err(
            "comparison body-class error envelope summary should reject extra arguments"
        ),
        "comparison-body-class-error-envelope-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-body-class-error-envelope", "extra"])
            .expect_err("comparison body-class error envelope alias should reject extra arguments"),
        "comparison-body-class-error-envelope does not accept extra arguments"
    );
}

#[test]
fn comparison_body_class_error_envelope_summary_renderer_fails_closed_on_invalid_rows() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summaries = report.body_class_summaries();
    summaries[0].sample_count = 0;

    let rendered =
        render_comparison_body_class_error_envelope_summary_text_from_summaries(Ok(summaries));

    assert!(rendered.contains("Comparison body-class error envelope summary"));
    assert!(rendered.contains("Comparison body-class error envelope unavailable"));
    assert!(rendered.contains("body-class summary is unavailable"));
}

#[test]
fn comparison_body_class_tolerance_summary_renderer_fails_closed_on_invalid_rows() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summaries = report.body_class_tolerance_summaries();
    summaries[0].body_count += 1;

    let rendered =
        render_comparison_body_class_tolerance_summary_text_from_summaries(Ok(summaries));

    assert!(rendered.contains("Comparison body-class tolerance summary"));
    assert!(rendered.contains("Comparison body-class tolerance unavailable"));
    assert!(rendered.contains("body-class tolerance summary body-count mismatch"));
}

#[test]
fn comparison_tolerance_catalog_entries_track_the_backend_family_and_scopes() {
    let entries = comparison_tolerance_catalog_entries(&BackendFamily::Algorithmic);

    assert_eq!(entries.len(), 6);
    assert_eq!(entries[0].scope, ComparisonToleranceScope::Luminary);
    assert_eq!(entries[1].scope, ComparisonToleranceScope::MajorPlanet);
    assert_eq!(entries[2].scope, ComparisonToleranceScope::LunarPoint);
    assert_eq!(entries[3].scope, ComparisonToleranceScope::Asteroid);
    assert_eq!(entries[4].scope, ComparisonToleranceScope::Custom);
    assert_eq!(entries[5].scope, ComparisonToleranceScope::Pluto);
    assert!(entries
        .iter()
        .all(|entry| entry.tolerance.backend_family == BackendFamily::Algorithmic));
}

#[test]
fn comparison_tolerance_catalog_entries_use_body_class_specific_limits() {
    let entries = comparison_tolerance_catalog_entries(&BackendFamily::Composite);

    assert_eq!(
        entries[0].summary_line(),
        "Luminaries: Δlon≤7.500°, Δlat≤0.750°, Δdist=0.001 AU"
    );
    assert_eq!(
        entries[1].summary_line(),
        "Major planets: Δlon≤0.010°, Δlat≤0.010°, Δdist=0.001 AU"
    );
    assert_eq!(
        entries[2].summary_line(),
        "Lunar points: Δlon≤0.100°, Δlat≤0.010°, Δdist=0.001 AU"
    );
    assert_eq!(
        entries[5].summary_line(),
        "Pluto fallback (approximate): Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU"
    );
}

#[test]
fn comparison_tolerance_scope_coverage_summary_validated_summary_line_rejects_body_count_drift() {
    let summary = ComparisonToleranceScopeCoverageSummary {
        entry: ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        },
        bodies: vec![CelestialBody::Sun],
        body_count: 2,
        sample_count: 1,
    };

    let error = summary
        .validated_summary_line()
        .expect_err("summary should reject body-count drift");
    assert!(error.to_string().contains("body-count mismatch"));
}

#[test]
fn comparison_tolerance_entry_has_a_validated_summary_line() {
    let entry = ComparisonToleranceEntry {
        scope: ComparisonToleranceScope::Luminary,
        tolerance: ComparisonTolerance {
            backend_family: BackendFamily::Algorithmic,
            profile: "test tolerance",
            max_longitude_delta_deg: 0.1,
            max_latitude_delta_deg: 0.2,
            max_distance_delta_au: Some(0.3),
        },
    };

    assert_eq!(entry.summary_line(), entry.to_string());
    assert_eq!(entry.validated_summary_line(), Ok(entry.summary_line()));
}

#[test]
fn comparison_tolerance_validation_rejects_blank_profile() {
    let error = validate_comparison_tolerance(&ComparisonTolerance {
        backend_family: BackendFamily::Algorithmic,
        profile: "",
        max_longitude_delta_deg: 0.1,
        max_latitude_delta_deg: 0.2,
        max_distance_delta_au: Some(0.3),
    })
    .expect_err("tolerance should reject a blank profile label");
    assert!(error.to_string().contains("must not be blank"));
}

#[test]
fn comparison_tolerance_scope_coverage_summary_has_a_validated_summary_line() {
    let summary = ComparisonToleranceScopeCoverageSummary {
        entry: ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        },
        bodies: vec![CelestialBody::Sun],
        body_count: 1,
        sample_count: 1,
    };

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn comparison_tolerance_validation_rejects_padded_profile() {
    let error = validate_comparison_tolerance(&ComparisonTolerance {
        backend_family: BackendFamily::Algorithmic,
        profile: " test tolerance ",
        max_longitude_delta_deg: 0.1,
        max_latitude_delta_deg: 0.2,
        max_distance_delta_au: Some(0.3),
    })
    .expect_err("tolerance should reject a padded profile label");
    assert!(error
        .to_string()
        .contains("contains surrounding whitespace"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_drift() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        }],
        coverage: vec![ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 1,
            sample_count: 1,
        }],
        comparison_body_count: 2,
        comparison_sample_count: 1,
        comparison_window: TimeRange::new(None, None),
        coordinate_frames: vec![CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject body-count drift");
    assert!(error.to_string().contains("body-count mismatch"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_invalid_comparison_window() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        }],
        coverage: vec![ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 1,
            sample_count: 1,
        }],
        comparison_body_count: 1,
        comparison_sample_count: 1,
        comparison_window: TimeRange::new(
            Some(Instant::new(
                JulianDay::from_days(2_451_546.0),
                TimeScale::Tt,
            )),
            Some(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            )),
        ),
        coordinate_frames: vec![CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject an invalid comparison window");
    assert!(error.to_string().contains("invalid comparison window"));
    assert!(error.to_string().contains("must not precede the start"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_duplicate_coordinate_frames() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        }],
        coverage: vec![ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 1,
            sample_count: 1,
        }],
        comparison_body_count: 1,
        comparison_sample_count: 1,
        comparison_window: TimeRange::new(None, None),
        coordinate_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject duplicate coordinate frames");
    assert!(error.to_string().contains("duplicate coordinate frame"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_duplicate_bodies_across_scopes() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![
            ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::MajorPlanet,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.4,
                    max_latitude_delta_deg: 0.5,
                    max_distance_delta_au: Some(0.6),
                },
            },
        ],
        coverage: vec![
            ComparisonToleranceScopeCoverageSummary {
                entry: ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::Luminary,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.1,
                        max_latitude_delta_deg: 0.2,
                        max_distance_delta_au: Some(0.3),
                    },
                },
                bodies: vec![CelestialBody::Sun],
                body_count: 1,
                sample_count: 1,
            },
            ComparisonToleranceScopeCoverageSummary {
                entry: ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::MajorPlanet,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.4,
                        max_latitude_delta_deg: 0.5,
                        max_distance_delta_au: Some(0.6),
                    },
                },
                bodies: vec![CelestialBody::Sun],
                body_count: 1,
                sample_count: 1,
            },
        ],
        comparison_body_count: 2,
        comparison_sample_count: 2,
        comparison_window: TimeRange::new(None, None),
        coordinate_frames: vec![CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject duplicate bodies across scopes");
    assert!(error.to_string().contains("appears in multiple scope rows"));
    assert!(error.to_string().contains("Luminaries"));
    assert!(error.to_string().contains("Major planets"));
}

#[test]
fn comparison_tolerance_entry_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let summary = report.tolerance_policy_summary();
    let entry = summary
        .entries
        .first()
        .expect("comparison should include at least one tolerance entry");

    assert_eq!(entry.summary_line(), entry.to_string());
    entry
        .validate()
        .expect("reported tolerance entry should validate");
    assert!(entry.summary_line().contains(entry.scope.label()));
    assert!(entry.summary_line().contains("Δlon≤"));
    assert!(entry.summary_line().contains("Δlat≤"));
}

#[test]
fn comparison_tolerance_scope_coverage_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let summary = report.tolerance_policy_summary();
    let coverage = summary
        .coverage
        .first()
        .expect("comparison should include at least one tolerance coverage row");

    assert_eq!(coverage.summary_line(), coverage.to_string());
    assert!(coverage.summary_line().contains("backend family="));
    assert!(coverage
        .summary_line()
        .contains(coverage.entry.scope.label()));
    assert_eq!(coverage.body_count, coverage.bodies.len());
}

#[test]
fn body_tolerance_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary");

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains("backend family="));
    assert!(summary.summary_line().contains("status="));
}

#[test]
fn body_tolerance_summary_validate_accepts_the_reported_summary() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary");

    summary
        .validate()
        .expect("reported body tolerance summary should validate");
}

#[test]
fn body_tolerance_summary_validate_rejects_margin_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.longitude_margin_deg += 1.0;
    let error = summary
        .validate()
        .expect_err("mutated body tolerance summary should fail validation");
    assert!(error.to_string().contains("longitude margin"));
}

#[test]
fn body_tolerance_summary_validate_rejects_zero_sample_counts() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.sample_count = 0;
    let error = summary
        .validate()
        .expect_err("zero-sample body tolerance summary should fail validation");
    assert!(error.to_string().contains("has no samples to compare"));
}

#[test]
fn body_tolerance_summary_validated_summary_line_rejects_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.sample_count = 0;
    let error = summary
        .validated_summary_line()
        .expect_err("zero-sample body tolerance summary should fail validation");
    assert!(error.to_string().contains("has no samples to compare"));
}

#[test]
fn body_tolerance_summary_validate_rejects_non_finite_metrics() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.max_latitude_delta_deg = f64::NAN;
    let error = summary
        .validate()
        .expect_err("non-finite body tolerance summary should fail validation");
    assert!(error.to_string().contains("has invalid latitude"));
}

#[test]
fn body_comparison_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let body_summaries = report.body_summaries();
    assert!(!corpus
        .summary()
        .epochs
        .iter()
        .any(|epoch| epoch.julian_day.days() == 2_451_913.5));
    let summary = body_summaries
        .first()
        .expect("comparison should include at least one body summary");

    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary.summary_line().contains("samples="));
    assert!(summary.summary_line().contains("max Δlon="));
}

#[test]
fn body_comparison_summary_validate_accepts_the_reported_body_summary() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let body_summaries = report.body_summaries();
    let summary = body_summaries
        .first()
        .expect("comparison should include at least one body summary");

    summary
        .validate()
        .expect("reported body summary should validate");
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn body_comparison_summary_validate_rejects_inconsistent_distance_fields() {
    let summary = BodyComparisonSummary {
        body: CelestialBody::Sun,
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.1,
        mean_longitude_delta_deg: 0.1,
        rms_longitude_delta_deg: 0.1,
        max_latitude_delta_body: Some(CelestialBody::Sun),
        max_latitude_delta_deg: 0.1,
        mean_latitude_delta_deg: 0.1,
        rms_latitude_delta_deg: 0.1,
        max_distance_delta_body: Some(CelestialBody::Sun),
        max_distance_delta_au: None,
        mean_distance_delta_au: None,
        rms_distance_delta_au: None,
    };

    let error = summary
        .validate()
        .expect_err("mismatched distance fields should fail");
    assert!(error
        .to_string()
        .contains("distance metrics must either all be present or all be absent"));
}

#[test]
fn comparison_percentile_envelope_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope = comparison_percentile_envelope(&report.samples, 0.95);

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert!(envelope
        .summary_line()
        .contains("95th percentile absolute deltas:"));
    assert!(envelope.summary_line().contains("longitude"));
    assert!(envelope.summary_line().contains("latitude"));
}

#[test]
fn comparison_median_envelope_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope =
        comparison_median_envelope(&report.samples).expect("median envelope should exist");

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert!(envelope.summary_line().contains("median longitude delta:"));
    assert!(envelope.summary_line().contains("median latitude delta:"));
}

#[test]
fn comparison_envelope_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope = comparison_envelope_summary(&report.summary, &report.samples);

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert_eq!(
        envelope.validated_summary_line(&report.samples),
        Ok(envelope.summary_line())
    );
    assert_eq!(
        envelope.percentile_line(),
        comparison_percentile_envelope(&report.samples, 0.95).summary_line()
    );
    assert_eq!(
        envelope.validated_percentile_line(&report.samples),
        Ok(envelope.percentile_line())
    );
    assert!(envelope.summary_line().contains("median longitude delta:"));
    assert!(envelope
        .percentile_line()
        .contains("95th percentile absolute deltas:"));
}

#[test]
fn comparison_envelope_summary_rejects_median_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut envelope = comparison_envelope_summary(&report.summary, &report.samples);
    envelope.median.longitude_delta_deg += 0.0001;

    let error = envelope
        .validated_summary_line(&report.samples)
        .expect_err("drifted median should fail validation");
    assert!(error
        .to_string()
        .contains("median drifted from the sampled comparison values"));
}

#[test]
fn comparison_envelope_summary_rejects_percentile_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut envelope = comparison_envelope_summary(&report.summary, &report.samples);
    envelope.percentile.longitude_delta_deg += 0.0001;

    let error = envelope
        .validated_percentile_line(&report.samples)
        .expect_err("drifted percentile should fail validation");
    assert!(error
        .to_string()
        .contains("percentile drifted from the sampled comparison values"));
}

#[test]
fn comparison_median_and_percentile_envelopes_validate_their_fields() {
    let median = ComparisonMedianEnvelope {
        longitude_delta_deg: 1.25,
        latitude_delta_deg: 0.75,
        distance_delta_au: Some(0.03125),
    };
    let percentile = ComparisonPercentileEnvelope {
        longitude_delta_deg: 2.5,
        latitude_delta_deg: 1.5,
        distance_delta_au: Some(0.0625),
    };

    assert_eq!(median.validate(), Ok(()));
    assert_eq!(percentile.validate(), Ok(()));
    assert_eq!(median.summary_line(), median.to_string());
    assert_eq!(percentile.summary_line(), percentile.to_string());

    let invalid_median = ComparisonMedianEnvelope {
        longitude_delta_deg: -1.0,
        ..median
    };
    let median_error = invalid_median
        .validate()
        .expect_err("negative median deltas should fail validation");
    assert!(median_error
        .to_string()
        .contains("comparison median envelope field `longitude_delta_deg`"));

    let invalid_percentile = ComparisonPercentileEnvelope {
        distance_delta_au: Some(-0.5),
        ..percentile
    };
    let percentile_error = invalid_percentile
        .validate()
        .expect_err("negative percentile deltas should fail validation");
    assert!(percentile_error
        .to_string()
        .contains("comparison percentile envelope field `distance_delta_au`"));
}

#[test]
fn comparison_tail_envelope_is_publicly_reusable() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope = comparison_tail_envelope(&report.samples).expect("tail envelope should exist");

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert_eq!(
        envelope.summary_line(),
        format_comparison_percentile_envelope_for_report(&report.samples)
    );
}

#[test]
fn regression_finding_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let notable_regressions = report.notable_regressions();
    let finding = notable_regressions
        .first()
        .expect("comparison should include at least one notable regression");

    assert_eq!(finding.summary_line(), finding.to_string());
    assert_eq!(finding.validated_summary_line(), Ok(finding.summary_line()));
    assert!(finding.summary_line().contains("Δlon="));
    assert!(finding.summary_line().contains("Δlat="));
    assert!(finding.summary_line().contains("Δdist="));
}

#[test]
fn regression_finding_validated_summary_line_rejects_blank_notes() {
    let finding = RegressionFinding {
        body: CelestialBody::Mars,
        longitude_delta_deg: 0.25,
        latitude_delta_deg: 0.15,
        distance_delta_au: Some(0.01),
        note: "  ".to_string(),
    };

    let error = finding
        .validated_summary_line()
        .expect_err("blank regression notes should fail validation");
    assert!(error
        .to_string()
        .contains("regression finding note must not be blank"));
}

#[test]
fn comparison_audit_summary_has_a_displayable_summary_line() {
    let summary = ComparisonAuditSummary {
        body_count: 10,
        within_tolerance_body_count: 4,
        outside_tolerance_body_count: 6,
        regression_count: 12,
    };

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
            summary.summary_line(),
            "status=regressions found, bodies checked=10, within tolerance bodies=4, outside tolerance bodies=6, notable regressions=12"
        );
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn comparison_audit_summary_validate_rejects_body_count_mismatch() {
    let summary = ComparisonAuditSummary {
        body_count: 10,
        within_tolerance_body_count: 4,
        outside_tolerance_body_count: 5,
        regression_count: 0,
    };

    let error = summary
        .validated_summary_line()
        .expect_err("mismatched audit counts should fail validation");
    assert!(error
        .to_string()
        .contains("comparison audit summary body-count mismatch"));
}

#[test]
fn comparison_audit_summary_validate_rejects_empty_body_counts() {
    let summary = ComparisonAuditSummary {
        body_count: 0,
        within_tolerance_body_count: 0,
        outside_tolerance_body_count: 0,
        regression_count: 0,
    };

    let error = summary
        .validate()
        .expect_err("an empty audit summary should fail validation");
    assert!(error
        .to_string()
        .contains("must include at least one compared body"));
}

#[test]
fn comparison_report_exposes_a_public_audit_summary() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison report should build");
    let summary = report.comparison_audit_summary();

    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary.validate().is_ok());
    assert_eq!(summary.body_count, report.tolerance_summaries().len());
    assert_eq!(summary.regression_count, report.notable_regressions().len());
}

#[test]
fn comparison_report_alias_renders_the_comparison_report() {
    let alias = render_cli(&["comparison-report"]).expect("comparison report should render");
    let command = render_cli(&["compare-backends"]).expect("compare-backends should render");

    assert_eq!(alias, command);
    assert!(alias.contains("Comparison report"));
    assert_eq!(
        render_cli(&["comparison-report", "extra"]).unwrap_err(),
        "comparison-report does not accept extra arguments"
    );
}

#[test]
fn comparison_audit_command_reports_clean_release_grade_corpus() {
    let report = render_cli(&["compare-backends-audit"]).expect("comparison audit should render");
    assert!(report.contains("Comparison tolerance audit"));
    assert!(report.contains("comparison corpus"));
    assert!(report.contains("julian day span:"));
    assert!(report.contains("Body-class error envelopes"));
    assert!(report.contains("rms longitude delta:"));
    assert!(report.contains("rms latitude delta:"));
    assert!(report.contains("rms distance delta:"));
    assert!(report.contains("Body-class tolerance posture"));
    assert!(report.contains("body-class tolerance posture:"));
    assert!(report.contains("outlier bodies: none"));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("Notable regressions\n  none"));
    assert!(report.contains("regression bodies: none"));
    assert!(report.contains("Pluto fallback (approximate): backend family=composite, profile=phase-1 Pluto approximate fallback evidence, bodies=0 (none), samples=0"));

    let summary =
        render_cli(&["comparison-audit-summary"]).expect("comparison audit summary should render");
    assert!(summary.contains("status="));
    assert!(summary.contains("bodies checked="));
    assert!(summary.contains("within tolerance bodies="));
    assert!(summary.contains("outside tolerance bodies="));
    assert!(summary.contains("notable regressions="));
    assert_eq!(
        summary,
        render_cli(&["comparison-audit"]).expect("comparison audit alias should render")
    );
    assert_eq!(
        render_cli(&["comparison-audit-summary", "extra"]).unwrap_err(),
        "comparison-audit-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-audit", "extra"]).unwrap_err(),
        "comparison-audit does not accept extra arguments"
    );
}

#[test]
fn benchmark_report_renders_a_time_summary() {
    let report = render_benchmark_report(10).expect("benchmark should render");
    let provenance = workspace_provenance();
    assert_eq!(provenance.validate(), Ok(()));
    assert!(report.contains(&provenance.summary_line()));
    assert!(report.contains("Benchmark report"));
    assert!(report.contains("Summary: backend="));
    assert!(report.contains("Representative 1500-2500 window"));
    assert!(report.contains("Apparentness: Mean"));
    assert!(report.contains("Methodology:"));
    assert!(report.contains("single-request and batch paths are measured separately"));
    assert!(report.contains("chart assembly is measured end to end"));
    assert!(report.contains("Single-request elapsed:"));
    assert!(report.contains("Batch elapsed:"));
    assert!(report.contains("Estimated corpus heap footprint:"));
    assert!(report.contains("Nanoseconds per request (single):"));
    assert!(report.contains("Nanoseconds per request (batch):"));
    assert!(report.contains("Batch throughput:"));
    assert!(report.contains("Artifact lookup benchmark report"));
    assert!(report.contains("Encoded bytes:"));
    assert!(report.contains("Nanoseconds per lookup:"));
    assert!(report.contains("Lookups per second:"));
    assert!(report.contains("Artifact decode benchmark report"));
    assert!(report.contains("Nanoseconds per decode:"));
    assert!(report.contains("Decodes per second:"));
    assert!(report.contains("Chart benchmark report"));
    assert!(report.contains("Summary: backend="));
    assert!(report.contains("Representative chart validation scenarios"));
    assert!(report.contains("Chart elapsed:"));
    assert!(report.contains("Nanoseconds per chart:"));
    assert!(report.contains("Charts per second:"));
}

#[test]
fn benchmark_report_summary_line_mentions_the_backend_and_throughput() {
    let corpus = benchmark_corpus();
    let backend = default_candidate_backend();
    let report =
        benchmark_backend(&backend, &corpus, 1).expect("benchmark should produce a report");
    let summary = report.summary_line();
    assert_eq!(report.validated_summary_line(), Ok(summary.clone()));
    assert!(summary.contains("backend="));
    assert!(summary.contains("corpus="));
    assert!(summary.contains("apparentness="));
    assert!(summary.contains("samples per round="));
    assert!(summary.contains("single ns/request="));
    assert!(summary.contains("batch ns/request="));
    assert!(summary.contains("batch throughput="));
    assert!(summary.contains("estimated corpus heap footprint="));
}

#[test]
fn benchmark_matrix_summary_command_renders_the_matrix_block() {
    let rendered = render_cli(&["benchmark-matrix-summary", "--rounds", "1"])
        .expect("benchmark matrix summary should render");
    assert!(rendered.contains("Benchmark matrix summary"));
    assert!(rendered.contains("Benchmark corpora"));
    assert!(rendered.contains("comparison corpus: corpus name="));
    assert!(rendered.contains("benchmark corpus: corpus name="));
    assert!(rendered.contains("packaged-data benchmark corpus: corpus name="));
    assert!(rendered.contains("chart benchmark corpus: corpus name="));
    assert!(rendered.contains("Benchmark rows"));
    assert!(rendered.contains("reference benchmark: backend="));
    assert!(rendered.contains("candidate benchmark: backend="));
    assert!(rendered.contains("packaged-data benchmark: backend="));
    assert!(rendered.contains("chart benchmark: backend="));
    assert!(rendered.contains("artifact decode benchmark: artifact="));
    assert!(rendered.contains("packaged-artifact size: "));
    assert!(rendered.contains("Packaged-artifact fit posture"));
    assert!(rendered.contains("fit envelope: "));
    assert!(rendered.contains("fit margins: "));
    assert!(rendered.contains("fit threshold violation count: 0"));
    assert!(rendered.contains("fit threshold violations: 0; details: none"));
    assert!(rendered.contains("fit sample classes: boundary continuity="));
    assert!(rendered.contains("fit thresholds: mean Δlon≤"));
    assert!(rendered.contains("target thresholds: profile id="));
    assert!(rendered.contains("target-threshold scope envelopes: scope=luminaries;"));
    assert!(render_benchmark_matrix_summary(1)
        .expect("benchmark matrix summary should build")
        .contains("Packaged-artifact fit posture"));

    let alias_rendered = render_cli(&["benchmark-matrix", "--rounds", "1"])
        .expect("benchmark matrix alias should render");
    assert!(alias_rendered.contains("Benchmark matrix summary"));
    assert!(alias_rendered.contains("Benchmark corpora"));
    assert!(alias_rendered.contains("Benchmark rows"));
    assert!(alias_rendered.contains("Packaged-artifact fit posture"));
}

#[test]
fn benchmark_backend_rejects_zero_rounds() {
    let corpus = benchmark_corpus();
    let backend = default_candidate_backend();

    let error = benchmark_backend(&backend, &corpus, 0)
        .expect_err("zero-round benchmarks should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("benchmark rounds must be greater than zero"));
}

#[test]
fn benchmark_chart_backend_rejects_zero_rounds() {
    let error = benchmark_chart_backend(default_candidate_backend(), 0)
        .expect_err("zero-round chart benchmarks should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("chart benchmark rounds must be greater than zero"));
}

#[test]
fn benchmark_backend_rejects_zero_heap_footprint() {
    let corpus = benchmark_corpus();
    let backend = default_candidate_backend();
    let mut report =
        benchmark_backend(&backend, &corpus, 1).expect("benchmark should produce a report");
    report.estimated_corpus_heap_bytes = 0;

    let error = report
        .validate()
        .expect_err("zero-heap benchmarks should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("benchmark estimated corpus heap footprint must be greater than zero"));
}

#[test]
fn benchmark_workspace_provenance_display_matches_the_summary_helper() {
    let provenance = workspace_provenance();
    let expected = format!(
            "Benchmark provenance\n  source revision: {}\n  workspace status: {}\n  rustc version: {}\n  cargo version: {}\n  rustfmt version: {}\n  clippy version: {}",
            provenance.source_revision,
            provenance.workspace_status,
            provenance.rustc_version,
            provenance.cargo_version,
            provenance.rustfmt_version,
            provenance.clippy_version
        );

    assert_eq!(provenance.validate(), Ok(()));
    assert_eq!(provenance.summary_line(), expected);
    assert_eq!(provenance.to_string(), expected);
}

#[test]
fn benchmark_workspace_provenance_validation_rejects_blank_or_multiline_fields() {
    let blank_source_revision = WorkspaceProvenance {
        source_revision: String::new(),
        workspace_status: "clean".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        blank_source_revision.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "source revision"
        }
    );

    let multiline_rustc_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0\nextra".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        multiline_rustc_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "rustc version"
        }
    );

    let multiline_cargo_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0\nextra".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        multiline_cargo_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "cargo version"
        }
    );

    let blank_rustfmt_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: String::new(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        blank_rustfmt_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "rustfmt version"
        }
    );

    let multiline_clippy_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0\nextra".to_string(),
    };
    assert_eq!(
        multiline_clippy_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "clippy version"
        }
    );
}

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
fn validation_report_validate_rejects_drifted_chart_benchmark_corpus() {
    let mut report = build_validation_report(10).expect("validation report should build");
    report.chart_benchmark_corpus.name.clear();

    let error = report
        .validate()
        .expect_err("chart benchmark corpus drift should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("validation report chart benchmark corpus is invalid"));
    assert!(error
        .message
        .contains("corpus summary name must not be blank"));
}

#[test]
fn validation_report_display_rejects_drifted_regression_archive() {
    let mut report = build_validation_report(10).expect("validation report should build");
    report.archived_regressions.corpus_name.clear();

    let rendered = report.to_string();
    assert!(rendered.contains("Validation report unavailable"));
    assert!(rendered.contains("regression archive corpus name must not be blank"));
}

#[test]
fn validation_report_includes_corpus_metadata() {
    let report = render_validation_report(10).expect("validation report should render");
    let validation_report = build_validation_report(10).expect("validation report should build");
    assert!(report.contains("Validation report"));
    let release_profiles = current_release_profile_identifiers();
    assert!(report.contains("Compatibility profile"));
    assert!(report.contains(&format!(
        "  id: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(report.contains("API stability posture"));
    assert!(report.contains(&format!(
        "  id: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(report.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(report.contains("Implemented backend matrices"));
    assert!(report.contains("Selected asteroid coverage"));
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(report.contains("exact J2000 evidence: 6 bodies at JD 2451545.0"));
    assert!(report.contains("Ceres"));
    assert!(report.contains("Pallas"));
    assert!(report.contains("Juno"));
    assert!(report.contains("Vesta"));
    assert!(report.contains("asteroid:433-Eros"));
    assert!(report.contains("JPL snapshot reference backend"));
    assert!(report.contains("VSOP87 planetary backend"));
    assert!(report.contains("ELP lunar backend (Moon and lunar nodes)"));
    assert!(report.contains("Packaged data backend"));
    assert!(report.contains("Composite routed backend"));
    assert!(report.contains("Target compatibility catalog:"));
    assert!(report.contains("Comparison corpus"));
    assert!(report.contains("JPL Horizons release-grade comparison window"));
    assert!(
        report.contains("Comparison snapshot coverage: 232 rows across 10 bodies and 28 epochs")
    );
    assert!(report.contains("Apparentness: Mean"));
    assert!(report.contains("Benchmark corpus"));
    assert!(report.contains("Representative 1500-2500 window"));
    assert!(report.contains("estimated corpus heap footprint:"));
    assert!(report.contains("Chart benchmark corpus"));
    assert!(report.contains("Representative chart validation scenarios"));
    assert!(report.contains("Packaged artifact decode benchmark"));
    assert!(report.contains("Chart benchmark"));
    assert!(report.contains("House validation corpus"));
    assert!(report.contains("request: instant="));
    assert!(report.contains("Mid-latitude reference chart"));
    assert!(report.contains("Polar stress chart"));
    assert!(report.contains("Northern high-latitude stress chart"));
    assert!(report.contains("Equatorial reference chart"));
    assert!(report.contains("Southern hemisphere reference chart"));
    assert!(report.contains("Reference backend"));
    assert!(report.contains("Candidate backend"));
    assert!(report.contains("Comparison summary"));
    assert!(report.contains("95th percentile absolute deltas:"));
    assert!(report.contains(&format!(
        "max longitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_longitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_longitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max longitude body")
    )));
    assert!(report.contains(&format!(
        "max latitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_latitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_latitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max latitude body")
    )));
    assert!(report.contains(&format!(
        "max distance delta: {:.12} AU ({})",
        validation_report
            .comparison
            .summary
            .max_distance_delta_au
            .expect("comparison summary should include a max distance delta"),
        validation_report
            .comparison
            .summary
            .max_distance_delta_body
            .as_ref()
            .expect("comparison summary should include a max distance body")
    )));
    assert!(report.contains("Body-class error envelopes"));
    let body_class_envelopes = report
        .split("Body-class error envelopes")
        .nth(1)
        .expect("report should include body-class error envelopes");
    assert!(body_class_envelopes.contains("max longitude delta:"));
    assert!(body_class_envelopes.contains("median longitude delta:"));
    assert!(body_class_envelopes.contains("95th percentile longitude delta:"));
    assert!(body_class_envelopes.contains("median latitude delta:"));
    assert!(body_class_envelopes.contains("95th percentile latitude delta:"));
    assert!(body_class_envelopes.contains("rms longitude delta:"));
    assert!(body_class_envelopes.contains("rms latitude delta:"));
    assert!(body_class_envelopes.contains("median distance delta:"));
    assert!(body_class_envelopes.contains("95th percentile distance delta:"));
    assert!(body_class_envelopes.contains("rms distance delta:"));
    assert!(body_class_envelopes.contains(" ("));
    assert!(report.contains("Body-class tolerance posture"));
    let body_class_tolerance_posture = report
        .split("Body-class tolerance posture")
        .nth(1)
        .expect("report should include body-class tolerance posture");
    assert!(body_class_tolerance_posture.contains("backend family: composite"));
    assert!(body_class_tolerance_posture
        .contains("profile: phase-1 full-file VSOP87B planetary evidence"));
    assert!(body_class_tolerance_posture.contains("mean longitude delta:"));
    assert!(body_class_tolerance_posture.contains("median longitude delta:"));
    assert!(body_class_tolerance_posture.contains("95th percentile longitude delta:"));
    assert!(body_class_tolerance_posture.contains("rms longitude delta:"));
    assert!(body_class_tolerance_posture.contains("mean latitude delta:"));
    assert!(body_class_tolerance_posture.contains("median latitude delta:"));
    assert!(body_class_tolerance_posture.contains("95th percentile latitude delta:"));
    assert!(body_class_tolerance_posture.contains("rms latitude delta:"));
    assert!(body_class_tolerance_posture.contains("mean distance delta:"));
    assert!(body_class_tolerance_posture.contains("median distance delta:"));
    assert!(body_class_tolerance_posture.contains("95th percentile distance delta:"));
    assert!(body_class_tolerance_posture.contains("rms distance delta:"));
    assert!(body_class_tolerance_posture.contains("outside tolerance samples:"));
    assert!(report.contains("Expected tolerance status"));
    assert!(report.contains("margin Δlon="));
    assert!(report.contains("margin Δdist="));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("candidate backend family: composite"));
    assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
    assert!(report.contains(
            "Pluto fallback (approximate): backend family=composite, profile=phase-1 Pluto approximate fallback evidence, bodies=0 (none), samples=0"
        ));
    assert!(report.contains("Luminaries"));
    assert!(report.contains("Major planets"));
    assert!(report.contains("interpolation quality checks:"));
    assert!(report.contains("JPL interpolation quality: 293 samples across 16 bodies"));
    assert!(report.contains("JPL interpolation quality kind coverage:"));
    assert!(report.contains("JPL interpolation posture: source="));
    assert!(report.contains("JPL interpolation body-class error envelopes:"));
    assert!(report.contains("Reference/hold-out overlap:"));
    assert!(report.contains("JPL independent hold-out:"));
    assert!(report.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(report.contains("JPL independent hold-out equatorial parity:"));
    assert!(report.contains("JPL independent hold-out batch parity:"));
    assert!(report.contains("Lunar reference"));
    assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5 (TT) → JD 2459278.5 (TT)"
        ));
    assert!(report.contains(
            "lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies, TT requests=5, TDB requests=4, order=preserved, single-query parity=preserved"
        ));
    assert!(report.contains(
            "lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"
        ));
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains("exact J2000 evidence: 6 bodies at JD 2451545.0"));
    assert!(report.contains("Body comparison summaries"));
    assert!(report.contains("Sun: samples="));
    assert!(report.contains("Notable regressions"));
    assert!(report.contains("Archived regression cases"));
    assert!(report.contains("Reference benchmark"));
    assert!(report.contains("Candidate benchmark"));
    assert!(report.contains("Packaged-data benchmark corpus"));
    assert!(report.contains("Packaged-data benchmark"));
    assert!(report.contains("ns/request (single):"));
    assert!(report.contains("ns/request (batch):"));
    assert!(report.contains("batch throughput:"));
}

#[test]
fn validation_report_summary_renders_a_compact_overview() {
    let report = render_validation_report_summary(10).expect("validation summary should render");
    let validation_report = build_validation_report(10).expect("validation report should build");
    let release_profiles = current_release_profile_identifiers();
    assert!(report.contains("Validation report summary"));
    assert!(report.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(report.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(report.contains("Comparison corpus"));
    assert!(report.contains("Source corpus: comparison corpus release-grade guard:"));
    assert!(report.contains("JPL source corpus contract:"));
    assert!(report.contains("evidence classification=release-tolerance=reference/comparison/production-generation validation summaries; hold-out=independent hold-out rows and interpolation-quality summaries; fixture exactness=reference snapshot exact J2000 evidence; provenance-only=source and manifest summaries"));
    assert!(report.contains("provenance-only=source and manifest summaries are provenance-only evidence; they validate corpus provenance and checksum posture but are excluded from tolerance, hold-out, and fixture-exactness claims"));
    assert!(!report.contains("JPL source corpus contract: JPL source corpus contract:"));
    assert!(report.contains("phase-2 corpus alignment:"));
    assert_eq!(
        validated_source_corpus_summary_for_report()
            .expect("source corpus summary should validate"),
        source_corpus_summary_for_report()
    );
    assert!(validation_report
        .to_string()
        .contains("Source corpus: comparison corpus release-grade guard:"));
    assert!(report.contains(&request_surface_summary_for_report()));
    assert!(report.contains("Reference snapshot"));
    assert!(report.contains(&reference_snapshot_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report()));
    assert!(report.contains(
        &pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary_for_report()
    ));
    assert!(report.contains(
        &pleiades_jpl::reference_snapshot_2451917_major_body_bridge_summary_for_report()
    ));
    assert!(report.contains(
        &pleiades_jpl::reference_snapshot_2451917_major_body_boundary_summary_for_report()
    ));
    assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
    assert!(report.contains("release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(report.contains("epoch labels: JD 2268932.5 (TT)"));
    assert!(report.contains("House validation corpus"));
    assert!(report.contains("House validation corpus: 9 scenarios"));
    assert!(report.contains("Comparison summary"));
    assert!(report.contains("95th percentile absolute deltas:"));
    assert!(report.contains("median longitude delta:"));
    assert!(report.contains("median latitude delta:"));
    assert!(report.contains("rms longitude delta:"));
    assert!(report.contains("rms latitude delta:"));
    assert!(report.contains("rms distance delta:"));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("candidate backend family: composite"));
    assert!(report.contains("comparison window: JD"));
    assert!(report.contains("coordinate frames: Ecliptic"));
    assert!(report.contains(&format!(
        "max longitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_longitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_longitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max longitude body")
    )));
    assert!(report.contains(&format!(
        "max latitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_latitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_latitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max latitude body")
    )));
    assert!(report.contains(&format!(
        "max distance delta: {:.12} AU ({})",
        validation_report
            .comparison
            .summary
            .max_distance_delta_au
            .expect("comparison summary should include a max distance delta"),
        validation_report
            .comparison
            .summary
            .max_distance_delta_body
            .as_ref()
            .expect("comparison summary should include a max distance body")
    )));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("candidate backend family: composite"));
    assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
    assert!(report.contains("Comparison tolerance audit"));
    assert!(report.contains("command: compare-backends-audit"));
    assert!(report.contains("status: clean"));
    assert!(!report.contains("regression bodies: Pluto"));
    let body_class_tolerance_posture = report
        .split("Body-class tolerance posture")
        .nth(1)
        .expect("report should include body-class tolerance posture");
    assert!(body_class_tolerance_posture.contains("mean Δlon="));
    assert!(body_class_tolerance_posture.contains("rms Δlon="));
    assert!(body_class_tolerance_posture.contains("mean Δlat="));
    assert!(body_class_tolerance_posture.contains("rms Δlat="));
    assert!(body_class_tolerance_posture.contains("mean Δdist="));
    assert!(body_class_tolerance_posture.contains("rms Δdist="));
    assert!(report.contains("JPL interpolation quality"));
    assert!(report.contains("JPL interpolation quality: 293 samples across 16 bodies"));
    assert!(report.contains("Reference/hold-out overlap:"));
    assert!(report.contains("JPL independent hold-out:"));
    assert!(report.contains("JPL independent hold-out equatorial parity:"));
    assert!(report.contains("JPL independent hold-out batch parity:"));
    assert!(report.contains("leave-one-out runtime interpolation evidence"));
    assert!(report.contains("@ JD"));
    assert!(report.contains("ELP lunar capability: lunar capability summary:"));
    assert!(report.contains(
            "ELP lunar request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        ));
    assert!(report.contains("Lunar reference"));
    assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5 (TT) → JD 2459278.5 (TT)"
        ));
    assert!(report.contains("Lunar source windows\n  lunar source windows: 7 exact Moon samples across 1 bodies in 2 exact windows; 4 reference-only apparent Moon samples across 1 bodies in 4 apparent windows"));
    assert!(report.contains("Lunar high-curvature continuity evidence"));
    assert!(report.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(report.contains("within regression limits=true"));
    assert!(report.contains("Body comparison summaries"));
    assert!(report.contains("Body-class error envelopes"));
    assert!(report.contains("max longitude delta:"));
    assert!(report.contains("rms longitude delta:"));
    assert!(report.contains("rms latitude delta:"));
    assert!(report.contains("rms distance delta:"));
    let body_class_tolerance_posture = report
        .split("Body-class tolerance posture")
        .nth(1)
        .expect("report should include body-class tolerance posture");
    assert!(body_class_tolerance_posture.contains("mean Δlon="));
    assert!(body_class_tolerance_posture.contains("rms Δlon="));
    assert!(body_class_tolerance_posture.contains("mean Δlat="));
    assert!(body_class_tolerance_posture.contains("rms Δlat="));
    assert!(body_class_tolerance_posture.contains("mean Δdist="));
    assert!(body_class_tolerance_posture.contains("rms Δdist="));
    assert!(report.contains(" ("));
    assert!(report.contains("VSOP87 source-backed evidence"));
    assert!(report
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
    assert!(
        report.contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files")
    );
    assert!(report.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
    assert!(report.contains("VSOP87 canonical J2000 interim outliers: none"));
    assert!(report.contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
    assert!(report.contains("generated binary VSOP87B"));
    assert!(report.contains("generated binary VSOP87B; VSOP87B."));
    assert!(report.contains("max Δlon="));
    assert!(report.contains("max Δlat="));
    assert!(report.contains("max Δdist="));
    assert!(report.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
    assert!(report.contains("House validation corpus"));
    assert!(report.contains("Benchmark provenance"));
    assert!(report.contains("source revision:"));
    assert!(report.contains("workspace status:"));
    assert!(report.contains("rustc version:"));
    assert!(report.contains("Benchmark summaries"));
    assert!(report.contains("Release bundle verification: verify-release-bundle"));
    assert!(report.contains("Workspace audit: workspace-audit / audit"));
    assert!(report.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(report.contains("Release notes summary: release-notes-summary"));
    assert!(report.contains("Release checklist summary: release-checklist-summary"));
    assert!(report.contains("Release summary: release-summary"));
    assert!(report.contains("Reference benchmark"));
    assert!(report.contains("Candidate benchmark"));
    assert!(report.contains("Packaged-data benchmark"));
    assert!(report.contains("Packaged artifact decode benchmark"));
    assert!(report.contains("Chart benchmark"));
}

#[test]
fn jpl_interpolation_quality_summary_includes_worst_case_body_labels() {
    let summary = jpl_interpolation_quality_summary().expect("summary should exist");
    assert!(!summary.max_bracket_span_body.is_empty());
    assert!(!summary.max_longitude_error_body.is_empty());
    assert!(!summary.max_latitude_error_body.is_empty());
    assert!(!summary.max_distance_error_body.is_empty());

    let rendered = format_jpl_interpolation_quality_summary(&summary);
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_bracket_span_body,
        format_instant(summary.max_bracket_span_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_longitude_error_body,
        format_instant(summary.max_longitude_error_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_latitude_error_body,
        format_instant(summary.max_latitude_error_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_distance_error_body,
        format_instant(summary.max_distance_error_epoch)
    )));
}

#[test]
fn cli_report_summary_lists_the_summary_command() {
    let rendered =
        render_cli(&["report-summary", "--rounds", "10"]).expect("report summary should render");
    assert!(rendered.contains("Validation report summary"));
    assert!(rendered.contains("Comparison corpus"));
    assert!(rendered.contains("Reference snapshot"));
    assert!(rendered.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_summary_for_report()));
    assert!(rendered.contains("Reference 2453000 major-body boundary evidence: 10 exact samples at JD 2453000.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2453000.5 boundary sample"));
    assert!(rendered.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
    assert!(
        rendered.contains("Comparison snapshot coverage: 232 rows across 10 bodies and 28 epochs")
    );
    assert!(rendered.contains("Body comparison summaries"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Packaged-artifact profile"));
    assert!(rendered.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported; unlisted outputs: []; support counts: stored=0, derived=2, approximated=0, unsupported=4, unlisted=0"
        ));
    assert!(rendered.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-draft; target thresholds: production thresholds recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
    assert!(rendered.contains("Packaged-artifact target-threshold state: target-threshold state: production thresholds recorded"));
    assert!(rendered.contains("Packaged-artifact fit envelope: fit envelope:"));
    assert!(rendered.contains("Packaged-artifact fit sample classes: fit sample classes:"));
    assert!(rendered.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
    assert!(rendered.contains("Packaged-artifact phase-2 corpus alignment: "));
    assert!(rendered.contains(
        "selected asteroid source request corpus=Selected asteroid source request corpus:"
    ));
    assert!(rendered
        .contains("Packaged-artifact generation manifest: Packaged artifact generation manifest:"));
    assert!(rendered.contains("Packaged batch parity:"));
    assert!(rendered.contains("Packaged frame parity:"));
    assert!(rendered.contains("Benchmark provenance"));
    assert!(rendered.contains("source revision:"));
    assert!(rendered.contains("workspace status:"));
    assert!(rendered.contains("rustc version:"));
    assert!(rendered.contains("Benchmark summaries"));
    assert!(rendered.contains("Packaged artifact decode benchmark"));

    let artifact_profile_coverage = render_cli(&["artifact-profile-coverage-summary"])
        .expect("artifact-profile-coverage-summary should render");
    assert_eq!(
        artifact_profile_coverage,
        format!(
            "Artifact profile coverage: {}",
            packaged_artifact_profile_coverage_summary_for_report()
        )
    );
    let packaged_frame_parity = render_cli(&["packaged-frame-parity-summary"])
        .expect("packaged-frame-parity-summary should render");
    assert_eq!(
        packaged_frame_parity,
        format!(
            "Packaged frame parity: {}",
            packaged_frame_parity_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-frame-parity"]).expect("packaged-frame-parity should render"),
        packaged_frame_parity
    );
    assert_eq!(
        render_cli(&["packaged-frame-parity", "extra"])
            .expect_err("packaged-frame-parity should reject extra arguments"),
        "packaged-frame-parity-summary does not accept extra arguments"
    );
    let packaged_frame_treatment = render_cli(&["packaged-frame-treatment-summary"])
        .expect("packaged-frame-treatment-summary should render");
    assert_eq!(
        packaged_frame_treatment,
        format!(
            "Packaged frame treatment: {}",
            packaged_frame_treatment_summary_for_report()
        )
    );

    let validation_report_summary = render_cli(&["validation-report-summary", "--rounds", "10"])
        .expect("validation-report-summary should render");
    assert!(validation_report_summary.contains("Validation report summary"));
    assert!(validation_report_summary.contains("Comparison corpus"));
    assert!(validation_report_summary.contains("Reference snapshot"));
    assert!(validation_report_summary.contains(&reference_snapshot_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(
        validation_report_summary.contains(&reference_snapshot_source_window_summary_for_report())
    );
    assert!(validation_report_summary
        .contains(&reference_snapshot_body_class_coverage_summary_for_report()));
    assert!(validation_report_summary.contains("Body comparison summaries"));
    assert!(validation_report_summary.contains("Body-class error envelopes"));
    assert!(validation_report_summary.contains("Body-class tolerance posture"));
    assert!(validation_report_summary.contains("Expected tolerance status"));
    assert!(validation_report_summary.contains("margin Δlon="));
    assert!(validation_report_summary.contains("margin Δdist="));
    assert!(validation_report_summary.contains("Chart benchmark"));
    assert!(validation_report_summary.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
    assert!(validation_report_summary.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
    assert!(validation_report_summary.contains("Comparison tolerance audit"));
    assert!(validation_report_summary.contains("command: compare-backends-audit"));
    assert!(validation_report_summary.contains("status: clean"));
    assert!(validation_report_summary.contains("regression bodies:"));
    assert!(!validation_report_summary.contains("regression bodies: Pluto"));
    assert!(validation_report_summary
        .contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(validation_report_summary.contains("ayanamsa catalog validation: ok"));
    assert!(validation_report_summary.contains("Release notes summary: release-notes-summary"));
    assert!(validation_report_summary.contains("Packaged-artifact profile"));
    assert!(validation_report_summary.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported; unlisted outputs: []; support counts: stored=0, derived=2, approximated=0, unsupported=4, unlisted=0"
        ));
    assert!(validation_report_summary.contains(
        "Packaged-artifact speed policy: Unsupported; motion output support=unsupported"
    ));
    assert!(validation_report_summary.contains(
            "Packaged-artifact storage/reconstruction: Quantized linear segments stored in pleiades-compression artifact format; body-indexed segment tables support random access by body and lookup time across the advertised range; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, sidereal, and motion outputs remain unsupported"
        ));
    assert!(validation_report_summary.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-draft; target thresholds: production thresholds recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
    assert!(validation_report_summary.contains("Packaged-artifact fit envelope: fit envelope:"));
    assert!(validation_report_summary.contains("Packaged-artifact fit margins: mean Δlon="));
    assert!(
        validation_report_summary.contains("Packaged-artifact fit threshold violation count: 0")
    );
    assert!(validation_report_summary
        .contains("Packaged-artifact fit threshold violations: 0; details: none"));
    assert!(validation_report_summary
        .contains("Packaged-artifact fit sample classes: fit sample classes:"));
    assert!(validation_report_summary.contains("Packaged-artifact fit outliers: fit outliers:"));
    assert!(validation_report_summary.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
    assert!(validation_report_summary.contains("Packaged-artifact source-fit and hold-out sync: "));
    assert!(validation_report_summary.contains("Packaged-artifact phase-2 corpus alignment: "));
    assert!(validation_report_summary
        .contains("Packaged-artifact generation manifest: Packaged artifact generation manifest:"));
    assert!(validation_report_summary.contains("Packaged-artifact size: "));
    assert_report_contains_exact_line(
        &validation_report_summary,
        &format!(
            "Packaged-artifact generation policy: {}",
            packaged_artifact_generation_policy_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &validation_report_summary,
        &format!(
            "Packaged-artifact normalized intermediates: {}",
            packaged_artifact_normalized_intermediate_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &validation_report_summary,
        &format!(
            "Packaged-artifact generation residual bodies: {}",
            validated_packaged_artifact_generation_residual_bodies_summary_for_report()
                .expect("packaged artifact residual bodies summary should validate")
        ),
    );
    assert!(validation_report_summary.contains("Packaged request policy"));
    assert!(validation_report_summary
        .contains(&reference_snapshot_major_body_boundary_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_mars_jupiter_boundary_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_mars_outer_boundary_summary_for_report()));
    assert_report_contains_exact_line(
            &validation_report_summary,
            "Packaged lookup epoch policy: TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
        );
    assert!(validation_report_summary.contains("Packaged frame parity"));
    assert!(validation_report_summary.lines().any(|line| {
        line == format!(
            "  Packaged frame treatment: {}",
            packaged_frame_treatment_summary_for_report()
        )
    }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
    assert!(validation_report_summary.contains("Frame policy:"));
    let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    assert!(validation_report_summary.lines().any(|line| {
        line == format!(
            "Mean-obliquity frame round-trip: {}",
            mean_obliquity_frame_round_trip
        )
    }));
    assert!(validation_report_summary.contains("Zodiac policy:"));
    assert!(validation_report_summary.contains(
            "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.6.123, api-stability=pleiades-api-stability/0.1.0"
        ));
    assert!(validation_report_summary
        .contains("lookup epoch policy=TT-grid retag without relativistic correction"));
    assert!(validation_report_summary.contains("Benchmark summaries"));
    assert!(validation_report_summary.contains("Packaged artifact decode benchmark"));
}

#[test]
fn cli_rejects_zero_rounds_and_duplicate_bundle_release_args() {
    let benchmark_error = render_cli(&["benchmark", "--rounds", "0"])
        .expect_err("benchmark should reject zero rounds");
    assert!(benchmark_error.contains("invalid value for --rounds: expected a positive integer"));

    let duplicate_benchmark_rounds_error =
        render_cli(&["benchmark", "--rounds", "1", "--rounds", "2"])
            .expect_err("benchmark should reject duplicate rounds arguments");
    assert!(duplicate_benchmark_rounds_error.contains("duplicate value for --rounds argument"));

    let bundle_dir = unique_temp_dir("pleiades-release-bundle-zero-rounds-command");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    let bundle_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "0",
    ])
    .expect_err("bundle-release should reject zero rounds");
    assert!(bundle_error.contains("invalid value for --rounds: expected a positive integer"));

    let duplicate_out_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--out",
        &bundle_dir_string,
    ])
    .expect_err("bundle-release should reject duplicate --out arguments");
    assert!(duplicate_out_error.contains("duplicate value for --out <dir> argument"));

    let duplicate_rounds_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
        "--rounds",
        "2",
    ])
    .expect_err("bundle-release should reject duplicate --rounds arguments");
    assert!(duplicate_rounds_error.contains("duplicate value for --rounds argument"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

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
fn benchmark_corpus_spans_the_target_window() {
    let corpus = benchmark_corpus();
    let summary = corpus.summary();
    assert_eq!(summary.epoch_count, 11);
    assert_eq!(summary.body_count, default_chart_bodies().len());
    assert_eq!(summary.request_count, 110);
    assert_eq!(summary.apparentness, Apparentness::Mean);
    assert!(summary.earliest_julian_day < summary.latest_julian_day);
}

#[test]
fn corpus_summary_has_a_displayable_summary_line() {
    let summary = benchmark_corpus().summary();
    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary
        .summary_line()
        .contains("corpus name=Representative 1500-2500 window"));
    assert!(summary.summary_line().contains("requests=110"));
    assert!(summary.summary_line().contains("epochs=11"));
    assert!(summary.summary_line().contains("bodies="));
}

#[test]
fn corpus_summary_validate_accepts_the_reported_summary() {
    let summary = benchmark_corpus().summary();
    summary
        .validate()
        .expect("reported corpus summary should validate");
}

#[test]
fn corpus_summary_validate_rejects_epoch_order_drift() {
    let mut summary = benchmark_corpus().summary();
    summary.epochs.reverse();
    let error = summary
        .validate()
        .expect_err("mutated corpus summary should fail validation");
    assert!(
        error.to_string().contains("out of order") || error.to_string().contains("first epoch")
    );
}

#[test]
fn packaged_benchmark_corpus_uses_packaged_artifact_coverage() {
    let corpus = artifact::packaged_artifact_corpus();
    let summary = corpus.summary();
    assert!(summary.name.contains("Packaged artifact"));
    assert_eq!(summary.apparentness, Apparentness::Mean);
    assert_eq!(
        summary.body_count,
        pleiades_data::packaged_artifact().bodies.len()
    );
    assert!(summary.request_count > 0);
    assert!(summary.earliest_julian_day <= summary.latest_julian_day);
}

#[test]
fn house_validation_report_includes_representative_scenarios() {
    let report = house_validation_report();
    assert_eq!(report.scenarios.len(), 9);
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Western hemisphere reference chart"));
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Southern polar stress chart"));
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Northern high-latitude mountain stress chart"));
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Southern hemisphere reference chart"));
}

#[test]
fn comparison_report_uses_release_grade_corpus_without_pluto() {
    let corpus = release_grade_corpus();
    let report = compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &corpus,
    )
    .expect("comparison should succeed");

    let regressions = report.notable_regressions();
    assert!(regressions.is_empty());

    let body_summaries = report.body_summaries();
    assert_eq!(body_summaries.len(), report.tolerance_summaries().len());
    assert!(body_summaries
        .iter()
        .all(|summary| summary.sample_count > 0 && summary.body != CelestialBody::Pluto));
    assert!(body_summaries
        .iter()
        .any(|summary| summary.body == CelestialBody::Jupiter
            && summary.max_longitude_delta_deg > 0.0
            && summary.max_longitude_delta_deg < 0.01));

    let archive = report.regression_archive();
    assert_eq!(archive.corpus_name, corpus.name);
    assert!(archive.cases.is_empty());
    let tolerance_summaries = report.tolerance_summaries();
    assert!(tolerance_summaries.iter().any(|summary| {
        summary.body == CelestialBody::Jupiter
            && summary.tolerance.profile.contains("full-file VSOP87B")
    }));
    assert!(tolerance_summaries
        .iter()
        .all(|summary| summary.body != CelestialBody::Pluto));
    let tolerance_policy_entries = report.tolerance_policy_entries();
    assert_eq!(tolerance_policy_entries.len(), 6);
    assert!(tolerance_policy_entries
        .iter()
        .any(|entry| entry.scope == ComparisonToleranceScope::Luminary));
    assert!(tolerance_policy_entries
        .iter()
        .any(|entry| entry.scope == ComparisonToleranceScope::Pluto));
    let body_class_tolerance_summaries = report.body_class_tolerance_summaries();
    assert!(body_class_tolerance_summaries.iter().any(|summary| {
        summary.class == BodyClass::MajorPlanet
            && summary.body_count >= 1
            && summary.sample_count >= summary.body_count
            && summary.outside_tolerance_body_count == 0
            && !summary.outside_bodies.contains(&CelestialBody::Pluto)
            && summary.max_longitude_delta_body.is_some()
            && summary.max_latitude_delta_body.is_some()
            && summary.max_distance_delta_body.is_some()
    }));

    let rendered = report.to_string();
    assert!(rendered.contains("Body comparison summaries"));
    assert!(rendered.contains("Body-class tolerance posture"));
    assert!(rendered.contains("Expected tolerance status"));
    assert!(rendered.contains("phase-1 full-file VSOP87B planetary evidence"));
    assert!(rendered.contains("Notable regressions"));
}

#[test]
fn body_class_summary_line_reuses_the_typed_formatter() {
    let summary = BodyClassSummary {
        class: BodyClass::MajorPlanet,
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: 1.234,
        sum_longitude_delta_deg: 1.0,
        sum_longitude_delta_sq_deg: 1.0,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        max_latitude_delta_body: Some(CelestialBody::Mars),
        max_latitude_delta_deg: 0.5,
        sum_latitude_delta_deg: 0.5,
        sum_latitude_delta_sq_deg: 0.25,
        median_latitude_delta_deg: 0.5,
        percentile_latitude_delta_deg: 0.5,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(2.5),
        sum_distance_delta_au: 2.5,
        sum_distance_delta_sq_au: 6.25,
        distance_count: 1,
        median_distance_delta_au: Some(2.5),
        percentile_distance_delta_au: Some(2.5),
    };

    let expected = "samples=1, max Δlon=1.234000000000° (Mars), mean Δlon=1.000000000000°, median Δlon=1.000000000000°, 95th percentile longitude delta: 1.000000000000°, rms Δlon=1.000000000000°, max Δlat=0.500000000000° (Mars), mean Δlat=0.500000000000°, median Δlat=0.500000000000°, 95th percentile latitude delta: 0.500000000000°, rms Δlat=0.500000000000°, max Δdist=2.500000000000 AU (Mars), mean Δdist=2.500000000000 AU, median Δdist=2.500000000000 AU, 95th percentile distance delta: 2.500000000000 AU, rms Δdist=2.500000000000 AU";

    assert_eq!(summary.summary_line(), expected);
    assert_eq!(summary.validated_summary_line(), Ok(expected.to_string()));
    assert_eq!(summary.to_string(), expected);
    assert_eq!(
        format_body_class_comparison_envelope_for_report(&summary),
        expected
    );
}

#[test]
fn body_class_summary_validation_rejects_drift() {
    let mut summary = BodyClassSummary {
        class: BodyClass::MajorPlanet,
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: 1.234,
        sum_longitude_delta_deg: 1.0,
        sum_longitude_delta_sq_deg: 1.0,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        max_latitude_delta_body: Some(CelestialBody::Mars),
        max_latitude_delta_deg: 0.5,
        sum_latitude_delta_deg: 0.5,
        sum_latitude_delta_sq_deg: 0.25,
        median_latitude_delta_deg: 0.5,
        percentile_latitude_delta_deg: 0.5,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(2.5),
        sum_distance_delta_au: 2.5,
        sum_distance_delta_sq_au: 6.25,
        distance_count: 1,
        median_distance_delta_au: Some(2.5),
        percentile_distance_delta_au: Some(2.5),
    };

    summary.max_latitude_delta_body = None;

    assert_eq!(
        summary.validate(),
        Err(BodyClassSummaryValidationError::FieldOutOfSync {
            class: BodyClass::MajorPlanet
        })
    );
    assert_eq!(
        summary.validated_summary_line(),
        Err(BodyClassSummaryValidationError::FieldOutOfSync {
            class: BodyClass::MajorPlanet
        })
    );
}

#[test]
fn body_class_tolerance_summary_reuses_the_typed_formatter() {
    let summary = BodyClassToleranceSummary {
        class: BodyClass::MajorPlanet,
        tolerance: ComparisonTolerance {
            backend_family: BackendFamily::ReferenceData,
            profile: "phase-1 body-class tolerance",
            max_longitude_delta_deg: 1.5,
            max_latitude_delta_deg: 0.5,
            max_distance_delta_au: Some(3.0),
        },
        body_count: 2,
        sample_count: 2,
        within_tolerance_body_count: 1,
        outside_tolerance_body_count: 1,
        outside_tolerance_sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: Some(1.0),
        max_latitude_delta_body: Some(CelestialBody::Jupiter),
        max_latitude_delta_deg: Some(0.25),
        max_distance_delta_body: Some(CelestialBody::Saturn),
        max_distance_delta_au: Some(2.5),
        sum_longitude_delta_deg: 2.0,
        sum_longitude_delta_sq_deg: 2.0,
        sum_latitude_delta_deg: 0.5,
        sum_latitude_delta_sq_deg: 0.125,
        sum_distance_delta_au: 3.0,
        sum_distance_delta_sq_au: 4.5,
        distance_count: 2,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        median_latitude_delta_deg: 0.25,
        percentile_latitude_delta_deg: 0.25,
        median_distance_delta_au: Some(1.5),
        percentile_distance_delta_au: Some(1.5),
        outside_bodies: vec![CelestialBody::Mars],
    };

    let expected = "Major planets: backend family=reference data, profile=phase-1 body-class tolerance, bodies=2, samples=2, within tolerance bodies=1, outside tolerance bodies=1, limit Δlon≤1.500000°, margin Δlon=+0.500000000000°, limit Δlat≤0.500000°, margin Δlat=+0.250000000000°, limit Δdist=3.000000 AU, margin Δdist=+0.500000000000 AU, max Δlon=1.000000000000° (Mars), max Δlat=0.250000000000° (Jupiter), max Δdist=2.500000000000 AU (Saturn)";

    assert!(summary.validate().is_ok());
    assert_eq!(summary.summary_line(), expected);
    assert_eq!(summary.to_string(), expected);
    assert_eq!(
        format_body_class_tolerance_envelope_for_report(&summary),
        expected
    );
}

#[test]
fn body_class_tolerance_summary_rejects_count_drift() {
    let summary = BodyClassToleranceSummary {
        class: BodyClass::MajorPlanet,
        tolerance: ComparisonTolerance {
            backend_family: BackendFamily::ReferenceData,
            profile: "phase-1 body-class tolerance",
            max_longitude_delta_deg: 1.5,
            max_latitude_delta_deg: 0.5,
            max_distance_delta_au: Some(3.0),
        },
        body_count: 1,
        sample_count: 1,
        within_tolerance_body_count: 1,
        outside_tolerance_body_count: 0,
        outside_tolerance_sample_count: 0,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: Some(1.0),
        max_latitude_delta_body: Some(CelestialBody::Jupiter),
        max_latitude_delta_deg: Some(0.25),
        max_distance_delta_body: Some(CelestialBody::Saturn),
        max_distance_delta_au: Some(2.5),
        sum_longitude_delta_deg: 1.0,
        sum_longitude_delta_sq_deg: 1.0,
        sum_latitude_delta_deg: 0.25,
        sum_latitude_delta_sq_deg: 0.0625,
        sum_distance_delta_au: 2.5,
        sum_distance_delta_sq_au: 6.25,
        distance_count: 1,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        median_latitude_delta_deg: 0.25,
        percentile_latitude_delta_deg: 0.25,
        median_distance_delta_au: Some(2.5),
        percentile_distance_delta_au: Some(2.5),
        outside_bodies: vec![CelestialBody::Mars, CelestialBody::Jupiter],
    };

    let rendered = format_body_class_tolerance_envelope_for_report(&summary);
    assert!(rendered.contains("body-class tolerance envelope unavailable"));
    assert!(summary.validate().is_err());
}

#[test]
fn cli_help_lists_the_validation_commands() {
    let rendered = render_cli(&["help"]).expect("help should render");
    assert!(rendered.contains("compare-backends"));
    assert!(rendered.contains("compare-backends-audit"));
    assert!(rendered.contains("backend-matrix"));
    assert!(rendered.contains("capability-matrix"));
    assert!(rendered.contains("backend-matrix-summary"));
    assert!(rendered.contains("matrix-summary"));
    assert!(rendered.contains("compatibility-profile"));
    assert!(rendered.contains("compatibility-caveats-summary"));
    assert!(rendered.contains("compatibility-caveats    Alias for compatibility-caveats-summary"));
    assert!(rendered.contains("catalog-inventory"));
    assert!(rendered.contains("Alias for compatibility-profile"));
    assert!(rendered.contains("benchmark [--rounds N]"));
    assert!(rendered.contains("benchmark-matrix [--rounds N]"));
    assert!(rendered.contains("benchmark-matrix-summary [--rounds N]"));
    assert!(rendered.contains("comparison-corpus-summary"));
    assert!(rendered.contains("comparison-corpus         Alias for comparison-corpus-summary"));
    assert!(rendered.contains("comparison-corpus-release-guard-summary"));
    assert!(rendered.contains(
        "comparison-corpus-release-guard  Alias for comparison-corpus-release-guard-summary"
    ));
    assert!(rendered.contains("comparison-corpus-guard-summary"));
    assert!(rendered.contains("comparison-envelope-summary"));
    assert!(rendered.contains("comparison-envelope       Alias for comparison-envelope-summary"));
    assert!(rendered.contains("comparison-tolerance-summary"));
    assert!(rendered
        .contains("comparison-tolerance-summary  Alias for comparison-tolerance-policy-summary"));
    assert!(rendered.contains("comparison-tolerance-scope-coverage-summary"));
    assert!(rendered.contains(
            "comparison-tolerance-scope-coverage  Alias for comparison-tolerance-scope-coverage-summary"
        ));
    assert!(rendered.contains("benchmark-corpus-summary"));
    assert!(rendered.contains("interpolation-quality-request-corpus-summary"));
    assert!(rendered.contains("lunar-reference-evidence-summary"));
    assert!(
        rendered.contains("lunar-reference-evidence  Alias for lunar-reference-evidence-summary")
    );
    assert!(rendered.contains("lunar-reference-mixed-time-scale-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-mixed-tt-tdb-batch-parity  Alias for reference-snapshot-mixed-time-scale-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-batch-parity          Alias for reference-snapshot-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-equatorial-parity-summary"));
    assert!(rendered.contains("reference-snapshot-equatorial-parity     Alias for reference-snapshot-equatorial-parity-summary"));
    assert!(rendered.contains(
            "lunar-reference-mixed-tt-tdb-batch-parity-summary  Alias for lunar-reference-mixed-time-scale-batch-parity-summary"
        ));
    assert!(rendered.contains(
            "lunar-reference-mixed-tt-tdb-batch-parity  Alias for lunar-reference-mixed-time-scale-batch-parity-summary"
        ));
    assert!(rendered.contains("jpl-source-posture-summary"));
    assert!(rendered.contains("jpl-source-posture         Alias for jpl-source-posture-summary"));
    assert!(rendered.contains("jpl-provenance-only-summary"));
    assert!(rendered.contains("jpl-provenance-only  Alias for jpl-provenance-only-summary"));
    assert!(rendered.contains("report [--rounds N]"));
    assert!(rendered.contains("generate-report"));
    assert!(rendered.contains("validation-report-summary [--rounds N]"));
    assert!(rendered.contains("report-summary [--rounds N]"));
    assert!(rendered.contains("validation-summary"));
    assert!(rendered.contains("validate-artifact"));
    assert!(rendered.contains("generate-packaged-artifact"));
    assert!(rendered.contains("regenerate-packaged-artifact"));
    assert!(rendered.contains("artifact-summary"));
    assert!(rendered.contains("artifact-boundary-envelope-summary"));
    assert!(rendered.contains("packaged-artifact-output-support-summary"));
    assert!(rendered.contains(
        "packaged-artifact-output-support       Alias for packaged-artifact-output-support-summary"
    ));
    assert!(rendered.contains("packaged-artifact-speed-policy-summary"));
    assert!(rendered.contains(
        "packaged-artifact-speed-policy       Alias for packaged-artifact-speed-policy-summary"
    ));
    assert!(rendered.contains("motion-policy-summary"));
    assert!(rendered.contains("motion-policy               Alias for motion-policy-summary"));
    assert!(rendered.contains("packaged-artifact-access-summary"));
    assert!(
        rendered.contains("packaged-artifact-access  Alias for packaged-artifact-access-summary")
    );
    assert!(rendered.contains(
        "packaged-artifact-path-policy-summary  Alias for packaged-artifact-access-summary"
    ));
    assert!(rendered.contains(
        "packaged-artifact-path-policy  Alias for packaged-artifact-path-policy-summary"
    ));
    assert!(rendered.contains("packaged-artifact-storage-summary"));
    assert!(rendered.contains(
        "packaged-artifact-storage           Alias for packaged-artifact-storage-summary"
    ));
    assert!(rendered.contains("packaged-artifact-production-profile-summary"));
    assert!(rendered.contains("packaged-artifact-production-profile"));
    assert!(rendered.contains("packaged-artifact-target-threshold-summary"));
    assert!(rendered.contains(
        "packaged-artifact-target-threshold  Alias for packaged-artifact-target-threshold-summary"
    ));
    assert!(rendered.contains("packaged-artifact-target-threshold-state-summary"));
    assert!(rendered.contains("packaged-artifact-target-threshold-state  Alias for packaged-artifact-target-threshold-state-summary"));
    assert!(rendered.contains("packaged-artifact-target-threshold-scope-envelopes-summary"));
    assert!(rendered.contains("packaged-artifact-target-threshold-scope-envelopes  Alias for packaged-artifact-target-threshold-scope-envelopes-summary"));
    assert!(rendered.contains("packaged-artifact-source-fit-holdout-sync-summary"));
    assert!(rendered.contains("packaged-artifact-source-fit-holdout-sync  Alias for packaged-artifact-source-fit-holdout-sync-summary"));
    assert!(rendered.contains("packaged-artifact-fit-envelope-summary"));
    assert!(rendered.contains(
        "packaged-artifact-fit-envelope  Alias for packaged-artifact-fit-envelope-summary"
    ));
    assert!(rendered.contains("packaged-artifact-fit-margins-summary"));
    assert!(rendered.contains(
        "packaged-artifact-fit-margins      Alias for packaged-artifact-fit-margins-summary"
    ));
    assert!(rendered.contains("packaged-artifact-fit-sample-classes-summary"));
    assert!(rendered.contains("packaged-artifact-fit-sample-classes  Alias for packaged-artifact-fit-sample-classes-summary"));
    assert!(rendered.contains("packaged-artifact-fit-outliers-summary"));
    assert!(rendered.contains(
        "packaged-artifact-fit-outliers  Alias for packaged-artifact-fit-outliers-summary"
    ));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violation-count-summary"));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violation-count  Alias for packaged-artifact-fit-threshold-violation-count-summary"));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violations-summary"));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violations  Alias for packaged-artifact-fit-threshold-violations-summary"));
    assert!(rendered.contains("packaged-artifact-generation-manifest-summary"));
    assert!(rendered.contains(
            "packaged-artifact-generation-manifest  Alias for packaged-artifact-generation-manifest-summary"
        ));
    assert!(rendered.contains("packaged-artifact-generation-manifest-checksum-summary"));
    assert!(rendered.contains(
            "packaged-artifact-generation-manifest-checksum  Alias for packaged-artifact-generation-manifest-checksum-summary"
        ));
    assert!(rendered.contains("packaged-artifact-generation-policy-summary"));
    assert!(rendered.contains("packaged-artifact-normalized-intermediate-summary"));
    assert!(rendered.contains(
            "packaged-artifact generation manifest, packaged-artifact generation manifest summary, packaged-artifact generation manifest checksum summary, packaged-artifact generation manifest checksum sidecar, benchmark-corpus summary"
        ));
    assert!(rendered.contains(
            "packaged-artifact-generation-policy     Alias for packaged-artifact-generation-policy-summary"
        ));
    assert!(rendered.contains(
            "packaged-artifact-normalized-intermediate  Alias for packaged-artifact-normalized-intermediate-summary"
        ));
    assert!(rendered.contains("packaged-artifact-lookup-epoch-policy-summary"));
    assert!(rendered.contains("Alias for packaged-artifact-lookup-epoch-policy-summary"));
    assert!(rendered.contains("packaged-artifact-generation-residual-summary"));
    assert!(rendered.contains("packaged-artifact-generation-residual-bodies-summary"));
    assert!(rendered.contains("packaged-artifact-regeneration-summary"));
    assert!(rendered.contains(
        "packaged-artifact-regeneration      Alias for packaged-artifact-regeneration-summary"
    ));
    assert!(rendered.contains("packaged-frame-parity-summary"));
    assert!(
        rendered.contains("packaged-frame-parity         Alias for packaged-frame-parity-summary")
    );
    assert!(rendered.contains("packaged-frame-treatment-summary"));
    assert!(rendered.contains("workspace-audit"));
    assert!(rendered.contains("native-dependency-audit"));
    assert!(rendered.contains("native-dependency-audit-summary"));
    assert!(rendered.contains("source-documentation-summary"));
    assert!(
        rendered.contains("source-documentation         Alias for source-documentation-summary")
    );
    assert!(rendered.contains("source-documentation-health-summary"));
    assert!(rendered
        .contains("source-documentation-health  Alias for source-documentation-health-summary"));
    assert!(rendered
        .contains("source-audit-summary      Print the compact VSOP87 source audit summary"));
    assert!(rendered.contains("source-audit              Alias for source-audit-summary"));
    assert!(rendered.contains(
        "generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary"
    ));
    assert!(rendered.contains("generated-binary-audit    Alias for generated-binary-audit-summary"));
    assert!(rendered.contains("api-stability"));
    assert!(rendered.contains("api-stability-summary"));
    assert!(rendered.contains("compatibility-profile-summary"));
    assert!(rendered.contains("house-formula-families-summary"));
    assert!(rendered.contains("house-formula-families    Alias for house-formula-families-summary"));
    assert!(rendered.contains("house-latitude-sensitive-summary"));
    assert!(rendered.contains("house-latitude-sensitive-constraints-summary"));
    assert!(rendered.contains("house-latitude-sensitive-failure-modes-summary"));
    assert!(rendered.contains(
            "house-latitude-sensitive-failure-modes  Alias for house-latitude-sensitive-failure-modes-summary"
        ));
    assert!(rendered.contains(
            "house-latitude-sensitive-constraints  Alias for house-latitude-sensitive-constraints-summary"
        ));
    assert!(
        rendered.contains("house-latitude-sensitive  Alias for house-latitude-sensitive-summary")
    );
    assert!(rendered.contains("house-code-aliases-summary"));
    assert!(rendered.contains("house-code-alias-summary"));
    assert!(rendered.contains("ayanamsa-catalog-validation-summary"));
    assert!(rendered
        .contains("ayanamsa-catalog-validation  Alias for ayanamsa-catalog-validation-summary"));
    assert!(rendered.contains("ayanamsa-metadata-coverage-summary"));
    assert!(rendered
        .contains("ayanamsa-metadata-coverage  Alias for ayanamsa-metadata-coverage-summary"));
    assert!(rendered.contains("ayanamsa-reference-offsets-summary"));
    assert!(rendered
        .contains("ayanamsa-reference-offsets  Alias for ayanamsa-reference-offsets-summary"));
    assert!(rendered.contains("time-scale-policy-summary"));
    assert!(rendered.contains("time-scale-policy       Alias for time-scale-policy-summary"));
    assert!(rendered.contains("utc-convenience-policy-summary"));
    assert!(rendered.contains("utc-convenience-policy  Alias for utc-convenience-policy-summary"));
    assert!(rendered.contains("delta-t-policy-summary"));
    assert!(rendered.contains("delta-t-policy         Alias for delta-t-policy-summary"));
    assert!(rendered.contains("zodiac-policy-summary"));
    assert!(rendered.contains("zodiac-policy         Alias for zodiac-policy-summary"));
    assert!(rendered.contains("observer-policy-summary"));
    assert!(rendered.contains("observer-policy        Alias for observer-policy-summary"));
    assert!(rendered.contains("apparentness-policy-summary"));
    assert!(rendered.contains("apparentness-policy     Alias for apparentness-policy-summary"));
    assert!(rendered.contains("native-sidereal-policy-summary"));
    assert!(rendered.contains("native-sidereal-policy   Alias for native-sidereal-policy-summary"));
    assert!(rendered.contains("frame-policy-summary"));
    assert!(rendered.contains("frame-policy             Alias for frame-policy-summary"));
    assert!(rendered.contains("production-generation-boundary-summary"));
    assert!(rendered.contains(
        "production-generation-boundary         Alias for production-generation-boundary-summary"
    ));
    assert!(rendered.contains("production-generation-boundary-request-corpus-summary"));
    assert!(rendered.contains("production-generation-boundary-request-corpus  Alias for production-generation-boundary-request-corpus-summary"));
    assert!(rendered.contains("production-generation-boundary-request-corpus-equatorial-summary"));
    assert!(rendered.contains("production-generation-boundary-request-corpus-equatorial  Alias for production-generation-boundary-request-corpus-equatorial-summary"));
    assert!(rendered.contains("production-generation-body-class-coverage-summary"));
    assert!(rendered.contains("production-body-class-coverage-summary"));
    assert!(rendered.contains("production-generation-source-window-summary"));
    assert!(rendered.contains("production-generation-source-window  Alias for production-generation-source-window-summary"));
    assert!(rendered.contains("production-generation-summary"));
    assert!(rendered
        .contains("production-generation           Alias for production-generation-summary"));
    assert!(rendered.contains("production-generation-quarter-day-boundary-summary"));
    assert!(rendered.contains("production-generation-quarter-day-boundary  Alias for production-generation-quarter-day-boundary-summary"));
    assert!(rendered.contains("production-generation-boundary-source-summary"));
    assert!(rendered.contains("production-generation-boundary-source  Alias for production-generation-boundary-source-summary"));
    assert!(rendered.contains("production-generation-boundary-window-summary"));
    assert!(rendered.contains("production-generation-boundary-window  Alias for production-generation-boundary-window-summary"));
    assert!(rendered.contains("production-generation-manifest-summary"));
    assert!(rendered.contains(
        "production-generation-manifest  Alias for production-generation-manifest-summary"
    ));
    assert!(rendered.contains("production-generation-manifest-checksum-summary"));
    assert!(rendered.contains(
            "production-generation-manifest-checksum  Alias for production-generation-manifest-checksum-summary"
        ));
    assert!(rendered.contains("production-generation-source-summary"));
    assert!(rendered.contains("production-generation-source-revision-summary"));
    assert!(rendered.contains(
            "production-generation-source-revision  Alias for production-generation-source-revision-summary"
        ));
    assert!(rendered.contains("comparison-snapshot-source-window-summary"));
    assert!(rendered.contains(
        "comparison-snapshot-source-window  Alias for comparison-snapshot-source-window-summary"
    ));
    assert!(rendered.contains("comparison-snapshot-source-summary"));
    assert!(rendered.contains(
        "comparison-snapshot-source        Alias for comparison-snapshot-source-summary"
    ));
    assert!(rendered.contains("comparison-snapshot-body-class-coverage-summary"));
    assert!(rendered.contains("comparison-body-class-coverage-summary"));
    assert!(rendered.contains("comparison-snapshot-manifest-summary"));
    assert!(rendered.contains("comparison-snapshot-summary"));
    assert!(rendered.contains("j2000-snapshot           Alias for comparison-snapshot-summary"));
    assert!(rendered.contains("comparison-snapshot         Alias for comparison-snapshot-summary"));
    assert!(rendered.contains("comparison-snapshot-batch-parity-summary"));
    assert!(rendered.contains(
        "comparison-snapshot-batch-parity  Alias for comparison-snapshot-batch-parity-summary"
    ));
    assert!(rendered.contains("reference-snapshot-source-window-summary"));
    assert!(rendered.contains(
        "reference-snapshot-source-window  Alias for reference-snapshot-source-window-summary"
    ));
    assert!(rendered.contains("reference-snapshot-source-summary"));
    assert!(rendered
        .contains("reference-snapshot-source        Alias for reference-snapshot-source-summary"));
    assert!(rendered.contains("reference-snapshot-lunar-boundary-summary"));
    assert!(rendered
        .contains("lunar-boundary-summary   Alias for reference-snapshot-lunar-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-1600-selected-body-boundary-summary"));
    assert!(rendered.contains("1600-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2268932-selected-body-boundary-summary"));
    assert!(rendered.contains("2268932-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2305457-selected-body-boundary-summary"));
    assert!(rendered.contains("2305457-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-1750-selected-body-boundary-summary"));
    assert!(rendered.contains("1750-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-1750-major-body-interior-summary"));
    assert!(rendered.contains("1750-major-body-interior-summary"));
    assert!(rendered.contains("reference-snapshot-2360234-major-body-interior-summary"));
    assert!(rendered.contains("2360234-major-body-interior-summary"));
    assert!(rendered.contains("reference-snapshot-2451912-major-body-boundary-summary"));
    assert!(rendered.contains("2451912-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2200-selected-body-boundary-summary"));
    assert!(rendered.contains("2200-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2524593-selected-body-boundary-summary"));
    assert!(rendered.contains("2524593-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2415020-selected-body-boundary-summary"));
    assert!(rendered.contains("2415020-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2634167-selected-body-boundary-summary"));
    assert!(rendered.contains("2634167-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-1749-major-body-boundary-summary"));
    assert!(rendered.contains("1749-major-body-boundary-summary"));
    assert!(rendered.contains("2360233-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-early-major-body-boundary-summary"));
    assert!(rendered.contains("early-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2378498-major-body-boundary-summary"));
    assert!(rendered.contains("2378498-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-1800-major-body-boundary-summary"));
    assert!(rendered.contains("1800-major-body-boundary-summary"));
    assert!(rendered.contains("2378499-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2400000-major-body-boundary-summary"));
    assert!(rendered.contains("2400000-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451545-major-body-boundary-summary"));
    assert!(rendered.contains("2451545-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2500-major-body-boundary-summary"));
    assert!(rendered.contains("2500-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2453000-major-body-boundary-summary"));
    assert!(rendered.contains("2453000-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2500000-major-body-boundary-summary"));
    assert!(rendered.contains("2500000-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2600000-major-body-boundary-summary"));
    assert!(rendered.contains("2600000-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451910-major-body-boundary-summary"));
    assert!(rendered.contains("2451910-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451911-major-body-boundary-summary"));
    assert!(rendered.contains("2451911-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451913-major-body-boundary-summary"));
    assert!(rendered.contains("2451913-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-boundary-summary"));
    assert!(rendered.contains("2451914-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-bridge-day-summary"));
    assert!(rendered.contains("2451914-major-body-bridge-day-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-bridge-summary"));
    assert!(rendered.contains("2451914-major-body-bridge-summary"));
    assert!(rendered.contains(
        "2451914-major-body-bridge  Alias for reference-snapshot-2451914-major-body-bridge-summary"
    ));
    assert!(rendered.contains("reference-snapshot-2451915-major-body-boundary-summary"));
    assert!(rendered.contains("2451915-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451915-major-body-bridge-summary"));
    assert!(rendered.contains("2451915-major-body-bridge-summary"));
    assert!(rendered.contains(
        "2451915-major-body-bridge  Alias for reference-snapshot-2451915-major-body-bridge-summary"
    ));
    assert!(rendered.contains("reference-snapshot-2451916-major-body-interior-summary"));
    assert!(rendered.contains("2451916-major-body-interior-summary"));
}

#[test]
fn lunar_reference_mixed_time_scale_batch_parity_summary_renders_the_explicit_slice() {
    let rendered = render_cli(&["lunar-reference-mixed-time-scale-batch-parity-summary"])
        .expect("lunar reference mixed TT/TDB batch parity summary should render");
    assert!(rendered.contains("lunar reference mixed TT/TDB batch parity"));
    assert_eq!(rendered, lunar_reference_batch_parity_summary_for_report());
    assert_eq!(
        render_cli(&["lunar-reference-mixed-tt-tdb-batch-parity-summary"])
            .expect("lunar reference mixed TT/TDB batch parity alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&[
            "lunar-reference-mixed-time-scale-batch-parity-summary",
            "extra"
        ])
        .expect_err(
            "lunar reference mixed TT/TDB batch parity summary should reject extra arguments"
        ),
        "lunar-reference-mixed-time-scale-batch-parity-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_mixed_time_scale_batch_parity_summary_renders_the_short_alias() {
    let rendered = render_cli(&["reference-snapshot-mixed-time-scale-batch-parity-summary"])
        .expect("reference snapshot mixed TT/TDB batch parity summary should render");
    assert_eq!(
        rendered,
        reference_snapshot_mixed_time_scale_batch_parity_summary_text()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity-summary"])
            .expect("reference snapshot mixed TT/TDB batch parity alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity"])
            .expect("reference snapshot mixed TT/TDB batch parity short alias should render"),
        rendered
    );
    assert_eq!(
            render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity", "extra"])
                .expect_err(
                    "reference snapshot mixed TT/TDB batch parity short alias should reject extra arguments"
                ),
            "reference-snapshot-mixed-time-scale-batch-parity-summary does not accept extra arguments"
        );
}

#[test]
fn interpolation_quality_request_corpus_summary_renders_the_explicit_slice() {
    let rendered = render_cli(&["interpolation-quality-request-corpus-summary"])
        .expect("interpolation quality request corpus summary should render");
    assert!(rendered.contains("Interpolation-quality sample request corpus:"));
    assert_eq!(
        rendered,
        interpolation_quality_sample_request_corpus_summary_for_report()
    );
    assert_eq!(
        render_cli(&["interpolation-quality-request-corpus"])
            .expect("interpolation quality request corpus alias should render"),
        rendered
    );
}

#[test]
fn release_profile_identifiers_summary_alias_command_renders_and_rejects_extra_arguments() {
    let rendered = render_cli(&["release-profile-identifiers-summary"])
        .expect("release-profile identifiers summary should render");
    assert_eq!(
        render_cli(&["release-profile-identifiers"]).unwrap(),
        rendered
    );
    assert!(rendered.contains("Release profile identifiers summary"));
    assert!(rendered.contains("Summary line: v1 compatibility="));
    assert_eq!(
        render_cli(&["release-profile-identifiers-summary", "extra"])
            .expect_err("summary alias should reject extra arguments"),
        "release-profile-identifiers-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["release-profile-identifiers", "extra"])
            .expect_err("summary alias should reject extra arguments"),
        "release-profile-identifiers does not accept extra arguments"
    );
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
fn api_stability_command_renders_the_posture() {
    let rendered = render_cli(&["api-stability"]).expect("api posture should render");
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Stable consumer surfaces:"));
    assert!(rendered.contains("Experimental or operational surfaces:"));
    assert!(rendered.contains("Deprecation policy:"));
}

#[test]
fn api_stability_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["api-stability-summary"]).expect("api stability summary should render");
    let release_profiles = current_release_profile_identifiers();
    let api_stability = current_api_stability_profile();
    assert!(rendered.contains("API stability summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains(&format!(
            "Summary line: API stability posture: {}; stable surfaces: {}; experimental surfaces: {}; deprecation policy items: {}; intentional limits: {}",
            release_profiles.api_stability_profile_id,
            api_stability.stable_surfaces.len(),
            api_stability.experimental_surfaces.len(),
            api_stability.deprecation_policy.len(),
            api_stability.intentional_limits.len()
        )));
    assert!(rendered.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("Stable surfaces:"));
    assert!(rendered.contains("Experimental surfaces:"));
    assert!(rendered.contains("Deprecation policy items:"));
    assert!(rendered.contains("Intentional limits:"));
    assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn compatibility_profile_command_renders_the_full_profile() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains("Stage 6 release profile:"));
    assert!(rendered.contains(&format!(
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    )));
    assert!(rendered.contains("Target compatibility catalog:"));
    assert!(rendered.contains(
            "the full Swiss-Ephemeris-class house-system catalog remains the long-term compatibility goal."
        ));
    assert!(rendered.contains("Target ayanamsa catalog:"));
    assert!(rendered.contains(
        "the full Swiss-Ephemeris-class ayanamsa catalog remains the long-term compatibility goal."
    ));
    assert!(rendered.contains("Release-specific coverage beyond baseline:"));
    assert!(rendered.contains("Alias mappings for built-in house systems:"));
    assert!(rendered.contains("Source-label aliases for built-in house systems:"));
    assert!(rendered.contains("Source-label aliases for built-in ayanamsas:"));
    assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
    assert!(rendered.contains("Polich Page"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains("Poli-equatorial"));
    assert!(rendered.contains("Poli-Equatorial"));
    assert!(rendered.contains("horizon/azimuth"));
    assert!(rendered.contains("Meridian table of houses"));
    assert!(rendered.contains("Meridian house system"));
    assert!(rendered.contains("Horizon house system"));
    assert!(rendered.contains("Whole-sign"));
    assert!(rendered.contains("Equal Midheaven house system"));
    assert!(rendered.contains("Equal Quadrant"));
    assert!(rendered.contains("Horizontal house system"));
    assert!(rendered.contains("Azimuth house system"));
    assert!(rendered.contains("Azimuthal house system"));
    assert!(rendered.contains("Carter's poli-equatorial"));
    assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
    assert!(rendered.contains("Babylonian Huber"));
    assert!(rendered.contains("Babylonian (House)"));
    assert!(rendered.contains("Babylonian (Sissy)"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
    assert!(rendered.contains("True Balarama"));
    assert!(rendered.contains("Aphoric"));
    assert!(rendered.contains("Takra"));
    assert!(rendered.contains("Galactic Equator (True)"));
    assert!(rendered.contains("Galactic Equator mid-Mula, Mula galactic equator, Galactic equator Mula -> Galactic Equator (Mula)"));
    assert!(rendered.contains("Valens Moon ayanamsa"));
}

#[test]
fn compatibility_profile_command_surfaces_recent_release_profile_entries() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Equal (MC) table of houses"));
    assert!(rendered.contains("Equal (MC) house system"));
    assert!(rendered.contains("Equal Midheaven house system"));
    assert!(rendered.contains("Equal from MC"));
    assert!(rendered.contains("Equal (from MC)"));
    assert!(rendered.contains("Equal (from MC) table of houses"));
    assert!(rendered.contains("Equal (1=Aries) table of houses"));
    assert!(rendered.contains("Equal (1=Aries) house system"));
    assert!(rendered.contains("Equal MC"));
    assert!(rendered.contains("Equal Midheaven"));
    assert!(rendered.contains("Babylonian 1"));
    assert!(rendered.contains("Babylonian 2"));
    assert!(rendered.contains("Babylonian 3"));
    assert!(rendered.contains("Vehlow Equal table of houses"));
    assert!(rendered.contains("Vehlow Equal house system"));
    assert!(rendered.contains("Vehlow equal"));
    assert!(rendered.contains("V equal Vehlow, Vehlow, Vehlow equal, Vehlow house system, Vehlow Equal house system, Vehlow-equal, Vehlow-equal table of houses, Vehlow Equal table of houses -> Vehlow Equal"));
    assert!(rendered.contains("Topocentric house system"));
    assert!(rendered.contains("Meridian house system"));
    assert!(rendered.contains("Horizon house system"));
    assert!(rendered.contains("Horizontal house system"));
    assert!(rendered.contains("Azimuth house system"));
    assert!(rendered.contains("Azimuthal house system"));
    assert!(rendered.contains("Carter's poli-equatorial"));
    assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
    assert!(rendered.contains("Albategnius"));
    assert!(rendered.contains("Gauquelin sectors"));
    assert!(rendered.contains("Equal table of houses"));
    assert!(rendered.contains("Equal (cusp 1 = Asc)"));
    assert!(rendered.contains("Whole Sign system"));
    assert!(rendered.contains("Whole Sign house system"));
    assert!(rendered.contains("Whole Sign (house 1 = Aries)"));
    assert!(rendered.contains("Morinus house system"));
    assert!(rendered.contains("Pullen SR (Sinusoidal Ratio) table of houses"));
    assert!(rendered.contains("Pullen SD (Sinusoidal Delta)"));
    assert!(rendered.contains("Pullen SD (Sinusoidal Delta) table of houses"));
    assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
    assert!(rendered.contains("Neo-Porphyry"));
    assert!(rendered.contains("WvA"));
    assert!(rendered.contains("Equal from MC"));
    assert!(rendered.contains("Equal (from MC)"));
    assert!(rendered.contains("Equal (from MC) table of houses"));
    assert!(rendered.contains("Makransky Sunshine"));
    assert!(rendered.contains("True Citra Paksha"));
    assert!(rendered.contains("True Chitra Paksha"));
    assert!(rendered.contains("True Chitrapaksha"));
    assert!(rendered.contains("Galactic Equator (Fiorenza)"));
    assert!(rendered.contains("Nick Anthony Fiorenza"));
    assert!(rendered.contains("Galactic Center (Cochrane)"));
    assert!(rendered.contains("Galactic Center (Gil Brand)"));
    assert!(rendered.contains("Gil Brand"));
    assert!(rendered.contains("P.V.R. Narasimha Rao"));
    assert!(rendered.contains("Bob Makransky"));
    assert!(rendered.contains("Sunshine table of houses, by Bob Makransky"));
    assert!(rendered.contains("Treindl Sunshine"));
    assert!(rendered.contains("Valens Moon"));
    assert!(rendered.contains("Babylonian (House Obs)"));
    assert!(rendered.contains("Sunil Sheoran / Vedic Sheoran / Sheoran ayanamsa spellings"));
    assert!(rendered.contains("True Sheoran"));
    assert!(rendered.contains("Lahiri (VP285)"));
    assert!(rendered.contains("Krishnamurti (VP291)"));
    assert!(rendered.contains("P.V.R. Narasimha Rao"));
    assert!(rendered.contains("B. V. Raman"));
    assert!(rendered.contains("Raman Ayanamsha"));
    assert!(rendered.contains("Raman ayanamsa"));
    assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
    assert!(rendered.contains("Polich/Page"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains("T topocentric"));
    assert!(rendered.contains("Poli-equatorial"));
    assert!(rendered.contains("horizon/azimuth"));
    assert!(rendered.contains("horizon/azimut"));
    assert!(rendered.contains("Horizon/Azimuth table of houses"));
    assert!(rendered.contains("U krusinski-pisa-goelzer"));
    assert!(rendered.contains("X axial rotation system/ Meridian houses"));
    assert!(rendered.contains("Zariel"));
    assert!(rendered.contains("Babylonian Huber"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Galactic Equator (True)"));
    assert!(rendered.contains("True galactic equator"));
    assert!(rendered.contains("Galactic equator true"));
    assert!(rendered.contains("Valens Moon ayanamsa"));
    assert!(rendered.contains("Lahiri (ICRC)"));
    assert!(rendered.contains("Lahiri (1940)"));
    assert!(rendered.contains("Yukteshwar"));
    assert!(rendered.contains("True Revati"));
    assert!(rendered.contains("True Pushya"));
    assert!(rendered.contains("Equal/MC = 10th"));
    assert!(rendered.contains("Equal Midheaven table of houses"));
    assert!(rendered.contains("Vehlow Equal table of houses"));
    assert!(rendered.contains("Vehlow equal"));
    assert!(rendered.contains("Wang"));
    assert!(rendered.contains("Aries houses"));
    assert!(rendered.contains("Fagan/Bradley"));
    assert!(rendered.contains("Usha Shashi"));
}

#[test]
fn compatibility_profile_command_surfaces_additional_equal_release_labels() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Equal/MC table of houses"));
    assert!(rendered.contains("Equal/MC house system"));
    assert!(rendered.contains("Equal/1=Aries table of houses"));
    assert!(rendered.contains("Equal/1=Aries house system"));
    assert!(rendered.contains("Equal/1=0 Aries"));
    assert!(rendered.contains("Equal (cusp 1 = 0° Aries)"));
    assert!(rendered.contains("Whole Sign (house 1 = Aries) table of houses"));
}

#[test]
fn compatibility_profile_command_surfaces_reference_frame_and_zero_point_entries() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Suryasiddhanta (499 CE)"));
    assert!(rendered.contains("Aryabhata (499 CE)"));
    assert!(rendered.contains("Sassanian"));
    assert!(rendered.contains("Sasanian"));
    assert!(rendered.contains("Zij al-Shah"));
    assert!(rendered.contains("DeLuce"));
    assert!(rendered.contains("Aryabhata (522 CE)"));
    assert!(rendered.contains("PVR Pushya-paksha"));
    assert!(rendered.contains("Galactic Center (Rgilbrand)"));
    assert!(rendered.contains("Galactic Center (Mardyks)"));
    assert!(rendered.contains("Skydram/Galactic Alignment"));
    assert!(rendered.contains("Skydram (Mardyks)"));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
    assert!(rendered.contains("Galactic Center (Cochrane)"));
    assert!(rendered.contains("Gal. Center = 0 Sag"));
    assert!(rendered.contains("Gal. Center = 0 Cap"));
}

#[test]
fn compatibility_profile_command_surfaces_additional_ayanamsa_transliterations() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Aryabhatan Kaliyuga"));
    assert!(rendered.contains("Krishnamurti-Senthilathiban"));
    assert!(rendered.contains("Sri Yukteshwar"));
    assert!(rendered.contains("Shri Yukteshwar"));
    assert!(rendered.contains("De Luce"));
}

#[test]
fn compatibility_profile_command_surfaces_additional_reference_mode_entries() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Babylonian (Britton)"));
    assert!(rendered.contains("Babylonian/Britton"));
    assert!(rendered.contains("Babylonian (Aldebaran)"));
    assert!(rendered.contains("Babylonian/Aldebaran = 15 Tau"));
    assert!(rendered.contains("Babylonian (Eta Piscium)"));
    assert!(rendered.contains("Babylonian/Eta Piscium"));
    assert!(rendered.contains("Babylonian Eta Piscium"));
    assert!(rendered.contains("Eta Piscium"));
    assert!(rendered.contains("Hipparchus"));
    assert!(rendered.contains("Djwhal Khul"));
    assert!(rendered.contains("Udayagiri"));
    assert!(rendered.contains("True Mula"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun)"));
    assert!(rendered.contains("Aryabhata (Mean Sun)"));
    assert!(rendered.contains("Galactic Equator (IAU 1958)"));
    assert!(rendered.contains("Galactic Equator (Mula)"));
}

#[test]
fn compatibility_profile_command_surfaces_remaining_ayanamsa_and_reference_aliases() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Suryasiddhanta (Revati)"));
    assert!(rendered.contains("Suryasiddhanta (Citra)"));
    assert!(rendered.contains("True Pushya (PVRN Rao)"));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula)"));
    assert!(rendered.contains("Dhruva Galactic Center Middle Mula"));
    assert!(rendered.contains("Dhruva/Gal.Center/Mula (Wilhelm)"));
    assert!(rendered.contains("Mula Wilhelm"));
    assert!(rendered.contains("Wilhelm"));
    assert!(rendered.contains("Middle of Mula"));
}

#[test]
fn compatibility_profile_command_surfaces_house_table_code_spellings() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("A equal, E equal = A"));
    assert!(rendered.contains("D equal / MC"));
    assert!(rendered.contains("N, Equal/1=Aries"));
    assert!(rendered.contains("S, S sripati"));
    assert!(rendered.contains("I, I sunshine"));
    assert!(rendered.contains("W equal, whole sign"));
    assert!(rendered.contains("V equal Vehlow"));
    assert!(rendered.contains("T, Polich-Page"));
    assert!(rendered.contains("U, Krusinski"));
    assert!(rendered.contains("X, Meridian houses"));
    assert!(rendered.contains("Y APC houses"));
    assert!(rendered.contains("M, Morinus houses"));
    assert!(rendered.contains("G, Gauquelin"));
}

#[test]
fn compatibility_profile_command_surfaces_ayanamsa_code_spellings() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("J2000.0 -> J2000"));
    assert!(rendered.contains("J1900.0 -> J1900"));
    assert!(rendered.contains("B1950.0 -> B1950"));
    assert!(rendered.contains(
        "SS Revati, Suryasiddhanta Revati, Surya Siddhanta Revati -> Suryasiddhanta (Revati)"
    ));
    assert!(rendered.contains(
        "SS Citra, Suryasiddhanta Citra, Surya Siddhanta Citra -> Suryasiddhanta (Citra)"
    ));
    assert!(rendered.contains("Galact. Center = 0 Sag, Gal. Center = 0 Sag -> Galactic Center"));
    assert!(rendered.contains("Gal. Eq."));
}

#[test]
fn release_notes_command_renders_the_release_notes() {
    let rendered = render_cli(&["release-notes"]).expect("release notes should render");
    assert!(rendered.contains("Release notes"));
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("API stability posture:"));
    assert!(rendered.contains("Deprecation policy:"));
    assert!(rendered.contains("Release-specific coverage:"));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("selected asteroid coverage"));
    assert!(rendered.contains("WvA"));
    assert!(rendered.contains("Selected asteroid evidence: 6 exact J2000 samples"));
    assert!(rendered.contains("Selected asteroid batch parity: 6 requests across 6 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); frame mix: 3 ecliptic, 3 equatorial; batch/single parity preserved"));
    assert!(rendered.contains("Reference snapshot coverage: 357 rows across 16 bodies and 31 epochs (95 asteroid rows; JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
    assert!(rendered.contains("Reference snapshot body-class coverage: major bodies: 262 rows across 10 bodies and 31 epochs; major windows: "));
    assert!(rendered.contains(&reference_snapshot_pre_bridge_boundary_summary_for_report()));
    assert!(
        rendered.contains(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report())
    );
    assert!(rendered.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_dense_boundary_summary_for_report()));
    assert!(rendered
        .contains("selected asteroids: 95 rows across 6 bodies and 17 epochs; asteroid windows: "));
    assert!(rendered.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2500_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_manifest_summary_for_report()));
    assert!(
        rendered.contains("Comparison snapshot coverage: 232 rows across 10 bodies and 28 epochs")
    );
    assert!(rendered.contains("asteroid:433-Eros"));
    assert!(rendered.contains("Validation reference points:"));
    assert!(rendered.contains("Compatibility caveats:"));
    assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
    assert!(rendered.contains("Neo-Porphyry"));
    assert!(rendered.contains("Makransky Sunshine"));
    assert!(rendered.contains("Babylonian Huber"));
    assert!(rendered.contains("Babylonian (Britton)"));
    assert!(rendered.contains("Babylonian (Aldebaran)"));
    assert!(rendered.contains("Babylonian (Eta Piscium)"));
    assert!(rendered.contains("Babylonian (True Geoc)"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Babylonian (House Obs)"));
    assert!(rendered.contains("Equal MC"));
    assert!(rendered.contains("Equal/MC house system"));
    assert!(rendered.contains("Equal Midheaven"));
    assert!(rendered.contains("Equal Midheaven house system"));
    assert!(rendered.contains("Babylonian (Kugler 1)"));
    assert!(rendered.contains("Krusinski/Pisa/Goelzer"));
    assert!(rendered.contains("Equal/MC = 10th"));
    assert!(rendered.contains("Galactic Equator (True)"));
    assert!(rendered.contains("Galactic Equator (IAU 1958)"));
    assert!(rendered.contains("Valens Moon ayanamsa"));
}

#[test]
fn compatibility_profile_summary_command_renders_the_summary() {
    let rendered = render_cli(&["compatibility-profile-summary"])
        .expect("compatibility profile summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Compatibility profile summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    let coverage = metadata_coverage();
    assert!(rendered.contains("House systems:"));
    assert!(rendered.contains("House systems: 25 total (12 baseline, 13 release-specific)"));
    assert!(rendered.contains(&format!(
        "Latitude-sensitive house constraints: {}",
        profile.latitude_sensitive_house_constraints_summary_line()
    )));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("Ayanamsas:"));
    assert!(rendered.contains("Compatibility caveats documented:"));
    assert!(rendered.contains(&format!(
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    )));
    assert!(rendered.contains(profile.known_gaps[0]));
    assert!(rendered.contains(profile.known_gaps[1]));
    assert!(rendered.contains("ayanamsa sidereal metadata: 53/59 entries with both a reference epoch and offset; custom-definition-only=6 labels: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs); missing-sidereal-metadata=none"));
    assert!(rendered.contains(&format!(
        "House formula families: {}",
        profile.house_formula_families_summary_line()
    )));
    let catalog_inventory_summary = render_cli(&["catalog-inventory-summary"])
        .expect("catalog inventory summary should render");
    assert_eq!(
        catalog_inventory_summary,
        profile
            .validated_catalog_inventory_summary_line()
            .expect("catalog inventory summary should validate")
    );
    assert_eq!(
        catalog_inventory_summary,
        validated_catalog_inventory_summary_for_report()
            .expect("catalog inventory summary helper should validate")
    );
    assert_eq!(
        render_cli(&["catalog-inventory"]).expect("catalog inventory alias should render"),
        catalog_inventory_summary
    );
    let catalog_inventory_summary_error = render_cli(&["catalog-inventory-summary", "extra"])
        .expect_err("catalog inventory summary should reject extra arguments");
    assert_eq!(
        catalog_inventory_summary_error,
        "catalog-inventory-summary does not accept extra arguments"
    );
    let catalog_inventory_alias_error = render_cli(&["catalog-inventory", "extra"])
        .expect_err("catalog inventory alias should reject extra arguments");
    assert_eq!(
        catalog_inventory_alias_error,
        "catalog-inventory does not accept extra arguments"
    );
    let catalog_posture_summary =
        render_cli(&["catalog-posture-summary"]).expect("catalog posture summary should render");
    assert_eq!(
        catalog_posture_summary,
        profile
            .validated_catalog_posture_summary_line()
            .expect("catalog posture summary should validate")
    );
    assert_eq!(
        catalog_posture_summary,
        core_validated_catalog_posture_summary_for_report()
            .expect("catalog posture summary helper should validate")
    );
    assert_eq!(
        render_cli(&["catalog-posture"]).expect("catalog posture alias should render"),
        catalog_posture_summary
    );
    assert_eq!(
        render_cli(&["catalog-posture-summary", "extra"])
            .expect_err("catalog posture summary should reject extra arguments"),
        "catalog-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["catalog-posture", "extra"])
            .expect_err("catalog posture alias should reject extra arguments"),
        "catalog-posture does not accept extra arguments"
    );
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_house_scope.join("; ")));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
    assert!(rendered.contains(&coverage.summary_line()));
    assert!(rendered.contains("ayanamsa catalog validation: ok"));
    let caveats_summary = render_cli(&["compatibility-caveats-summary"])
        .expect("compatibility caveats summary should render");
    let profile = current_compatibility_profile();
    assert!(caveats_summary.contains("Compatibility caveats summary"));
    assert!(caveats_summary.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(caveats_summary.contains("Compatibility caveats: 2"));
    assert!(caveats_summary.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(caveats_summary.contains("Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"));
    assert!(caveats_summary.contains("Latitude-sensitive house constraints: 8 ("));
    assert!(caveats_summary.contains("Placidus ["));
    assert!(caveats_summary.contains("Koch ["));
    assert!(caveats_summary.contains("Topocentric ["));
    assert!(caveats_summary.contains("Gauquelin sectors ["));
    assert!(caveats_summary.contains("Descriptor-only ayanamsa labels: 6 (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs))"));
    assert!(caveats_summary.contains(profile.known_gaps[0]));
    assert!(caveats_summary.contains(profile.known_gaps[1]));
    assert_eq!(
        render_cli(&["compatibility-caveats"]).expect("compatibility caveats alias should render"),
        caveats_summary
    );
    assert_eq!(
        render_cli(&["compatibility-caveats-summary", "extra"]).unwrap_err(),
        "compatibility-caveats-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["compatibility-caveats", "extra"]).unwrap_err(),
        "compatibility-caveats does not accept extra arguments"
    );
    let known_gaps_summary =
        render_cli(&["known-gaps-summary"]).expect("known gaps summary should render");
    assert_eq!(
        known_gaps_summary,
        format!("Known gaps: {}", profile.known_gaps_summary_line())
    );
    assert_eq!(
        render_cli(&["known-gaps"]).expect("known gaps alias should render"),
        known_gaps_summary
    );
    assert_eq!(
        render_cli(&["known-gaps-summary", "extra"]).unwrap_err(),
        "known-gaps-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["known-gaps", "extra"]).unwrap_err(),
        "known-gaps does not accept extra arguments"
    );
    let ayanamsa_metadata_coverage_summary = render_cli(&["ayanamsa-metadata-coverage-summary"])
        .expect("ayanamsa metadata coverage summary should render");
    assert_eq!(
        ayanamsa_metadata_coverage_summary,
        super::format_ayanamsa_metadata_coverage_for_report()
    );
    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
    assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
    assert!(rendered.contains("Usha Shashi: epoch=JD 2415020.5; offset=18.66096111111111°"));
    assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
    assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
    assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
    assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("True Revati: epoch=JD 1926902.658267; offset=0°"));
    assert!(rendered.contains("True Mula: epoch=JD 1805889.671313; offset=0°"));
    assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
    assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
    assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
    assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
    assert!(rendered.contains("J2000: epoch=JD 2451545; offset=23.85317778°"));
    assert!(rendered.contains("J1900: epoch=JD 2415020; offset=0°"));
    assert!(rendered.contains("B1950: epoch=JD 2433281.5; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 2): epoch=JD 1797039.20682; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 3): epoch=JD 1774637.420172; offset=0°"));
    assert!(
        rendered.contains("Babylonian (Huber): epoch=JD 1721171.5; offset=-0.12055555555555555°")
    );
    assert!(rendered.contains("Babylonian (Eta Piscium): epoch=JD 1807871.964797; offset=0°"));
    assert!(rendered.contains("Babylonian (Aldebaran): epoch=JD 1801643.133503; offset=0°"));
    assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
    assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
    assert!(rendered.contains("Sheoran: epoch=JD 1789947.090881; offset=0°"));
    assert!(rendered.contains("True Sheoran: epoch="));
    assert!(rendered.contains("Hipparchus: epoch=JD 1674484; offset=-9.333333333333334°"));
    assert!(rendered.contains("Djwhal Khul: epoch=JD 1706703.948006; offset=0°"));
    assert!(rendered.contains("Galactic Center: epoch="));
    assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
    assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
    assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm): epoch="));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
    assert!(rendered.contains("Galactic Equator (IAU 1958): epoch=JD 1667118.376332; offset=0°"));
    assert!(rendered.contains("Galactic Equator (True): epoch=JD 1665728.603158; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Mula): epoch=JD 1840527.426262; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
    assert!(rendered.contains("Suryasiddhanta (499 CE): epoch=JD 1903396.8128653935; offset=0°"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun): epoch=JD 1909045.584433; offset=0°"));
    assert!(rendered.contains("Aryabhata (Mean Sun): epoch=JD 1909650.815331; offset=0°"));
    assert!(rendered.contains("Aryabhata (522 CE): epoch=JD 1911797.740782; offset=0°"));
    assert!(rendered.contains("Release-specific house-system canonical names: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("Release-specific ayanamsa canonical names:"));
    assert!(rendered.contains("Release-specific ayanamsa canonical names: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
    assert!(rendered.contains("Custom-definition labels: 9"));
    assert!(rendered.contains(&format!(
        "Custom-definition label names: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
    assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(rendered.contains("Compatibility caveats: 2"));
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn compatibility_profile_summary_text_validation_rejects_split_drift() {
    let profile = validated_compatibility_profile_for_report()
        .expect("compatibility profile should validate");
    let release_profiles = validated_release_profile_identifiers_for_report()
        .expect("release profile identifiers should validate");
    let rendered = render_compatibility_profile_summary_text();
    let expected_house_line = format!(
        "House systems: {} total ({} baseline, {} release-specific)",
        profile.house_systems.len(),
        profile.baseline_house_systems.len(),
        profile.release_house_systems.len()
    );
    let expected_ayanamsa_line = format!(
        "Ayanamsas: {} total ({} baseline, {} release-specific)",
        profile.ayanamsas.len(),
        profile.baseline_ayanamsas.len(),
        profile.release_ayanamsas.len()
    );

    assert!(
        validate_compatibility_profile_summary_text(&rendered, &profile, &release_profiles,)
            .is_ok()
    );

    let tampered = rendered.replacen(&expected_house_line, "House systems: split omitted", 1);
    let error = validate_compatibility_profile_summary_text(&tampered, &profile, &release_profiles)
        .expect_err("tampered compatibility profile summary should fail validation");
    assert!(error.contains("baseline/release split mismatch"));

    let tampered = rendered.replacen(&expected_ayanamsa_line, "Ayanamsas: split omitted", 1);
    let error = validate_compatibility_profile_summary_text(&tampered, &profile, &release_profiles)
        .expect_err("tampered compatibility profile summary should fail validation");
    assert!(error.contains("baseline/release split mismatch"));
}

#[test]
fn custom_definition_ayanamsa_labels_summary_command_renders_the_labels() {
    let profile = current_compatibility_profile();
    let rendered = render_cli(&["custom-definition-ayanamsa-labels-summary"])
        .expect("custom-definition ayanamsa labels summary should render");

    assert_eq!(
        render_cli(&["custom-definition-ayanamsa-labels"]).unwrap(),
        rendered
    );
    assert_eq!(
        rendered,
        profile
            .validated_custom_definition_ayanamsa_labels_summary_line()
            .expect("custom-definition ayanamsa labels summary should validate")
    );
    assert_eq!(
        render_cli(&["custom-definition-ayanamsa-labels-summary", "extra"]).unwrap_err(),
        "custom-definition-ayanamsa-labels-summary does not accept extra arguments"
    );
    assert!(rendered.contains("Babylonian (House)"));
    assert!(rendered.contains("Babylonian (True Geoc)"));
}

#[test]
fn release_specific_canonical_name_summary_commands_render_the_labels() {
    let profile = current_compatibility_profile();

    let house_names = render_cli(&["release-house-system-canonical-names-summary"])
        .expect("release-specific house-system canonical names summary should render");
    assert_eq!(
        render_cli(&["release-house-system-canonical-names"]).unwrap(),
        house_names
    );
    assert_eq!(
        house_names,
        format!(
            "Release-specific house-system canonical names: {}",
            profile
                .validated_release_house_system_canonical_names_summary_line()
                .expect("release-specific house-system canonical names should validate")
        )
    );
    assert_eq!(
        render_cli(&["release-house-system-canonical-names-summary", "extra"]).unwrap_err(),
        "release-house-system-canonical-names-summary does not accept extra arguments"
    );
    assert!(house_names.contains("Equal (MC)"));
    assert!(house_names.contains("Gauquelin sectors"));

    let ayanamsa_names = render_cli(&["release-ayanamsa-canonical-names-summary"])
        .expect("release-specific ayanamsa canonical names summary should render");
    assert_eq!(
        render_cli(&["release-ayanamsa-canonical-names"]).unwrap(),
        ayanamsa_names
    );
    assert_eq!(
        ayanamsa_names,
        format!(
            "Release-specific ayanamsa canonical names: {}",
            profile
                .validated_release_ayanamsa_canonical_names_summary_line()
                .expect("release-specific ayanamsa canonical names should validate")
        )
    );
    assert_eq!(
        render_cli(&["release-ayanamsa-canonical-names-summary", "extra"]).unwrap_err(),
        "release-ayanamsa-canonical-names-summary does not accept extra arguments"
    );
    assert!(ayanamsa_names.contains("True Citra"));
    assert!(ayanamsa_names.contains("Valens Moon"));
}

#[test]
fn ayanamsa_reference_offsets_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-reference-offsets-summary"])
        .expect("ayanamsa reference offsets summary should render");
    assert_eq!(
        render_cli(&["ayanamsa-reference-offsets"]).unwrap(),
        rendered
    );

    let summary =
        summarize_ayanamsa_reference_offsets().expect("reference offsets summary should validate");
    assert_eq!(
        validated_ayanamsa_reference_offsets_summary_for_report(&summary),
        Ok(summary.to_string())
    );

    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
    assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
    assert!(rendered.contains("Usha Shashi: epoch=JD 2415020.5; offset=18.66096111111111°"));
    assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
    assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
    assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
    assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("True Revati: epoch=JD 1926902.658267; offset=0°"));
    assert!(rendered.contains("True Mula: epoch=JD 1805889.671313; offset=0°"));
    assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
    assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
    assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
    assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
    assert!(rendered.contains("J2000: epoch=JD 2451545; offset=23.85317778°"));
    assert!(rendered.contains("J1900: epoch=JD 2415020; offset=0°"));
    assert!(rendered.contains("B1950: epoch=JD 2433281.5; offset=0°"));
    assert!(rendered.contains("True Pushya: epoch=JD 1855769.248315; offset=0°"));
    assert!(rendered.contains("Udayagiri: epoch=JD 1825235.164583; offset=0°"));
    assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
    assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
    assert!(rendered.contains("Sheoran: epoch=JD 1789947.090881; offset=0°"));
    assert!(rendered.contains("True Sheoran: epoch="));
    assert!(rendered.contains("Hipparchus: epoch=JD 1674484; offset=-9.333333333333334°"));
    assert!(rendered.contains("Djwhal Khul: epoch=JD 1706703.948006; offset=0°"));
    assert!(rendered.contains("Galactic Center: epoch="));
    assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
    assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
    assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm): epoch="));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
    assert!(rendered.contains("Galactic Equator (IAU 1958): epoch=JD 1667118.376332; offset=0°"));
    assert!(rendered.contains("Galactic Equator (True): epoch=JD 1665728.603158; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Mula): epoch=JD 1840527.426262; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
    assert!(rendered.contains("Suryasiddhanta (499 CE): epoch=JD 1903396.8128653935; offset=0°"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun): epoch=JD 1909045.584433; offset=0°"));
    assert!(rendered.contains("Aryabhata (Mean Sun): epoch=JD 1909650.815331; offset=0°"));
    assert!(rendered.contains("Aryabhata (522 CE): epoch=JD 1911797.740782; offset=0°"));
}

#[test]
fn ayanamsa_provenance_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-provenance-summary"])
        .expect("ayanamsa provenance summary should render");
    assert_eq!(render_cli(&["ayanamsa-provenance"]).unwrap(), rendered);
    assert_eq!(rendered, format_ayanamsa_provenance_for_report());
    assert!(rendered.contains("Ayanamsa provenance: representative provenance examples:"));
    assert!(rendered.contains("True Citra —"));
    assert!(rendered.contains("True Revati —"));
    assert!(rendered.contains("True Mula —"));
    assert!(rendered.contains("True Pushya —"));
    assert!(rendered.contains("Udayagiri —"));
    assert!(rendered.contains("True Sheoran —"));
    assert!(rendered.contains("Babylonian (Britton) —"));
    assert!(rendered.contains("Galactic Center (Rgilbrand) —"));
    assert!(rendered.contains("Babylonian (Kugler 1) —"));
    assert!(rendered.contains("Galactic Equator —"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun) —"));
    assert!(rendered.contains("Aryabhata (522 CE) —"));
    assert!(rendered.contains("Valens Moon —"));
    assert_eq!(
        render_cli(&["ayanamsa-provenance-summary", "extra"])
            .expect_err("ayanamsa provenance summary should reject extra arguments"),
        "ayanamsa-provenance-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["ayanamsa-provenance", "extra"])
            .expect_err("ayanamsa provenance alias should reject extra arguments"),
        "ayanamsa-provenance does not accept extra arguments"
    );
}

#[test]
fn ayanamsa_audit_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["ayanamsa-audit-summary"]).expect("ayanamsa audit summary should render");
    assert_eq!(render_cli(&["ayanamsa-audit"]).unwrap(), rendered);
    assert_eq!(rendered, format_ayanamsa_audit_for_report());
    assert!(rendered.contains("Ayanamsa audit: ayanamsa catalog validation:"));
    assert!(rendered.contains("ayanamsa sidereal metadata:"));
    assert!(rendered.contains("Ayanamsa reference offsets:"));
    assert!(rendered.contains("Ayanamsa provenance:"));
    assert_eq!(
        render_cli(&["ayanamsa-audit-summary", "extra"])
            .expect_err("ayanamsa audit summary should reject extra arguments"),
        "ayanamsa-audit-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["ayanamsa-audit", "extra"])
            .expect_err("ayanamsa audit alias should reject extra arguments"),
        "ayanamsa-audit does not accept extra arguments"
    );
}

#[test]
fn ayanamsa_provenance_summary_validated_summary_line_rejects_note_drift() {
    let summary = AyanamsaProvenanceSummary {
        examples: vec![AyanamsaProvenanceExample {
            canonical_name: "Example",
            provenance_note: " drifted note ",
        }],
    };

    assert!(summary.summary_line().contains(" drifted note "));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn ayanamsa_metadata_coverage_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-metadata-coverage-summary"])
        .expect("ayanamsa metadata coverage summary should render");
    assert_eq!(
        render_cli(&["ayanamsa-metadata-coverage"]).unwrap(),
        rendered
    );

    assert_eq!(rendered, metadata_coverage().summary_line());
    assert!(rendered.contains(
            "ayanamsa sidereal metadata: 53/59 entries with both a reference epoch and offset; custom-definition-only=6 labels: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs); missing-sidereal-metadata=none"
        ));
}

#[test]
fn house_validation_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["house-validation-summary"]).expect("house validation summary should render");

    assert!(rendered.contains("House validation corpus: 9 scenarios"));
    assert!(
        rendered.contains("formula families: Equal, Whole Sign, Quadrant, Equatorial projection")
    );
    assert!(rendered.contains("latitude-sensitive systems: Koch, Placidus, Topocentric"));
    assert_eq!(rendered, house_validation_summary_for_report());
}

#[test]
fn house_validation_alias_command_renders_the_summary() {
    let rendered = render_cli(&["house-validation"]).expect("house validation alias should render");

    assert_eq!(rendered, house_validation_summary_for_report());
    assert_eq!(
        render_cli(&["house-validation", "extra"])
            .expect_err("house validation alias should reject extra arguments"),
        "house-validation does not accept extra arguments"
    );
}

#[test]
fn release_house_validation_summary_command_renders_the_release_summary() {
    let rendered = render_cli(&["release-house-validation-summary"])
        .expect("release house validation summary should render");

    assert!(rendered.contains("House code aliases:"));
    assert_eq!(rendered, release_house_validation_summary_for_report());
    assert_eq!(
        render_cli(&["release-house-validation"])
            .expect("release house validation alias should render"),
        release_house_validation_summary_for_report()
    );
    assert_eq!(
        render_cli(&["release-house-validation", "extra"])
            .expect_err("release house validation alias should reject extra arguments"),
        "release-house-validation does not accept extra arguments"
    );
}

#[test]
fn release_house_validation_summary_validation_rejects_drift() {
    let summary = release_house_validation_summary_for_report();
    let drifted_summary = format!("{summary} drift");

    let error = ensure_release_house_validation_summary_matches_current_rendering(&drifted_summary)
        .expect_err("drifted release house validation summary should be rejected");

    assert!(error
        .to_string()
        .contains("release house validation summary no longer matches"));
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
fn house_formula_families_summary_command_renders_the_family_list() {
    let rendered = render_cli(&["house-formula-families-summary"])
        .expect("house formula families summary should render");

    assert!(rendered.contains("Equal"));
    assert!(rendered.contains("Whole Sign"));
    assert!(rendered.contains("Quadrant"));
    assert_eq!(rendered, format_house_formula_families_for_report());
}

#[test]
fn house_formula_families_alias_command_renders_the_family_list() {
    let rendered = render_cli(&["house-formula-families"])
        .expect("house formula families alias should render");

    assert_eq!(rendered, format_house_formula_families_for_report());
}

#[test]
fn house_latitude_sensitive_summary_command_renders_the_system_list() {
    let rendered = render_cli(&["house-latitude-sensitive-summary"])
        .expect("latitude-sensitive house systems summary should render");

    assert_eq!(
            rendered,
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        );
    assert_eq!(
        rendered,
        format_latitude_sensitive_house_systems_for_report()
    );
}

#[test]
fn house_latitude_sensitive_alias_command_renders_the_system_list() {
    let rendered = render_cli(&["house-latitude-sensitive"])
        .expect("latitude-sensitive house systems alias should render");

    assert_eq!(
        rendered,
        format_latitude_sensitive_house_systems_for_report()
    );
}

#[test]
fn house_latitude_sensitive_constraints_summary_command_renders_the_constraints() {
    let rendered = render_cli(&["house-latitude-sensitive-constraints-summary"])
        .expect("latitude-sensitive house constraints summary should render");

    assert_eq!(
        render_cli(&["house-latitude-sensitive-constraints"]).unwrap(),
        rendered
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive-constraints-summary", "extra"]).unwrap_err(),
        "house-latitude-sensitive-constraints-summary does not accept extra arguments"
    );
    assert_eq!(
        rendered,
        format_latitude_sensitive_house_constraints_for_report()
    );
    assert!(rendered.contains("Latitude-sensitive house constraints: 8"));
    assert!(rendered
        .contains("Placidus [Quadrant system; can fail or become unstable at extreme latitudes.]"));
    assert!(rendered.contains("Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]"));
}

#[test]
fn house_latitude_sensitive_summary_command_rejects_extra_arguments() {
    assert_eq!(
        render_cli(&["house-latitude-sensitive-summary", "extra"])
            .expect_err("latitude-sensitive house systems summary should reject extra arguments"),
        "house-latitude-sensitive-summary does not accept extra arguments"
    );
}

#[test]
fn house_latitude_sensitive_failure_modes_summary_command_renders_the_failure_modes() {
    let rendered = render_cli(&["house-latitude-sensitive-failure-modes-summary"])
        .expect("latitude-sensitive house failure modes summary should render");

    assert_eq!(
        rendered,
        format_latitude_sensitive_house_failure_modes_for_report()
    );
}

#[test]
fn house_latitude_sensitive_failure_modes_alias_command_renders_the_failure_modes() {
    let rendered = render_cli(&["house-latitude-sensitive-failure-modes"])
        .expect("latitude-sensitive house failure modes alias should render");

    assert_eq!(
        rendered,
        format_latitude_sensitive_house_failure_modes_for_report()
    );
}

#[test]
fn house_latitude_sensitive_failure_modes_summary_command_rejects_extra_arguments() {
    assert_eq!(
        render_cli(&["house-latitude-sensitive-failure-modes-summary", "extra"]).expect_err(
            "latitude-sensitive house failure modes summary should reject extra arguments"
        ),
        "house-latitude-sensitive-failure-modes-summary does not accept extra arguments"
    );
}

#[test]
fn house_code_aliases_summary_command_renders_the_alias_table() {
    let rendered = render_cli(&["house-code-aliases-summary"])
        .expect("house code aliases summary should render");

    assert!(rendered.contains("P -> Placidus"));
    assert!(rendered.contains("T -> Topocentric"));
    assert!(rendered.contains("X -> Meridian"));
    assert_eq!(rendered, format_house_code_aliases_for_report());
}

#[test]
fn target_house_scope_summary_command_renders_the_scope() {
    let rendered = render_cli(&["target-house-scope-summary"])
        .expect("target house scope summary should render");

    assert_eq!(rendered, render_target_house_scope_summary());
    assert_eq!(render_cli(&["target-house-scope"]).unwrap(), rendered);
    assert!(rendered.contains("Target house scope:"));
    assert!(rendered.contains("Baseline milestone:"));
}

#[test]
fn target_ayanamsa_scope_summary_command_renders_the_scope() {
    let rendered = render_cli(&["target-ayanamsa-scope-summary"])
        .expect("target ayanamsa scope summary should render");

    assert_eq!(rendered, render_target_ayanamsa_scope_summary());
    assert_eq!(render_cli(&["target-ayanamsa-scope"]).unwrap(), rendered);
    assert!(rendered.contains("Target ayanamsa scope:"));
    assert!(rendered.contains("Baseline milestone:"));
}

#[test]
fn house_code_alias_summary_command_rejects_extra_arguments() {
    let error = render_cli(&["house-code-alias-summary", "extra"])
        .expect_err("house code alias summary should reject extra arguments");

    assert_eq!(
        error,
        "house-code-alias-summary does not accept extra arguments"
    );
}

#[test]
fn ayanamsa_catalog_validation_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-catalog-validation-summary"])
        .expect("ayanamsa catalog validation summary should render");

    assert_eq!(
        render_cli(&["ayanamsa-catalog-validation"]).unwrap(),
        rendered
    );
    assert!(rendered.contains("ayanamsa catalog validation: ok"));
    assert!(rendered.contains("baseline=5, release=54"));
    assert_eq!(
        rendered,
        ayanamsa_catalog_validation_summary()
            .validated_summary_line()
            .expect("ayanamsa catalog validation summary should validate")
    );
}

#[test]
fn lunar_theory_catalog_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-theory-catalog-summary"])
        .expect("lunar theory catalog summary should render");

    assert_eq!(render_cli(&["lunar-theory-catalog"]).unwrap(), rendered);
    assert_eq!(rendered, lunar_theory_catalog_summary_for_report());
    assert!(rendered.contains("lunar theory catalog: 1 entry, 1 selected entry"));
}

#[test]
fn lunar_theory_catalog_validation_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-theory-catalog-validation-summary"])
        .expect("lunar theory catalog validation summary should render");

    assert_eq!(
        render_cli(&["lunar-theory-catalog-validation"]).unwrap(),
        rendered
    );
    assert_eq!(
        rendered,
        validated_lunar_theory_catalog_validation_summary_for_report()
    );
    assert!(rendered.contains("lunar theory catalog validation: ok"));
}

#[test]
fn lunar_theory_catalog_validation_summary_matches_current_rendering() {
    let summary = validated_lunar_theory_catalog_validation_summary_for_report();

    ensure_lunar_theory_catalog_validation_summary_matches_current_rendering(&summary)
        .expect("lunar theory catalog validation summary should match the current rendering");
}

#[test]
fn lunar_theory_catalog_validation_summary_validation_rejects_drift() {
    let summary = validated_lunar_theory_catalog_validation_summary_for_report();
    let drifted_summary = summary.replace("aliases=1", "aliases=2");

    let error =
        ensure_lunar_theory_catalog_validation_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted lunar theory catalog validation summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current lunar theory catalog posture"));
}

#[test]
fn lunar_reference_evidence_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-reference-evidence-summary"])
        .expect("lunar reference evidence summary should render");

    assert_eq!(render_cli(&["lunar-reference-evidence"]).unwrap(), rendered);
    assert_eq!(
        rendered,
        format!(
            "Lunar reference evidence summary\n{}\n",
            lunar_reference_evidence_summary_for_report()
        )
    );
    assert!(rendered.contains("Lunar reference evidence summary"));
    assert!(rendered.contains("lunar reference evidence: 9 samples across 5 bodies"));
}

#[test]
fn lunar_theory_source_selection_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-theory-source-selection-summary"])
        .expect("lunar theory source selection summary should render");

    assert_eq!(
        render_cli(&["lunar-theory-source-selection"]).unwrap(),
        rendered
    );
    assert_eq!(
        rendered,
        pleiades_elp::lunar_theory_source_selection_summary_for_report()
    );
    assert!(rendered.contains("lunar source selection:"));
    assert!(rendered.contains("Meeus-style truncated lunar baseline"));
}

#[test]
fn ayanamsa_reference_offsets_summary_rejects_duplicate_labels() {
    let summary = AyanamsaReferenceOffsetsSummary {
        examples: vec![
            AyanamsaReferenceOffsetExample {
                canonical_name: "Lahiri",
                epoch: JulianDay::from_days(2_435_553.5),
                offset_degrees: Angle::from_degrees(23.245_524_743),
            },
            AyanamsaReferenceOffsetExample {
                canonical_name: "lahiri",
                epoch: JulianDay::from_days(2_451_544.5),
                offset_degrees: Angle::from_degrees(25.0),
            },
        ],
    };

    let error = validated_ayanamsa_reference_offsets_summary_for_report(&summary)
        .expect_err("duplicate ayanamsa reference labels should fail validation");
    assert!(error
        .to_string()
        .contains("ayanamsa reference offsets contains a case-insensitive duplicate name"));
}

#[test]
fn compatibility_profile_report_helper_validates_the_current_profile() {
    let profile = validated_compatibility_profile_for_report()
        .expect("compatibility profile should validate");
    assert_eq!(
        profile.profile_id,
        current_compatibility_profile().profile_id
    );
}

#[test]
fn compatibility_profile_verification_command_checks_the_catalogs() {
    let rendered = render_cli(&["verify-compatibility-profile"])
        .expect("compatibility profile verification should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Compatibility profile verification"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains("House systems verified: 25 descriptors, 181 labels"));
    assert!(rendered.contains(&format!(
        "House code aliases verified: {} short-form labels",
        profile.house_code_alias_count()
    )));
    assert!(rendered.contains(&format!(
            "Alias uniqueness checks: house={} aliases, ayanamsa={} aliases; exact and case-insensitive labels verified",
            profile
                .house_systems
                .iter()
                .map(|entry| 1 + entry.aliases.len())
                .sum::<usize>()
                - profile.house_systems.len(),
            profile
                .ayanamsas
                .iter()
                .map(|entry| 1 + entry.aliases.len())
                .sum::<usize>()
                - profile.ayanamsas.len()
        )));
    assert!(rendered.contains(
            "Latitude-sensitive house systems verified: 8 descriptors, 8 labels (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
    assert!(rendered.contains("Ayanamsas verified: 59 descriptors, 245 labels"));
    assert!(rendered.contains("House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"));
    assert!(rendered.contains(
            "Ayanamsa reference metadata verified: 53 descriptors with epoch/offset metadata, 6 metadata gaps"
        ));
    assert!(rendered.contains(&format!(
            "Catalog posture: house systems=25 descriptors (8 constrained, 17 unconstrained); ayanamsas=59 descriptors (53 metadata-bearing, 6 descriptor-only); ayanamsa alias-bearing entries={}; ayanamsa metadata gaps=6; custom-definition labels=9; custom-definition ayanamsa labels=6; known gaps={}",
            profile
                .ayanamsas
                .iter()
                .filter(|entry| !entry.aliases.is_empty())
                .count(),
            profile.known_gaps_summary_line()
        )));
    assert!(rendered.contains(&format!(
        "Custom-definition labels verified: {} labels, all remain custom-definition territory",
        profile.custom_definition_labels.len()
    )));
    assert!(rendered.contains("Baseline/release slices:"));
    assert!(rendered.contains("Release-specific house-system canonical names verified: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
    assert!(rendered.contains("Release-specific ayanamsa canonical names verified: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
    assert!(rendered.contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
    assert!(rendered.contains(&format!(
        "Custom-definition label names verified: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains(&format!(
            "Custom-definition ayanamsa labels verified: {} labels, all remain custom-definition territory",
            profile.custom_definition_ayanamsa_labels().len()
        )));
    assert!(rendered.contains(&format!(
        "Custom-definition ayanamsa label names verified: {}",
        profile.custom_definition_ayanamsa_labels().join(", ")
    )));
    assert!(rendered.contains(&format!(
        "Release notes documented: {} entries",
        profile.release_notes.len()
    )));
    assert!(rendered.contains(&format!(
        "Validation reference points documented: {} entries",
        profile.validation_reference_points.len()
    )));
    assert!(rendered.contains(&format!(
        "Custom-definition labels verified: {} labels, all remain custom-definition territory",
        profile.custom_definition_labels.len()
    )));
    assert!(rendered.contains(&format!(
        "Custom-definition label names verified: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains(&format!(
        "Compatibility caveats documented: {}",
        profile.known_gaps.len()
    )));
}

#[test]
fn compatibility_profile_verification_summary_renders_consistently() {
    let summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();

    assert_eq!(
        summary.profile_id,
        release_profiles.compatibility_profile_id
    );
    assert_eq!(
        summary.house_system_descriptor_count,
        profile.house_systems.len()
    );
    assert_eq!(
        summary.house_code_alias_count,
        profile.house_code_alias_count()
    );
    assert_eq!(
        summary.house_code_aliases_summary,
        profile.house_code_aliases_summary_line()
    );
    assert_eq!(
        summary.house_system_alias_count,
        profile
            .house_systems
            .iter()
            .map(|entry| 1 + entry.aliases.len())
            .sum::<usize>()
            - profile.house_systems.len()
    );
    assert_eq!(summary.ayanamsa_descriptor_count, profile.ayanamsas.len());
    assert_eq!(
        summary.baseline_house_system_count,
        profile.baseline_house_systems.len()
    );
    assert_eq!(
        summary.release_house_system_count,
        profile.release_house_systems.len()
    );
    assert_eq!(
        summary.baseline_ayanamsa_count,
        profile.baseline_ayanamsas.len()
    );
    assert_eq!(
        summary.release_ayanamsa_count,
        profile.release_ayanamsas.len()
    );
    assert_eq!(summary.release_note_count, profile.release_notes.len());
    assert_eq!(
        summary.validation_reference_point_count,
        profile.validation_reference_points.len()
    );
    assert_eq!(
        summary.custom_definition_label_count,
        profile.custom_definition_labels.len()
    );
    assert_eq!(
        summary.custom_definition_label_names,
        profile.custom_definition_labels.join(", ")
    );
    assert_eq!(
        summary.custom_definition_ayanamsa_label_count,
        profile.custom_definition_ayanamsa_labels().len()
    );
    assert_eq!(
        summary.custom_definition_ayanamsa_label_names,
        profile.custom_definition_ayanamsa_labels().join(", ")
    );
    assert_eq!(
        summary.ayanamsa_alias_count,
        profile
            .ayanamsas
            .iter()
            .map(|entry| 1 + entry.aliases.len())
            .sum::<usize>()
            - profile.ayanamsas.len()
    );
    assert_eq!(
        summary.house_formula_family_names,
        profile.house_formula_family_names().join(", ")
    );
    assert_eq!(
        summary.ayanamsa_metadata_count,
        profile
            .ayanamsas
            .iter()
            .filter(|entry| entry.has_sidereal_metadata())
            .count()
    );
    assert_eq!(
        summary.ayanamsa_metadata_gap_count,
        profile.ayanamsas.len() - summary.ayanamsa_metadata_count
    );
    assert_eq!(summary.compatibility_caveat_count, profile.known_gaps.len());
    summary
        .validate()
        .expect("fresh compatibility profile verification summary should validate");
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.summary_line()
    );
    assert_eq!(
        verify_compatibility_profile().unwrap(),
        summary.summary_line()
    );
    assert!(summary
        .summary_line()
        .contains("Compatibility profile verification"));
    assert!(summary.summary_line().contains(&format!(
        "House code aliases verified: {} short-form labels",
        profile.house_code_alias_count()
    )));
    assert!(summary.summary_line().contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(summary.summary_line().contains(&format!(
            "Alias uniqueness checks: house={} aliases, ayanamsa={} aliases; exact and case-insensitive labels verified",
            summary.house_system_alias_count,
            summary.ayanamsa_alias_count
        )));
    assert!(summary
            .summary_line()
            .contains("House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"));
    assert!(summary.summary_line().contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
    assert!(summary.summary_line().contains(&format!(
            "Ayanamsa reference metadata verified: {} descriptors with epoch/offset metadata, {} metadata gaps",
            profile
                .ayanamsas
                .iter()
                .filter(|entry| entry.has_sidereal_metadata())
                .count(),
            profile.ayanamsas.len() - profile
                .ayanamsas
                .iter()
                .filter(|entry| entry.has_sidereal_metadata())
                .count()
        )));
    assert!(summary.summary_line().contains(&format!(
            "Catalog posture: house systems=25 descriptors (8 constrained, 17 unconstrained); ayanamsas=59 descriptors (53 metadata-bearing, 6 descriptor-only); ayanamsa alias-bearing entries={}; ayanamsa metadata gaps=6; custom-definition labels=9; custom-definition ayanamsa labels=6; known gaps={}",
            profile
                .ayanamsas
                .iter()
                .filter(|entry| !entry.aliases.is_empty())
                .count(),
            profile.known_gaps_summary_line()
        )));
    assert!(
            summary
                .summary_line()
                .contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented")
        );
}

#[test]
fn compatibility_profile_verification_summary_validation_rejects_stale_fields() {
    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_ayanamsa_canonical_names = "stale summary".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("stale compatibility profile verification summary should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("release ayanamsa canonical names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_ayanamsa_canonical_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale compatibility profile verification summary should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("release ayanamsa canonical names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_house_canonical_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale release house-system canonical names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("release house-system canonical names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_system_alias_count = 0;

    let error = summary
        .validate()
        .expect_err("stale house-system alias count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-system alias count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_code_alias_count = 0;

    let error = summary
        .validate()
        .expect_err("stale house-code alias count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-code alias count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_code_aliases_summary = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale house-code alias summary should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-code aliases mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.custom_definition_label_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale custom-definition label names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition label names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.custom_definition_ayanamsa_label_count = 0;

    let error = summary
        .validate()
        .expect_err("stale custom-definition ayanamsa label count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition ayanamsa label count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.custom_definition_ayanamsa_label_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale custom-definition ayanamsa label names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition ayanamsa label names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_formula_family_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale house formula families should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house formula families mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_posture = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale release posture should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("release posture mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.ayanamsa_alias_count = 0;

    let error = summary
        .validate()
        .expect_err("stale ayanamsa alias count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("ayanamsa alias count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.ayanamsa_metadata_count = 0;

    let error = summary
        .validate()
        .expect_err("stale ayanamsa metadata counts should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("ayanamsa metadata count mismatch"));
}

#[test]
fn compatibility_profile_verification_summary_validation_rejects_stale_latitude_sensitive_house_systems(
) {
    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.latitude_sensitive_house_systems.reverse();

    let error = summary
        .validate()
        .expect_err("stale latitude-sensitive house systems should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("latitude-sensitive house systems mismatch"));
}

#[test]
fn descriptor_names_summary_validation_rejects_blank_entries() {
    let summary = DescriptorNamesSummary {
        names: vec!["Equal (MC)", "   "],
    };

    let error = summary
        .validate()
        .expect_err("blank descriptor names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("blank name"));
}

#[test]
fn descriptor_names_summary_validation_rejects_case_insensitive_duplicates() {
    let summary = DescriptorNamesSummary {
        names: vec!["Equal (MC)", "equal (mc)"],
    };

    let error = summary
        .validate()
        .expect_err("case-insensitive duplicate descriptor names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("case-insensitive duplicate name"));
}

#[test]
fn validate_name_sequence_rejects_whitespace_padded_owned_names() {
    let names = [String::from("Equal (MC)"), String::from(" release family ")];

    let error = validate_name_sequence(
        "compatibility profile house formula families",
        names.iter().map(String::as_str),
    )
    .expect_err("whitespace-padded owned names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("surrounding whitespace"));
}

#[test]
fn compatibility_profile_partition_checks_cover_the_current_catalog() {
    let profile = current_compatibility_profile();

    verify_profile_catalog_partitions_are_disjoint(
        "house-system",
        profile.baseline_house_systems,
        profile.release_house_systems,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect("the current house catalog partitions should remain disjoint");

    verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        profile.baseline_ayanamsas,
        profile.release_ayanamsas,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect("the current ayanamsa catalog partitions should remain disjoint");
}

#[test]
fn compatibility_profile_partition_checks_reject_overlapping_labels() {
    let house_baseline = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system"],
        "Quadrant system used for partition-overlap coverage.",
        true,
    )];
    let house_release = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["Placidus"],
        "Quadrant system used for partition-overlap coverage.",
        true,
    )];

    let error = verify_profile_catalog_partitions_are_disjoint(
        "house-system",
        &house_baseline,
        &house_release,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect_err("overlapping house-system labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("baseline and release slices overlap on label 'Placidus'"));

    let ayanamsa_baseline = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &["Lahiri ayanamsa"],
        "Sidereal mode used for partition-overlap coverage.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
    )];
    let ayanamsa_release = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Raman,
        "Raman",
        &["Lahiri"],
        "Sidereal mode used for partition-overlap coverage.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(pleiades_core::Angle::from_degrees(21.014_44)),
    )];

    let error = verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        &ayanamsa_baseline,
        &ayanamsa_release,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect_err("overlapping ayanamsa labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("baseline and release slices overlap on label 'Lahiri'"));
}

#[test]
fn compatibility_profile_partition_checks_reject_case_normalized_alias_overlaps() {
    #[derive(Clone, Copy)]
    struct Entry {
        canonical_name: &'static str,
        aliases: &'static [&'static str],
    }

    let baseline = [Entry {
        canonical_name: "Lahiri",
        aliases: &["Lahiri ayanamsa"],
    }];
    let release = [Entry {
        canonical_name: "Raman",
        aliases: &["lahiri"],
    }];

    let error = verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        &baseline,
        &release,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect_err("case-normalized overlapping alias labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("baseline and release slices overlap on label 'lahiri'"));
}

#[test]
fn descriptor_names_summary_formats_empty_single_and_multiple_entries() {
    #[derive(Clone, Copy)]
    struct Item(&'static str);

    let empty = summarize_descriptor_names(&[] as &[Item], |item| item.0);
    assert_eq!(empty.summary_line(), "0 (none)");
    assert_eq!(empty.to_string(), "0 (none)");

    let single = summarize_descriptor_names(&[Item("Alpha")], |item| item.0);
    assert_eq!(single.summary_line(), "1 (Alpha)");
    assert_eq!(single.to_string(), "1 (Alpha)");

    let multiple = summarize_descriptor_names(&[Item("Alpha"), Item("Beta")], |item| item.0);
    assert_eq!(multiple.summary_line(), "2 (Alpha, Beta)");
    assert_eq!(multiple.to_string(), "2 (Alpha, Beta)");
}

#[test]
fn compatibility_profile_verification_rejects_duplicate_house_labels() {
    let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus"],
        "Quadrant system used for duplicate-label verification coverage.",
        true,
    )];

    let error = verify_house_system_aliases(&descriptors)
        .expect_err("duplicate labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-system labels are not unique"));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_house_labels() {
    let descriptors = [
        pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Placidus",
            &[],
            "Quadrant system used for case-insensitive duplicate-label coverage.",
            true,
        ),
        pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Koch,
            "placidus",
            &[],
            "Quadrant system used for case-insensitive duplicate-label coverage.",
            true,
        ),
    ];

    let error = verify_house_system_aliases(&descriptors)
        .expect_err("case-insensitive duplicate labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("labels are not unique ignoring case"));
}

#[test]
fn compatibility_profile_verification_allows_case_insensitive_duplicate_house_aliases_within_entry()
{
    let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["placidus"],
        "Quadrant system used for intra-entry duplicate-label coverage.",
        true,
    )];

    let checked = verify_house_system_aliases(&descriptors)
        .expect("case-insensitive duplicate aliases within one descriptor should remain allowed");
    assert_eq!(checked, 2);
}

#[test]
fn compatibility_profile_verification_uses_display_labels_for_alias_mismatches() {
    let house_descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::EqualAries,
        "Equal (MC)",
        &[],
        "Quadrant system used for display-label mismatch coverage.",
        false,
    )];

    let error = verify_house_system_aliases(&house_descriptors)
        .expect_err("mismatched house labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("canonical label 'Equal (MC)' should resolve to Equal (1=Aries)"));
    assert!(!error.message.contains("EqualMidheaven"));
    assert!(!error.message.contains("EqualAries"));

    let ayanamsa_descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::TrueCitra,
        "True Chitra",
        &[],
        "Sidereal mode used for display-label mismatch coverage.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(pleiades_core::Angle::from_degrees(23.0)),
    )];

    let error = verify_ayanamsa_aliases(&ayanamsa_descriptors)
        .expect_err("mismatched ayanamsa labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("canonical label 'True Chitra' should resolve to True Citra"));
    assert!(!error.message.contains("TrueChitra"));
    assert!(!error.message.contains("TrueCitra"));
}

#[test]
fn compatibility_profile_verification_rejects_missing_descriptor_notes() {
    let descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        " ",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
    )];

    let error = verify_ayanamsa_aliases(&descriptors)
        .expect_err("missing descriptor notes should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("missing notes metadata"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_canonical_names() {
    let error = ensure_profile_descriptor_metadata(
        "house-system",
        " Placidus ",
        "Quadrant system used for whitespace-padded metadata coverage.",
    )
    .expect_err("whitespace-padded canonical names should fail profile verification");

    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its canonical name"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_notes() {
    let error = ensure_profile_descriptor_metadata(
        "ayanamsa",
        "Lahiri",
        " whitespace-padded notes metadata ",
    )
    .expect_err("whitespace-padded notes should fail profile verification");

    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its notes metadata"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_house_aliases() {
    let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &[" Placidus alias "],
        "Quadrant system used for whitespace-padded alias coverage.",
        true,
    )];

    let error = verify_house_system_aliases(&descriptors)
        .expect_err("whitespace-padded house aliases should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its label"));
    assert!(error.message.contains(" Placidus alias "));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_ayanamsa_aliases() {
    let descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[" Lahiri alias "],
        "Ayanamsa used for whitespace-padded alias coverage.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
    )];

    let error = verify_ayanamsa_aliases(&descriptors)
        .expect_err("whitespace-padded ayanamsa aliases should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its label"));
    assert!(error.message.contains(" Lahiri alias "));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_labels() {
    let mut seen_labels = BTreeSet::new();
    let mut seen_labels_case_insensitive = BTreeMap::new();
    let error = ensure_unique_profile_label(
        "custom-definition",
        "  custom delta  ",
        "custom delta",
        &mut seen_labels,
        &mut seen_labels_case_insensitive,
    )
    .expect_err("whitespace-padded labels should fail profile verification");

    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its label"));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_ayanamsa_labels() {
    let descriptors = [
        pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &[],
            "Sidereal mode used for case-insensitive duplicate-label coverage.",
            Some(JulianDay::from_days(2_435_553.5)),
            Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
        ),
        pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::TrueRevati,
            "lahiri",
            &[],
            "Sidereal mode used for case-insensitive duplicate-label coverage.",
            Some(JulianDay::from_days(2_444_907.5)),
            Some(pleiades_core::Angle::from_degrees(0.0)),
        ),
    ];

    let error = verify_ayanamsa_aliases(&descriptors)
        .expect_err("case-insensitive duplicate labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("labels are not unique ignoring case"));
}

#[test]
fn compatibility_profile_verification_rejects_custom_definition_labels_that_resolve_to_builtins() {
    let labels = ["Placidus"];

    let error = verify_custom_definition_labels(&labels)
        .expect_err("custom-definition labels should stay outside built-ins");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("should remain unresolved as a built-in house system or ayanamsa"));
}

#[test]
fn compatibility_profile_verification_rejects_custom_definition_labels_that_resolve_to_ayanamsas() {
    let labels = ["Lahiri"];

    let error = verify_custom_definition_labels(&labels)
        .expect_err("custom-definition labels should stay outside built-in ayanamsas");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("should remain unresolved as a built-in house system or ayanamsa"));
}

#[test]
fn compatibility_profile_verification_allows_intentional_ayanamsa_homographs() {
    let labels = INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS;

    let checked = verify_custom_definition_labels(labels)
        .expect("intentional custom-definition homographs should remain allowed");
    assert_eq!(checked, labels.len());
    assert!(is_intentional_custom_definition_ayanamsa_homograph(
        labels[0]
    ));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_custom_definition_labels()
{
    let labels = ["custom delta", "Custom Delta"];

    let error = verify_custom_definition_labels(&labels)
        .expect_err("case-insensitive duplicate custom-definition labels should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition entries are not unique"));
}

#[test]
fn compatibility_profile_verification_rejects_blank_release_note_entries() {
    let entries = ["release note", "   "];

    let error = verify_profile_text_section("release-note", &entries)
        .expect_err("blank release-note entries should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("entry is blank"));
}

#[test]
fn compatibility_profile_verification_rejects_duplicate_compatibility_caveats() {
    let entries = ["known gap", "known gap"];

    let error = verify_profile_text_section("compatibility-caveat", &entries)
        .expect_err("duplicate caveat entries should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("entries are not unique"));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_release_notes() {
    let entries = ["shared release text", "Shared Release Text"];

    let error = verify_profile_text_section("release-note", &entries)
        .expect_err("case-insensitive duplicate release notes should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("entries are not unique ignoring case"));
    assert!(error.message.contains("Shared Release Text"));
}

#[test]
fn compatibility_profile_verification_rejects_duplicate_text_across_sections() {
    let error = verify_profile_text_sections_are_disjoint(&[
        ("release-note", &["shared release text"]),
        ("compatibility-caveat", &["shared release text"]),
    ])
    .expect_err("duplicate prose across sections should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains(
            "duplicate entry 'shared release text' appears in both release-note and compatibility-caveat"
        ));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_text_across_sections() {
    let error = verify_profile_text_sections_are_disjoint(&[
        ("release-note", &["shared release text"]),
        ("compatibility-caveat", &["Shared Release Text"]),
    ])
    .expect_err("case-insensitive duplicate prose across sections should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("not unique ignoring case"));
    assert!(error.message.contains("release-note"));
    assert!(error.message.contains("compatibility-caveat"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_text_across_sections() {
    let error = verify_profile_text_sections_are_disjoint(&[
        ("release-note", &["shared release text "]),
        ("compatibility-caveat", &["shared release text"]),
    ])
    .expect_err("whitespace-padded prose across sections should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("contains surrounding whitespace"));
    assert!(error.message.contains("release-note"));
}

#[test]
fn compatibility_profile_verification_validates_target_scope_sections() {
    let profile = current_compatibility_profile();

    verify_profile_text_section("target-house-scope", profile.target_house_scope)
        .expect("target house scope should validate");
    verify_profile_text_section("target-ayanamsa-scope", profile.target_ayanamsa_scope)
        .expect("target ayanamsa scope should validate");
    verify_profile_text_sections_are_disjoint(&[
        ("target-house-scope", profile.target_house_scope),
        ("target-ayanamsa-scope", profile.target_ayanamsa_scope),
        ("release-note", profile.release_notes),
        (
            "validation-reference-point",
            profile.validation_reference_points,
        ),
        ("compatibility-caveat", profile.known_gaps),
    ])
    .expect("target scope prose should remain disjoint from release prose");
    assert_eq!(
        profile
            .validated_target_house_scope_summary_line()
            .expect("target house scope summary should validate"),
        profile.target_house_scope.join("; ")
    );
    assert_eq!(
        profile
            .validated_target_ayanamsa_scope_summary_line()
            .expect("target ayanamsa scope summary should validate"),
        profile.target_ayanamsa_scope.join("; ")
    );
}

#[test]
fn release_notes_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["release-notes-summary"]).expect("release notes summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Release notes summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release-specific coverage:"));
    assert!(rendered.contains("Selected asteroid source evidence: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"));
    assert!(rendered.contains("Selected asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451915_major_body_bridge_summary_for_report()));
    assert!(
        rendered.contains(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report())
    );
    assert!(rendered.contains(&reference_snapshot_2451914_bridge_day_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(rendered
        .contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(rendered.contains(&format!(
        "{}\n{}",
        reference_snapshot_1600_selected_body_boundary_summary_for_report(),
        reference_snapshot_1750_selected_body_boundary_summary_for_report()
    )));
    assert!(rendered.contains("Custom-definition labels:"));
    assert!(rendered.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
    assert!(rendered.contains("Compatibility caveats:"));
    assert!(rendered.contains(&format!(
        "Custom-definition labels: {}",
        profile.custom_definition_labels.len()
    )));
    assert!(rendered.contains(&format!(
        "Custom-definition label names: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains(
        profile
            .validated_target_house_scope_summary_line()
            .expect("target house scope summary should validate")
            .as_str()
    ));
    assert!(rendered.contains(
        profile
            .validated_target_ayanamsa_scope_summary_line()
            .expect("target ayanamsa scope summary should validate")
            .as_str()
    ));
    assert!(rendered.contains(&format!(
        "Compatibility caveats: {}",
        profile.known_gaps.len()
    )));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_house_scope.join("; ")));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
    assert!(rendered.contains("API stability summary line: API stability posture: pleiades-api-stability/0.1.0; stable surfaces: 6; experimental surfaces: 3; deprecation policy items: 4; intentional limits: 3"));
    assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
    assert!(rendered.contains("Reference snapshot body-class coverage: major bodies: 262 rows across 10 bodies and 31 epochs; major windows: "));
    assert!(rendered
        .contains("selected asteroids: 95 rows across 6 bodies and 17 epochs; asteroid windows: "));
    assert!(rendered.contains(&pleiades_jpl::comparison_snapshot_source_summary_for_report()));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains(&format!(
        "Packaged-artifact access: {}",
        format_packaged_artifact_access_summary()
    )));
    assert!(rendered.contains("Packaged request policy:"));
    assert!(rendered.contains(&format!(
        "Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    )));
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Packaged batch parity: {}",
            packaged_mixed_tt_tdb_batch_parity_summary_for_report()
        )
    }));
    assert!(rendered.contains("Packaged batch parity:"));
    assert!(rendered.contains("Release notes: release-notes"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert_report_contains_exact_line(
        &rendered,
        "Workspace audit summary: workspace-audit-summary",
    );
    assert!(rendered.contains(profile.target_house_scope.join("; ").as_str()));
    assert!(rendered.contains(profile.target_ayanamsa_scope.join("; ").as_str()));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Reference snapshot coverage: 357 rows across 16 bodies and 31 epochs (95 asteroid rows; JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
    assert!(rendered.contains(&reference_snapshot_2500_major_body_boundary_summary_for_report()));
    assert!(
        rendered.contains("Comparison snapshot coverage: 232 rows across 10 bodies and 28 epochs")
    );
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Artifact boundary envelope:"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("See release-notes for the full maintainer-facing artifact."));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn release_checklist_summary_helper_reports_expected_posture() {
    let summary = release_checklist_summary();

    assert_eq!(
        summary.release_profile_identifiers,
        current_release_profile_identifiers()
    );
    assert_eq!(
        summary.repository_managed_release_gates,
        release_checklist_repository_managed_release_gates().len()
    );
    assert_eq!(
        summary.manual_bundle_workflow_items,
        release_checklist_manual_bundle_workflow().len()
    );
    assert_eq!(
        summary.bundle_contents_items,
        release_checklist_bundle_contents().len()
    );
    assert_eq!(
        summary.external_publishing_reminders,
        release_checklist_external_publishing_reminders().len()
    );
    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.summary_line().contains("v1 compatibility="));
}

#[test]
fn release_checklist_command_renders_the_release_checklist() {
    let rendered = render_cli(&["release-checklist"]).expect("release checklist should render");
    assert!(rendered.contains("Release checklist"));
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("API stability summary: api-stability-summary"));
    assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
    assert!(rendered.contains("Repository-managed release gates:"));
    assert!(rendered.contains("[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release"));
    assert!(rendered.contains("[x] cargo run -q -p pleiades-validate -- benchmark --rounds 5"));
    assert!(rendered.contains("[x] cargo run -q -p pleiades-validate -- report --rounds 5"));
    assert!(rendered.contains("Manual bundle workflow:"));
    assert!(rendered.contains("Bundle contents:"));
    assert!(rendered.contains("backend-matrix-summary.txt"));
    assert!(rendered.contains("api-stability-summary.txt"));
    assert!(rendered.contains("release-checklist-summary.txt"));
}

#[test]
fn release_checklist_summary_command_renders_the_summary() {
    let rendered = render_cli(&["release-checklist-summary"])
        .expect("release checklist summary should render");
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains("Release checklist summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("API stability summary: api-stability-summary"));
    assert!(rendered.contains("Zodiac policy:"));
    assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
    assert!(rendered.contains("Repository-managed release gates: 10 items"));
    assert!(rendered.contains("Manual bundle workflow: 4 items"));
    assert!(rendered.contains("Bundle contents: 25 items"));
    assert!(rendered.contains("External publishing reminders: 3 items"));
    assert!(rendered.contains("See release-checklist for the full maintainer-facing artifact."));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn release_gate_command_aliases_the_release_checklist() {
    let checklist = render_cli(&["release-checklist"]).expect("release checklist should render");
    let gate = render_cli(&["release-gate"]).expect("release gate should render");
    let checklist_summary = render_cli(&["release-checklist-summary"])
        .expect("release checklist summary should render");
    let gate_summary =
        render_cli(&["release-gate-summary"]).expect("release gate summary should render");

    assert_eq!(gate, checklist);
    assert_eq!(gate_summary, checklist_summary);
    assert!(render_cli(&["release-gate", "extra"]).is_err());
    assert!(render_cli(&["release-gate-summary", "extra"]).is_err());
}

#[test]
fn release_smoke_command_renders_the_smoke_report() {
    let rendered = render_cli(&["release-smoke"]).expect("release smoke should render");

    assert!(rendered.contains("Release smoke"));
    assert!(rendered.contains("workspace audit: ok"));
    assert!(rendered.contains("compatibility profile verification: ok"));
    assert!(rendered.contains("artifact validation: ok"));
    assert!(rendered.contains("release bundle generation: ok"));
    assert!(rendered.contains("release bundle verification: ok"));
    assert!(render_cli(&["release-smoke", "extra"]).is_err());
}

#[test]
fn release_gate_checks_reject_non_directory_output_paths() {
    let output_path = unique_temp_dir("pleiades-release-gate-file").with_extension("txt");
    std::fs::write(&output_path, "not a directory").expect("temporary file should be creatable");

    let error = validate_release_gate_at(&output_path)
        .expect_err("release gate checks should reject file-backed output paths");

    assert!(error.contains("release gate"));
}

#[test]
fn release_summary_command_renders_the_quick_overview() {
    let rendered = render_cli(&["release-summary"]).expect("release summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Release summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert_report_contains_exact_line(
        &rendered,
        &format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        ),
    );
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_house_scope.join("; ")));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
    assert!(rendered.lines().any(|line| line == "Release summary"));
    assert!(rendered.contains("Comparison body-class tolerance: body-class tolerance posture:"));
    assert!(rendered.contains("Comparison body-class error envelopes:"));
    assert!(rendered.contains("Source corpus: comparison corpus release-grade guard:"));
    assert!(rendered.contains("Source corpus posture: comparison corpus release-grade guard:"));
    assert!(rendered.contains("Catalog posture: house systems="));
    assert_report_contains_exact_line(
        &rendered,
        &format!("Known gaps: {}", profile.known_gaps_summary_line()),
    );
    assert!(rendered.contains("Pluto fallback: "));
    assert!(rendered.contains("JPL source corpus contract:"));
    assert!(rendered.contains("phase-2 corpus alignment:"));
    assert!(rendered.contains("Release summary line:"));
    assert!(rendered.contains("Production generation body-class coverage:"));
    assert!(rendered.contains("Production generation corpus shape:"));
    assert!(rendered.contains(&format!(
        "Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    )));
    assert!(rendered
        .lines()
        .any(|line| line == "Backend matrix summary: backend-matrix-summary"));
    assert_report_contains_exact_line(&rendered, &profile.catalog_inventory_summary_line());
    assert!(rendered.contains(&format!(
        "house latitude-sensitive constraints={}",
        profile.latitude_sensitive_house_constraints_summary_line()
    )));
    assert_report_contains_exact_line(
        &rendered,
        &format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        ),
    );
    assert!(rendered.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains(&reference_snapshot_2451914_bridge_day_summary_for_report()));
    assert!(rendered.contains(&format!(
        "{}\n{}",
        reference_snapshot_1600_selected_body_boundary_summary_for_report(),
        reference_snapshot_1750_selected_body_boundary_summary_for_report()
    )));
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Packaged batch parity: {}",
            packaged_mixed_tt_tdb_batch_parity_summary_for_report()
        )
    }));
    assert!(rendered
        .lines()
        .any(|line| line == "Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert_report_contains_exact_line(
        &rendered,
        "Workspace audit summary: workspace-audit-summary",
    );
    assert_report_contains_exact_line(
        &rendered,
        "Release checklist summary: release-checklist-summary",
    );
    assert!(rendered.contains("Workspace audit: workspace-audit / audit"));
    assert!(rendered.contains("Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"));
    assert!(rendered.contains("Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"));
    assert!(rendered.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"));
    assert!(rendered.contains("Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"));
    assert!(rendered.contains("Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert!(rendered.contains("Frame policy: ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert_eq!(
        render_cli(&["time-scale-policy"]).expect("time-scale policy alias should render"),
        render_time_scale_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["delta-t-policy"]).expect("delta T policy alias should render"),
        render_delta_t_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["observer-policy"]).expect("observer policy alias should render"),
        render_observer_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["apparentness-policy"]).expect("apparentness policy alias should render"),
        render_apparentness_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["frame-policy"]).expect("frame policy alias should render"),
        render_frame_policy_summary_text()
    );

    for (args, expected) in [
        (
            ["time-scale-policy-summary", "extra"],
            "time-scale-policy-summary does not accept extra arguments",
        ),
        (
            ["time-scale-policy", "extra"],
            "time-scale-policy does not accept extra arguments",
        ),
        (
            ["utc-convenience-policy-summary", "extra"],
            "utc-convenience-policy-summary does not accept extra arguments",
        ),
        (
            ["utc-convenience-policy", "extra"],
            "utc-convenience-policy does not accept extra arguments",
        ),
        (
            ["delta-t-policy-summary", "extra"],
            "delta-t-policy-summary does not accept extra arguments",
        ),
        (
            ["delta-t-policy", "extra"],
            "delta-t-policy does not accept extra arguments",
        ),
        (
            ["observer-policy-summary", "extra"],
            "observer-policy-summary does not accept extra arguments",
        ),
        (
            ["observer-policy", "extra"],
            "observer-policy does not accept extra arguments",
        ),
        (
            ["apparentness-policy-summary", "extra"],
            "apparentness-policy-summary does not accept extra arguments",
        ),
        (
            ["apparentness-policy", "extra"],
            "apparentness-policy does not accept extra arguments",
        ),
        (
            ["frame-policy-summary", "extra"],
            "frame-policy-summary does not accept extra arguments",
        ),
        (
            ["frame-policy", "extra"],
            "frame-policy does not accept extra arguments",
        ),
    ] {
        assert_eq!(
            render_cli(&args).expect_err("policy summary should reject extra arguments"),
            expected
        );
    }

    assert!(rendered.contains(&request_surface_summary_for_report()));
    let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Mean-obliquity frame round-trip: {}",
            mean_obliquity_frame_round_trip
        )
    }));
    assert!(rendered.contains("Zodiac policy: tropical only"));
    assert!(rendered.contains("ayanamsa catalog validation: ok"));
    assert!(rendered.contains("House systems:"));
    assert!(rendered.contains("House systems: 25 total (12 baseline, 13 release-specific)"));
    assert!(rendered.contains(&format!(
        "House-code aliases: {}",
        profile.house_code_alias_count()
    )));
    assert!(rendered.contains("Release-specific house-system canonical names: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
    assert!(rendered.contains("Wang"));
    assert!(rendered.contains("Aries houses"));
    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
    assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
    assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
    assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
    assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
    assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("True Revati: epoch=JD 1926902.658267; offset=0°"));
    assert!(rendered.contains("True Mula: epoch=JD 1805889.671313; offset=0°"));
    assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
    assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
    assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
    assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
    assert!(rendered.contains("Babylonian (Britton): epoch=JD 1805415.712776; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 2): epoch=JD 1797039.20682; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 3): epoch=JD 1774637.420172; offset=0°"));
    assert!(rendered.contains("Babylonian (Eta Piscium): epoch=JD 1807871.964797; offset=0°"));
    assert!(rendered.contains("Babylonian (Aldebaran): epoch=JD 1801643.133503; offset=0°"));
    assert!(rendered.contains("Aryabhata (499 CE): epoch=JD 1903396.7895320603; offset=0°"));
    assert!(rendered.contains("Sassanian: epoch=JD 1927135.8747793; offset=0°"));
    assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
    assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
    assert!(rendered.contains("Sheoran: epoch=JD 1789947.090881; offset=0°"));
    assert!(rendered.contains("True Sheoran: epoch="));
    assert!(rendered.contains("Hipparchus: epoch=JD 1674484; offset=-9.333333333333334°"));
    assert!(rendered.contains("Djwhal Khul: epoch=JD 1706703.948006; offset=0°"));
    assert!(rendered.contains("Galactic Center: epoch="));
    assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
    assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
    assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm): epoch="));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
    assert!(rendered.contains("Galactic Equator (IAU 1958): epoch=JD 1667118.376332; offset=0°"));
    assert!(rendered.contains("Galactic Equator (True): epoch=JD 1665728.603158; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Mula): epoch=JD 1840527.426262; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun): epoch=JD 1909045.584433; offset=0°"));
    assert!(rendered.contains("Aryabhata (Mean Sun): epoch=JD 1909650.815331; offset=0°"));
    assert!(rendered.contains("Release-specific ayanamsa canonical names: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Galactic Equator (IAU 1958): epoch=JD 1667118.376332; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(rendered.contains("Ayanamsa provenance: representative provenance examples:"));
    assert!(rendered.contains("True Citra — True Citra sidereal mode with the published zero point used by Swiss Ephemeris-style interoperability tables."));
    assert!(rendered.contains("True Revati — True-nakshatra mode with the Revati reference point fixed to the Swiss Ephemeris zero date."));
    assert!(rendered.contains("True Mula — True-nakshatra mode with the Mula reference point fixed to the Swiss Ephemeris zero date."));
    assert!(rendered.contains("True Pushya — True-nakshatra Pushya reference mode exposed by Swiss Ephemeris and anchored to the published zero date."));
    assert!(rendered.contains("Udayagiri — Udayagiri sidereal mode treated as the Lahiri/Chitrapaksha/Chitra Paksha 285 CE reference family in the Swiss Ephemeris interoperability catalog."));
    assert!(rendered.contains("True Sheoran — True-nakshatra Sheoran reference mode with the Swiss Ephemeris zero point at JD 1789947.090881 (+0188/08/09 14:10:52.11 UT)."));
    assert!(rendered.contains("Galactic Center (Rgilbrand) — Galactic-center reference mode attributed to Rgilbrand, with the Swiss Ephemeris zero point at JD 1861740.329525 (+0385/03/03 19:54:30.99 UT)."));
    assert!(rendered.contains("Babylonian (Kugler 1) — Babylonian sidereal mode associated with Kugler's first reconstruction, with the Swiss Ephemeris zero point at JD 1833923.577692 (+0309/01/05 01:51:52.62 UT)."));
    assert!(rendered.contains("Valens Moon — Valens Moon sidereal mode, catalogued with the Swiss Ephemeris reference epoch and offset from the header metadata."));
    assert!(rendered.contains("JN Bhasin"));
    assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
    assert!(rendered.contains("Custom-definition labels: 9"));
    assert!(rendered.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));
    assert!(rendered.contains("Custom-definition ayanamsas:"));
    assert!(rendered.contains("Compatibility caveats: 2"));
    assert!(rendered.contains(&format!(
        "Ayanamsas: {} total ({} baseline, {} release-specific)",
        profile.ayanamsas.len(),
        profile.baseline_ayanamsas.len(),
        profile.release_ayanamsas.len()
    )));
    assert!(rendered.contains("Comparison envelope:"));
    assert!(rendered.contains("median longitude delta:"));
    assert!(rendered.contains("95th percentile longitude delta:"));
    assert!(rendered.contains("median latitude delta:"));
    assert!(rendered.contains("95th percentile latitude delta:"));
    assert!(
        rendered.contains("Comparison snapshot coverage: 232 rows across 10 bodies and 28 epochs")
    );
    assert!(rendered.contains("Body-class error envelopes:"));
    assert!(rendered.contains("max Δlon="));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("rms Δlat="));
    assert!(rendered.contains("max longitude delta:"));
    assert!(rendered.contains("median longitude delta:"));
    assert!(rendered.contains("rms longitude delta:"));
    assert!(rendered.contains("max latitude delta:"));
    assert!(rendered.contains("median latitude delta:"));
    assert!(rendered.contains("95th percentile latitude delta:"));
    assert!(rendered.contains("rms latitude delta:"));
    assert!(rendered.contains("Validation evidence:"));
    assert!(rendered.contains("House validation corpus: 9 scenarios (Mid-latitude reference chart, Western hemisphere reference chart, Equatorial reference chart, Polar stress chart, Northern high-latitude stress chart, Northern high-latitude mountain stress chart, Southern high-latitude mountain stress chart, Southern polar stress chart, Southern hemisphere reference chart), 108 samples, 108 successes, 0 failures; hemisphere coverage: north=5, south=3, equatorial=1; longitude coverage: prime-meridian=2, non-prime-meridian=7; formula families: Equal, Whole Sign, Quadrant, Equatorial projection; latitude-sensitive systems: Koch, Placidus, Topocentric; constraints: Koch [Quadrant system with documented high-latitude pathologies.], Placidus [Quadrant system; can fail or become unstable at extreme latitudes.], Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]"));
    assert!(rendered.contains("comparison samples"));
    assert!(rendered.contains("Time-scale policy:"));
    assert!(rendered.contains("Observer policy:"));
    assert!(rendered.contains("Apparentness policy:"));
    assert!(rendered.contains("Native sidereal policy:"));
    assert!(rendered.contains("Zodiac policy:"));
    assert!(rendered.contains("notable regressions"));
    assert!(rendered.contains("outside-tolerance bodies"));
    assert!(rendered.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
    assert!(rendered.contains("coverage=Luminaries: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence, bodies=2 (Sun, Moon), samples="));
    assert!(rendered.contains("window=JD 2268932.5 (TT) → JD 2634167.0 (TT)"));
    assert!(rendered.contains("frames=Ecliptic"));
    assert!(rendered.contains("Luminaries: Δlon≤7.500°, Δlat≤0.750°, Δdist=0.001 AU"));
    assert!(rendered.contains("Major planets: Δlon≤0.010°, Δlat≤0.010°, Δdist=0.001 AU"));
    assert!(rendered
        .contains("Pluto fallback (approximate): Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU"));
    assert!(rendered.contains("evidence=9 bodies"));
    assert!(rendered.contains("Body-class tolerance posture:"));
    assert!(rendered.contains("Expected tolerance status:"));
    assert!(rendered.contains("Comparison audit: status=clean, bodies checked=9"));
    assert!(rendered.contains("JPL interpolation evidence:"));
    assert!(rendered.contains("Reference/hold-out overlap:"));
    assert!(rendered.contains("JPL independent hold-out:"));
    assert!(rendered.contains("JPL independent hold-out equatorial parity:"));
    assert!(rendered.contains("JPL independent hold-out batch parity:"));
    assert_report_contains_exact_line(
            &rendered,
            "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false",
        );
    assert_report_contains_exact_line(
            &rendered,
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant",
        );
    assert!(rendered.contains("JPL frame treatment: checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"));
    assert!(rendered.contains("Reference snapshot coverage:"));
    assert!(rendered.contains("Selected asteroid evidence:"));
    assert!(rendered.contains("Selected asteroid batch parity:"));
    assert!(rendered.contains("VSOP87 evidence:"));
    assert!(rendered.contains("VSOP87 source-backed body-class envelopes:"));
    assert!(rendered.contains("VSOP87 canonical J2000 equatorial body-class envelopes:"));
    assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("median Δlat="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("median Δdist="));
    assert!(rendered.contains("p95 Δdist="));
    assert!(rendered.contains(
        "Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
    ));
    assert!(rendered.contains("VSOP87 source documentation:"));
    assert!(rendered.contains("VSOP87 frame treatment:"));
    assert!(rendered.contains("VSOP87 request policy:"));
    assert!(rendered.contains("VSOP87 source audit:"));
    assert!(rendered.contains("VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"));
    let source_documentation_alias =
        render_cli(&["source-documentation"]).expect("source documentation alias should render");
    assert_eq!(
        source_documentation_alias,
        format_vsop87_source_documentation_summary()
    );
    let source_documentation_health_alias = render_cli(&["source-documentation-health"])
        .expect("source documentation health alias should render");
    assert_eq!(
        source_documentation_health_alias,
        format_vsop87_source_documentation_health_summary()
    );
    assert_eq!(
        render_cli(&["source-documentation-health", "extra"])
            .expect_err("source documentation health alias should reject extra arguments"),
        "source-documentation-health does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-documentation", "extra"])
            .expect_err("source documentation alias should reject extra arguments"),
        "source-documentation does not accept extra arguments"
    );
    let mut source_documentation_summary = source_documentation_summary();
    source_documentation_summary.source_specification_count += 1;
    let source_documentation_error = source_documentation_summary
        .validate()
        .expect_err("source documentation summary should detect catalog drift");
    assert_eq!(
        format_validated_vsop87_source_documentation_summary_for_report(
            &source_documentation_summary
        ),
        format!("VSOP87 source documentation: unavailable ({source_documentation_error})")
    );
    let mut source_documentation_health_summary = source_documentation_health_summary();
    source_documentation_health_summary.source_file_count += 1;
    let source_documentation_health_error = source_documentation_health_summary
        .validate()
        .expect_err("source documentation health should detect catalog drift");
    assert_eq!(
        format_validated_vsop87_source_documentation_health_summary_for_report(
            &source_documentation_health_summary
        ),
        format!(
            "VSOP87 source documentation health: unavailable ({source_documentation_health_error})"
        )
    );
    let source_audit_alias =
        render_cli(&["source-audit"]).expect("source audit alias should render");
    assert_eq!(source_audit_alias, source_audit_summary_for_report());
    assert_eq!(
        render_cli(&["source-audit", "extra"])
            .expect_err("source audit alias should reject extra arguments"),
        "source-audit does not accept extra arguments"
    );
    let generated_binary_audit_alias = render_cli(&["generated-binary-audit"])
        .expect("generated binary audit alias should render");
    assert_eq!(
        generated_binary_audit_alias,
        generated_binary_audit_summary_for_report()
    );
    assert_eq!(
        render_cli(&["generated-binary-audit", "extra"])
            .expect_err("generated binary audit alias should reject extra arguments"),
        "generated-binary-audit does not accept extra arguments"
    );
    let reference_asteroid_source_summary_alias =
        render_cli(&["reference-asteroid-source-summary"])
            .expect("reference asteroid source summary alias should render");
    assert_eq!(
        reference_asteroid_source_summary_alias,
        reference_asteroid_source_window_summary_for_report()
    );
    assert!(rendered.contains("VSOP87 canonical J2000 source-backed evidence:"));
    assert!(rendered.contains("VSOP87 canonical J2000 equatorial companion evidence:"));
    assert!(rendered.contains("VSOP87 canonical J2000 batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
    assert!(rendered.contains("JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"));
    assert!(rendered.contains("VSOP87 canonical J1900 batch parity:"));
    assert!(rendered.contains("VSOP87 source-backed body evidence:"));
    assert!(rendered.contains("Lunar reference envelope:"));
    assert!(rendered.contains("Lunar equatorial reference envelope:"));
    assert!(rendered.contains("Lunar source windows:"));
    assert!(rendered.contains("JPL interpolation quality:"));
    assert!(rendered.contains(&reference_snapshot_1750_major_body_interior_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(rendered.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Artifact boundary envelope:"));
    assert!(rendered.contains(
        &artifact_inspection_summary_for_report()
            .expect("artifact inspection summary should build")
    ));
    assert!(rendered.contains("residual-bearing bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release gate reminders:"));
    assert!(rendered.contains("verify-compatibility-profile"));
    assert!(rendered.contains("See release-notes and release-checklist"));
}

#[test]
fn backend_matrix_command_renders_the_implemented_catalog() {
    let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
    assert!(rendered.contains("Implemented backend matrices"));
    assert!(rendered.contains("summary: id=jpl-snapshot; version="));
    assert!(rendered.contains("JPL snapshot reference backend"));
    assert!(!rendered.contains("JPL source corpus contract: JPL source corpus contract:"));
    assert!(rendered.contains(
            "selected asteroid coverage: 6 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        ));
    assert!(rendered.contains("Selected asteroid source evidence: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"));
    assert!(rendered.contains("Selected asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(rendered.contains("exact J2000 evidence: 6 bodies at JD 2451545.0"));
    assert!(rendered.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(rendered.contains("nominal range:"));
    assert!(rendered.contains("provenance sources:"));
    assert!(rendered.contains("implementation status: fixture-reference"));
    assert!(rendered.contains("implementation status: partial-source-backed"));
    assert!(rendered.contains("implementation status: preliminary-algorithm"));
    assert!(rendered.contains("implementation status: draft-artifact"));
    assert!(rendered.contains("implementation status: routing-facade"));
    assert!(rendered.contains("family posture: data-backed"));
    assert!(rendered.contains("family posture: algorithmic"));
    assert!(rendered.contains("family posture: routing"));
    assert!(rendered.contains("implementation note:"));
    assert!(rendered.contains("Sun through Neptune now use generated binary VSOP87B source tables derived from the vendored full-file inputs"));
    assert!(rendered.contains("expected error classes:"));
    assert!(rendered.contains("required external data files:"));
    assert!(rendered.contains("crates/pleiades-jpl/data/reference_snapshot.csv"));
    assert!(rendered.contains("source documentation:"));
    assert!(rendered.contains("source audit:"));
    assert!(rendered.contains("Sun: IMCCE/CELMECH VSOP87B VSOP87B.ear"));
    assert!(rendered.contains("Paul Schlyter-style mean orbital elements for planets"));
    assert!(rendered.contains("body source profiles:"));
    assert!(rendered.contains("VSOP87B.ear"));
    assert!(rendered.contains("geocentric planetary reduction against Earth coefficients"));
    assert!(rendered.contains("solar reduction from Earth coefficients"));
    assert!(rendered.contains("canonical J2000 VSOP87B evidence:"));
    assert!(rendered.contains("Sun: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Mercury: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Venus: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Mars: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Jupiter: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Saturn: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Uranus: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Neptune: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Pluto: kind=mean orbital elements fallback, accuracy=Approximate"));
    assert!(rendered.contains("Meeus-style truncated lunar orbit formulas"));
    assert!(rendered.contains("NASA/JPL Horizons API vector tables (DE441)"));
    assert!(rendered.contains("interpolation quality checks:"));
    assert!(rendered.contains("JPL interpolation posture: source="));
    assert!(rendered.contains("interpolation, bracket span"));
    assert!(rendered.contains("TDB"));
    assert!(rendered.contains("VSOP87 planetary backend"));
    assert!(rendered.contains("Pluto remains the current approximate mean-element fallback special case until a Pluto-specific source path is selected"));
    assert!(rendered.contains("ELP lunar backend (Moon and lunar nodes)"));
    assert!(rendered.contains("specification summary: ELP lunar theory specification:"));
    assert!(rendered.contains("compact lunar and lunar-point formulas provide the current deterministic baseline while documented production lunar-theory ingestion remains open"));
    assert!(rendered.contains("Lunar source windows"));
    assert!(rendered.contains("Lunar high-curvature continuity evidence"));
    assert!(rendered.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(rendered.contains("unsupported bodies: True Apogee, True Perigee"));
    assert!(rendered.contains("Body/date/channel claims:"));
    assert!(rendered.contains("Packaged data backend"));
    assert!(rendered.contains("Composite routed backend"));
}

#[test]
fn selected_asteroid_coverage_summary_validates_selected_body_lists() {
    let asteroids = reference_asteroids();
    let summary = selected_asteroid_coverage_summary(asteroids)
        .expect("reference asteroids should produce a coverage summary");
    assert_eq!(
            summary.validated_summary_line(),
            Ok("selected asteroid coverage: 6 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)".to_string())
        );

    let mut drifted_bodies = asteroids.to_vec();
    drifted_bodies[0] = CelestialBody::Moon;
    let error = SelectedAsteroidCoverageSummary {
        body_count: drifted_bodies.len(),
        bodies: drifted_bodies,
    }
    .validate()
    .expect_err("non-asteroid bodies should be rejected");
    assert_eq!(
        error,
        SelectedAsteroidCoverageSummaryValidationError::UnsupportedBody {
            index: 0,
            body: "Moon".to_string(),
        }
    );
}

#[test]
fn backend_matrix_report_rejects_invalid_backend_metadata() {
    let entry = BackendMatrixEntry {
        label: "broken backend",
        metadata: BackendMetadata {
            id: pleiades_core::BackendId::new("broken"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: pleiades_core::BackendProvenance {
                summary: "  ".to_string(),
                data_sources: vec![],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![CelestialBody::Sun],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        },
        implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
        status_note: "broken for testing",
        expected_error_kinds: &[],
        required_data_files: &[],
    };

    let error = validate_backend_matrix_entry(&entry)
        .expect_err("invalid backend metadata should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .to_string()
        .contains("backend matrix entry `broken backend` has invalid metadata"));
    assert!(error.to_string().contains("provenance summary"));
}

#[test]
fn validated_house_code_aliases_summary_for_report_matches_current_profile() {
    let profile = current_compatibility_profile();
    assert_eq!(
        validated_house_code_aliases_summary_for_profile(&profile).unwrap(),
        profile.house_code_aliases_summary_line()
    );
}

#[test]
fn validated_house_code_aliases_summary_for_report_rejects_invalid_profiles() {
    let profile = CompatibilityProfile {
        summary: "",
        ..current_compatibility_profile()
    };

    let error = validated_house_code_aliases_summary_for_profile(&profile)
        .expect_err("invalid compatibility profiles should be rejected");
    assert!(error.contains("compatibility profile summary is blank"));
}

#[test]
fn format_capabilities_reports_unavailable_for_invalid_flag_sets() {
    let capabilities = BackendCapabilities {
        geocentric: true,
        topocentric: false,
        apparent: false,
        mean: false,
        batch: true,
        native_sidereal: false,
    };

    assert_eq!(
        format_capabilities(&capabilities),
        "unavailable (backend capabilities must support mean or apparent output)"
    );
}

#[test]
fn backend_matrix_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["backend-matrix-summary"]).expect("backend matrix summary should render");
    assert!(rendered.contains("Backend matrix summary"));
    assert!(rendered.contains("Backends: 5"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        current_compatibility_profile().profile_id
    )));
    let jpl_entry = implemented_backend_catalog()
        .into_iter()
        .find(|entry| entry.label == "JPL snapshot reference backend")
        .expect("JPL backend matrix entry should exist");
    assert!(jpl_entry
        .status_note
        .contains("reference corpus now spans 357 rows across 16 bodies and 31 epochs"));
    assert!(rendered.contains("Families:"));
    assert!(rendered.contains("Algorithmic: 2"));
    assert!(rendered.contains("ReferenceData: 1"));
    assert!(rendered.contains("CompressedData: 1"));
    assert!(rendered.contains("Composite: 1"));
    assert!(rendered.contains("Implementation statuses:"));
    assert!(rendered.contains("Native sidereal posture: unsupported across first-party backends"));
    assert!(rendered.contains("Nominal ranges: bounded: 2, open-ended: 3"));
    assert!(rendered.contains("fixture-reference: 1"));
    assert!(rendered.contains("partial-source-backed: 1"));
    assert!(rendered.contains("preliminary-algorithm: 1"));
    assert!(rendered.contains("draft-artifact: 1"));
    assert!(rendered.contains("routing-facade: 1"));
    assert!(rendered.contains("Accuracy classes:"));
    assert!(rendered.contains("Exact: 1"));
    assert!(rendered.contains("Approximate: 4"));
    assert!(rendered.contains("VSOP87 source documentation: 8 source specs, 8 source-backed body profiles, 1 approximate fallback mean-element body profile (Pluto); source-backed bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
    assert!(rendered.contains(
            "source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"
        ));
    assert!(rendered.contains(
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        ));
    assert!(rendered.contains("VSOP87 canonical J2000 batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
    assert!(rendered.contains("JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"));
    assert!(rendered.contains("JPL production-generation coverage:"));
    assert!(rendered.contains("Production generation source windows:"));
    assert!(rendered.contains("JPL production-generation body-class coverage:"));
    assert!(rendered.contains("JPL production-generation corpus shape:"));
    assert!(rendered.contains("JPL production-generation boundary request corpus equatorial:"));
    assert!(rendered.contains("JPL source corpus contract: JPL evidence classification:"));
    assert!(rendered.contains("JPL source posture: documented hybrid snapshot/hold-out fixture backend with a separate generation-input path"));
    assert!(rendered
        .contains("Comparison corpus release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(rendered.contains("Reference/hold-out overlap:"));
    assert!(rendered.contains("JPL independent hold-out:"));
    assert!(rendered.contains("Release-grade body claims:"));
    let source_corpus_summary = source_corpus_summary_for_report();
    assert!(rendered.contains(&format!("Source corpus: {source_corpus_summary}")));
    assert!(rendered
        .lines()
        .any(|line| line == format!("Source corpus posture: {source_corpus_summary}")));
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Body/date/channel claims: {}",
            format_body_date_channel_claims_summary_for_report()
        )
    }));
    assert!(rendered.contains("Catalog posture: house systems="));
    assert!(rendered.contains("Target house scope:"));
    assert!(rendered.contains("Target ayanamsa scope:"));
    assert!(rendered.contains("Pluto fallback: Pluto remains an explicitly approximate fallback"));
    assert!(rendered.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_sparse_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_pre_bridge_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_dense_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(rendered
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
    assert!(rendered
        .contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files"));
    assert!(rendered.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
    assert!(rendered.contains("VSOP87 canonical J2000 interim outliers: none"));
    assert!(rendered.contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
    assert!(rendered.contains("VSOP87 canonical J1900 batch parity:"));
    assert!(rendered.contains("quality counts: Exact=8, Interpolated=0, Approximate=1, Unknown=0"));
    assert!(rendered.contains("generated binary VSOP87B"));
    assert!(rendered.contains("generated binary VSOP87B; VSOP87B."));
    assert!(rendered.contains("max Δlon="));
    assert!(rendered.contains("max Δlat="));
    assert!(rendered.contains("max Δdist="));
    assert!(rendered.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
    assert!(rendered.contains(
            "ELP lunar theory specification: Compact Meeus-style truncated lunar baseline [meeus-style-truncated-lunar-baseline; family: Meeus-style truncated analytical baseline; selected key: source identifier=meeus-style-truncated-lunar-baseline]"
        ));
    assert!(rendered.contains(
            "lunar theory catalog: 1 entry, 1 selected entry; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
    assert!(rendered.contains(
            "lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; selected key: source identifier=meeus-style-truncated-lunar-baseline; selected family key: source family=Meeus-style truncated analytical baseline; aliases=1; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)"
        ));
    assert!(rendered.contains("lunar reference error envelope: 9 samples across 5 bodies"));
    assert!(rendered.contains("max Δlon="));
    assert!(rendered.contains("mean Δlon="));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("max Δlat="));
    assert!(rendered.contains("mean Δlat="));
    assert!(rendered.contains("median Δlat="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("limits: Δlon≤1e-4°"));
    assert!(rendered.contains("request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"));
    assert!(rendered.contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
    assert!(rendered.contains("date-range note: Validated against the published 1992-04-12 geocentric Moon example, the published 1992-04-12 geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform, the reference-only published 1968-12-24 apparent geocentric Moon comparison datum, the reference-only published 2004-04-01 NASA RP 1349 apparent Moon table row, the reference-only published 2006-09-07 EclipseWise apparent Moon coordinate row, J2000 lunar-point anchors including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example"));
    assert!(rendered.contains("lunar equatorial reference evidence: 3 samples across 1 bodies"));
    assert!(
        rendered.contains("lunar equatorial reference error envelope: 3 samples across 1 bodies")
    );
    assert!(rendered.contains("mean ΔRA="));
    assert!(rendered.contains("median ΔRA="));
    assert!(rendered.contains("p95 ΔRA="));
    assert!(rendered.contains("limits: ΔRA≤1e-2°"));
    assert!(rendered.contains("Lunar high-curvature continuity evidence"));
    assert!(rendered.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(
        rendered.contains("lunar high-curvature continuity evidence: 6 samples across 1 bodies")
    );
    assert!(rendered.contains(
        "lunar high-curvature equatorial continuity evidence: 6 samples across 1 bodies"
    ));
    assert!(rendered.contains("within regression limits=true"));
    assert!(rendered.contains("citation: Jean Meeus"));
    assert!(
        rendered.contains("provenance: Published lunar position, node, and mean-point formulas")
    );
    assert!(rendered
        .contains("redistribution: No external coefficient-file redistribution constraints apply"));
    assert!(rendered.contains("license: The current baseline is handwritten pure Rust"));
    assert!(rendered.contains("2 unsupported bodies: True Apogee, True Perigee"));
    assert!(rendered.contains("Distinct bodies covered:"));
    assert!(rendered.contains("Distinct coordinate frames: 2 (Ecliptic, Equatorial)"));
    assert!(rendered.contains("Distinct time scales: 2 (TT, TDB)"));
    assert!(rendered.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
    assert_eq!(
        validated_request_policy_summary_for_report()
            .expect("current request policy summary should validate")
            .summary_line(),
        request_policy_summary_for_report()
            .validated_summary_line()
            .expect("current request policy summary should render")
    );
    assert!(rendered.contains(
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        ));
    assert!(rendered.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        }));
    assert!(rendered
        .contains("pleiades-core::ChartRequest (chart assembly plus house-observer preflight)"));
    assert!(rendered.contains("Native sidereal policy:"));
    assert!(rendered.contains("Frame policy:"));
    let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Mean-obliquity frame round-trip: {}",
            mean_obliquity_frame_round_trip
        )
    }));
    assert!(rendered.contains("Zodiac policy:"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("API stability summary: api-stability-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Reference snapshot coverage: 357 rows across 16 bodies and 31 epochs (95 asteroid rows; JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
    assert!(rendered.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_1500_selected_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_1600_selected_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2500_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
    assert!(
        rendered.contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report())
    );
    assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_manifest_summary_for_report()));
    assert!(rendered.contains("Comparison audit: compare-backends-audit; status="));
    assert!(rendered.contains("within tolerance bodies="));
    assert!(rendered.contains("outside tolerance bodies="));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));

    let capability_matrix =
        render_cli(&["capability-matrix"]).expect("capability matrix should render");
    assert_eq!(
        capability_matrix,
        render_cli(&["backend-matrix"]).expect("backend matrix should render")
    );

    let matrix_summary = render_cli(&["matrix-summary"]).expect("matrix summary should render");
    assert_eq!(matrix_summary, rendered);
}

#[test]
fn workspace_audit_reports_a_clean_workspace() {
    let report = workspace_audit_report().expect("workspace audit should render");
    assert!(report.is_clean());
    assert!(report
        .to_string()
        .contains("no workspace policy violations detected"));
    assert!(report.to_string().contains("Checked manifests:"));
    assert!(report.to_string().contains("Checked tool manifest:"));
}

#[test]
fn workspace_audit_summary_reports_a_clean_workspace() {
    let report = workspace_audit_report().expect("workspace audit should render");
    let summary = workspace_audit_summary(&report);
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.summary_line().contains("violations: 0"));
    assert!(summary
        .summary_line()
        .contains("no workspace policy violations detected"));

    let rendered =
        render_cli(&["workspace-audit-summary"]).expect("workspace audit summary should render");
    let native_dependency_rendered = render_native_dependency_audit_summary()
        .expect("native dependency audit summary should render");
    let alias = render_cli(&["native-dependency-audit-summary"])
        .expect("native dependency audit summary should render");
    assert_eq!(rendered, native_dependency_rendered);
    assert_eq!(rendered, alias);
    assert!(rendered.contains("Workspace audit summary"));
    assert!(rendered.contains("Summary: workspace root:"));
    assert!(rendered.contains("Checked manifests:"));
    assert!(rendered.contains("Checked tool manifest:"));
    assert!(rendered.contains("Checked lockfile:"));
    assert!(rendered.contains("Result: no workspace policy violations detected"));

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
    let summary = workspace_provenance_summary_for_report();
    assert!(summary.contains("Workspace provenance"));
    assert!(summary.contains("source revision:"));
    assert!(summary.contains("workspace status:"));
    assert!(summary.contains("rustc version:"));
    assert!(summary.contains("cargo version:"));
    assert!(summary.contains("rustfmt version:"));
    assert!(summary.contains("clippy version:"));

    let rendered = render_cli(&["workspace-provenance-summary"])
        .expect("workspace provenance summary should render");
    let alias =
        render_cli(&["workspace-provenance"]).expect("workspace provenance alias should render");
    assert_eq!(rendered, summary);
    assert_eq!(rendered, alias);
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
fn workspace_audit_detects_native_hooks_in_manifests_and_lockfile() {
    let manifest = r#"[package]
name = "example"
build = "build.rs"
links = "example-native"

[dependencies]
cc = "1"
openssl-sys = "0.9"
[target.'cfg(unix)'.dependencies]
bindgen = { version = "0.69" }
renamed-native = { package = "zstd-sys", version = "2" }
"#;
    let manifest_violations = audit_manifest_text(Path::new("/tmp/Cargo.toml"), manifest);
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "package.build"));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "package.links"));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.detail.contains("cc")));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.detail.contains("bindgen")));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "dependency.native-package"
            && violation.detail.contains("openssl-sys")));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "dependency.native-package"
            && violation.detail.contains("zstd-sys")));

    let build_script_dir = unique_temp_dir("pleiades-workspace-audit-build-script");
    let build_script_manifest = build_script_dir.join("Cargo.toml");
    let build_script_path = build_script_dir.join("build.rs");
    std::fs::write(
        &build_script_manifest,
        "[package]\nname = \"example-build-script\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be writable");
    std::fs::write(&build_script_path, "fn main() {}\n").expect("build.rs should be writable");
    let build_script_violation =
        audit_build_script_path(&build_script_manifest).expect("build.rs should be detected");
    assert_eq!(build_script_violation.rule, "package.build-script");
    assert_eq!(build_script_violation.path, build_script_path);
    assert!(build_script_violation.detail.contains("build.rs"));

    let lockfile = r#"[[package]]
name = "openssl-sys"
version = "0.9.0"
"#;
    let lockfile_violations = audit_lockfile_text(Path::new("/tmp/Cargo.lock"), lockfile);
    assert!(lockfile_violations
        .iter()
        .any(|violation| violation.rule == "lockfile.native-package"));
}

#[test]
fn workspace_audit_detects_tool_manifest_provenance_drift() {
    let tool_manifest = r#"[tools]
rust = { version = "1.96.0", components = "rustfmt" }
"#;
    let violations = audit_tool_manifest_text(
        Path::new("/tmp/mise.toml"),
        tool_manifest,
        Some("1.95.0".to_string()),
    );

    assert!(violations
        .iter()
        .any(|violation| violation.rule == "tool-manifest.rust-version-mismatch"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "tool-manifest.rust-components-missing"));
}

#[test]
fn workspace_audit_detects_workspace_publish_metadata_drift() {
    let manifest = r#"[workspace.package]
version = "0.1.0"
license = "MIT"

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types", version = "0.2.0" }
pleiades-backend = { version = "0.1.0" }
serde = { version = "1" }
"#;
    let violations = audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);

    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.workspace-license"));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-metadata-missing"
        && violation.detail.contains("repository")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-metadata-missing"
        && violation.detail.contains("keywords")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-dependency-version"
        && violation.detail.contains("pleiades-types")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-dependency-path"
        && violation.detail.contains("pleiades-backend")));
    assert!(!violations
        .iter()
        .any(|violation| violation.detail.contains("`serde`")));
}

#[test]
fn workspace_audit_accepts_publish_ready_workspace_manifest() {
    let manifest = r#"[workspace.package]
version = "0.1.0"
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
serde = { version = "1" }
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
"#;
    let violations = audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
}

#[test]
fn workspace_audit_reports_missing_workspace_version_once() {
    let manifest = r#"[workspace.package]
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
pleiades-backend = { path = "crates/pleiades-backend", version = "0.1.0" }
"#;
    let violations = audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);
    let count = violations
        .iter()
        .filter(|violation| violation.rule == "publish.workspace-version-missing")
        .count();
    assert_eq!(count, 1, "violations: {violations:?}");
}

#[test]
fn workspace_audit_identifies_publishable_packages() {
    assert!(manifest_is_package("[package]\nname = \"a\"\n"));
    assert!(!manifest_is_package("[workspace]\nmembers = []\n"));
    assert!(manifest_declares_publish_false(
        "[package]\nname = \"a\"\npublish = false\n"
    ));
    assert!(manifest_declares_publish_false(
        "[package]\nname = \"a\"\npublish = []\n"
    ));
    assert!(!manifest_declares_publish_false(
        "[package]\nname = \"a\"\n"
    ));
    assert_eq!(
        manifest_package_name("[package]\nname = \"pleiades-types\"\n"),
        Some("pleiades-types".to_string())
    );
}

#[test]
fn workspace_audit_detects_publishable_crate_manifest_gaps() {
    let manifest = r#"[package]
name = "pleiades-example"
version.workspace = true
edition.workspace = true

[dependencies]
pleiades-types = { path = "../pleiades-types" }
pleiades-data = { workspace = true }
serde = { workspace = true, optional = true }
renamed = { package = "pleiades-houses", path = "../pleiades-houses" }

[build-dependencies]
pleiades-elp = { path = "../pleiades-elp" }

[dev-dependencies]
pleiades-jpl = { workspace = true }
"#;
    let publishable = vec!["pleiades-example".to_string(), "pleiades-types".to_string()];
    let violations =
        audit_publishable_manifest_text(Path::new("/tmp/Cargo.toml"), manifest, &publishable);

    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.description-missing"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.license-not-inherited"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.readme-field-missing"));
    assert!(violations.iter().any(
        |violation| violation.rule == "publish.metadata-field-missing"
            && violation.detail.contains("repository")
    ));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-not-workspace"
        && violation.detail.contains("pleiades-types")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-unpublishable"
        && violation.detail.contains("pleiades-data")));
    assert!(!violations
        .iter()
        .any(|violation| violation.detail.contains("`serde`")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-not-workspace"
        && violation.detail.contains("pleiades-houses")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-unpublishable"
        && violation.detail.contains("pleiades-elp")));
    assert!(!violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-unpublishable"
        && violation.detail.contains("pleiades-jpl")));
}

#[test]
fn workspace_audit_accepts_publish_ready_crate_manifest() {
    let manifest = r#"[package]
name = "pleiades-example"
description = "Example publishable crate."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-types = { workspace = true }

[dev-dependencies]
serde_json = "1"
"#;
    let publishable = vec!["pleiades-example".to_string(), "pleiades-types".to_string()];
    let violations =
        audit_publishable_manifest_text(Path::new("/tmp/Cargo.toml"), manifest, &publishable);
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
}

#[test]
fn workspace_audit_detects_publish_file_gaps() {
    let root = unique_temp_dir("pleiades-publish-file-audit");
    let crate_dir = root.join("crates").join("pleiades-example");
    std::fs::create_dir_all(&crate_dir).expect("crate dir should be creatable");
    std::fs::write(root.join("LICENSE-APACHE"), "apache text")
        .expect("root apache license should be writable");
    std::fs::write(root.join("LICENSE-MIT"), "mit text")
        .expect("root mit license should be writable");
    std::fs::write(crate_dir.join("LICENSE-APACHE"), "apache text")
        .expect("crate apache license should be writable");
    std::fs::write(crate_dir.join("LICENSE-MIT"), "different text")
        .expect("crate mit license should be writable");

    let violations = audit_publishable_crate_files(&crate_dir.join("Cargo.toml"), &root);
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.readme-file-missing"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.license-file-drift"
            && violation.detail.contains("LICENSE-MIT")));
    assert!(!violations
        .iter()
        .any(|violation| violation.rule == "publish.license-file-missing"));

    std::fs::write(crate_dir.join("README.md"), "# pleiades-example\n")
        .expect("crate readme should be writable");
    std::fs::write(crate_dir.join("LICENSE-MIT"), "mit text")
        .expect("crate mit license should be writable");
    let violations = audit_publishable_crate_files(&crate_dir.join("Cargo.toml"), &root);
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn workspace_audit_summary_groups_rule_counts_for_violations() {
    let report = WorkspaceAuditReport {
            workspace_root: PathBuf::from("/workspace"),
            manifest_paths: vec![PathBuf::from("/workspace/Cargo.toml")],
            tool_manifest_path: PathBuf::from("/workspace/mise.toml"),
            lockfile_path: PathBuf::from("/workspace/Cargo.lock"),
            violations: vec![
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.toml"),
                    rule: "package.build",
                    detail: "package declares a build script".to_string(),
                },
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.toml"),
                    rule: "package.build",
                    detail: "package declares a build script".to_string(),
                },
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.lock"),
                    rule: "lockfile.native-package",
                    detail: "lockfile package `openssl-sys` suggests a native build dependency and should be reviewed".to_string(),
                },
            ],
        };

    let summary = workspace_audit_summary(&report);
    assert!(summary.summary_line().contains("violations: 3"));
    assert_eq!(
        summary.rule_counts,
        vec![("lockfile.native-package", 1), ("package.build", 2)]
    );
    summary
        .validate()
        .expect("workspace audit summary should validate");

    let rendered = render_workspace_audit_summary_text(&report);
    assert!(rendered.contains("Summary: workspace root:"));
    assert!(rendered.contains("tool manifest:"));
    assert!(rendered.contains("Violations: 3"));
    assert!(rendered.contains("Rule counts:"));
    assert!(rendered.contains("package.build: 2"));
    assert!(rendered.contains("lockfile.native-package: 1"));
    assert!(rendered.contains("Result: violations found"));

    let display = report.to_string();
    assert!(display.contains("Rule counts:"));
    assert!(display.contains("package.build: 2"));
    assert!(display.contains("lockfile.native-package: 1"));
}

#[test]
fn workspace_audit_summary_validate_rejects_incoherent_counts() {
    let summary = WorkspaceAuditSummary {
        workspace_root: PathBuf::from("/workspace"),
        manifest_count: 1,
        tool_manifest_path: PathBuf::from("/workspace/mise.toml"),
        lockfile_path: PathBuf::from("/workspace/Cargo.lock"),
        violation_count: 2,
        rule_counts: vec![("package.build", 1)],
        clean: false,
    };

    let error = summary
        .validate()
        .expect_err("incoherent workspace audit summary should fail validation");
    assert!(error
        .to_string()
        .contains("workspace audit summary violation count mismatch"));
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

#[test]
fn verify_release_bundle_rejects_tampered_release_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-release-summary",
        "release-summary.txt",
        "release summary checksum mismatch",
    );
}

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

#[test]
fn verify_release_bundle_rejects_semantically_tampered_source_corpus_summary_file_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-semantic-source-corpus-summary",
            "source-corpus-summary.txt",
            "source-corpus summary checksum (fnv1a-64):",
            "coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1500-2500 CE window; coverage=",
            "coverage posture=drifted production-generation coverage and corpus shape remain aligned across the advertised 1500-2500 CE window; coverage=",
            "source corpus summary no longer matches the current source-corpus posture",
        );
}

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

#[test]
fn verify_release_bundle_rejects_tampered_release_notes_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-release-notes-summary",
        "release-notes-summary.txt",
        "release notes summary checksum mismatch",
    );
}

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

#[test]
fn verify_release_bundle_rejects_tampered_production_generation_boundary_source_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-boundary-source-semantic",
            "production-generation-boundary-source-summary.txt",
            "production generation boundary source summary checksum (fnv1a-64):",
            "coverage=Mars and Jupiter",
            "coverage=drifted Mars and Jupiter",
            "production generation boundary source summary no longer matches the current production-generation boundary source posture",
        );
}

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

#[test]
fn verify_release_bundle_rejects_tampered_production_generation_boundary_request_corpus_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-boundary-request-corpus-semantic",
            "production-generation-boundary-request-corpus-summary.txt",
            "production generation boundary request corpus summary checksum (fnv1a-64):",
            "84 requests",
            "85 drifted requests",
            "production generation boundary request corpus summary no longer matches the current production-generation boundary request corpus posture",
        );
}

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

#[test]
fn verify_release_bundle_rejects_tampered_production_generation_boundary_request_corpus_equatorial_summary_even_with_updated_checksum(
) {
    assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
            "pleiades-release-bundle-tampered-production-generation-boundary-request-corpus-equatorial-semantic",
            "production-generation-boundary-request-corpus-equatorial-summary.txt",
            "production generation boundary request corpus equatorial summary checksum (fnv1a-64):",
            "84 requests",
            "85 drifted requests",
            "production generation boundary request corpus equatorial summary no longer matches the current production-generation boundary request corpus equatorial posture",
        );
}

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
fn independent_holdout_source_window_summary_validation_rejects_drift() {
    let summary = independent_holdout_snapshot_source_window_summary_for_report();
    let drifted_summary =
        summary.replace("source-backed samples", "tampered source-backed samples");

    let error = ensure_independent_holdout_source_window_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted independent-holdout source window summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current independent-holdout source-window posture"));
}

#[test]
fn independent_holdout_manifest_summary_command_renders_the_manifest_block() {
    let rendered = render_cli(&["independent-holdout-manifest-summary"])
        .expect("independent hold-out manifest summary should render");
    let alias = render_cli(&["independent-holdout-manifest"])
        .expect("independent hold-out manifest alias should render");

    assert_eq!(rendered, independent_holdout_manifest_summary_for_report());
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&["independent-holdout-manifest", "extra"])
            .expect_err("independent hold-out manifest alias should reject extra arguments"),
        "independent-holdout-manifest does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_source_summary_matches_current_rendering() {
    let summary = reference_snapshot_source_summary_for_report();

    ensure_reference_snapshot_source_summary_matches_current_rendering(&summary)
        .expect("reference snapshot source summary should match the current rendering");
}

#[test]
fn reference_snapshot_bridge_day_summary_validation_rejects_drift() {
    let summary = reference_snapshot_bridge_day_summary_for_report();
    let drifted_summary = summary.replace("2451914.0", "2451914.1");

    let error =
        ensure_reference_snapshot_bridge_day_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot bridge day summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot bridge day posture"));
}

#[test]
fn reference_snapshot_source_summary_validation_rejects_drift() {
    let summary = reference_snapshot_source_summary_for_report();
    let drifted_summary = summary.replace("coverage=", "coverage=drifted-");

    let error =
        ensure_reference_snapshot_source_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot source summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot source posture"));
}

#[test]
fn reference_snapshot_manifest_summary_validation_rejects_drift() {
    let summary = reference_snapshot_manifest_summary_for_report();
    let drifted_summary = summary.replace(
        "source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "source=drifted NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
    );

    let error =
        ensure_reference_snapshot_manifest_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot manifest summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot manifest posture"));
}

#[test]
fn reference_snapshot_source_window_summary_validation_rejects_drift() {
    let summary = reference_snapshot_source_window_summary_for_report();
    let drifted_summary = summary.replace(
        "Reference snapshot source windows:",
        "Reference snapshot source windows (drifted):",
    );

    let error =
        ensure_reference_snapshot_source_window_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot source window summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot source-window posture"));
}

#[test]
fn comparison_snapshot_source_summary_matches_current_rendering() {
    let summary = comparison_snapshot_source_summary_for_report();

    ensure_comparison_snapshot_source_summary_matches_current_rendering(&summary)
        .expect("comparison snapshot source summary should match the current rendering");
}

#[test]
fn comparison_snapshot_source_summary_validation_rejects_drift() {
    let summary = comparison_snapshot_source_summary_for_report();
    let drifted_summary = summary.replace("coverage=", "coverage=drifted-");

    let error =
        ensure_comparison_snapshot_source_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted comparison snapshot source summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current comparison snapshot source posture"));
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

#[test]
fn release_profile_identifier_summary_helper_rejects_drifted_pairs() {
    let compatibility_profile_id = current_compatibility_profile().profile_id;
    let release_profiles = ReleaseProfileIdentifiers {
        compatibility_profile_id,
        api_stability_profile_id: compatibility_profile_id,
    };

    let rendered = validated_release_profile_identifiers_summary_for_report(&release_profiles);
    assert!(rendered.contains("unavailable"));
    assert!(rendered.contains("release-profile identifiers must be distinct"));
}

#[test]
fn verify_release_bundle_rejects_tampered_artifact_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-artifact-summary",
        "artifact-summary.txt",
        "artifact summary checksum mismatch",
    );
}

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

#[test]
fn verify_release_bundle_rejects_tampered_compatibility_profile_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-profile",
        "compatibility-profile.txt",
        "compatibility profile checksum mismatch",
    );
}

#[test]
fn verify_release_bundle_rejects_tampered_release_checklist_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-checklist",
        "release-checklist.txt",
        "release checklist checksum mismatch",
    );
}

#[test]
fn verify_release_bundle_rejects_tampered_release_checklist_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-checklist-summary",
        "release-checklist-summary.txt",
        "release checklist summary checksum mismatch",
    );
}

#[test]
fn verify_release_bundle_rejects_tampered_backend_matrix_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-matrix",
        "backend-matrix.txt",
        "backend matrix checksum mismatch",
    );
}

#[test]
fn verify_release_bundle_rejects_tampered_api_stability_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-api-stability",
        "api-stability.txt",
        "API stability checksum mismatch",
    );
}

#[test]
fn verify_release_bundle_rejects_tampered_validation_report_summary_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-validation-report-summary",
        "validation-report-summary.txt",
        "validation report summary checksum mismatch",
    );
}

#[test]
fn verify_release_bundle_rejects_tampered_validation_report_file() {
    assert_release_bundle_rejects_tampered_text_file(
        "pleiades-release-bundle-tampered-validation-report",
        "validation-report.txt",
        "validation report checksum mismatch",
    );
}

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

#[test]
fn artifact_validation_report_mentions_boundary_checks() {
    let report = render_artifact_report().expect("artifact report should render");
    assert!(report.contains("Artifact validation report"));
    assert!(report.contains("stage-5 packaged-data draft"));
    assert!(report.contains("byte order: little-endian"));
    assert!(report.contains("roundtrip decode: ok"));
    assert!(report.contains("checksum verified: ok"));
    assert!(report.contains("Bodies"));
    assert!(report.contains("Sun"));
    assert!(report.contains("Moon"));
    assert!(report.contains("Jupiter"));
    assert!(report.contains("Pluto"));
    assert!(report.contains("boundary checks"));
    assert!(report.contains("Artifact boundary envelope"));
    assert!(report.contains("Artifact request policy"));
    assert!(report.contains("Model error envelope"));
    assert!(report.contains("Body-class error envelopes"));
    assert!(report.contains("Artifact lookup benchmark"));
    assert!(report.contains("Artifact batch lookup benchmark"));
    assert!(report.contains("Artifact decode benchmark"));
    assert!(report.contains("nanoseconds per decode:"));
    assert!(report.contains("decodes per second:"));
    let body_class_envelopes = report
        .split("Body-class error envelopes")
        .nth(1)
        .expect("artifact report should include body-class error envelopes");
    assert!(body_class_envelopes.contains("max longitude delta:"));
    assert!(body_class_envelopes.contains(" ("));
    assert!(report.contains("Luminaries"));
    assert!(report.contains("Major planets"));
    assert!(report.contains("baseline backend"));
}

#[test]
fn comparison_summary_summary_line_includes_bodies_and_counts() {
    let summary = ComparisonSummary {
        sample_count: 3,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };

    let rendered = summary.summary_line();
    assert!(rendered.contains("samples: 3"));
    assert!(rendered.contains("max longitude delta: 0.123456789012° (Sun)"));
    assert!(rendered.contains("max latitude delta: 0.223456789012° (Moon)"));
    assert!(rendered.contains("max distance delta: 0.001234567890 AU (Mars)"));
    assert_eq!(rendered, format!("{summary}"));
    assert_eq!(summary.validated_summary_line(), Ok(rendered));
    assert!(summary.validate().is_ok());
}

#[test]
fn comparison_summary_validated_summary_line_rejects_zero_sample_drift() {
    let summary = ComparisonSummary {
        sample_count: 0,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };

    let error = summary
        .validated_summary_line()
        .expect_err("zero-sample drift should fail validation");
    assert!(error.to_string().contains(
        "comparison summary with zero samples must not carry per-body or distance extrema"
    ));
}

#[test]
fn comparison_envelope_formatter_rejects_empty_sample_slices() {
    let summary = ComparisonSummary {
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };

    let envelope = format_comparison_envelope_for_report(&summary, &[]);
    assert!(envelope.contains("comparison envelope unavailable"));
    assert!(envelope.contains("sample-count mismatch") || envelope.contains("no samples"));

    let percentile = format_comparison_percentile_envelope_for_report(&[]);
    assert!(percentile.contains("comparison percentile envelope unavailable"));
    assert!(percentile.contains("comparison sample slice is empty"));
}

#[test]
fn comparison_envelope_formatter_rejects_mixed_distance_channels() {
    let summary = ComparisonSummary {
        sample_count: 2,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };
    let samples = vec![
        ComparisonSample {
            body: CelestialBody::Sun,
            reference: EclipticCoordinates::new(
                Longitude::from_degrees(10.0),
                Latitude::from_degrees(1.0),
                Some(1.0),
            ),
            candidate: EclipticCoordinates::new(
                Longitude::from_degrees(10.1),
                Latitude::from_degrees(1.1),
                Some(1.1),
            ),
            longitude_delta_deg: 0.1,
            latitude_delta_deg: 0.1,
            distance_delta_au: Some(0.1),
        },
        ComparisonSample {
            body: CelestialBody::Moon,
            reference: EclipticCoordinates::new(
                Longitude::from_degrees(20.0),
                Latitude::from_degrees(2.0),
                None,
            ),
            candidate: EclipticCoordinates::new(
                Longitude::from_degrees(20.2),
                Latitude::from_degrees(2.2),
                None,
            ),
            longitude_delta_deg: 0.2,
            latitude_delta_deg: 0.2,
            distance_delta_au: None,
        },
    ];

    let envelope = format_comparison_envelope_for_report(&summary, &samples);
    assert!(
        envelope.contains("comparison envelope unavailable")
            || envelope.contains("distance deltas must either all be present or all be absent")
    );
}

#[test]
fn mean_obliquity_frame_round_trip_summary_has_a_displayable_summary_line() {
    let summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");

    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary.summary_line().contains("7 samples"));
    assert!(summary.summary_line().contains("max |Δlon|="));
    assert!(summary.summary_line().contains("mean |Δlon|="));
    assert!(summary.summary_line().contains("p95 |Δlon|="));
    assert!(summary.validate().is_ok());
}

#[test]
fn mean_obliquity_frame_round_trip_summary_render_cli_command_matches_display() {
    let rendered = render_cli(&["mean-obliquity-frame-round-trip-summary"])
        .expect("frame round-trip summary command should render");
    let alias = render_cli(&["mean-obliquity-frame-round-trip"])
        .expect("frame round-trip summary alias should render");
    let summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");

    assert_eq!(rendered, summary.to_string());
    assert_eq!(alias, summary.to_string());
    assert_eq!(
        render_cli(&["mean-obliquity-frame-round-trip", "extra"])
            .expect_err("frame round-trip summary alias should reject extra arguments"),
        "mean-obliquity-frame-round-trip does not accept extra arguments"
    );
}

#[test]
fn mean_obliquity_frame_round_trip_sample_corpus_matches_the_canonical_summary() {
    let samples = mean_obliquity_frame_round_trip_sample_corpus();

    assert_eq!(samples.len(), 7);
    assert!(samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() > 80.0));
    assert!(samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() < -80.0));
    assert!(samples
        .iter()
        .any(|(coordinates, _)| coordinates.longitude.degrees() > 350.0));
    assert_eq!(
        samples
            .first()
            .expect("sample corpus should not be empty")
            .1
            .scale,
        TimeScale::Tt
    );
    assert_eq!(
        samples
            .last()
            .expect("sample corpus should not be empty")
            .1
            .scale,
        TimeScale::Tt
    );

    let summary = mean_obliquity_frame_round_trip_summary_from_samples(&samples)
        .expect("canonical sample corpus should remain valid");
    assert_eq!(summary, mean_obliquity_frame_round_trip_summary().unwrap());
}

#[test]
fn mean_obliquity_frame_round_trip_sample_corpus_rejects_missing_equator() {
    let mut samples = mean_obliquity_frame_round_trip_sample_corpus();
    samples[1].0 = EclipticCoordinates::new(
        Longitude::from_degrees(90.0),
        pleiades_core::Latitude::from_degrees(1.0),
        Some(1.0),
    );

    let error = mean_obliquity_frame_round_trip_summary_from_samples(&samples)
        .expect_err("sample corpus without an equatorial sample should fail");
    assert!(error.contains("must include an equatorial sample"));
}

#[test]
fn mean_obliquity_frame_round_trip_sample_corpus_rejects_missing_wraparound() {
    let mut samples = mean_obliquity_frame_round_trip_sample_corpus();
    samples[5].0 = EclipticCoordinates::new(
        Longitude::from_degrees(320.0),
        pleiades_core::Latitude::from_degrees(89.25),
        Some(0.5),
    );

    let error = mean_obliquity_frame_round_trip_summary_from_samples(&samples)
        .expect_err("sample corpus without a wraparound sample should fail");
    assert!(error.contains("must include a wraparound longitude sample"));
}

#[test]
fn mean_obliquity_frame_round_trip_summary_formatter_rejects_drift() {
    let mut summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    summary.sample_count = 0;

    let rendered = format_mean_obliquity_frame_round_trip_summary_for_report(&summary);
    assert!(rendered.contains("mean-obliquity frame round-trip unavailable"));
    assert!(rendered.contains("mean-obliquity frame round-trip summary has no samples"));
}

#[test]
fn mean_obliquity_frame_round_trip_summary_validate_rejects_metric_drift() {
    let mut summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    summary.mean_longitude_delta_deg += 0.001;

    let error = summary
        .validate()
        .expect_err("metric drift should fail validation");
    assert!(error.contains("drifted from the canonical sample set"));
}

#[test]
fn mean_obliquity_frame_round_trip_summary_validated_summary_line_matches_display() {
    let summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");

    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.to_string()
    );
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

#[test]
fn body_class_coverage_summary_commands_render_the_matching_blocks() {
    let production_generation = render_cli(&["production-generation-body-class-coverage-summary"])
        .expect("production generation body-class coverage summary should render");
    assert!(production_generation.contains("Production generation body-class coverage:"));
    let production_generation_alias = render_cli(&["production-body-class-coverage-summary"])
        .expect("production body-class coverage summary alias should render");
    assert_eq!(production_generation_alias, production_generation);
    assert_eq!(
        production_generation,
        production_generation_snapshot_body_class_coverage_summary_for_report()
    );

    let comparison = render_cli(&["comparison-snapshot-body-class-coverage-summary"])
        .expect("comparison snapshot body-class coverage summary should render");
    assert!(comparison.contains("Comparison snapshot body-class coverage:"));
    let comparison_alias = render_cli(&["comparison-body-class-coverage-summary"])
        .expect("comparison body-class coverage summary alias should render");
    assert_eq!(comparison_alias, comparison);
    assert_eq!(
        comparison,
        comparison_snapshot_body_class_coverage_summary_for_report()
    );

    let reference = render_cli(&["reference-snapshot-body-class-coverage-summary"])
        .expect("reference snapshot body-class coverage summary should render");
    assert!(reference.contains("Reference snapshot body-class coverage:"));
    let reference_alias = render_cli(&["reference-body-class-coverage-summary"])
        .expect("reference body-class coverage summary alias should render");
    assert_eq!(reference_alias, reference);
    assert_eq!(
        reference,
        reference_snapshot_body_class_coverage_summary_for_report()
    );

    let independent_holdout_source_window =
        render_cli(&["independent-holdout-source-window-summary"])
            .expect("independent hold-out source window summary should render");
    assert!(independent_holdout_source_window.contains("Independent hold-out source windows:"));
    assert!(independent_holdout_source_window.contains("source-backed samples"));
    assert_eq!(
        independent_holdout_source_window,
        independent_holdout_snapshot_source_window_summary_for_report()
    );

    let independent_holdout_quarter_day_boundary =
        render_cli(&["independent-holdout-quarter-day-boundary-summary"])
            .expect("independent hold-out quarter-day boundary summary should render");
    assert!(independent_holdout_quarter_day_boundary
        .contains("Independent hold-out quarter-day boundary samples:"));
    assert!(independent_holdout_quarter_day_boundary.contains("quarter-day boundary samples"));
    assert_eq!(
        independent_holdout_quarter_day_boundary,
        independent_holdout_snapshot_quarter_day_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["independent-holdout-quarter-day-boundary"])
            .expect("independent hold-out quarter-day boundary alias should render"),
        independent_holdout_quarter_day_boundary
    );

    let independent_holdout = render_cli(&["independent-holdout-summary"])
        .expect("independent hold-out summary should render");
    assert!(independent_holdout.contains("JPL independent hold-out:"));
    assert!(independent_holdout.contains("transparency evidence only"));
    assert_eq!(
        independent_holdout,
        jpl_independent_holdout_summary_for_report()
    );

    let independent_holdout_source = render_cli(&["independent-holdout-source-summary"])
        .expect("independent hold-out source summary should render");
    assert!(independent_holdout_source.contains("Independent hold-out source:"));
    assert!(independent_holdout_source.contains("hold-out source"));
    assert_eq!(
        independent_holdout_source,
        independent_holdout_source_summary_for_report()
    );

    let independent_holdout_high_curvature =
        render_cli(&["independent-holdout-high-curvature-summary"])
            .expect("independent hold-out high-curvature summary should render");
    assert!(independent_holdout_high_curvature
        .contains("JPL independent hold-out high-curvature evidence:"));
    assert!(independent_holdout_high_curvature.contains("high-curvature interpolation window"));
    assert_eq!(
        independent_holdout_high_curvature,
        independent_holdout_high_curvature_summary_for_report()
    );

    let holdout = render_cli(&["independent-holdout-body-class-coverage-summary"])
        .expect("independent hold-out body-class coverage summary should render");
    assert!(holdout.contains("Independent hold-out body-class coverage:"));
    let holdout_alias = render_cli(&["holdout-body-class-coverage-summary"])
        .expect("holdout body-class coverage summary alias should render");
    assert_eq!(holdout_alias, holdout);
    assert_eq!(
        holdout,
        independent_holdout_snapshot_body_class_coverage_summary_for_report()
    );
}

#[test]
fn comparison_snapshot_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["comparison-snapshot-source-window-summary"])
        .expect("comparison snapshot source window summary should render");

    assert!(rendered.contains("Comparison snapshot source windows:"));
    assert!(rendered.contains("232 source-backed samples"));
    assert_eq!(
        rendered,
        comparison_snapshot_source_window_summary_for_report()
    );
}

#[test]
fn comparison_snapshot_source_window_alias_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["comparison-snapshot-source-window"])
        .expect("comparison snapshot source window alias should render");

    assert_eq!(
        rendered,
        comparison_snapshot_source_window_summary_for_report()
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-source-window", "extra"])
            .expect_err("comparison snapshot source window alias should reject extra arguments"),
        "comparison-snapshot-source-window does not accept extra arguments"
    );
}

#[test]
fn comparison_and_reference_snapshot_source_summary_commands_render_the_source_blocks() {
    let comparison = render_cli(&["comparison-snapshot-source-summary"])
        .expect("comparison snapshot source summary should render");
    assert!(comparison.contains("Comparison snapshot source:"));
    assert_eq!(comparison, comparison_snapshot_source_summary_for_report());

    let reference = render_cli(&["reference-snapshot-source-summary"])
        .expect("reference snapshot source summary should render");
    assert!(reference.contains("Reference snapshot source:"));
    assert_eq!(reference, reference_snapshot_source_summary_for_report());

    let comparison_alias = render_cli(&["comparison-snapshot-source"])
        .expect("comparison snapshot source alias should render");
    assert_eq!(
        comparison_alias,
        comparison_snapshot_source_summary_for_report()
    );

    let reference_alias = render_cli(&["reference-snapshot-source"])
        .expect("reference snapshot source alias should render");
    assert_eq!(
        reference_alias,
        reference_snapshot_source_summary_for_report()
    );
}

#[test]
fn reference_major_body_bridge_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-major-body-bridge-summary"])
        .expect("reference major-body bridge summary should render");
    assert!(rendered.contains("Reference major-body bridge evidence:"));
    assert!(rendered.contains("2451915.0"));
    assert_eq!(
        rendered,
        reference_snapshot_major_body_bridge_summary_for_report()
    );
    let alias =
        render_cli(&["major-body-bridge-summary"]).expect("major body bridge alias should render");
    assert_eq!(alias, rendered);
    let concise_alias = render_cli(&["bridge-summary"]).expect("bridge alias should render");
    assert_eq!(concise_alias, rendered);
    let epoch_alias = render_cli(&["2451915-major-body-bridge-summary"])
        .expect("2451915 major body bridge alias should render");
    assert!(epoch_alias.contains("Reference 2451915 major-body bridge evidence:"));
    assert_eq!(
        epoch_alias,
        pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary_for_report()
    );
    let concise_epoch_alias = render_cli(&["2451915-major-body-bridge"])
        .expect("2451915 major body bridge concise alias should render");
    assert_eq!(concise_epoch_alias, epoch_alias);
    assert_eq!(
        render_cli(&["major-body-bridge-summary", "extra"])
            .expect_err("major body bridge alias should reject extra arguments"),
        "major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["bridge-summary", "extra"])
            .expect_err("bridge alias should reject extra arguments"),
        "bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451915-major-body-bridge-summary", "extra"])
            .expect_err("2451915 major body bridge alias should reject extra arguments"),
        "reference-snapshot-2451915-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451915-major-body-bridge", "extra"])
            .expect_err("2451915 major body bridge concise alias should reject extra arguments"),
        "reference-snapshot-2451915-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-major-body-bridge-summary", "extra"])
            .expect_err("reference major-body bridge summary should reject extra arguments"),
        "reference-snapshot-major-body-bridge-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451917_major_body_bridge_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-2451917-major-body-bridge-summary"])
        .expect("reference 2451917 major-body bridge summary should render");
    assert!(rendered.contains("Reference 2451917 major-body bridge evidence:"));
    assert!(rendered.contains("2451917.0"));
    assert_eq!(
        rendered,
        reference_snapshot_2451917_major_body_bridge_summary_for_report()
    );
    let alias = render_cli(&["2451917-major-body-bridge-summary"])
        .expect("2451917 major body bridge alias should render");
    assert_eq!(alias, rendered);
    let concise_alias = render_cli(&["2451917-major-body-bridge"])
        .expect("2451917 major body bridge concise alias should render");
    assert_eq!(concise_alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451917-major-body-bridge-summary",
            "extra"
        ])
        .expect_err("reference 2451917 major body bridge summary should reject extra arguments"),
        "reference-snapshot-2451917-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451917-major-body-bridge-summary", "extra"])
            .expect_err("2451917 major body bridge alias should reject extra arguments"),
        "reference-snapshot-2451917-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451917-major-body-bridge", "extra"])
            .expect_err("2451917 major body bridge concise alias should reject extra arguments"),
        "reference-snapshot-2451917-major-body-bridge-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_bridge_day_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-bridge-day-summary"])
        .expect("reference snapshot bridge day summary should render");
    assert!(rendered.contains("Reference snapshot bridge day:"));
    assert!(rendered.contains("2451914.0"));
    assert_eq!(rendered, reference_snapshot_bridge_day_summary_for_report());
    assert_eq!(
        rendered,
        reference_snapshot_2451914_bridge_day_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-bridge-day-summary", "extra"])
            .expect_err("reference snapshot bridge day summary should reject extra arguments"),
        "reference-snapshot-bridge-day-summary does not accept extra arguments"
    );
    let bridge_day_epoch_alias = render_cli(&["2451914-bridge-day-summary"])
        .expect("2451914 bridge day alias should render");
    assert_eq!(bridge_day_epoch_alias, rendered);
    assert_eq!(
        render_cli(&["2451914-bridge-day-summary", "extra"])
            .expect_err("2451914 bridge day alias should reject extra arguments"),
        "reference-snapshot-2451914-bridge-day-summary does not accept extra arguments"
    );
    let bridge_day_major_alias = render_cli(&["2451914-major-body-bridge-day-summary"])
        .expect("2451914 major body bridge-day alias should render");
    assert_eq!(
        bridge_day_major_alias,
        reference_snapshot_2451914_major_body_bridge_day_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451914-major-body-bridge-day-summary", "extra"])
            .expect_err("2451914 major body bridge-day alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-bridge-day-summary does not accept extra arguments"
    );
    let bridge_epoch_alias = render_cli(&["2451914-major-body-bridge-summary"])
        .expect("2451914 major body bridge alias should render");
    assert_eq!(bridge_epoch_alias, rendered);
    let concise_bridge_epoch_alias = render_cli(&["2451914-major-body-bridge"])
        .expect("2451914 major body bridge concise alias should render");
    assert_eq!(concise_bridge_epoch_alias, rendered);
    assert_eq!(
        render_cli(&["2451914-major-body-bridge-summary", "extra"])
            .expect_err("2451914 major body bridge alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451914-major-body-bridge", "extra"])
            .expect_err("2451914 major body bridge concise alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-bridge-summary does not accept extra arguments"
    );
    let bridge_day_alias =
        render_cli(&["bridge-day-summary"]).expect("bridge day alias should render");
    assert_eq!(bridge_day_alias, rendered);
    assert_eq!(
        render_cli(&["bridge-day-summary", "extra"])
            .expect_err("bridge day alias should reject extra arguments"),
        "bridge-day-summary does not accept extra arguments"
    );
}

#[test]
fn help_text_lists_the_bridge_day_and_boundary_epoch_coverage_commands() {
    let help = render_cli(&["help"]).expect("help text should render");
    assert!(help.contains(
            "reference-snapshot-bridge-day-summary  Print the compact reference snapshot bridge day summary"
        ));
    assert!(
        help.contains("bridge-day-summary       Alias for reference-snapshot-bridge-day-summary")
    );
    assert!(help.contains(
            "reference-snapshot-boundary-epoch-coverage-summary  Print the compact reference snapshot boundary epoch coverage summary"
        ));
    assert!(help.contains(
            "reference-snapshot-boundary-epoch-coverage  Alias for reference-snapshot-boundary-epoch-coverage-summary"
        ));
    assert!(help.contains(
            "boundary-epoch-coverage-summary  Alias for reference-snapshot-boundary-epoch-coverage-summary"
        ));
    assert!(help.contains(
            "reference-snapshot-pre-bridge-boundary  Alias for reference-snapshot-pre-bridge-boundary-summary"
        ));
    assert!(help.contains(
        "production-generation-boundary         Alias for production-generation-boundary-summary"
    ));
    assert!(help.contains("source-corpus-posture-summary  Alias for source-corpus-summary"));
    assert!(help.contains("source-corpus-posture     Alias for source-corpus-posture-summary"));
    assert!(help.contains("ayanamsa-audit-summary    Print the compact ayanamsa audit summary"));
    assert!(help.contains("ayanamsa-audit            Alias for ayanamsa-audit-summary"));
}

#[test]
fn reference_snapshot_major_body_boundary_window_summary_command_renders_the_boundary_window_block()
{
    let rendered = render_cli(&["reference-snapshot-major-body-boundary-window-summary"])
        .expect("reference major-body boundary window summary should render");
    assert!(rendered.contains("Reference major-body boundary windows:"));
    assert_eq!(
        rendered,
        reference_snapshot_major_body_boundary_window_summary_for_report()
    );
    let alias = render_cli(&["major-body-boundary-window-summary"])
        .expect("major body boundary window alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-major-body-boundary-window-summary",
            "extra"
        ])
        .expect_err("reference major-body boundary window summary should reject extra arguments"),
        "reference-snapshot-major-body-boundary-window-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["major-body-boundary-window-summary", "extra"])
            .expect_err("major body boundary window alias should reject extra arguments"),
        "reference-snapshot-major-body-boundary-window-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_boundary_epoch_coverage_summary_command_renders_the_boundary_epoch_coverage_block(
) {
    let rendered = render_cli(&["reference-snapshot-boundary-epoch-coverage-summary"])
        .expect("reference snapshot boundary epoch coverage summary should render");
    assert!(rendered.contains("Reference snapshot boundary epoch coverage:"));
    assert_eq!(
        rendered,
        reference_snapshot_boundary_epoch_coverage_summary_for_report()
    );
    let alias = render_cli(&["boundary-epoch-coverage-summary"])
        .expect("boundary epoch coverage alias should render");
    let reference_alias = render_cli(&["reference-snapshot-boundary-epoch-coverage"])
        .expect("reference snapshot boundary epoch coverage alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(reference_alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-boundary-epoch-coverage-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot boundary epoch coverage summary should reject extra arguments"
        ),
        "reference-snapshot-boundary-epoch-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-epoch-coverage", "extra"]).expect_err(
            "reference snapshot boundary epoch coverage alias should reject extra arguments"
        ),
        "reference-snapshot-boundary-epoch-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["boundary-epoch-coverage-summary", "extra"])
            .expect_err("boundary epoch coverage alias should reject extra arguments"),
        "reference-snapshot-boundary-epoch-coverage-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_bridge_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["selected-asteroid-bridge-summary"])
        .expect("selected asteroid bridge summary should render");
    assert!(rendered.contains("Selected asteroid bridge evidence:"));
    assert!(rendered.contains("2451915.0"));
    assert_eq!(rendered, selected_asteroid_bridge_summary_for_report());

    assert_eq!(
        render_cli(&["selected-asteroid-bridge-summary", "extra"])
            .expect_err("selected asteroid bridge summary should reject extra arguments"),
        "selected-asteroid-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-selected-asteroid-bridge-summary",
            "extra"
        ])
        .expect_err(
            "reference-snapshot-selected-asteroid-bridge-summary should reject extra arguments"
        ),
        "reference-snapshot-selected-asteroid-bridge-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_source_evidence_summary_command_renders_the_source_evidence_block() {
    let rendered = render_cli(&["selected-asteroid-source-evidence-summary"])
        .expect("selected asteroid source evidence summary should render");
    assert!(rendered.contains("Selected asteroid source evidence:"));
    assert!(rendered.contains("Ceres"));
    assert_eq!(
        rendered,
        selected_asteroid_source_evidence_summary_for_report()
    );

    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-summary"])
            .expect("reference snapshot selected asteroid source summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-summary"])
            .expect("selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-summary", "extra"])
            .expect_err("selected asteroid source summary alias should reject extra arguments"),
        "selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2378498_source_summary_command_renders_the_epoch_slice() {
    let rendered = render_cli(&["reference-snapshot-2378498-selected-asteroid-source-summary"])
        .expect("reference snapshot 2378498 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2378498.5 source evidence:"));
    assert!(rendered.contains("JD 2378498.5 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2378498_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2378498-selected-asteroid-source-summary"])
            .expect("2378498 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2378498-selected-asteroid-source-summary", "extra"]).expect_err(
            "2378498 selected asteroid source summary alias should reject extra arguments"
        ),
        "2378498-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2001_source_summary_command_renders_the_epoch_slice() {
    let rendered = render_cli(&["reference-snapshot-2451917-selected-asteroid-source-summary"])
        .expect("reference snapshot 2451917 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2001-01-08 source evidence:"));
    assert!(rendered.contains("JD 2451917.5 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2451917_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451917-selected-asteroid-source-summary"])
            .expect("2451917 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2451917-selected-asteroid-source-summary", "extra"]).expect_err(
            "2451917 selected asteroid source summary alias should reject extra arguments"
        ),
        "2451917-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2003_source_summary_command_renders_the_epoch_slice() {
    let rendered = render_cli(&["reference-snapshot-2453000-selected-asteroid-source-summary"])
        .expect("reference snapshot 2453000 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2003-12-27 source evidence:"));
    assert!(rendered.contains("JD 2453000.5 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2453000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2453000-selected-asteroid-source-summary"])
            .expect("2453000 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2453000-selected-asteroid-source-summary", "extra"]).expect_err(
            "2453000 selected asteroid source summary alias should reject extra arguments"
        ),
        "2453000-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2500000_source_summary_command_renders_the_late_boundary_slice() {
    let rendered = render_cli(&["reference-snapshot-2500000-selected-asteroid-source-summary"])
        .expect("reference snapshot 2500000 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2500000 source evidence:"));
    assert!(rendered.contains("JD 2500000.0 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2500000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2500000-selected-asteroid-source-summary"])
            .expect("2500000 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2500000-selected-asteroid-source-summary", "extra"]).expect_err(
            "2500000 selected asteroid source summary alias should reject extra arguments"
        ),
        "2500000-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2634167_source_summary_command_renders_the_outer_boundary_slice() {
    let rendered = render_cli(&["reference-snapshot-2634167-selected-asteroid-source-summary"])
        .expect("reference snapshot 2634167 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2634167 source evidence:"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2634167_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2634167-selected-asteroid-source-summary"])
            .expect("2634167 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2634167-selected-asteroid-source-summary", "extra"]).expect_err(
            "2634167 selected asteroid source summary alias should reject extra arguments"
        ),
        "2634167-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_source_request_corpus_summary_command_renders_the_request_corpus_slice() {
    let rendered = render_cli(&["selected-asteroid-source-request-corpus-summary"])
        .expect("selected asteroid source request corpus summary should render");
    assert!(rendered.contains("Selected asteroid source request corpus:"));
    assert!(rendered.contains("observerless"));
    assert_eq!(
        rendered,
        validated_selected_asteroid_source_request_corpus_summary_for_report()
            .expect("selected asteroid source request corpus summary should validate")
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus"])
            .expect("selected asteroid source request corpus alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus-equatorial-summary"])
            .expect("selected asteroid source request corpus equatorial summary should render"),
        pleiades_jpl::selected_asteroid_source_request_corpus_equatorial_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus-equatorial"])
            .expect("selected asteroid source request corpus equatorial alias should render"),
        pleiades_jpl::selected_asteroid_source_request_corpus_equatorial_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus-summary", "extra"]).expect_err(
            "selected asteroid source request corpus summary should reject extra arguments"
        ),
        "selected-asteroid-source-request-corpus-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["selected-asteroid-source-window-summary"])
        .expect("selected asteroid source window summary should render");
    assert!(rendered.contains("Selected asteroid source windows:"));
    assert!(rendered.contains("Ceres"));
    assert_eq!(
        rendered,
        validated_selected_asteroid_source_window_summary_for_report()
            .expect("selected asteroid source window summary should validate")
    );

    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-window-summary"])
            .expect("reference snapshot selected asteroid source window summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-window"])
            .expect("reference snapshot selected asteroid source window alias should render"),
        rendered
    );
    assert_eq!(
            render_cli(&["reference-snapshot-selected-asteroid-source-window", "extra"])
                .expect_err("reference snapshot selected asteroid source window alias should reject extra arguments"),
            "reference-snapshot-selected-asteroid-source-window does not accept extra arguments"
        );
    assert_eq!(
        render_cli(&["selected-asteroid-source-window"])
            .expect("selected asteroid source window alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-window", "extra"])
            .expect_err("selected asteroid source window alias should reject extra arguments"),
        "selected-asteroid-source-window does not accept extra arguments"
    );
}

#[test]
fn reference_asteroid_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["reference-asteroid-source-window-summary"])
        .expect("reference asteroid source window summary should render");
    assert!(rendered.contains("Reference asteroid source windows:"));
    assert!(rendered.contains("Ceres"));
    assert_eq!(
        rendered,
        validated_reference_asteroid_source_window_summary_for_report()
            .expect("reference asteroid source window summary should validate")
    );

    assert_eq!(
        render_cli(&["reference-asteroid-source-window"])
            .expect("reference asteroid source window alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-asteroid-source-window", "extra"])
            .expect_err("reference asteroid source window alias should reject extra arguments"),
        "reference-asteroid-source-window-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-asteroid-source-summary"])
            .expect("reference asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-asteroid-source-summary", "extra"])
            .expect_err("reference asteroid source summary alias should reject extra arguments"),
        "reference-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn reference_asteroid_equatorial_evidence_summary_command_renders_the_equatorial_evidence_block() {
    let rendered = render_cli(&["reference-asteroid-equatorial-evidence-summary"])
        .expect("reference asteroid equatorial evidence summary should render");
    assert!(rendered.contains("Selected asteroid equatorial evidence:"));
    assert!(rendered.contains("mean-obliquity equatorial transform"));
    assert_eq!(
        rendered,
        reference_asteroid_equatorial_evidence_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-asteroid-equatorial-evidence"])
            .expect("reference asteroid equatorial evidence alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-asteroid-equatorial-evidence", "extra"]).expect_err(
            "reference asteroid equatorial evidence alias should reject extra arguments"
        ),
        "reference-asteroid-equatorial-evidence-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_boundary_day_summary_command_renders_the_boundary_day() {
    let rendered =
        render_cli(&["boundary-day-summary"]).expect("boundary day summary alias should render");
    assert!(rendered.contains("Reference snapshot boundary day:"));
    assert!(rendered.contains("2451915.5"));
    assert_eq!(
        rendered,
        reference_snapshot_sparse_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day-summary"])
            .expect("reference snapshot boundary day summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day"])
            .expect("reference snapshot boundary day alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-sparse-boundary-summary"])
            .expect("reference snapshot sparse boundary summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["boundary-day-summary", "extra"])
            .expect_err("boundary day summary alias should reject extra arguments"),
        "reference-snapshot-boundary-day-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day-summary", "extra"])
            .expect_err("reference snapshot boundary day summary should reject extra arguments"),
        "reference-snapshot-boundary-day-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day", "extra"])
            .expect_err("reference snapshot boundary day alias should reject extra arguments"),
        "reference-snapshot-boundary-day-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-sparse-boundary-summary", "extra"])
            .expect_err("reference snapshot sparse boundary summary should reject extra arguments"),
        "reference-snapshot-sparse-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_pre_bridge_boundary_summary_command_renders_the_pre_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-pre-bridge-boundary-summary"])
        .expect("reference snapshot pre-bridge boundary summary should render");
    assert!(rendered.contains("Reference snapshot pre-bridge boundary day:"));
    assert!(rendered.contains(
            "JD 2451914.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); pre-bridge boundary day"
        ));
    assert_eq!(
        rendered,
        reference_snapshot_pre_bridge_boundary_summary_for_report()
    );

    let pre_bridge_alias = render_cli(&["pre-bridge-boundary-summary"])
        .expect("pre-bridge boundary summary alias should render");
    let pre_bridge_reference_alias = render_cli(&["reference-snapshot-pre-bridge-boundary"])
        .expect("reference snapshot pre-bridge boundary alias should render");
    assert_eq!(pre_bridge_alias, rendered);
    assert_eq!(pre_bridge_reference_alias, rendered);
    let pre_bridge_epoch_alias = render_cli(&["2451914-major-body-pre-bridge-summary"])
        .expect("2451914 major-body pre-bridge summary alias should render");
    assert_eq!(pre_bridge_epoch_alias, rendered);
    assert_eq!(
        render_cli(&["reference-snapshot-pre-bridge-boundary-summary", "extra"]).expect_err(
            "reference snapshot pre-bridge boundary summary should reject extra arguments"
        ),
        "reference-snapshot-pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-pre-bridge-boundary", "extra"]).expect_err(
            "reference snapshot pre-bridge boundary alias should reject extra arguments"
        ),
        "reference-snapshot-pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["pre-bridge-boundary-summary", "extra"])
            .expect_err("pre-bridge boundary summary alias should reject extra arguments"),
        "pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451914-major-body-pre-bridge-summary", "extra"]).expect_err(
            "2451914 major-body pre-bridge summary alias should reject extra arguments"
        ),
        "reference-snapshot-2451914-major-body-pre-bridge-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_dense_boundary_summary_command_renders_the_dense_day() {
    let rendered = render_cli(&["reference-snapshot-dense-boundary-summary"])
        .expect("reference snapshot dense boundary summary should render");
    assert!(rendered.contains("Reference snapshot dense boundary day:"));
    assert!(rendered.contains(
            "JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        ));
    assert_eq!(
        rendered,
        reference_snapshot_dense_boundary_summary_for_report()
    );

    let dense_boundary_alias = render_cli(&["dense-boundary-summary"])
        .expect("dense boundary summary alias should render");
    assert_eq!(dense_boundary_alias, rendered);
    assert_eq!(
        render_cli(&["reference-snapshot-dense-boundary-summary", "extra"])
            .expect_err("reference snapshot dense boundary summary should reject extra arguments"),
        "reference-snapshot-dense-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["dense-boundary-summary", "extra"])
            .expect_err("dense boundary summary alias should reject extra arguments"),
        "dense-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451916_major_body_dense_boundary_summary_command_renders_the_dense_day() {
    let rendered = render_cli(&["reference-snapshot-2451916-major-body-dense-boundary-summary"])
        .expect("reference snapshot 2451916 major-body dense boundary summary should render");
    assert!(rendered.contains("Reference 2451916 major-body dense boundary evidence:"));
    assert!(rendered.contains("JD 2451916.5 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()
    );

    let alias = render_cli(&["2451916-major-body-dense-boundary-summary"])
        .expect("2451916 major-body dense boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
            render_cli(&["reference-snapshot-2451916-major-body-dense-boundary-summary", "extra"]).expect_err(
                "reference snapshot 2451916 major-body dense boundary summary should reject extra arguments"
            ),
            "reference-snapshot-2451916-major-body-dense-boundary-summary does not accept extra arguments"
        );
    assert_eq!(
            render_cli(&["2451916-major-body-dense-boundary-summary", "extra"])
                .expect_err("2451916 major-body dense boundary alias should reject extra arguments"),
            "reference-snapshot-2451916-major-body-dense-boundary-summary does not accept extra arguments"
        );
}

#[test]
fn selected_asteroid_dense_boundary_summary_command_renders_the_dense_day() {
    let rendered = render_cli(&["selected-asteroid-dense-boundary-summary"])
        .expect("selected asteroid dense boundary summary should render");
    assert!(rendered.contains("Selected asteroid dense boundary evidence:"));
    assert!(rendered.contains("2451916.5"));
    assert_eq!(
        rendered,
        selected_asteroid_dense_boundary_summary_for_report()
    );

    assert_eq!(
        render_cli(&["selected-asteroid-dense-boundary-summary", "extra"])
            .expect_err("selected asteroid dense boundary summary should reject extra arguments"),
        "selected-asteroid-dense-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_terminal_boundary_summary_command_renders_the_terminal_day() {
    let rendered = render_cli(&["selected-asteroid-terminal-boundary-summary"])
        .expect("selected asteroid terminal boundary summary should render");
    assert!(rendered.contains("Reference selected-asteroid terminal boundary evidence:"));
    assert!(rendered.contains("2500-01-01"));
    assert_eq!(
        rendered,
        selected_asteroid_terminal_boundary_summary_for_report()
    );

    assert_eq!(
        render_cli(&["selected-asteroid-terminal-boundary-summary", "extra"]).expect_err(
            "selected asteroid terminal boundary summary should reject extra arguments"
        ),
        "selected-asteroid-terminal-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn request_policy_report_title_validation_rejects_drift() {
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Policy,
        "Request semantics summary\n",
    )
    .is_err());
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Semantics,
        "Request policy summary\n",
    )
    .is_err());
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Policy,
        "Request policy summary\n",
    )
    .is_ok());
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Semantics,
        "Request semantics summary\n",
    )
    .is_ok());
}

#[test]
fn request_policy_summary_and_alias_commands_render_the_policy_block() {
    let request_policy =
        render_cli(&["request-policy-summary"]).expect("request policy summary should render");
    assert_eq!(request_policy, render_request_policy_summary());

    let request_policy_alias =
        render_cli(&["request-policy"]).expect("request policy alias should render");
    assert_eq!(request_policy_alias, request_policy);

    let utc_convenience_policy = render_cli(&["utc-convenience-policy-summary"])
        .expect("UTC convenience policy summary should render");
    assert!(utc_convenience_policy.contains("UTC convenience policy summary"));
    assert!(utc_convenience_policy.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
    assert_eq!(
        render_cli(&["utc-convenience-policy"])
            .expect("UTC convenience policy alias should render"),
        utc_convenience_policy
    );

    let request_semantics = render_cli(&["request-semantics-summary"])
        .expect("request semantics summary should render");
    assert!(request_semantics.contains("Request semantics summary"));
    assert!(request_semantics.contains("Unsupported modes:"));
    assert_eq!(
            request_semantics,
            format!(
                "{}Unsupported modes: built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; topocentric body positions remain unsupported; apparent-place corrections are rejected unless a backend explicitly advertises support; native sidereal backend output remains unsupported unless a backend explicitly advertises it\n",
                request_policy.replacen("Request policy summary", "Request semantics summary", 1)
            )
        );

    let request_semantics_alias =
        render_cli(&["request-semantics"]).expect("request semantics alias should render");
    assert!(request_semantics_alias.contains("Request semantics summary"));
    assert!(request_semantics_alias.contains("Unsupported modes:"));
    assert_eq!(request_semantics_alias, request_semantics);

    let unsupported_modes_summary = render_cli(&["unsupported-modes-summary"])
        .expect("unsupported modes summary should render");
    assert!(unsupported_modes_summary.contains("Unsupported modes summary"));
    assert!(unsupported_modes_summary.contains("Unsupported modes:"));
    assert_eq!(
        render_cli(&["unsupported-modes"]).expect("unsupported modes alias should render"),
        unsupported_modes_summary
    );

    let native_sidereal_policy = render_cli(&["native-sidereal-policy-summary"])
        .expect("native sidereal policy summary should render");
    assert!(native_sidereal_policy.contains("Native sidereal policy summary"));
    assert!(native_sidereal_policy.contains("Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert_eq!(
        render_cli(&["native-sidereal-policy"])
            .expect("native sidereal policy alias should render"),
        native_sidereal_policy
    );

    let zodiac_policy =
        render_cli(&["zodiac-policy-summary"]).expect("zodiac policy summary should render");
    assert!(zodiac_policy.contains("Zodiac policy summary"));
    assert!(zodiac_policy.contains("Zodiac policy: tropical only"));
    assert_eq!(
        render_cli(&["zodiac-policy"]).expect("zodiac policy alias should render"),
        zodiac_policy
    );

    let request_policy_summary_error = render_cli(&["request-policy-summary", "extra"])
        .expect_err("request policy summary should reject extra arguments");
    assert_eq!(
        request_policy_summary_error,
        "request-policy-summary does not accept extra arguments"
    );

    let request_policy_error = render_cli(&["request-policy", "extra"])
        .expect_err("request policy alias should reject extra arguments");
    assert_eq!(
        request_policy_error,
        "request-policy does not accept extra arguments"
    );

    let request_semantics_summary_error = render_cli(&["request-semantics-summary", "extra"])
        .expect_err("request semantics summary should reject extra arguments");
    assert_eq!(
        request_semantics_summary_error,
        "request-semantics-summary does not accept extra arguments"
    );

    let request_semantics_error = render_cli(&["request-semantics", "extra"])
        .expect_err("request semantics alias should reject extra arguments");
    assert_eq!(
        request_semantics_error,
        "request-semantics does not accept extra arguments"
    );

    let native_sidereal_policy_summary_error =
        render_cli(&["native-sidereal-policy-summary", "extra"])
            .expect_err("native sidereal policy summary should reject extra arguments");
    assert_eq!(
        native_sidereal_policy_summary_error,
        "native-sidereal-policy-summary does not accept extra arguments"
    );

    let zodiac_policy_summary_error = render_cli(&["zodiac-policy-summary", "extra"])
        .expect_err("zodiac policy summary should reject extra arguments");
    assert_eq!(
        zodiac_policy_summary_error,
        "zodiac-policy-summary does not accept extra arguments"
    );

    let zodiac_policy_error = render_cli(&["zodiac-policy", "extra"])
        .expect_err("zodiac policy alias should reject extra arguments");
    assert_eq!(
        zodiac_policy_error,
        "zodiac-policy does not accept extra arguments"
    );

    let native_sidereal_policy_error = render_cli(&["native-sidereal-policy", "extra"])
        .expect_err("native sidereal policy alias should reject extra arguments");
    assert_eq!(
        native_sidereal_policy_error,
        "native-sidereal-policy does not accept extra arguments"
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

#[test]
fn lunar_theory_request_policy_and_frame_treatment_commands_render_the_policy_blocks() {
    let request_policy = render_cli(&["lunar-theory-request-policy-summary"])
        .expect("lunar theory request policy summary should render");
    assert_eq!(request_policy, lunar_theory_request_policy_summary());
    assert_eq!(
        render_cli(&["lunar-theory-request-policy"])
            .expect("lunar theory request policy alias should render"),
        request_policy
    );
    assert_eq!(
        render_cli(&["lunar-theory-request-policy", "extra"])
            .expect_err("lunar theory request policy alias should reject extra arguments"),
        "lunar-theory-request-policy does not accept extra arguments"
    );

    let frame_treatment = render_cli(&["lunar-theory-frame-treatment-summary"])
        .expect("lunar theory frame treatment summary should render");
    assert_eq!(
        frame_treatment,
        lunar_theory_frame_treatment_summary_for_report()
    );
    assert_eq!(
        render_cli(&["lunar-theory-frame-treatment"])
            .expect("lunar theory frame treatment alias should render"),
        frame_treatment
    );
    assert_eq!(
        render_cli(&["lunar-theory-frame-treatment", "extra"])
            .expect_err("lunar theory frame treatment alias should reject extra arguments"),
        "lunar-theory-frame-treatment does not accept extra arguments"
    );

    let limitations = render_cli(&["lunar-theory-limitations-summary"])
        .expect("lunar theory limitations summary should render");
    assert_eq!(limitations, lunar_theory_limitations_summary_for_report());
    assert_eq!(
        render_cli(&["lunar-theory-limitations"]).unwrap(),
        lunar_theory_limitations_summary_for_report()
    );
    let limitations_error = render_cli(&["lunar-theory-limitations", "extra"])
        .expect_err("lunar theory limitations alias should reject extra arguments");
    assert_eq!(
        limitations_error,
        "lunar-theory-limitations-summary does not accept extra arguments"
    );
}

#[test]
fn comparison_and_reference_snapshot_summary_commands_render_the_overall_blocks() {
    let comparison = render_cli(&["comparison-snapshot-summary"])
        .expect("comparison snapshot summary should render");
    assert!(comparison.contains("Comparison snapshot summary"));
    assert!(comparison.contains("Comparison snapshot coverage:"));
    assert_eq!(
        comparison,
        format!(
            "Comparison snapshot summary\n{}\n",
            comparison_snapshot_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["j2000-snapshot"]).expect("j2000 snapshot alias should render"),
        comparison
    );
    assert_eq!(
        render_cli(&["comparison-snapshot"]).expect("comparison snapshot alias should render"),
        comparison
    );
    assert_eq!(
        render_cli(&["j2000-snapshot", "extra"])
            .expect_err("j2000 snapshot alias should reject extra arguments"),
        "j2000-snapshot does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-summary", "extra"])
            .expect_err("comparison snapshot summary should reject extra arguments"),
        "comparison-snapshot-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-snapshot", "extra"])
            .expect_err("comparison snapshot alias should reject extra arguments"),
        "comparison-snapshot does not accept extra arguments"
    );

    let reference = render_cli(&["reference-snapshot-summary"])
        .expect("reference snapshot summary should render");
    assert!(reference.contains("Reference snapshot summary"));
    assert!(reference.contains("Reference snapshot coverage:"));
    assert!(reference.contains("Reference 2500 major-body boundary evidence:"));
    assert!(reference
        .contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()));
    assert!(
        reference.contains(&reference_snapshot_2451916_major_body_boundary_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&reference_snapshot_bridge_day_summary_for_report()));
    assert!(
        reference.contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&reference_snapshot_2451915_major_body_bridge_summary_for_report()));
    assert!(reference.contains("Reference 2500 selected-body boundary evidence:"));
    assert!(reference.contains("Reference 2200 selected-body boundary evidence:"));
    assert!(reference.contains("Reference 2415020 selected-body boundary evidence:"));
    assert!(
        reference.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_dense_boundary_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_source_evidence_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_source_window_summary_for_report()));
    assert!(reference.contains("Reference 2453000 major-body boundary evidence:"));
    assert!(reference.contains("Reference 2600000 major-body boundary evidence:"));
    assert!(reference.contains("Reference 2400000 major-body boundary evidence:"));
    assert!(reference.contains("Reference 2451545 major-body boundary evidence:"));
    assert!(
        reference.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
    assert!(
        reference.contains(&reference_snapshot_2360234_major_body_interior_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report())
    );
    assert_eq!(
        reference,
        format!(
            "Reference snapshot summary\n{}\n",
            reference_snapshot_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["reference-snapshot"]).expect("reference snapshot alias should render"),
        reference
    );
    assert_eq!(
        render_cli(&["reference-snapshot-summary", "extra"])
            .expect_err("reference snapshot summary should reject extra arguments"),
        "reference-snapshot-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot", "extra"])
            .expect_err("reference snapshot alias should reject extra arguments"),
        "reference-snapshot does not accept extra arguments"
    );

    let reference_exact_j2000 = render_cli(&["reference-snapshot-exact-j2000-evidence-summary"])
        .expect("reference snapshot exact J2000 evidence should render");
    assert!(reference_exact_j2000.contains("Reference snapshot exact J2000 evidence summary"));
    assert!(reference_exact_j2000.contains(
        "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0"
    ));
    assert_eq!(
        reference_exact_j2000,
        format!(
            "Reference snapshot exact J2000 evidence summary\n{}\n",
            reference_snapshot_exact_j2000_evidence_summary_for_report()
        )
    );
    let exact_j2000_evidence =
        render_cli(&["exact-j2000-evidence"]).expect("exact J2000 evidence alias should render");
    assert_eq!(exact_j2000_evidence, reference_exact_j2000);
    let reference_exact_j2000_alias = render_cli(&["reference-snapshot-exact-j2000-evidence"])
        .expect("reference snapshot exact J2000 evidence alias should render");
    assert_eq!(reference_exact_j2000_alias, reference_exact_j2000);
    assert_eq!(
        render_cli(&["exact-j2000-evidence", "extra"])
            .expect_err("exact J2000 evidence alias should reject extra arguments"),
        "exact-j2000-evidence does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-exact-j2000-evidence", "extra"]).expect_err(
            "reference snapshot exact J2000 evidence alias should reject extra arguments"
        ),
        "reference-snapshot-exact-j2000-evidence does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-exact-j2000-evidence-summary", "extra"])
            .expect_err("reference snapshot exact J2000 evidence should reject extra arguments"),
        "reference-snapshot-exact-j2000-evidence-summary does not accept extra arguments"
    );
}

#[test]
fn comparison_and_benchmark_corpus_summary_commands_render_the_corpus_blocks() {
    assert_eq!(
        validated_comparison_corpus_release_guard_summary_for_report(),
        Ok("Pluto excluded from tolerance evidence")
    );

    let comparison = render_cli(&["comparison-corpus-summary"])
        .expect("comparison corpus summary should render");
    assert!(comparison.contains("Comparison corpus summary"));
    assert!(comparison.contains("name: JPL Horizons release-grade comparison window"));
    assert!(comparison.contains("release-grade guard: Pluto excluded from tolerance evidence"));
    assert_eq!(comparison, render_comparison_corpus_summary_text());

    let guard = render_cli(&["comparison-corpus-release-guard-summary"])
        .expect("comparison corpus release guard summary should render");
    assert!(guard.contains("Comparison corpus release-grade guard summary"));
    assert!(guard.contains("Release-grade guard: Pluto excluded from tolerance evidence"));
    assert_eq!(guard, render_comparison_corpus_release_guard_summary_text());
    assert_eq!(
        render_cli(&["comparison-corpus"]).expect("comparison corpus alias should render"),
        comparison
    );
    assert_eq!(
        render_cli(&["comparison-corpus"])
            .expect("comparison corpus alias should match canonical rendering"),
        render_comparison_corpus_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-corpus-release-guard"])
            .expect("comparison corpus release guard short alias should render"),
        guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard-summary"])
            .expect("comparison corpus guard alias should render"),
        guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard"])
            .expect("comparison corpus guard short alias should render"),
        guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-release-guard-summary", "extra"])
            .expect_err("comparison corpus release guard summary should reject extra arguments"),
        "comparison-corpus-release-guard-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus-release-guard", "extra"]).expect_err(
            "comparison corpus release guard short alias should reject extra arguments"
        ),
        "comparison-corpus-release-guard does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard-summary", "extra"])
            .expect_err("comparison corpus guard alias should reject extra arguments"),
        "comparison-corpus-guard-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard", "extra"])
            .expect_err("comparison corpus guard short alias should reject extra arguments"),
        "comparison-corpus-guard does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus", "extra"])
            .expect_err("comparison corpus alias should reject extra arguments"),
        "comparison-corpus-summary does not accept extra arguments"
    );

    let comparison_envelope = render_cli(&["comparison-envelope-summary"])
        .expect("comparison envelope summary should render");
    assert!(comparison_envelope.contains("Comparison envelope summary"));
    assert_eq!(
        comparison_envelope,
        render_comparison_envelope_summary_text()
    );

    let comparison_envelope_alias =
        render_cli(&["comparison-envelope"]).expect("comparison envelope alias should render");
    assert_eq!(comparison_envelope_alias, comparison_envelope);

    let release_body_claims_summary = render_cli(&["release-body-claims-summary"])
        .expect("release body claims summary should render");
    assert_eq!(
        release_body_claims_summary,
        render_release_body_claims_summary_text()
    );
    assert!(release_body_claims_summary.contains(
            "Release-grade body claims: Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies"
        ));
    assert_eq!(
        render_cli(&["body-claims-summary"]).expect("body claims alias should render"),
        release_body_claims_summary
    );
    assert_eq!(
        render_cli(&["body-claims-summary", "extra"])
            .expect_err("body claims alias should reject extra arguments"),
        "body-claims-summary does not accept extra arguments"
    );

    let body_date_channel_claims_summary = render_cli(&["body-date-channel-claims-summary"])
        .expect("body/date/channel claims summary should render");
    assert_eq!(
        body_date_channel_claims_summary,
        render_body_date_channel_claims_summary_text()
    );
    assert!(body_date_channel_claims_summary.contains("Body/date/channel claims: bodies="));
    assert!(body_date_channel_claims_summary
        .contains("frame policy=ecliptic body positions are the default request shape"));
    assert!(body_date_channel_claims_summary
        .contains("date range=JD 2268932.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(body_date_channel_claims_summary
        .contains("production generation coverage=Production generation coverage:"));
    assert!(body_date_channel_claims_summary.contains(
            "coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1500-2500 CE window; coverage="
        ));
    assert!(body_date_channel_claims_summary.contains("body-class coverage=major bodies:"));
    let source_corpus_summary =
        source_corpus_summary_details().expect("source corpus summary should exist");
    let body_date_channel_claims_details =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    assert_eq!(
        body_date_channel_claims_details.production_generation_date_range,
        source_corpus_summary.production_generation_date_range
    );
    assert_eq!(
        body_date_channel_claims_details.coverage_posture,
        source_corpus_summary.coverage_posture
    );
    assert_eq!(
        render_cli(&["body-date-channel-claims"])
            .expect("body/date/channel claims alias should render"),
        body_date_channel_claims_summary
    );
    assert_eq!(
        render_cli(&["body-date-channel-claims", "extra"])
            .expect_err("body/date/channel claims alias should reject extra arguments"),
        "body-date-channel-claims does not accept extra arguments"
    );
    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims
        .release_body_claims
        .push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("release body claims drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `release_body_claims` is out of sync with the current posture"
        );
    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims.frame_policy.push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("frame policy drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `frame_policy` is out of sync with the current posture"
        );

    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims
        .production_generation_date_range
        .push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("production generation date range drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `production_generation_date_range` is out of sync with the current posture"
        );

    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims
        .production_generation_coverage
        .push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("production generation coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `production_generation_coverage` is out of sync with the current posture"
        );

    let pluto_fallback_summary =
        render_cli(&["pluto-fallback-summary"]).expect("Pluto fallback summary should render");
    assert_eq!(pluto_fallback_summary, render_pluto_fallback_summary_text());
    assert!(pluto_fallback_summary.contains(
            "Release-grade body claims: Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies"
        ));
    assert!(pluto_fallback_summary.contains(
            "Pluto fallback policy: Pluto remains an explicitly approximate fallback; release-grade major-body claims exclude Pluto"
        ));
    let release_body_claims_line = release_body_claims_summary_for_report()
        .validated_summary_line()
        .expect("release body claims posture should validate");
    let pluto_fallback_line = pluto_fallback_summary_for_report()
        .validated_summary_line()
        .expect("Pluto fallback posture should validate");
    assert_eq!(
        validate_release_body_claims_posture(release_body_claims_line, pluto_fallback_line,),
        Ok(())
    );
    assert!(validate_release_body_claims_posture(
            &release_body_claims_line.replace("Pluto remains an explicitly approximate fallback; ", ""),
            "Pluto remains an explicitly approximate fallback; release-grade major-body claims include Pluto",
        )
        .is_err());
    assert_eq!(
        render_cli(&["pluto-fallback"]).expect("Pluto fallback alias should render"),
        pluto_fallback_summary
    );
    assert_eq!(
        render_cli(&["pluto-fallback", "extra"])
            .expect_err("Pluto fallback alias should reject extra arguments"),
        "pluto-fallback does not accept extra arguments"
    );

    let comparison_tolerance_policy_error = render_cli(&["comparison-tolerance-summary", "extra"])
        .expect_err("comparison tolerance alias should reject extra arguments");
    assert_eq!(
        comparison_tolerance_policy_error,
        "comparison-tolerance-summary does not accept extra arguments"
    );

    let comparison_envelope_error = render_cli(&["comparison-envelope", "extra"])
        .expect_err("comparison envelope alias should reject extra arguments");
    assert_eq!(
        comparison_envelope_error,
        "comparison-envelope does not accept extra arguments"
    );

    let benchmark_corpus = benchmark_corpus();
    let benchmark_summary = benchmark_corpus.summary();
    assert_eq!(benchmark_summary.epoch_count, 11);
    assert_eq!(benchmark_summary.earliest_julian_day, 2_268_559.0);
    assert_eq!(benchmark_summary.latest_julian_day, 2_634_532.0);
    assert!(benchmark_summary
        .epochs
        .iter()
        .any(|instant| instant.julian_day.days() == 2_451_545.0));
    assert!(benchmark_summary
        .epochs
        .iter()
        .any(|instant| instant.julian_day.days() == 2_634_167.0));

    let benchmark =
        render_cli(&["benchmark-corpus-summary"]).expect("benchmark corpus summary should render");
    assert!(benchmark.contains("Benchmark corpus summary"));
    assert!(benchmark.contains("name: Representative 1500-2500 window"));
    assert!(benchmark.contains("epoch labels: JD 2268559.0 (TT)"));
    assert!(benchmark.contains("JD 2451545.0 (TT)"));
    assert!(benchmark.contains("JD 2634532.0 (TT)"));
    assert_eq!(benchmark, render_benchmark_corpus_summary_text());
    assert_eq!(
        benchmark,
        validated_benchmark_corpus_summary_for_report()
            .expect("benchmark corpus summary should validate")
    );
    assert!(ensure_benchmark_corpus_summary_matches_current_rendering(&benchmark).is_ok());
    let chart_benchmark = render_cli(&["chart-benchmark-corpus-summary"])
        .expect("chart benchmark corpus summary should render");
    assert!(chart_benchmark.contains("Chart benchmark corpus summary"));
    assert!(chart_benchmark.contains("name: Representative chart validation scenarios"));
    assert!(chart_benchmark.contains("requests: 9"));
    assert!(chart_benchmark.contains("epochs: 1"));
    assert!(chart_benchmark.contains("epoch labels: JD 2451545.0 (TT)"));
    assert!(chart_benchmark.contains("bodies: 10"));
    assert_eq!(
        chart_benchmark,
        render_chart_benchmark_corpus_summary_text()
    );
    assert_eq!(
        chart_benchmark,
        validated_chart_benchmark_corpus_summary_for_report()
            .expect("chart benchmark corpus summary should validate")
    );
    assert!(
        ensure_chart_benchmark_corpus_summary_matches_current_rendering(&chart_benchmark).is_ok()
    );
    let drifted_chart_benchmark = chart_benchmark.replace(
        "Chart benchmark corpus summary",
        "Drifted chart benchmark corpus summary",
    );
    let error =
        ensure_chart_benchmark_corpus_summary_matches_current_rendering(&drifted_chart_benchmark)
            .expect_err("chart benchmark corpus summary drift should be rejected");
    assert!(error
        .to_string()
        .contains("chart benchmark corpus summary no longer matches"));
    let drifted_benchmark = benchmark.replace(
        "Benchmark corpus summary",
        "Drifted benchmark corpus summary",
    );
    let error = ensure_benchmark_corpus_summary_matches_current_rendering(&drifted_benchmark)
        .expect_err("benchmark corpus summary drift should be rejected");
    assert!(error
        .to_string()
        .contains("benchmark corpus summary no longer matches"));
}

#[test]
fn manifest_summary_commands_render_the_matching_blocks() {
    let comparison = render_cli(&["comparison-snapshot-manifest-summary"])
        .expect("comparison snapshot manifest summary should render");
    assert!(comparison.contains("Comparison snapshot manifest:"));
    assert_eq!(
        comparison,
        validated_comparison_snapshot_manifest_summary_for_report()
            .expect("comparison snapshot manifest summary should validate")
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-manifest"])
            .expect("comparison snapshot manifest alias should render"),
        comparison
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-manifest", "extra"])
            .expect_err("comparison snapshot manifest alias should reject extra arguments"),
        "comparison-snapshot-manifest does not accept extra arguments"
    );

    let reference = render_cli(&["reference-snapshot-manifest-summary"])
        .expect("reference snapshot manifest summary should render");
    assert!(reference.contains("Reference snapshot manifest:"));
    assert_eq!(reference, reference_snapshot_manifest_summary_for_report());
    assert_eq!(
        render_cli(&["reference-snapshot-manifest"])
            .expect("reference snapshot manifest alias should render"),
        reference
    );
    assert_eq!(
        render_cli(&["reference-snapshot-manifest", "extra"])
            .expect_err("reference snapshot manifest alias should reject extra arguments"),
        "reference-snapshot-manifest does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_batch_and_equatorial_parity_aliases_render_the_same_reports() {
    let batch = render_cli(&["reference-snapshot-batch-parity"])
        .expect("reference snapshot batch parity alias should render");
    assert_eq!(batch, reference_snapshot_batch_parity_summary_text());
    assert_eq!(
        render_cli(&["reference-snapshot-batch-parity", "extra"])
            .expect_err("reference snapshot batch parity alias should reject extra arguments"),
        "reference-snapshot-batch-parity-summary does not accept extra arguments"
    );

    let equatorial = render_cli(&["reference-snapshot-equatorial-parity"])
        .expect("reference snapshot equatorial parity alias should render");
    assert_eq!(
        equatorial,
        reference_snapshot_equatorial_parity_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-equatorial-parity", "extra"])
            .expect_err("reference snapshot equatorial parity alias should reject extra arguments"),
        "reference-snapshot-equatorial-parity-summary does not accept extra arguments"
    );
}

#[test]
fn comparison_snapshot_batch_parity_alias_renders_the_same_report() {
    let batch = render_cli(&["comparison-snapshot-batch-parity"])
        .expect("comparison snapshot batch parity alias should render");
    assert_eq!(batch, comparison_snapshot_batch_parity_summary_text());
    assert_eq!(
        render_cli(&["comparison-snapshot-batch-parity", "extra"])
            .expect_err("comparison snapshot batch parity alias should reject extra arguments"),
        "comparison-snapshot-batch-parity-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["reference-snapshot-source-window-summary"])
        .expect("reference snapshot source window summary should render");

    assert!(rendered.contains("Reference snapshot source windows:"));
    assert!(rendered.contains("source-backed samples"));
    assert_eq!(
        rendered,
        reference_snapshot_source_window_summary_for_report()
    );

    let alias = render_cli(&["reference-snapshot-source-window"])
        .expect("reference snapshot source window alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&["reference-snapshot-source-window", "extra"])
            .expect_err("reference snapshot source window alias should reject extra arguments"),
        "reference-snapshot-source-window does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_lunar_boundary_summary_command_renders_the_lunar_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-lunar-boundary-summary"])
        .expect("reference snapshot lunar boundary summary should render");

    assert!(rendered.contains("Reference lunar boundary evidence:"));
    assert_eq!(
        rendered,
        reference_snapshot_lunar_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1500_selected_body_boundary_summary_command_renders_the_early_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1500-selected-body-boundary-summary"])
        .expect("reference snapshot 1500 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1500 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2268932.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_1500_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1500-selected-body-boundary-summary"])
        .expect("1500 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1500_selected_body_boundary_summary_for_report()
    );

    let epoch_alias = render_cli(&["2268932-selected-body-boundary-summary"])
        .expect("2268932 selected-body boundary summary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_2268932_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        reference_snapshot_2268932_selected_body_boundary_summary_for_report(),
        reference_snapshot_1500_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2268932-selected-body-boundary-summary", "extra"]).expect_err(
            "2268932 selected-body boundary summary alias should reject extra arguments"
        ),
        "reference-snapshot-2268932-selected-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_1600_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1600-selected-body-boundary-summary"])
        .expect("reference snapshot 1600 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1600 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2305457.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune"));
    assert_eq!(
        rendered,
        reference_snapshot_1600_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1600-selected-body-boundary-summary"])
        .expect("1600 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1600_selected_body_boundary_summary_for_report()
    );

    let epoch_alias = render_cli(&["2305457-selected-body-boundary-summary"])
        .expect("2305457 selected-body boundary summary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_2305457_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        reference_snapshot_2305457_selected_body_boundary_summary_for_report(),
        reference_snapshot_1600_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2305457-selected-body-boundary-summary", "extra"]).expect_err(
            "2305457 selected-body boundary summary alias should reject extra arguments"
        ),
        "reference-snapshot-2305457-selected-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_1750_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1750-selected-body-boundary-summary"])
        .expect("reference snapshot 1750 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1750 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2360234.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    assert_eq!(
        rendered,
        reference_snapshot_1750_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1750-selected-body-boundary-summary"])
        .expect("1750 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1750_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1750_major_body_interior_summary_command_renders_the_interior_block() {
    let rendered = render_cli(&["reference-snapshot-1750-major-body-interior-summary"])
        .expect("reference snapshot 1750 major-body interior summary should render");

    assert!(rendered.contains("Reference 1750 major-body interior comparison evidence:"));
    assert!(rendered.contains("JD 2360234.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    assert_eq!(
        rendered,
        reference_snapshot_1750_major_body_interior_summary_for_report()
    );

    let alias = render_cli(&["1750-major-body-interior-summary"])
        .expect("1750 major-body interior summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1750_major_body_interior_summary_for_report()
    );
}

#[test]
fn reference_snapshot_2200_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2200-selected-body-boundary-summary"])
        .expect("reference snapshot 2200 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 2200 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2524593.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_2200_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["2200-selected-body-boundary-summary"])
        .expect("2200 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_2200_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1900_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1900-selected-body-boundary-summary"])
        .expect("reference snapshot 1900 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1900 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2415020.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_1900_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1900-selected-body-boundary-summary"])
        .expect("1900 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1900_selected_body_boundary_summary_for_report()
    );

    let epoch_alias = render_cli(&["2415020-selected-body-boundary-summary"])
        .expect("2415020 selected-body boundary summary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_2415020_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2415020-selected-body-boundary-summary",
            "extra"
        ])
        .expect_err("2415020 selected-body boundary summary should reject extra arguments"),
        "reference-snapshot-2415020-selected-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2415020-selected-body-boundary-summary", "extra"]).expect_err(
            "2415020 selected-body boundary summary alias should reject extra arguments"
        ),
        "reference-snapshot-2415020-selected-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2500_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2500-selected-body-boundary-summary"])
        .expect("reference snapshot 2500 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 2500 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert!(rendered.contains("Mars, Mercury, Moon, Sun, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_2500_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["2500-selected-body-boundary-summary"])
        .expect("2500 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_2500_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_2634167_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2634167-selected-body-boundary-summary"])
        .expect("reference snapshot 2634167 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 2634167 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert!(rendered.contains("Mars, Mercury, Moon, Sun, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_2634167_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["2634167-selected-body-boundary-summary"])
        .expect("2634167 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_2634167_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn lunar_boundary_summary_alias_command_renders_the_lunar_boundary_block() {
    let rendered = render_cli(&["lunar-boundary-summary"])
        .expect("lunar boundary summary alias should render");

    assert!(rendered.contains("Reference lunar boundary evidence:"));
    assert_eq!(
        rendered,
        reference_snapshot_lunar_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1749_major_body_boundary_summary_command_renders_the_1749_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-1749-major-body-boundary-summary"])
        .expect("reference snapshot 1749 major-body boundary summary should render");

    assert!(rendered.contains("Reference 1749 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2360233.5 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_1749_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["1749-major-body-boundary-summary"])
        .expect("1749 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    let epoch_alias = render_cli(&["2360233-major-body-boundary-summary"])
        .expect("2360233 major-body boundary alias should render");
    assert_eq!(epoch_alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-1749-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 1749 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-1749-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["1749-major-body-boundary-summary", "extra"])
            .expect_err("1749 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1749-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2360233-major-body-boundary-summary", "extra"])
            .expect_err("2360233 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1749-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_early_major_body_boundary_summary_command_renders_the_early_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-early-major-body-boundary-summary"])
        .expect("reference snapshot early major-body boundary summary should render");

    assert!(rendered.contains("Reference early major-body boundary evidence:"));
    assert_eq!(
        rendered,
        reference_snapshot_early_major_body_boundary_summary_for_report()
    );
    let exact_jd_alias = render_cli(&["2378498-major-body-boundary-summary"])
        .expect("2378498 major-body boundary alias should render");
    assert_eq!(
        exact_jd_alias,
        reference_snapshot_2378498_major_body_boundary_summary_for_report()
    );
    assert_eq!(
        exact_jd_alias,
        rendered.replace(
            "Reference early major-body boundary evidence",
            "Reference 2378498 major-body boundary evidence"
        )
    );
    let alias = render_cli(&["early-major-body-boundary-summary"])
        .expect("early major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&["2378498-major-body-boundary-summary", "extra"])
            .expect_err("2378498 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2378498-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-early-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot early major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-early-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["early-major-body-boundary-summary", "extra"])
            .expect_err("early major-body boundary alias should reject extra arguments"),
        "reference-snapshot-early-major-body-boundary-summary does not accept extra arguments"
    );

    let reference_snapshot_1800_major_body_boundary_summary =
        render_cli(&["reference-snapshot-1800-major-body-boundary-summary"])
            .expect("reference snapshot 1800 major-body boundary summary should render");

    assert!(reference_snapshot_1800_major_body_boundary_summary
        .contains("Reference 1800 major-body boundary evidence:"));
    assert_eq!(
        reference_snapshot_1800_major_body_boundary_summary,
        reference_snapshot_1800_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["1800-major-body-boundary-summary"])
        .expect("1800 major-body boundary alias should render");
    assert_eq!(alias, reference_snapshot_1800_major_body_boundary_summary);
    let epoch_alias = render_cli(&["2378499-major-body-boundary-summary"])
        .expect("2378499 major-body boundary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_1800_major_body_boundary_summary
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-1800-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 1800 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-1800-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["1800-major-body-boundary-summary", "extra"])
            .expect_err("1800 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1800-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2378499-major-body-boundary-summary", "extra"])
            .expect_err("2378499 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1800-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2500_major_body_boundary_summary_command_renders_the_terminal_boundary_block()
{
    let rendered = render_cli(&["reference-snapshot-2500-major-body-boundary-summary"])
        .expect("reference snapshot 2500 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2500 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2500000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2500_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2500-major-body-boundary-summary"])
        .expect("2500 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2500-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2500 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2500-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2500-major-body-boundary-summary", "extra"])
            .expect_err("2500 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2500-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2400000_major_body_boundary_summary_command_renders_the_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-2400000-major-body-boundary-summary"])
        .expect("reference snapshot 2400000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2400000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2400000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2400000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2400000-major-body-boundary-summary"])
        .expect("2400000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2400000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2400000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2400000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2400000-major-body-boundary-summary", "extra"])
            .expect_err("2400000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2400000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451545_major_body_boundary_summary_command_renders_the_j2000_block() {
    let rendered = render_cli(&["reference-snapshot-2451545-major-body-boundary-summary"])
        .expect("reference snapshot 2451545 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2451545 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2451545.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2451545_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2451545-major-body-boundary-summary"])
        .expect("2451545 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451545-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2451545 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2451545-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451545-major-body-boundary-summary", "extra"])
            .expect_err("2451545 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451545-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2453000_major_body_boundary_summary_command_renders_the_late_boundary_block()
{
    let rendered = render_cli(&["reference-snapshot-2453000-major-body-boundary-summary"])
        .expect("reference snapshot 2453000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2453000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2453000.5 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2453000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2453000-major-body-boundary-summary"])
        .expect("2453000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2453000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2453000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2453000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2453000-major-body-boundary-summary", "extra"])
            .expect_err("2453000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2453000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2500000_major_body_boundary_summary_command_renders_the_terminal_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2500000-major-body-boundary-summary"])
        .expect("reference snapshot 2500000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2500000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2500000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2500000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2500000-major-body-boundary-summary"])
        .expect("2500000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2500000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2500000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2500000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2500000-major-body-boundary-summary", "extra"])
            .expect_err("2500000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2500000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2600000_major_body_boundary_summary_command_renders_the_outer_boundary_block()
{
    let rendered = render_cli(&["reference-snapshot-2600000-major-body-boundary-summary"])
        .expect("reference snapshot 2600000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2600000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2600000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2600000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2600000-major-body-boundary-summary"])
        .expect("2600000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2600000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2600000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2600000-major-body-boundary-summary", "extra"])
            .expect_err("2600000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451910_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451910 = render_cli(&["reference-snapshot-2451910-major-body-boundary-summary"])
        .expect("2451910 major-body boundary summary should render");
    assert!(boundary_2451910.contains("Reference 2451910 major-body boundary evidence:"));
    assert!(boundary_2451910.contains("JD 2451910.5 (TDB)"));
    let boundary_2451910_alias = render_cli(&["2451910-major-body-boundary-summary"])
        .expect("2451910 major-body boundary alias should render");
    assert_eq!(boundary_2451910_alias, boundary_2451910);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451910-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451910 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451910-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451910-major-body-boundary-summary", "extra"])
            .expect_err("2451910 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451910-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451911_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451911 = render_cli(&["reference-snapshot-2451911-major-body-boundary-summary"])
        .expect("2451911 major-body boundary summary should render");
    assert!(boundary_2451911.contains("Reference 2451911 major-body boundary evidence:"));
    assert!(boundary_2451911.contains("JD 2451911.5 (TDB)"));
    let boundary_2451911_alias = render_cli(&["2451911-major-body-boundary-summary"])
        .expect("2451911 major-body boundary alias should render");
    assert_eq!(boundary_2451911_alias, boundary_2451911);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451911-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451911 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451911-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451911-major-body-boundary-summary", "extra"])
            .expect_err("2451911 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451911-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451912_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451912 = render_cli(&["reference-snapshot-2451912-major-body-boundary-summary"])
        .expect("2451912 major-body boundary summary should render");
    assert!(boundary_2451912.contains("Reference 2451912 major-body boundary evidence:"));
    assert!(boundary_2451912.contains("JD 2451912.5 (TDB)"));
    let boundary_2451912_alias = render_cli(&["2451912-major-body-boundary-summary"])
        .expect("2451912 major-body boundary alias should render");
    assert_eq!(boundary_2451912_alias, boundary_2451912);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451912-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451912 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451912-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451912-major-body-boundary-summary", "extra"])
            .expect_err("2451912 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451912-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451913_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451913 = render_cli(&["reference-snapshot-2451913-major-body-boundary-summary"])
        .expect("2451913 major-body boundary summary should render");
    assert!(boundary_2451913.contains("Reference 2451913 major-body boundary evidence:"));
    assert!(boundary_2451913.contains("JD 2451913.5 (TDB)"));
    let boundary_2451913_alias = render_cli(&["2451913-major-body-boundary-summary"])
        .expect("2451913 major-body boundary alias should render");
    assert_eq!(boundary_2451913_alias, boundary_2451913);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451913-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451913 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451913-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451913-major-body-boundary-summary", "extra"])
            .expect_err("2451913 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451913-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451914_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451914 = render_cli(&["reference-snapshot-2451914-major-body-boundary-summary"])
        .expect("2451914 major-body boundary summary should render");
    assert!(boundary_2451914.contains("Reference 2451914 major-body boundary evidence:"));
    assert!(boundary_2451914.contains("JD 2451914.5 (TDB)"));
    let boundary_2451914_alias = render_cli(&["2451914-major-body-boundary-summary"])
        .expect("2451914 major-body boundary alias should render");
    assert_eq!(boundary_2451914_alias, boundary_2451914);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451914-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451914 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451914-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451914-major-body-boundary-summary", "extra"])
            .expect_err("2451914 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451915_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451915 = render_cli(&["reference-snapshot-2451915-major-body-boundary-summary"])
        .expect("2451915 major-body boundary summary should render");
    assert!(boundary_2451915.contains("Reference 2451915 major-body boundary evidence:"));
    assert!(boundary_2451915.contains("JD 2451915.5 (TDB)"));
    let boundary_2451915_alias = render_cli(&["2451915-major-body-boundary-summary"])
        .expect("2451915 major-body boundary alias should render");
    assert_eq!(boundary_2451915_alias, boundary_2451915);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451915-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451915 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451915-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451915-major-body-boundary-summary", "extra"])
            .expect_err("2451915 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451915-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451916_major_body_interior_summary_aliases_render_the_same_reports() {
    let interior_2451916 = render_cli(&["reference-snapshot-2451916-major-body-interior-summary"])
        .expect("2451916 major-body interior summary should render");
    assert!(interior_2451916.contains("Reference 2451916 major-body interior evidence:"));
    assert!(interior_2451916.contains("JD 2451916.0 (TDB)"));
    let interior_2451916_alias = render_cli(&["2451916-major-body-interior-summary"])
        .expect("2451916 major-body interior alias should render");
    assert_eq!(interior_2451916_alias, interior_2451916);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451916-major-body-interior-summary",
            "extra"
        ])
        .expect_err("2451916 major-body interior summary should reject extra arguments"),
        "reference-snapshot-2451916-major-body-interior-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451916-major-body-interior-summary", "extra"])
            .expect_err("2451916 major-body interior alias should reject extra arguments"),
        "reference-snapshot-2451916-major-body-interior-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451917_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451917 = render_cli(&["reference-snapshot-2451917-major-body-boundary-summary"])
        .expect("2451917 major-body boundary summary should render");
    assert!(boundary_2451917.contains("Reference 2451917 major-body boundary evidence:"));
    assert!(boundary_2451917.contains("JD 2451917.5 (TDB)"));
    let boundary_2451917_alias = render_cli(&["2451917-major-body-boundary-summary"])
        .expect("2451917 major-body boundary alias should render");
    assert_eq!(boundary_2451917_alias, boundary_2451917);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451917-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451917 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451917-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451917-major-body-boundary-summary", "extra"])
            .expect_err("2451917 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451917-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451918_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451918 = render_cli(&["reference-snapshot-2451918-major-body-boundary-summary"])
        .expect("2451918 major-body boundary summary should render");
    assert!(boundary_2451918.contains("Reference 2451918 major-body boundary evidence:"));
    assert!(boundary_2451918.contains("JD 2451918.5 (TDB)"));
    let boundary_2451918_alias = render_cli(&["2451918-major-body-boundary-summary"])
        .expect("2451918 major-body boundary alias should render");
    assert_eq!(boundary_2451918_alias, boundary_2451918);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451918-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451918 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451918-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451918-major-body-boundary-summary", "extra"])
            .expect_err("2451918 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451918-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451919_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451919 = render_cli(&["reference-snapshot-2451919-major-body-boundary-summary"])
        .expect("2451919 major-body boundary summary should render");
    assert!(boundary_2451919.contains("Reference 2451919 major-body boundary evidence:"));
    assert!(boundary_2451919.contains("JD 2451919.5 (TDB)"));
    let boundary_2451919_alias = render_cli(&["2451919-major-body-boundary-summary"])
        .expect("2451919 major-body boundary alias should render");
    assert_eq!(boundary_2451919_alias, boundary_2451919);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451919-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451919 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451919-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451919-major-body-boundary-summary", "extra"])
            .expect_err("2451919 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451919-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451920_major_body_interior_summary_aliases_render_the_same_reports() {
    let interior_2451920 = render_cli(&["reference-snapshot-2451920-major-body-interior-summary"])
        .expect("2451920 major-body interior summary should render");
    assert!(interior_2451920.contains("Reference 2451920 major-body interior evidence:"));
    assert!(interior_2451920.contains("JD 2451920.5 (TDB)"));
    let interior_2451920_alias = render_cli(&["2451920-major-body-interior-summary"])
        .expect("2451920 major-body interior alias should render");
    assert_eq!(interior_2451920_alias, interior_2451920);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451920-major-body-interior-summary",
            "extra"
        ])
        .expect_err("2451920 major-body interior summary should reject extra arguments"),
        "reference-snapshot-2451920-major-body-interior-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451920-major-body-interior-summary", "extra"])
            .expect_err("2451920 major-body interior alias should reject extra arguments"),
        "reference-snapshot-2451920-major-body-interior-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2360234_major_body_interior_summary_aliases_render_the_same_reports() {
    let interior_2360234 = render_cli(&["reference-snapshot-2360234-major-body-interior-summary"])
        .expect("2360234 major-body interior comparison summary should render");
    assert!(interior_2360234.contains("Reference 2360234 major-body interior comparison evidence:"));
    assert!(interior_2360234.contains("JD 2360234.5 (TDB)"));
    let interior_2360234_alias = render_cli(&["2360234-major-body-interior-summary"])
        .expect("2360234 major-body interior comparison alias should render");
    assert_eq!(interior_2360234_alias, interior_2360234);
    assert_eq!(
            render_cli(&["reference-snapshot-2360234-major-body-interior-summary", "extra"])
                .expect_err(
                    "2360234 major-body interior comparison summary should reject extra arguments",
                ),
            "reference-snapshot-2360234-major-body-interior-summary does not accept extra arguments"
        );
    assert_eq!(
        render_cli(&["2360234-major-body-interior-summary", "extra"]).expect_err(
            "2360234 major-body interior comparison alias should reject extra arguments",
        ),
        "reference-snapshot-2360234-major-body-interior-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_mars_outer_boundary_summary_command_renders_the_outer_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-mars-outer-boundary-summary"])
        .expect("reference snapshot Mars outer-boundary summary should render");

    assert!(rendered.contains("Reference Mars outer-boundary evidence:"));
    assert!(rendered.contains("JD 2600000.0 (TDB)"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_mars_outer_boundary_summary_for_report()
    );
    let alias = render_cli(&["mars-outer-boundary-summary"])
        .expect("Mars outer-boundary alias should render");
    assert_eq!(alias, rendered);
}

#[test]
fn reference_snapshot_2600000_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2600000 = render_cli(&["reference-snapshot-2600000-major-body-boundary-summary"])
        .expect("2600000 major-body boundary summary should render");
    assert!(boundary_2600000.contains("Reference 2600000 major-body boundary evidence:"));
    assert!(boundary_2600000.contains("JD 2600000.0 (TDB)"));
    let boundary_2600000_alias = render_cli(&["2600000-major-body-boundary-summary"])
        .expect("2600000 major-body boundary alias should render");
    assert_eq!(boundary_2600000_alias, boundary_2600000);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2600000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2600000 major-body boundary summary should reject extra arguments",),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2600000-major-body-boundary-summary", "extra"])
            .expect_err("2600000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn source_corpus_summary_aliases_render_the_same_report() {
    let rendered =
        render_cli(&["source-corpus-summary"]).expect("source corpus summary should render");
    assert_eq!(rendered, source_corpus_summary_for_report());
    assert!(rendered.contains("shared schema=epoch_jd, body, x_km, y_km, z_km"));
    assert!(rendered.contains("generation command=generate-packaged-artifact --check"));
    assert!(
        rendered.contains("production generation source=strategy=documented hybrid fixture corpus")
    );
    assert!(rendered.contains("license posture=public-source provenance only; checked-in fixtures remain repository-local regression data"));
    assert!(rendered.contains(
        "redistribution posture=repository-checked regression fixtures, not a broad public corpus"
    ));
    assert!(rendered.contains("schema=epoch_jd, body, x_km, y_km, z_km"));
    assert!(rendered.contains("production generation source revision=source revision="));
    assert!(rendered.contains("reference_snapshot.csv checksum=0x"));
    assert!(rendered.contains("independent_holdout_snapshot.csv checksum=0x"));
    assert!(rendered.contains("production generation source windows=357 source-backed samples across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB))"));
    assert!(rendered.contains("source density floors=reference major bodies:"));
    assert!(rendered.contains("production generation body-class coverage=major bodies: 262 rows across 10 bodies and 31 epochs"));
    assert!(rendered
        .contains("production generation date range=JD 2268932.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains("production generation quarter-day boundary samples=8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB))"));
    assert!(rendered.contains("coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1500-2500 CE window; coverage="));
    assert!(rendered.contains("body-class coverage=major bodies:"));
    assert!(rendered.contains("production generation boundary window="));
    assert!(rendered.contains("production generation boundary source="));
    assert!(rendered.contains("production generation boundary request corpus="));
    assert!(rendered.contains("production generation boundary request corpus equatorial="));
    assert!(rendered
        .contains("reference snapshot sparse boundary=16 exact samples at JD 2451915.5 (TDB)"));
    assert!(rendered.contains(
        "reference snapshot exact J2000 evidence=16 exact J2000 samples at JD 2451545.0 (TDB)"
    ));
    assert!(rendered.contains("reference snapshot equatorial parity=357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB))"));
    assert!(rendered.contains("reference snapshot body-class coverage=major bodies: 262 rows across 10 bodies and 31 epochs"));
    let reference_snapshot_manifest = required_summary_payload(
        reference_snapshot_manifest_summary_for_report(),
        "Reference snapshot manifest: ",
        "reference snapshot manifest",
    )
    .expect("reference snapshot manifest payload should be available");
    assert!(rendered.contains(&format!(
        "reference snapshot manifest={reference_snapshot_manifest}"
    )));
    assert!(rendered.contains(
        "independent-holdout body-class coverage=84 rows across 16 bodies and 14 epochs"
    ));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"));
    assert!(rendered.contains("evidence classification=release-tolerance=reference/comparison/production-generation validation summaries; hold-out=independent hold-out rows and interpolation-quality summaries; fixture exactness=reference snapshot exact J2000 evidence; provenance-only=source and manifest summaries"));
    assert!(rendered.contains("provenance-only=source and manifest summaries are provenance-only evidence; they validate corpus provenance and checksum posture but are excluded from tolerance, hold-out, and fixture-exactness claims"));
    assert!(rendered
        .contains("lunar source windows=7 exact Moon samples across 1 bodies in 2 exact windows"));
    assert!(rendered.contains("release-grade body claims=Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies"));
    assert!(rendered.contains("date range=JD 2268932.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains("production generation coverage=Production generation coverage:"));
    assert!(rendered.contains(
            "coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1500-2500 CE window; coverage="
        ));
    assert!(rendered.contains("body-class coverage=major bodies:"));
    assert!(rendered.contains(
            "equatorial output is backend-specific and derived via mean-obliquity transforms when supported"
        ));
    assert!(rendered.contains("corpus shape=Production generation corpus shape:"));
    assert!(rendered
        .contains("provenance-only=source and manifest summaries are provenance-only evidence"));
    assert_eq!(
        render_cli(&["jpl-provenance-only-summary"])
            .expect("JPL provenance-only summary should render"),
        pleiades_jpl::jpl_provenance_only_summary_for_report()
    );
    assert_eq!(
        render_cli(&["jpl-provenance-only"]).expect("JPL provenance-only alias should render"),
        pleiades_jpl::jpl_provenance_only_summary_for_report()
    );
    assert_eq!(
        render_cli(&["jpl-provenance-only-summary", "extra"])
            .expect_err("JPL provenance-only summary should reject extra arguments"),
        "jpl-provenance-only-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["jpl-provenance-only", "extra"])
            .expect_err("JPL provenance-only alias should reject extra arguments"),
        "jpl-provenance-only does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-corpus"]).expect("source corpus alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["source-corpus", "extra"])
            .expect_err("source corpus alias should reject extra arguments"),
        "source-corpus does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-corpus-posture-summary"])
            .expect("source corpus posture summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["source-corpus-posture"]).expect("source corpus posture alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["source-corpus-posture-summary", "extra"])
            .expect_err("source corpus posture summary should reject extra arguments"),
        "source-corpus-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-corpus-posture", "extra"])
            .expect_err("source corpus posture alias should reject extra arguments"),
        "source-corpus-posture does not accept extra arguments"
    );
}

#[test]
fn jpl_source_posture_summary_aliases_render_the_same_report() {
    let rendered = render_cli(&["jpl-source-posture-summary"])
        .expect("JPL source posture summary should render");
    assert_eq!(
        rendered,
        pleiades_jpl::jpl_source_posture_summary_for_report()
    );
    assert_eq!(
        render_cli(&["jpl-source-posture"]).expect("JPL source posture alias should render"),
        pleiades_jpl::jpl_source_posture_summary_for_report()
    );
    assert_eq!(
        render_cli(&["jpl-source-posture-summary", "extra"])
            .expect_err("JPL source posture summary should reject extra arguments"),
        "jpl-source-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["jpl-source-posture", "extra"])
            .expect_err("JPL source posture alias should reject extra arguments"),
        "jpl-source-posture-summary does not accept extra arguments"
    );
}

#[test]
fn required_summary_payload_rejects_missing_prefixes() {
    let error = required_summary_payload(
        "drifted payload".to_string(),
        "expected prefix: ",
        "example field",
    )
    .expect_err("missing prefix should fail closed");
    assert_eq!(
        error,
        "source corpus summary field `example field` is out of sync with the current posture"
    );
}

#[test]
fn required_labelled_summary_payload_rejects_duplicate_prefixes() {
    let error = required_labelled_summary_payload(
        "JPL source corpus contract: JPL source corpus contract: nested drift".to_string(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    )
    .expect_err("duplicate labelled prefixes should fail closed");
    assert_eq!(
            error,
            "source corpus summary field `JPL source corpus contract` is out of sync with the current posture"
        );
}

#[test]
fn source_corpus_summary_details_validate_field_drift() {
    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.shared_schema = "epoch_jd, body, x_km, y_km, z_km, drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("shared schema drift should fail closed");
    assert_eq!(
        error.to_string(),
        "the source corpus summary field `shared_schema` is out of sync with the current posture"
    );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.generation_command = "generate-packaged-artifact --check --drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("generation command drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `generation_command` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.comparison_corpus_release_grade_guard =
        "Comparison corpus release-grade guard: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("comparison corpus release-grade guard drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `comparison_corpus_release_grade_guard` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.jpl_source_corpus_contract = "JPL source corpus contract: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("jpl source corpus contract drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `jpl_source_corpus_contract` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.jpl_evidence_classification = "JPL evidence classification: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("jpl evidence classification drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `jpl_evidence_classification` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.jpl_provenance_only = "JPL provenance-only evidence: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("jpl provenance-only drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `jpl_provenance_only` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.lunar_source_window = "lunar source windows=drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("lunar source window drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `lunar_source_window` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_source = "Production generation source: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation source drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_source` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_source_revision = "source revision=drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation source revision drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_source_revision` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_coverage = "Production generation coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_source_windows =
        "Production generation source windows: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation source windows drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_source_windows` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_body_class_coverage =
        "Production generation body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_window =
        "Production generation boundary windows: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation boundary window drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_window` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_source =
        "Production generation boundary overlay source: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation boundary source drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_source` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_request_corpus =
        "Production generation boundary request corpus: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation boundary request corpus drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_request_corpus` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_request_corpus_equatorial =
        "Production generation boundary request corpus: drifted".to_string();

    let error = summary.validated_summary_line().expect_err(
        "production generation boundary request corpus equatorial drift should fail closed",
    );
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_request_corpus_equatorial` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_date_range = "JD drifted..JD drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation date range drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_date_range` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_quarter_day_boundary_samples =
        "8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB))"
            .to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation quarter-day boundary samples drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_quarter_day_boundary_samples` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_sparse_boundary =
        "Reference snapshot boundary day: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot sparse boundary drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_sparse_boundary` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_exact_j2000_evidence =
        "Reference snapshot exact J2000 evidence: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot exact J2000 evidence drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_exact_j2000_evidence` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_exact_j2000_body_class_coverage =
        "Reference snapshot exact J2000 body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot exact J2000 body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_exact_j2000_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_equatorial_parity =
        "JPL reference snapshot equatorial parity: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot equatorial parity drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_equatorial_parity` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_body_class_coverage =
        "Reference snapshot body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_manifest = "Reference snapshot manifest: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot manifest drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_manifest` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.independent_holdout_body_class_coverage =
        "Independent hold-out body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("independent hold-out body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `independent_holdout_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.independent_holdout_source_window =
        "Independent hold-out source windows: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("independent hold-out source window drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `independent_holdout_source_window` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.pluto_fallback = "Pluto fallback: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("pluto fallback drift should fail closed");
    assert_eq!(
        error.to_string(),
        "the source corpus summary field `pluto_fallback` is out of sync with the current posture"
    );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.release_grade_body_claims = "Release-grade body claims: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("release-grade body claims drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `release_grade_body_claims` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.body_date_channel_claims = "Body/date/channel claims: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("body/date/channel claims drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `body_date_channel_claims` is out of sync with the current posture"
        );

    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims.coverage_posture = "coverage posture=drifted".to_string();

    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("body/date/channel coverage posture drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `coverage_posture` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.coverage_posture = "coverage posture=drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("coverage posture drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `coverage_posture` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.phase2_corpus_alignment = "Phase-2 corpus alignment: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("phase-2 corpus alignment drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `phase2_corpus_alignment` is out of sync with the current posture"
        );
}
