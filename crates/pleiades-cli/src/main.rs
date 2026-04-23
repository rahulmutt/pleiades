//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! The CLI now exposes the compatibility profile and a small chart report
//! command so contributors can exercise the first end-to-end workflow without
//! leaving the repository. The chart report keeps the mean/apparent position
//! choice explicit so report consumers can see which backend mode was used.

#![forbid(unsafe_code)]

use pleiades_core::{
    current_api_stability_profile, default_chart_bodies, resolve_ayanamsa, resolve_house_system,
    Apparentness, Ayanamsa, CelestialBody, ChartEngine, ChartRequest, CompositeBackend,
    EphemerisError, HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation,
    RoutingBackend, TimeScale, ZodiacMode,
};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_validate::render_backend_matrix_report;
use pleiades_vsop87::Vsop87Backend;

fn banner() -> &'static str {
    "pleiades-cli chart MVP"
}

fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compatibility-profile") | Some("profile") => {
            Ok(pleiades_core::current_compatibility_profile().to_string())
        }
        Some("api-stability") | Some("api-posture") => {
            Ok(current_api_stability_profile().to_string())
        }
        Some("backend-matrix") | Some("capability-matrix") => {
            render_backend_matrix_report().map_err(render_error)
        }
        Some("chart") => render_chart(&args[1..]),
        Some("help") | Some("--help") | Some("-h") => Ok(format!(
            "{}\n\nCommands:\n  compatibility-profile  Print the release compatibility profile\n  profile                Alias for compatibility-profile\n  api-stability          Print the release API stability posture\n  api-posture            Alias for api-stability\n  backend-matrix         Print the implemented backend capability matrices\n  capability-matrix      Alias for backend-matrix\n  chart                  Render a basic chart report\n    --mean               Force mean positions for backend queries\n    --apparent           Force apparent positions for backend queries\n  help                   Show this help text",
            banner()
        )),
        _ => Ok(banner().to_string()),
    }
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
                    "{}\n\nUsage:\n  chart [--jd <julian-day>] [--lat <deg> --lon <deg>] [--mean|--apparent] [--ayanamsa <name>] [--house-system <name>] [--body <name> ...]",
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

fn parse_f64(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let value = value.ok_or_else(|| format!("missing value for {flag}"))?;
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid value for {flag}: {error}"))
}

fn parse_body(value: Option<&str>) -> Result<CelestialBody, String> {
    let value = value.ok_or_else(|| "missing value for --body".to_string())?;
    match value.to_ascii_lowercase().as_str() {
        "sun" => Ok(CelestialBody::Sun),
        "moon" => Ok(CelestialBody::Moon),
        "mercury" => Ok(CelestialBody::Mercury),
        "venus" => Ok(CelestialBody::Venus),
        "mars" => Ok(CelestialBody::Mars),
        "jupiter" => Ok(CelestialBody::Jupiter),
        "saturn" => Ok(CelestialBody::Saturn),
        "uranus" => Ok(CelestialBody::Uranus),
        "neptune" => Ok(CelestialBody::Neptune),
        "pluto" => Ok(CelestialBody::Pluto),
        "ceres" => Ok(CelestialBody::Ceres),
        "pallas" => Ok(CelestialBody::Pallas),
        "juno" => Ok(CelestialBody::Juno),
        "vesta" => Ok(CelestialBody::Vesta),
        "mean node" | "mean lunar node" => Ok(CelestialBody::MeanNode),
        "true node" | "true lunar node" => Ok(CelestialBody::TrueNode),
        "mean apogee" => Ok(CelestialBody::MeanApogee),
        "true apogee" => Ok(CelestialBody::TrueApogee),
        "mean perigee" => Ok(CelestialBody::MeanPerigee),
        "true perigee" => Ok(CelestialBody::TruePerigee),
        other => Err(format!("unsupported body name: {other}")),
    }
}

fn parse_ayanamsa(value: &str) -> Result<Ayanamsa, String> {
    resolve_ayanamsa(value).ok_or_else(|| format!("unsupported ayanamsa name: {value}"))
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

    use super::{banner, parse_body, render_chart, render_cli, CelestialBody};

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
