//! Command-line argument parsing helpers.

use pleiades_core::{
    resolve_ayanamsa, resolve_house_system, Angle, Ayanamsa, CelestialBody, CustomAyanamsa,
    CustomBodyId, HouseSystem, JulianDay,
};

pub(crate) fn parse_rounds(args: &[&str], default: usize) -> Result<usize, String> {
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

pub(crate) fn parse_release_bundle_output_dir<'a>(args: &'a [&'a str]) -> Result<&'a str, String> {
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

pub(crate) fn parse_f64(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let value = value.ok_or_else(|| format!("missing value for {flag}"))?;
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid value for {flag}: {error}"))
}

pub(crate) fn parse_seconds(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let seconds = parse_f64(value, flag)?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(format!(
            "invalid value for {flag}: expected a finite nonnegative number"
        ));
    }

    Ok(seconds)
}

pub(crate) fn parse_signed_seconds(value: Option<&str>, flag: &str) -> Result<f64, String> {
    let seconds = parse_f64(value, flag)?;
    if !seconds.is_finite() {
        return Err(format!(
            "invalid value for {flag}: expected a finite number"
        ));
    }

    Ok(seconds)
}

pub(crate) fn parse_body(value: Option<&str>) -> Result<CelestialBody, String> {
    let value = value.ok_or_else(|| "missing value for --body".to_string())?;
    if let Some(body) = parse_builtin_body(value) {
        return Ok(body);
    }

    parse_custom_body(value)
}

pub(crate) fn parse_builtin_body(value: &str) -> Option<CelestialBody> {
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
        "cupido" => Some(CelestialBody::Cupido),
        "hades" => Some(CelestialBody::Hades),
        "zeus" => Some(CelestialBody::Zeus),
        "kronos" => Some(CelestialBody::Kronos),
        "apollon" => Some(CelestialBody::Apollon),
        "admetos" => Some(CelestialBody::Admetos),
        "vulkanus" => Some(CelestialBody::Vulkanus),
        "poseidon" => Some(CelestialBody::Poseidon),
        "transpluto" => Some(CelestialBody::Transpluto),
        "nibiru" => Some(CelestialBody::Nibiru),
        "harrington" => Some(CelestialBody::Harrington),
        "neptune (leverrier)" => Some(CelestialBody::NeptuneLeverrier),
        "neptune (adams)" => Some(CelestialBody::NeptuneAdams),
        "pluto (lowell)" => Some(CelestialBody::PlutoLowell),
        "pluto (pickering)" => Some(CelestialBody::PlutoPickering),
        "vulcan" => Some(CelestialBody::Vulcan),
        "white moon" => Some(CelestialBody::WhiteMoon),
        "proserpina" => Some(CelestialBody::Proserpina),
        "waldemath" => Some(CelestialBody::Waldemath),
        _ => None,
    }
}

pub(crate) fn parse_custom_body(value: &str) -> Result<CelestialBody, String> {
    let (catalog, designation) = value
        .split_once(':')
        .ok_or_else(|| format!("unsupported body name: {value}"))?;

    let custom = CustomBodyId::new(catalog, designation);
    custom.validate().map_err(|error| error.to_string())?;

    Ok(CelestialBody::Custom(custom))
}

pub(crate) fn parse_ayanamsa(value: &str) -> Result<Ayanamsa, String> {
    if let Some(builtin) = resolve_ayanamsa(value) {
        return Ok(builtin);
    }

    if let Some(custom) = parse_custom_ayanamsa(value)? {
        return Ok(custom);
    }

    Err(format!("unsupported ayanamsa name: {value}"))
}

pub(crate) fn parse_custom_ayanamsa(value: &str) -> Result<Option<Ayanamsa>, String> {
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

pub(crate) fn strip_custom_ayanamsa_prefix(value: &str) -> Option<&str> {
    strip_case_insensitive_prefix(value, "custom:")
        .or_else(|| strip_case_insensitive_prefix(value, "custom-definition:"))
}

pub(crate) fn strip_case_insensitive_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let head = value.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then_some(&value[prefix.len()..])
}

pub(crate) fn parse_house_system(value: &str) -> Result<HouseSystem, String> {
    resolve_house_system(value).ok_or_else(|| format!("unsupported house system name: {value}"))
}
