//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! The CLI now exposes the compatibility profile and a small chart report
//! command so contributors can exercise the first end-to-end workflow without
//! leaving the repository. The chart report keeps the mean/apparent position
//! choice explicit so report consumers can see which backend mode was used.

#![forbid(unsafe_code)]

use pleiades_core::{
    current_api_stability_profile, default_chart_bodies, resolve_ayanamsa, resolve_house_system,
    Angle, Apparentness, Ayanamsa, CelestialBody, ChartEngine, ChartRequest, CompositeBackend,
    CustomAyanamsa, CustomBodyId, EphemerisError, HouseSystem, Instant, JulianDay, Latitude,
    Longitude, ObserverLocation, RoutingBackend, TimeScale, ZodiacMode,
};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_validate::{
    render_api_stability_summary, render_artifact_summary, render_backend_matrix_report,
    render_backend_matrix_summary, render_cli as validate_render_cli,
    render_compatibility_profile_summary, render_release_checklist,
    render_release_checklist_summary, render_release_notes, render_release_notes_summary,
    render_release_summary, render_validation_report_summary, verify_compatibility_profile,
};
use pleiades_vsop87::Vsop87Backend;

fn banner() -> &'static str {
    "pleiades-cli chart MVP"
}

fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compatibility-profile") | Some("profile") => {
            Ok(pleiades_core::current_compatibility_profile().to_string())
        }
        Some("compatibility-profile-summary") | Some("profile-summary") => {
            Ok(render_compatibility_profile_summary())
        }
        Some("verify-compatibility-profile") => {
            verify_compatibility_profile().map_err(render_error)
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
        Some("report") | Some("generate-report") => validate_render_cli(args),
        Some("validation-report-summary") | Some("validation-summary") | Some("report-summary") => {
            render_validation_report_summary(10_000).map_err(render_error)
        }
        Some("chart") => render_chart(&args[1..]),
        Some("help") | Some("--help") | Some("-h") => Ok(help_text()),
        None => Ok(banner().to_string()),
        Some(other) => Err(format!("unknown command: {other}\n\n{}", help_text())),
    }
}

fn help_text() -> String {
    format!(
        "{}\n\nCommands:\n  compatibility-profile  Print the release compatibility profile\n  profile                Alias for compatibility-profile\n  compatibility-profile-summary  Print the compact compatibility profile summary\n  profile-summary        Alias for compatibility-profile-summary\n  verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs\n  verify-release-bundle  Read a staged release bundle back and verify its manifest checksums\n  api-stability          Print the release API stability posture\n  api-posture            Alias for api-stability\n  api-stability-summary  Print the compact API stability summary\n  api-posture-summary    Alias for api-stability-summary\n  backend-matrix         Print the implemented backend capability matrices\n  capability-matrix      Alias for backend-matrix\n  backend-matrix-summary Print the compact backend capability matrix summary\n  matrix-summary         Alias for backend-matrix-summary\n  release-notes          Print the release compatibility notes\n  release-notes-summary   Print the compact release notes summary\n  release-checklist      Print the release maintainer checklist\n  release-checklist-summary Print the compact release checklist summary\n  checklist-summary      Alias for release-checklist-summary\n  release-summary        Print the compact release summary\n  artifact-summary       Print the compact packaged-artifact summary\n  artifact-posture-summary  Alias for artifact-summary\n  report                 Print the full validation report\n  generate-report        Alias for report\n  validation-report-summary  Print the compact validation report summary\n  validation-summary     Alias for validation-report-summary\n  report-summary         Alias for validation-report-summary\n  chart                  Render a basic chart report\n    --mean               Force mean positions for backend queries\n    --apparent           Force apparent positions for backend queries\n    --body <name>        Use a built-in body or a custom catalog:designation identifier\n  help                   Show this help text",
        banner()
    )
}

