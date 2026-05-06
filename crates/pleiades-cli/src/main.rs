//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! The CLI now exposes the compatibility profile and a small chart report
//! command so contributors can exercise the first end-to-end workflow without
//! leaving the repository. The chart report keeps the mean/apparent position
//! choice explicit so report consumers can see which backend mode was used.

#![forbid(unsafe_code)]

use core::time::Duration;

use pleiades_core::{
    default_chart_bodies, native_sidereal_policy_summary_for_report, resolve_ayanamsa,
    resolve_house_system, Angle, Apparentness, Ayanamsa, CelestialBody, ChartEngine, ChartRequest,
    CompositeBackend, CustomAyanamsa, CustomBodyId, EphemerisError, HouseSystem, Instant,
    JulianDay, Latitude, Longitude, ObserverLocation, RoutingBackend, TimeScale, ZodiacMode,
};
use pleiades_data::{
    packaged_artifact_bytes, packaged_artifact_regeneration_summary_for_report,
    regenerate_packaged_artifact, PackagedDataBackend,
};
use pleiades_elp::ElpBackend;
use pleiades_jpl::{
    comparison_snapshot_source_summary_for_report, reference_snapshot_source_summary_for_report,
    JplSnapshotBackend,
};
use pleiades_validate::{
    current_request_surface_summary, render_benchmark_report, render_cli as validate_render_cli,
    render_release_bundle, render_validation_report_summary, verify_compatibility_profile,
};
use pleiades_vsop87::Vsop87Backend;

fn banner() -> &'static str {
    "pleiades-cli chart utility"
}

fn ensure_no_extra_args(args: &[&str], command: &str) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("{command} does not accept extra arguments"))
    }
}

fn shared_request_policy_help_block() -> String {
    let request_policy = pleiades_core::request_policy_summary_for_report();
    let time_scale_policy = pleiades_core::time_scale_policy_summary_for_report();
    let utc_convenience_policy = pleiades_core::utc_convenience_policy_summary_for_report();
    let delta_t_policy = pleiades_core::delta_t_policy_summary_for_report();
    let observer_policy = pleiades_core::observer_policy_summary_for_report();
    let apparentness_policy = pleiades_core::apparentness_policy_summary_for_report();
    let native_sidereal_policy = native_sidereal_policy_summary_for_report();
    let frame_policy = pleiades_core::frame_policy_summary_for_report();

    format!(
        "  Request policy: {}\n  Time-scale policy: {}\n  UTC convenience policy: {}\n  Delta T policy: {}\n  Observer policy: {}\n  Apparentness policy: {}\n  Native sidereal policy: {}\n  Frame policy: {}",
        request_policy.summary_line(),
        time_scale_policy.summary_line(),
        utc_convenience_policy.summary_line(),
        delta_t_policy.summary_line(),
        observer_policy.summary_line(),
        apparentness_policy.summary_line(),
        native_sidereal_policy.summary_line(),
        frame_policy,
    )
}

fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compare-backends") => {
            ensure_no_extra_args(&args[1..], "compare-backends")?;
            validate_render_cli(args)
        }
        Some("compare-backends-audit") => {
            ensure_no_extra_args(&args[1..], "compare-backends-audit")?;
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
        Some("release-notes") => validate_render_cli(args),
        Some("release-notes-summary") => validate_render_cli(args),
        Some("release-checklist") => validate_render_cli(args),
        Some("release-gate") => validate_render_cli(args),
        Some("release-checklist-summary") | Some("checklist-summary") => validate_render_cli(args),
        Some("release-gate-summary") => validate_render_cli(args),
        Some("release-summary") => validate_render_cli(args),
        Some("jpl-batch-error-taxonomy-summary") => validate_render_cli(args),
        Some("jpl-snapshot-evidence-summary") => validate_render_cli(args),
        Some("production-generation-boundary-summary") => validate_render_cli(args),
        Some("production-generation-boundary-request-corpus-summary") => validate_render_cli(args),
        Some("production-generation-body-class-coverage-summary")
        | Some("production-body-class-coverage-summary") => validate_render_cli(args),
        Some("production-generation-source-window-summary") => validate_render_cli(args),
        Some("production-generation") | Some("production-generation-summary") => {
            validate_render_cli(args)
        }
        Some("production-generation-boundary-source-summary") => validate_render_cli(args),
        Some("production-generation-boundary-window-summary") => validate_render_cli(args),
        Some("production-generation-boundary-window") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-window")?;
            validate_render_cli(args)
        }
        Some("production-generation-source-summary") => validate_render_cli(args),
        Some("production-generation-source") => {
            ensure_no_extra_args(&args[1..], "production-generation-source")?;
            validate_render_cli(args)
        }
        Some("comparison-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source-summary")?;
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
        Some("comparison-snapshot-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-summary")?;
            validate_render_cli(args)
        }
        Some("comparison-snapshot-batch-parity-summary") => validate_render_cli(args),
        Some("reference-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source-summary")?;
            Ok(reference_snapshot_source_summary_for_report())
        }
        Some("reference-snapshot-source-window-summary") => validate_render_cli(args),
        Some("reference-snapshot-source-window") => validate_render_cli(args),
        Some("reference-snapshot-lunar-boundary-summary") => validate_render_cli(args),
        Some("lunar-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1500-selected-body-boundary-summary")
        | Some("1500-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1600-selected-body-boundary-summary")
        | Some("1600-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1750-selected-body-boundary-summary")
        | Some("1750-selected-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-1750-major-body-interior-summary")
        | Some("1750-major-body-interior-summary") => validate_render_cli(args),
        Some("reference-snapshot-1900-selected-body-boundary-summary")
        | Some("1900-selected-body-boundary-summary") => validate_render_cli(args),
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
        Some("reference-snapshot-2600000-major-body-boundary-summary")
        | Some("2600000-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451915-major-body-boundary-summary")
        | Some("2451915-major-body-boundary-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451917-major-body-bridge-summary")
        | Some("2451917-major-body-bridge-summary") => validate_render_cli(args),
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
        Some("reference-snapshot-2451914-major-body-bridge-summary")
        | Some("2451914-major-body-bridge-summary") => validate_render_cli(args),
        Some("reference-snapshot-2451915-major-body-bridge-summary")
        | Some("2451915-major-body-bridge-summary") => validate_render_cli(args),
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
        Some("exact-j2000-evidence") => {
            ensure_no_extra_args(&args[1..], "exact-j2000-evidence")?;
            validate_render_cli(args)
        }
        Some("reference-snapshot-batch-parity-summary") => validate_render_cli(args),
        Some("reference-snapshot-equatorial-parity-summary") => validate_render_cli(args),
        Some("reference-high-curvature-summary") | Some("high-curvature-summary") => {
            validate_render_cli(args)
        }
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
        Some("reference-high-curvature-window-summary") | Some("high-curvature-window-summary") => {
            validate_render_cli(args)
        }
        Some("reference-high-curvature-epoch-coverage-summary")
        | Some("high-curvature-epoch-coverage-summary") => validate_render_cli(args),
        Some("reference-snapshot-boundary-epoch-coverage-summary")
        | Some("boundary-epoch-coverage-summary") => validate_render_cli(args),
        Some("reference-snapshot-sparse-boundary-summary") | Some("sparse-boundary-summary") => {
            validate_render_cli(args)
        }
        Some("reference-snapshot-pre-bridge-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-pre-bridge-boundary-summary")?;
            validate_render_cli(args)
        }
        Some("pre-bridge-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "pre-bridge-boundary-summary")?;
            validate_render_cli(args)
        }
        Some("boundary-day-summary") => {
            ensure_no_extra_args(&args[1..], "boundary-day-summary")?;
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
        Some("interpolation-posture-summary") => validate_render_cli(args),
        Some("interpolation-posture") => validate_render_cli(args),
        Some("interpolation-quality-summary") => validate_render_cli(args),
        Some("interpolation-quality-kind-coverage-summary") => validate_render_cli(args),
        Some("lunar-reference-error-envelope-summary") => validate_render_cli(args),
        Some("lunar-equatorial-reference-error-envelope-summary") => validate_render_cli(args),
        Some("lunar-apparent-comparison-summary") => validate_render_cli(args),
        Some("lunar-source-window-summary") => validate_render_cli(args),
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
        Some("selected-asteroid-source-summary") => validate_render_cli(args),
        Some("selected-asteroid-source-window-summary") => validate_render_cli(args),
        Some("selected-asteroid-source-window") => validate_render_cli(args),
        Some("selected-asteroid-batch-parity-summary") => validate_render_cli(args),
        Some("reference-asteroid-evidence-summary") => validate_render_cli(args),
        Some("reference-asteroid-equatorial-evidence-summary") => validate_render_cli(args),
        Some("reference-asteroid-source-window-summary") => validate_render_cli(args),
        Some("reference-asteroid-source-summary") => validate_render_cli(args),
        Some("reference-holdout-overlap-summary") => validate_render_cli(args),
        Some("holdout-overlap-summary") => validate_render_cli(args),
        Some("independent-holdout-source-window-summary") => validate_render_cli(args),
        Some("independent-holdout-summary") => validate_render_cli(args),
        Some("independent-holdout-source-summary") => validate_render_cli(args),
        Some("independent-holdout-high-curvature-summary") => validate_render_cli(args),
        Some("holdout-high-curvature-summary") => validate_render_cli(args),
        Some("independent-holdout-body-class-coverage-summary")
        | Some("holdout-body-class-coverage-summary") => validate_render_cli(args),
        Some("independent-holdout-batch-parity-summary") => validate_render_cli(args),
        Some("independent-holdout-equatorial-parity-summary") => validate_render_cli(args),
        Some("house-validation-summary") | Some("house-validation") => validate_render_cli(args),
        Some("house-formula-families-summary") => validate_render_cli(args),
        Some("house-formula-families") => validate_render_cli(args),
        Some("house-latitude-sensitive-summary") | Some("house-latitude-sensitive") => {
            validate_render_cli(args)
        }
        Some("house-code-aliases-summary") => validate_render_cli(args),
        Some("house-code-alias-summary") => validate_render_cli(args),
        Some("ayanamsa-catalog-validation-summary") => validate_render_cli(args),
        Some("ayanamsa-catalog-validation") => validate_render_cli(args),
        Some("ayanamsa-metadata-coverage-summary") => validate_render_cli(args),
        Some("ayanamsa-metadata-coverage") => validate_render_cli(args),
        Some("ayanamsa-reference-offsets-summary") => validate_render_cli(args),
        Some("ayanamsa-reference-offsets") => validate_render_cli(args),
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
        Some("request-surface-summary") | Some("request-surface") => validate_render_cli(args),
        Some("request-policy-summary") => validate_render_cli(args),
        Some("request-policy") => validate_render_cli(args),
        Some("request-semantics-summary") => validate_render_cli(args),
        Some("request-semantics") => validate_render_cli(args),
        Some("comparison-tolerance-policy-summary") | Some("comparison-tolerance-summary") => {
            validate_render_cli(args)
        }
        Some("comparison-body-class-tolerance-summary")
        | Some("comparison-body-class-tolerance") => validate_render_cli(args),
        Some("comparison-envelope-summary") | Some("comparison-envelope") => {
            validate_render_cli(args)
        }
        Some("release-body-claims-summary") | Some("body-claims-summary") => {
            validate_render_cli(args)
        }
        Some("pluto-fallback-summary") => validate_render_cli(args),
        Some("pluto-fallback") => validate_render_cli(args),
        Some("workspace-audit-summary") | Some("native-dependency-audit-summary") => {
            validate_render_cli(args)
        }
        Some("artifact-summary") | Some("artifact-posture-summary") => validate_render_cli(args),
        Some("artifact-boundary-envelope-summary") => validate_render_cli(args),
        Some("artifact-profile-coverage-summary") => validate_render_cli(args),
        Some("packaged-artifact-output-support-summary")
        | Some("packaged-artifact-output-support") => validate_render_cli(args),
        Some("packaged-artifact-speed-policy-summary") | Some("packaged-artifact-speed-policy") => {
            validate_render_cli(args)
        }
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
        Some("packaged-artifact-generation-manifest-summary") => validate_render_cli(args),
        Some("packaged-artifact-generation-policy-summary")
        | Some("packaged-artifact-generation-policy") => validate_render_cli(args),
        Some("packaged-artifact-generation-residual-summary") => validate_render_cli(args),
        Some("packaged-artifact-generation-residual-bodies-summary") => validate_render_cli(args),
        Some("packaged-artifact-regeneration-summary") | Some("packaged-artifact-regeneration") => {
            validate_render_cli(args)
        }
        Some("packaged-frame-parity-summary") => validate_render_cli(args),
        Some("packaged-frame-treatment-summary") => validate_render_cli(args),
        Some("packaged-lookup-epoch-policy-summary") | Some("packaged-lookup-epoch-policy") => {
            validate_render_cli(args)
        }
        Some("validate-artifact") => validate_render_cli(args),
        Some("regenerate-packaged-artifact") => {
            if args[1..].iter().any(|arg| *arg == "--help" || *arg == "-h") {
                return Ok(help_text());
            }
            match parse_packaged_artifact_command(&args[1..])? {
                PackagedArtifactCommand::Write { output_path } => {
                    render_packaged_artifact_regeneration(output_path)
                }
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
        Some("help") | Some("--help") | Some("-h") => Ok(help_text()),
        None => Ok(banner().to_string()),
        Some(other) => Err(format!("unknown command: {other}\n\n{}", help_text())),
    }
}

fn help_text() -> String {
    let validation_help = validate_render_cli(&["help"]).expect("validation help should render");
    let validation_commands = validation_help
        .split_once(
            "

Commands:
",
        )
        .map(|(_, tail)| tail)
        .unwrap_or_else(|| validation_help.as_str());
    let validation_commands = validation_commands
        .rsplit_once(
            "
  help                      Show this help text",
        )
        .map(|(commands, _)| commands)
        .unwrap_or(validation_commands);

    format!(
        "{}

Commands:
{}
  chart                  Render a basic chart report
    --tt|--tdb|--utc|--ut1  Tag the chart instant with a time scale
    --tt-offset-seconds <seconds>  Caller-supplied TT offset for UTC/UT1-tagged instants
    --tt-from-utc-offset-seconds <seconds>  Alias for --tt-offset-seconds when the chart instant is tagged as UTC
    --tt-from-ut1-offset-seconds <seconds>  Alias for --tt-offset-seconds when the chart instant is tagged as UT1
    --tdb-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT/UTC/UT1-tagged instants
    --tdb-from-utc-offset-seconds <seconds> Explicit UTC-tagged alias for the signed TDB-TT offset
    --tdb-from-ut1-offset-seconds <seconds> Explicit UT1-tagged alias for the signed TDB-TT offset
    --tdb-from-tt-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT-tagged instants
    --tt-from-tdb-offset-seconds <seconds> Caller-supplied signed TT-TDB offset for TDB-tagged instants
    --mean               Force mean positions for backend queries
    --apparent           Force apparent positions for backend queries
    --body <name>        Use a built-in body or a custom catalog:designation identifier
  {}
  help                   Show this help text",
        banner(),
        validation_commands,
        shared_request_policy_help_block(),
    )
}

fn render_chart(args: &[&str]) -> Result<String, String> {
    let mut jd: Option<f64> = None;
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut bodies: Vec<CelestialBody> = Vec::new();
    let mut zodiac_mode = ZodiacMode::Tropical;
    let mut time_scale = TimeScale::Tt;
    let mut time_scale_explicit = false;
    let mut tt_offset_seconds: Option<f64> = None;
    let mut tdb_offset_seconds: Option<f64> = None;
    let mut tdb_from_utc_offset_seconds: Option<f64> = None;
    let mut tdb_from_ut1_offset_seconds: Option<f64> = None;
    let mut tdb_from_tt_offset_seconds: Option<f64> = None;
    let mut apparentness = Apparentness::Mean;
    let mut apparentness_explicit = false;
    let mut house_system: Option<HouseSystem> = None;
    let mut tt_from_tdb_offset_seconds: Option<f64> = None;

    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--jd" => jd = Some(parse_f64(iter.next(), "--jd")?),
            "--lat" => lat = Some(parse_f64(iter.next(), "--lat")?),
            "--lon" => lon = Some(parse_f64(iter.next(), "--lon")?),
            "--body" => bodies.push(parse_body(iter.next())?),
            "--tt" => {
                if time_scale_explicit {
                    return Err(
                        "conflicting time-scale flags: use only one of --tt, --tdb, --utc, or --ut1"
                            .to_string(),
                    );
                }
                time_scale = TimeScale::Tt;
                time_scale_explicit = true;
            }
            "--tdb" => {
                if time_scale_explicit {
                    return Err(
                        "conflicting time-scale flags: use only one of --tt, --tdb, --utc, or --ut1"
                            .to_string(),
                    );
                }
                time_scale = TimeScale::Tdb;
                time_scale_explicit = true;
            }
            "--utc" => {
                if time_scale_explicit {
                    return Err(
                        "conflicting time-scale flags: use only one of --tt, --tdb, --utc, or --ut1"
                            .to_string(),
                    );
                }
                time_scale = TimeScale::Utc;
                time_scale_explicit = true;
            }
            "--ut1" => {
                if time_scale_explicit {
                    return Err(
                        "conflicting time-scale flags: use only one of --tt, --tdb, --utc, or --ut1"
                            .to_string(),
                    );
                }
                time_scale = TimeScale::Ut1;
                time_scale_explicit = true;
            }
            "--tt-offset-seconds" => {
                if tt_offset_seconds.is_some() {
                    return Err(
                        "conflicting TT offset flags: use only one of --tt-offset-seconds, --tt-from-utc-offset-seconds, or --tt-from-ut1-offset-seconds"
                            .to_string(),
                    );
                }
                tt_offset_seconds = Some(parse_seconds(iter.next(), "--tt-offset-seconds")?);
            }
            "--tt-from-utc-offset-seconds" => {
                if tt_offset_seconds.is_some() {
                    return Err(
                        "conflicting TT offset flags: use only one of --tt-offset-seconds, --tt-from-utc-offset-seconds, or --tt-from-ut1-offset-seconds"
                            .to_string(),
                    );
                }
                tt_offset_seconds = Some(parse_seconds(iter.next(), "--tt-offset-seconds")?);
            }
            "--tt-from-ut1-offset-seconds" => {
                if tt_offset_seconds.is_some() {
                    return Err(
                        "conflicting TT offset flags: use only one of --tt-offset-seconds, --tt-from-utc-offset-seconds, or --tt-from-ut1-offset-seconds"
                            .to_string(),
                    );
                }
                tt_offset_seconds = Some(parse_seconds(iter.next(), "--tt-offset-seconds")?);
            }
            "--tdb-offset-seconds" => {
                if tdb_offset_seconds.is_some()
                    || tdb_from_utc_offset_seconds.is_some()
                    || tdb_from_ut1_offset_seconds.is_some()
                    || tdb_from_tt_offset_seconds.is_some()
                {
                    return Err(
                        "conflicting TDB-TT offset flags: use only one of --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, or --tdb-from-tt-offset-seconds"
                            .to_string(),
                    );
                }
                tdb_offset_seconds =
                    Some(parse_signed_seconds(iter.next(), "--tdb-offset-seconds")?);
            }
            "--tdb-from-utc-offset-seconds" => {
                if tdb_offset_seconds.is_some()
                    || tdb_from_utc_offset_seconds.is_some()
                    || tdb_from_ut1_offset_seconds.is_some()
                    || tdb_from_tt_offset_seconds.is_some()
                {
                    return Err(
                        "conflicting TDB-TT offset flags: use only one of --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, or --tdb-from-tt-offset-seconds"
                            .to_string(),
                    );
                }
                tdb_from_utc_offset_seconds = Some(parse_signed_seconds(
                    iter.next(),
                    "--tdb-from-utc-offset-seconds",
                )?);
            }
            "--tdb-from-ut1-offset-seconds" => {
                if tdb_offset_seconds.is_some()
                    || tdb_from_utc_offset_seconds.is_some()
                    || tdb_from_ut1_offset_seconds.is_some()
                    || tdb_from_tt_offset_seconds.is_some()
                {
                    return Err(
                        "conflicting TDB-TT offset flags: use only one of --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, or --tdb-from-tt-offset-seconds"
                            .to_string(),
                    );
                }
                tdb_from_ut1_offset_seconds = Some(parse_signed_seconds(
                    iter.next(),
                    "--tdb-from-ut1-offset-seconds",
                )?);
            }
            "--tdb-from-tt-offset-seconds" => {
                if tdb_offset_seconds.is_some()
                    || tdb_from_utc_offset_seconds.is_some()
                    || tdb_from_ut1_offset_seconds.is_some()
                    || tdb_from_tt_offset_seconds.is_some()
                {
                    return Err(
                        "conflicting TDB-TT offset flags: use only one of --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, or --tdb-from-tt-offset-seconds"
                            .to_string(),
                    );
                }
                tdb_from_tt_offset_seconds = Some(parse_signed_seconds(
                    iter.next(),
                    "--tdb-from-tt-offset-seconds",
                )?);
            }
            "--tt-from-tdb-offset-seconds" => {
                if tt_from_tdb_offset_seconds.is_some() {
                    return Err(
                        "conflicting TT-TDB offset flags: use only one --tt-from-tdb-offset-seconds value"
                            .to_string(),
                    );
                }
                tt_from_tdb_offset_seconds = Some(parse_signed_seconds(
                    iter.next(),
                    "--tt-from-tdb-offset-seconds",
                )?);
            }
            "--mean" => {
                if apparentness_explicit {
                    return Err(
                        "conflicting apparentness flags: use only one of --mean or --apparent"
                            .to_string(),
                    );
                }
                apparentness = Apparentness::Mean;
                apparentness_explicit = true;
            }
            "--apparent" => {
                if apparentness_explicit {
                    return Err(
                        "conflicting apparentness flags: use only one of --mean or --apparent"
                            .to_string(),
                    );
                }
                apparentness = Apparentness::Apparent;
                apparentness_explicit = true;
            }
            "--ayanamsa" => {
                let label = iter
                    .next()
                    .ok_or_else(|| "missing value for --ayanamsa".to_string())?;
                zodiac_mode = ZodiacMode::Sidereal {
                    ayanamsa: parse_ayanamsa(label)?,
                };
            }
            "--house-system" => {
                let label = iter
                    .next()
                    .ok_or_else(|| "missing value for --house-system".to_string())?;
                house_system = Some(parse_house_system(label)?);
            }
            "--help" | "-h" => {
                let chart_request_surface = current_request_surface_summary();
                return Ok(format!(
                    "{}\n\nUsage:\n  chart [--jd <julian-day>] [--lat <deg> --lon <deg>] [--tt|--tdb|--utc|--ut1] [--tt-offset-seconds <seconds>|--tt-from-utc-offset-seconds <seconds>|--tt-from-ut1-offset-seconds <seconds>] [--tdb-offset-seconds <seconds>|--tdb-from-utc-offset-seconds <seconds>|--tdb-from-ut1-offset-seconds <seconds>] [--tdb-from-tt-offset-seconds <seconds>] [--tt-from-tdb-offset-seconds <seconds>] [--mean|--apparent] [--ayanamsa <name>] [--house-system <name>] [--body <name> ...]\n\nAyanamsa names may be built-in entries or custom definitions in the form custom:<name>|<epoch-jd>|<offset-degrees> (or custom-definition:<name>|<epoch-jd>|<offset-degrees>). Body names may be built-in bodies such as Sun or Moon, or custom identifiers in the form catalog:designation. {}\n\n{}\n",
                    banner(),
                    chart_request_surface.chart_help_clause(),
                    shared_request_policy_help_block()
                ));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let jd = jd.unwrap_or(2_451_545.0);
    let instant = build_chart_instant(
        jd,
        time_scale,
        ChartInstantConversionFlags {
            tt_offset_seconds,
            tdb_offset_seconds,
            tdb_from_utc_offset_seconds,
            tdb_from_ut1_offset_seconds,
            tdb_from_tt_offset_seconds,
            tt_from_tdb_offset_seconds,
        },
    )?;
    let observer = match (lat, lon) {
        (Some(lat), Some(lon)) => Some(ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(lon),
            None,
        )),
        (None, None) => None,
        _ => return Err("both --lat and --lon must be provided together".to_string()),
    };

    if bodies.is_empty() {
        bodies = default_chart_bodies().to_vec();
    }

    let backend = RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
    ]);
    let engine = ChartEngine::new(backend);
    let mut request = ChartRequest::new(instant)
        .with_bodies(bodies)
        .with_zodiac_mode(zodiac_mode)
        .with_apparentness(apparentness);
    if let Some(observer) = observer {
        request = request.with_observer(observer);
    }
    if let Some(house_system) = house_system {
        request = request.with_house_system(house_system);
    }

    engine
        .chart(&request)
        .map(|chart| chart.to_string())
        .map_err(render_error)
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

fn parse_release_bundle_output_dir<'a>(args: &'a [&'a str]) -> Result<&'a str, String> {
    let mut output_dir: Option<&str> = None;
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
                output_dir = Some(value);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    output_dir.ok_or_else(|| "missing required --out <dir> argument".to_string())
}

enum PackagedArtifactCommand {
    Write { output_path: String },
    Check,
}

fn parse_packaged_artifact_command(args: &[&str]) -> Result<PackagedArtifactCommand, String> {
    match args {
        [] => Err(
            "missing required output path argument; pass a file path, --out <file>, --output <file>, or --check"
                .to_string(),
        ),
        ["--check"] => Ok(PackagedArtifactCommand::Check),
        ["--out"] => Err("missing value for --out".to_string()),
        ["--output"] => Err("missing value for --output".to_string()),
        ["--out", path] | ["--output", path] => Ok(PackagedArtifactCommand::Write {
            output_path: (*path).to_string(),
        }),
        ["--out", _, extra, ..] | ["--output", _, extra, ..] => {
            Err(format!("unknown argument: {extra}"))
        }
        [path] if !path.starts_with('-') => Ok(PackagedArtifactCommand::Write {
            output_path: (*path).to_string(),
        }),
        [other, ..] => Err(format!("unknown argument: {other}")),
    }
}

fn render_packaged_artifact_regeneration(output_path: String) -> Result<String, String> {
    let artifact = regenerate_packaged_artifact();
    let encoded = artifact.encode().map_err(|error| error.to_string())?;
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }
    std::fs::write(&output_path, &encoded)
        .map_err(|error| format!("failed to write {}: {error}", output_path))?;

    Ok(format!(
        "Packaged artifact regenerated\n  path: {}\n  label: {}\n  source: {}\n  checksum: 0x{:016x}\n  bytes: {}\n  {}",
        output_path,
        artifact.header.generation_label,
        artifact.header.source,
        artifact.checksum,
        encoded.len(),
        packaged_artifact_regeneration_summary_for_report(),
    ))
}

