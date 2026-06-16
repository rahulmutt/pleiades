//! Top-level CLI dispatch.

use pleiades_jpl::{
    comparison_snapshot_source_summary_for_report, reference_snapshot_source_summary_for_report,
    selected_asteroid_source_2451917_summary_for_report,
    selected_asteroid_source_2453000_summary_for_report,
    selected_asteroid_source_2500000_summary_for_report,
    selected_asteroid_source_2634167_summary_for_report,
};
use pleiades_validate::{
    render_benchmark_report, render_cli as validate_render_cli, render_release_bundle,
    render_validation_report_summary, verify_compatibility_profile,
};

use crate::commands::chart::render_chart;
use crate::commands::fixture_golden::render_fixture_golden;
use crate::commands::packaged_artifact::{
    parse_packaged_artifact_command, render_packaged_artifact_regeneration,
    render_packaged_artifact_regeneration_check, PackagedArtifactCommand,
};
use crate::commands::spk_corpus::render_spk_corpus;
use crate::help::help_text;
use crate::parse::{parse_release_bundle_output_dir, parse_rounds};
use crate::render::render_error;

pub(crate) fn banner() -> &'static str {
    "pleiades-cli chart utility"
}

pub(crate) fn ensure_no_extra_args(args: &[&str], command: &str) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("{command} does not accept extra arguments"))
    }
}

