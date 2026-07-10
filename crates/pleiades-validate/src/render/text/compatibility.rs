//! Compatibility profile/caveats text, packaged-artifact fit posture, and the validation report spine.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::*;

pub(crate) fn validate_compatibility_profile_summary_text(
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
        profile.unsupported_modes_summary_line()
    );
    if !text.contains(&expected_unsupported_modes_line) {
        return Err(format!(
            "compatibility profile summary unsupported-modes mismatch: expected `{expected_unsupported_modes_line}`"
        ));
    }

    Ok(())
}

pub(crate) fn render_compatibility_profile_summary_text() -> String {
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
    text.push_str(profile.unsupported_modes_summary_line());
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

pub(crate) fn render_compatibility_caveats_summary_text() -> String {
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

pub(crate) fn validate_packaged_artifact_fit_posture() -> Result<(), EphemerisError> {
    let fit_envelope = packaged_artifact_fit_envelope_summary_details();
    let thresholds = packaged_artifact_fit_threshold_summary_details();
    let target_threshold = packaged_artifact_target_threshold_summary_details();
    validate_packaged_artifact_fit_posture_with(&fit_envelope, &thresholds, &target_threshold)
}

pub(crate) fn validate_packaged_artifact_fit_posture_with(
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

pub(crate) fn build_validation_report(rounds: usize) -> Result<ValidationReport, EphemerisError> {
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

pub(crate) fn build_validation_report_uncached(
    rounds: usize,
) -> Result<ValidationReport, EphemerisError> {
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

pub(crate) fn render_validation_report_uncached(rounds: usize) -> Result<String, EphemerisError> {
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

pub(crate) fn render_validation_report_summary_uncached(
    rounds: usize,
) -> Result<String, EphemerisError> {
    let report = build_validation_report(rounds)?;
    Ok(render_validation_report_summary_text(&report))
}

pub(crate) fn validated_packaged_artifact_fit_sample_classes_summary_for_report(
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