fn render_chart(args: &[&str]) -> Result<String, String> {
    let mut jd: Option<f64> = None;
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut bodies: Vec<CelestialBody> = Vec::new();
    let mut zodiac_mode = ZodiacMode::Tropical;
    let mut apparentness = Apparentness::Mean;
    let mut apparentness_explicit = false;
    let mut house_system: Option<HouseSystem> = None;

    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--jd" => jd = Some(parse_f64(iter.next(), "--jd")?),
            "--lat" => lat = Some(parse_f64(iter.next(), "--lat")?),
            "--lon" => lon = Some(parse_f64(iter.next(), "--lon")?),
            "--body" => bodies.push(parse_body(iter.next())?),
            "--mean" => {
                if apparentness_explicit && apparentness != Apparentness::Mean {
                    return Err("conflicting apparentness flags: --mean and --apparent".to_string());
                }
                apparentness = Apparentness::Mean;
                apparentness_explicit = true;
            }
            "--apparent" => {
                if apparentness_explicit && apparentness != Apparentness::Apparent {
                    return Err("conflicting apparentness flags: --mean and --apparent".to_string());
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
                    "{}\n\nUsage:\n  chart [--jd <julian-day>] [--lat <deg> --lon <deg>] [--mean|--apparent] [--ayanamsa <name>] [--house-system <name>] [--body <name> ...]\n\nAyanamsa names may be built-in entries or custom definitions in the form custom:<name>|<epoch-jd>|<offset-degrees> (or custom-definition:<name>|<epoch-jd>|<offset-degrees>). Body names may be built-in bodies such as Sun or Moon, or custom identifiers in the form catalog:designation.",
                    banner()
                ));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let jd = jd.unwrap_or(2_451_545.0);
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
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

fn parse_f64(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let value = value.ok_or_else(|| format!("missing value for {flag}"))?;
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid value for {flag}: {error}"))
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
    let catalog = catalog.trim();
    let designation = designation.trim();
    if catalog.is_empty() || designation.is_empty() {
        return Err(format!("unsupported body name: {value}"));
    }

    Ok(CelestialBody::Custom(CustomBodyId::new(
        catalog,
        designation,
    )))
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

    let mut parts = value.split('|').map(str::trim);
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

    Ok(Some(Ayanamsa::Custom(CustomAyanamsa {
        name: name.to_owned(),
        description: Some("Custom ayanamsa definition supplied via the CLI".to_owned()),
        epoch: Some(JulianDay::from_days(epoch)),
        offset_degrees: Some(Angle::from_degrees(offset)),
    })))
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
    format!("{}", error)
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
    use pleiades_core::current_release_profile_identifiers;

    use super::{
        banner, parse_ayanamsa, parse_body, render_chart, render_cli, Angle, Ayanamsa,
        CelestialBody, CustomAyanamsa, CustomBodyId, JulianDay,
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

    #[test]
    fn banner_mentions_package() {
        assert!(banner().contains("pleiades-cli"));
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

        let compatibility = render_cli(&["compatibility-profile-summary"])
            .expect("compatibility summary should render");
        assert!(compatibility.contains("Compatibility profile summary"));
        assert!(compatibility.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(compatibility.contains("House systems: 25 total"));
        assert!(compatibility
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(compatibility.contains("Release notes summary: release-notes-summary"));
        assert!(compatibility.contains("Release checklist summary: release-checklist-summary"));
        assert!(compatibility
            .contains("See release-summary for the compact one-screen release overview."));

        let verification = render_cli(&["verify-compatibility-profile"])
            .expect("compatibility profile verification should render");
        assert!(verification.contains("Compatibility profile verification"));
        assert!(verification.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(verification.contains("House systems verified:"));
        assert!(verification.contains("Ayanamsas verified:"));

        let backend_matrix =
            render_cli(&["backend-matrix-summary"]).expect("backend matrix summary should render");
        assert!(backend_matrix.contains("Backend matrix summary"));
        assert!(backend_matrix.contains("Backends: 5"));
        assert!(backend_matrix.contains("Accuracy classes: Exact: 1"));

        let api_stability =
            render_cli(&["api-stability-summary"]).expect("api stability summary should render");
        assert!(api_stability.contains("API stability summary"));
        assert!(api_stability.contains("Stable surfaces: 6"));
        assert!(api_stability.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
        )));

        let release_notes = render_cli(&["release-notes"]).expect("release notes should render");
        assert!(release_notes.contains("Release notes"));
        assert!(release_notes.contains("Release notes summary: release-notes-summary"));
        assert!(release_notes.contains("Release checklist summary: release-checklist-summary"));
        assert!(release_notes.contains("Release bundle verification: verify-release-bundle"));
        assert!(release_notes
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_notes.contains("API stability posture:"));
        assert!(release_notes.contains("Bundle provenance:"));

        let release_notes_summary =
            render_cli(&["release-notes-summary"]).expect("release notes summary should render");
        assert!(release_notes_summary.contains("Release notes summary"));
        assert!(release_notes_summary.contains("API stability summary line:"));
        assert!(release_notes_summary.contains("Release notes: release-notes"));
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

        let release_checklist =
            render_cli(&["release-checklist"]).expect("release checklist should render");
        assert!(release_checklist.contains("Release checklist"));
        assert!(release_checklist.contains("Release notes summary: release-notes-summary"));
        assert!(release_checklist.contains("Compact summary views: backend-matrix-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
        assert!(release_checklist.contains("Repository-managed release gates:"));
        assert!(release_checklist.contains("bundle-release --out /tmp/pleiades-release"));
        assert!(release_checklist.contains("release-checklist-summary.txt"));

        let release_checklist_summary = render_cli(&["release-checklist-summary"])
            .expect("release checklist summary should render");
        assert!(release_checklist_summary.contains("Release checklist summary"));
        assert!(release_checklist_summary.contains("Release notes summary: release-notes-summary"));
        assert!(release_checklist_summary
            .contains("Release bundle verification: verify-release-bundle"));
        assert!(release_checklist_summary.contains("Compact summary views: backend-matrix-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
        assert!(release_checklist_summary.contains("Repository-managed release gates: 5 items"));
        assert!(release_checklist_summary.contains("Manual bundle workflow: 3 items"));
        assert!(release_checklist_summary.contains("Bundle contents: 16 items"));
        assert!(release_checklist_summary.contains("External publishing reminders: 3 items"));
        assert!(release_checklist_summary
            .contains("See release-summary for the compact one-screen release overview."));

        let release_summary =
            render_cli(&["release-summary"]).expect("release summary should render");
        assert!(release_summary.contains("Release summary"));
        assert!(release_summary.contains("House systems:"));
        assert!(release_summary
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(release_summary.contains("Release gate reminders:"));
        assert!(release_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(release_summary.contains("Release notes summary: release-notes-summary"));
        assert!(release_summary.contains("Release bundle verification: verify-release-bundle"));
        assert!(release_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_summary.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(release_summary.contains("Release checklist summary: release-checklist-summary"));
        assert!(release_summary.contains("See release-notes and release-checklist"));

        let artifact_summary =
            render_cli(&["artifact-summary"]).expect("artifact summary should render");
        assert!(artifact_summary.contains("Artifact summary"));
        assert!(artifact_summary.contains("Model error envelope"));

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
        assert!(validation_summary.contains("House validation corpus"));
        assert!(validation_summary.contains("Benchmark summaries"));
        assert!(validation_summary.contains("Packaged-data benchmark"));

        let validation_report_summary = render_cli(&["validation-report-summary"])
            .expect("validation-report-summary should render");
        assert!(validation_report_summary.contains("Validation report summary"));
        assert!(validation_report_summary.contains("Comparison corpus"));
        assert!(validation_report_summary.contains("Benchmark summaries"));
    }

    #[test]
    fn verify_release_bundle_command_verifies_a_staged_bundle() {
        let bundle_dir = unique_temp_dir("pleiades-cli-release-bundle");
        let bundle_dir_string = bundle_dir.display().to_string();

        pleiades_validate::render_cli(&["bundle-release", "--out", &bundle_dir_string])
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
        assert!(error.contains("compatibility-profile  Print the release compatibility profile"));
        assert!(error.contains("verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs"));
        assert!(error.contains("verify-release-bundle  Read a staged release bundle back and verify its manifest checksums"));
        assert!(error.contains("release-notes          Print the release compatibility notes"));
        assert!(error.contains("release-notes-summary   Print the compact release notes summary"));
        assert!(
            error.contains("release-checklist-summary Print the compact release checklist summary")
        );
        assert!(error.contains("release-summary        Print the compact release summary"));
        assert!(error.contains("report                 Print the full validation report"));
        assert!(error.contains("generate-report        Alias for report"));
        assert!(error
            .contains("validation-report-summary  Print the compact validation report summary"));
        assert!(error.contains("validation-summary     Alias for validation-report-summary"));
        assert!(error.contains("report-summary         Alias for validation-report-summary"));
        assert!(error.contains("chart                  Render a basic chart report"));
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
    fn chart_command_can_force_apparent_positions() {
        let rendered = render_chart(&["--jd", "2451545.0", "--apparent", "--body", "Sun"])
            .expect("chart should render");
        assert!(rendered.contains("Apparentness: Apparent"));
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
    fn chart_command_accepts_sidereal_ayanamsa() {
        let rendered =
            render_chart(&["--jd", "2451545.0", "--ayanamsa", "Lahiri", "--body", "Sun"])
                .expect("sidereal chart should render");
        assert!(rendered.contains("Sidereal"));
        assert!(rendered.contains("Lahiri"));
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
