//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! The CLI now exposes the compatibility profile and a small chart report
//! command so contributors can exercise the first end-to-end workflow without
//! leaving the repository. The chart report keeps the mean/apparent position
//! choice explicit so report consumers can see which backend mode was used.

#![forbid(unsafe_code)]

use core::time::Duration;

use pleiades_core::{
    current_api_stability_profile, default_chart_bodies, resolve_ayanamsa, resolve_house_system,
    Angle, Apparentness, Ayanamsa, CelestialBody, ChartEngine, ChartRequest, CompositeBackend,
    CustomAyanamsa, CustomBodyId, EphemerisError, HouseSystem, Instant, JulianDay, Latitude,
    Longitude, ObserverLocation, RoutingBackend, TimeScale, ZodiacMode,
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
    render_api_stability_summary, render_artifact_summary, render_backend_matrix_report,
    render_backend_matrix_summary, render_benchmark_report, render_cli as validate_render_cli,
    render_compatibility_profile_summary, render_release_bundle, render_release_checklist,
    render_release_checklist_summary, render_release_notes, render_release_notes_summary,
    render_release_summary, render_request_surface_summary, render_validation_report_summary,
    verify_compatibility_profile,
};
use pleiades_vsop87::Vsop87Backend;

fn banner() -> &'static str {
    "pleiades-cli chart utility"
}

fn shared_request_policy_help_block() -> String {
    let request_policy = pleiades_core::request_policy_summary_for_report();
    let time_scale_policy = pleiades_core::time_scale_policy_summary_for_report();
    let delta_t_policy = pleiades_core::delta_t_policy_summary_for_report();
    let observer_policy = pleiades_core::observer_policy_summary_for_report();
    let apparentness_policy = pleiades_core::apparentness_policy_summary_for_report();
    let frame_policy = pleiades_core::frame_policy_summary_for_report();

    format!(
        "  Request policy: {}\n  Time-scale policy: {}\n  Delta T policy: {}\n  Observer policy: {}\n  Apparentness policy: {}\n  Frame policy: {}",
        request_policy.summary_line(),
        time_scale_policy.summary_line(),
        delta_t_policy.summary_line(),
        observer_policy.summary_line(),
        apparentness_policy.summary_line(),
        frame_policy,
    )
}

fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compare-backends") => validate_render_cli(&["compare-backends"]),
        Some("compare-backends-audit") => validate_render_cli(&["compare-backends-audit"]),
        Some("benchmark") => {
            let rounds = parse_rounds(&args[1..], 10_000)?;
            render_benchmark_report(rounds).map_err(render_error)
        }
        Some("comparison-corpus-summary") => validate_render_cli(&["comparison-corpus-summary"]),
        Some("benchmark-corpus-summary") => validate_render_cli(&["benchmark-corpus-summary"]),
        Some("compatibility-profile") | Some("profile") => {
            Ok(pleiades_core::current_compatibility_profile().to_string())
        }
        Some("compatibility-profile-summary") | Some("profile-summary") => {
            Ok(render_compatibility_profile_summary())
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
        Some("api-stability") | Some("api-posture") => {
            Ok(current_api_stability_profile().to_string())
        }
        Some("api-stability-summary") | Some("api-posture-summary") => {
            Ok(render_api_stability_summary())
        }
        Some("backend-matrix") | Some("capability-matrix") => {
            render_backend_matrix_report().map_err(render_error)
        }
        Some("backend-matrix-summary") | Some("matrix-summary") => {
            Ok(render_backend_matrix_summary())
        }
        Some("release-notes") => Ok(render_release_notes()),
        Some("release-notes-summary") => Ok(render_release_notes_summary()),
        Some("release-checklist") => Ok(render_release_checklist()),
        Some("release-checklist-summary") | Some("checklist-summary") => {
            Ok(render_release_checklist_summary())
        }
        Some("release-summary") => Ok(render_release_summary()),
        Some("jpl-batch-error-taxonomy-summary") => {
            validate_render_cli(&["jpl-batch-error-taxonomy-summary"])
        }
        Some("production-generation-boundary-summary") => {
            validate_render_cli(&["production-generation-boundary-summary"])
        }
        Some("production-generation-boundary-request-corpus-summary") => {
            validate_render_cli(&["production-generation-boundary-request-corpus-summary"])
        }
        Some("production-generation-body-class-coverage-summary") => {
            validate_render_cli(&["production-generation-body-class-coverage-summary"])
        }
        Some("production-generation-source-window-summary") => {
            validate_render_cli(&["production-generation-source-window-summary"])
        }
        Some("comparison-snapshot-source-summary") => {
            Ok(comparison_snapshot_source_summary_for_report())
        }
        Some("comparison-snapshot-source-window-summary") => {
            validate_render_cli(&["comparison-snapshot-source-window-summary"])
        }
        Some("comparison-snapshot-body-class-coverage-summary") => {
            validate_render_cli(&["comparison-snapshot-body-class-coverage-summary"])
        }
        Some("comparison-snapshot-manifest-summary") => {
            validate_render_cli(&["comparison-snapshot-manifest-summary"])
        }
        Some("comparison-snapshot-summary") => {
            validate_render_cli(&["comparison-snapshot-summary"])
        }
        Some("comparison-snapshot-batch-parity-summary") => {
            validate_render_cli(&["comparison-snapshot-batch-parity-summary"])
        }
        Some("reference-snapshot-source-summary") => {
            Ok(reference_snapshot_source_summary_for_report())
        }
        Some("reference-snapshot-source-window-summary") => {
            validate_render_cli(&["reference-snapshot-source-window-summary"])
        }
        Some("reference-snapshot-lunar-boundary-summary") => {
            validate_render_cli(&["reference-snapshot-lunar-boundary-summary"])
        }
        Some("reference-snapshot-body-class-coverage-summary") => {
            validate_render_cli(&["reference-snapshot-body-class-coverage-summary"])
        }
        Some("reference-snapshot-manifest-summary") => {
            validate_render_cli(&["reference-snapshot-manifest-summary"])
        }
        Some("reference-snapshot-summary") => validate_render_cli(&["reference-snapshot-summary"]),
        Some("reference-snapshot-batch-parity-summary") => {
            validate_render_cli(&["reference-snapshot-batch-parity-summary"])
        }
        Some("reference-snapshot-equatorial-parity-summary") => {
            validate_render_cli(&["reference-snapshot-equatorial-parity-summary"])
        }
        Some("reference-high-curvature-summary") => {
            validate_render_cli(&["reference-high-curvature-summary"])
        }
        Some("reference-high-curvature-window-summary") => {
            validate_render_cli(&["reference-high-curvature-window-summary"])
        }
        Some("source-documentation-summary") => {
            validate_render_cli(&["source-documentation-summary"])
        }
        Some("source-documentation-health-summary") => {
            validate_render_cli(&["source-documentation-health-summary"])
        }
        Some("time-scale-policy-summary") => validate_render_cli(&["time-scale-policy-summary"]),
        Some("delta-t-policy-summary") => validate_render_cli(&["delta-t-policy-summary"]),
        Some("observer-policy-summary") => validate_render_cli(&["observer-policy-summary"]),
        Some("apparentness-policy-summary") => {
            validate_render_cli(&["apparentness-policy-summary"])
        }
        Some("interpolation-posture-summary") => {
            validate_render_cli(&["interpolation-posture-summary"])
        }
        Some("interpolation-quality-summary") => {
            validate_render_cli(&["interpolation-quality-summary"])
        }
        Some("lunar-reference-error-envelope-summary") => {
            validate_render_cli(&["lunar-reference-error-envelope-summary"])
        }
        Some("lunar-equatorial-reference-error-envelope-summary") => {
            validate_render_cli(&["lunar-equatorial-reference-error-envelope-summary"])
        }
        Some("lunar-apparent-comparison-summary") => {
            validate_render_cli(&["lunar-apparent-comparison-summary"])
        }
        Some("lunar-source-window-summary") => {
            validate_render_cli(&["lunar-source-window-summary"])
        }
        Some("lunar-theory-summary") => validate_render_cli(&["lunar-theory-summary"]),
        Some("lunar-theory-capability-summary") => {
            validate_render_cli(&["lunar-theory-capability-summary"])
        }
        Some("lunar-theory-source-summary") => {
            validate_render_cli(&["lunar-theory-source-summary"])
        }
        Some("selected-asteroid-boundary-summary") => {
            validate_render_cli(&["selected-asteroid-boundary-summary"])
        }
        Some("selected-asteroid-source-evidence-summary") => {
            validate_render_cli(&["selected-asteroid-source-evidence-summary"])
        }
        Some("selected-asteroid-source-window-summary") => {
            validate_render_cli(&["selected-asteroid-source-window-summary"])
        }
        Some("selected-asteroid-batch-parity-summary") => {
            validate_render_cli(&["selected-asteroid-batch-parity-summary"])
        }
        Some("reference-asteroid-evidence-summary") => {
            validate_render_cli(&["reference-asteroid-evidence-summary"])
        }
        Some("reference-asteroid-equatorial-evidence-summary") => {
            validate_render_cli(&["reference-asteroid-equatorial-evidence-summary"])
        }
        Some("reference-asteroid-source-window-summary") => {
            validate_render_cli(&["reference-asteroid-source-window-summary"])
        }
        Some("reference-holdout-overlap-summary") => {
            validate_render_cli(&["reference-holdout-overlap-summary"])
        }
        Some("independent-holdout-source-window-summary") => {
            validate_render_cli(&["independent-holdout-source-window-summary"])
        }
        Some("independent-holdout-body-class-coverage-summary") => {
            validate_render_cli(&["independent-holdout-body-class-coverage-summary"])
        }
        Some("independent-holdout-batch-parity-summary") => {
            validate_render_cli(&["independent-holdout-batch-parity-summary"])
        }
        Some("independent-holdout-equatorial-parity-summary") => {
            validate_render_cli(&["independent-holdout-equatorial-parity-summary"])
        }
        Some("house-validation-summary") => validate_render_cli(&["house-validation-summary"]),
        Some("ayanamsa-catalog-validation-summary") => {
            validate_render_cli(&["ayanamsa-catalog-validation-summary"])
        }
        Some("ayanamsa-metadata-coverage-summary") => {
            validate_render_cli(&["ayanamsa-metadata-coverage-summary"])
        }
        Some("ayanamsa-reference-offsets-summary") => {
            validate_render_cli(&["ayanamsa-reference-offsets-summary"])
        }
        Some("frame-policy-summary") => validate_render_cli(&["frame-policy-summary"]),
        Some("release-profile-identifiers-summary") => {
            validate_render_cli(&["release-profile-identifiers-summary"])
        }
        Some("request-surface-summary") => Ok(render_request_surface_summary()),
        Some("request-policy-summary") | Some("request-semantics-summary") => {
            validate_render_cli(args)
        }
        Some("comparison-tolerance-policy-summary") => {
            validate_render_cli(&["comparison-tolerance-policy-summary"])
        }
        Some("pluto-fallback-summary") => validate_render_cli(&["pluto-fallback-summary"]),
        Some("workspace-audit-summary") | Some("native-dependency-audit-summary") => {
            validate_render_cli(&["workspace-audit-summary"])
        }
        Some("artifact-summary") | Some("artifact-posture-summary") => {
            render_artifact_summary().map_err(|error| error.to_string())
        }
        Some("artifact-profile-coverage-summary") => {
            validate_render_cli(&["artifact-profile-coverage-summary"])
        }
        Some("packaged-artifact-output-support-summary") => {
            validate_render_cli(&["packaged-artifact-output-support-summary"])
        }
        Some("packaged-artifact-production-profile-summary") => {
            validate_render_cli(&["packaged-artifact-production-profile-summary"])
        }
        Some("packaged-artifact-generation-manifest-summary") => {
            validate_render_cli(&["packaged-artifact-generation-manifest-summary"])
        }
        Some("packaged-artifact-generation-policy-summary") => {
            validate_render_cli(&["packaged-artifact-generation-policy-summary"])
        }
        Some("packaged-artifact-generation-residual-bodies-summary") => {
            validate_render_cli(&["packaged-artifact-generation-residual-bodies-summary"])
        }
        Some("packaged-lookup-epoch-policy-summary") => {
            validate_render_cli(&["packaged-lookup-epoch-policy-summary"])
        }
        Some("validate-artifact") => validate_render_cli(&["validate-artifact"]),
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
            validate_render_cli(&["workspace-audit"])
        }
        Some("report") | Some("generate-report") => validate_render_cli(args),
        Some("validation-report-summary") | Some("validation-summary") | Some("report-summary") => {
            render_validation_report_summary(1).map_err(render_error)
        }
        Some("chart") => render_chart(&args[1..]),
        Some("help") | Some("--help") | Some("-h") => Ok(help_text()),
        None => Ok(banner().to_string()),
        Some(other) => Err(format!("unknown command: {other}\n\n{}", help_text())),
    }
}

