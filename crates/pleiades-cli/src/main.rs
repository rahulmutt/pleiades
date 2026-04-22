//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! The CLI now exposes the compatibility profile and a small tropical chart
//! report command so contributors can exercise the first end-to-end workflow
//! without leaving the repository.

#![forbid(unsafe_code)]

use pleiades_core::{
    default_chart_bodies, CelestialBody, ChartEngine, ChartRequest, CompositeBackend,
    EphemerisError, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};
use pleiades_elp::ElpBackend;
use pleiades_vsop87::Vsop87Backend;

fn banner() -> &'static str {
    "pleiades-cli chart MVP"
}

fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compatibility-profile") | Some("profile") => {
            Ok(pleiades_core::current_compatibility_profile().to_string())
        }
        Some("chart") => render_chart(&args[1..]),
        Some("help") | Some("--help") | Some("-h") => Ok(format!(
            "{}\n\nCommands:\n  compatibility-profile  Print the current compatibility profile\n  profile                Alias for compatibility-profile\n  chart                  Render a basic tropical chart report\n  help                   Show this help text",
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

    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--jd" => jd = Some(parse_f64(iter.next(), "--jd")?),
            "--lat" => lat = Some(parse_f64(iter.next(), "--lat")?),
            "--lon" => lon = Some(parse_f64(iter.next(), "--lon")?),
            "--body" => bodies.push(parse_body(iter.next())?),
            "--help" | "-h" => {
                return Ok(format!(
                    "{}\n\nUsage:\n  chart [--jd <julian-day>] [--lat <deg> --lon <deg>] [--body <name> ...]",
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

    let backend = CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new());
    let engine = ChartEngine::new(backend);
    let mut request = ChartRequest::new(instant).with_bodies(bodies);
    if let Some(observer) = observer {
        request = request.with_observer(observer);
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
        other => Err(format!("unsupported body name: {other}")),
    }
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
    use super::{banner, render_chart, render_cli};

    #[test]
    fn banner_mentions_package() {
        assert!(banner().contains("pleiades-cli"));
    }

    #[test]
    fn profile_command_renders_catalogs() {
        let rendered = render_cli(&["compatibility-profile"]).expect("profile should render");
        assert!(rendered.contains("Built-in house systems:"));
        assert!(rendered.contains("Topocentric"));
        assert!(rendered.contains("Built-in ayanamsas:"));
        assert!(rendered.contains("Lahiri"));
    }

    #[test]
    fn chart_command_renders_bodies() {
        let rendered = render_chart(&["--jd", "2451545.0", "--body", "Sun", "--body", "Moon"])
            .expect("chart should render");
        assert!(rendered.contains("Backend:"));
        assert!(rendered.contains("Sun"));
        assert!(rendered.contains("Moon"));
    }
}