fn render_packaged_artifact_regeneration_check() -> Result<String, String> {
    let artifact = regenerate_packaged_artifact();
    let regenerated = artifact.encode().map_err(|error| error.to_string())?;
    let committed = packaged_artifact_bytes();

    if regenerated.as_slice() != committed {
        return Err(format!(
            "packaged artifact regeneration check failed: regenerated {} bytes did not match the checked-in fixture {} bytes",
            regenerated.len(),
            committed.len()
        ));
    }

    Ok(format!(
        "Packaged artifact regeneration check passed\n  label: {}\n  source: {}\n  checksum: 0x{:016x}\n  bytes: {}\n  {}",
        artifact.header.generation_label,
        artifact.header.source,
        artifact.checksum,
        regenerated.len(),
        packaged_artifact_regeneration_summary_for_report(),
    ))
}

fn parse_f64(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let value = value.ok_or_else(|| format!("missing value for {flag}"))?;
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid value for {flag}: {error}"))
}

fn parse_seconds(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let seconds = parse_f64(value, flag)?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(format!(
            "invalid value for {flag}: expected a finite nonnegative number"
        ));
    }

    Ok(seconds)
}

fn parse_signed_seconds(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let seconds = parse_f64(value, flag)?;
    if !seconds.is_finite() {
        return Err(format!(
            "invalid value for {flag}: expected a finite number"
        ));
    }

    Ok(seconds)
}

#[derive(Clone, Copy, Debug, Default)]
struct ChartInstantConversionFlags {
    tt_offset_seconds: Option<f64>,
    tdb_offset_seconds: Option<f64>,
    tdb_from_utc_offset_seconds: Option<f64>,
    tdb_from_ut1_offset_seconds: Option<f64>,
    tdb_from_tt_offset_seconds: Option<f64>,
    tt_from_tdb_offset_seconds: Option<f64>,
}

fn build_chart_instant(
    jd: f64,
    time_scale: TimeScale,
    flags: ChartInstantConversionFlags,
) -> Result<Instant, String> {
    let instant = Instant::new(JulianDay::from_days(jd), time_scale);
    let tt_offset = flags.tt_offset_seconds.map(Duration::from_secs_f64);
    let tdb_offset = flags.tdb_offset_seconds;
    let tdb_from_utc_offset = flags.tdb_from_utc_offset_seconds;
    let tdb_from_ut1_offset = flags.tdb_from_ut1_offset_seconds;
    let tdb_from_tt_offset = flags.tdb_from_tt_offset_seconds;
    let tt_from_tdb_offset = flags.tt_from_tdb_offset_seconds;

    if tdb_offset.is_some()
        && (tdb_from_utc_offset.is_some()
            || tdb_from_ut1_offset.is_some()
            || tdb_from_tt_offset.is_some())
    {
        return Err(
            "conflicting TDB-TT offset flags: use only one of --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, or --tdb-from-tt-offset-seconds"
                .to_string(),
        );
    }
    if tdb_from_utc_offset.is_some()
        && (tdb_from_ut1_offset.is_some() || tdb_from_tt_offset.is_some())
    {
        return Err(
            "conflicting TDB-TT offset flags: use only one of --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, or --tdb-from-tt-offset-seconds"
                .to_string(),
        );
    }
    if tdb_from_ut1_offset.is_some() && tdb_from_tt_offset.is_some() {
        return Err(
            "conflicting TDB-TT offset flags: use only one of --tdb-from-ut1-offset-seconds or --tdb-from-tt-offset-seconds"
                .to_string(),
        );
    }
    match time_scale {
        TimeScale::Utc => {
            if tt_from_tdb_offset.is_some() {
                return Err(
                    "--tt-from-tdb-offset-seconds is only valid when the chart instant is tagged as TDB"
                        .to_string(),
                );
            }
            if tdb_from_ut1_offset.is_some() {
                return Err(
                    "--tdb-from-ut1-offset-seconds is only valid when the chart instant is tagged as UT1"
                        .to_string(),
                );
            }
            if tdb_from_tt_offset.is_some() {
                return Err(
                    "--tdb-from-tt-offset-seconds is only valid when the chart instant is tagged as TT"
                        .to_string(),
                );
            }
            let tt_offset = tt_offset.ok_or_else(|| {
                "missing value for --tt-offset-seconds when the chart instant is tagged as UTC"
                    .to_string()
            })?;
            if let Some(tdb_offset_seconds) = tdb_from_utc_offset.or(tdb_offset) {
                instant
                    .tdb_from_utc_signed(tt_offset, tdb_offset_seconds)
                    .map_err(|error| error.to_string())
            } else {
                instant
                    .tt_from_utc(tt_offset)
                    .map_err(|error| error.to_string())
            }
        }
        TimeScale::Ut1 => {
            if tt_from_tdb_offset.is_some() {
                return Err(
                    "--tt-from-tdb-offset-seconds is only valid when the chart instant is tagged as TDB"
                        .to_string(),
                );
            }
            if tdb_from_utc_offset.is_some() {
                return Err(
                    "--tdb-from-utc-offset-seconds is only valid when the chart instant is tagged as UTC"
                        .to_string(),
                );
            }
            if tdb_from_tt_offset.is_some() {
                return Err(
                    "--tdb-from-tt-offset-seconds is only valid when the chart instant is tagged as TT"
                        .to_string(),
                );
            }
            let tt_offset = tt_offset.ok_or_else(|| {
                "missing value for --tt-offset-seconds when the chart instant is tagged as UT1"
                    .to_string()
            })?;
            if let Some(tdb_offset_seconds) = tdb_from_ut1_offset.or(tdb_offset) {
                instant
                    .tdb_from_ut1_signed(tt_offset, tdb_offset_seconds)
                    .map_err(|error| error.to_string())
            } else {
                instant
                    .tt_from_ut1(tt_offset)
                    .map_err(|error| error.to_string())
            }
        }
        TimeScale::Tt => {
            if tt_offset.is_some() {
                return Err(
                    "--tt-offset-seconds is only valid when the chart instant is tagged as UTC or UT1"
                        .to_string(),
                );
            }
            if tdb_from_utc_offset.is_some() {
                return Err(
                    "--tdb-from-utc-offset-seconds is only valid when the chart instant is tagged as UTC"
                        .to_string(),
                );
            }
            if tdb_from_ut1_offset.is_some() {
                return Err(
                    "--tdb-from-ut1-offset-seconds is only valid when the chart instant is tagged as UT1"
                        .to_string(),
                );
            }
            if tt_from_tdb_offset.is_some() {
                return Err(
                    "--tt-from-tdb-offset-seconds is only valid when the chart instant is tagged as TDB"
                        .to_string(),
                );
            }
            if let Some(tdb_offset_seconds) = tdb_from_tt_offset.or(tdb_offset) {
                instant
                    .tdb_from_tt_signed(tdb_offset_seconds)
                    .map_err(|error| error.to_string())
            } else {
                Ok(instant)
            }
        }
        TimeScale::Tdb => {
            if tt_offset.is_some() {
                return Err(
                    "--tt-offset-seconds is only valid when the chart instant is tagged as UTC or UT1"
                        .to_string(),
                );
            }
            if tdb_offset.is_some() {
                return Err(
                    "--tdb-offset-seconds is only valid when the chart instant is tagged as TT, UTC, or UT1"
                        .to_string(),
                );
            }
            if tdb_from_utc_offset.is_some() {
                return Err(
                    "--tdb-from-utc-offset-seconds is only valid when the chart instant is tagged as UTC"
                        .to_string(),
                );
            }
            if tdb_from_ut1_offset.is_some() {
                return Err(
                    "--tdb-from-ut1-offset-seconds is only valid when the chart instant is tagged as UT1"
                        .to_string(),
                );
            }
            if tdb_from_tt_offset.is_some() {
                return Err(
                    "--tdb-from-tt-offset-seconds is only valid when the chart instant is tagged as TT"
                        .to_string(),
                );
            }
            if let Some(tt_from_tdb_offset_seconds) = tt_from_tdb_offset {
                instant
                    .tt_from_tdb_signed(tt_from_tdb_offset_seconds)
                    .map_err(|error| error.to_string())
            } else {
                Ok(instant)
            }
        }
        _ => Err(format!("unsupported time scale: {}", time_scale)),
    }
}

fn parse_body(value: Option<&str>) -> Result<CelestialBody, String> {
    let value = value.ok_or_else(|| "missing value for --body".to_string())?;
    if let Some(body) = parse_builtin_body(value) {
        return Ok(body);
    }

    parse_custom_body(value)
}

fn parse_builtin_body(value: &str) -> Option<CelestialBody> {
    match value.to_ascii_lowercase().as_str() {
        "sun" => Some(CelestialBody::Sun),
        "moon" => Some(CelestialBody::Moon),
        "mercury" => Some(CelestialBody::Mercury),
        "venus" => Some(CelestialBody::Venus),
        "mars" => Some(CelestialBody::Mars),
        "jupiter" => Some(CelestialBody::Jupiter),
        "saturn" => Some(CelestialBody::Saturn),
        "uranus" => Some(CelestialBody::Uranus),
        "neptune" => Some(CelestialBody::Neptune),
        "pluto" => Some(CelestialBody::Pluto),
        "ceres" => Some(CelestialBody::Ceres),
        "pallas" => Some(CelestialBody::Pallas),
        "juno" => Some(CelestialBody::Juno),
        "vesta" => Some(CelestialBody::Vesta),
        "mean node" | "mean lunar node" => Some(CelestialBody::MeanNode),
        "true node" | "true lunar node" => Some(CelestialBody::TrueNode),
        "mean apogee" => Some(CelestialBody::MeanApogee),
        "true apogee" => Some(CelestialBody::TrueApogee),
        "mean perigee" => Some(CelestialBody::MeanPerigee),
        "true perigee" => Some(CelestialBody::TruePerigee),
        _ => None,
    }
}

fn parse_custom_body(value: &str) -> Result<CelestialBody, String> {
    let (catalog, designation) = value
        .split_once(':')
        .ok_or_else(|| format!("unsupported body name: {value}"))?;

    let custom = CustomBodyId::new(catalog, designation);
    custom.validate().map_err(|error| error.to_string())?;

    Ok(CelestialBody::Custom(custom))
}

fn parse_ayanamsa(value: &str) -> Result<Ayanamsa, String> {
    if let Some(builtin) = resolve_ayanamsa(value) {
        return Ok(builtin);
    }

    if let Some(custom) = parse_custom_ayanamsa(value)? {
        return Ok(custom);
    }

    Err(format!("unsupported ayanamsa name: {value}"))
}

fn parse_custom_ayanamsa(value: &str) -> Result<Option<Ayanamsa>, String> {
    let value = match strip_custom_ayanamsa_prefix(value) {
        Some(value) => value,
        None => return Ok(None),
    };

    let mut parts = value.split('|');
    let name = parts.next().unwrap_or("");
    let epoch_text = parts.next().ok_or_else(|| {
        format!(
            "custom ayanamsa definitions must use custom:<name>|<epoch-jd>|<offset-degrees>: {value}"
        )
    })?;
    let offset_text = parts.next().ok_or_else(|| {
        format!(
            "custom ayanamsa definitions must use custom:<name>|<epoch-jd>|<offset-degrees>: {value}"
        )
    })?;
    if parts.next().is_some() {
        return Err(format!(
            "custom ayanamsa definitions must use custom:<name>|<epoch-jd>|<offset-degrees>: {value}"
        ));
    }
    if name.is_empty() {
        return Err("custom ayanamsa names must not be empty".to_string());
    }

    let epoch = epoch_text
        .parse::<f64>()
        .map_err(|error| format!("invalid custom ayanamsa epoch in {value}: {error}"))?;
    let offset = offset_text
        .parse::<f64>()
        .map_err(|error| format!("invalid custom ayanamsa offset in {value}: {error}"))?;

    let custom = CustomAyanamsa {
        name: name.to_owned(),
        description: Some("Custom ayanamsa definition supplied via the CLI".to_owned()),
        epoch: Some(JulianDay::from_days(epoch)),
        offset_degrees: Some(Angle::from_degrees(offset)),
    };
    custom.validate().map_err(|error| error.to_string())?;

    Ok(Some(Ayanamsa::Custom(custom)))
}

fn strip_custom_ayanamsa_prefix(value: &str) -> Option<&str> {
    strip_case_insensitive_prefix(value, "custom:")
        .or_else(|| strip_case_insensitive_prefix(value, "custom-definition:"))
}

fn strip_case_insensitive_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let head = value.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then_some(&value[prefix.len()..])
}

fn parse_house_system(value: &str) -> Result<HouseSystem, String> {
    resolve_house_system(value).ok_or_else(|| format!("unsupported house system name: {value}"))
}

