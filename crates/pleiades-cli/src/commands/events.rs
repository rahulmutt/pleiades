//! Event commands: `stations` and `aspects`, the user-facing front of the
//! `pleiades-events` finders (issues #167 (c), #168 (e) and #165).
//!
//! Both commands search either a range (`--from`/`--to`) or one event either
//! side of an instant (`--next`/`--previous` with `--at`), over one or more
//! bodies or pair-and-angle combinations, and print the events in time
//! order, each with its civil time and its TDB Julian day.

use pleiades_core::{
    Angle, Ayanamsa, CelestialBody, CivilDateTime, CivilTimeError, Instant, JulianDay,
    SiderealStarPlace, TimeScale,
};
use pleiades_events::{
    AspectEvent, CrossingFrame, CrossingReference, EventEngine, EventError, Station, StationKind,
    WINDOW_END_JD, WINDOW_START_JD,
};
use pleiades_time::{tdb_from_ut1_civil, tdb_from_utc_civil, CivilConversion};

use crate::commands::chart::{default_chart_backend, parse_civil};
use crate::parse::{parse_ayanamsa, parse_body, parse_f64, parse_star_place};

const STATIONS_USAGE: &str = "Usage:\n  stations --body <name> [--body <name> ...] (--from <instant> --to <instant> | (--next|--previous) --at <instant>) [--frame geo|mean|helio] [--ayanamsa <name>] [--star-place mean|apparent]";
const ASPECTS_USAGE: &str = "Usage:\n  aspects --pair <first>,<second> [--pair ...] --angle <degrees> [--angle ...] (--from <instant> --to <instant> | (--next|--previous) --at <instant>) [--frame geo|mean|helio] [--ayanamsa <name>] [--star-place mean|apparent]";
const SHARED_HELP: &str = "  <instant> is a TDB Julian day (2451545.0) or a civil datetime YYYY-MM-DDTHH:MM:SS,\n  read as UTC from 1972 on and as UT1 before.\n  --from/--to list every event in the range; --next/--previous with --at give one event\n  per body (stations) or per pair and angle (aspects).\n  --frame is geo (apparent geocentric, the default), mean (mean geocentric of date) or helio.\n  --ayanamsa reads longitudes in a sidereal zodiac.\n  --star-place apparent reads a star-anchored ayanamsa from its anchor star's apparent place (Swiss Ephemeris default).\n  Each line gives the event's civil time (UTC from 1972, UT1 before), its TDB Julian day,\n  and the event.";

/// Which events a command searches for.
enum Search {
    Range { from: Instant, to: Instant },
    Next(Instant),
    Previous(Instant),
}

/// The flags both event commands share.
#[derive(Default)]
struct SharedArgs {
    from: Option<Instant>,
    to: Option<Instant>,
    at: Option<Instant>,
    next: bool,
    previous: bool,
    frame: Option<CrossingFrame>,
    ayanamsa: Option<Ayanamsa>,
    star_place: Option<SiderealStarPlace>,
}

