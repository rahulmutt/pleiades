//! Chart command: render a basic chart report.

use core::time::Duration;

use pleiades_core::{
    default_chart_bodies, Apparentness, CelestialBody, ChartEngine, ChartRequest, CivilDateTime,
    CompositeBackend, HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation,
    RoutingBackend, TimeScale, ZodiacMode,
};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_validate::current_request_surface_summary;
use pleiades_vsop87::Vsop87Backend;

use crate::help::shared_request_policy_help_block;
use crate::parse::{
    parse_ayanamsa, parse_body, parse_f64, parse_house_system, parse_seconds, parse_signed_seconds,
};
use crate::render::render_error;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ChartInstantConversionFlags {
    pub(crate) tt_offset_seconds: Option<f64>,
    pub(crate) tdb_offset_seconds: Option<f64>,
    pub(crate) tdb_from_utc_offset_seconds: Option<f64>,
    pub(crate) tdb_from_ut1_offset_seconds: Option<f64>,
    pub(crate) tdb_from_tt_offset_seconds: Option<f64>,
    pub(crate) tt_from_tdb_offset_seconds: Option<f64>,
}

pub(crate) fn build_chart_instant(
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

fn parse_civil(value: Option<&str>) -> Result<CivilDateTime, String> {
    let raw = value.ok_or_else(|| "--civil requires a YYYY-MM-DDTHH:MM:SS value".to_string())?;
    let (date, time) = raw
        .split_once('T')
        .ok_or_else(|| format!("--civil value '{raw}' must be YYYY-MM-DDTHH:MM:SS"))?;
    let d: Vec<&str> = date.split('-').collect();
    let t: Vec<&str> = time.split(':').collect();
    if d.len() != 3 || t.len() != 3 {
        return Err(format!("--civil value '{raw}' must be YYYY-MM-DDTHH:MM:SS"));
    }
    let year = d[0].parse::<i32>().map_err(|_| {
        format!(
            "--civil: invalid year '{}' (expected YYYY-MM-DDTHH:MM:SS)",
            d[0]
        )
    })?;
    let month = d[1].parse::<u8>().map_err(|_| {
        format!(
            "--civil: invalid month '{}' (expected YYYY-MM-DDTHH:MM:SS)",
            d[1]
        )
    })?;
    let day = d[2].parse::<u8>().map_err(|_| {
        format!(
            "--civil: invalid day '{}' (expected YYYY-MM-DDTHH:MM:SS)",
            d[2]
        )
    })?;
    let hour = t[0].parse::<u8>().map_err(|_| {
        format!(
            "--civil: invalid hour '{}' (expected YYYY-MM-DDTHH:MM:SS)",
            t[0]
        )
    })?;
    let minute = t[1].parse::<u8>().map_err(|_| {
        format!(
            "--civil: invalid minute '{}' (expected YYYY-MM-DDTHH:MM:SS)",
            t[1]
        )
    })?;
    let second = t[2].parse::<f64>().map_err(|_| {
        format!(
            "--civil: invalid second '{}' (expected YYYY-MM-DDTHH:MM:SS)",
            t[2]
        )
    })?;
    Ok(CivilDateTime::new(year, month, day, hour, minute, second))
}

pub(crate) fn render_chart(args: &[&str]) -> Result<String, String> {
    let mut jd: Option<f64> = None;
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut elevation: Option<f64> = None;
    let mut topocentric = false;
    let mut bodies: Vec<CelestialBody> = Vec::new();
    let mut zodiac_mode = ZodiacMode::Tropical;
    let mut time_scale = TimeScale::Tt;
    let mut time_scale_explicit = false;
    let mut tt_offset_seconds: Option<f64> = None;
    let mut tdb_offset_seconds: Option<f64> = None;
    let mut tdb_from_utc_offset_seconds: Option<f64> = None;
    let mut tdb_from_ut1_offset_seconds: Option<f64> = None;
    let mut tdb_from_tt_offset_seconds: Option<f64> = None;
    let mut apparentness = Apparentness::Apparent;
    let mut apparentness_explicit = false;
    let mut house_system: Option<HouseSystem> = None;
    let mut tt_from_tdb_offset_seconds: Option<f64> = None;
    let mut civil: Option<CivilDateTime> = None;
    let mut civil_scale = TimeScale::Utc;
    let mut civil_target = TimeScale::Tt;

    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--jd" => jd = Some(parse_f64(iter.next(), "--jd")?),
            "--lat" => lat = Some(parse_f64(iter.next(), "--lat")?),
            "--lon" => lon = Some(parse_f64(iter.next(), "--lon")?),
            "--elevation" => elevation = Some(parse_f64(iter.next(), "--elevation")?),
            "--topocentric" => topocentric = true,
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
            "--civil" => civil = Some(parse_civil(iter.next())?),
            "--civil-scale" => {
                civil_scale = match iter.next() {
                    Some("utc") => TimeScale::Utc,
                    Some("ut1") => TimeScale::Ut1,
                    other => return Err(format!("--civil-scale must be utc|ut1, got {other:?}")),
                };
            }
            "--civil-target" => {
                civil_target = match iter.next() {
                    Some("tt") => TimeScale::Tt,
                    Some("tdb") => TimeScale::Tdb,
                    other => return Err(format!("--civil-target must be tt|tdb, got {other:?}")),
                };
            }
            "--help" | "-h" => {
                let chart_request_surface = current_request_surface_summary();
                let chart_help_clause = chart_request_surface
                    .validated_chart_help_clause()
                    .map_err(render_error)?;
                return Ok(format!(
                    "{}\n\nUsage:\n  chart [--jd <julian-day>] [--lat <deg> --lon <deg> [--elevation <m>]] [--tt|--tdb|--utc|--ut1] [--tt-offset-seconds <seconds>|--tt-from-utc-offset-seconds <seconds>|--tt-from-ut1-offset-seconds <seconds>] [--tdb-offset-seconds <seconds>|--tdb-from-utc-offset-seconds <seconds>|--tdb-from-ut1-offset-seconds <seconds>] [--tdb-from-tt-offset-seconds <seconds>] [--tt-from-tdb-offset-seconds <seconds>] [--civil <YYYY-MM-DDTHH:MM:SS>] [--civil-scale utc|ut1] [--civil-target tt|tdb] [--mean (diagnostic: raw J2000)|--apparent (default for release-grade bodies)] [--topocentric] [--ayanamsa <name>] [--house-system <name>] [--body <name> ...]\n\nApparent place of date is the default for release-grade bodies (Sun and others with a known distance); per-body provenance lines are appended to the output. Use --mean for raw J2000 diagnostic output. --topocentric applies diurnal parallax + diurnal aberration for the --lat/--lon/--elevation observer; requires apparent mode. Ayanamsa names may be built-in entries or custom definitions in the form custom:<name>|<epoch-jd>|<offset-degrees> (or custom-definition:<name>|<epoch-jd>|<offset-degrees>). Body names may be built-in bodies such as Sun or Moon, or custom identifiers in the form catalog:designation. {}\n\n{}\n",
                    crate::cli::banner(),
                    chart_help_clause,
                    shared_request_policy_help_block()
                ));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    if civil.is_some()
        && (jd.is_some()
            || time_scale_explicit
            || tt_offset_seconds.is_some()
            || tdb_offset_seconds.is_some()
            || tdb_from_utc_offset_seconds.is_some()
            || tdb_from_ut1_offset_seconds.is_some()
            || tdb_from_tt_offset_seconds.is_some()
            || tt_from_tdb_offset_seconds.is_some())
    {
        return Err(
            "--civil cannot be combined with --jd, manual time-scale flags, or offset flags"
                .to_string(),
        );
    }

    if topocentric && apparentness == Apparentness::Mean {
        return Err("topocentric positions require apparent place; remove --mean".to_string());
    }
    if topocentric && (lat.is_none() || lon.is_none()) {
        return Err("topocentric positions require both --lat and --lon".to_string());
    }

    let (instant, civil_provenance) = match civil {
        Some(c) => {
            let built = ChartRequest::from_civil(c, civil_scale, civil_target, Vec::new())
                .map_err(|e| e.summary_line())?;
            (built.request.instant, Some(built.provenance))
        }
        None => {
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
            (instant, None)
        }
    };
    let observer = match (lat, lon) {
        (Some(lat), Some(lon)) => Some(ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(lon),
            elevation,
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
    if topocentric {
        request = request.with_topocentric(true);
    }
    if let Some(house_system) = house_system {
        request = request.with_house_system(house_system);
    }

    let snapshot = engine.chart(&request).map_err(render_error)?;
    let mut output = snapshot.to_string();
    for placement in &snapshot.placements {
        if let Some(provenance) = &placement.apparent {
            output.push_str(&format!(
                "  {} apparent: {}\n",
                placement.body,
                provenance.summary_line()
            ));
        }
        if let Some(topo_prov) = &placement.topocentric {
            output.push_str(&format!("  {}\n", topo_prov.summary_line()));
        }
    }
    match civil_provenance {
        Some(p) => Ok(format!("{output}\n{}", p.summary_line())),
        None => Ok(output),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_chart_emits_apparent_provenance_line() {
        let out = render_chart(&["--jd", "2451545.0", "--body", "Sun"]).unwrap();
        // ApparentProvenance::summary_line() starts with "apparent-place light_time=..."
        assert!(
            out.contains("apparent-place light_time"),
            "missing provenance line in:\n{out}"
        );
    }

    #[test]
    fn mean_flag_suppresses_apparent_provenance() {
        let out = render_chart(&["--jd", "2451545.0", "--body", "Sun", "--mean"]).unwrap();
        // With --mean, no per-body ApparentProvenance provenance line should appear
        assert!(
            !out.contains("apparent-place light_time"),
            "mean output should have no provenance line:\n{out}"
        );
    }

    #[test]
    fn topocentric_requires_observer() {
        let err =
            render_chart(&["--jd", "2451545.0", "--body", "Moon", "--topocentric"]).unwrap_err();
        assert!(
            err.contains("observer") || err.contains("--lat"),
            "got: {err}"
        );
    }

    #[test]
    fn topocentric_conflicts_with_mean() {
        let err = render_chart(&[
            "--jd",
            "2451545.0",
            "--body",
            "Moon",
            "--lat",
            "40",
            "--lon",
            "-3.7",
            "--topocentric",
            "--mean",
        ])
        .unwrap_err();
        assert!(
            err.contains("apparent") || err.contains("--mean"),
            "got: {err}"
        );
    }

    #[test]
    fn topocentric_moon_emits_provenance_line() {
        let out = render_chart(&[
            "--jd",
            "2451545.0",
            "--body",
            "Moon",
            "--lat",
            "40",
            "--lon",
            "-3.7",
            "--elevation",
            "650",
            "--topocentric",
        ])
        .unwrap();
        assert!(
            out.contains("topocentric"),
            "missing topocentric provenance: {out}"
        );
    }
}