pub(crate) fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compare-backends") => {
            ensure_no_extra_args(&args[1..], "compare-backends")?;
            validate_render_cli(args)
        }
        Some("comparison-report") => {
            ensure_no_extra_args(&args[1..], "comparison-report")?;
            validate_render_cli(args)
        }
        Some("compare-backends-audit") => {
            ensure_no_extra_args(&args[1..], "compare-backends-audit")?;
            validate_render_cli(args)
        }
        Some("comparison-audit-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-audit-summary")?;
            validate_render_cli(args)
        }
        Some("comparison-audit") => {
            ensure_no_extra_args(&args[1..], "comparison-audit")?;
            validate_render_cli(args)
        }
        Some("benchmark") => {
            let rounds = parse_rounds(&args[1..], 10_000)?;
            render_benchmark_report(rounds).map_err(render_error)
        }
        Some("comparison-corpus-summary") | Some("comparison-corpus") => validate_render_cli(args),
        Some("comparison-corpus-release-guard-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-release-guard-summary")?;
            validate_render_cli(args)
        }
        Some("comparison-corpus-release-guard") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-release-guard")?;
            validate_render_cli(args)
        }
        Some("comparison-corpus-guard-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-guard-summary")?;
            validate_render_cli(args)
        }
        Some("comparison-corpus-guard") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-guard")?;
            validate_render_cli(args)
        }
        Some("benchmark-corpus-summary") => validate_render_cli(args),
        Some("chart-benchmark-corpus-summary") | Some("chart-benchmark-corpus") => {
            validate_render_cli(args)
        }
        Some("compatibility-profile") | Some("profile") => validate_render_cli(args),
        Some("compatibility-profile-summary") | Some("profile-summary") => {
            validate_render_cli(args)
        }
        Some("compatibility-caveats-summary") => {
            ensure_no_extra_args(&args[1..], "compatibility-caveats-summary")?;
            validate_render_cli(args)
        }
        Some("compatibility-caveats") => {
            ensure_no_extra_args(&args[1..], "compatibility-caveats")?;
            validate_render_cli(args)
        }
        Some("catalog-inventory-summary") => validate_render_cli(args),
        Some("catalog-inventory") => validate_render_cli(args),
        Some("custom-definition-ayanamsa-labels-summary")
        | Some("custom-definition-ayanamsa-labels") => validate_render_cli(args),
        Some("release-house-system-canonical-names-summary")
        | Some("release-house-system-canonical-names") => validate_render_cli(args),
        Some("release-ayanamsa-canonical-names-summary")
        | Some("release-ayanamsa-canonical-names") => validate_render_cli(args),
        Some("release-house-validation-summary") | Some("release-house-validation") => {
            validate_render_cli(args)
        }
        Some("verify-compatibility-profile") => {
            verify_compatibility_profile().map_err(render_error)
        }
        Some("bundle-release") => {
            if args[1..].iter().any(|arg| *arg == "--help" || *arg == "-h") {
                return Ok(help_text());
            }
            let output_dir = parse_release_bundle_output_dir(&args[1..])?;
            render_release_bundle(1, output_dir)
                .map(|bundle| bundle.to_string())
                .map_err(|error| error.to_string())
        }
        Some("verify-release-bundle") => {
            if args[1..].iter().any(|arg| *arg == "--help" || *arg == "-h") {
                return Ok(help_text());
            }
            let output_dir = parse_release_bundle_output_dir(&args[1..])?;
            validate_render_cli(&["verify-release-bundle", "--out", output_dir])
        }
        Some("api-stability") | Some("api-posture") => validate_render_cli(args),
        Some("api-stability-summary") | Some("api-posture-summary") => validate_render_cli(args),
        Some("backend-matrix") | Some("capability-matrix") => validate_render_cli(args),
        Some("backend-matrix-summary") | Some("matrix-summary") => validate_render_cli(args),
        Some("benchmark-matrix-summary") | Some("benchmark-matrix") => validate_render_cli(args),
        Some("release-notes") => validate_render_cli(args),
        Some("release-notes-summary") => validate_render_cli(args),
        Some("release-checklist") => validate_render_cli(args),
        Some("release-smoke") => validate_render_cli(args),
        Some("release-gate") => validate_render_cli(args),
        Some("release-checklist-summary") | Some("checklist-summary") => validate_render_cli(args),
        Some("release-gate-summary") => validate_render_cli(args),
        Some("release-summary") => validate_render_cli(args),
        Some("source-corpus-summary") => validate_render_cli(args),
        Some("source-corpus") => validate_render_cli(args),
        Some("source-corpus-posture-summary") | Some("source-corpus-posture") => {
            validate_render_cli(args)
        }
        Some("jpl-batch-error-taxonomy-summary") => validate_render_cli(args),
        Some("jpl-snapshot-evidence-summary") => validate_render_cli(args),
        Some("jpl-source-corpus-contract-summary") | Some("jpl-source-corpus-contract") => {
            validate_render_cli(args)
        }
        Some("jpl-source-posture-summary") | Some("jpl-source-posture") => {
            validate_render_cli(args)
        }
        Some("jpl-provenance-only-summary") | Some("jpl-provenance-only") => {
            validate_render_cli(args)
        }
        Some("production-generation-boundary-summary") | Some("production-generation-boundary") => {
            validate_render_cli(args)
        }
        Some("production-generation-boundary-request-corpus-summary")
        | Some("production-generation-boundary-request-corpus") => validate_render_cli(args),
        Some("production-generation-boundary-request-corpus-equatorial-summary")
        | Some("production-generation-boundary-request-corpus-equatorial") => {
            validate_render_cli(args)
        }
        Some("production-generation-body-class-coverage-summary")
        | Some("production-body-class-coverage-summary") => validate_render_cli(args),
        Some("production-generation-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-source-window-summary")?;
            Ok(pleiades_jpl::production_generation_snapshot_window_summary_for_report())
        }
        Some("production-generation-source-window") => {
            ensure_no_extra_args(&args[1..], "production-generation-source-window")?;
            Ok(pleiades_jpl::production_generation_snapshot_window_summary_for_report())
        }
        Some("production-generation") | Some("production-generation-summary") => {
            validate_render_cli(args)
        }
        Some("production-generation-quarter-day-boundary-summary")
        | Some("production-generation-quarter-day-boundary") => validate_render_cli(args),
        Some("production-generation-boundary-source-summary")
        | Some("production-generation-boundary-source") => validate_render_cli(args),
        Some("production-generation-boundary-window-summary") => validate_render_cli(args),
        Some("production-generation-boundary-window") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-window")?;
            validate_render_cli(args)
        }
        Some("production-generation-corpus-shape-summary")
        | Some("production-generation-corpus-shape") => validate_render_cli(args),
        Some("production-generation-source-summary") => validate_render_cli(args),
        Some("production-generation-source-revision-summary") => validate_render_cli(args),
        Some("production-generation-source-revision") => validate_render_cli(args),
        Some("production-generation-manifest-summary") | Some("production-generation-manifest") => {
            validate_render_cli(args)
        }
        Some("production-generation-manifest-checksum-summary")
        | Some("production-generation-manifest-checksum") => validate_render_cli(args),
        Some("production-generation-source") => {
            ensure_no_extra_args(&args[1..], "production-generation-source")?;
            validate_render_cli(args)
        }
        Some("comparison-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source-summary")?;
            Ok(comparison_snapshot_source_summary_for_report())
        }
        Some("comparison-snapshot-source") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source")?;
            Ok(comparison_snapshot_source_summary_for_report())
        }
        Some("comparison-snapshot-source-window-summary") => validate_render_cli(args),
        Some("comparison-snapshot-source-window") => validate_render_cli(args),
        Some("comparison-snapshot-body-class-coverage-summary")
        | Some("comparison-body-class-coverage-summary") => validate_render_cli(args),
        Some("comparison-snapshot-manifest-summary") | Some("comparison-snapshot-manifest") => {
            validate_render_cli(args)
        }
        Some("comparison-snapshot") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot")?;
            Ok(format!(
                "Comparison snapshot summary\n{}\n",
                pleiades_jpl::comparison_snapshot_summary_for_report()
            ))
        }
        Some("j2000-snapshot") => {
            ensure_no_extra_args(&args[1..], "j2000-snapshot")?;
            validate_render_cli(args)
        }
        Some("comparison-snapshot-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-summary")?;
            validate_render_cli(args)
        }
        Some("comparison-snapshot-batch-parity-summary")
        | Some("comparison-snapshot-batch-parity") => validate_render_cli(args),
        Some("reference-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source-summary")?;
            Ok(reference_snapshot_source_summary_for_report())
        }
        Some("reference-snapshot-source") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source")?;
            Ok(reference_snapshot_source_summary_for_report())
        }
        Some("reference-snapshot-source-window-summary") => validate_render_cli(args),
        Some("reference-snapshot-source-window") => validate_render_cli(args),
        Some("reference-snapshot-lunar-boundary-summary") => validate_render_cli(args),
        Some("lunar-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1500-selected-body-boundary-summary")
        | Some("1500-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2268932-selected-body-boundary-summary")
        | Some("2268932-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1600-selected-body-boundary-summary")
        | Some("1600-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2305457-selected-body-boundary-summary")
        | Some("2305457-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1750-selected-body-boundary-summary")
        | Some("1750-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1750-major-body-interior-summary")
        | Some("1750-major-body-interior-summary") => validate_render_cli(args),
        Some("reference-snapshot-1900-selected-body-boundary-summary")
        | Some("1900-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2415020-selected-body-boundary-summary")
        | Some("2415020-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2200-selected-body-boundary-summary")
        | Some("2200-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2360234-major-body-interior-summary")
        | Some("2360234-major-body-interior-summary") => validate_render_cli(args),
        Some("reference-snapshot-2524593-selected-body-boundary-summary")
        | Some("2524593-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2634167-selected-body-boundary-summary")
        | Some("2634167-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2500-selected-body-boundary-summary")
        | Some("2500-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1749-major-body-boundary-summary")
        | Some("1749-major-body-boundary-summary")
        | Some("2360233-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-early-major-body-boundary-summary")
        | Some("early-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2378498-major-body-boundary-summary")
        | Some("2378498-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1800-major-body-boundary-summary")
        | Some("1800-major-body-boundary-summary")
        | Some("2378499-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2400000-major-body-boundary-summary")
        | Some("2400000-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451545-major-body-boundary-summary")
        | Some("2451545-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451910-major-body-boundary-summary")
        | Some("2451910-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451911-major-body-boundary-summary")
        | Some("2451911-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451912-major-body-boundary-summary")
        | Some("2451912-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451913-major-body-boundary-summary")
        | Some("2451913-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451914-major-body-boundary-summary")
        | Some("2451914-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2500-major-body-boundary-summary")
        | Some("2500-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2453000-major-body-boundary-summary")
        | Some("2453000-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2500000-major-body-boundary-summary")
        | Some("2500000-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2600000-major-body-boundary-summary")
        | Some("2600000-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451915-major-body-boundary-summary")
        | Some("2451915-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451917-major-body-bridge-summary")
        | Some("2451917-major-body-bridge-summary")
        | Some("2451917-major-body-bridge") => validate_render_cli(args),
        Some("reference-snapshot-2451917-major-body-boundary-summary")
        | Some("2451917-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451918-major-body-boundary-summary")
        | Some("2451918-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451919-major-body-boundary-summary")
        | Some("2451919-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451914-major-body-pre-bridge-summary")
        | Some("2451914-major-body-pre-bridge-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451914-major-body-bridge-day-summary")
        | Some("2451914-major-body-bridge-day-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451914-bridge-day-summary")
        | Some("2451914-bridge-day-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451914-major-body-bridge-summary")
        | Some("2451914-major-body-bridge-summary")
        | Some("2451914-major-body-bridge") => validate_render_cli(args),
        Some("reference-snapshot-2451915-major-body-bridge-summary")
        | Some("2451915-major-body-bridge-summary")
        | Some("2451915-major-body-bridge") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451915-major-body-bridge-summary",
            )?;
            Ok(pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary_for_report())
        }
        Some("reference-snapshot-2451916-major-body-boundary-summary")
        | Some("2451916-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451916-major-body-dense-boundary-summary")
        | Some("2451916-major-body-dense-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451916-major-body-interior-summary")
        | Some("2451916-major-body-interior-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451920-major-body-interior-summary")
        | Some("2451920-major-body-interior-summary") => validate_render_cli(args),
        Some("reference-snapshot-body-class-coverage-summary")
        | Some("reference-body-class-coverage-summary") => validate_render_cli(args),
        Some("reference-snapshot-manifest-summary") | Some("reference-snapshot-manifest") => {
            validate_render_cli(args)
        }
        Some("reference-snapshot") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot")?;
            Ok(format!(
                "Reference snapshot summary\n{}\n",
                pleiades_jpl::reference_snapshot_summary_for_report()
            ))
        }
        Some("reference-snapshot-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-summary")?;
            validate_render_cli(args)
        }
        Some("reference-snapshot-exact-j2000-evidence-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-exact-j2000-evidence-summary",
            )?;
            validate_render_cli(args)
        }
        Some("reference-snapshot-exact-j2000-evidence") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-exact-j2000-evidence")?;
            validate_render_cli(args)
        }
        Some("exact-j2000-evidence") => {
            ensure_no_extra_args(&args[1..], "exact-j2000-evidence")?;
            validate_render_cli(args)
        }
        Some("reference-snapshot-batch-parity-summary") => validate_render_cli(args),
        Some("reference-snapshot-batch-parity") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-batch-parity")?;
            validate_render_cli(args)
        }
        Some("reference-snapshot-mixed-time-scale-batch-parity-summary")
        | Some("reference-snapshot-mixed-tt-tdb-batch-parity-summary")
        | Some("reference-snapshot-mixed-tt-tdb-batch-parity") => validate_render_cli(args),
        Some("reference-snapshot-equatorial-parity-summary") => validate_render_cli(args),
        Some("reference-snapshot-equatorial-parity") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-equatorial-parity")?;
            validate_render_cli(args)
        }
        Some("reference-high-curvature-summary")
        | Some("high-curvature-summary")
        | Some("reference-snapshot-major-body-high-curvature-summary")
        | Some("major-body-high-curvature-summary") => validate_render_cli(args),
        Some("reference-snapshot-major-body-boundary-summary")
        | Some("major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-major-body-bridge-summary")
        | Some("major-body-bridge-summary")
        | Some("bridge-summary") => validate_render_cli(args),
        Some("reference-snapshot-bridge-day-summary") => validate_render_cli(args),
        Some("bridge-day-summary") => validate_render_cli(args),
        Some("reference-snapshot-mars-jupiter-boundary-summary")
        | Some("mars-jupiter-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-mars-outer-boundary-summary")
        | Some("mars-outer-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-major-body-boundary-window-summary")
        | Some("major-body-boundary-window-summary") => validate_render_cli(args),
        Some("reference-high-curvature-window-summary")
        | Some("high-curvature-window-summary")
        | Some("reference-snapshot-major-body-high-curvature-window-summary")
        | Some("major-body-high-curvature-window-summary") => validate_render_cli(args),
        Some("reference-high-curvature-epoch-coverage-summary")
        | Some("high-curvature-epoch-coverage-summary")
        | Some("reference-snapshot-major-body-high-curvature-epoch-coverage-summary")
        | Some("major-body-high-curvature-epoch-coverage-summary") => validate_render_cli(args),
        Some("reference-snapshot-boundary-epoch-coverage-summary")
        | Some("reference-snapshot-boundary-epoch-coverage")
        | Some("boundary-epoch-coverage-summary") => validate_render_cli(args),
        Some("reference-snapshot-sparse-boundary-summary") | Some("sparse-boundary-summary") => {
            validate_render_cli(args)
        }
        Some("reference-snapshot-pre-bridge-boundary-summary")
        | Some("reference-snapshot-pre-bridge-boundary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-pre-bridge-boundary-summary")?;
            validate_render_cli(args)
        }
        Some("pre-bridge-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "pre-bridge-boundary-summary")?;
            validate_render_cli(args)
        }
        Some("reference-snapshot-boundary-day-summary")
        | Some("reference-snapshot-boundary-day")
        | Some("boundary-day-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-boundary-day-summary")?;
            validate_render_cli(args)
        }
        Some("reference-snapshot-dense-boundary-summary") => validate_render_cli(args),
        Some("dense-boundary-summary") => validate_render_cli(args),
        Some("source-documentation-summary") => validate_render_cli(args),
        Some("source-documentation") => validate_render_cli(args),
        Some("source-documentation-health-summary") => validate_render_cli(args),
        Some("source-documentation-health") => validate_render_cli(args),
        Some("source-audit-summary") => validate_render_cli(args),
        Some("source-audit") => validate_render_cli(args),
        Some("generated-binary-audit-summary") => validate_render_cli(args),
        Some("generated-binary-audit") => validate_render_cli(args),
        Some("time-scale-policy-summary") => {
            ensure_no_extra_args(&args[1..], "time-scale-policy-summary")?;
            validate_render_cli(args)
        }
        Some("time-scale-policy") => {
            ensure_no_extra_args(&args[1..], "time-scale-policy")?;
            validate_render_cli(args)
        }
        Some("utc-convenience-policy-summary") => {
            ensure_no_extra_args(&args[1..], "utc-convenience-policy-summary")?;
            validate_render_cli(args)
        }
        Some("utc-convenience-policy") => {
            ensure_no_extra_args(&args[1..], "utc-convenience-policy")?;
            validate_render_cli(args)
        }
        Some("delta-t-policy-summary") => {
            ensure_no_extra_args(&args[1..], "delta-t-policy-summary")?;
            validate_render_cli(args)
        }
        Some("delta-t-policy") => {
            ensure_no_extra_args(&args[1..], "delta-t-policy")?;
            validate_render_cli(args)
        }
        Some("observer-policy-summary") => {
            ensure_no_extra_args(&args[1..], "observer-policy-summary")?;
            validate_render_cli(args)
        }
        Some("observer-policy") => {
            ensure_no_extra_args(&args[1..], "observer-policy")?;
            validate_render_cli(args)
        }
        Some("apparentness-policy-summary") => {
            ensure_no_extra_args(&args[1..], "apparentness-policy-summary")?;
            validate_render_cli(args)
        }
        Some("apparentness-policy") => {
            ensure_no_extra_args(&args[1..], "apparentness-policy")?;
            validate_render_cli(args)
        }
        Some("native-sidereal-policy-summary") => {
            ensure_no_extra_args(&args[1..], "native-sidereal-policy-summary")?;
            validate_render_cli(args)
        }
        Some("native-sidereal-policy") => {
            ensure_no_extra_args(&args[1..], "native-sidereal-policy")?;
            validate_render_cli(args)
        }
        Some("zodiac-policy-summary") => {
            ensure_no_extra_args(&args[1..], "zodiac-policy-summary")?;
            validate_render_cli(args)
        }
        Some("zodiac-policy") => {
            ensure_no_extra_args(&args[1..], "zodiac-policy")?;
            validate_render_cli(args)
        }
        Some("interpolation-posture-summary") => validate_render_cli(args),
        Some("interpolation-posture") => validate_render_cli(args),
        Some("interpolation-quality-summary") => validate_render_cli(args),
        Some("interpolation-quality-kind-coverage-summary") => validate_render_cli(args),
        Some("interpolation-quality-request-corpus-summary") => validate_render_cli(args),
        Some("interpolation-quality-request-corpus") => validate_render_cli(args),
        Some("lunar-reference-error-envelope-summary") | Some("lunar-reference-error-envelope") => {
            validate_render_cli(args)
        }
        Some("lunar-equatorial-reference-error-envelope-summary")
        | Some("lunar-equatorial-reference-error-envelope") => validate_render_cli(args),
        Some("lunar-apparent-comparison-summary") | Some("lunar-apparent-comparison") => {
            validate_render_cli(args)
        }
        Some("reference-snapshot-lunar-source-window-summary")
        | Some("lunar-source-window-summary")
        | Some("lunar-source-window") => validate_render_cli(args),
        Some("lunar-reference-mixed-time-scale-batch-parity-summary")
        | Some("lunar-reference-mixed-tt-tdb-batch-parity-summary")
        | Some("lunar-reference-mixed-tt-tdb-batch-parity") => validate_render_cli(args),
        Some("lunar-theory-request-policy-summary") => validate_render_cli(args),
        Some("lunar-theory-request-policy") => validate_render_cli(args),
        Some("lunar-theory-frame-treatment-summary") => validate_render_cli(args),
        Some("lunar-theory-frame-treatment") => validate_render_cli(args),
        Some("lunar-theory-limitations-summary") | Some("lunar-theory-limitations") => {
            validate_render_cli(args)
        }
        Some("lunar-theory-summary") => validate_render_cli(args),
        Some("lunar-theory-capability-summary") => validate_render_cli(args),
        Some("lunar-theory-source-summary") => validate_render_cli(args),
        Some("lunar-theory-source-selection-summary") | Some("lunar-theory-source-selection") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-source-selection-summary")?;
            Ok(pleiades_elp::lunar_theory_source_selection_summary_for_report())
        }
        Some("lunar-theory-source-family-summary") | Some("lunar-theory-source-family") => {
            validate_render_cli(args)
        }
        Some("lunar-theory-catalog-summary") | Some("lunar-theory-catalog") => {
            validate_render_cli(args)
        }
        Some("lunar-theory-catalog-validation-summary")
        | Some("lunar-theory-catalog-validation") => validate_render_cli(args),
        Some("selected-asteroid-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-selected-asteroid-bridge-summary")
        | Some("selected-asteroid-bridge-summary") => validate_render_cli(args),
        Some("reference-snapshot-selected-asteroid-dense-boundary-summary")
        | Some("selected-asteroid-dense-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-selected-asteroid-terminal-boundary-summary")
        | Some("selected-asteroid-terminal-boundary-summary") => validate_render_cli(args),
        Some("selected-asteroid-source-evidence-summary") => validate_render_cli(args),
        Some("reference-snapshot-2378498-selected-asteroid-source-summary")
        | Some("2378498-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2378498-selected-asteroid-source-summary",
            )?;
            Ok(pleiades_jpl::selected_asteroid_source_2378498_summary_for_report())
        }
        Some("reference-snapshot-2451917-selected-asteroid-source-summary")
        | Some("2451917-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2451917-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2451917_summary_for_report())
        }
        Some("reference-snapshot-selected-asteroid-source-summary")
        | Some("selected-asteroid-source-summary") => validate_render_cli(args),
        Some("selected-asteroid-source-request-corpus-summary") => validate_render_cli(args),
        Some("selected-asteroid-source-request-corpus") => validate_render_cli(args),
        Some("selected-asteroid-source-request-corpus-equatorial-summary") => {
            validate_render_cli(args)
        }
        Some("selected-asteroid-source-request-corpus-equatorial") => validate_render_cli(args),
        Some("reference-snapshot-selected-asteroid-source-window-summary")
        | Some("reference-snapshot-selected-asteroid-source-window")
        | Some("selected-asteroid-source-window-summary") => validate_render_cli(args),
        Some("reference-snapshot-2453000-selected-asteroid-source-summary")
        | Some("2453000-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2453000-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2453000_summary_for_report())
        }
        Some("reference-snapshot-2500000-selected-asteroid-source-summary")
        | Some("2500000-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2500000-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2500000_summary_for_report())
        }
        Some("reference-snapshot-2634167-selected-asteroid-source-summary")
        | Some("2634167-selected-asteroid-source-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2634167-selected-asteroid-source-summary",
            )?;
            Ok(selected_asteroid_source_2634167_summary_for_report())
        }
        Some("selected-asteroid-source-window") => validate_render_cli(args),
        Some("selected-asteroid-batch-parity-summary") | Some("selected-asteroid-batch-parity") => {
            validate_render_cli(args)
        }
        Some("reference-asteroid-evidence-summary") => validate_render_cli(args),
        Some("reference-asteroid-equatorial-evidence-summary")
        | Some("reference-asteroid-equatorial-evidence") => validate_render_cli(args),
        Some("reference-asteroid-source-window-summary")
        | Some("reference-asteroid-source-window") => validate_render_cli(args),
        Some("reference-asteroid-source-summary") => validate_render_cli(args),
        Some("reference-holdout-overlap-summary") => validate_render_cli(args),
        Some("holdout-overlap-summary") => validate_render_cli(args),
        Some("independent-holdout-source-window-summary") => validate_render_cli(args),
        Some("independent-holdout-manifest-summary") => validate_render_cli(args),
        Some("independent-holdout-manifest") => validate_render_cli(args),
        Some("independent-holdout-quarter-day-boundary-summary")
        | Some("independent-holdout-quarter-day-boundary") => validate_render_cli(args),
        Some("independent-holdout-summary") => validate_render_cli(args),
        Some("independent-holdout-source-summary") => validate_render_cli(args),
        Some("independent-holdout-high-curvature-summary") => validate_render_cli(args),
        Some("holdout-high-curvature-summary") => validate_render_cli(args),
        Some("independent-holdout-body-class-coverage-summary")
        | Some("holdout-body-class-coverage-summary") => validate_render_cli(args),
        Some("independent-holdout-batch-parity-summary")
        | Some("independent-holdout-batch-parity") => validate_render_cli(args),
        Some("independent-holdout-equatorial-parity-summary")
        | Some("independent-holdout-equatorial-parity") => validate_render_cli(args),
        Some("house-validation-summary") | Some("house-validation") => validate_render_cli(args),
        Some("house-formula-families-summary") => validate_render_cli(args),
        Some("house-formula-families") => validate_render_cli(args),
        Some("house-latitude-sensitive-summary") | Some("house-latitude-sensitive") => {
            validate_render_cli(args)
        }
        Some("house-latitude-sensitive-failure-modes-summary")
        | Some("house-latitude-sensitive-failure-modes") => validate_render_cli(args),
        Some("house-latitude-sensitive-constraints-summary")
        | Some("house-latitude-sensitive-constraints") => validate_render_cli(args),
        Some("house-code-aliases-summary") => validate_render_cli(args),
        Some("house-code-alias-summary") => validate_render_cli(args),
        Some("catalog-posture-summary") => {
            ensure_no_extra_args(&args[1..], "catalog-posture-summary")?;
            validate_render_cli(args)
        }
        Some("catalog-posture") => {
            ensure_no_extra_args(&args[1..], "catalog-posture")?;
            validate_render_cli(args)
        }
        Some("known-gaps-summary") => {
            ensure_no_extra_args(&args[1..], "known-gaps-summary")?;
            validate_render_cli(args)
        }
        Some("known-gaps") => {
            ensure_no_extra_args(&args[1..], "known-gaps")?;
            validate_render_cli(args)
        }
        Some("ayanamsa-catalog-validation-summary") => validate_render_cli(args),
        Some("ayanamsa-catalog-validation") => validate_render_cli(args),
        Some("ayanamsa-metadata-coverage-summary") => validate_render_cli(args),
        Some("ayanamsa-metadata-coverage") => validate_render_cli(args),
        Some("ayanamsa-reference-offsets-summary") => validate_render_cli(args),
        Some("ayanamsa-reference-offsets") => validate_render_cli(args),
        Some("ayanamsa-provenance-summary") => validate_render_cli(args),
        Some("ayanamsa-provenance") => validate_render_cli(args),
        Some("frame-policy-summary") => {
            ensure_no_extra_args(&args[1..], "frame-policy-summary")?;
            validate_render_cli(args)
        }
        Some("frame-policy") => {
            ensure_no_extra_args(&args[1..], "frame-policy")?;
            validate_render_cli(args)
        }
        Some("mean-obliquity-frame-round-trip-summary") => validate_render_cli(args),
        Some("mean-obliquity-frame-round-trip") => validate_render_cli(args),
        Some("release-profile-identifiers-summary") => {
            ensure_no_extra_args(&args[1..], "release-profile-identifiers-summary")?;
            validate_render_cli(args)
        }
        Some("release-profile-identifiers") => {
            ensure_no_extra_args(&args[1..], "release-profile-identifiers")?;
            validate_render_cli(args)
        }
        Some("target-house-scope-summary") | Some("target-house-scope") => {
            validate_render_cli(args)
        }
        Some("target-ayanamsa-scope-summary") | Some("target-ayanamsa-scope") => {
            validate_render_cli(args)
        }
        Some("request-surface-summary") | Some("request-surface") => validate_render_cli(args),
        Some("request-policy-summary") => validate_render_cli(args),
        Some("request-policy") => validate_render_cli(args),
        Some("request-semantics-summary") => validate_render_cli(args),
        Some("request-semantics") => validate_render_cli(args),
        Some("unsupported-modes-summary") | Some("unsupported-modes") => validate_render_cli(args),
        Some("comparison-tolerance-policy-summary") | Some("comparison-tolerance-summary") => {
            validate_render_cli(args)
        }
        Some("comparison-tolerance-scope-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-tolerance-scope-coverage-summary")?;
            validate_render_cli(args)
        }
        Some("comparison-tolerance-scope-coverage") => {
            ensure_no_extra_args(&args[1..], "comparison-tolerance-scope-coverage")?;
            validate_render_cli(args)
        }
        Some("comparison-body-class-tolerance-summary")
        | Some("comparison-body-class-tolerance") => validate_render_cli(args),
        Some("comparison-body-class-tolerance-posture-summary")
        | Some("comparison-body-class-tolerance-posture") => validate_render_cli(args),
        Some("comparison-envelope-summary") | Some("comparison-envelope") => {
            validate_render_cli(args)
        }
        Some("comparison-body-class-error-envelope-summary")
        | Some("comparison-body-class-error-envelope") => validate_render_cli(args),
        Some("release-body-claims-summary") | Some("body-claims-summary") => {
            validate_render_cli(args)
        }
        Some("body-date-channel-claims-summary") | Some("body-date-channel-claims") => {
            validate_render_cli(args)
        }
        Some("lunar-reference-evidence-summary") | Some("lunar-reference-evidence") => {
            validate_render_cli(args)
        }
        Some("pluto-fallback-summary") => validate_render_cli(args),
        Some("pluto-fallback") => validate_render_cli(args),
        Some("ayanamsa-audit-summary") | Some("ayanamsa-audit") => validate_render_cli(args),
        Some("packaged-artifact-body-cadence-summary") | Some("packaged-artifact-body-cadence") => {
            validate_render_cli(args)
        }
        Some("packaged-artifact-fit-margins-summary") | Some("packaged-artifact-fit-margins") => {
            validate_render_cli(args)
        }
        Some("packaged-artifact-generation-manifest-checksum-summary")
        | Some("packaged-artifact-generation-manifest-checksum") => validate_render_cli(args),
        Some("packaged-artifact-normalized-intermediate-summary")
        | Some("packaged-artifact-normalized-intermediate") => validate_render_cli(args),
        Some("packaged-artifact-phase2-corpus-alignment-summary")
        | Some("packaged-artifact-phase2-corpus-alignment") => validate_render_cli(args),
        Some("packaged-artifact-target-threshold-state-summary")
        | Some("packaged-artifact-target-threshold-state") => validate_render_cli(args),
        Some("workspace-audit-summary") | Some("native-dependency-audit-summary") => {
            validate_render_cli(args)
        }
        Some("workspace-provenance-summary") | Some("workspace-provenance") => {
            validate_render_cli(args)
        }
        Some("artifact-summary") | Some("artifact-posture-summary") => validate_render_cli(args),
        Some("artifact-boundary-envelope-summary") => validate_render_cli(args),
        Some("artifact-profile-coverage-summary") => validate_render_cli(args),
        Some("packaged-artifact-output-support-summary")
        | Some("packaged-artifact-output-support") => validate_render_cli(args),
        Some("packaged-artifact-body-class-span-cap-summary")
        | Some("packaged-artifact-body-class-span-cap") => validate_render_cli(args),
        Some("packaged-artifact-speed-policy-summary") | Some("packaged-artifact-speed-policy") => {
            validate_render_cli(args)
        }
        Some("motion-policy-summary") | Some("motion-policy") => validate_render_cli(args),
        Some("packaged-artifact-access-summary") | Some("packaged-artifact-access") => {
            validate_render_cli(args)
        }
        Some("packaged-artifact-path-policy-summary") | Some("packaged-artifact-path-policy") => {
            validate_render_cli(args)
        }
        Some("packaged-artifact-storage-summary") | Some("packaged-artifact-storage") => {
            validate_render_cli(args)
        }
        Some("packaged-artifact-production-profile-summary")
        | Some("packaged-artifact-production-profile") => validate_render_cli(args),
        Some("packaged-artifact-target-threshold-summary")
        | Some("packaged-artifact-target-threshold") => validate_render_cli(args),
        Some("packaged-artifact-target-threshold-scope-envelopes-summary")
        | Some("packaged-artifact-target-threshold-scope-envelopes") => validate_render_cli(args),
        Some("packaged-artifact-source-fit-holdout-sync-summary")
        | Some("packaged-artifact-source-fit-holdout-sync") => validate_render_cli(args),
        Some("packaged-artifact-fit-envelope-summary") | Some("packaged-artifact-fit-envelope") => {
            validate_render_cli(args)
        }
        Some("packaged-artifact-fit-sample-classes-summary")
        | Some("packaged-artifact-fit-sample-classes") => validate_render_cli(args),
        Some("packaged-artifact-fit-outliers-summary") | Some("packaged-artifact-fit-outliers") => {
            validate_render_cli(args)
        }
        Some("packaged-artifact-fit-threshold-violation-count-summary")
        | Some("packaged-artifact-fit-threshold-violation-count") => validate_render_cli(args),
        Some("packaged-artifact-fit-threshold-violations-summary")
        | Some("packaged-artifact-fit-threshold-violations") => validate_render_cli(args),
        Some("packaged-artifact-generation-manifest-summary")
        | Some("packaged-artifact-generation-manifest") => validate_render_cli(args),
        Some("packaged-artifact-generation-policy-summary")
        | Some("packaged-artifact-generation-policy") => validate_render_cli(args),
        Some("packaged-artifact-generation-residual-summary") => validate_render_cli(args),
        Some("packaged-artifact-generation-residual-bodies-summary") => validate_render_cli(args),
        Some("packaged-artifact-regeneration-summary") | Some("packaged-artifact-regeneration") => {
            validate_render_cli(args)
        }
        Some("packaged-frame-parity-summary") | Some("packaged-frame-parity") => {
            validate_render_cli(args)
        }
        Some("packaged-frame-treatment-summary") => validate_render_cli(args),
        Some("packaged-lookup-epoch-policy-summary")
        | Some("packaged-lookup-epoch-policy")
        | Some("packaged-artifact-lookup-epoch-policy-summary")
        | Some("packaged-artifact-lookup-epoch-policy") => validate_render_cli(args),
        Some("validate-artifact") => validate_render_cli(args),
        Some("generate-packaged-artifact") | Some("regenerate-packaged-artifact") => {
            if args[1..].iter().any(|arg| *arg == "--help" || *arg == "-h") {
                return Ok(help_text());
            }
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
        Some("workspace-audit") | Some("audit") | Some("native-dependency-audit") => {
            validate_render_cli(args)
        }
        Some("report") | Some("generate-report") => validate_render_cli(args),
        Some("validation-report-summary") | Some("validation-summary") | Some("report-summary") => {
            let rounds = parse_rounds(&args[1..], 1)?;
            render_validation_report_summary(rounds).map_err(render_error)
        }
        Some("chart") => render_chart(&args[1..]),
        Some("generate-spk-corpus") => render_spk_corpus(&args[1..]),
        Some("generate-fixture-golden") => render_fixture_golden(&args[1..]),
        Some("help") | Some("--help") | Some("-h") => Ok(help_text()),
        None => Ok(banner().to_string()),
        Some(other) => match validate_render_cli(args) {
            Ok(rendered) => Ok(rendered),
            Err(error) if error.starts_with("unknown command: ") => {
                Err(format!("unknown command: {other}\n\n{}", help_text()))
            }
            Err(error) => Err(error),
        },
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