impl SharedArgs {
    /// Consumes `flag` (and its value from `rest`) when it is a shared flag.
    fn take<'a>(
        &mut self,
        flag: &str,
        rest: &mut impl Iterator<Item = &'a str>,
    ) -> Result<bool, String> {
        match flag {
            "--from" => self.from = Some(parse_instant(rest.next(), "--from")?),
            "--to" => self.to = Some(parse_instant(rest.next(), "--to")?),
            "--at" => self.at = Some(parse_instant(rest.next(), "--at")?),
            "--next" => self.next = true,
            "--previous" => self.previous = true,
            "--frame" => self.frame = Some(parse_frame(rest.next())?),
            "--ayanamsa" => {
                let value = rest
                    .next()
                    .ok_or_else(|| "missing value for --ayanamsa".to_string())?;
                self.ayanamsa = Some(parse_ayanamsa(value)?);
            }
            "--star-place" => self.star_place = Some(parse_star_place(rest.next())?),
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn search(&self) -> Result<Search, String> {
        if self.star_place.is_some() && self.ayanamsa.is_none() {
            return Err("--star-place requires --ayanamsa".to_string());
        }
        // Only the apparent geocentric frame reads the anchor star's apparent
        // place; the mean-of-date and heliocentric frames keep the mean ayanamsa.
        if self.star_place == Some(SiderealStarPlace::Apparent)
            && self
                .frame
                .is_some_and(|frame| frame != CrossingFrame::GeocentricApparentOfDate)
        {
            return Err(
                "--star-place apparent requires the apparent geocentric frame (drop --frame mean/helio)"
                    .to_string(),
            );
        }
        let directed = self.next || self.previous;
        if directed && (self.from.is_some() || self.to.is_some()) {
            return Err("--next/--previous cannot be combined with --from/--to".to_string());
        }
        if self.next && self.previous {
            return Err("use only one of --next and --previous".to_string());
        }
        match (directed, self.at, self.from, self.to) {
            (true, Some(at), _, _) => {
                check_in_window(at)?;
                Ok(if self.next {
                    Search::Next(at)
                } else {
                    Search::Previous(at)
                })
            }
            (true, None, _, _) => Err("--next and --previous need --at <instant>".to_string()),
            (false, Some(_), _, _) => Err("--at needs --next or --previous".to_string()),
            (false, None, Some(from), Some(to)) => Ok(Search::Range { from, to }),
            (false, None, Some(_), None) => Err("--from needs --to <instant>".to_string()),
            (false, None, None, Some(_)) => Err("--to needs --from <instant>".to_string()),
            (false, None, None, None) => {
                Err("give a range with --from and --to, or --next/--previous with --at".to_string())
            }
        }
    }

    fn reference(&self) -> CrossingReference {
        let frame = self
            .frame
            .unwrap_or(CrossingFrame::GeocentricApparentOfDate);
        match &self.ayanamsa {
            Some(ayanamsa) => CrossingReference::sidereal(frame, ayanamsa.clone())
                .with_star_place(self.star_place.unwrap_or_default()),
            None => CrossingReference::tropical(frame),
        }
    }
}

fn parse_frame(value: Option<&str>) -> Result<CrossingFrame, String> {
    match value {
        Some("geo") => Ok(CrossingFrame::GeocentricApparentOfDate),
        Some("mean") => Ok(CrossingFrame::GeocentricMeanOfDate),
        Some("helio") => Ok(CrossingFrame::Heliocentric),
        other => Err(format!("--frame must be geo|mean|helio, got {other:?}")),
    }
}

/// A TDB Julian day, or a civil datetime (recognised by its `T`) read as UTC
/// from 1972 on and as UT1 before, the rule the output's civil times follow.
fn parse_instant(value: Option<&str>, flag: &str) -> Result<Instant, String> {
    let raw = value.ok_or_else(|| format!("missing value for {flag}"))?;
    if !raw.contains('T') {
        let jd = parse_f64(Some(raw), flag).map_err(|_| {
            format!("{flag} value '{raw}' must be a Julian day or YYYY-MM-DDTHH:MM:SS")
        })?;
        return Ok(Instant::new(JulianDay::from_days(jd), TimeScale::Tdb));
    }
    let civil = parse_civil(Some(raw), flag)?;
    civil_to_tdb(civil).map_err(|error| format!("{flag} value '{raw}': {error}"))
}

fn civil_to_tdb(civil: CivilDateTime) -> Result<Instant, CivilTimeError> {
    match tdb_from_utc_civil(civil) {
        Err(CivilTimeError::UtcBeforeLeapEpoch) => tdb_from_ut1_civil(civil),
        converted => converted,
    }
    .map(|converted| converted.instant)
}

fn frame_label(frame: CrossingFrame) -> &'static str {
    match frame {
        CrossingFrame::GeocentricApparentOfDate => "geocentric apparent",
        CrossingFrame::GeocentricMeanOfDate => "geocentric mean of date",
        CrossingFrame::Heliocentric => "heliocentric",
        _ => "other",
    }
}

/// One rendered event with the day it sorts by.
struct Line {
    julian_day: f64,
    text: String,
}

/// `<civil time> <scale>  JD <day> TDB  <what>`.
fn event_line(
    instant: Instant,
    civil: Result<CivilConversion, CivilTimeError>,
    what: String,
) -> Result<Line, String> {
    let converted = civil.map_err(|error| error.to_string())?;
    let c = converted.civil;
    let julian_day = instant.julian_day.days();
    Ok(Line {
        julian_day,
        text: format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:06.3} {}  JD {:.5} TDB  {}",
            c.year, c.month, c.day, c.hour, c.minute, c.second, converted.scale, julian_day, what
        ),
    })
}

