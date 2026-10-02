//! Fail-closed gate: `EventEngine::stations_in_range` on the packaged backend
//! vs the committed Swiss Ephemeris speed-zero reference corpus (issue #85).
//!
//! Swiss Ephemeris has no station finder; the corpus holds the sign changes
//! of its own longitude speed (`tools/se-stations-reference`). Planets must
//! match the corpus station for station. The true node is compared only on
//! separated stations: its speed touches zero about every two weeks, and
//! whether a touch crosses zero for a few hours depends on the ephemeris.
//! See `stations_thresholds` for the basis of the ceilings.

// Consumed by the gate entry point added next.
#![allow(dead_code)]

use crate::stations_thresholds::{Ceilings, SEPARATION_DAYS};
use pleiades_events::StationKind;
use pleiades_types::CelestialBody;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// The frame and zodiac a corpus group was generated in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Group {
    /// Geocentric apparent, tropical.
    Geo,
    /// Geocentric mean of date, tropical.
    Mean,
    /// Geocentric apparent, sidereal Lahiri.
    Sid,
}

impl Group {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "geo" => Self::Geo,
            "mean" => Self::Mean,
            "sid" => Self::Sid,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Geo => "geo",
            Self::Mean => "mean",
            Self::Sid => "sid",
        }
    }
}

/// One station, from either side of the comparison.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Found {
    jd: f64,
    lon_deg: f64,
    kind: StationKind,
}

/// The corpus stations of one body in one group, ascending.
#[derive(Clone, Debug)]
struct Series {
    group: Group,
    body: CelestialBody,
    body_name: &'static str,
    stations: Vec<Found>,
}

#[derive(Debug)]
pub enum StationsError {
    MalformedRow(String),
    MalformedManifest(String),
    ChecksumMismatch {
        got: u64,
        want: u64,
    },
    ManifestDrift {
        rows_csv: usize,
        rows_manifest: usize,
    },
    TooFewRowsValidated {
        validated: usize,
        floor: usize,
    },
    CalculationFailed {
        series: String,
        reason: String,
    },
    /// The engine and the corpus disagree on how many stations a planet has.
    CountMismatch {
        series: String,
        got: usize,
        want: usize,
    },
    /// The engine and the corpus disagree on which way a planet turns.
    KindMismatch {
        series: String,
        index: usize,
        jd_tt: f64,
    },
    CeilingExceeded {
        series: String,
        jd_tt: f64,
        kind: &'static str,
        residual: f64,
        ceiling: f64,
    },
    /// A separated true-node station with no counterpart of the same kind
    /// within the time ceiling. `side` names the list the station is in.
    Unmatched {
        series: String,
        side: &'static str,
        jd_tt: f64,
    },
}

impl std::fmt::Display for StationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got} want {want}")
            }
            Self::ManifestDrift {
                rows_csv,
                rows_manifest,
            } => write!(
                f,
                "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}"
            ),
            Self::TooFewRowsValidated { validated, floor } => {
                write!(f, "only {validated} stations validated, floor is {floor}")
            }
            Self::CalculationFailed { series, reason } => {
                write!(f, "{series} station search failed: {reason}")
            }
            Self::CountMismatch { series, got, want } => write!(
                f,
                "{series}: engine found {got} stations, corpus has {want}"
            ),
            Self::KindMismatch {
                series,
                index,
                jd_tt,
            } => write!(
                f,
                "{series}: station {index} near jd_tt={jd_tt} turns the other way in the corpus"
            ),
            Self::CeilingExceeded {
                series,
                jd_tt,
                kind,
                residual,
                ceiling,
            } => write!(
                f,
                "{series} {kind} ceiling exceeded at jd_tt={jd_tt}: residual {residual:.6e} > ceiling {ceiling:.6e}"
            ),
            Self::Unmatched {
                series,
                side,
                jd_tt,
            } => write!(
                f,
                "{series}: separated {side} station at jd_tt={jd_tt} has no counterpart of the same kind within the time ceiling"
            ),
        }
    }
}

impl std::error::Error for StationsError {}

fn body_from_name(name: &str) -> Option<(CelestialBody, &'static str)> {
    Some(match name {
        "Mercury" => (CelestialBody::Mercury, "Mercury"),
        "Venus" => (CelestialBody::Venus, "Venus"),
        "Mars" => (CelestialBody::Mars, "Mars"),
        "Jupiter" => (CelestialBody::Jupiter, "Jupiter"),
        "Saturn" => (CelestialBody::Saturn, "Saturn"),
        "Uranus" => (CelestialBody::Uranus, "Uranus"),
        "Neptune" => (CelestialBody::Neptune, "Neptune"),
        "Pluto" => (CelestialBody::Pluto, "Pluto"),
        "TrueNode" => (CelestialBody::TrueNode, "TrueNode"),
        _ => return None,
    })
}