fn render_error(error: EphemerisError) -> String {
    error.summary_line()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    match render_cli(&arg_refs) {
        Ok(rendered) => println!("{}", rendered),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use pleiades_core::{current_compatibility_profile, current_release_profile_identifiers};
    use pleiades_data::packaged_artifact_generation_manifest_for_report;
    use pleiades_validate::{house_validation_report, house_validation_summary_line_for_report};

    use super::{
        banner, parse_ayanamsa, parse_body, regenerate_packaged_artifact, render_chart, render_cli,
        shared_request_policy_help_block, validate_render_cli, Angle, Ayanamsa, CelestialBody,
        CustomAyanamsa, CustomBodyId, JulianDay,
    };

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = format!(
            "{}-{}-{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after UNIX_EPOCH")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&path).expect("temp dir should be creatable");
        path
    }

    fn packaged_artifact_access_report_line() -> String {
        format!(
            "Packaged-artifact access: {}",
            pleiades_data::packaged_artifact_access_summary()
        )
    }

    fn help_command_names(help: &str) -> std::collections::BTreeSet<String> {
        help.lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                if !line.starts_with("  ") {
                    return None;
                }
                let command = trimmed.split_whitespace().next()?;
                if command.starts_with('-')
                    || !command
                        .chars()
                        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    return None;
                }
                Some(command.to_string())
            })
            .collect()
    }

    #[test]
    fn banner_mentions_package() {
        assert!(banner().contains("pleiades-cli"));
    }

    #[test]
    fn help_text_mentions_tdb_to_tt_retagging_flag() {
        let rendered = render_cli(&["help"]).expect("help should render");
        assert!(rendered.contains("--tt-from-utc-offset-seconds"));
        assert!(rendered.contains("--tt-from-ut1-offset-seconds"));
        assert!(rendered.contains("--tdb-from-utc-offset-seconds"));
        assert!(rendered.contains("--tdb-from-ut1-offset-seconds"));
        assert!(rendered.contains("--tdb-from-tt-offset-seconds"));
        assert!(rendered.contains("--tt-from-tdb-offset-seconds"));
        assert!(rendered.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
        assert!(rendered.contains("reference-high-curvature-summary"));
        assert!(rendered.contains("high-curvature-summary"));
        assert!(rendered.contains("reference-snapshot-2500-major-body-boundary-summary"));
        assert!(rendered.contains("2500-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2500-selected-body-boundary-summary"));
        assert!(rendered.contains("2500-selected-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2453000-major-body-boundary-summary"));
        assert!(rendered.contains("2453000-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451545-major-body-boundary-summary"));
        assert!(rendered.contains("2451545-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451910-major-body-boundary-summary"));
        assert!(rendered.contains("2451910-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451911-major-body-boundary-summary"));
        assert!(rendered.contains("2451911-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451912-major-body-boundary-summary"));
        assert!(rendered.contains("2451912-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451913-major-body-boundary-summary"));
        assert!(rendered.contains("2451913-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451914-major-body-boundary-summary"));
        assert!(rendered.contains("2451914-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451914-major-body-pre-bridge-summary"));
        assert!(rendered.contains("2451914-major-body-pre-bridge-summary"));
        assert!(rendered.contains("reference-snapshot-2451914-major-body-bridge-summary"));
        assert!(rendered.contains("2451914-major-body-bridge-summary"));
        assert!(rendered.contains("reference-snapshot-2451915-major-body-boundary-summary"));
        assert!(rendered.contains("2451915-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451917-major-body-boundary-summary"));
        assert!(rendered.contains("2451917-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451918-major-body-boundary-summary"));
        assert!(rendered.contains("2451918-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451916-major-body-dense-boundary-summary"));
        assert!(rendered.contains("2451916-major-body-dense-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2451916-major-body-interior-summary"));
        assert!(rendered.contains("2451916-major-body-interior-summary"));
        assert!(rendered.contains("reference-snapshot-2451920-major-body-interior-summary"));
        assert!(rendered.contains("2451920-major-body-interior-summary"));
        assert!(rendered.contains("reference-snapshot-1749-major-body-boundary-summary"));
        assert!(rendered.contains("1749-major-body-boundary-summary"));
        assert!(rendered.contains("2360233-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-1800-major-body-boundary-summary"));
        assert!(rendered.contains("1800-major-body-boundary-summary"));
        assert!(rendered.contains("2378499-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-major-body-boundary-summary"));
        assert!(rendered.contains("major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-mars-jupiter-boundary-summary"));
        assert!(rendered.contains("mars-jupiter-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-major-body-boundary-window-summary"));
        assert!(rendered.contains("major-body-boundary-window-summary"));
        assert!(rendered.contains("reference-high-curvature-window-summary"));
        assert!(rendered.contains("high-curvature-window-summary"));
        assert!(rendered.contains("reference-high-curvature-epoch-coverage-summary"));
        assert!(rendered.contains("high-curvature-epoch-coverage-summary"));
        assert!(rendered.contains("reference-snapshot-sparse-boundary-summary"));
        assert!(rendered.contains("sparse-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-pre-bridge-boundary-summary"));
        assert!(rendered.contains("pre-bridge-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-dense-boundary-summary"));
        assert!(rendered.contains("dense-boundary-summary"));
        assert!(rendered.contains("early-major-body-boundary-summary"));
        assert!(rendered.contains("1800-major-body-boundary-summary"));
        assert!(rendered.contains("source-documentation-summary"));
        assert!(rendered
            .contains("source-documentation         Alias for source-documentation-summary"));
        assert!(rendered.contains("source-documentation-health-summary"));
        assert!(rendered.contains(
            "source-documentation-health  Alias for source-documentation-health-summary"
        ));
        assert!(rendered
            .contains("source-audit-summary      Print the compact VSOP87 source audit summary"));
        assert!(rendered.contains("source-audit              Alias for source-audit-summary"));
        assert!(rendered.contains(
            "generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary"
        ));
        assert!(
            rendered.contains("generated-binary-audit    Alias for generated-binary-audit-summary")
        );
        assert!(rendered.contains("time-scale-policy-summary"));
        assert!(rendered.contains("mean-obliquity-frame-round-trip-summary"));
        assert!(rendered.contains(
            "mean-obliquity-frame-round-trip  Alias for mean-obliquity-frame-round-trip-summary"
        ));
        assert!(rendered.contains("production-generation-body-class-coverage-summary"));
        assert!(rendered.contains("production-body-class-coverage-summary"));
        assert!(rendered.contains("production-generation-boundary-request-corpus-summary"));
        assert!(rendered.contains("comparison-snapshot-body-class-coverage-summary"));
        assert!(rendered.contains("comparison-body-class-coverage-summary"));
        assert!(rendered.contains("comparison-corpus-summary"));
        assert!(rendered.contains("comparison-corpus         Alias for comparison-corpus-summary"));
        assert!(rendered.contains("comparison-corpus-release-guard-summary"));
        assert!(rendered.contains(
            "comparison-corpus-release-guard  Alias for comparison-corpus-release-guard-summary"
        ));
        assert!(rendered.contains("comparison-corpus-guard-summary"));
        assert!(rendered.contains("comparison-envelope-summary"));
        assert!(
            rendered.contains("comparison-envelope       Alias for comparison-envelope-summary")
        );
        assert!(rendered.contains("comparison-tolerance-summary"));
        assert!(rendered.contains(
            "comparison-tolerance-summary  Alias for comparison-tolerance-policy-summary"
        ));
        assert!(rendered.contains("comparison-body-class-tolerance-summary"));
        assert!(rendered.contains(
            "comparison-body-class-tolerance-summary  Print the compact comparison body-class tolerance summary"
        ));
        assert!(rendered.contains(
            "comparison-body-class-tolerance  Alias for comparison-body-class-tolerance-summary"
        ));
        assert!(rendered.contains("benchmark-corpus-summary"));
        assert!(rendered.contains("comparison-snapshot-summary"));
        assert!(rendered.contains("comparison-snapshot-batch-parity-summary"));
        assert!(rendered.contains("reference-snapshot-body-class-coverage-summary"));
        assert!(rendered.contains("reference-body-class-coverage-summary"));
        assert!(rendered.contains("reference-snapshot-summary"));
        assert!(rendered.contains("reference-snapshot-batch-parity-summary"));
        assert!(rendered.contains("reference-snapshot-equatorial-parity-summary"));
        assert!(rendered.contains("workspace-audit-summary"));
        assert!(rendered.contains("native-dependency-audit-summary"));
        assert!(rendered.contains("independent-holdout-source-window-summary"));
        assert!(rendered.contains("independent-holdout-body-class-coverage-summary"));
        assert!(rendered.contains("holdout-body-class-coverage-summary"));
        assert!(rendered.contains("independent-holdout-batch-parity-summary"));
        assert!(rendered.contains("independent-holdout-equatorial-parity-summary"));
        assert!(rendered.contains("lunar-theory-summary"));
        assert!(rendered.contains("lunar-theory-request-policy-summary"));
        assert!(rendered.contains(
            "lunar-theory-request-policy  Alias for lunar-theory-request-policy-summary"
        ));
        assert!(
            rendered.contains("request-policy-summary    Print the compact request-policy summary")
        );
        assert!(rendered.contains("request-policy           Alias for request-policy-summary"));
        assert!(rendered
            .contains("request-semantics-summary  Print the compact request-semantics summary"));
        assert!(rendered.contains("request-semantics        Alias for request-semantics-summary"));
        assert!(rendered.contains("lunar-theory-frame-treatment-summary"));
        assert!(rendered.contains(
            "lunar-theory-frame-treatment  Alias for lunar-theory-frame-treatment-summary"
        ));
        assert!(rendered.contains("lunar-theory-limitations-summary"));
        assert!(rendered.contains("lunar-theory-capability-summary"));
        assert!(rendered.contains("lunar-theory-source-summary"));
        assert!(rendered.contains("lunar-theory-catalog-summary"));
        assert!(rendered.contains("lunar-theory-catalog-validation-summary"));
        assert!(
            rendered.contains("lunar-theory-catalog      Alias for lunar-theory-catalog-summary")
        );
        assert!(rendered.contains(
            "lunar-theory-catalog-validation  Alias for lunar-theory-catalog-validation-summary"
        ));
        assert!(rendered.contains("observer-policy-summary"));
        assert!(rendered.contains("apparentness-policy-summary"));
        assert!(rendered.contains("compare-backends-audit"));
        assert!(rendered.contains("Caller-supplied signed TDB-TT offset for TT-tagged instants"));
        assert!(rendered.contains("Caller-supplied signed TT-TDB offset for TDB-tagged instants"));
    }

    #[test]
    fn shared_command_help_is_kept_in_sync_with_the_validation_binary() {
        let cli_help = render_cli(&["help"]).expect("cli help should render");
        let validation_help =
            validate_render_cli(&["help"]).expect("validation help should render");

        let mut cli_commands = help_command_names(&cli_help);
        let validation_commands = help_command_names(&validation_help);

        assert!(
            cli_commands.remove("chart"),
            "cli should remain the only binary with the chart command"
        );
        assert_eq!(cli_commands, validation_commands);
    }

    #[test]
    fn compare_backends_command_renders_the_comparison_report() {
        let rendered = render_cli(&["compare-backends"]).expect("compare-backends should render");
        assert!(rendered.contains("Comparison report"));
        assert!(rendered.contains("Comparison corpus"));
        assert!(rendered.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day remains in the release-grade comparison window"));
        assert!(rendered.contains("epoch labels:"));
        assert!(rendered.contains("Reference backend:"));
        assert!(rendered.contains("Candidate backend:"));
        assert!(rendered.contains("Samples"));
    }

    #[test]
    fn compare_backends_audit_command_renders_the_comparison_report() {
        let rendered =
            render_cli(&["compare-backends-audit"]).expect("compare-backends-audit should render");
        assert!(rendered.contains("Comparison tolerance audit"));
        assert!(rendered.contains("result: clean"));
        assert!(rendered.contains("within tolerance bodies: 9"));
        assert!(rendered.contains("outside tolerance bodies: 0"));
    }

    #[test]
    fn compare_backends_command_rejects_extra_arguments() {
        let error = render_cli(&["compare-backends", "extra"])
            .expect_err("compare-backends should reject extra arguments");
        assert_eq!(error, "compare-backends does not accept extra arguments");
    }

    #[test]
    fn compare_backends_audit_command_rejects_extra_arguments() {
        let error = render_cli(&["compare-backends-audit", "extra"])
            .expect_err("compare-backends-audit should reject extra arguments");
        assert_eq!(
            error,
            "compare-backends-audit does not accept extra arguments"
        );
    }

    #[test]
    fn reference_snapshot_1600_selected_body_boundary_aliases_render_the_same_reports() {
        let boundary_1600 = render_cli(&["reference-snapshot-1600-selected-body-boundary-summary"])
            .expect("1600 selected-body boundary summary should render");
        assert!(boundary_1600.contains("Reference 1600 selected-body boundary evidence:"));
        assert!(boundary_1600.contains("JD 2305457.5 (TDB)"));
        let boundary_1600_alias = render_cli(&["1600-selected-body-boundary-summary"])
            .expect("1600 selected-body boundary alias should render");
        assert_eq!(boundary_1600_alias, boundary_1600);
    }

    #[test]
    fn reference_snapshot_1750_selected_body_boundary_aliases_render_the_same_reports() {
        let boundary_1750 = render_cli(&["reference-snapshot-1750-selected-body-boundary-summary"])
            .expect("1750 selected-body boundary summary should render");
        assert!(boundary_1750.contains("Reference 1750 selected-body boundary evidence:"));
        assert!(boundary_1750.contains("JD 2360234.5 (TDB)"));
        let boundary_1750_alias = render_cli(&["1750-selected-body-boundary-summary"])
            .expect("1750 selected-body boundary alias should render");
        assert_eq!(boundary_1750_alias, boundary_1750);
    }

    #[test]
    fn reference_snapshot_1749_major_body_boundary_aliases_render_the_same_reports() {
        let boundary_1749 = render_cli(&["reference-snapshot-1749-major-body-boundary-summary"])
            .expect("1749 major-body boundary summary should render");
        assert!(boundary_1749.contains("Reference 1749 major-body boundary evidence:"));
        assert!(boundary_1749.contains("JD 2360233.5 (TDB)"));
        let boundary_1749_alias = render_cli(&["1749-major-body-boundary-summary"])
            .expect("1749 major-body boundary alias should render");
        assert_eq!(boundary_1749_alias, boundary_1749);
        let boundary_2360233_alias = render_cli(&["2360233-major-body-boundary-summary"])
            .expect("2360233 major-body boundary alias should render");
        assert_eq!(boundary_2360233_alias, boundary_1749);
    }

    #[test]
    fn early_and_1800_major_body_boundary_aliases_render_the_same_reports() {
        let early = render_cli(&["reference-snapshot-early-major-body-boundary-summary"])
            .expect("early major-body boundary summary should render");
        let early_alias = render_cli(&["early-major-body-boundary-summary"])
            .expect("early major-body boundary alias should render");
        assert_eq!(early_alias, early);

        let boundary_1800 = render_cli(&["reference-snapshot-1800-major-body-boundary-summary"])
            .expect("1800 major-body boundary summary should render");
        let boundary_1800_alias = render_cli(&["1800-major-body-boundary-summary"])
            .expect("1800 major-body boundary alias should render");
        assert_eq!(boundary_1800_alias, boundary_1800);
        let boundary_2378499_alias = render_cli(&["2378499-major-body-boundary-summary"])
            .expect("2378499 major-body boundary alias should render");
        assert_eq!(boundary_2378499_alias, boundary_1800);
    }

    #[test]
    fn reference_snapshot_2500_selected_body_boundary_aliases_render_the_same_reports() {
        let boundary_2500 = render_cli(&["reference-snapshot-2500-selected-body-boundary-summary"])
            .expect("2500 selected-body boundary summary should render");
        assert!(boundary_2500.contains("Reference 2500 selected-body boundary evidence:"));
        assert!(boundary_2500.contains("JD 2634167.0 (TDB)"));
        let boundary_2500_alias = render_cli(&["2500-selected-body-boundary-summary"])
            .expect("2500 selected-body boundary alias should render");
        assert_eq!(boundary_2500_alias, boundary_2500);
    }

    #[test]
    fn reference_snapshot_2500_major_body_boundary_aliases_render_the_same_reports() {
        let boundary_2500 = render_cli(&["reference-snapshot-2500-major-body-boundary-summary"])
            .expect("2500 major-body boundary summary should render");
        assert!(boundary_2500.contains("Reference 2500 major-body boundary evidence:"));
        let boundary_2500_alias = render_cli(&["2500-major-body-boundary-summary"])
            .expect("2500 major-body boundary alias should render");
        assert_eq!(boundary_2500_alias, boundary_2500);
    }

    #[test]
    fn reference_snapshot_2400000_major_body_boundary_aliases_render_the_same_reports() {
        let boundary_2400000 =
            render_cli(&["reference-snapshot-2400000-major-body-boundary-summary"])
                .expect("2400000 major-body boundary summary should render");
        assert!(boundary_2400000.contains("Reference 2400000 major-body boundary evidence:"));
        assert!(boundary_2400000.contains("JD 2400000.0 (TDB)"));
        let boundary_2400000_alias = render_cli(&["2400000-major-body-boundary-summary"])
            .expect("2400000 major-body boundary alias should render");
        assert_eq!(boundary_2400000_alias, boundary_2400000);
    }

    #[test]
    fn reference_snapshot_2451545_major_body_boundary_aliases_render_the_same_reports() {
        let boundary_2451545 =
            render_cli(&["reference-snapshot-2451545-major-body-boundary-summary"])
                .expect("2451545 major-body boundary summary should render");
        assert!(boundary_2451545.contains("Reference 2451545 major-body boundary evidence:"));
        assert!(boundary_2451545.contains("JD 2451545.0 (TDB)"));
        let boundary_2451545_alias = render_cli(&["2451545-major-body-boundary-summary"])
            .expect("2451545 major-body boundary alias should render");
        assert_eq!(boundary_2451545_alias, boundary_2451545);
    }

    #[test]
    fn reference_snapshot_2453000_major_body_boundary_aliases_render_the_same_reports() {
        let boundary_2453000 =
            render_cli(&["reference-snapshot-2453000-major-body-boundary-summary"])
                .expect("2453000 major-body boundary summary should render");
        assert!(boundary_2453000.contains("Reference 2453000 major-body boundary evidence:"));
        let boundary_2453000_alias = render_cli(&["2453000-major-body-boundary-summary"])
            .expect("2453000 major-body boundary alias should render");
        assert_eq!(boundary_2453000_alias, boundary_2453000);
    }

    #[test]
    fn reference_snapshot_1500_1750_1900_and_2360234_aliases_render_the_same_reports() {
        let boundary_1500 = render_cli(&["reference-snapshot-1500-selected-body-boundary-summary"])
            .expect("1500 selected-body boundary summary should render");
        assert!(boundary_1500.contains("Reference 1500 selected-body boundary evidence:"));
        assert!(boundary_1500.contains("JD 2268932.5 (TDB)"));
        assert_eq!(
            render_cli(&["1500-selected-body-boundary-summary"])
                .expect("1500 selected-body boundary alias should render"),
            boundary_1500
        );

        let interior_1750 = render_cli(&["reference-snapshot-1750-major-body-interior-summary"])
            .expect("1750 major-body interior summary should render");
        assert!(interior_1750.contains("Reference 1750 major-body interior comparison evidence:"));
        assert!(interior_1750.contains("JD 2360234.5 (TDB)"));
        assert_eq!(
            render_cli(&["1750-major-body-interior-summary"])
                .expect("1750 major-body interior alias should render"),
            interior_1750
        );

        let boundary_1900 = render_cli(&["reference-snapshot-1900-selected-body-boundary-summary"])
            .expect("1900 selected-body boundary summary should render");
        assert!(boundary_1900.contains("Reference 1900 selected-body boundary evidence:"));
        assert!(boundary_1900.contains("JD 2415020.5 (TDB)"));
        assert_eq!(
            render_cli(&["1900-selected-body-boundary-summary"])
                .expect("1900 selected-body boundary alias should render"),
            boundary_1900
        );

        let interior_2360234 =
            render_cli(&["reference-snapshot-2360234-major-body-interior-summary"])
                .expect("2360234 major-body interior summary should render");
        assert!(
            interior_2360234.contains("Reference 2360234 major-body interior comparison evidence:")
        );
        assert!(interior_2360234.contains("JD 2360234.5 (TDB)"));
        assert_eq!(
            render_cli(&["2360234-major-body-interior-summary"])
                .expect("2360234 major-body interior alias should render"),
            interior_2360234
        );
    }

    #[test]
    fn reference_snapshot_2451916_major_body_interior_aliases_render_the_same_reports() {
        let interior_2451916 =
            render_cli(&["reference-snapshot-2451916-major-body-interior-summary"])
                .expect("2451916 major-body interior summary should render");
        assert!(interior_2451916.contains("Reference 2451916 major-body interior evidence:"));
        assert!(interior_2451916.contains("JD 2451916.0 (TDB)"));
        let interior_2451916_alias = render_cli(&["2451916-major-body-interior-summary"])
            .expect("2451916 major-body interior alias should render");
        assert_eq!(interior_2451916_alias, interior_2451916);
    }

    #[test]
    fn reference_snapshot_2451912_2451913_2451914_and_2451918_major_body_boundary_aliases_render_the_same_reports(
    ) {
        let boundary_2451912 =
            render_cli(&["reference-snapshot-2451912-major-body-boundary-summary"])
                .expect("2451912 major-body boundary summary should render");
        assert!(boundary_2451912.contains("Reference 2451912 major-body boundary evidence:"));
        assert!(boundary_2451912.contains("JD 2451912.5 (TDB)"));
        assert_eq!(
            render_cli(&["2451912-major-body-boundary-summary"])
                .expect("2451912 major-body boundary alias should render"),
            boundary_2451912
        );

        let boundary_2451913 =
            render_cli(&["reference-snapshot-2451913-major-body-boundary-summary"])
                .expect("2451913 major-body boundary summary should render");
        assert!(boundary_2451913.contains("Reference 2451913 major-body boundary evidence:"));
        assert!(boundary_2451913.contains("JD 2451913.5 (TDB)"));
        assert_eq!(
            render_cli(&["2451913-major-body-boundary-summary"])
                .expect("2451913 major-body boundary alias should render"),
            boundary_2451913
        );

        let boundary_2451914 =
            render_cli(&["reference-snapshot-2451914-major-body-boundary-summary"])
                .expect("2451914 major-body boundary summary should render");
        assert!(boundary_2451914.contains("Reference 2451914 major-body boundary evidence:"));
        assert!(boundary_2451914.contains("JD 2451914.5 (TDB)"));
        assert_eq!(
            render_cli(&["2451914-major-body-boundary-summary"])
                .expect("2451914 major-body boundary alias should render"),
            boundary_2451914
        );

        let boundary_2451918 =
            render_cli(&["reference-snapshot-2451918-major-body-boundary-summary"])
                .expect("2451918 major-body boundary summary should render");
        assert!(boundary_2451918.contains("Reference 2451918 major-body boundary evidence:"));
        assert!(boundary_2451918.contains("JD 2451918.5 (TDB)"));
        assert_eq!(
            render_cli(&["2451918-major-body-boundary-summary"])
                .expect("2451918 major-body boundary alias should render"),
            boundary_2451918
        );
    }

    #[test]
    fn reference_snapshot_2451914_pre_bridge_2451914_bridge_2451915_bridge_and_2451916_dense_boundary_aliases_render_the_same_reports(
    ) {
        let pre_bridge = render_cli(&["reference-snapshot-2451914-major-body-pre-bridge-summary"])
            .expect("2451914 pre-bridge summary should render");
        assert!(pre_bridge.contains("Reference snapshot pre-bridge boundary day:"));
        assert!(pre_bridge.contains("JD 2451914.5 (TDB)"));
        assert_eq!(
            render_cli(&["2451914-major-body-pre-bridge-summary"])
                .expect("2451914 pre-bridge alias should render"),
            pre_bridge
        );

        let bridge_day = render_cli(&["reference-snapshot-2451914-major-body-bridge-summary"])
            .expect("2451914 bridge summary should render");
        assert!(bridge_day.contains("Reference snapshot bridge day:"));
        assert!(bridge_day.contains("JD 2451914.0 (TDB)"));
        assert_eq!(
            render_cli(&["2451914-major-body-bridge-summary"])
                .expect("2451914 bridge alias should render"),
            bridge_day
        );

        let bridge_2451915 = render_cli(&["reference-snapshot-2451915-major-body-bridge-summary"])
            .expect("2451915 bridge summary should render");
        assert!(bridge_2451915.contains("Reference major-body bridge evidence:"));
        assert!(bridge_2451915.contains("JD 2451915.0 (TDB)"));
        assert_eq!(
            render_cli(&["2451915-major-body-bridge-summary"])
                .expect("2451915 bridge alias should render"),
            bridge_2451915
        );
        assert_eq!(
            render_cli(&["bridge-summary"]).expect("bridge alias should render"),
            bridge_2451915
        );
        assert_eq!(
            render_cli(&["bridge-summary", "extra"])
                .expect_err("bridge alias should reject extra arguments"),
            "bridge-summary does not accept extra arguments"
        );

        let dense_boundary =
            render_cli(&["reference-snapshot-2451916-major-body-dense-boundary-summary"])
                .expect("2451916 dense boundary summary should render");
        assert!(dense_boundary.contains("Reference 2451916 major-body dense boundary evidence:"));
        assert!(dense_boundary.contains("JD 2451916.5 (TDB)"));
        assert_eq!(
            render_cli(&["2451916-major-body-dense-boundary-summary"])
                .expect("2451916 dense boundary alias should render"),
            dense_boundary
        );
    }

    #[test]
    fn reference_snapshot_2451917_major_body_boundary_aliases_render_the_same_reports() {
        let boundary_2451917 =
            render_cli(&["reference-snapshot-2451917-major-body-boundary-summary"])
                .expect("2451917 major-body boundary summary should render");
        assert!(boundary_2451917.contains("Reference 2451917 major-body boundary evidence:"));
        assert!(boundary_2451917.contains("JD 2451917.5 (TDB)"));
        let boundary_2451917_alias = render_cli(&["2451917-major-body-boundary-summary"])
            .expect("2451917 major-body boundary alias should render");
        assert_eq!(boundary_2451917_alias, boundary_2451917);

        let bridge_2451917 = render_cli(&["reference-snapshot-2451917-major-body-bridge-summary"])
            .expect("2451917 major-body bridge summary should render");
        assert!(bridge_2451917.contains("Reference 2451917 major-body bridge evidence:"));
        assert!(bridge_2451917.contains("JD 2451917.0 (TDB)"));
        assert_eq!(
            render_cli(&["2451917-major-body-bridge-summary"])
                .expect("2451917 major-body bridge alias should render"),
            bridge_2451917
        );
    }

    #[test]
    fn reference_snapshot_2451919_major_body_boundary_aliases_render_the_same_reports() {
        let boundary_2451919 =
            render_cli(&["reference-snapshot-2451919-major-body-boundary-summary"])
                .expect("2451919 major-body boundary summary should render");
        assert!(boundary_2451919.contains("Reference 2451919 major-body boundary evidence:"));
        assert!(boundary_2451919.contains("JD 2451919.5 (TDB)"));
        let boundary_2451919_alias = render_cli(&["2451919-major-body-boundary-summary"])
            .expect("2451919 major-body boundary alias should render");
        assert_eq!(boundary_2451919_alias, boundary_2451919);
    }

    #[test]
    fn reference_snapshot_2451920_major_body_interior_aliases_render_the_same_reports() {
        let interior_2451920 =
            render_cli(&["reference-snapshot-2451920-major-body-interior-summary"])
                .expect("2451920 major-body interior summary should render");
        assert!(interior_2451920.contains("Reference 2451920 major-body interior evidence:"));
        assert!(interior_2451920.contains("JD 2451920.5 (TDB)"));
        let interior_2451920_alias = render_cli(&["2451920-major-body-interior-summary"])
            .expect("2451920 major-body interior alias should render");
        assert_eq!(interior_2451920_alias, interior_2451920);
    }

    #[test]
    fn profile_command_renders_catalogs() {
        let rendered = render_cli(&["compatibility-profile"]).expect("profile should render");
        let release_profiles = current_release_profile_identifiers();
        assert!(rendered.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains("Target compatibility catalog:"));
        assert!(rendered.contains("Baseline compatibility milestone:"));
        assert!(rendered.contains("Release-specific coverage beyond baseline:"));
        assert!(rendered.contains("Topocentric"));
        assert!(rendered.contains("Meridian house system"));
        assert!(rendered.contains("Horizon house system"));
        assert!(rendered.contains("Horizontal house system"));
        assert!(rendered.contains("Azimuth house system"));
        assert!(rendered.contains("Azimuthal house system"));
        assert!(rendered.contains("Whole Sign system"));
        assert!(rendered.contains("Whole Sign house system"));
        assert!(rendered.contains("Whole Sign (house 1 = Aries)"));
        assert!(rendered.contains("Whole-sign"));
        assert!(rendered.contains("Carter's poli-equatorial"));
        assert!(rendered.contains("Poli-equatorial"));
        assert!(rendered.contains("horizon/azimuth"));
        assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
        assert!(rendered.contains("Zariel"));
        assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
        assert!(rendered.contains("Equal (cusp 1 = Asc)"));
        assert!(rendered.contains("Equal from MC"));
        assert!(rendered.contains("Equal (1=Aries) table of houses"));
        assert!(rendered.contains("Equal/MC = 10th"));
        assert!(rendered.contains("Equal Midheaven table of houses"));
        assert!(rendered.contains("Equal Midheaven house system"));
        assert!(rendered.contains("Vehlow equal"));
        assert!(rendered.contains("Equal/1=0 Aries"));
        assert!(rendered.contains("Equal (cusp 1 = 0° Aries)"));
        assert!(rendered.contains("WvA"));
        assert!(rendered.contains("Gauquelin table of sectors"));
        assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
        assert!(rendered.contains("Pullen SD (Sinusoidal Delta)"));
        assert!(rendered.contains("Makransky Sunshine"));
        assert!(rendered.contains("Sunshine table of houses, by Bob Makransky"));
        assert!(rendered.contains("Treindl Sunshine"));
        assert!(rendered.contains("Y APC houses"));
        assert!(rendered.contains("Wang"));
        assert!(rendered.contains("Aries houses"));
        assert!(rendered.contains("Fagan/Bradley"));
        assert!(rendered.contains("Usha Shashi"));
        assert!(rendered.contains("JN Bhasin"));
        assert!(rendered.contains("X, Meridian houses, Meridian table of houses, Meridian house system, ARMC, Axial Rotation, Axial rotation system, Zariel, X axial rotation system/ Meridian houses -> Meridian"));
        assert!(rendered.contains("Target ayanamsa catalog:"));
        assert!(rendered.contains("Alias mappings for built-in house systems:"));
        assert!(rendered.contains("Source-label aliases for built-in house systems:"));
        assert!(rendered.contains("Alias mappings for built-in ayanamsas:"));
        assert!(rendered.contains("Coverage summary:"));
        assert!(rendered.contains("ayanamsa sidereal metadata:"));
        assert!(rendered.contains("J2000"));
        assert!(rendered.contains("True Pushya"));
        assert!(rendered.contains("Djwhal Khul"));
        assert!(rendered.contains("True Revati"));
        assert!(rendered.contains("Babylonian (Eta Piscium)"));
        assert!(rendered.contains(
            "Babylonian/Kugler 2, Babylonian Kugler 2, Babylonian 2 -> Babylonian (Kugler 2)"
        ));
        assert!(rendered.contains(
            "Babylonian/Kugler 3, Babylonian Kugler 3, Babylonian 3 -> Babylonian (Kugler 3)"
        ));
        assert!(rendered.contains("Galactic Equator (Mula)"));
        assert!(rendered.contains("True Mula (Chandra Hari)"));
        assert!(rendered.contains("Galactic Equator (Fiorenza)"));
        assert!(rendered.contains("Galactic Equator (True)"));
        assert!(rendered.contains("Galactic Equator mid-Mula, Mula galactic equator, Galactic equator Mula -> Galactic Equator (Mula)"));
        assert!(rendered.contains("True galactic equator"));
        assert!(rendered.contains("Galactic equator true"));
        assert!(rendered.contains("Galactic Equator (IAU 1958)"));
        assert!(rendered.contains("Dhruva Galactic Center (Middle Mula)"));
        assert!(rendered.contains("Nick Anthony Fiorenza"));
        assert!(rendered.contains("Galactic Center (Cochrane)"));
        assert!(rendered.contains("Gal. Center = 0 Cap"));
        assert!(rendered.contains("Cochrane (Gal.Center = 0 Cap)"));
        assert!(rendered.contains("Galactic Center (Mardyks)"));
        assert!(rendered.contains("Skydram (Mardyks)"));
        assert!(rendered.contains("Mula Wilhelm"));
        assert!(rendered.contains("Wilhelm"));
        assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
        assert!(rendered.contains("Galactic Center (Rgilbrand)"));
        assert!(rendered.contains("Galactic Center (Gil Brand)"));
        assert!(rendered.contains("Gil Brand"));
        assert!(rendered.contains("P.V.R. Narasimha Rao"));
        assert!(rendered.contains("Pullen SR (Sinusoidal Ratio) table of houses"));
        assert!(rendered.contains("True Citra Paksha"));
        assert!(rendered.contains("True Chitra Paksha"));
        assert!(rendered.contains("True Chitrapaksha"));
        assert!(rendered.contains("Babylonian (True Geoc)"));
        assert!(rendered.contains("Babylonian (True Topc)"));
        assert!(rendered.contains("Babylonian (True Obs)"));
        assert!(rendered.contains("Lahiri (VP285)"));
        assert!(rendered.contains("Krishnamurti (VP291)"));
        assert!(rendered.contains("Lahiri (ICRC)"));
        assert!(rendered.contains("Lahiri (1940)"));
        assert!(rendered.contains("Udayagiri"));
        assert!(rendered.contains("Valens Moon"));
        assert!(rendered.contains("Babylonian (House Obs)"));
        assert!(rendered.contains("B. V. Raman"));
        assert!(rendered.contains("Raman Ayanamsha"));
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
    }

    #[test]
    fn backend_matrix_command_renders_the_implemented_catalog() {
        let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
        assert!(rendered.contains("Implemented backend matrices"));
        assert!(rendered.contains("JPL snapshot reference backend"));
        assert!(rendered.contains("expected error classes:"));
        assert!(rendered.contains("required external data files:"));
        assert!(rendered.contains("VSOP87 planetary backend"));
        assert!(rendered.contains("ELP lunar backend"));
        assert!(rendered.contains("Packaged data backend"));
        assert!(rendered.contains("Composite routed backend"));
    }

    #[test]
    fn summary_commands_render_compact_reports() {
        let release_profiles = current_release_profile_identifiers();
        let profile = current_compatibility_profile();

        let compatibility = render_cli(&["compatibility-profile-summary"])
            .expect("compatibility summary should render");
        assert_eq!(render_cli(&["profile-summary"]).unwrap(), compatibility);
        assert!(compatibility.contains("Compatibility profile summary"));
        assert!(compatibility.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(compatibility.contains("House systems: 25 total"));
        assert!(compatibility.contains(&format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        )));
        assert!(compatibility
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(compatibility.lines().any(|line| {
            line == "Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
        }));
        assert!(compatibility.contains("Release notes summary: release-notes-summary"));
        assert!(compatibility.contains("Release summary: release-summary"));
        assert!(compatibility.contains("Release checklist summary: release-checklist-summary"));
        assert!(compatibility.contains("Release bundle verification: verify-release-bundle"));
        let caveats = render_cli(&["compatibility-caveats-summary"])
            .expect("compatibility caveats summary should render");
        assert!(caveats.contains("Compatibility caveats summary"));
        assert!(caveats.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(caveats.contains("Compatibility caveats: 2"));
        assert!(caveats.contains("Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"));
        assert!(caveats.contains("Descriptor-only ayanamsa labels: 6 (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs))"));
        assert!(caveats.contains(profile.known_gaps[0]));
        assert!(caveats.contains(profile.known_gaps[1]));
        assert_eq!(render_cli(&["compatibility-caveats"]).unwrap(), caveats);
        assert_eq!(
            render_cli(&["compatibility-caveats-summary", "extra"]).unwrap_err(),
            "compatibility-caveats-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["compatibility-caveats", "extra"]).unwrap_err(),
            "compatibility-caveats does not accept extra arguments"
        );
        assert!(compatibility.contains("Babylonian (Eta Piscium)"));
        assert!(compatibility.contains("Galactic Equator (Mula)"));
        assert!(compatibility.contains("Galactic Equator (Fiorenza)"));
        assert!(compatibility.contains("JN Bhasin"));
        assert!(compatibility.contains("Lahiri (ICRC)"));
        assert!(compatibility.contains("Udayagiri"));
        assert!(compatibility.contains("Valens Moon"));
        assert!(compatibility.contains("Babylonian (House Obs)"));
        assert!(compatibility.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));

        let verification = render_cli(&["verify-compatibility-profile"])
            .expect("compatibility profile verification should render");
        assert!(verification.contains("Compatibility profile verification"));
        assert!(verification.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(verification.contains("House systems verified:"));
        assert!(verification.contains(&format!(
            "House code aliases verified: {} short-form labels",
            profile.house_code_alias_count()
        )));
        assert!(verification.contains("Ayanamsas verified:"));
        assert!(verification.contains(
            "House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"
        ));
        assert!(verification.contains(&format!(
            "Custom-definition label names verified: {}",
            profile.custom_definition_labels.join(", ")
        )));
        assert!(verification.contains(&format!(
            "Custom-definition ayanamsa labels verified: {} labels, all remain custom-definition territory",
            profile.custom_definition_ayanamsa_labels().len()
        )));
        assert!(verification.contains(&format!(
            "Custom-definition ayanamsa label names verified: {}",
            profile.custom_definition_ayanamsa_labels().join(", ")
        )));
        assert!(verification.contains("Ayanamsa reference metadata verified: "));
        assert!(verification.contains(
            "Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"
        ));

        let backend_matrix =
            render_cli(&["backend-matrix-summary"]).expect("backend matrix summary should render");
        assert!(backend_matrix.contains("Backend matrix summary"));
        assert!(backend_matrix.contains("Backends: 5"));
        assert!(backend_matrix.contains("Accuracy classes: Exact: 1"));
        assert!(backend_matrix.contains("Reference snapshot dense boundary day:"));
        assert!(backend_matrix.contains("Reference major-body bridge evidence:"));
        assert!(backend_matrix.contains("Selected asteroid bridge evidence:"));
        assert!(backend_matrix.contains("Release notes summary: release-notes-summary"));
        assert!(backend_matrix
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(backend_matrix.contains("Release bundle verification: verify-release-bundle"));
        assert!(backend_matrix
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(backend_matrix.contains("Release checklist summary: release-checklist-summary"));

        let comparison_corpus = render_cli(&["comparison-corpus-summary"])
            .expect("comparison corpus summary should render");
        assert!(comparison_corpus.contains("Comparison corpus summary"));
        assert!(comparison_corpus.contains("name: JPL Horizons release-grade comparison window"));
        assert!(comparison_corpus.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day remains in the release-grade comparison window"));
        assert_eq!(
            comparison_corpus,
            validate_render_cli(&["comparison-corpus-summary"])
                .expect("comparison corpus summary should match validation output")
        );
        assert_eq!(
            render_cli(&["comparison-corpus"]).expect("comparison corpus alias should render"),
            comparison_corpus
        );
        assert_eq!(
            validate_render_cli(&["comparison-corpus"])
                .expect("comparison corpus alias should match validation output"),
            comparison_corpus
        );
        assert_eq!(
            render_cli(&["comparison-corpus-summary", "extra"])
                .expect_err("comparison corpus summary should reject extra arguments"),
            "comparison-corpus-summary does not accept extra arguments"
        );

        let comparison_guard = render_cli(&["comparison-corpus-release-guard-summary"])
            .expect("comparison corpus release guard summary should render");
        assert!(comparison_guard.contains("Comparison corpus release-grade guard summary"));
        assert!(comparison_guard.contains("Release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day remains in the release-grade comparison window"));
        assert_eq!(
            comparison_guard,
            validate_render_cli(&["comparison-corpus-release-guard-summary"])
                .expect("comparison corpus release guard summary should match validation output")
        );
        assert_eq!(
            render_cli(&["comparison-corpus-release-guard"])
                .expect("comparison corpus release guard short alias should render"),
            comparison_guard
        );
        assert_eq!(
            render_cli(&["comparison-corpus-guard-summary"])
                .expect("comparison corpus guard alias should render"),
            comparison_guard
        );
        assert_eq!(
            render_cli(&["comparison-corpus-guard"])
                .expect("comparison corpus guard short alias should render"),
            comparison_guard
        );
        assert_eq!(
            render_cli(&["comparison-corpus-release-guard-summary", "extra"]).expect_err(
                "comparison corpus release guard summary should reject extra arguments"
            ),
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

        let benchmark_corpus = render_cli(&["benchmark-corpus-summary"])
            .expect("benchmark corpus summary should render");
        assert!(benchmark_corpus.contains("Benchmark corpus summary"));
        assert!(benchmark_corpus.contains("name: Representative 1500-2500 window"));
        assert!(benchmark_corpus.contains("epoch labels: JD 2268559.0 (TT)"));
        assert!(benchmark_corpus.contains("JD 2451545.0 (TT)"));
        assert!(benchmark_corpus.contains("JD 2634532.0 (TT)"));
        assert_eq!(
            benchmark_corpus,
            validate_render_cli(&["benchmark-corpus-summary"])
                .expect("benchmark corpus summary should match validation output")
        );
        assert_eq!(
            render_cli(&["benchmark-corpus-summary", "extra"])
                .expect_err("benchmark corpus summary should reject extra arguments"),
            "benchmark-corpus-summary does not accept extra arguments"
        );

        let api_stability =
            render_cli(&["api-stability-summary"]).expect("api stability summary should render");
        assert!(api_stability.contains("API stability summary"));
        assert!(api_stability.contains("Summary line: API stability posture:"));
        assert!(api_stability.contains("Stable surfaces: 6"));
        assert!(api_stability.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(api_stability.contains("Release notes summary: release-notes-summary"));
        assert!(api_stability.contains("Release checklist summary: release-checklist-summary"));
        assert!(api_stability.contains("Release bundle verification: verify-release-bundle"));

        let release_notes = render_cli(&["release-notes"]).expect("release notes should render");
        assert!(release_notes.contains("Release notes"));
        assert!(release_notes.contains("Release notes summary: release-notes-summary"));
        assert!(release_notes.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(release_notes.contains("Artifact validation: validate-artifact"));
        assert!(release_notes.lines().any(|line| {
            line == "Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
        }));
        assert!(release_notes.contains("Release checklist summary: release-checklist-summary"));
        assert!(release_notes.contains("Release bundle verification: verify-release-bundle"));
        assert!(release_notes
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_notes.contains("API stability posture:"));
        assert!(release_notes.contains("Bundle provenance:"));
        assert!(release_notes.contains("True Mula (Chandra Hari)"));
        assert!(release_notes.contains("Babylonian (Eta Piscium)"));
        assert!(release_notes.contains("Babylonian (Kugler 2)"));
        assert!(release_notes.contains("Babylonian (Kugler 3)"));
        assert!(release_notes.contains("Galactic Equator (Mula)"));
        assert!(release_notes.contains("Galactic Equator (Fiorenza)"));
        assert!(release_notes.contains("JN Bhasin"));
        assert!(release_notes.contains("Galactic Equator (True)"));
        assert!(release_notes.contains("True galactic equator"));
        assert!(release_notes.contains("Galactic equator true"));
        assert!(release_notes.contains("Galactic Center (Mardyks)"));
        assert!(release_notes.contains("Gal. Center = 0 Cap"));
        assert!(release_notes.contains("Skydram (Mardyks)"));
        assert!(release_notes.contains("Mula Wilhelm"));
        assert!(release_notes.contains("Wilhelm"));
        assert!(release_notes.contains("Galactic Center (Rgilbrand)"));
        assert!(release_notes.contains("Babylonian (True Geoc)"));
        assert!(release_notes.contains("Babylonian (True Topc)"));
        assert!(release_notes.contains("Babylonian (True Obs)"));
        assert!(release_notes.contains("Pullen SD (Sinusoidal Delta)"));
        assert!(release_notes.contains("Equal/MC house system"));
        assert!(release_notes.contains("Equal Midheaven house system"));
        assert!(release_notes.contains("True Balarama"));
        assert!(release_notes.contains("Aphoric"));
        assert!(release_notes.contains("Takra"));

        let release_notes_summary =
            render_cli(&["release-notes-summary"]).expect("release notes summary should render");
        assert!(release_notes_summary.contains("Release notes summary"));
        assert!(release_notes_summary
            .contains("Comparison tolerance policy: backend family=Composite; scopes=6"));
        assert!(release_notes_summary.contains("Pluto fallback (approximate)"));
        assert!(release_notes_summary.contains(&format!(
            "House code aliases: {}",
            current_compatibility_profile().house_code_aliases_summary_line()
        )));
        assert!(release_notes_summary.contains("API stability summary line:"));
        assert!(release_notes_summary
            .contains(&pleiades_jpl::selected_asteroid_terminal_boundary_summary_for_report()));
        assert!(release_notes_summary.contains(
            &pleiades_jpl::reference_snapshot_2451916_major_body_dense_boundary_summary_for_report(
            )
        ));
        assert!(release_notes_summary.contains(profile.target_house_scope.join("; ").as_str()));
        assert!(release_notes_summary.contains(profile.target_ayanamsa_scope.join("; ").as_str()));
        assert!(release_notes_summary.contains(&format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )));
        assert!(release_notes_summary.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        }));
        assert!(release_notes_summary.lines().any(|line| {
            line == "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
        }));
        assert!(release_notes_summary.lines().any(|line| {
            line == "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        }));
        assert!(release_notes_summary.lines().any(|line| {
            line == "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        }));
        assert!(release_notes_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_notes_summary.lines().any(|line| {
            line == "Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
        }));
        assert!(release_notes_summary.contains("Release notes: release-notes"));
        assert!(release_notes_summary.contains("Packaged-artifact storage/reconstruction:"));
        assert!(release_notes_summary
            .lines()
            .any(|line| line == packaged_artifact_access_report_line()));
        assert!(release_notes_summary.lines().any(|line| {
            line == "Packaged-artifact generation policy: adjacent same-body linear segments; bodies with a single sampled epoch use point segments; multi-epoch non-lunar bodies are fit with linear segments between adjacent same-body source epochs; the Moon uses overlapping three-point spans with quadratic residual corrections to keep the high-curvature fit compact"
        }));
        assert!(release_notes_summary.contains("Packaged request policy:"));
        assert!(release_notes_summary.contains("Packaged lookup epoch policy:"));
        assert!(release_notes_summary.lines().any(|line| {
            line == format!(
                "Packaged batch parity: {}",
                pleiades_data::packaged_mixed_tt_tdb_batch_parity_summary_for_report()
            )
        }));
        assert!(release_notes_summary.contains("Packaged batch parity:"));
        assert!(release_notes_summary
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(release_notes_summary
            .contains("Artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(
            release_notes_summary.contains("Release checklist summary: release-checklist-summary")
        );
        assert!(
            release_notes_summary.contains("Release bundle verification: verify-release-bundle")
        );
        assert!(release_notes_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_notes_summary
            .contains("See release-notes for the full maintainer-facing artifact."));
        assert!(release_notes_summary
            .contains("See release-summary for the compact one-screen release overview."));
        assert!(release_notes_summary.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));

        let release_checklist =
            render_cli(&["release-checklist"]).expect("release checklist should render");
        assert!(release_checklist.contains("Release checklist"));
        assert!(release_checklist.contains("Release notes summary: release-notes-summary"));
        assert!(release_checklist.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(release_checklist.contains("API stability summary: api-stability-summary"));
        assert!(release_checklist.contains("Artifact validation: validate-artifact"));
        assert!(release_checklist.lines().any(|line| {
            line == "Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"
        }));
        assert!(release_checklist.contains("Repository-managed release gates:"));
        assert!(release_checklist
            .contains("[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile"));
        assert!(release_checklist.contains("bundle-release --out /tmp/pleiades-release"));
        assert!(release_checklist.contains("release-checklist-summary.txt"));

        let release_checklist_summary = render_cli(&["release-checklist-summary"])
            .expect("release checklist summary should render");
        assert!(release_checklist_summary.contains("Release checklist summary"));
        assert!(release_checklist_summary.contains("Release notes summary: release-notes-summary"));
        assert!(
            release_checklist_summary.contains("Backend matrix summary: backend-matrix-summary")
        );
        assert!(release_checklist_summary.contains("API stability summary: api-stability-summary"));
        assert!(release_checklist_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_checklist_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_checklist_summary
            .contains("Release bundle verification: verify-release-bundle"));
        assert!(release_checklist_summary.contains("Workspace audit: workspace-audit / audit"));
        assert!(release_checklist_summary.contains("Release summary: release-summary"));
        assert!(release_checklist_summary.lines().any(|line| {
            line == "Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"
        }));
        assert!(release_checklist_summary.contains("Repository-managed release gates: 7 items"));
        assert!(release_checklist_summary.contains("Manual bundle workflow: 3 items"));
        assert!(release_checklist_summary.contains("Bundle contents: 17 items"));
        assert!(release_checklist_summary.contains("External publishing reminders: 3 items"));
        assert!(release_checklist_summary
            .contains("See release-summary for the compact one-screen release overview."));

        let release_gate = render_cli(&["release-gate"]).expect("release gate should render");
        let release_gate_summary =
            render_cli(&["release-gate-summary"]).expect("release gate summary should render");
        assert_eq!(release_gate, release_checklist);
        assert_eq!(release_gate_summary, release_checklist_summary);

        let release_summary =
            render_cli(&["release-summary"]).expect("release summary should render");
        assert!(release_summary.contains("Release summary"));
        assert!(release_summary.contains("House systems:"));
        assert!(release_summary.contains(&format!(
            "House-code aliases: {}",
            current_compatibility_profile().house_code_alias_count()
        )));
        assert!(release_summary.contains(&format!(
            "House code aliases: {}",
            current_compatibility_profile().house_code_aliases_summary_line()
        )));
        assert!(release_summary.contains("Compatibility catalog inventory: house systems=25 (12 baseline, 13 release-specific, 156 aliases); house formula families=7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign); house-code aliases=22; ayanamsas=59 (5 baseline, 54 release-specific, 183 aliases); custom-definition labels=9; custom-definition ayanamsa labels=6 (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs)); known gaps=2; claim audit: baseline catalogs are the published guarantees; release-specific entries are shipped additions; custom-definition labels remain intentionally unresolved; known gaps stay documented"));
        assert!(release_summary.contains("Comparison corpus release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day remains in the release-grade comparison window"));
        assert!(release_summary.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
        assert!(release_summary.lines().any(|line| {
            line == "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.6.123, api-stability=pleiades-api-stability/0.1.0"
        }));
        assert!(release_summary.contains("API stability summary line: API stability posture: pleiades-api-stability/0.1.0; stable surfaces: 6; experimental surfaces: 3; deprecation policy items: 4; intentional limits: 3"));
        assert!(release_summary.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));

        let request_surface_summary = render_cli(&["request-surface-summary"])
            .expect("request surface summary should render");
        assert_eq!(
            request_surface_summary,
            render_cli(&["request-surface"]).expect("request surface alias should render")
        );
        assert!(request_surface_summary.contains("Request surface summary"));
        assert!(request_surface_summary.contains("Primary request surfaces:"));
        assert!(request_surface_summary
            .contains("pleiades-types::Instant (tagged instant plus caller-supplied retagging)"));
        assert!(request_surface_summary.contains(
            "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)"
        ));
        assert!(request_surface_summary.contains(
            "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)"
        ));
        assert!(request_surface_summary
            .contains("pleiades-houses::HouseRequest (house-only observer calculations)"));
        assert!(
            request_surface_summary.contains("pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)")
        );
        assert_eq!(
            request_surface_summary,
            pleiades_validate::render_request_surface_summary()
        );
        assert_eq!(
            render_cli(&["request-surface-summary", "extra"])
                .expect_err("request surface summary should reject extra arguments"),
            "request-surface-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["request-surface", "extra"])
                .expect_err("request surface alias should reject extra arguments"),
            "request-surface does not accept extra arguments"
        );

        let request_policy_summary =
            render_cli(&["request-policy-summary"]).expect("request policy summary should render");
        assert!(request_policy_summary.contains("Request policy summary"));
        assert!(request_policy_summary.contains("Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"));
        assert!(request_policy_summary.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
        assert!(request_policy_summary.contains("Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"));
        assert!(request_policy_summary.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"));
        assert!(request_policy_summary.contains("Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"));
        assert!(request_policy_summary.contains("Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
        let request_policy_alias =
            render_cli(&["request-policy"]).expect("request policy alias should render");
        assert_eq!(request_policy_alias, request_policy_summary);

        let request_semantics_alias =
            render_cli(&["request-semantics"]).expect("request semantics alias should render");
        assert!(request_semantics_alias.contains("Request semantics summary"));
        assert_eq!(
            request_semantics_alias.replacen(
                "Request semantics summary",
                "Request policy summary",
                1
            ),
            request_policy_summary
        );

        let request_policy_error = render_cli(&["request-policy", "extra"])
            .expect_err("request policy alias should reject extra arguments");
        assert_eq!(
            request_policy_error,
            "request-policy does not accept extra arguments"
        );
        let request_policy_summary_error = render_cli(&["request-policy-summary", "extra"])
            .expect_err("request policy summary should reject extra arguments");
        assert_eq!(
            request_policy_summary_error,
            "request-policy-summary does not accept extra arguments"
        );
        let request_semantics_summary = render_cli(&["request-semantics-summary"])
            .expect("request semantics summary should render");
        assert!(request_semantics_summary.contains("Request semantics summary"));
        assert_eq!(
            request_semantics_summary.replacen(
                "Request semantics summary",
                "Request policy summary",
                1
            ),
            request_policy_summary
        );
        let request_semantics_error = render_cli(&["request-semantics-summary", "extra"])
            .expect_err("request semantics alias should reject extra arguments");
        assert_eq!(
            request_semantics_error,
            "request-semantics-summary does not accept extra arguments"
        );

        let request_semantics_alias_error = render_cli(&["request-semantics", "extra"])
            .expect_err("request-semantics alias should reject extra arguments");
        assert_eq!(
            request_semantics_alias_error,
            "request-semantics does not accept extra arguments"
        );

        for (args, expected) in [
            (
                ["catalog-inventory-summary", "extra"],
                "catalog-inventory-summary does not accept extra arguments",
            ),
            (
                ["catalog-inventory", "extra"],
                "catalog-inventory does not accept extra arguments",
            ),
            (
                ["backend-matrix", "extra"],
                "backend-matrix does not accept extra arguments",
            ),
            (
                ["compatibility-profile", "extra"],
                "compatibility-profile does not accept extra arguments",
            ),
            (
                ["api-posture-summary", "extra"],
                "api-stability-summary does not accept extra arguments",
            ),
            (
                ["api-stability", "extra"],
                "api-stability does not accept extra arguments",
            ),
            (
                ["release-checklist", "extra"],
                "release-checklist does not accept extra arguments",
            ),
            (
                ["release-summary", "extra"],
                "release-summary does not accept extra arguments",
            ),
            (
                ["release-notes", "extra"],
                "release-notes does not accept extra arguments",
            ),
            (
                ["artifact-posture-summary", "extra"],
                "artifact-summary does not accept extra arguments",
            ),
        ] {
            assert_eq!(
                render_cli(&args).expect_err("summary command should reject extra arguments"),
                expected
            );
        }

        let comparison_tolerance_policy_summary =
            render_cli(&["comparison-tolerance-policy-summary"])
                .expect("comparison tolerance policy summary should render");
        assert_eq!(
            comparison_tolerance_policy_summary,
            super::validate_render_cli(&["comparison-tolerance-policy-summary"])
                .expect("comparison tolerance policy summary should match validate CLI")
        );

        let comparison_tolerance_summary = render_cli(&["comparison-tolerance-summary"])
            .expect("comparison tolerance summary alias should render");
        assert_eq!(
            comparison_tolerance_summary, comparison_tolerance_policy_summary,
            "comparison tolerance summary alias should match the canonical command"
        );

        let comparison_body_class_tolerance_summary =
            render_cli(&["comparison-body-class-tolerance-summary"])
                .expect("comparison body-class tolerance summary should render");
        assert_eq!(
            comparison_body_class_tolerance_summary,
            super::validate_render_cli(&["comparison-body-class-tolerance-summary"])
                .expect("comparison body-class tolerance summary should match validate CLI")
        );

        let comparison_body_class_tolerance_alias =
            render_cli(&["comparison-body-class-tolerance"])
                .expect("comparison body-class tolerance alias should render");
        assert_eq!(
            comparison_body_class_tolerance_alias, comparison_body_class_tolerance_summary,
            "comparison body-class tolerance alias should match the canonical command"
        );

        let comparison_envelope_summary = render_cli(&["comparison-envelope-summary"])
            .expect("comparison envelope summary should render");
        assert_eq!(
            comparison_envelope_summary,
            super::validate_render_cli(&["comparison-envelope-summary"])
                .expect("comparison envelope summary should match validate CLI")
        );

        let comparison_envelope_alias =
            render_cli(&["comparison-envelope"]).expect("comparison envelope alias should render");
        assert_eq!(comparison_envelope_alias, comparison_envelope_summary);

        let comparison_tolerance_alias_error =
            render_cli(&["comparison-tolerance-summary", "extra"])
                .expect_err("comparison tolerance alias should reject extra arguments");
        assert!(comparison_tolerance_alias_error
            .contains("comparison-tolerance-summary does not accept extra arguments"));

        let comparison_body_class_tolerance_alias_error =
            render_cli(&["comparison-body-class-tolerance", "extra"])
                .expect_err("comparison body-class tolerance alias should reject extra arguments");
        assert!(comparison_body_class_tolerance_alias_error
            .contains("comparison-body-class-tolerance does not accept extra arguments"));

        let comparison_envelope_alias_error = render_cli(&["comparison-envelope", "extra"])
            .expect_err("comparison envelope alias should reject extra arguments");
        assert!(comparison_envelope_alias_error
            .contains("comparison-envelope does not accept extra arguments"));

        let release_body_claims_summary = render_cli(&["release-body-claims-summary"])
            .expect("release body claims summary should render");
        assert_eq!(
            release_body_claims_summary,
            super::validate_render_cli(&["release-body-claims-summary"])
                .expect("release body claims summary should match validate CLI")
        );
        assert_eq!(
            render_cli(&["body-claims-summary"]).expect("body claims alias should render"),
            release_body_claims_summary
        );
        assert_eq!(
            render_cli(&["body-claims-summary", "extra"])
                .expect_err("body claims alias should reject extra arguments"),
            "body-claims-summary does not accept extra arguments"
        );

        let pluto_fallback_summary =
            render_cli(&["pluto-fallback-summary"]).expect("Pluto fallback summary should render");
        assert_eq!(
            pluto_fallback_summary,
            super::validate_render_cli(&["pluto-fallback-summary"])
                .expect("Pluto fallback summary should match validate CLI")
        );
        assert_eq!(
            render_cli(&["pluto-fallback"]).expect("Pluto fallback alias should render"),
            pluto_fallback_summary
        );
        assert_eq!(
            render_cli(&["pluto-fallback", "extra"])
                .expect_err("Pluto fallback alias should reject extra arguments"),
            "pluto-fallback does not accept extra arguments"
        );

        let jpl_batch_error_taxonomy_summary = render_cli(&["jpl-batch-error-taxonomy-summary"])
            .expect("JPL batch error taxonomy summary should render");
        assert_eq!(
            jpl_batch_error_taxonomy_summary,
            super::validate_render_cli(&["jpl-batch-error-taxonomy-summary"])
                .expect("validation JPL batch error taxonomy summary should render")
        );

        let jpl_snapshot_evidence_summary = render_cli(&["jpl-snapshot-evidence-summary"])
            .expect("JPL snapshot evidence summary should render");
        assert!(jpl_snapshot_evidence_summary
            .contains(&pleiades_jpl::jpl_source_posture_summary_for_report()));
        assert_eq!(
            jpl_snapshot_evidence_summary,
            super::validate_render_cli(&["jpl-snapshot-evidence-summary"])
                .expect("validation JPL snapshot evidence summary should render")
        );

        let packaged_lookup_epoch_policy_summary =
            render_cli(&["packaged-lookup-epoch-policy-summary"])
                .expect("packaged lookup epoch policy summary should render");
        assert!(packaged_lookup_epoch_policy_summary
            .contains("TT-grid retag without relativistic correction"));
        assert_eq!(
            packaged_lookup_epoch_policy_summary,
            pleiades_data::packaged_lookup_epoch_policy_summary_for_report()
        );
        assert_eq!(
            packaged_lookup_epoch_policy_summary,
            render_cli(&["packaged-lookup-epoch-policy"])
                .expect("packaged lookup epoch policy alias should render")
        );

        let production_generation_boundary_summary =
            render_cli(&["production-generation-boundary-summary"])
                .expect("production generation boundary summary should render");
        assert!(production_generation_boundary_summary
            .contains("Production generation boundary overlay:"));
        assert_eq!(
            production_generation_boundary_summary,
            pleiades_jpl::production_generation_boundary_summary_for_report()
        );
        let production_generation_summary = render_cli(&["production-generation-summary"])
            .expect("production generation summary should render");
        assert!(production_generation_summary.contains("Production generation coverage:"));
        assert_eq!(
            production_generation_summary,
            pleiades_jpl::production_generation_snapshot_summary_for_report()
        );
        let production_generation_alias = render_cli(&["production-generation"])
            .expect("production generation alias should render");
        assert_eq!(
            production_generation_alias,
            pleiades_jpl::production_generation_snapshot_summary_for_report()
        );
        let alias_error = render_cli(&["production-generation", "extra"])
            .expect_err("production generation alias should reject extra arguments");
        assert!(alias_error.contains("production-generation does not accept extra arguments"));
        assert_eq!(
            render_cli(&["production-generation-boundary-summary", "extra"]).unwrap_err(),
            "production-generation-boundary-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["production-generation-summary", "extra"]).unwrap_err(),
            "production-generation-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["production-generation-source-summary", "extra"]).unwrap_err(),
            "production-generation-source-summary does not accept extra arguments"
        );
        let production_generation_boundary_request_corpus_summary =
            render_cli(&["production-generation-boundary-request-corpus-summary"])
                .expect("production generation boundary request corpus summary should render");
        assert!(production_generation_boundary_request_corpus_summary
            .contains("Production generation boundary request corpus:"));
        assert_eq!(
            production_generation_boundary_request_corpus_summary,
            pleiades_jpl::production_generation_boundary_request_corpus_summary_for_report()
        );
        let production_generation_body_class_coverage_summary =
            render_cli(&["production-generation-body-class-coverage-summary"])
                .expect("production generation body-class coverage summary should render");
        assert!(production_generation_body_class_coverage_summary
            .contains("Production generation body-class coverage:"));
        let production_body_class_coverage_summary =
            render_cli(&["production-body-class-coverage-summary"])
                .expect("production body-class coverage summary alias should render");
        assert_eq!(
            production_body_class_coverage_summary,
            production_generation_body_class_coverage_summary
        );
        assert_eq!(
            production_generation_body_class_coverage_summary,
            super::validate_render_cli(&["production-generation-body-class-coverage-summary"])
                .expect(
                    "validation production generation body-class coverage summary should render"
                )
        );
        let comparison_body_class_coverage_summary =
            render_cli(&["comparison-body-class-coverage-summary"])
                .expect("comparison body-class coverage summary alias should render");
        assert_eq!(
            comparison_body_class_coverage_summary,
            super::validate_render_cli(&["comparison-snapshot-body-class-coverage-summary"])
                .expect("validation comparison snapshot body-class coverage summary should render")
        );
        let reference_body_class_coverage_summary =
            render_cli(&["reference-body-class-coverage-summary"])
                .expect("reference body-class coverage summary alias should render");
        assert_eq!(
            reference_body_class_coverage_summary,
            super::validate_render_cli(&["reference-snapshot-body-class-coverage-summary"])
                .expect("validation reference snapshot body-class coverage summary should render")
        );
        let holdout_body_class_coverage_summary =
            render_cli(&["holdout-body-class-coverage-summary"])
                .expect("holdout body-class coverage summary alias should render");
        assert_eq!(
            holdout_body_class_coverage_summary,
            super::validate_render_cli(&["independent-holdout-body-class-coverage-summary"])
                .expect(
                    "validation independent hold-out body-class coverage summary should render"
                )
        );
        let comparison_snapshot_source_window_summary =
            render_cli(&["comparison-snapshot-source-window-summary"])
                .expect("comparison snapshot source window summary should render");
        assert!(comparison_snapshot_source_window_summary
            .contains("Comparison snapshot source windows:"));
        assert_eq!(
            comparison_snapshot_source_window_summary,
            pleiades_jpl::comparison_snapshot_source_window_summary_for_report()
        );
        let comparison_snapshot_source_window_alias =
            render_cli(&["comparison-snapshot-source-window"])
                .expect("comparison snapshot source window alias should render");
        assert_eq!(
            comparison_snapshot_source_window_alias,
            comparison_snapshot_source_window_summary
        );
        assert_eq!(
            render_cli(&["comparison-snapshot-source-window", "extra"]).expect_err(
                "comparison snapshot source window alias should reject extra arguments"
            ),
            "comparison-snapshot-source-window does not accept extra arguments"
        );
        let comparison_snapshot_source_summary =
            render_cli(&["comparison-snapshot-source-summary"])
                .expect("comparison snapshot source summary should render");
        assert!(comparison_snapshot_source_summary.contains("Comparison snapshot source:"));
        assert_eq!(
            comparison_snapshot_source_summary,
            pleiades_jpl::comparison_snapshot_source_summary_for_report()
        );
        assert_eq!(
            render_cli(&["comparison-snapshot-source-summary", "extra"])
                .expect_err("comparison snapshot source summary should reject extra arguments"),
            "comparison-snapshot-source-summary does not accept extra arguments"
        );
        let reference_snapshot_source_window_summary =
            render_cli(&["reference-snapshot-source-window-summary"])
                .expect("reference snapshot source window summary should render");
        assert!(
            reference_snapshot_source_window_summary.contains("Reference snapshot source windows:")
        );
        assert_eq!(
            reference_snapshot_source_window_summary,
            pleiades_jpl::reference_snapshot_source_window_summary_for_report()
        );
        let reference_snapshot_source_window_alias =
            render_cli(&["reference-snapshot-source-window"])
                .expect("reference snapshot source window alias should render");
        assert_eq!(
            reference_snapshot_source_window_alias,
            reference_snapshot_source_window_summary
        );
        assert_eq!(
            render_cli(&["reference-snapshot-source-window", "extra"])
                .expect_err("reference snapshot source window alias should reject extra arguments"),
            "reference-snapshot-source-window does not accept extra arguments"
        );
        let production_generation_boundary_source_summary =
            render_cli(&["production-generation-boundary-source-summary"])
                .expect("production generation boundary source summary should render");
        assert!(production_generation_boundary_source_summary
            .contains("Production generation boundary overlay source:"));
        assert!(production_generation_boundary_source_summary.contains("boundary overlay source"));
        assert_eq!(
            production_generation_boundary_source_summary,
            pleiades_jpl::production_generation_boundary_source_summary_for_report()
        );
        let production_generation_boundary_window_summary =
            render_cli(&["production-generation-boundary-window-summary"])
                .expect("production generation boundary window summary should render");
        assert!(production_generation_boundary_window_summary
            .contains("Production generation boundary windows:"));
        assert!(production_generation_boundary_window_summary.contains("source-backed samples"));
        assert_eq!(
            production_generation_boundary_window_summary,
            pleiades_jpl::production_generation_boundary_window_summary_for_report()
        );
        let production_generation_boundary_window_alias =
            render_cli(&["production-generation-boundary-window"])
                .expect("production generation boundary window alias should render");
        assert_eq!(
            production_generation_boundary_window_alias,
            pleiades_jpl::production_generation_boundary_window_summary_for_report()
        );
        assert_eq!(
            render_cli(&["production-generation-boundary-window", "extra"]).expect_err(
                "production generation boundary window alias should reject extra arguments"
            ),
            "production-generation-boundary-window does not accept extra arguments"
        );
        let production_generation_source_summary =
            render_cli(&["production-generation-source-summary"])
                .expect("production generation source summary should render");
        assert!(production_generation_source_summary.contains("Production generation source:"));
        assert_eq!(
            production_generation_source_summary,
            pleiades_jpl::production_generation_source_summary_for_report()
        );
        let production_generation_source_alias = render_cli(&["production-generation-source"])
            .expect("production generation source alias should render");
        assert_eq!(
            production_generation_source_alias,
            pleiades_jpl::production_generation_source_summary_for_report()
        );
        assert_eq!(
            render_cli(&["production-generation-source", "extra"])
                .expect_err("production generation source alias should reject extra arguments"),
            "production-generation-source does not accept extra arguments"
        );
        let reference_snapshot_lunar_boundary_summary =
            render_cli(&["reference-snapshot-lunar-boundary-summary"])
                .expect("reference snapshot lunar boundary summary should render");
        assert!(reference_snapshot_lunar_boundary_summary
            .contains("Reference lunar boundary evidence:"));
        assert_eq!(
            reference_snapshot_lunar_boundary_summary,
            pleiades_jpl::reference_snapshot_lunar_boundary_summary_for_report()
        );
        let lunar_boundary_summary = render_cli(&["lunar-boundary-summary"])
            .expect("lunar boundary summary alias should render");
        assert!(lunar_boundary_summary.contains("Reference lunar boundary evidence:"));
        assert_eq!(
            lunar_boundary_summary,
            pleiades_jpl::reference_snapshot_lunar_boundary_summary_for_report()
        );
        let reference_snapshot_1600_selected_body_boundary_summary =
            render_cli(&["reference-snapshot-1600-selected-body-boundary-summary"])
                .expect("reference snapshot 1600 selected-body boundary summary should render");
        assert!(reference_snapshot_1600_selected_body_boundary_summary
            .contains("Reference 1600 selected-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_1600_selected_body_boundary_summary,
            pleiades_jpl::reference_snapshot_1600_selected_body_boundary_summary_for_report()
        );
        let reference_snapshot_2200_selected_body_boundary_summary =
            render_cli(&["reference-snapshot-2200-selected-body-boundary-summary"])
                .expect("reference snapshot 2200 selected-body boundary summary should render");
        assert!(reference_snapshot_2200_selected_body_boundary_summary
            .contains("Reference 2200 selected-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_2200_selected_body_boundary_summary,
            pleiades_jpl::reference_snapshot_2200_selected_body_boundary_summary_for_report()
        );
        let reference_snapshot_2524593_selected_body_boundary_summary =
            render_cli(&["reference-snapshot-2524593-selected-body-boundary-summary"])
                .expect("reference snapshot 2524593 selected-body boundary summary should render");
        assert!(reference_snapshot_2524593_selected_body_boundary_summary
            .contains("Reference 2524593 selected-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_2524593_selected_body_boundary_summary,
            pleiades_jpl::reference_snapshot_2524593_selected_body_boundary_summary_for_report()
        );
        let reference_snapshot_2634167_selected_body_boundary_summary =
            render_cli(&["reference-snapshot-2634167-selected-body-boundary-summary"])
                .expect("reference snapshot 2634167 selected-body boundary summary should render");
        assert!(reference_snapshot_2634167_selected_body_boundary_summary
            .contains("Reference 2634167 selected-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_2634167_selected_body_boundary_summary,
            pleiades_jpl::reference_snapshot_2634167_selected_body_boundary_summary_for_report()
        );
        let selected_body_boundary_alias = render_cli(&["2634167-selected-body-boundary-summary"])
            .expect("2634167 selected-body boundary summary alias should render");
        assert_eq!(
            selected_body_boundary_alias,
            pleiades_jpl::reference_snapshot_2634167_selected_body_boundary_summary_for_report()
        );
        let reference_snapshot_1749_major_body_boundary_summary =
            render_cli(&["reference-snapshot-1749-major-body-boundary-summary"])
                .expect("reference snapshot 1749 major-body boundary summary should render");
        assert!(reference_snapshot_1749_major_body_boundary_summary
            .contains("Reference 1749 major-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_1749_major_body_boundary_summary,
            pleiades_jpl::reference_snapshot_1749_major_body_boundary_summary_for_report()
        );
        let reference_snapshot_early_major_body_boundary_summary =
            render_cli(&["reference-snapshot-early-major-body-boundary-summary"])
                .expect("reference snapshot early major-body boundary summary should render");
        assert!(reference_snapshot_early_major_body_boundary_summary
            .contains("Reference early major-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_early_major_body_boundary_summary,
            pleiades_jpl::reference_snapshot_early_major_body_boundary_summary_for_report()
        );
        let reference_snapshot_1800_major_body_boundary_summary =
            render_cli(&["reference-snapshot-1800-major-body-boundary-summary"])
                .expect("reference snapshot 1800 major-body boundary summary should render");
        assert!(reference_snapshot_1800_major_body_boundary_summary
            .contains("Reference 1800 major-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_1800_major_body_boundary_summary,
            pleiades_jpl::reference_snapshot_1800_major_body_boundary_summary_for_report()
        );
        let reference_snapshot_2451910_major_body_boundary_summary =
            render_cli(&["reference-snapshot-2451910-major-body-boundary-summary"])
                .expect("reference snapshot 2451910 major-body boundary summary should render");
        assert!(reference_snapshot_2451910_major_body_boundary_summary
            .contains("Reference 2451910 major-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_2451910_major_body_boundary_summary,
            pleiades_jpl::reference_snapshot_2451910_major_body_boundary_summary_for_report()
        );
        let reference_snapshot_2451910_major_body_boundary_alias =
            render_cli(&["2451910-major-body-boundary-summary"])
                .expect("2451910 major-body boundary alias should render");
        assert_eq!(
            reference_snapshot_2451910_major_body_boundary_alias,
            reference_snapshot_2451910_major_body_boundary_summary
        );
        let comparison_snapshot_manifest_summary =
            render_cli(&["comparison-snapshot-manifest-summary"])
                .expect("comparison snapshot manifest summary should render");
        assert!(comparison_snapshot_manifest_summary.contains("Comparison snapshot manifest:"));
        assert_eq!(
            comparison_snapshot_manifest_summary,
            pleiades_jpl::comparison_snapshot_manifest_summary_for_report()
        );
        assert_eq!(
            render_cli(&["comparison-snapshot-manifest"])
                .expect("comparison snapshot manifest alias should render"),
            comparison_snapshot_manifest_summary
        );
        assert_eq!(
            render_cli(&["comparison-snapshot-manifest", "extra"])
                .expect_err("comparison snapshot manifest alias should reject extra arguments"),
            "comparison-snapshot-manifest does not accept extra arguments"
        );
        let comparison_snapshot_summary = render_cli(&["comparison-snapshot-summary"])
            .expect("comparison snapshot summary should render");
        assert!(comparison_snapshot_summary.contains("Comparison snapshot summary"));
        assert_eq!(
            comparison_snapshot_summary,
            format!(
                "Comparison snapshot summary\n{}\n",
                pleiades_jpl::comparison_snapshot_summary_for_report()
            )
        );
        assert_eq!(
            render_cli(&["comparison-snapshot"]).expect("comparison snapshot alias should render"),
            comparison_snapshot_summary
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
        let comparison_snapshot_batch_parity_summary =
            render_cli(&["comparison-snapshot-batch-parity-summary"])
                .expect("comparison snapshot batch parity summary should render");
        assert_eq!(
            comparison_snapshot_batch_parity_summary,
            super::validate_render_cli(&["comparison-snapshot-batch-parity-summary"])
                .expect("validation comparison snapshot batch parity summary should render")
        );
        let reference_snapshot_manifest_summary =
            render_cli(&["reference-snapshot-manifest-summary"])
                .expect("reference snapshot manifest summary should render");
        assert!(reference_snapshot_manifest_summary.contains("Reference snapshot manifest:"));
        assert_eq!(
            reference_snapshot_manifest_summary,
            pleiades_jpl::reference_snapshot_manifest_summary_for_report()
        );
        assert_eq!(
            render_cli(&["reference-snapshot-manifest"])
                .expect("reference snapshot manifest alias should render"),
            reference_snapshot_manifest_summary
        );
        assert_eq!(
            render_cli(&["reference-snapshot-manifest", "extra"])
                .expect_err("reference snapshot manifest alias should reject extra arguments"),
            "reference-snapshot-manifest does not accept extra arguments"
        );
        let reference_snapshot_source_summary = render_cli(&["reference-snapshot-source-summary"])
            .expect("reference snapshot source summary should render");
        assert!(reference_snapshot_source_summary.contains("Reference snapshot source:"));
        assert_eq!(
            reference_snapshot_source_summary,
            pleiades_jpl::reference_snapshot_source_summary_for_report()
        );
        assert_eq!(
            render_cli(&["reference-snapshot-source-summary", "extra"])
                .expect_err("reference snapshot source summary should reject extra arguments"),
            "reference-snapshot-source-summary does not accept extra arguments"
        );
        let reference_snapshot_summary = render_cli(&["reference-snapshot-summary"])
            .expect("reference snapshot summary should render");
        assert!(reference_snapshot_summary.contains("Reference snapshot summary"));
        assert!(reference_snapshot_summary.contains("Reference 2500 major-body boundary evidence:"));
        assert!(reference_snapshot_summary.contains(
            &pleiades_jpl::reference_snapshot_2451916_major_body_dense_boundary_summary_for_report(
            )
        ));
        assert!(reference_snapshot_summary.contains(
            &pleiades_jpl::reference_snapshot_2451918_major_body_boundary_summary_for_report()
        ));
        assert!(reference_snapshot_summary.contains(
            &pleiades_jpl::reference_snapshot_2451919_major_body_boundary_summary_for_report()
        ));
        assert!(reference_snapshot_summary.contains(
            &pleiades_jpl::reference_snapshot_2451914_major_body_bridge_day_summary_for_report()
        ));
        assert!(reference_snapshot_summary
            .contains(&pleiades_jpl::selected_asteroid_boundary_summary_for_report()));
        assert!(reference_snapshot_summary
            .contains(&pleiades_jpl::selected_asteroid_bridge_summary_for_report()));
        assert!(reference_snapshot_summary
            .contains(&pleiades_jpl::selected_asteroid_dense_boundary_summary_for_report()));
        assert!(reference_snapshot_summary
            .contains(&pleiades_jpl::selected_asteroid_terminal_boundary_summary_for_report()));
        assert_eq!(
            reference_snapshot_summary,
            format!(
                "Reference snapshot summary\n{}\n",
                pleiades_jpl::reference_snapshot_summary_for_report()
            )
        );
        assert_eq!(
            render_cli(&["reference-snapshot"]).expect("reference snapshot alias should render"),
            reference_snapshot_summary
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

        let reference_snapshot_exact_j2000 =
            render_cli(&["reference-snapshot-exact-j2000-evidence-summary"])
                .expect("reference snapshot exact J2000 evidence should render");
        assert!(reference_snapshot_exact_j2000
            .contains("Reference snapshot exact J2000 evidence summary"));
        assert!(reference_snapshot_exact_j2000.contains(
            "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0"
        ));
        assert_eq!(
            reference_snapshot_exact_j2000,
            format!(
                "Reference snapshot exact J2000 evidence summary\n{}\n",
                pleiades_jpl::reference_snapshot_exact_j2000_evidence_summary_for_report()
            )
        );
        let exact_j2000_evidence = render_cli(&["exact-j2000-evidence"])
            .expect("exact J2000 evidence alias should render");
        assert_eq!(exact_j2000_evidence, reference_snapshot_exact_j2000);
        assert_eq!(
            render_cli(&["exact-j2000-evidence", "extra"])
                .expect_err("exact J2000 evidence alias should reject extra arguments"),
            "exact-j2000-evidence does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["reference-snapshot-exact-j2000-evidence-summary", "extra"]).expect_err(
                "reference snapshot exact J2000 evidence should reject extra arguments"
            ),
            "reference-snapshot-exact-j2000-evidence-summary does not accept extra arguments"
        );
        let reference_snapshot_batch_parity_summary =
            render_cli(&["reference-snapshot-batch-parity-summary"])
                .expect("reference snapshot batch parity summary should render");
        assert_eq!(
            reference_snapshot_batch_parity_summary,
            super::validate_render_cli(&["reference-snapshot-batch-parity-summary"])
                .expect("validation reference snapshot batch parity summary should render")
        );
        let reference_snapshot_equatorial_parity_summary =
            render_cli(&["reference-snapshot-equatorial-parity-summary"])
                .expect("reference snapshot equatorial parity summary should render");
        assert_eq!(
            reference_snapshot_equatorial_parity_summary,
            super::validate_render_cli(&["reference-snapshot-equatorial-parity-summary"])
                .expect("validation reference snapshot equatorial parity summary should render")
        );
        let time_scale_policy_summary = render_cli(&["time-scale-policy-summary"])
            .expect("time-scale policy summary should render");
        assert!(time_scale_policy_summary.contains("Time-scale policy summary"));
        assert!(time_scale_policy_summary.contains("Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"));
        assert_eq!(
            render_cli(&["time-scale-policy"]).expect("time-scale policy alias should render"),
            time_scale_policy_summary
        );
        let utc_convenience_policy_summary = render_cli(&["utc-convenience-policy-summary"])
            .expect("UTC convenience policy summary should render");
        assert!(utc_convenience_policy_summary.contains("UTC convenience policy summary"));
        assert!(utc_convenience_policy_summary.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
        assert_eq!(
            render_cli(&["utc-convenience-policy"])
                .expect("UTC convenience policy alias should render"),
            utc_convenience_policy_summary
        );
        let delta_t_policy_summary =
            render_cli(&["delta-t-policy-summary"]).expect("delta T policy summary should render");
        assert!(delta_t_policy_summary.contains("Delta T policy summary"));
        assert!(delta_t_policy_summary.contains("Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"));
        assert_eq!(
            render_cli(&["delta-t-policy"]).expect("delta T policy alias should render"),
            delta_t_policy_summary
        );
        let observer_policy_summary = render_cli(&["observer-policy-summary"])
            .expect("observer policy summary should render");
        assert!(observer_policy_summary.contains("Observer policy summary"));
        assert!(observer_policy_summary.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"));
        assert_eq!(
            render_cli(&["observer-policy"]).expect("observer policy alias should render"),
            observer_policy_summary
        );
        let apparentness_policy_summary = render_cli(&["apparentness-policy-summary"])
            .expect("apparentness policy summary should render");
        assert!(apparentness_policy_summary.contains("Apparentness policy summary"));
        assert!(apparentness_policy_summary.contains("Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"));
        assert_eq!(
            render_cli(&["apparentness-policy"]).expect("apparentness policy alias should render"),
            apparentness_policy_summary
        );
        let native_sidereal_policy_summary = render_cli(&["native-sidereal-policy-summary"])
            .expect("native sidereal policy summary should render");
        assert!(native_sidereal_policy_summary.contains("Native sidereal policy summary"));
        assert!(native_sidereal_policy_summary.contains("Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
        assert_eq!(
            render_cli(&["native-sidereal-policy"])
                .expect("native sidereal policy alias should render"),
            native_sidereal_policy_summary
        );
        let interpolation_posture_summary = render_cli(&["interpolation-posture-summary"])
            .expect("interpolation posture summary should render");
        assert_eq!(
            interpolation_posture_summary,
            super::validate_render_cli(&["interpolation-posture-summary"])
                .expect("validation interpolation posture summary should render")
        );
        assert_eq!(
            render_cli(&["interpolation-posture"])
                .expect("interpolation posture alias should render"),
            interpolation_posture_summary
        );
        assert_eq!(
            render_cli(&["interpolation-posture", "extra"])
                .expect_err("interpolation posture alias should reject extra arguments"),
            "interpolation-posture does not accept extra arguments"
        );
        let mean_obliquity_frame_round_trip_summary =
            render_cli(&["mean-obliquity-frame-round-trip-summary"])
                .expect("mean-obliquity frame round-trip summary should render");
        assert_eq!(
            mean_obliquity_frame_round_trip_summary,
            super::validate_render_cli(&["mean-obliquity-frame-round-trip-summary"])
                .expect("validation mean-obliquity frame round-trip summary should render")
        );
        assert_eq!(
            render_cli(&["mean-obliquity-frame-round-trip"])
                .expect("mean-obliquity frame round-trip alias should render"),
            mean_obliquity_frame_round_trip_summary
        );
        assert_eq!(
            render_cli(&["mean-obliquity-frame-round-trip", "extra"])
                .expect_err("mean-obliquity frame round-trip alias should reject extra arguments"),
            "mean-obliquity-frame-round-trip does not accept extra arguments"
        );
        let interpolation_quality_kind_coverage_summary =
            render_cli(&["interpolation-quality-kind-coverage-summary"])
                .expect("interpolation quality kind coverage summary should render");
        assert!(interpolation_quality_kind_coverage_summary
            .contains("JPL interpolation quality kind coverage:"));
        let lunar_reference_error_envelope_summary =
            render_cli(&["lunar-reference-error-envelope-summary"])
                .expect("lunar reference error envelope summary should render");
        assert!(lunar_reference_error_envelope_summary
            .contains("Lunar reference error envelope summary"));
        assert_eq!(
            lunar_reference_error_envelope_summary,
            format!(
                "Lunar reference error envelope summary\n{}\n",
                pleiades_elp::lunar_reference_evidence_envelope_for_report()
            )
        );
        let lunar_equatorial_reference_error_envelope_summary =
            render_cli(&["lunar-equatorial-reference-error-envelope-summary"])
                .expect("lunar equatorial reference error envelope summary should render");
        assert!(lunar_equatorial_reference_error_envelope_summary
            .contains("Lunar equatorial reference error envelope summary"));
        assert_eq!(
            lunar_equatorial_reference_error_envelope_summary,
            format!(
                "Lunar equatorial reference error envelope summary\n{}\n",
                pleiades_elp::lunar_equatorial_reference_evidence_envelope_for_report()
            )
        );
        let lunar_apparent_comparison_summary = render_cli(&["lunar-apparent-comparison-summary"])
            .expect("lunar apparent comparison summary should render");
        assert!(lunar_apparent_comparison_summary.contains("Lunar apparent comparison summary"));
        assert_eq!(
            lunar_apparent_comparison_summary,
            format!(
                "Lunar apparent comparison summary\n{}\n",
                pleiades_elp::lunar_apparent_comparison_summary_for_report()
            )
        );
        let lunar_source_window_summary = render_cli(&["lunar-source-window-summary"])
            .expect("lunar source window summary should render");
        assert!(lunar_source_window_summary.contains("lunar source windows"));
        assert_eq!(
            lunar_source_window_summary,
            pleiades_elp::lunar_source_window_summary_for_report()
        );
        let lunar_theory_request_policy_summary =
            render_cli(&["lunar-theory-request-policy-summary"])
                .expect("lunar theory request policy summary should render");
        assert_eq!(
            lunar_theory_request_policy_summary,
            pleiades_elp::lunar_theory_request_policy_summary()
        );
        assert_eq!(
            render_cli(&["lunar-theory-request-policy"])
                .expect("lunar theory request policy alias should render"),
            lunar_theory_request_policy_summary
        );
        assert_eq!(
            render_cli(&["lunar-theory-request-policy", "extra"])
                .expect_err("lunar theory request policy alias should reject extra arguments"),
            "lunar-theory-request-policy does not accept extra arguments"
        );
        let lunar_theory_frame_treatment_summary =
            render_cli(&["lunar-theory-frame-treatment-summary"])
                .expect("lunar theory frame treatment summary should render");
        assert_eq!(
            lunar_theory_frame_treatment_summary,
            pleiades_elp::lunar_theory_frame_treatment_summary_for_report()
        );
        assert_eq!(
            render_cli(&["lunar-theory-frame-treatment"])
                .expect("lunar theory frame treatment alias should render"),
            lunar_theory_frame_treatment_summary
        );
        assert_eq!(
            render_cli(&["lunar-theory-frame-treatment", "extra"])
                .expect_err("lunar theory frame treatment alias should reject extra arguments"),
            "lunar-theory-frame-treatment does not accept extra arguments"
        );
        let lunar_theory_limitations_summary = render_cli(&["lunar-theory-limitations-summary"])
            .expect("lunar theory limitations summary should render");
        assert_eq!(
            lunar_theory_limitations_summary,
            pleiades_elp::lunar_theory_limitations_summary_for_report()
        );
        assert_eq!(
            render_cli(&["lunar-theory-limitations"]).unwrap(),
            pleiades_elp::lunar_theory_limitations_summary_for_report()
        );
        let lunar_theory_summary =
            render_cli(&["lunar-theory-summary"]).expect("lunar theory summary should render");
        assert!(lunar_theory_summary.contains("ELP lunar theory specification:"));
        assert_eq!(
            lunar_theory_summary,
            pleiades_elp::lunar_theory_summary_for_report()
        );
        let lunar_theory_capability_summary = render_cli(&["lunar-theory-capability-summary"])
            .expect("lunar theory capability summary should render");
        assert!(lunar_theory_capability_summary.contains("lunar capability summary:"));
        assert_eq!(
            lunar_theory_capability_summary,
            pleiades_elp::lunar_theory_capability_summary_for_report()
        );
        let lunar_theory_source_summary = render_cli(&["lunar-theory-source-summary"])
            .expect("lunar theory source summary should render");
        assert!(lunar_theory_source_summary.contains("lunar source selection:"));
        assert_eq!(
            lunar_theory_source_summary,
            pleiades_elp::lunar_theory_source_summary_for_report()
        );
        let lunar_theory_catalog_summary = render_cli(&["lunar-theory-catalog-summary"])
            .expect("lunar theory catalog summary should render");
        assert_eq!(
            lunar_theory_catalog_summary,
            pleiades_elp::lunar_theory_catalog_summary_for_report()
        );
        let lunar_theory_catalog_validation_summary =
            render_cli(&["lunar-theory-catalog-validation-summary"])
                .expect("lunar theory catalog validation summary should render");
        assert_eq!(
            lunar_theory_catalog_validation_summary,
            pleiades_elp::lunar_theory_catalog_validation_summary_for_report()
        );
        let selected_asteroid_boundary_summary =
            render_cli(&["selected-asteroid-boundary-summary"])
                .expect("selected asteroid boundary summary should render");
        assert!(selected_asteroid_boundary_summary.contains("Selected asteroid boundary evidence"));
        assert!(selected_asteroid_boundary_summary.contains("2451914.5"));
        assert!(
            selected_asteroid_boundary_summary.starts_with("Selected asteroid boundary evidence:")
        );
        let selected_asteroid_bridge_summary = render_cli(&["selected-asteroid-bridge-summary"])
            .expect("selected asteroid bridge summary should render");
        assert!(selected_asteroid_bridge_summary.contains("Selected asteroid bridge evidence"));
        assert!(selected_asteroid_bridge_summary.contains("2451915.0"));
        assert_eq!(
            selected_asteroid_bridge_summary,
            pleiades_jpl::selected_asteroid_bridge_summary_for_report()
        );
        assert_eq!(
            render_cli(&["selected-asteroid-bridge-summary", "extra"])
                .expect_err("selected asteroid bridge summary alias should reject extra arguments"),
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
        let reference_major_body_bridge_summary =
            render_cli(&["reference-snapshot-major-body-bridge-summary"])
                .expect("reference major-body bridge summary should render");
        assert!(
            reference_major_body_bridge_summary.contains("Reference major-body bridge evidence")
        );
        assert!(reference_major_body_bridge_summary.contains("2451915.0"));
        assert_eq!(
            reference_major_body_bridge_summary,
            pleiades_jpl::reference_snapshot_major_body_bridge_summary_for_report()
        );
        let reference_major_body_bridge_alias = render_cli(&["major-body-bridge-summary"])
            .expect("major body bridge alias should render");
        assert_eq!(
            reference_major_body_bridge_alias,
            reference_major_body_bridge_summary
        );
        assert_eq!(
            render_cli(&["major-body-bridge-summary", "extra"])
                .expect_err("major body bridge alias should reject extra arguments"),
            "major-body-bridge-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["reference-snapshot-major-body-bridge-summary", "extra"])
                .expect_err("reference major-body bridge summary should reject extra arguments"),
            "reference-snapshot-major-body-bridge-summary does not accept extra arguments"
        );
        let reference_bridge_day_summary = render_cli(&["reference-snapshot-bridge-day-summary"])
            .expect("reference snapshot bridge day summary should render");
        assert!(reference_bridge_day_summary.contains("Reference snapshot bridge day:"));
        assert!(reference_bridge_day_summary.contains("2451914.0"));
        assert_eq!(
            reference_bridge_day_summary,
            pleiades_jpl::reference_snapshot_bridge_day_summary_for_report()
        );
        assert_eq!(
            render_cli(&["reference-snapshot-bridge-day-summary", "extra"])
                .expect_err("reference snapshot bridge day summary should reject extra arguments"),
            "reference-snapshot-bridge-day-summary does not accept extra arguments"
        );
        let bridge_day_alias =
            render_cli(&["bridge-day-summary"]).expect("bridge day alias should render");
        assert_eq!(bridge_day_alias, reference_bridge_day_summary);
        assert_eq!(
            render_cli(&["bridge-day-summary", "extra"])
                .expect_err("bridge day alias should reject extra arguments"),
            "bridge-day-summary does not accept extra arguments"
        );
        let bridge_day_major_alias = render_cli(&["2451914-major-body-bridge-day-summary"])
            .expect("2451914 major body bridge-day alias should render");
        assert_eq!(
            bridge_day_major_alias,
            pleiades_jpl::reference_snapshot_2451914_major_body_bridge_day_summary_for_report()
        );
        assert_eq!(
            render_cli(&["2451914-major-body-bridge-day-summary", "extra"])
                .expect_err("2451914 major body bridge-day alias should reject extra arguments"),
            "reference-snapshot-2451914-major-body-bridge-day-summary does not accept extra arguments"
        );
        let selected_asteroid_dense_boundary_summary =
            render_cli(&["reference-snapshot-selected-asteroid-dense-boundary-summary"])
                .expect("selected asteroid dense boundary summary should render");
        assert!(selected_asteroid_dense_boundary_summary
            .contains("Selected asteroid dense boundary evidence"));
        assert!(selected_asteroid_dense_boundary_summary.contains("2451916.5"));
        assert_eq!(
            selected_asteroid_dense_boundary_summary,
            pleiades_jpl::selected_asteroid_dense_boundary_summary_for_report()
        );
        let selected_asteroid_dense_boundary_alias =
            render_cli(&["selected-asteroid-dense-boundary-summary"])
                .expect("selected asteroid dense boundary alias should render");
        assert_eq!(
            selected_asteroid_dense_boundary_alias,
            selected_asteroid_dense_boundary_summary
        );
        assert_eq!(
            render_cli(&["selected-asteroid-dense-boundary-summary", "extra"])
                .expect_err("selected asteroid dense boundary alias should reject extra arguments"),
            "selected-asteroid-dense-boundary-summary does not accept extra arguments"
        );
        let selected_asteroid_terminal_boundary_summary =
            render_cli(&["reference-snapshot-selected-asteroid-terminal-boundary-summary"])
                .expect("selected asteroid terminal boundary summary should render");
        assert!(selected_asteroid_terminal_boundary_summary.contains("terminal boundary evidence"));
        assert!(selected_asteroid_terminal_boundary_summary.contains("2500-01-01"));
        assert_eq!(
            selected_asteroid_terminal_boundary_summary,
            pleiades_jpl::selected_asteroid_terminal_boundary_summary_for_report()
        );
        let selected_asteroid_terminal_boundary_alias =
            render_cli(&["selected-asteroid-terminal-boundary-summary"])
                .expect("selected asteroid terminal boundary alias should render");
        assert_eq!(
            selected_asteroid_terminal_boundary_alias,
            selected_asteroid_terminal_boundary_summary
        );
        assert_eq!(
            render_cli(&["selected-asteroid-terminal-boundary-summary", "extra"]).expect_err(
                "selected asteroid terminal boundary alias should reject extra arguments"
            ),
            "selected-asteroid-terminal-boundary-summary does not accept extra arguments"
        );
        let selected_asteroid_source_evidence_summary =
            render_cli(&["selected-asteroid-source-evidence-summary"])
                .expect("selected asteroid source evidence summary should render");
        assert!(
            selected_asteroid_source_evidence_summary.contains("Selected asteroid source evidence")
        );
        assert!(selected_asteroid_source_evidence_summary.contains("Ceres"));
        assert_eq!(
            selected_asteroid_source_evidence_summary,
            pleiades_jpl::selected_asteroid_source_evidence_summary_for_report()
        );
        assert_eq!(
            render_cli(&["selected-asteroid-source-summary"])
                .expect("selected asteroid source summary alias should render"),
            selected_asteroid_source_evidence_summary
        );

        let selected_asteroid_source_window_summary =
            render_cli(&["selected-asteroid-source-window-summary"])
                .expect("selected asteroid source window summary should render");
        assert!(
            selected_asteroid_source_window_summary.contains("Selected asteroid source windows")
        );
        assert!(selected_asteroid_source_window_summary.contains("Ceres"));
        assert_eq!(
            selected_asteroid_source_window_summary,
            pleiades_jpl::selected_asteroid_source_window_summary_for_report()
        );
        assert_eq!(
            render_cli(&["selected-asteroid-source-window"])
                .expect("selected asteroid source window alias should render"),
            selected_asteroid_source_window_summary
        );
        assert_eq!(
            render_cli(&["selected-asteroid-source-window", "extra"])
                .expect_err("selected asteroid source window alias should reject extra arguments"),
            "selected-asteroid-source-window does not accept extra arguments"
        );
        let selected_asteroid_batch_parity_summary =
            render_cli(&["selected-asteroid-batch-parity-summary"])
                .expect("selected asteroid batch parity summary should render");
        assert!(selected_asteroid_batch_parity_summary.contains("Selected asteroid batch parity"));
        assert!(selected_asteroid_batch_parity_summary.contains("batch/single parity preserved"));
        assert_eq!(
            selected_asteroid_batch_parity_summary,
            pleiades_jpl::selected_asteroid_batch_parity_summary_for_report()
        );
        let reference_asteroid_evidence_summary =
            render_cli(&["reference-asteroid-evidence-summary"])
                .expect("reference asteroid evidence summary should render");
        assert!(reference_asteroid_evidence_summary.contains("Selected asteroid evidence:"));
        assert!(reference_asteroid_evidence_summary.contains("Ceres"));
        assert_eq!(
            reference_asteroid_evidence_summary,
            pleiades_jpl::reference_asteroid_evidence_summary_for_report()
        );
        let reference_asteroid_equatorial_evidence_summary =
            render_cli(&["reference-asteroid-equatorial-evidence-summary"])
                .expect("reference asteroid equatorial evidence summary should render");
        assert!(reference_asteroid_equatorial_evidence_summary
            .contains("Selected asteroid equatorial evidence:"));
        assert!(reference_asteroid_equatorial_evidence_summary
            .contains("mean-obliquity equatorial transform"));
        assert_eq!(
            reference_asteroid_equatorial_evidence_summary,
            pleiades_jpl::reference_asteroid_equatorial_evidence_summary_for_report()
        );
        let reference_asteroid_source_window_summary =
            render_cli(&["reference-asteroid-source-window-summary"])
                .expect("reference asteroid source window summary should render");
        assert!(
            reference_asteroid_source_window_summary.contains("Reference asteroid source windows:")
        );
        assert!(reference_asteroid_source_window_summary
            .contains("source-backed samples across 6 bodies and 17 epochs"));
        assert_eq!(
            reference_asteroid_source_window_summary,
            pleiades_jpl::reference_asteroid_source_window_summary_for_report()
        );
        let reference_asteroid_source_summary = render_cli(&["reference-asteroid-source-summary"])
            .expect("reference asteroid source summary should render");
        assert_eq!(
            reference_asteroid_source_summary,
            reference_asteroid_source_window_summary
        );
        let reference_holdout_overlap_summary = render_cli(&["reference-holdout-overlap-summary"])
            .expect("reference/hold-out overlap summary should render");
        assert!(reference_holdout_overlap_summary.contains("Reference/hold-out overlap:"));
        assert!(reference_holdout_overlap_summary.contains("shared body-epoch pairs"));
        assert_eq!(
            reference_holdout_overlap_summary,
            pleiades_jpl::reference_holdout_overlap_summary_for_report()
        );
        let holdout_overlap_summary =
            render_cli(&["holdout-overlap-summary"]).expect("hold-out overlap alias should render");
        assert_eq!(holdout_overlap_summary, reference_holdout_overlap_summary);
        assert_eq!(
            render_cli(&["holdout-overlap-summary", "extra"])
                .expect_err("hold-out overlap alias should reject extra arguments"),
            "holdout-overlap-summary does not accept extra arguments"
        );
        let independent_holdout_source_window_summary =
            render_cli(&["independent-holdout-source-window-summary"])
                .expect("independent hold-out source window summary should render");
        assert!(independent_holdout_source_window_summary
            .contains("Independent hold-out source windows:"));
        assert!(independent_holdout_source_window_summary.contains("source-backed samples"));
        assert_eq!(
            independent_holdout_source_window_summary,
            pleiades_jpl::independent_holdout_snapshot_source_window_summary_for_report()
        );
        let independent_holdout = render_cli(&["independent-holdout-summary"])
            .expect("independent hold-out summary should render");
        assert!(independent_holdout.contains("JPL independent hold-out:"));
        assert!(independent_holdout.contains("transparency evidence only"));
        assert_eq!(
            independent_holdout,
            pleiades_jpl::jpl_independent_holdout_summary_for_report()
        );

        let independent_holdout_source_summary =
            render_cli(&["independent-holdout-source-summary"])
                .expect("independent hold-out source summary should render");
        assert!(independent_holdout_source_summary.contains("Independent hold-out source:"));
        assert!(independent_holdout_source_summary.contains("hold-out source"));
        assert_eq!(
            independent_holdout_source_summary,
            pleiades_jpl::independent_holdout_source_summary_for_report()
        );
        let independent_holdout_high_curvature_summary =
            render_cli(&["independent-holdout-high-curvature-summary"])
                .expect("independent hold-out high-curvature summary should render");
        assert!(independent_holdout_high_curvature_summary
            .contains("JPL independent hold-out high-curvature evidence:"));
        assert!(independent_holdout_high_curvature_summary
            .contains("high-curvature interpolation window"));
        assert_eq!(
            independent_holdout_high_curvature_summary,
            pleiades_jpl::independent_holdout_high_curvature_summary_for_report()
        );
        let independent_holdout_batch_parity_summary =
            render_cli(&["independent-holdout-batch-parity-summary"])
                .expect("independent hold-out batch parity summary should render");
        assert!(independent_holdout_batch_parity_summary
            .contains("JPL independent hold-out batch parity:"));
        assert!(independent_holdout_batch_parity_summary.contains("single-query parity=preserved"));
        assert_eq!(
            independent_holdout_batch_parity_summary,
            pleiades_jpl::independent_holdout_snapshot_batch_parity_summary_for_report()
        );
        let independent_holdout_equatorial_parity_summary =
            render_cli(&["independent-holdout-equatorial-parity-summary"])
                .expect("independent hold-out equatorial parity summary should render");
        assert!(independent_holdout_equatorial_parity_summary
            .contains("JPL independent hold-out equatorial parity:"));
        assert!(independent_holdout_equatorial_parity_summary
            .contains("mean-obliquity transform against the checked-in ecliptic fixture"));
        assert_eq!(
            independent_holdout_equatorial_parity_summary,
            pleiades_jpl::independent_holdout_snapshot_equatorial_parity_summary_for_report()
        );
        let house_validation_summary = render_cli(&["house-validation-summary"])
            .expect("house validation summary should render");
        assert!(house_validation_summary.contains("House validation corpus: 9 scenarios"));
        assert!(house_validation_summary
            .contains("formula families: Equal, Whole Sign, Quadrant, Equatorial projection"));
        assert!(house_validation_summary
            .contains("latitude-sensitive systems: Koch, Placidus, Topocentric"));
        assert_eq!(
            house_validation_summary,
            house_validation_summary_line_for_report(&house_validation_report())
        );
        assert_eq!(
            render_cli(&["house-validation-summary", "extra"])
                .expect_err("house validation summary should reject extra arguments"),
            "house-validation-summary does not accept extra arguments"
        );
        let house_validation_alias =
            render_cli(&["house-validation"]).expect("house validation alias should render");
        assert_eq!(house_validation_alias, house_validation_summary);
        assert_eq!(
            render_cli(&["house-validation", "extra"])
                .expect_err("house validation alias should reject extra arguments"),
            "house-validation does not accept extra arguments"
        );
        let house_formula_families_summary = render_cli(&["house-formula-families-summary"])
            .expect("house formula families summary should render");
        assert!(house_formula_families_summary.contains("Equal"));
        assert!(house_formula_families_summary.contains("Whole Sign"));
        assert!(house_formula_families_summary.contains("Quadrant"));
        assert_eq!(
            house_formula_families_summary,
            validate_render_cli(&["house-formula-families-summary"])
                .expect("validate facade should render house formula families summary")
        );
        assert_eq!(
            render_cli(&["house-formula-families"])
                .expect("house formula families alias should render"),
            house_formula_families_summary
        );
        let house_latitude_sensitive_summary = render_cli(&["house-latitude-sensitive-summary"])
            .expect("latitude-sensitive house systems summary should render");
        assert_eq!(
            house_latitude_sensitive_summary,
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        );
        assert_eq!(
            house_latitude_sensitive_summary,
            validate_render_cli(&["house-latitude-sensitive-summary"])
                .expect("validate facade should render latitude-sensitive house systems summary")
        );
        assert_eq!(
            render_cli(&["house-latitude-sensitive"])
                .expect("latitude-sensitive house systems alias should render"),
            house_latitude_sensitive_summary
        );
        assert_eq!(
            render_cli(&["house-latitude-sensitive-summary", "extra"]).expect_err(
                "latitude-sensitive house systems summary should reject extra arguments"
            ),
            "house-latitude-sensitive-summary does not accept extra arguments"
        );
        let house_code_aliases_summary = render_cli(&["house-code-aliases-summary"])
            .expect("house code aliases summary should render");
        assert!(house_code_aliases_summary.contains("P -> Placidus"));
        assert_eq!(
            house_code_aliases_summary,
            validate_render_cli(&["house-code-aliases-summary"])
                .expect("validate facade should render house code aliases summary")
        );
        assert_eq!(
            render_cli(&["house-code-alias-summary"])
                .expect("house code alias shorthand should render"),
            house_code_aliases_summary
        );
        assert_eq!(
            render_cli(&["house-code-alias-summary", "extra"])
                .expect_err("house code alias shorthand should reject extra arguments"),
            "house-code-alias-summary does not accept extra arguments"
        );
        let catalog_inventory_summary = render_cli(&["catalog-inventory-summary"])
            .expect("catalog inventory summary should render");
        assert!(catalog_inventory_summary.contains("Compatibility catalog inventory:"));
        assert_eq!(
            catalog_inventory_summary,
            validate_render_cli(&["catalog-inventory-summary"])
                .expect("validate facade should render catalog inventory summary")
        );
        assert_eq!(
            render_cli(&["catalog-inventory"]).expect("catalog inventory alias should render"),
            catalog_inventory_summary
        );
        let ayanamsa_catalog_validation_summary_rendered =
            render_cli(&["ayanamsa-catalog-validation-summary"])
                .expect("ayanamsa catalog validation summary should render");
        assert_eq!(
            render_cli(&["ayanamsa-catalog-validation"]).unwrap(),
            ayanamsa_catalog_validation_summary_rendered
        );
        assert!(ayanamsa_catalog_validation_summary_rendered
            .contains("ayanamsa catalog validation: ok"));
        assert!(ayanamsa_catalog_validation_summary_rendered.contains("baseline=5, release=54"));
        assert!(ayanamsa_catalog_validation_summary_rendered.contains("custom-definition-only="));
        let ayanamsa_metadata_coverage_summary =
            render_cli(&["ayanamsa-metadata-coverage-summary"])
                .expect("ayanamsa metadata coverage summary should render");
        assert_eq!(
            render_cli(&["ayanamsa-metadata-coverage"]).unwrap(),
            ayanamsa_metadata_coverage_summary
        );
        assert!(ayanamsa_metadata_coverage_summary.contains("ayanamsa sidereal metadata:"));
        assert_eq!(
            ayanamsa_metadata_coverage_summary,
            super::validate_render_cli(&["ayanamsa-metadata-coverage-summary"])
                .expect("validation ayanamsa metadata coverage summary should render")
        );
        assert_eq!(
            render_cli(&["ayanamsa-metadata-coverage-summary", "extra"])
                .expect_err("ayanamsa metadata coverage summary should reject extra arguments"),
            "ayanamsa-metadata-coverage-summary does not accept extra arguments"
        );
        let ayanamsa_reference_offsets_summary =
            render_cli(&["ayanamsa-reference-offsets-summary"])
                .expect("ayanamsa reference offsets summary should render");
        assert_eq!(
            render_cli(&["ayanamsa-reference-offsets"]).unwrap(),
            ayanamsa_reference_offsets_summary
        );
        assert!(ayanamsa_reference_offsets_summary
            .contains("Ayanamsa reference offsets: representative zero-point examples:"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("True Revati: epoch=JD 1926902.658267; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("True Mula: epoch=JD 1805889.671313; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("True Pushya: epoch=JD 1855769.248315; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Udayagiri: epoch=JD 1825235.164583; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Hipparchus: epoch=JD 1674484; offset=-9.333333333333334°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Djwhal Khul: epoch=JD 1706703.948006; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Suryasiddhanta (Mean Sun): epoch=JD 1909045.584433; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Aryabhata (Mean Sun): epoch=JD 1909650.815331; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Babylonian (Kugler 2): epoch=JD 1797039.20682; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Babylonian (Kugler 3): epoch=JD 1774637.420172; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Babylonian (Huber): epoch=JD 1721171.5; offset=-0.12055555555555555°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Babylonian (Eta Piscium): epoch=JD 1807871.964797; offset=0°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Babylonian (Aldebaran): epoch=JD 1801643.133503; offset=0°"));
        assert_eq!(
            ayanamsa_reference_offsets_summary,
            super::validate_render_cli(&["ayanamsa-reference-offsets-summary"])
                .expect("validation ayanamsa reference offsets summary should render")
        );
        let reference_high_curvature_summary = render_cli(&["reference-high-curvature-summary"])
            .expect("reference high-curvature summary should render");
        assert!(reference_high_curvature_summary
            .contains("Reference major-body high-curvature evidence:"));
        assert_eq!(
            reference_high_curvature_summary,
            super::validate_render_cli(&["reference-high-curvature-summary"])
                .expect("validation high-curvature summary should render")
        );
        assert_eq!(
            reference_high_curvature_summary,
            super::validate_render_cli(&["high-curvature-summary"])
                .expect("validation high-curvature summary alias should render")
        );
        let reference_major_body_boundary_summary =
            render_cli(&["reference-snapshot-major-body-boundary-summary"])
                .expect("reference major-body boundary summary should render");
        assert!(reference_major_body_boundary_summary
            .contains("Reference major-body boundary evidence:"));
        assert_eq!(
            reference_major_body_boundary_summary,
            super::validate_render_cli(&["reference-snapshot-major-body-boundary-summary"])
                .expect("validation major-body boundary summary should render")
        );
        let major_body_boundary_alias = render_cli(&["major-body-boundary-summary"])
            .expect("major-body boundary alias should render");
        assert_eq!(
            major_body_boundary_alias,
            reference_major_body_boundary_summary
        );
        let reference_mars_jupiter_boundary_summary =
            render_cli(&["reference-snapshot-mars-jupiter-boundary-summary"])
                .expect("reference Mars/Jupiter boundary summary should render");
        assert!(reference_mars_jupiter_boundary_summary
            .contains("Reference Mars/Jupiter boundary evidence:"));
        assert_eq!(
            reference_mars_jupiter_boundary_summary,
            super::validate_render_cli(&["reference-snapshot-mars-jupiter-boundary-summary"])
                .expect("validation Mars/Jupiter boundary summary should render")
        );
        let mars_jupiter_boundary_alias = render_cli(&["mars-jupiter-boundary-summary"])
            .expect("Mars/Jupiter boundary alias should render");
        assert_eq!(
            mars_jupiter_boundary_alias,
            reference_mars_jupiter_boundary_summary
        );
        let reference_mars_outer_boundary_summary =
            render_cli(&["reference-snapshot-mars-outer-boundary-summary"])
                .expect("reference Mars outer-boundary summary should render");
        assert!(reference_mars_outer_boundary_summary
            .contains("Reference Mars outer-boundary evidence:"));
        assert_eq!(
            reference_mars_outer_boundary_summary,
            super::validate_render_cli(&["reference-snapshot-mars-outer-boundary-summary"])
                .expect("validation Mars outer-boundary summary should render")
        );
        let mars_outer_boundary_alias = render_cli(&["mars-outer-boundary-summary"])
            .expect("Mars outer-boundary alias should render");
        assert_eq!(
            mars_outer_boundary_alias,
            reference_mars_outer_boundary_summary
        );
        let reference_major_body_boundary_window_summary =
            render_cli(&["reference-snapshot-major-body-boundary-window-summary"])
                .expect("reference major-body boundary window summary should render");
        assert!(reference_major_body_boundary_window_summary
            .contains("Reference major-body boundary windows:"));
        assert_eq!(
            reference_major_body_boundary_window_summary,
            super::validate_render_cli(&["reference-snapshot-major-body-boundary-window-summary"])
                .expect("validation major-body boundary window summary should render")
        );
        let major_body_boundary_window_alias = render_cli(&["major-body-boundary-window-summary"])
            .expect("major-body boundary window alias should render");
        assert_eq!(
            major_body_boundary_window_alias,
            reference_major_body_boundary_window_summary
        );
        let reference_high_curvature_window_summary =
            render_cli(&["reference-high-curvature-window-summary"])
                .expect("reference high-curvature window summary should render");
        assert!(reference_high_curvature_window_summary
            .contains("Reference major-body high-curvature windows:"));
        assert_eq!(
            reference_high_curvature_window_summary,
            super::validate_render_cli(&["reference-high-curvature-window-summary"])
                .expect("validation high-curvature window summary should render")
        );
        assert_eq!(
            reference_high_curvature_window_summary,
            super::validate_render_cli(&["high-curvature-window-summary"])
                .expect("validation high-curvature window summary alias should render")
        );
        let reference_high_curvature_epoch_coverage_summary =
            render_cli(&["reference-high-curvature-epoch-coverage-summary"])
                .expect("reference high-curvature epoch coverage summary should render");
        assert!(reference_high_curvature_epoch_coverage_summary
            .contains("Reference major-body high-curvature epoch coverage:"));
        assert_eq!(
            reference_high_curvature_epoch_coverage_summary,
            super::validate_render_cli(&["reference-high-curvature-epoch-coverage-summary"])
                .expect("validation high-curvature epoch coverage summary should render")
        );
        assert_eq!(
            reference_high_curvature_epoch_coverage_summary,
            super::validate_render_cli(&["high-curvature-epoch-coverage-summary"])
                .expect("validation high-curvature epoch coverage summary alias should render")
        );
        let boundary_epoch_coverage_summary =
            render_cli(&["reference-snapshot-boundary-epoch-coverage-summary"])
                .expect("reference snapshot boundary epoch coverage summary should render");
        let boundary_epoch_coverage_alias = render_cli(&["boundary-epoch-coverage-summary"])
            .expect("boundary epoch coverage summary alias should render");
        assert!(
            boundary_epoch_coverage_summary.contains("Reference snapshot boundary epoch coverage:")
        );
        assert!(boundary_epoch_coverage_summary.contains(
            "JD 2451915.5 (TDB): 16 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        ));
        assert_eq!(
            boundary_epoch_coverage_alias,
            boundary_epoch_coverage_summary
        );
        assert_eq!(
            boundary_epoch_coverage_summary,
            super::validate_render_cli(&["reference-snapshot-boundary-epoch-coverage-summary"])
                .expect("validation boundary epoch coverage summary should render")
        );
        let sparse_boundary_summary = render_cli(&["reference-snapshot-sparse-boundary-summary"])
            .expect("reference snapshot sparse boundary summary should render");
        let sparse_boundary_alias = render_cli(&["sparse-boundary-summary"])
            .expect("sparse boundary summary alias should render");
        let boundary_day_alias = render_cli(&["boundary-day-summary"])
            .expect("boundary day summary alias should render");
        assert!(sparse_boundary_summary.contains("Reference snapshot boundary day:"));
        assert!(sparse_boundary_summary.contains(
            "JD 2451915.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        ));
        assert_eq!(sparse_boundary_alias, sparse_boundary_summary);
        assert_eq!(boundary_day_alias, sparse_boundary_summary);
        assert_eq!(
            sparse_boundary_summary,
            super::validate_render_cli(&["reference-snapshot-sparse-boundary-summary"])
                .expect("validation sparse boundary summary should render")
        );
        assert_eq!(
            boundary_day_alias,
            super::validate_render_cli(&["boundary-day-summary"])
                .expect("validation boundary day summary should render")
        );
        assert_eq!(
            render_cli(&["boundary-day-summary", "extra"])
                .expect_err("boundary day summary alias should reject extra arguments"),
            "boundary-day-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["reference-snapshot-sparse-boundary-summary", "extra"]).expect_err(
                "reference snapshot sparse boundary summary should reject extra arguments"
            ),
            "reference-snapshot-sparse-boundary-summary does not accept extra arguments"
        );
        let pre_bridge_boundary_summary =
            render_cli(&["reference-snapshot-pre-bridge-boundary-summary"])
                .expect("reference snapshot pre-bridge boundary summary should render");
        let pre_bridge_boundary_alias = render_cli(&["pre-bridge-boundary-summary"])
            .expect("pre-bridge boundary summary alias should render");
        assert!(pre_bridge_boundary_summary.contains("Reference snapshot pre-bridge boundary day:"));
        assert!(pre_bridge_boundary_summary.contains(
            "JD 2451914.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); pre-bridge boundary day"
        ));
        assert_eq!(pre_bridge_boundary_alias, pre_bridge_boundary_summary);
        assert_eq!(
            pre_bridge_boundary_summary,
            super::validate_render_cli(&["reference-snapshot-pre-bridge-boundary-summary"])
                .expect("validation pre-bridge boundary summary should render")
        );
        assert_eq!(
            render_cli(&["reference-snapshot-pre-bridge-boundary-summary", "extra"]).expect_err(
                "reference snapshot pre-bridge boundary summary should reject extra arguments"
            ),
            "reference-snapshot-pre-bridge-boundary-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["pre-bridge-boundary-summary", "extra"])
                .expect_err("pre-bridge boundary summary alias should reject extra arguments"),
            "pre-bridge-boundary-summary does not accept extra arguments"
        );
        let dense_boundary_summary = render_cli(&["reference-snapshot-dense-boundary-summary"])
            .expect("reference snapshot dense boundary summary should render");
        let dense_boundary_alias = render_cli(&["dense-boundary-summary"])
            .expect("dense boundary summary alias should render");
        assert!(dense_boundary_summary.contains("Reference snapshot dense boundary day:"));
        assert!(dense_boundary_summary.contains(
            "JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        ));
        assert_eq!(dense_boundary_alias, dense_boundary_summary);
        assert_eq!(
            dense_boundary_summary,
            super::validate_render_cli(&["reference-snapshot-dense-boundary-summary"])
                .expect("validation dense boundary summary should render")
        );
        assert_eq!(
            render_cli(&["reference-snapshot-dense-boundary-summary", "extra"]).expect_err(
                "reference snapshot dense boundary summary should reject extra arguments"
            ),
            "reference-snapshot-dense-boundary-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["dense-boundary-summary", "extra"])
                .expect_err("dense boundary summary alias should reject extra arguments"),
            "dense-boundary-summary does not accept extra arguments"
        );
        let source_documentation_summary = render_cli(&["source-documentation-summary"])
            .expect("source documentation summary should render");
        assert!(source_documentation_summary.contains("VSOP87 source documentation:"));
        assert_eq!(
            source_documentation_summary,
            super::validate_render_cli(&["source-documentation-summary"])
                .expect("validation source documentation summary should render")
        );
        let source_documentation_alias = render_cli(&["source-documentation"])
            .expect("source documentation alias should render");
        assert_eq!(
            source_documentation_alias,
            super::validate_render_cli(&["source-documentation"])
                .expect("validation source documentation alias should render")
        );
        assert_eq!(
            render_cli(&["source-documentation", "extra"])
                .expect_err("source documentation alias should reject extra arguments"),
            "source-documentation does not accept extra arguments"
        );
        let source_documentation_health_summary =
            render_cli(&["source-documentation-health-summary"])
                .expect("source documentation health summary should render");
        assert!(source_documentation_health_summary.contains("VSOP87 source documentation health:"));
        assert_eq!(
            source_documentation_health_summary,
            super::validate_render_cli(&["source-documentation-health-summary"])
                .expect("validation source documentation health summary should render")
        );
        let source_documentation_health_alias = render_cli(&["source-documentation-health"])
            .expect("source documentation health alias should render");
        assert_eq!(
            source_documentation_health_alias,
            source_documentation_health_summary
        );
        assert_eq!(
            render_cli(&["source-documentation-health", "extra"])
                .expect_err("source documentation health alias should reject extra arguments"),
            "source-documentation-health does not accept extra arguments"
        );
        let source_audit_summary =
            render_cli(&["source-audit-summary"]).expect("source audit summary should render");
        assert!(source_audit_summary.contains("VSOP87 source audit:"));
        assert_eq!(
            source_audit_summary,
            super::validate_render_cli(&["source-audit-summary"])
                .expect("validation source audit summary should render")
        );
        let source_audit_alias =
            render_cli(&["source-audit"]).expect("source audit alias should render");
        assert_eq!(source_audit_alias, source_audit_summary);
        assert_eq!(
            render_cli(&["source-audit", "extra"])
                .expect_err("source audit alias should reject extra arguments"),
            "source-audit does not accept extra arguments"
        );
        let generated_binary_audit_summary = render_cli(&["generated-binary-audit-summary"])
            .expect("generated binary audit summary should render");
        assert!(generated_binary_audit_summary.contains("VSOP87 generated binary audit:"));
        assert_eq!(
            generated_binary_audit_summary,
            super::validate_render_cli(&["generated-binary-audit-summary"])
                .expect("validation generated binary audit summary should render")
        );
        let generated_binary_audit_alias = render_cli(&["generated-binary-audit"])
            .expect("generated binary audit alias should render");
        assert_eq!(generated_binary_audit_alias, generated_binary_audit_summary);
        assert_eq!(
            render_cli(&["generated-binary-audit", "extra"])
                .expect_err("generated binary audit alias should reject extra arguments"),
            "generated-binary-audit does not accept extra arguments"
        );
        assert!(release_summary.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        }));
        assert!(release_summary
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(release_summary
            .lines()
            .any(|line| line == packaged_artifact_access_report_line()));
        assert!(release_summary.lines().any(|line| {
            line == "Packaged-artifact generation policy: adjacent same-body linear segments; bodies with a single sampled epoch use point segments; multi-epoch non-lunar bodies are fit with linear segments between adjacent same-body source epochs; the Moon uses overlapping three-point spans with quadratic residual corrections to keep the high-curvature fit compact"
        }));
        assert!(release_summary.lines().any(|line| {
            line == format!(
                "Packaged frame treatment: {}",
                pleiades_data::packaged_frame_treatment_summary_for_report()
            )
        }));
        assert!(release_summary.contains(
            "Packaged lookup epoch policy: TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
        ));
        assert!(release_summary.lines().any(|line| {
            line == format!(
                "Packaged batch parity: {}",
                pleiades_data::packaged_mixed_tt_tdb_batch_parity_summary_for_report()
            )
        }));
        assert!(release_summary.contains(
            "Packaged batch parity: Packaged mixed TT/TDB batch parity: 11 requests across 11 bodies, TT requests=6, TDB requests=5; quality counts: Exact=0, Interpolated=11, Approximate=0, Unknown=0; order=preserved, single-query parity=preserved"
        ));
        assert!(release_summary.contains("Lunar high-curvature equatorial continuity evidence"));
        assert!(release_summary.contains("Artifact inspection:"));
        assert!(release_summary.contains("Release gate reminders:"));
        assert!(release_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(release_summary
            .lines()
            .any(|line| line == "Release notes summary: release-notes-summary"));
        assert!(release_summary
            .contains("lunar source selection: Compact Meeus-style truncated lunar baseline"));
        assert!(release_summary.contains("Wang"));
        assert!(release_summary.contains("Aries houses"));
        assert!(release_summary.contains("Fagan/Bradley"));
        assert!(release_summary.contains("Usha Shashi"));
        assert!(release_summary.contains("Galactic Center (Mula/Wilhelm)"));
        assert!(release_summary.contains("Mula Wilhelm"));
        assert!(release_summary.contains("Wilhelm"));
        assert!(release_summary.contains("Galactic Equator (Fiorenza)"));
        assert!(release_summary.contains("coverage=Luminaries: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence, bodies=2 (Sun, Moon), samples="));
        assert!(release_summary.contains("JPL interpolation posture: source="));
        assert!(release_summary.lines().any(|line| {
            line == "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        }));
        assert!(release_summary.contains(
            "Validation report summary: validation-report-summary / validation-summary / report-summary"
        ));
        assert!(release_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_summary.contains("Release bundle verification: verify-release-bundle"));
        assert!(release_summary
            .lines()
            .any(|line| line == "Workspace audit: workspace-audit / audit"));
        assert!(release_summary
            .contains("[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile"));
        assert!(release_summary.lines().any(|line| {
            line == "Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
        }));
        assert!(release_summary.contains("Release checklist summary: release-checklist-summary"));
        assert!(release_summary.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));
        assert!(release_summary.contains("See release-notes and release-checklist"));

        let artifact_summary =
            render_cli(&["artifact-summary"]).expect("artifact summary should render");
        assert!(artifact_summary.contains("Artifact summary"));
        assert!(artifact_summary.contains("Artifact output support:"));
        assert!(artifact_summary.contains("Artifact boundary envelope"));
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

        let packaged_artifact_speed_policy =
            render_cli(&["packaged-artifact-speed-policy-summary"])
                .expect("packaged artifact speed policy summary should render");
        assert!(packaged_artifact_speed_policy.contains("Packaged-artifact speed policy: "));
        assert!(packaged_artifact_speed_policy
            .contains("Unsupported; motion output support=unsupported"));
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
        assert!(packaged_artifact_generation_manifest
            .contains("Packaged artifact generation manifest:"));
        assert_eq!(
            packaged_artifact_generation_manifest,
            packaged_artifact_generation_manifest_for_report()
        );

        let packaged_artifact_regeneration =
            render_cli(&["packaged-artifact-regeneration-summary"])
                .expect("packaged artifact regeneration summary should render");
        assert!(packaged_artifact_regeneration.contains("Packaged-artifact regeneration: "));
        assert!(packaged_artifact_regeneration.contains("profile id="));
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
        assert!(
            packaged_artifact_target_threshold.contains("Packaged-artifact target thresholds: ")
        );
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

        let packaged_artifact_regeneration =
            render_cli(&["packaged-artifact-regeneration-summary"])
                .expect("packaged artifact regeneration summary should render");
        assert!(packaged_artifact_regeneration.contains("Packaged-artifact regeneration: "));
        assert!(packaged_artifact_regeneration.contains("profile id="));
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
                &["packaged-artifact-generation-manifest-summary", "extra"][..],
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
        let regenerated = render_cli(&[
            "regenerate-packaged-artifact",
            "--out",
            &artifact_fixture_path_string,
        ])
        .expect("packaged artifact regeneration should render");
        assert!(regenerated.contains("Packaged artifact regenerated"));
        assert!(regenerated.contains("stage-5 packaged-data draft"));
        assert!(regenerated.contains("checksum=0x"));
        assert!(regenerated.contains("generation policy: adjacent same-body linear segments"));
        assert!(regenerated.contains("11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"));
        assert!(regenerated.contains("Packaged artifact regeneration source:"));
        assert!(regenerated.contains("Reference snapshot coverage:"));
        assert!(artifact_fixture_path.exists());
        let written = std::fs::read(&artifact_fixture_path)
            .expect("packaged artifact regeneration should write bytes");
        let expected = regenerate_packaged_artifact()
            .encode()
            .expect("regenerated packaged artifact should encode");
        assert_eq!(written, expected);

        let output_alias_fixture_path =
            artifact_fixture_dir.join("packaged-artifact-output-alias.bin");
        let output_alias_fixture_path_string = output_alias_fixture_path.display().to_string();
        let regenerated_output = render_cli(&[
            "regenerate-packaged-artifact",
            "--output",
            &output_alias_fixture_path_string,
        ])
        .expect("packaged artifact regeneration should accept --output");
        assert!(regenerated_output.contains("Packaged artifact regenerated"));
        assert!(regenerated_output.contains("stage-5 packaged-data draft"));
        assert!(output_alias_fixture_path.exists());
        let output_written = std::fs::read(&output_alias_fixture_path)
            .expect("packaged artifact regeneration should write the output alias path");
        assert_eq!(output_written, expected);

        let positional_fixture_path = artifact_fixture_dir.join("packaged-artifact-positional.bin");
        let positional_fixture_path_string = positional_fixture_path.display().to_string();
        let regenerated_positional = render_cli(&[
            "regenerate-packaged-artifact",
            &positional_fixture_path_string,
        ])
        .expect("packaged artifact regeneration should accept a positional output path");
        assert!(regenerated_positional.contains("Packaged artifact regenerated"));
        assert!(regenerated_positional.contains(&positional_fixture_path_string));
        assert!(positional_fixture_path.exists());
        let positional_written = std::fs::read(&positional_fixture_path)
            .expect("packaged artifact regeneration should write the positional path");
        assert_eq!(positional_written, expected);

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
        assert!(workspace_audit.contains("no mandatory native build hooks detected"));

        let audit = render_cli(&["audit"]).expect("audit alias should render through the CLI");
        assert!(audit.contains("Workspace audit"));
        assert!(audit.contains("no mandatory native build hooks detected"));

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

        let report = render_cli(&["report", "--rounds", "10"])
            .expect("report should render through the primary CLI");
        assert!(report.contains("Validation report"));
        assert!(report.contains("Comparison corpus"));
        assert!(report.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day remains in the release-grade comparison window"));
        assert!(report.contains("Benchmark corpus"));
        assert!(report.contains("Packaged-data benchmark corpus"));

        let generate_report = render_cli(&["generate-report", "--rounds", "10"])
            .expect("generate-report should render through the primary CLI");
        assert!(generate_report.contains("Validation report"));
        assert!(generate_report.contains("Comparison corpus"));

        let validation_summary =
            render_cli(&["validation-summary"]).expect("validation summary should render");
        assert!(validation_summary.contains("Validation report summary"));
        assert!(validation_summary.contains("Comparison corpus"));
        assert!(validation_summary.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day remains in the release-grade comparison window"));
        assert!(validation_summary.contains("Release bundle verification: verify-release-bundle"));
        assert!(validation_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(validation_summary.contains("Release notes summary: release-notes-summary"));
        assert!(validation_summary.contains("Release checklist summary: release-checklist-summary"));
        assert!(validation_summary.contains("Release summary: release-summary"));
        assert!(validation_summary.contains("House validation corpus"));
        assert!(validation_summary.contains("Benchmark summaries"));
        assert!(validation_summary.contains("Packaged-data benchmark"));

        let validation_summary_rounds = render_cli(&["validation-summary", "--rounds", "10"])
            .expect("validation summary should accept explicit rounds");
        let strip_benchmark_timings = |text: &str| -> String {
            text.lines()
                .filter(|line| {
                    !line.contains("ns/")
                        && !line.contains("throughput")
                        && !line.contains("per second")
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let report_summary_rounds = render_cli(&["report-summary", "--rounds", "10"])
            .expect("report summary should mirror the validation-summary rounds output");
        let validation_report_summary_rounds =
            render_cli(&["validation-report-summary", "--rounds", "10"]).expect(
                "validation-report-summary should mirror the validation-summary rounds output",
            );
        assert_eq!(
            strip_benchmark_timings(&validation_summary_rounds),
            strip_benchmark_timings(&report_summary_rounds)
        );
        assert_eq!(
            strip_benchmark_timings(&validation_summary_rounds),
            strip_benchmark_timings(&validation_report_summary_rounds)
        );
        assert_eq!(
            render_cli(&["validation-summary", "extra"]).unwrap_err(),
            "unknown argument: extra"
        );

        let validation_report_summary = render_cli(&["validation-report-summary"])
            .expect("validation-report-summary should render");
        assert!(validation_report_summary.contains("Validation report summary"));
        assert!(validation_report_summary.contains("Comparison corpus"));
        assert!(validation_report_summary
            .contains("Release bundle verification: verify-release-bundle"));
        assert!(validation_report_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(validation_report_summary.contains("Release notes summary: release-notes-summary"));
        assert!(validation_report_summary
            .contains("Release checklist summary: release-checklist-summary"));
        assert!(validation_report_summary.contains("Release summary: release-summary"));
        assert!(validation_report_summary.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
        assert!(validation_report_summary.lines().any(|line| {
            line == format!(
                "Release profile identifiers: v1 compatibility={}, api-stability={}",
                release_profiles.compatibility_profile_id,
                release_profiles.api_stability_profile_id
            )
        }));
        assert!(validation_report_summary.contains("Benchmark summaries"));
    }

    #[test]
    fn frame_policy_summary_command_renders_the_shared_frame_semantics_block() {
        let rendered =
            render_cli(&["frame-policy-summary"]).expect("frame policy summary should render");
        assert!(rendered.contains("Frame policy summary"));
        assert!(rendered.contains("Frame policy: ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
        assert_eq!(
            render_cli(&["frame-policy"]).expect("frame policy alias should render"),
            rendered
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
                ["native-sidereal-policy-summary", "extra"],
                "native-sidereal-policy-summary does not accept extra arguments",
            ),
            (
                ["native-sidereal-policy", "extra"],
                "native-sidereal-policy does not accept extra arguments",
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
    }

    #[test]
    fn release_profile_identifiers_summary_command_renders_the_shared_release_profile_identifiers_block(
    ) {
        let rendered = render_cli(&["release-profile-identifiers-summary"])
            .expect("release-profile identifiers summary should render");
        assert!(rendered.contains("Release profile identifiers summary"));
        assert!(rendered.contains("Summary line: v1 compatibility="));
        assert!(rendered.contains("Compatibility profile: "));
        assert!(rendered.contains("API stability posture: "));
        assert_eq!(
            rendered,
            super::validate_render_cli(&["release-profile-identifiers-summary"]).unwrap()
        );
        assert_eq!(
            render_cli(&["release-profile-identifiers"])
                .expect("release-profile identifiers alias should render"),
            rendered
        );
        assert_eq!(
            render_cli(&["release-profile-identifiers-summary", "extra"])
                .expect_err("release-profile identifiers summary should reject extra arguments"),
            "release-profile-identifiers-summary does not accept extra arguments"
        );
        assert_eq!(
            render_cli(&["release-profile-identifiers", "extra"])
                .expect_err("release-profile identifiers alias should reject extra arguments"),
            "release-profile-identifiers does not accept extra arguments"
        );
    }

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
        assert!(bundle_dir.join("bundle-manifest.txt").exists());
        assert!(bundle_dir
            .join("reference-snapshot-bridge-day-summary.txt")
            .exists());
        assert!(bundle_dir.join("catalog-inventory-summary.txt").exists());
        assert!(bundle_dir
            .join("custom-definition-ayanamsa-labels-summary.txt")
            .exists());
        assert!(bundle_dir
            .join("compatibility-caveats-summary.txt")
            .exists());
        assert!(bundle_dir.join("request-policy-summary.txt").exists());
        assert!(bundle_dir.join("request-semantics-summary.txt").exists());
        assert!(bundle_dir.join("release-body-claims-summary.txt").exists());
        assert!(bundle_dir.join("pluto-fallback-summary.txt").exists());
        assert!(bundle_dir.join("time-scale-policy-summary.txt").exists());
        assert!(bundle_dir
            .join("utc-convenience-policy-summary.txt")
            .exists());
        assert!(bundle_dir.join("delta-t-policy-summary.txt").exists());
        assert!(bundle_dir
            .join("native-sidereal-policy-summary.txt")
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
            .join("packaged-artifact-access-summary.txt")
            .exists());
        assert!(bundle_dir
            .join("packaged-artifact-production-profile-summary.txt")
            .exists());
        assert!(bundle_dir
            .join("packaged-artifact-target-threshold-summary.txt")
            .exists());
        assert!(bundle_dir
            .join("packaged-artifact-generation-manifest.txt")
            .exists());
        let manifest = std::fs::read_to_string(bundle_dir.join("bundle-manifest.txt"))
            .expect("bundle manifest should be written");
        assert!(manifest.contains("packaged-artifact-access-summary.txt"));
        assert!(manifest.contains("release-body-claims-summary.txt"));
        assert!(manifest.contains("pluto-fallback-summary.txt"));
        assert!(manifest.contains("reference-snapshot-bridge-day-summary.txt"));
        assert!(manifest.contains("reference snapshot bridge day summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("packaged-artifact access summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("packaged-artifact-production-profile-summary.txt"));
        assert!(manifest.contains("packaged-artifact-target-threshold-summary.txt"));
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
        assert!(verified.contains("catalog-inventory-summary.txt"));
        assert!(verified.contains("reference-snapshot-bridge-day-summary.txt"));
        assert!(verified.contains("custom-definition-ayanamsa-labels-summary.txt"));
        assert!(verified.contains("workspace-audit-summary.txt"));
        assert!(verified.contains("request-semantics-summary.txt"));
        assert!(verified.contains("time-scale-policy-summary.txt"));
        assert!(verified.contains("delta-t-policy-summary.txt"));
        assert!(verified.contains("native-sidereal-policy-summary.txt"));
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

    #[test]
    fn unknown_command_is_rejected() {
        let error = render_cli(&["compatibility-profile-snapshot"])
            .expect_err("unknown commands should fail");

        assert!(error.contains("unknown command: compatibility-profile-snapshot"));
        for expected in [
            "compare-backends",
            "compare-backends-audit",
            "compatibility-profile",
            "compatibility-caveats-summary",
            "verify-compatibility-profile",
            "bundle-release",
            "verify-release-bundle",
            "release-notes",
            "release-summary",
            "packaged-lookup-epoch-policy-summary",
            "validate-artifact",
            "workspace-audit",
            "report",
            "chart",
        ] {
            assert!(error.contains(expected), "missing help text for {expected}");
        }
    }

    #[test]
    fn custom_definition_ayanamsa_labels_summary_command_renders_the_labels() {
        let rendered = render_cli(&["custom-definition-ayanamsa-labels-summary"])
            .expect("custom-definition ayanamsa labels summary should render");

        assert_eq!(
            rendered,
            pleiades_validate::render_cli(&["custom-definition-ayanamsa-labels-summary"]).expect(
                "validation front end should render the custom-definition ayanamsa labels summary"
            )
        );
        assert_eq!(
            render_cli(&["custom-definition-ayanamsa-labels"])
                .expect("custom-definition ayanamsa labels alias should render"),
            rendered
        );
    }

    #[test]
    fn release_specific_canonical_name_summary_commands_render_the_labels() {
        let profile = pleiades_core::current_compatibility_profile();

        let house_names = render_cli(&["release-house-system-canonical-names-summary"])
            .expect("release-specific house-system canonical names summary should render");
        assert_eq!(
            house_names,
            pleiades_validate::render_cli(&["release-house-system-canonical-names-summary"]).expect(
                "validation front end should render the release-specific house-system canonical names summary"
            )
        );
        assert_eq!(
            render_cli(&["release-house-system-canonical-names"])
                .expect("release-specific house-system canonical names alias should render"),
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

        let ayanamsa_names = render_cli(&["release-ayanamsa-canonical-names-summary"])
            .expect("release-specific ayanamsa canonical names summary should render");
        assert_eq!(
            ayanamsa_names,
            pleiades_validate::render_cli(&["release-ayanamsa-canonical-names-summary"]).expect(
                "validation front end should render the release-specific ayanamsa canonical names summary"
            )
        );
        assert_eq!(
            render_cli(&["release-ayanamsa-canonical-names"])
                .expect("release-specific ayanamsa canonical names alias should render"),
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
    }

    #[test]
    fn chart_help_text_spells_out_the_shared_request_policy() {
        let help = render_chart(&["--help"]).expect("chart help should render");
        assert!(help.contains(&shared_request_policy_help_block()));
        assert!(help.contains(
            "observer-bearing chart requests stay geocentric and use the observer only for houses"
        ));
        assert!(
            help.contains(pleiades_validate::current_request_surface_summary().chart_help_clause())
        );
    }

    #[test]
    fn help_text_lists_the_packaged_lookup_epoch_policy_summary_command() {
        let help = render_cli(&["help"]).expect("help text should render");
        assert!(help.contains(
            "packaged-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary"
        ));
        assert!(help.contains(
            "packaged-lookup-epoch-policy         Alias for packaged-lookup-epoch-policy-summary"
        ));
        assert!(help.contains(
            "production-generation-summary  Print the compact production-generation coverage summary"
        ));
        assert!(help.contains(
            "production-generation-source-summary  Print the compact production-generation source summary"
        ));
        assert!(help.contains(
            "production-generation-source      Alias for production-generation-source-summary"
        ));
        assert!(help
            .contains("production-generation           Alias for production-generation-summary"));
        assert!(help.contains(
            "production-generation-boundary-window-summary  Print the compact production-generation boundary windows summary"
        ));
        assert!(help.contains(
            "production-generation-boundary-window  Alias for production-generation-boundary-window-summary"
        ));
        assert!(help.contains(
            "compatibility-caveats-summary  Print the compact compatibility caveats summary"
        ));
        assert!(help.contains("compatibility-caveats    Alias for compatibility-caveats-summary"));
        assert!(
            help.contains("workspace-audit-summary   Print the compact workspace audit summary")
        );
        assert!(help.contains("native-dependency-audit-summary  Alias for workspace-audit-summary"));
        assert!(help.contains(
            "catalog-inventory-summary  Print the compact compatibility catalog inventory summary"
        ));
        assert!(help.contains("catalog-inventory        Alias for catalog-inventory-summary"));
        assert!(help.contains(
            "custom-definition-ayanamsa-labels-summary  Print the compact custom-definition ayanamsa labels summary"
        ));
        assert!(help.contains(
            "custom-definition-ayanamsa-labels  Alias for custom-definition-ayanamsa-labels-summary"
        ));
        assert!(help.contains(
            "release-house-system-canonical-names-summary  Print the compact release-specific house-system canonical names summary"
        ));
        assert!(help.contains(
            "release-house-system-canonical-names  Alias for release-house-system-canonical-names-summary"
        ));
        assert!(help.contains(
            "release-ayanamsa-canonical-names-summary  Print the compact release-specific ayanamsa canonical names summary"
        ));
        assert!(help.contains(
            "release-ayanamsa-canonical-names  Alias for release-ayanamsa-canonical-names-summary"
        ));
        assert!(help.contains(
            "house-latitude-sensitive-summary  Print the compact latitude-sensitive house systems summary"
        ));
        assert!(
            help.contains("house-latitude-sensitive  Alias for house-latitude-sensitive-summary")
        );
        assert!(help.contains("profile-summary           Alias for compatibility-profile-summary"));
        assert!(help.contains(
            "release-profile-identifiers  Alias for release-profile-identifiers-summary"
        ));
        assert!(help.contains(
            "artifact-profile-coverage-summary  Print the packaged-artifact profile coverage summary"
        ));
        assert!(help.contains(
            "packaged-artifact-output-support-summary  Print the packaged-artifact output support summary"
        ));
        assert!(help.contains(
            "packaged-artifact-output-support       Alias for packaged-artifact-output-support-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-speed-policy-summary  Print the packaged-artifact speed policy summary"
        ));
        assert!(help.contains(
            "packaged-artifact-speed-policy       Alias for packaged-artifact-speed-policy-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-access-summary  Print the packaged-artifact access summary"
        ));
        assert!(
            help.contains("packaged-artifact-access  Alias for packaged-artifact-access-summary")
        );
        assert!(help.contains(
            "packaged-artifact-path-policy-summary  Alias for packaged-artifact-access-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-path-policy  Alias for packaged-artifact-path-policy-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-storage-summary  Print the packaged-artifact storage/reconstruction summary"
        ));
        assert!(help.contains(
            "packaged-artifact-storage           Alias for packaged-artifact-storage-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-production-profile-summary  Print the packaged-artifact production profile draft summary"
        ));
        assert!(help.contains(
            "packaged-artifact-production-profile  Alias for packaged-artifact-production-profile-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-target-threshold-summary  Print the packaged-artifact target thresholds summary"
        ));
        assert!(help.contains(
            "packaged-artifact-target-threshold  Alias for packaged-artifact-target-threshold-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-target-threshold-scope-envelopes-summary  Print the packaged-artifact target-threshold scope envelopes summary"
        ));
        assert!(help.contains(
            "packaged-artifact-target-threshold-scope-envelopes  Alias for packaged-artifact-target-threshold-scope-envelopes-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-manifest-summary  Print the packaged-artifact generation manifest summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-policy-summary  Print the packaged-artifact generation policy summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-policy     Alias for packaged-artifact-generation-policy-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-residual-summary  Alias for packaged-artifact-generation-residual-bodies-summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-residual-bodies-summary  Print the packaged-artifact generation residual bodies summary"
        ));
        assert!(help.contains(
            "packaged-artifact-regeneration-summary  Print the packaged-artifact regeneration summary"
        ));
        assert!(help.contains(
            "packaged-artifact-regeneration      Alias for packaged-artifact-regeneration-summary"
        ));
        assert!(
            help.contains("packaged-frame-parity-summary  Print the packaged frame parity summary")
        );
        assert!(help.contains(
            "packaged-frame-treatment-summary  Print the packaged frame treatment summary"
        ));
        assert!(help.contains(
            "comparison-envelope-summary  Print the compact comparison envelope summary"
        ));
        assert!(help.contains(
            "reference-snapshot-1749-major-body-boundary-summary  Print the compact reference 1749 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-1600-selected-body-boundary-summary  Print the compact reference 1600 selected-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-1750-selected-body-boundary-summary  Print the compact reference 1750 selected-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2200-selected-body-boundary-summary  Print the compact reference 2200 selected-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2524593-selected-body-boundary-summary  Print the compact reference 2524593 selected-body boundary evidence summary"
        ));
        assert!(help.contains(
            "2524593-selected-body-boundary-summary  Alias for reference-snapshot-2524593-selected-body-boundary-summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2634167-selected-body-boundary-summary  Print the compact reference 2634167 selected-body boundary evidence summary"
        ));
        assert!(help.contains(
            "2634167-selected-body-boundary-summary  Alias for reference-snapshot-2634167-selected-body-boundary-summary"
        ));
        assert!(help.contains(
            "reference-snapshot-early-major-body-boundary-summary  Print the compact reference early major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-1800-major-body-boundary-summary  Print the compact reference 1800 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2500-major-body-boundary-summary  Print the compact reference 2500 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2453000-major-body-boundary-summary  Print the compact reference 2453000 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2451910-major-body-boundary-summary  Print the compact reference 2451910 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2451915-major-body-boundary-summary  Print the compact reference 2451915 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2451917-major-body-boundary-summary  Print the compact reference 2451917 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2451919-major-body-boundary-summary  Print the compact reference 2451919 major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-2451920-major-body-interior-summary  Print the compact reference 2451920 major-body interior evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-major-body-boundary-summary  Print the compact reference major-body boundary evidence summary"
        ));
        assert!(help.contains(
            "major-body-boundary-summary  Alias for reference-snapshot-major-body-boundary-summary"
        ));
        assert!(help.contains(
            "reference-snapshot-major-body-bridge-summary  Print the compact reference major-body bridge evidence summary"
        ));
        assert!(
            help.contains("bridge-summary  Alias for reference-snapshot-major-body-bridge-summary")
        );
        assert!(help.contains(
            "reference-snapshot-major-body-boundary-window-summary  Print the compact reference major-body boundary windows summary"
        ));
        assert!(help.contains(
            "major-body-boundary-window-summary  Alias for reference-snapshot-major-body-boundary-window-summary"
        ));
        assert!(help.contains(
            "reference-snapshot-mars-jupiter-boundary-summary  Print the compact reference Mars/Jupiter boundary evidence summary"
        ));
        assert!(help.contains(
            "mars-jupiter-boundary-summary  Alias for reference-snapshot-mars-jupiter-boundary-summary"
        ));
        assert!(help.contains(
            "reference-snapshot-mars-outer-boundary-summary  Print the compact reference Mars outer-boundary evidence summary"
        ));
        assert!(help.contains(
            "mars-outer-boundary-summary  Alias for reference-snapshot-mars-outer-boundary-summary"
        ));
        assert!(help.contains(
            "lunar-reference-error-envelope-summary  Print the compact lunar reference error envelope summary"
        ));
        assert!(help.contains(
            "lunar-equatorial-reference-error-envelope-summary  Print the compact lunar equatorial reference error envelope summary"
        ));
        assert!(help.contains(
            "lunar-apparent-comparison-summary  Print the compact lunar apparent comparison summary"
        ));
        assert!(help.contains(
            "lunar-source-window-summary  Print the compact lunar source windows summary"
        ));
        assert!(help.contains(
            "comparison-snapshot-manifest-summary  Print the compact comparison snapshot manifest summary"
        ));
        assert!(help.contains(
            "comparison-snapshot-manifest  Alias for comparison-snapshot-manifest-summary"
        ));
        assert!(help.contains(
            "independent-holdout-batch-parity-summary  Print the compact independent hold-out batch parity summary"
        ));
        assert!(help.contains(
            "independent-holdout-equatorial-parity-summary  Print the compact independent hold-out equatorial parity summary"
        ));
        assert!(help.contains(
            "comparison-snapshot-summary  Print the compact comparison snapshot summary"
        ));
        assert!(help.contains("comparison-snapshot         Alias for comparison-snapshot-summary"));
        assert!(help.contains(
            "comparison-snapshot-source-summary  Print the compact comparison snapshot source summary"
        ));
        assert!(help.contains(
            "comparison-snapshot-source-window  Alias for comparison-snapshot-source-window-summary"
        ));
        assert!(help.contains(
            "reference-snapshot-manifest-summary  Print the compact reference snapshot manifest summary"
        ));
        assert!(help.contains(
            "reference-snapshot-manifest  Alias for reference-snapshot-manifest-summary"
        ));
        assert!(help.contains("reference-snapshot         Alias for reference-snapshot-summary"));
        assert!(help.contains(
            "reference-snapshot-source-window  Alias for reference-snapshot-source-window-summary"
        ));
        assert!(help.contains(
            "reference-snapshot-source-summary  Print the compact reference snapshot source summary"
        ));
        assert!(help.contains(
            "boundary-day-summary     Alias for reference-snapshot-sparse-boundary-summary"
        ));
        assert!(help
            .contains("reference-snapshot-summary  Print the compact reference snapshot summary"));
        assert!(help.contains(
            "reference-snapshot-exact-j2000-evidence-summary  Print the compact reference snapshot exact J2000 evidence summary"
        ));
        assert!(help.contains(
            "exact-j2000-evidence    Alias for reference-snapshot-exact-j2000-evidence-summary"
        ));
        assert!(help.contains(
            "selected-asteroid-source-evidence-summary  Print the compact selected-asteroid source evidence summary"
        ));
        assert!(help.contains(
            "reference-snapshot-selected-asteroid-dense-boundary-summary  Print the compact selected-asteroid dense boundary evidence summary"
        ));
        assert!(help.contains(
            "selected-asteroid-dense-boundary-summary  Alias for reference-snapshot-selected-asteroid-dense-boundary-summary"
        ));
        assert!(help.contains(
            "selected-asteroid-batch-parity-summary  Print the compact selected-asteroid batch-parity summary"
        ));
        assert!(help.contains(
            "reference-asteroid-evidence-summary  Print the compact reference asteroid evidence summary"
        ));
        assert!(help.contains("reference-asteroid-equatorial-evidence-summary  Print the compact reference asteroid equatorial evidence summary"));
        assert!(help.contains("reference-asteroid-source-window-summary  Print the compact reference asteroid source windows summary"));
        assert!(help.contains(
            "reference-asteroid-source-summary  Alias for reference-asteroid-source-window-summary"
        ));
        assert!(help.contains("selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary"));
        assert!(help.contains(
            "selected-asteroid-source-window  Alias for selected-asteroid-source-window-summary"
        ));
        assert!(help.contains("independent-holdout-source-window-summary  Print the compact independent hold-out source windows summary"));
        assert!(help.contains(
            "independent-holdout-summary  Print the compact independent hold-out summary"
        ));
        assert!(help.contains("independent-holdout-source-summary  Print the compact independent hold-out source summary"));
        assert!(help.contains("independent-holdout-high-curvature-summary  Print the compact independent hold-out high-curvature evidence summary"));
        assert!(help.contains(
            "holdout-high-curvature-summary  Alias for independent-holdout-high-curvature-summary"
        ));
        assert!(help
            .contains("source-audit-summary      Print the compact VSOP87 source audit summary"));
        assert!(help.contains(
            "generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary"
        ));
        assert!(help.contains(
            "benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus and full chart assembly on representative house scenarios"
        ));
    }

    #[test]
    fn benchmark_command_renders_a_report() {
        let rendered =
            render_cli(&["benchmark", "--rounds", "1"]).expect("benchmark should render");
        assert!(rendered.contains("Benchmark report"));
        assert!(rendered.contains("Summary: backend="));
    }

    #[test]
    fn benchmark_command_rejects_duplicate_rounds_arguments() {
        let error = render_cli(&["benchmark", "--rounds", "1", "--rounds", "2"])
            .expect_err("benchmark should reject duplicate rounds arguments");
        assert!(error.contains("duplicate value for --rounds argument"));
    }

    #[test]
    fn benchmark_command_rejects_extra_arguments() {
        let error = render_cli(&["benchmark", "--rounds", "1", "extra"])
            .expect_err("benchmark should reject extra arguments");
        assert_eq!(error, "unknown argument: extra");
    }

    #[test]
    fn chart_command_renders_bodies() {
        let rendered = render_chart(&["--jd", "2451545.0", "--body", "Sun", "--body", "Moon"])
            .expect("chart should render");
        assert!(rendered.contains("Backend:"));
        assert!(rendered.contains("Sun"));
        assert!(rendered.contains("Moon"));
        assert!(rendered.contains("Apparentness: Mean"));
        assert!(rendered.contains("Sign summary:"));
    }

    #[test]
    fn chart_command_rejects_apparent_positions_until_supported() {
        let error = render_chart(&["--jd", "2451545.0", "--apparent", "--body", "Sun"])
            .expect_err("current first-party backends should reject apparent requests");
        assert!(error.contains("UnsupportedApparentness"));
        assert!(error.contains("mean-state") || error.contains("mean geometric"));
    }

    #[test]
    fn chart_command_renders_aspect_information() {
        let rendered = render_chart(&["--jd", "2451545.0", "--body", "Sun", "--body", "Moon"])
            .expect("chart should render");
        assert!(rendered.contains("Aspect summary: 1 Sextile"));
        assert!(rendered.contains("Aspects:"));
        assert!(rendered.contains("Sun Sextile Moon"));
    }

    #[test]
    fn chart_command_can_convert_utc_to_tt_with_caller_supplied_delta_t() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-offset-seconds",
            "64.184",
            "--body",
            "Sun",
        ])
        .expect("UTC chart should convert to TT with an explicit offset");
        assert!(rendered.contains("Instant: JD 2451545"));
        assert!(rendered.contains("(TT)"));
        assert!(rendered.contains("Sun"));
    }

    #[test]
    fn chart_command_can_convert_utc_to_tt_with_explicit_alias() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-from-utc-offset-seconds",
            "64.184",
            "--body",
            "Sun",
        ])
        .expect("UTC chart should convert to TT with the explicit UTC alias");
        assert!(rendered.contains("Instant: JD 2451545"));
        assert!(rendered.contains("(TT)"));
        assert!(rendered.contains("Sun"));
    }

    #[test]
    fn chart_command_can_convert_ut1_to_tt_with_explicit_alias() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--ut1",
            "--tt-from-ut1-offset-seconds",
            "64.184",
            "--body",
            "Sun",
        ])
        .expect("UT1 chart should convert to TT with the explicit UT1 alias");
        assert!(rendered.contains("Instant: JD 2451545"));
        assert!(rendered.contains("(TT)"));
        assert!(rendered.contains("Sun"));
    }

    #[test]
    fn chart_command_can_render_tdb_tagged_instant() {
        let rendered = render_chart(&["--jd", "2451545.0", "--tdb", "--body", "Sun"])
            .expect("chart should render with a TDB-tagged instant");
        assert!(
            rendered.contains("Instant: JD 2451545 (TDB)")
                || rendered.contains("Instant: JD 2451545.0 (TDB)")
        );
        assert!(rendered.contains("Apparentness: Mean"));
    }

    #[test]
    fn chart_command_can_convert_tdb_to_tt_with_signed_offset() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--tdb",
            "--tt-from-tdb-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect("TDB-tagged chart should accept a signed TT-TDB offset");
        assert!(rendered.contains("Instant: JD"));
        assert!(rendered.contains("(TT)"));
    }

    #[test]
    fn chart_command_can_convert_tt_to_tdb_with_explicit_tt_source_offset() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--tt",
            "--tdb-from-tt-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect("TT-tagged chart should accept an explicit TT-to-TDB offset flag");
        assert!(rendered.contains("Instant: JD"));
        assert!(rendered.contains("(TDB)"));
    }

    #[test]
    fn chart_command_can_convert_utc_to_tdb_with_explicit_alias() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-offset-seconds",
            "64.184",
            "--tdb-from-utc-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect("UTC-tagged chart should accept an explicit UTC-to-TDB alias");
        assert!(rendered.contains("Instant: JD"));
        assert!(rendered.contains("(TDB)"));
    }

    #[test]
    fn chart_command_can_convert_ut1_to_tdb_with_explicit_alias() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--ut1",
            "--tt-offset-seconds",
            "64.184",
            "--tdb-from-ut1-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect("UT1-tagged chart should accept an explicit UT1-to-TDB alias");
        assert!(rendered.contains("Instant: JD"));
        assert!(rendered.contains("(TDB)"));
    }

    #[test]
    fn chart_command_rejects_conflicting_tdb_offset_aliases() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--tt",
            "--tdb-offset-seconds",
            "-0.001657",
            "--tdb-from-tt-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect_err("TT-tagged chart requests should reject conflicting TDB-TT aliases");
        assert!(error.contains("conflicting TDB-TT offset flags"));
    }

    #[test]
    fn chart_command_rejects_conflicting_tdb_offset_aliases_in_either_order() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--tt",
            "--tdb-from-tt-offset-seconds",
            "-0.001657",
            "--tdb-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect_err(
            "TT-tagged chart requests should reject conflicting TDB-TT aliases regardless of order",
        );
        assert!(error.contains("conflicting TDB-TT offset flags"));
    }

    #[test]
    fn chart_command_rejects_repeated_tt_offset_flags() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-offset-seconds",
            "64.184",
            "--tt-offset-seconds",
            "65.0",
            "--body",
            "Sun",
        ])
        .expect_err("UTC-tagged chart requests should reject duplicate TT offset flags");
        assert!(error.contains("conflicting TT offset flags"));
    }

    #[test]
    fn chart_command_rejects_repeated_tt_offset_aliases() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-offset-seconds",
            "64.184",
            "--tt-from-utc-offset-seconds",
            "65.0",
            "--body",
            "Sun",
        ])
        .expect_err("UTC-tagged chart requests should reject duplicate TT offset aliases");
        assert!(error.contains("conflicting TT offset flags"));
    }

    #[test]
    fn chart_command_rejects_repeated_tdb_offset_flags() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--tt",
            "--tdb-offset-seconds",
            "-0.001657",
            "--tdb-offset-seconds",
            "-0.002",
            "--body",
            "Sun",
        ])
        .expect_err("TT-tagged chart requests should reject duplicate TDB-TT offset flags");
        assert!(error.contains("conflicting TDB-TT offset flags"));
    }

    #[test]
    fn chart_command_rejects_repeated_time_scale_flags_even_when_the_same_scale_is_reused() {
        let error = render_chart(&["--jd", "2451545.0", "--tt", "--tt", "--body", "Sun"])
            .expect_err("chart requests should reject duplicate time-scale tags");
        assert!(error.contains("conflicting time-scale flags"));
    }

    #[test]
    fn chart_command_rejects_repeated_apparentness_flags_even_when_the_same_mode_is_reused() {
        let error = render_chart(&["--jd", "2451545.0", "--mean", "--mean", "--body", "Sun"])
            .expect_err("chart requests should reject duplicate apparentness tags");
        assert!(error.contains("conflicting apparentness flags"));
    }

    #[test]
    fn chart_command_rejects_tdb_offsets_for_tdb_tagged_instants() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--tdb",
            "--tdb-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect_err("TDB-tagged chart requests should reject a caller-supplied TDB-TT offset");
        assert!(error.contains("--tdb-offset-seconds"));
    }

    #[test]
    fn chart_command_rejects_tdb_from_tt_offsets_for_utc_tagged_instants() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-offset-seconds",
            "64.184",
            "--tdb-from-tt-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect_err("UTC-tagged chart requests should reject the TT-only TDB offset alias");
        assert!(error.contains("--tdb-from-tt-offset-seconds"));
    }

    #[test]
    fn chart_command_can_convert_tt_to_tdb_with_signed_offset() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--tt",
            "--tdb-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect("TT-tagged chart should accept a signed TDB-TT offset");
        assert!(rendered.contains("Instant: JD"));
        assert!(rendered.contains("(TDB)"));
    }

    #[test]
    fn chart_command_can_convert_utc_to_tdb_with_signed_offset() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-offset-seconds",
            "64.184",
            "--tdb-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect("UTC chart should accept a signed TDB-TT offset");
        assert!(rendered.contains("Instant: JD 2451545"));
        assert!(rendered.contains("(TDB)"));
    }

    #[test]
    fn chart_command_rejects_tt_offsets_for_tt_tagged_instants() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--tt-offset-seconds",
            "64.184",
            "--body",
            "Sun",
        ])
        .expect_err("TT-tagged chart requests should reject a caller-supplied TT offset");
        assert!(error.contains("--tt-offset-seconds"));
    }

    #[test]
    fn chart_command_rejects_tt_offsets_for_tdb_tagged_instants() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--tdb",
            "--tt-offset-seconds",
            "64.184",
            "--body",
            "Sun",
        ])
        .expect_err("TDB-tagged chart requests should reject a caller-supplied TT offset");
        assert!(error.contains("--tt-offset-seconds"));
    }

    #[test]
    fn chart_command_rejects_tdb_retagging_offsets_for_utc_tagged_instants() {
        let error = render_chart(&[
            "--jd",
            "2451545.0",
            "--utc",
            "--tt-offset-seconds",
            "64.184",
            "--tt-from-tdb-offset-seconds",
            "-0.001657",
            "--body",
            "Sun",
        ])
        .expect_err("UTC-tagged chart requests should reject a TDB-to-TT retagging offset");
        assert!(error.contains("--tt-from-tdb-offset-seconds"));
    }

    #[test]
    fn chart_command_can_convert_ut1_to_tdb_with_caller_supplied_offsets() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--ut1",
            "--tt-offset-seconds",
            "64.184",
            "--tdb-offset-seconds",
            "0.001657",
            "--body",
            "Sun",
        ])
        .expect("UT1 chart should convert to TDB with explicit offsets");
        assert!(rendered.contains("Instant: JD 2451545"));
        assert!(rendered.contains("(TDB)"));
        assert!(rendered.contains("Sun"));
    }

    #[test]
    fn chart_command_accepts_sidereal_ayanamsa() {
        let rendered =
            render_chart(&["--jd", "2451545.0", "--ayanamsa", "Lahiri", "--body", "Sun"])
                .expect("sidereal chart should render");
        assert!(rendered.contains("Sidereal"));
        assert!(rendered.contains("Lahiri"));
    }

    #[test]
    fn chart_command_can_render_house_information() {
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--lat",
            "0.0",
            "--lon",
            "0.0",
            "--house-system",
            "Whole Sign",
            "--body",
            "Sun",
        ])
        .expect("house-aware chart should render");
        assert!(rendered.contains("House system:"));
        assert!(rendered.contains("House cusps:"));
        assert!(rendered.contains("Sun"));
        assert!(rendered.contains(" 1:"));
    }

    #[test]
    fn chart_command_accepts_custom_ayanamsa_definitions() {
        for (label, offset) in [
            ("True Balarama", 12.5),
            ("Aphoric", -3.25),
            ("Takra", 0.125),
        ] {
            let ayanamsa = format!("custom:{label}|2451545.0|{offset}");
            let rendered = render_chart(&[
                "--jd",
                "2451545.0",
                "--ayanamsa",
                &ayanamsa,
                "--body",
                "Sun",
            ])
            .expect("custom ayanamsa chart should render");

            assert!(rendered.contains("Sidereal"));
            assert!(rendered.contains(label));
            assert!(rendered.contains(&offset.to_string()));
            assert!(rendered.contains("Custom ayanamsa definition supplied via the CLI"));
        }
    }

    #[test]
    fn parse_ayanamsa_accepts_custom_definition_labels() {
        for (label, offset) in [
            ("True Balarama", 12.5),
            ("Aphoric", -3.25),
            ("Takra", 0.125),
        ] {
            let definition = format!("custom-definition:{label}|2451545.0|{offset}");
            let custom = parse_ayanamsa(&definition).expect("custom ayanamsa should parse");

            assert_eq!(
                custom,
                Ayanamsa::Custom(CustomAyanamsa {
                    name: label.to_owned(),
                    description: Some("Custom ayanamsa definition supplied via the CLI".to_owned()),
                    epoch: Some(JulianDay::from_days(2_451_545.0)),
                    offset_degrees: Some(Angle::from_degrees(offset)),
                })
            );
        }
    }

    #[test]
    fn parse_ayanamsa_rejects_padded_custom_definition_names() {
        let error = parse_ayanamsa("custom: True Balarama|2451545.0|12.5")
            .expect_err("padding should fail");
        assert_eq!(
            error,
            "custom ayanamsa name must not have leading or trailing whitespace"
        );
    }

    #[test]
    fn chart_command_routes_selected_asteroids_via_jpl_fallback() {
        let rendered = render_chart(&["--jd", "2451545.0", "--body", "Ceres"])
            .expect("asteroid chart should render");
        assert!(rendered.contains("Ceres"));
        assert!(rendered.contains("Backend:"));
    }

    #[test]
    fn parse_body_accepts_lunar_apogee_and_perigee_labels() {
        assert_eq!(
            parse_body(Some("mean apogee")).unwrap(),
            CelestialBody::MeanApogee
        );
        assert_eq!(
            parse_body(Some("true apogee")).unwrap(),
            CelestialBody::TrueApogee
        );
        assert_eq!(
            parse_body(Some("mean perigee")).unwrap(),
            CelestialBody::MeanPerigee
        );
        assert_eq!(
            parse_body(Some("true perigee")).unwrap(),
            CelestialBody::TruePerigee
        );
    }

    #[test]
    fn parse_body_accepts_custom_catalog_designations() {
        let body = parse_body(Some("asteroid:433-Eros")).expect("custom body should parse");
        assert_eq!(
            body,
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
        assert_eq!(body.to_string(), "asteroid:433-Eros");
    }

    #[test]
    fn parse_body_rejects_padded_custom_catalog_designations() {
        let error = parse_body(Some("asteroid: 433-Eros")).expect_err("padding should fail");
        assert_eq!(
            error,
            "custom body id designation must not have leading or trailing whitespace"
        );
    }

    #[test]
    fn parse_body_accepts_lunar_nodes() {
        assert_eq!(
            parse_body(Some("mean node")).unwrap(),
            CelestialBody::MeanNode
        );
        assert_eq!(
            parse_body(Some("mean lunar node")).unwrap(),
            CelestialBody::MeanNode
        );
        assert_eq!(
            parse_body(Some("true node")).unwrap(),
            CelestialBody::TrueNode
        );
        assert_eq!(
            parse_body(Some("true lunar node")).unwrap(),
            CelestialBody::TrueNode
        );
    }
}