fn station_line(station: &Station) -> Result<Line, String> {
    let turns = match station.kind {
        StationKind::TurnsRetrograde => "turns retrograde",
        StationKind::TurnsDirect => "turns direct",
        _ => "stations",
    };
    event_line(
        station.instant,
        station.civil(),
        format!(
            "{} {} at {:.4}°",
            station.body,
            turns,
            station.longitude.degrees()
        ),
    )
}

fn aspect_line(event: &AspectEvent) -> Result<Line, String> {
    event_line(
        event.instant,
        event.civil(),
        format!(
            "{}–{} {}° ({:.4}° / {:.4}°)",
            event.first,
            event.second,
            event.angle.degrees(),
            event.first_longitude.degrees(),
            event.second_longitude.degrees()
        ),
    )
}

/// The header line, then the events in time order, then a note for each search
/// the window cut short; a line saying there are none when there is neither.
fn render(
    title: &str,
    shared: &SharedArgs,
    mut lines: Vec<Line>,
    notes: &[String],
    none: &str,
) -> String {
    let reference = shared.reference();
    let star_place = if shared.star_place == Some(SiderealStarPlace::Apparent)
        && reference.frame == CrossingFrame::GeocentricApparentOfDate
    {
        ", apparent star place"
    } else {
        ""
    };
    let mut out = format!(
        "{title} ({}; {} zodiac{star_place})\n",
        frame_label(reference.frame),
        reference.zodiac
    );
    if lines.is_empty() && notes.is_empty() {
        out.push_str(none);
        return out;
    }
    lines.sort_by(|a, b| a.julian_day.total_cmp(&b.julian_day));
    let texts: Vec<&str> = lines
        .iter()
        .map(|line| line.text.as_str())
        .chain(notes.iter().map(String::as_str))
        .collect();
    out.push_str(&texts.join("\n"));
    out
}

fn event_error(error: EventError) -> String {
    error.to_string()
}

/// A `--next`/`--previous` search starts inside the 1900–2100 window; outside
/// it the search is not cut short, it never had anywhere to start.
fn check_in_window(at: Instant) -> Result<(), String> {
    let jd = at.julian_day.days();
    if (WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
        Ok(())
    } else {
        Err(format!(
            "--at JD {jd} is outside the search window (JD {WINDOW_START_JD}..={WINDOW_END_JD}, 1900-2100)"
        ))
    }
}

/// Which way a `--next`/`--previous` search ran.
#[derive(Clone, Copy)]
enum Direction {
    Next,
    Previous,
}

/// The note for a `--next`/`--previous` search the window cut short.
fn cut_short_note(label: &str, direction: Direction) -> String {
    match direction {
        Direction::Previous => format!("{label}: none after the window's start (1900-01-01)"),
        Direction::Next => format!("{label}: none before the window's end (2100-01-01)"),
    }
}

/// Whether `error` says the search ran past the window's limit in its own
/// direction. Any other `OutOfWindow` (a read at the search's own start, say)
/// is a failure, not an exhausted search.
fn cut_short(error: &EventError, direction: Direction) -> bool {
    match (error, direction) {
        (EventError::OutOfWindow { julian_day }, Direction::Next) => *julian_day > WINDOW_END_JD,
        (EventError::OutOfWindow { julian_day }, Direction::Previous) => {
            *julian_day <= WINDOW_START_JD
        }
        _ => false,
    }
}

/// Adds a `--next`/`--previous` result to `found`, or its note to `notes`
/// when the window ended first; any other error fails the command.
fn found_or_note<T>(
    result: Result<Option<T>, EventError>,
    direction: Direction,
    found: &mut Vec<T>,
    notes: &mut Vec<String>,
    label: &str,
) -> Result<(), String> {
    match result {
        Ok(event) => {
            found.extend(event);
            Ok(())
        }
        Err(error) if cut_short(&error, direction) => {
            notes.push(cut_short_note(label, direction));
            Ok(())
        }
        Err(error) => Err(event_error(error)),
    }
}