fn parse_corpus(csv: &str) -> Result<Vec<Series>, StationsError> {
    let malformed = |what: String| StationsError::MalformedRow(what);
    let mut all: Vec<Series> = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 5 {
            return Err(malformed(format!(
                "expected 5 fields, got {} in {line}",
                fields.len()
            )));
        }
        let group = Group::from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown group {} in {line}", fields[0])))?;
        let (body, body_name) = body_from_name(fields[1])
            .ok_or_else(|| malformed(format!("unknown body {} in {line}", fields[1])))?;
        let num = |i: usize| -> Result<f64, StationsError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let kind = match fields[4] {
            "R" => StationKind::TurnsRetrograde,
            "D" => StationKind::TurnsDirect,
            other => return Err(malformed(format!("unknown kind {other} in {line}"))),
        };
        let station = Found {
            jd: num(2)?,
            lon_deg: num(3)?,
            kind,
        };
        match all
            .iter_mut()
            .find(|s| s.group == group && s.body_name == body_name)
        {
            Some(series) => {
                if series
                    .stations
                    .last()
                    .is_some_and(|last| last.jd >= station.jd)
                {
                    return Err(malformed(format!("rows are not ascending at {line}")));
                }
                series.stations.push(station);
            }
            None => all.push(Series {
                group,
                body,
                body_name,
                stations: vec![station],
            }),
        }
    }
    Ok(all)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), StationsError> {
    let malformed = |what: String| StationsError::MalformedManifest(what);
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| malformed("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| malformed(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| malformed(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| malformed("rows= missing".into()))?,
        checksum.ok_or_else(|| malformed("checksum= missing".into()))?,
    ))
}

/// What one series' comparison measured.
#[derive(Clone, Copy, Debug, Default)]
struct Residuals {
    /// Stations compared.
    matched: usize,
    max_time_s: f64,
    max_lon_arcsec: f64,
    /// Sum of engine − corpus time over the compared stations; a mean far
    /// from zero means the two speeds differ by convention, not by noise.
    sum_signed_time_s: f64,
}

fn lon_residual_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    ((got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0).abs() * 3600.0
}

/// Checks one engine/corpus pair against both ceilings and records it. A NaN
/// residual fails closed.
fn record(
    label: &str,
    engine: &Found,
    corpus: &Found,
    ceilings: Ceilings,
    residuals: &mut Residuals,
) -> Result<(), StationsError> {
    let signed_time_s = (engine.jd - corpus.jd) * SECONDS_PER_DAY;
    let checks = [
        ("time_seconds", signed_time_s.abs(), ceilings.time_s),
        (
            "longitude_arcsec",
            lon_residual_arcsec(engine.lon_deg, corpus.lon_deg),
            ceilings.lon_arcsec,
        ),
    ];
    for (kind, residual, ceiling) in checks {
        if residual.is_nan() || residual > ceiling {
            return Err(StationsError::CeilingExceeded {
                series: label.to_string(),
                jd_tt: corpus.jd,
                kind,
                residual,
                ceiling,
            });
        }
    }
    residuals.matched += 1;
    residuals.max_time_s = residuals.max_time_s.max(checks[0].1);
    residuals.max_lon_arcsec = residuals.max_lon_arcsec.max(checks[1].1);
    residuals.sum_signed_time_s += signed_time_s;
    Ok(())
}

/// Planets: the two lists must agree station for station.
fn compare_exact(
    label: &str,
    engine: &[Found],
    corpus: &[Found],
    ceilings: Ceilings,
) -> Result<Residuals, StationsError> {
    if engine.len() != corpus.len() {
        return Err(StationsError::CountMismatch {
            series: label.to_string(),
            got: engine.len(),
            want: corpus.len(),
        });
    }
    let mut residuals = Residuals::default();
    for (index, (got, want)) in engine.iter().zip(corpus).enumerate() {
        if got.kind != want.kind {
            return Err(StationsError::KindMismatch {
                series: label.to_string(),
                index,
                jd_tt: want.jd,
            });
        }
        record(label, got, want, ceilings, &mut residuals)?;
    }
    Ok(residuals)
}

/// Whether `list[index]`'s nearest neighbour is at least
/// [`SEPARATION_DAYS`] away.
fn is_separated(list: &[Found], index: usize) -> bool {
    let here = list[index].jd;
    let far = |other: Option<&Found>| {
        other.is_none_or(|other| (other.jd - here).abs() >= SEPARATION_DAYS)
    };
    far(index.checked_sub(1).and_then(|i| list.get(i))) && far(list.get(index + 1))
}

/// The station of `kind` in `list` nearest in time to `jd`, if it is within
/// `time_s` seconds.
fn nearest_of_kind(list: &[Found], jd: f64, kind: StationKind, time_s: f64) -> Option<&Found> {
    list.iter()
        .filter(|candidate| candidate.kind == kind)
        .min_by(|a, b| (a.jd - jd).abs().total_cmp(&(b.jd - jd).abs()))
        .filter(|nearest| (nearest.jd - jd).abs() * SECONDS_PER_DAY <= time_s)
}

/// The true node: every separated station on either side must have a
/// counterpart of the same kind within the time ceiling; stations in closer
/// pairs are unconstrained.
fn compare_separated(
    label: &str,
    engine: &[Found],
    corpus: &[Found],
    ceilings: Ceilings,
) -> Result<Residuals, StationsError> {
    let unmatched = |side: &'static str, jd_tt: f64| StationsError::Unmatched {
        series: label.to_string(),
        side,
        jd_tt,
    };
    let mut residuals = Residuals::default();
    for (index, want) in corpus.iter().enumerate() {
        if !is_separated(corpus, index) {
            continue;
        }
        let got = nearest_of_kind(engine, want.jd, want.kind, ceilings.time_s)
            .ok_or_else(|| unmatched("corpus", want.jd))?;
        record(label, got, want, ceilings, &mut residuals)?;
    }
    for (index, got) in engine.iter().enumerate() {
        if is_separated(engine, index)
            && nearest_of_kind(corpus, got.jd, got.kind, ceilings.time_s).is_none()
        {
            return Err(unmatched("engine", got.jd));
        }
    }
    Ok(residuals)
}

#[cfg(test)]
mod tests;