fn help_text() -> String {
    format!(
        "{}\n\nCommands:\n  compatibility-profile  Print the release compatibility profile\n  profile                Alias for compatibility-profile\n  compatibility-profile-summary  Print the compact compatibility profile summary\n  profile-summary        Alias for compatibility-profile-summary\n  benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus and full chart assembly on representative house scenarios\n  comparison-corpus-summary  Print the compact release-grade comparison corpus summary\n  benchmark-corpus-summary  Print the compact representative benchmark corpus summary\n  verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs\n  bundle-release         Write the staged release bundle, artifact summary, packaged-artifact generation manifest, benchmark report, and manifest files\n  verify-release-bundle  Read a staged release bundle back and verify its manifest checksums\n  api-stability          Print the release API stability posture\n  api-posture            Alias for api-stability\n  api-stability-summary  Print the compact API stability summary\n  api-posture-summary    Alias for api-stability-summary\n  compare-backends       Compare the JPL snapshot against the algorithmic composite backend\n  compare-backends-audit Compare the JPL snapshot against the algorithmic composite backend and fail if the tolerance audit reports regressions\n  backend-matrix         Print the implemented backend capability matrices\n  capability-matrix      Alias for backend-matrix\n  backend-matrix-summary Print the compact backend capability matrix summary\n  matrix-summary         Alias for backend-matrix-summary\n  release-notes          Print the release compatibility notes\n  release-notes-summary   Print the compact release notes summary\n  release-checklist      Print the release maintainer checklist\n  release-checklist-summary Print the compact release checklist summary\n  checklist-summary      Alias for release-checklist-summary\n  release-summary        Print the compact release summary\n  jpl-batch-error-taxonomy-summary  Print the compact JPL batch error taxonomy summary\n  production-generation-boundary-summary  Print the compact production-generation boundary overlay summary\n  production-generation-boundary-request-corpus-summary  Print the compact production-generation boundary request corpus summary\n  production-generation-body-class-coverage-summary  Print the compact production-generation body-class coverage summary\n  production-generation-source-window-summary  Print the compact production-generation source windows summary\n  comparison-snapshot-source-window-summary  Print the compact comparison snapshot source windows summary\n  comparison-snapshot-source-summary  Print the compact comparison snapshot source summary\n  comparison-snapshot-body-class-coverage-summary  Print the compact comparison snapshot body-class coverage summary\n  comparison-snapshot-manifest-summary  Print the compact comparison snapshot manifest summary\n  comparison-snapshot-summary  Print the compact comparison snapshot summary\n  comparison-snapshot-batch-parity-summary  Print the compact comparison snapshot batch parity summary\n  reference-snapshot-source-window-summary  Print the compact reference snapshot source windows summary\n  reference-snapshot-source-summary  Print the compact reference snapshot source summary\n  reference-snapshot-lunar-boundary-summary  Print the compact reference lunar boundary evidence summary\n  reference-snapshot-body-class-coverage-summary  Print the compact reference snapshot body-class coverage summary\n  reference-snapshot-manifest-summary  Print the compact reference snapshot manifest summary\n  reference-snapshot-summary  Print the compact reference snapshot summary\n  reference-snapshot-batch-parity-summary  Print the compact reference snapshot batch parity summary\n  reference-snapshot-equatorial-parity-summary  Print the compact reference snapshot equatorial parity summary\n  reference-high-curvature-summary  Print the compact reference major-body high-curvature evidence summary\n  reference-high-curvature-window-summary  Print the compact reference major-body high-curvature windows summary\n  source-documentation-summary  Print the compact VSOP87 source-documentation summary\n  source-documentation-health-summary  Print the compact VSOP87 source-documentation health summary\n  time-scale-policy-summary  Print the compact time-scale policy summary\n  delta-t-policy-summary   Print the compact Delta T policy summary\n  observer-policy-summary  Print the compact observer policy summary\n  apparentness-policy-summary  Print the compact apparentness policy summary\n  interpolation-posture-summary  Print the compact JPL interpolation posture summary\n  interpolation-quality-summary  Print the compact JPL interpolation quality summary\n  lunar-reference-error-envelope-summary  Print the compact lunar reference error envelope summary\n  lunar-equatorial-reference-error-envelope-summary  Print the compact lunar equatorial reference error envelope summary\n  lunar-apparent-comparison-summary  Print the compact lunar apparent comparison summary\n  lunar-source-window-summary  Print the compact lunar source windows summary\n  lunar-theory-summary      Print the compact ELP lunar theory specification\n  lunar-theory-capability-summary  Print the compact ELP lunar capability summary\n  lunar-theory-source-summary  Print the compact ELP lunar source summary\n  selected-asteroid-boundary-summary  Print the compact selected-asteroid boundary evidence summary\n  selected-asteroid-source-evidence-summary  Print the compact selected-asteroid source evidence summary\n  selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary\n  selected-asteroid-batch-parity-summary  Print the compact selected-asteroid batch-parity summary\n  reference-asteroid-evidence-summary  Print the compact reference asteroid evidence summary\n  reference-asteroid-equatorial-evidence-summary  Print the compact reference asteroid equatorial evidence summary\n  reference-asteroid-source-window-summary  Print the compact reference asteroid source windows summary\n  reference-holdout-overlap-summary  Print the compact reference/hold-out overlap summary\n  independent-holdout-source-window-summary  Print the compact independent hold-out source windows summary\n  independent-holdout-body-class-coverage-summary  Print the compact independent hold-out body-class coverage summary
  independent-holdout-batch-parity-summary  Print the compact independent hold-out batch parity summary
  independent-holdout-equatorial-parity-summary  Print the compact independent hold-out equatorial parity summary\n  house-validation-summary   Print the compact house-validation corpus summary\n  ayanamsa-catalog-validation-summary  Print the compact ayanamsa catalog validation summary\n  ayanamsa-metadata-coverage-summary  Print the compact ayanamsa sidereal metadata coverage summary\n  ayanamsa-reference-offsets-summary  Print the compact ayanamsa reference offsets summary\n  frame-policy-summary   Print the compact frame-policy summary\n  release-profile-identifiers-summary  Print the compact release-profile identifiers summary\n  request-surface-summary  Print the compact request-surface inventory summary\n  request-policy-summary  Print the compact request-policy summary\n  request-semantics-summary Alias for request-policy-summary\n  comparison-tolerance-policy-summary  Print the compact comparison tolerance policy summary\n  pluto-fallback-summary   Print the compact Pluto fallback summary\n  workspace-audit-summary  Print the compact workspace audit summary\n  native-dependency-audit-summary  Alias for workspace-audit-summary\n  artifact-summary       Print the compact packaged-artifact summary\n  artifact-posture-summary  Alias for artifact-summary\n  artifact-profile-coverage-summary  Print the packaged-artifact profile coverage summary\n  packaged-artifact-output-support-summary  Print the packaged-artifact output support summary\n  packaged-artifact-production-profile-summary  Print the packaged-artifact production profile skeleton summary\n  packaged-artifact-generation-manifest-summary  Print the packaged-artifact generation manifest summary\n  packaged-artifact-generation-policy-summary  Print the packaged-artifact generation policy summary\n  packaged-artifact-generation-residual-bodies-summary  Print the packaged-artifact generation residual bodies summary\n  packaged-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary\n  validate-artifact      Inspect and validate the bundled compressed artifact\n  regenerate-packaged-artifact  Rebuild or verify the packaged artifact fixture from the checked-in reference snapshot; pass a file path, --out FILE, or --check\n  workspace-audit        Check the workspace for mandatory native build hooks\n  audit                  Alias for workspace-audit\n  native-dependency-audit  Alias for workspace-audit\n  report                 Print the full validation report\n  generate-report        Alias for report\n  validation-report-summary  Print the compact validation report summary\n  validation-summary     Alias for validation-report-summary\n  report-summary         Alias for validation-report-summary\n  chart                  Render a basic chart report\n    --tt|--tdb|--utc|--ut1  Tag the chart instant with a time scale\n    --tt-offset-seconds <seconds>  Caller-supplied TT offset for UTC/UT1-tagged instants\n    --tt-from-utc-offset-seconds <seconds>  Alias for --tt-offset-seconds when the chart instant is tagged as UTC\n    --tt-from-ut1-offset-seconds <seconds>  Alias for --tt-offset-seconds when the chart instant is tagged as UT1\n    --tdb-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT/UTC/UT1-tagged instants\n    --tdb-from-utc-offset-seconds <seconds> Explicit UTC-tagged alias for the signed TDB-TT offset\n    --tdb-from-ut1-offset-seconds <seconds> Explicit UT1-tagged alias for the signed TDB-TT offset\n    --tdb-from-tt-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT-tagged instants\n    --tt-from-tdb-offset-seconds <seconds> Caller-supplied signed TT-TDB offset for TDB-tagged instants\n    --mean               Force mean positions for backend queries\n    --apparent           Force apparent positions for backend queries\n    --body <name>        Use a built-in body or a custom catalog:designation identifier\n  {}\n  help                   Show this help text",
        banner(),
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
                return Ok(format!(
                    "{}\n\nUsage:\n  chart [--jd <julian-day>] [--lat <deg> --lon <deg>] [--tt|--tdb|--utc|--ut1] [--tt-offset-seconds <seconds>|--tt-from-utc-offset-seconds <seconds>|--tt-from-ut1-offset-seconds <seconds>] [--tdb-offset-seconds <seconds>|--tdb-from-utc-offset-seconds <seconds>|--tdb-from-ut1-offset-seconds <seconds>] [--tdb-from-tt-offset-seconds <seconds>] [--tt-from-tdb-offset-seconds <seconds>] [--mean|--apparent] [--ayanamsa <name>] [--house-system <name>] [--body <name> ...]\n\nAyanamsa names may be built-in entries or custom definitions in the form custom:<name>|<epoch-jd>|<offset-degrees> (or custom-definition:<name>|<epoch-jd>|<offset-degrees>). Body names may be built-in bodies such as Sun or Moon, or custom identifiers in the form catalog:designation. When the chart instant is tagged as UTC or UT1, the caller must also supply the explicit TT offset before chart assembly, either via --tt-offset-seconds or the more explicit --tt-from-utc-offset-seconds / --tt-from-ut1-offset-seconds aliases, and may also supply a signed TDB-TT offset when converting to TDB, either via --tdb-offset-seconds or the more explicit UTC/UT1 aliases. When the chart instant is tagged as TT, the caller may supply that signed TDB-TT offset via --tdb-offset-seconds or the more explicit --tdb-from-tt-offset-seconds alias. When the chart instant is tagged as TDB, the caller may supply a signed TT-TDB offset to re-tag the request as TT before assembly.\n\n{}\n",
                    banner(),
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
    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--rounds" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value for --rounds".to_string())?;
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
            "--out" => {
                output_dir = Some(
                    iter.next()
                        .ok_or_else(|| "missing value for --out".to_string())?,
                );
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
            "missing required output path argument; pass a file path, --out <file>, or --check"
                .to_string(),
        ),
        ["--check"] => Ok(PackagedArtifactCommand::Check),
        ["--out"] => Err("missing value for --out".to_string()),
        ["--out", path] => Ok(PackagedArtifactCommand::Write {
            output_path: (*path).to_string(),
        }),
        ["--out", _, extra, ..] => Err(format!("unknown argument: {extra}")),
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
        render_request_surface_summary, shared_request_policy_help_block, validate_render_cli,
        Angle, Ayanamsa, CelestialBody, CustomAyanamsa, CustomBodyId, JulianDay,
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
        assert!(rendered.contains("reference-high-curvature-summary"));
        assert!(rendered.contains("reference-high-curvature-window-summary"));
        assert!(rendered.contains("source-documentation-summary"));
        assert!(rendered.contains("source-documentation-health-summary"));
        assert!(rendered.contains("time-scale-policy-summary"));
        assert!(rendered.contains("production-generation-body-class-coverage-summary"));
        assert!(rendered.contains("production-generation-boundary-request-corpus-summary"));
        assert!(rendered.contains("comparison-snapshot-body-class-coverage-summary"));
        assert!(rendered.contains("comparison-corpus-summary"));
        assert!(rendered.contains("benchmark-corpus-summary"));
        assert!(rendered.contains("comparison-snapshot-summary"));
        assert!(rendered.contains("comparison-snapshot-batch-parity-summary"));
        assert!(rendered.contains("reference-snapshot-body-class-coverage-summary"));
        assert!(rendered.contains("reference-snapshot-summary"));
        assert!(rendered.contains("reference-snapshot-batch-parity-summary"));
        assert!(rendered.contains("reference-snapshot-equatorial-parity-summary"));
        assert!(rendered.contains("workspace-audit-summary"));
        assert!(rendered.contains("native-dependency-audit-summary"));
        assert!(rendered.contains("independent-holdout-source-window-summary"));
        assert!(rendered.contains("independent-holdout-body-class-coverage-summary"));
        assert!(rendered.contains("independent-holdout-batch-parity-summary"));
        assert!(rendered.contains("independent-holdout-equatorial-parity-summary"));
        assert!(rendered.contains("lunar-theory-summary"));
        assert!(rendered.contains("lunar-theory-capability-summary"));
        assert!(rendered.contains("lunar-theory-source-summary"));
        assert!(rendered.contains("observer-policy-summary"));
        assert!(rendered.contains("apparentness-policy-summary"));
        assert!(rendered.contains("compare-backends-audit"));
        assert!(rendered.contains("Caller-supplied signed TDB-TT offset for TT-tagged instants"));
        assert!(rendered.contains("Caller-supplied signed TT-TDB offset for TDB-tagged instants"));
    }

    #[test]
    fn compare_backends_command_renders_the_comparison_report() {
        let rendered = render_cli(&["compare-backends"]).expect("compare-backends should render");
        assert!(rendered.contains("Comparison report"));
        assert!(rendered.contains("Comparison corpus"));
        assert!(rendered.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"));
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
        assert!(verification.contains("Ayanamsa reference metadata verified: "));
        assert!(verification.contains(
            "Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"
        ));

        let backend_matrix =
            render_cli(&["backend-matrix-summary"]).expect("backend matrix summary should render");
        assert!(backend_matrix.contains("Backend matrix summary"));
        assert!(backend_matrix.contains("Backends: 5"));
        assert!(backend_matrix.contains("Accuracy classes: Exact: 1"));
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
        assert!(comparison_corpus.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"));
        assert_eq!(
            comparison_corpus,
            validate_render_cli(&["comparison-corpus-summary"])
                .expect("comparison corpus summary should match validation output")
        );

        let benchmark_corpus = render_cli(&["benchmark-corpus-summary"])
            .expect("benchmark corpus summary should render");
        assert!(benchmark_corpus.contains("Benchmark corpus summary"));
        assert!(benchmark_corpus.contains("name: Representative 1500-2500 window"));
        assert_eq!(
            benchmark_corpus,
            validate_render_cli(&["benchmark-corpus-summary"])
                .expect("benchmark corpus summary should match validation output")
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
        assert!(release_notes_summary.contains(profile.target_house_scope.join("; ").as_str()));
        assert!(release_notes_summary.contains(profile.target_ayanamsa_scope.join("; ").as_str()));
        assert!(release_notes_summary.contains(&format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )));
        assert!(release_notes_summary.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit TT/TDB/UTC/UT1 flags)"
        }));
        assert!(release_notes_summary.lines().any(|line| {
            line == "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
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
        assert!(release_summary.contains("Compatibility catalog inventory: house systems=25 (12 baseline, 13 release-specific, 156 aliases); house-code aliases=22; ayanamsas=59 (5 baseline, 54 release-specific, 183 aliases); custom-definition labels=9; known gaps=2"));
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
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests; topocentric body positions remain unsupported"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));

        let request_surface_summary = render_cli(&["request-surface-summary"])
            .expect("request surface summary should render");
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
            request_surface_summary.contains("pleiades-cli chart (explicit TT/TDB/UTC/UT1 flags)")
        );
        assert_eq!(request_surface_summary, render_request_surface_summary());

        let request_policy_summary =
            render_cli(&["request-policy-summary"]).expect("request policy summary should render");
        assert!(request_policy_summary.contains("Request policy summary"));
        assert!(request_policy_summary.contains("Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"));
        assert!(request_policy_summary.contains("Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"));
        assert!(request_policy_summary.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests; topocentric body positions remain unsupported"));
        assert!(request_policy_summary.contains("Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"));
        assert!(request_policy_summary.contains("Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
        assert_eq!(
            request_policy_summary,
            render_cli(&["request-semantics-summary"])
                .expect("request semantics summary should render")
        );

        let comparison_tolerance_policy_summary =
            render_cli(&["comparison-tolerance-policy-summary"])
                .expect("comparison tolerance policy summary should render");
        assert_eq!(
            comparison_tolerance_policy_summary,
            super::validate_render_cli(&["comparison-tolerance-policy-summary"])
                .expect("comparison tolerance policy summary should match validate CLI")
        );

        let pluto_fallback_summary =
            render_cli(&["pluto-fallback-summary"]).expect("Pluto fallback summary should render");
        assert_eq!(
            pluto_fallback_summary,
            super::validate_render_cli(&["pluto-fallback-summary"])
                .expect("Pluto fallback summary should match validate CLI")
        );

        let jpl_batch_error_taxonomy_summary = render_cli(&["jpl-batch-error-taxonomy-summary"])
            .expect("JPL batch error taxonomy summary should render");
        assert_eq!(
            jpl_batch_error_taxonomy_summary,
            super::validate_render_cli(&["jpl-batch-error-taxonomy-summary"])
                .expect("validation JPL batch error taxonomy summary should render")
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

        let production_generation_boundary_summary =
            render_cli(&["production-generation-boundary-summary"])
                .expect("production generation boundary summary should render");
        assert!(production_generation_boundary_summary
            .contains("Production generation boundary overlay:"));
        assert_eq!(
            production_generation_boundary_summary,
            pleiades_jpl::production_generation_boundary_summary_for_report()
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
        assert_eq!(
            production_generation_body_class_coverage_summary,
            super::validate_render_cli(&["production-generation-body-class-coverage-summary"])
                .expect(
                    "validation production generation body-class coverage summary should render"
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
        let comparison_snapshot_source_summary =
            render_cli(&["comparison-snapshot-source-summary"])
                .expect("comparison snapshot source summary should render");
        assert!(comparison_snapshot_source_summary.contains("Comparison snapshot source:"));
        assert_eq!(
            comparison_snapshot_source_summary,
            pleiades_jpl::comparison_snapshot_source_summary_for_report()
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
        let comparison_snapshot_manifest_summary =
            render_cli(&["comparison-snapshot-manifest-summary"])
                .expect("comparison snapshot manifest summary should render");
        assert!(comparison_snapshot_manifest_summary.contains("Comparison snapshot manifest:"));
        assert_eq!(
            comparison_snapshot_manifest_summary,
            pleiades_jpl::comparison_snapshot_manifest_summary_for_report()
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
        let reference_snapshot_source_summary = render_cli(&["reference-snapshot-source-summary"])
            .expect("reference snapshot source summary should render");
        assert!(reference_snapshot_source_summary.contains("Reference snapshot source:"));
        assert_eq!(
            reference_snapshot_source_summary,
            pleiades_jpl::reference_snapshot_source_summary_for_report()
        );
        let reference_snapshot_summary = render_cli(&["reference-snapshot-summary"])
            .expect("reference snapshot summary should render");
        assert!(reference_snapshot_summary.contains("Reference snapshot summary"));
        assert_eq!(
            reference_snapshot_summary,
            format!(
                "Reference snapshot summary\n{}\n",
                pleiades_jpl::reference_snapshot_summary_for_report()
            )
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
        let delta_t_policy_summary =
            render_cli(&["delta-t-policy-summary"]).expect("delta T policy summary should render");
        assert!(delta_t_policy_summary.contains("Delta T policy summary"));
        assert!(delta_t_policy_summary.contains("Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"));
        let observer_policy_summary = render_cli(&["observer-policy-summary"])
            .expect("observer policy summary should render");
        assert!(observer_policy_summary.contains("Observer policy summary"));
        assert!(observer_policy_summary.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests; topocentric body positions remain unsupported"));
        let apparentness_policy_summary = render_cli(&["apparentness-policy-summary"])
            .expect("apparentness policy summary should render");
        assert!(apparentness_policy_summary.contains("Apparentness policy summary"));
        assert!(apparentness_policy_summary.contains("Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"));
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
        let selected_asteroid_boundary_summary =
            render_cli(&["selected-asteroid-boundary-summary"])
                .expect("selected asteroid boundary summary should render");
        assert!(selected_asteroid_boundary_summary.contains("Selected asteroid boundary evidence"));
        assert!(selected_asteroid_boundary_summary.contains("2451914.5"));
        assert!(
            selected_asteroid_boundary_summary.starts_with("Selected asteroid boundary evidence:")
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
            .contains("source-backed samples across 5 bodies and 11 epochs"));
        assert_eq!(
            reference_asteroid_source_window_summary,
            pleiades_jpl::reference_asteroid_source_window_summary_for_report()
        );
        let reference_holdout_overlap_summary = render_cli(&["reference-holdout-overlap-summary"])
            .expect("reference/hold-out overlap summary should render");
        assert!(reference_holdout_overlap_summary.contains("Reference/hold-out overlap:"));
        assert!(reference_holdout_overlap_summary.contains("shared body-epoch pairs"));
        assert_eq!(
            reference_holdout_overlap_summary,
            pleiades_jpl::reference_holdout_overlap_summary_for_report()
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
        assert!(house_validation_summary.contains("House validation corpus: 5 scenarios"));
        assert!(house_validation_summary
            .contains("formula families: Equal, Whole Sign, Quadrant, Equatorial projection"));
        assert!(house_validation_summary
            .contains("latitude-sensitive systems: Koch, Placidus, Topocentric"));
        assert_eq!(
            house_validation_summary,
            house_validation_summary_line_for_report(&house_validation_report())
        );
        let ayanamsa_catalog_validation_summary_rendered =
            render_cli(&["ayanamsa-catalog-validation-summary"])
                .expect("ayanamsa catalog validation summary should render");
        assert!(ayanamsa_catalog_validation_summary_rendered
            .contains("ayanamsa catalog validation: ok"));
        assert!(ayanamsa_catalog_validation_summary_rendered.contains("baseline=5, release=54"));
        assert!(ayanamsa_catalog_validation_summary_rendered.contains("custom-definition-only="));
        let ayanamsa_metadata_coverage_summary =
            render_cli(&["ayanamsa-metadata-coverage-summary"])
                .expect("ayanamsa metadata coverage summary should render");
        assert!(ayanamsa_metadata_coverage_summary.contains("ayanamsa sidereal metadata:"));
        assert_eq!(
            ayanamsa_metadata_coverage_summary,
            super::validate_render_cli(&["ayanamsa-metadata-coverage-summary"])
                .expect("validation ayanamsa metadata coverage summary should render")
        );
        let ayanamsa_reference_offsets_summary =
            render_cli(&["ayanamsa-reference-offsets-summary"])
                .expect("ayanamsa reference offsets summary should render");
        assert!(ayanamsa_reference_offsets_summary
            .contains("Ayanamsa reference offsets: representative zero-point examples:"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
        assert!(ayanamsa_reference_offsets_summary
            .contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
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
        let source_documentation_summary = render_cli(&["source-documentation-summary"])
            .expect("source documentation summary should render");
        assert!(source_documentation_summary.contains("VSOP87 source documentation:"));
        assert_eq!(
            source_documentation_summary,
            super::validate_render_cli(&["source-documentation-summary"])
                .expect("validation source documentation summary should render")
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
        assert!(release_summary.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit TT/TDB/UTC/UT1 flags)"
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

        let packaged_artifact_production_profile =
            render_cli(&["packaged-artifact-production-profile-summary"])
                .expect("packaged artifact production profile summary should render");
        assert!(packaged_artifact_production_profile
            .contains("Packaged artifact production profile skeleton:"));
        assert!(packaged_artifact_production_profile
            .contains("profile id=pleiades-packaged-artifact-profile/stage-5-prototype"));
        assert_eq!(
            packaged_artifact_production_profile,
            pleiades_data::packaged_artifact_production_profile_summary_for_report()
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

        let packaged_artifact_generation_policy =
            render_cli(&["packaged-artifact-generation-policy-summary"])
                .expect("packaged artifact generation policy summary should render");
        assert_eq!(
            packaged_artifact_generation_policy,
            pleiades_data::packaged_artifact_generation_policy_summary_for_report()
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
        assert!(regenerated.contains("stage-5 packaged-data prototype"));
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

        let report = render_cli(&["report", "--rounds", "10"])
            .expect("report should render through the primary CLI");
        assert!(report.contains("Validation report"));
        assert!(report.contains("Comparison corpus"));
        assert!(report.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"));
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
        assert!(validation_summary.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"));
        assert!(validation_summary.contains("Release bundle verification: verify-release-bundle"));
        assert!(validation_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(validation_summary.contains("Release notes summary: release-notes-summary"));
        assert!(validation_summary.contains("Release checklist summary: release-checklist-summary"));
        assert!(validation_summary.contains("Release summary: release-summary"));
        assert!(validation_summary.contains("House validation corpus"));
        assert!(validation_summary.contains("Benchmark summaries"));
        assert!(validation_summary.contains("Packaged-data benchmark"));

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
        assert!(rendered.contains("Frame policy: ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
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
        assert!(bundle_dir.join("bundle-manifest.txt").exists());
        assert!(bundle_dir
            .join("packaged-artifact-generation-manifest.txt")
            .exists());
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
        assert!(verified.contains("bundle-manifest.checksum.txt"));
    }

    #[test]
    fn unknown_command_is_rejected() {
        let error = render_cli(&["compatibility-profile-snapshot"])
            .expect_err("unknown commands should fail");
        assert!(error.contains("unknown command: compatibility-profile-snapshot"));
        assert!(error.contains("compare-backends       Compare the JPL snapshot against the algorithmic composite backend"));
        assert!(error.contains("compare-backends-audit Compare the JPL snapshot against the algorithmic composite backend and fail if the tolerance audit reports regressions"));
        assert!(error.contains("compatibility-profile  Print the release compatibility profile"));
        assert!(error.contains("verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs"));
        assert!(error
            .contains("bundle-release         Write the staged release bundle, artifact summary, packaged-artifact generation manifest, benchmark report, and manifest files"));
        assert!(error.contains("verify-release-bundle  Read a staged release bundle back and verify its manifest checksums"));
        assert!(error.contains("release-notes          Print the release compatibility notes"));
        assert!(error.contains("release-notes-summary   Print the compact release notes summary"));
        assert!(
            error.contains("release-checklist-summary Print the compact release checklist summary")
        );
        assert!(error.contains("release-summary        Print the compact release summary"));
        assert!(error.contains(
            "jpl-batch-error-taxonomy-summary  Print the compact JPL batch error taxonomy summary"
        ));
        assert!(error.contains("production-generation-boundary-summary  Print the compact production-generation boundary overlay summary"));
        assert!(error.contains("production-generation-source-window-summary  Print the compact production-generation source windows summary"));
        assert!(error.contains("comparison-snapshot-source-window-summary  Print the compact comparison snapshot source windows summary"));
        assert!(error.contains("comparison-snapshot-manifest-summary  Print the compact comparison snapshot manifest summary"));
        assert!(error.contains(
            "comparison-snapshot-summary  Print the compact comparison snapshot summary"
        ));
        assert!(error.contains("reference-snapshot-source-window-summary  Print the compact reference snapshot source windows summary"));
        assert!(error.contains("reference-snapshot-lunar-boundary-summary  Print the compact reference lunar boundary evidence summary"));
        assert!(error.contains("reference-snapshot-manifest-summary  Print the compact reference snapshot manifest summary"));
        assert!(error
            .contains("reference-snapshot-summary  Print the compact reference snapshot summary"));
        assert!(error
            .contains("time-scale-policy-summary  Print the compact time-scale policy summary"));
        assert!(error.contains("delta-t-policy-summary   Print the compact Delta T policy summary"));
        assert!(
            error.contains("observer-policy-summary  Print the compact observer policy summary")
        );
        assert!(error.contains(
            "apparentness-policy-summary  Print the compact apparentness policy summary"
        ));
        assert!(error.contains(
            "interpolation-posture-summary  Print the compact JPL interpolation posture summary"
        ));
        assert!(error.contains(
            "interpolation-quality-summary  Print the compact JPL interpolation quality summary"
        ));
        assert!(error.contains(
            "lunar-source-window-summary  Print the compact lunar source windows summary"
        ));
        assert!(error.contains("selected-asteroid-boundary-summary  Print the compact selected-asteroid boundary evidence summary"));
        assert!(error.contains("selected-asteroid-source-evidence-summary  Print the compact selected-asteroid source evidence summary"));
        assert!(error.contains("selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary"));
        assert!(error.contains("selected-asteroid-batch-parity-summary  Print the compact selected-asteroid batch-parity summary"));
        assert!(error.contains("reference-asteroid-evidence-summary  Print the compact reference asteroid evidence summary"));
        assert!(error.contains("reference-asteroid-equatorial-evidence-summary  Print the compact reference asteroid equatorial evidence summary"));
        assert!(error.contains("reference-holdout-overlap-summary  Print the compact reference/hold-out overlap summary"));
        assert!(error.contains(
            "house-validation-summary   Print the compact house-validation corpus summary"
        ));
        assert!(error.contains("ayanamsa-catalog-validation-summary  Print the compact ayanamsa catalog validation summary"));
        assert!(error.contains("ayanamsa-metadata-coverage-summary  Print the compact ayanamsa sidereal metadata coverage summary"));
        assert!(error.contains("ayanamsa-reference-offsets-summary  Print the compact ayanamsa reference offsets summary"));
        assert!(error.contains("frame-policy-summary   Print the compact frame-policy summary"));
        assert!(error.contains("release-profile-identifiers-summary  Print the compact release-profile identifiers summary"));
        assert!(error.contains(
            "request-surface-summary  Print the compact request-surface inventory summary"
        ));
        assert!(error.contains("request-policy-summary  Print the compact request-policy summary"));
        assert!(error.contains("request-semantics-summary Alias for request-policy-summary"));
        assert!(error.contains(
            "comparison-tolerance-policy-summary  Print the compact comparison tolerance policy summary"
        ));
        assert!(error.contains("pluto-fallback-summary   Print the compact Pluto fallback summary"));
        assert!(error.contains(
            "packaged-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary"
        ));
        assert!(error.contains(
            "artifact-profile-coverage-summary  Print the packaged-artifact profile coverage summary"
        ));
        assert!(error.contains(
            "packaged-artifact-production-profile-summary  Print the packaged-artifact production profile skeleton summary"
        ));
        assert!(error.contains(
            "packaged-artifact-generation-manifest-summary  Print the packaged-artifact generation manifest summary"
        ));
        assert!(
            error.contains("artifact-summary       Print the compact packaged-artifact summary")
        );
        assert!(error.contains(
            "validate-artifact      Inspect and validate the bundled compressed artifact"
        ));
        assert!(error.contains("report                 Print the full validation report"));
        assert!(error.contains("generate-report        Alias for report"));
        assert!(error
            .contains("validation-report-summary  Print the compact validation report summary"));
        assert!(error.contains("validation-summary     Alias for validation-report-summary"));
        assert!(error.contains("report-summary         Alias for validation-report-summary"));
        assert!(error.contains("chart                  Render a basic chart report"));
    }

    #[test]
    fn chart_help_text_spells_out_the_shared_request_policy() {
        let help = render_chart(&["--help"]).expect("chart help should render");
        assert!(help.contains(&shared_request_policy_help_block()));
    }

    #[test]
    fn help_text_lists_the_packaged_lookup_epoch_policy_summary_command() {
        let help = render_cli(&["help"]).expect("help text should render");
        assert!(help.contains(
            "packaged-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary"
        ));
        assert!(help.contains(
            "artifact-profile-coverage-summary  Print the packaged-artifact profile coverage summary"
        ));
        assert!(help.contains(
            "packaged-artifact-output-support-summary  Print the packaged-artifact output support summary"
        ));
        assert!(help.contains(
            "packaged-artifact-production-profile-summary  Print the packaged-artifact production profile skeleton summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-manifest-summary  Print the packaged-artifact generation manifest summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-policy-summary  Print the packaged-artifact generation policy summary"
        ));
        assert!(help.contains(
            "packaged-artifact-generation-residual-bodies-summary  Print the packaged-artifact generation residual bodies summary"
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
            "independent-holdout-batch-parity-summary  Print the compact independent hold-out batch parity summary"
        ));
        assert!(help.contains(
            "independent-holdout-equatorial-parity-summary  Print the compact independent hold-out equatorial parity summary"
        ));
        assert!(help.contains(
            "comparison-snapshot-summary  Print the compact comparison snapshot summary"
        ));
        assert!(help.contains(
            "comparison-snapshot-source-summary  Print the compact comparison snapshot source summary"
        ));
        assert!(help.contains(
            "reference-snapshot-manifest-summary  Print the compact reference snapshot manifest summary"
        ));
        assert!(help.contains(
            "reference-snapshot-source-summary  Print the compact reference snapshot source summary"
        ));
        assert!(help
            .contains("reference-snapshot-summary  Print the compact reference snapshot summary"));
        assert!(help.contains(
            "selected-asteroid-source-evidence-summary  Print the compact selected-asteroid source evidence summary"
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
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--ayanamsa",
            "custom:True Balarama|2451545.0|12.5",
            "--body",
            "Sun",
        ])
        .expect("custom ayanamsa chart should render");

        assert!(rendered.contains("Sidereal"));
        assert!(rendered.contains("True Balarama"));
        assert!(rendered.contains("12.5"));
        assert!(rendered.contains("Custom ayanamsa definition supplied via the CLI"));
    }

    #[test]
    fn parse_ayanamsa_accepts_custom_definition_labels() {
        let custom = parse_ayanamsa("custom-definition:True Balarama|2451545.0|12.5")
            .expect("custom ayanamsa should parse");

        assert_eq!(
            custom,
            Ayanamsa::Custom(CustomAyanamsa {
                name: "True Balarama".to_owned(),
                description: Some("Custom ayanamsa definition supplied via the CLI".to_owned()),
                epoch: Some(JulianDay::from_days(2_451_545.0)),
                offset_degrees: Some(Angle::from_degrees(12.5)),
            })
        );
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