pub(crate) fn render_stations(args: &[&str]) -> Result<String, String> {
    let mut shared = SharedArgs::default();
    let mut bodies: Vec<CelestialBody> = Vec::new();
    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--help" | "-h" => {
                return Ok(format!(
                    "{}\n\n{STATIONS_USAGE}\n{SHARED_HELP}",
                    crate::cli::banner()
                ))
            }
            "--body" => bodies.push(parse_body(iter.next())?),
            other => {
                if !shared.take(other, &mut iter)? {
                    return Err(format!("unknown argument: {other}"));
                }
            }
        }
    }
    if bodies.is_empty() {
        return Err("stations needs at least one --body <name>".to_string());
    }
    let search = shared.search()?;
    let reference = shared.reference();
    let engine = EventEngine::new(default_chart_backend());
    let mut found: Vec<Station> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    for body in bodies {
        let label = body.to_string();
        match &search {
            Search::Range { from, to } => found.extend(
                engine
                    .stations_in_range(body, reference.clone(), *from, *to)
                    .map_err(event_error)?,
            ),
            Search::Next(at) => found_or_note(
                engine.next_station(body, reference.clone(), *at),
                Direction::Next,
                &mut found,
                &mut notes,
                &label,
            )?,
            Search::Previous(at) => found_or_note(
                engine.previous_station(body, reference.clone(), *at),
                Direction::Previous,
                &mut found,
                &mut notes,
                &label,
            )?,
        }
    }
    let lines = found
        .iter()
        .map(station_line)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(render(
        "Stations",
        &shared,
        lines,
        &notes,
        "no stations found",
    ))
}

/// `<first>,<second>` as two bodies.
fn parse_pair(value: Option<&str>) -> Result<(CelestialBody, CelestialBody), String> {
    let raw = value.ok_or_else(|| "missing value for --pair".to_string())?;
    let (first, second) = raw
        .split_once(',')
        .ok_or_else(|| format!("--pair value '{raw}' must be <first>,<second>"))?;
    Ok((parse_body(Some(first))?, parse_body(Some(second))?))
}

pub(crate) fn render_aspects(args: &[&str]) -> Result<String, String> {
    let mut shared = SharedArgs::default();
    let mut pairs: Vec<(CelestialBody, CelestialBody)> = Vec::new();
    let mut angles: Vec<Angle> = Vec::new();
    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--help" | "-h" => {
                return Ok(format!(
                    "{}\n\n{ASPECTS_USAGE}\n{SHARED_HELP}",
                    crate::cli::banner()
                ))
            }
            "--pair" => pairs.push(parse_pair(iter.next())?),
            "--angle" => angles.push(Angle::from_degrees(parse_f64(iter.next(), "--angle")?)),
            other => {
                if !shared.take(other, &mut iter)? {
                    return Err(format!("unknown argument: {other}"));
                }
            }
        }
    }
    if pairs.is_empty() {
        return Err("aspects needs at least one --pair <first>,<second>".to_string());
    }
    if angles.is_empty() {
        return Err("aspects needs at least one --angle <degrees>".to_string());
    }
    let search = shared.search()?;
    let reference = shared.reference();
    let engine = EventEngine::new(default_chart_backend());
    let mut found: Vec<AspectEvent> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    for (first, second) in &pairs {
        for &angle in &angles {
            let label = format!("{first}–{second} {}°", angle.degrees());
            let (first, second, reference) = (first.clone(), second.clone(), reference.clone());
            match &search {
                Search::Range { from, to } => found.extend(
                    engine
                        .aspects_in_range(first, second, angle, reference, *from, *to)
                        .map_err(event_error)?,
                ),
                Search::Next(at) => found_or_note(
                    engine.next_aspect(first, second, angle, reference, *at),
                    Direction::Next,
                    &mut found,
                    &mut notes,
                    &label,
                )?,
                Search::Previous(at) => found_or_note(
                    engine.previous_aspect(first, second, angle, reference, *at),
                    Direction::Previous,
                    &mut found,
                    &mut notes,
                    &label,
                )?,
            }
        }
    }
    let lines = found
        .iter()
        .map(aspect_line)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(render(
        "Aspects",
        &shared,
        lines,
        &notes,
        "no aspects found",
    ))
}
