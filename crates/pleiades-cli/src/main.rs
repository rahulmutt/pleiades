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
use pleiades_jpl::JplSnapshotBackend;
use pleiades_validate::{
    render_api_stability_summary, render_artifact_summary, render_backend_matrix_report,
    render_backend_matrix_summary, render_cli as validate_render_cli,
    render_compatibility_profile_summary, render_release_bundle, render_release_checklist,
    render_release_checklist_summary, render_release_notes, render_release_notes_summary,
    render_release_summary, render_validation_report_summary, verify_compatibility_profile,
};
use pleiades_vsop87::Vsop87Backend;

fn banner() -> &'static str {
    "pleiades-cli chart utility"
}

fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compare-backends") => validate_render_cli(&["compare-backends"]),
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
        Some("artifact-summary") | Some("artifact-posture-summary") => {
            render_artifact_summary().map_err(|error| error.to_string())
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
        Some("workspace-audit") | Some("audit") => validate_render_cli(&["workspace-audit"]),
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
        "{}\n\nCommands:\n  compatibility-profile  Print the release compatibility profile\n  profile                Alias for compatibility-profile\n  compatibility-profile-summary  Print the compact compatibility profile summary\n  profile-summary        Alias for compatibility-profile-summary\n  verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs\n  bundle-release         Write the staged release bundle, benchmark report, and manifest files\n  verify-release-bundle  Read a staged release bundle back and verify its manifest checksums\n  api-stability          Print the release API stability posture\n  api-posture            Alias for api-stability\n  api-stability-summary  Print the compact API stability summary\n  api-posture-summary    Alias for api-stability-summary\n  compare-backends       Compare the JPL snapshot against the algorithmic composite backend\n  backend-matrix         Print the implemented backend capability matrices\n  capability-matrix      Alias for backend-matrix\n  backend-matrix-summary Print the compact backend capability matrix summary\n  matrix-summary         Alias for backend-matrix-summary\n  release-notes          Print the release compatibility notes\n  release-notes-summary   Print the compact release notes summary\n  release-checklist      Print the release maintainer checklist\n  release-checklist-summary Print the compact release checklist summary\n  checklist-summary      Alias for release-checklist-summary\n  release-summary        Print the compact release summary\n  artifact-summary       Print the compact packaged-artifact summary\n  artifact-posture-summary  Alias for artifact-summary\n  validate-artifact      Inspect and validate the bundled compressed artifact\n  regenerate-packaged-artifact  Rebuild or verify the packaged artifact fixture from the checked-in reference snapshot; pass a file path, --out FILE, or --check\n  workspace-audit        Check the workspace for mandatory native build hooks\n  audit                  Alias for workspace-audit\n  report                 Print the full validation report\n  generate-report        Alias for report\n  validation-report-summary  Print the compact validation report summary\n  validation-summary     Alias for validation-report-summary\n  report-summary         Alias for validation-report-summary\n  chart                  Render a basic chart report\n    --tt|--tdb|--utc|--ut1  Tag the chart instant with a time scale
    --tt-offset-seconds <seconds>  Caller-supplied TT offset for UTC/UT1-tagged instants
    --tdb-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT/UTC/UT1-tagged instants
    --tdb-from-tt-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT-tagged instants
    --tt-from-tdb-offset-seconds <seconds> Caller-supplied signed TT-TDB offset for TDB-tagged instants
    --mean               Force mean positions for backend queries\n    --apparent           Force apparent positions for backend queries\n    --body <name>        Use a built-in body or a custom catalog:designation identifier\n  help                   Show this help text",
        banner()
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
                        "conflicting TT offset flags: use only one --tt-offset-seconds value"
                            .to_string(),
                    );
                }
                tt_offset_seconds = Some(parse_seconds(iter.next(), "--tt-offset-seconds")?);
            }
            "--tdb-offset-seconds" => {
                if tdb_offset_seconds.is_some() || tdb_from_tt_offset_seconds.is_some() {
                    return Err(
                        "conflicting TDB-TT offset flags: use either --tdb-offset-seconds or --tdb-from-tt-offset-seconds"
                            .to_string(),
                    );
                }
                tdb_offset_seconds =
                    Some(parse_signed_seconds(iter.next(), "--tdb-offset-seconds")?);
            }
            "--tdb-from-tt-offset-seconds" => {
                if tdb_offset_seconds.is_some() || tdb_from_tt_offset_seconds.is_some() {
                    return Err(
                        "conflicting TDB-TT offset flags: use either --tdb-offset-seconds or --tdb-from-tt-offset-seconds"
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
                    "{}\n\nUsage:\n  chart [--jd <julian-day>] [--lat <deg> --lon <deg>] [--tt|--tdb|--utc|--ut1] [--tt-offset-seconds <seconds>] [--tdb-offset-seconds <seconds>] [--tdb-from-tt-offset-seconds <seconds>] [--tt-from-tdb-offset-seconds <seconds>] [--mean|--apparent] [--ayanamsa <name>] [--house-system <name>] [--body <name> ...]\n\nAyanamsa names may be built-in entries or custom definitions in the form custom:<name>|<epoch-jd>|<offset-degrees> (or custom-definition:<name>|<epoch-jd>|<offset-degrees>). Body names may be built-in bodies such as Sun or Moon, or custom identifiers in the form catalog:designation. When the chart instant is tagged as UTC or UT1, the caller must also supply the explicit TT offset before chart assembly, and may also supply a signed TDB-TT offset when converting to TDB. When the chart instant is tagged as TT, the caller may supply that signed TDB-TT offset via --tdb-offset-seconds or the more explicit --tdb-from-tt-offset-seconds alias. When the chart instant is tagged as TDB, the caller may supply a signed TT-TDB offset to re-tag the request as TT before assembly. When an observer is provided, it is used for house calculations only; body positions remain geocentric unless a future topocentric mode is added.",
                    banner()
                ));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let jd = jd.unwrap_or(2_451_545.0);
    let instant = build_chart_instant(
        jd,
        time_scale,
        tt_offset_seconds,
        tdb_offset_seconds,
        tdb_from_tt_offset_seconds,
        tt_from_tdb_offset_seconds,
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

fn build_chart_instant(
    jd: f64,
    time_scale: TimeScale,
    tt_offset_seconds: Option<f64>,
    tdb_offset_seconds: Option<f64>,
    tdb_from_tt_offset_seconds: Option<f64>,
    tt_from_tdb_offset_seconds: Option<f64>,
) -> Result<Instant, String> {
    let instant = Instant::new(JulianDay::from_days(jd), time_scale);
    let tt_offset = tt_offset_seconds.map(Duration::from_secs_f64);
    let tdb_offset = tdb_offset_seconds;
    let tdb_from_tt_offset = tdb_from_tt_offset_seconds;
    let tt_from_tdb_offset = tt_from_tdb_offset_seconds;

    if tdb_offset.is_some() && tdb_from_tt_offset.is_some() {
        return Err(
            "conflicting TDB-TT offset flags: use either --tdb-offset-seconds or --tdb-from-tt-offset-seconds"
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
            if let Some(tdb_offset_seconds) = tdb_offset {
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
            if let Some(tdb_offset_seconds) = tdb_offset {
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

    use super::{
        banner, parse_ayanamsa, parse_body, regenerate_packaged_artifact, render_chart, render_cli,
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
        assert!(rendered.contains("--tdb-from-tt-offset-seconds"));
        assert!(rendered.contains("--tt-from-tdb-offset-seconds"));
        assert!(rendered.contains("Caller-supplied signed TDB-TT offset for TT-tagged instants"));
        assert!(rendered.contains("Caller-supplied signed TT-TDB offset for TDB-tagged instants"));
    }

    #[test]
    fn compare_backends_command_renders_the_comparison_report() {
        let rendered = render_cli(&["compare-backends"]).expect("compare-backends should render");
        assert!(rendered.contains("Comparison report"));
        assert!(rendered.contains("Comparison corpus"));
        assert!(rendered.contains("epoch labels:"));
        assert!(rendered.contains("Reference backend:"));
        assert!(rendered.contains("Candidate backend:"));
        assert!(rendered.contains("Samples"));
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
        assert!(compatibility.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
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
        assert!(release_notes.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
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
        assert!(release_notes_summary.contains(
            "Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto override); limits="
        ));
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
            line == "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        }));
        assert!(release_notes_summary.lines().any(|line| {
            line == "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        }));
        assert!(release_notes_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_notes_summary.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
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
        assert!(release_checklist.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
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
        assert!(release_checklist_summary.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
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
        assert!(release_summary.contains("Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.6.123, api-stability=pleiades-api-stability/0.1.0"));
        assert!(release_summary.contains("API stability summary line: API stability posture: pleiades-api-stability/0.1.0; stable surfaces: 6; experimental surfaces: 3; deprecation policy items: 4; intentional limits: 3"));
        assert!(release_summary.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T model"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; body requests stay geocentric; geocentric-only backends reject observer-bearing requests"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent requests are rejected unless a backend explicitly advertises support"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T model; observer=chart houses use observer locations; body requests stay geocentric; geocentric-only backends reject observer-bearing requests; apparentness=current first-party backends accept mean geometric output only; apparent requests are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported"
        }));
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
        assert!(release_summary.contains("Packaged frame treatment"));
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
        assert!(release_summary.contains("Release notes summary: release-notes-summary"));
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
        assert!(release_summary.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto override)"));
        assert!(release_summary.contains("coverage=Luminaries: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence, bodies=2 (Moon, Sun), samples="));
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
        assert!(release_summary.contains("Workspace audit: workspace-audit / audit"));
        assert!(release_summary
            .contains("[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile"));
        assert!(release_summary.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(release_summary.contains("Release checklist summary: release-checklist-summary"));
        assert!(release_summary.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));
        assert!(release_summary.contains("See release-notes and release-checklist"));

        let artifact_summary =
            render_cli(&["artifact-summary"]).expect("artifact summary should render");
        assert!(artifact_summary.contains("Artifact summary"));
        assert!(artifact_summary.contains("Artifact boundary envelope"));
        assert!(artifact_summary.contains("Model error envelope"));
        assert!(artifact_summary.contains("Packaged frame treatment"));
        assert!(artifact_summary.contains("Release summary: release-summary"));
        assert!(artifact_summary.contains("Release notes summary: release-notes-summary"));
        assert!(artifact_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(artifact_summary.contains("Workspace audit: workspace-audit / audit"));

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
        assert!(workspace_audit.contains("Workspace audit"));
        assert!(workspace_audit.contains("no mandatory native build hooks detected"));

        let audit = render_cli(&["audit"]).expect("audit alias should render through the CLI");
        assert!(audit.contains("Workspace audit"));
        assert!(audit.contains("no mandatory native build hooks detected"));

        let report = render_cli(&["report", "--rounds", "10"])
            .expect("report should render through the primary CLI");
        assert!(report.contains("Validation report"));
        assert!(report.contains("Comparison corpus"));
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
        assert!(validation_report_summary.contains("Benchmark summaries"));
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
        assert!(error.contains("compatibility-profile  Print the release compatibility profile"));
        assert!(error.contains("verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs"));
        assert!(error
            .contains("bundle-release         Write the staged release bundle, benchmark report, and manifest files"));
        assert!(error.contains("verify-release-bundle  Read a staged release bundle back and verify its manifest checksums"));
        assert!(error.contains("release-notes          Print the release compatibility notes"));
        assert!(error.contains("release-notes-summary   Print the compact release notes summary"));
        assert!(
            error.contains("release-checklist-summary Print the compact release checklist summary")
        );
        assert!(error.contains("release-summary        Print the compact release summary"));
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
    fn chart_help_text_spells_out_the_house_observer_separation() {
        let help = render_chart(&["--help"]).expect("chart help should render");
        assert!(help.contains(
            "When an observer is provided, it is used for house calculations only; body positions remain geocentric unless a future topocentric mode is added."
        ));
        assert!(help.contains(
            "When the chart instant is tagged as UTC or UT1, the caller must also supply the explicit TT offset before chart assembly"
        ));
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
        assert!(error.contains("InvalidRequest"));
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
