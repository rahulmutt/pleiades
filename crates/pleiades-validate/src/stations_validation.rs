//! Fail-closed gate: `EventEngine::stations_in_range` on the packaged backend
//! vs the committed Swiss Ephemeris speed-zero reference corpus (issue #85).
//!
//! Swiss Ephemeris has no station finder; the corpus holds the sign changes
//! of its own longitude speed (`tools/se-stations-reference`). Planets must
//! match the corpus station for station. The true node is compared only on
//! separated stations: its speed touches zero about every two weeks, and
//! whether a touch crosses zero for a few hours depends on the ephemeris.
//! See `stations_thresholds` for the basis of the ceilings.
//!
//! Asteroids (issue #167 (d)): Ceres, Pallas, Juno and Vesta are compared
//! station for station against `asteroids.csv`, generated with SWIEPH and
//! seas_18 by the same tool's `--asteroids` mode, in the full gate only.

use crate::stations_thresholds::{
    ceilings_for, Ceilings, MIN_ROWS_VALIDATED, MIN_ROWS_VALIDATED_ASTEROIDS,
    MIN_ROWS_VALIDATED_MEAN_SID_SUBSET, SEPARATION_DAYS, TRUE_NODE_CLOSE_DAYS,
    TRUE_NODE_MIN_CLOSE_PERCENT,
};
use pleiades_apparent::fnv1a64;
use pleiades_data::{packaged_backend, PackagedDataBackend};
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine, Station, StationKind};
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale};
use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/stations.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/manifest.txt"
));
const ASTEROID_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/asteroids.csv"
));
const ASTEROID_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/asteroids-manifest.txt"
));

/// The spans `tools/se-stations-reference` scanned (Julian days, TT). The
/// full span is the engine's window less five days at each end, so neither
/// side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

/// Days per scan chunk, about ten years. A series is scanned in chunks on a
/// shared pool of threads (FU-23 (e)), so the gate's wall-clock is its CPU
/// total spread over the cores rather than its longest series. The length is
/// a multiple of 2 days, the least common multiple of the engine's sampling
/// steps (0.25, 1 and 2 days), so every chunk boundary is one of the single
/// scan's own samples: a chunk evaluates exactly the brackets the single scan
/// evaluates between its bounds, and `join_chunks` reproduces the single
/// scan's station list to the bit.
const CHUNK_DAYS: f64 = 3652.0;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// Which corpus series a run compares. The checksum and the manifest row
/// count are always verified against the whole corpus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scope {
    /// Every series: `validate-stations` (`mise run gate-stations`) and the
    /// opt-in `PLEIADES_FULL_STATIONS_GATE=1` gate test.
    Full,
    /// The `mean` and `sid` series only, for the release battery
    /// (`run_all_numeric_gates`), where the full 1900–2100 scan is too slow.
    MeanSidSubset,
    /// The asteroid corpus (`asteroids.csv`, Swiss Ephemeris SWIEPH with
    /// seas_18): Ceres, Pallas, Juno, Vesta over 1900–2100. Part of the full
    /// gate only (issue #167 (d)).
    Asteroids,
}

impl Scope {
    fn includes(self, group: Group) -> bool {
        match self {
            Self::Full => true,
            Self::MeanSidSubset => group != Group::Geo,
            Self::Asteroids => true,
        }
    }

    fn floor(self) -> usize {
        match self {
            Self::Full => MIN_ROWS_VALIDATED,
            Self::MeanSidSubset => MIN_ROWS_VALIDATED_MEAN_SID_SUBSET,
            Self::Asteroids => MIN_ROWS_VALIDATED_ASTEROIDS,
        }
    }

    /// How the summary line names the run.
    fn title(self) -> &'static str {
        match self {
            Self::Full => "Stations gate",
            Self::MeanSidSubset => "Stations gate (mean/sid subset)",
            Self::Asteroids => "Asteroid stations",
        }
    }
}

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

    fn reference(self) -> CrossingReference {
        match self {
            Self::Geo => CrossingFrame::GeocentricApparentOfDate.into(),
            Self::Mean => CrossingFrame::GeocentricMeanOfDate.into(),
            Self::Sid => CrossingReference::sidereal(
                CrossingFrame::GeocentricApparentOfDate,
                Ayanamsa::Lahiri,
            ),
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

impl Found {
    fn from_station(station: &Station) -> Self {
        Self {
            jd: station.instant.julian_day.days(),
            lon_deg: station.longitude.degrees(),
            kind: station.kind,
        }
    }
}

/// The corpus stations of one body in one group, ascending.
#[derive(Clone, Debug)]
struct Series {
    group: Group,
    body: CelestialBody,
    body_name: &'static str,
    stations: Vec<Found>,
}

impl Series {
    /// How the report and the errors name the series, e.g. `geo Mercury`.
    fn label(&self) -> String {
        format!("{} {}", self.group.name(), self.body_name)
    }
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
    /// Too few of the compared true-node stations are close to their
    /// counterpart: the series as a whole has moved.
    TooFewClose {
        series: String,
        close: usize,
        compared: usize,
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
            Self::TooFewClose {
                series,
                close,
                compared,
            } => write!(
                f,
                "{series}: only {close} of {compared} compared stations are within {TRUE_NODE_CLOSE_DAYS} d of their counterpart, floor is {TRUE_NODE_MIN_CLOSE_PERCENT} %"
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
        "Ceres" => (CelestialBody::Ceres, "Ceres"),
        "Pallas" => (CelestialBody::Pallas, "Pallas"),
        "Juno" => (CelestialBody::Juno, "Juno"),
        "Vesta" => (CelestialBody::Vesta, "Vesta"),
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
    crate::corpus_manifest::slice_entry(manifest)
        .map_err(|e| StationsError::MalformedManifest(e.to_string()))
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
    /// True node only: compared stations within [`TRUE_NODE_CLOSE_DAYS`] of
    /// their counterpart.
    close: usize,
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
/// pairs are unconstrained. That ceiling is days wide, so
/// [`TRUE_NODE_MIN_CLOSE_PERCENT`] of the compared stations must also be
/// within [`TRUE_NODE_CLOSE_DAYS`] of their counterpart.
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
        if (got.jd - want.jd).abs() <= TRUE_NODE_CLOSE_DAYS {
            residuals.close += 1;
        }
    }
    if residuals.close * 100 < residuals.matched * TRUE_NODE_MIN_CLOSE_PERCENT {
        return Err(StationsError::TooFewClose {
            series: label.to_string(),
            close: residuals.close,
            compared: residuals.matched,
        });
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

#[derive(Debug)]
pub struct StationsReport {
    /// Stations compared against the corpus.
    pub rows_validated: usize,
    series_lines: Vec<String>,
    summary_line: String,
}

impl StationsReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }

    /// One line per corpus series with its measured maxima; the basis for
    /// the ceilings in `stations_thresholds`.
    pub fn series_lines(&self) -> &[String] {
        &self.series_lines
    }

    /// The planet report followed by the asteroid report: rows summed, series
    /// lines in order, summary lines joined with "; ".
    fn merged_with(self, other: StationsReport) -> StationsReport {
        let mut series_lines = self.series_lines;
        series_lines.extend(other.series_lines);
        StationsReport {
            rows_validated: self.rows_validated + other.rows_validated,
            series_lines,
            summary_line: format!("{}; {}", self.summary_line, other.summary_line),
        }
    }
}

fn span(series: &Series) -> (f64, f64) {
    if series.group == Group::Geo && series.body != CelestialBody::TrueNode {
        FULL_SPAN
    } else {
        SHORT_SPAN
    }
}

fn validate(csv: &str, manifest: &str) -> Result<StationsReport, StationsError> {
    validate_scoped(csv, manifest, Scope::Full)
}

/// One series' comparison, folded into the report by `validate_scoped`.
struct SeriesOutcome {
    residuals: Residuals,
    /// The report line for the series.
    line: String,
}

/// The corpus epoch is TT; the engine reads the Julian day as TDB. The two
/// differ by under 2 ms, far below every ceiling here.
fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

/// `[start, end]` as contiguous windows of `chunk_days` from `start`, the
/// last one ending at `end`.
fn chunk_bounds(start: f64, end: f64, chunk_days: f64) -> Vec<(f64, f64)> {
    debug_assert!(chunk_days > 0.0, "chunk length must be positive");
    let mut bounds = Vec::new();
    let mut lo = start;
    let mut k = 0u32;
    loop {
        k += 1;
        let hi = start + f64::from(k) * chunk_days;
        if hi >= end {
            bounds.push((lo, end));
            return bounds;
        }
        bounds.push((lo, hi));
        lo = hi;
    }
}

/// One series' engine stations from its chunk scans, in time order. The
/// engine scans one step past a window's end and keeps a root on either
/// closed end, so neighbouring chunks share exactly one bracket, the one
/// starting at their boundary, and a station exactly on the boundary is the
/// last station of one chunk and the first of the next: it is kept once.
/// Every other bracket belongs to one chunk, so nothing else can repeat.
fn join_chunks(chunks: Vec<Vec<Found>>) -> Vec<Found> {
    let mut joined: Vec<Found> = Vec::new();
    let mut previous_last: Option<f64> = None;
    for chunk in chunks {
        let seam_duplicate =
            previous_last.is_some_and(|last| chunk.first().is_some_and(|first| first.jd == last));
        previous_last = chunk.last().map(|found| found.jd);
        joined.extend(chunk.into_iter().skip(usize::from(seam_duplicate)));
    }
    joined
}

/// Scans `[start, end]` of one series with the engine.
fn scan_chunk(
    engine: &EventEngine<PackagedDataBackend>,
    series: &Series,
    start: f64,
    end: f64,
) -> Result<Vec<Found>, StationsError> {
    engine
        .stations_in_range(
            series.body.clone(),
            series.group.reference(),
            tdb(start),
            tdb(end),
        )
        .map(|stations| stations.iter().map(Found::from_station).collect())
        .map_err(|e| StationsError::CalculationFailed {
            series: series.label(),
            reason: e.to_string(),
        })
}

/// Every selected series' engine stations, in `selected` order. Each series
/// is scanned in `chunk_days` windows (`chunk_bounds`), and the windows of
/// all the series share one pool of `available_parallelism` threads, so the
/// wall-clock is the CPU total spread over the cores rather than the longest
/// series. A series' error is its first failing window in time order.
fn scan_series_chunked(
    engine: &EventEngine<PackagedDataBackend>,
    selected: &[&Series],
    chunk_days: f64,
) -> Vec<Result<Vec<Found>, StationsError>> {
    struct Job<'a> {
        series: &'a Series,
        start: f64,
        end: f64,
    }
    let mut jobs: Vec<Job> = Vec::new();
    let mut chunks_per_series = Vec::with_capacity(selected.len());
    for &series in selected {
        let (start, end) = span(series);
        let bounds = chunk_bounds(start, end, chunk_days);
        chunks_per_series.push(bounds.len());
        jobs.extend(
            bounds
                .into_iter()
                .map(|(start, end)| Job { series, start, end }),
        );
    }
    // Filled in by index below; a slot left as it is would mean a window the
    // pool never scanned, which fails closed rather than passing silently.
    let mut results: Vec<Result<Vec<Found>, StationsError>> = jobs
        .iter()
        .map(|job| {
            Err(StationsError::CalculationFailed {
                series: job.series.label(),
                reason: format!("window {}..{} was not scanned", job.start, job.end),
            })
        })
        .collect();
    let next = AtomicUsize::new(0);
    let workers = std::thread::available_parallelism()
        .map_or(1, NonZero::get)
        .clamp(1, jobs.len().max(1));
    std::thread::scope(|threads| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                threads.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(job) = jobs.get(index) else { break };
                        done.push((index, scan_chunk(engine, job.series, job.start, job.end)));
                    }
                    done
                })
            })
            .collect();
        for handle in handles {
            match handle.join() {
                Ok(done) => {
                    for (index, result) in done {
                        results[index] = result;
                    }
                }
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
    });
    let mut results = results.into_iter();
    chunks_per_series
        .into_iter()
        .map(|count| {
            results
                .by_ref()
                .take(count)
                .collect::<Result<Vec<Vec<Found>>, StationsError>>()
                .map(join_chunks)
        })
        .collect()
}

/// Compares one series' engine stations with the corpus.
fn compare_series(
    series: &Series,
    ceilings: Ceilings,
    found: &[Found],
) -> Result<SeriesOutcome, StationsError> {
    let label = series.label();
    let residuals = if series.body == CelestialBody::TrueNode {
        compare_separated(&label, found, &series.stations, ceilings)?
    } else {
        compare_exact(&label, found, &series.stations, ceilings)?
    };
    let mean_signed_s = if residuals.matched == 0 {
        0.0
    } else {
        residuals.sum_signed_time_s / residuals.matched as f64
    };
    let mut line = format!(
        "{label}: {} compared (engine {}, corpus {}), max time {:.1} s, mean signed time {:+.1} s, max lon {:.3}\"",
        residuals.matched,
        found.len(),
        series.stations.len(),
        residuals.max_time_s,
        mean_signed_s,
        residuals.max_lon_arcsec,
    );
    if series.body == CelestialBody::TrueNode {
        line.push_str(&format!(
            ", {} within {TRUE_NODE_CLOSE_DAYS} d",
            residuals.close
        ));
    }
    Ok(SeriesOutcome { residuals, line })
}

fn validate_scoped(
    csv: &str,
    manifest: &str,
    scope: Scope,
) -> Result<StationsReport, StationsError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(StationsError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let all = parse_corpus(csv)?;
    let rows_csv: usize = all.iter().map(|series| series.stations.len()).sum();
    if rows_csv != manifest_rows {
        return Err(StationsError::ManifestDrift {
            rows_csv,
            rows_manifest: manifest_rows,
        });
    }

    let selected: Vec<&Series> = all
        .iter()
        .filter(|series| scope.includes(series.group))
        .collect();
    // Every body needs ceilings before any scanning starts.
    let ceilings = selected
        .iter()
        .map(|series| {
            ceilings_for(series.body_name).ok_or_else(|| StationsError::CalculationFailed {
                series: series.label(),
                reason: "no ceilings for this body".into(),
            })
        })
        .collect::<Result<Vec<Ceilings>, StationsError>>()?;
    // The engine only borrows the backend, so the series' windows scan on a
    // pool of threads (FU-23 (e)). The scans are folded in corpus order, so
    // the report lines and the first error are the same as from a sequential
    // loop over the series.
    let engine = EventEngine::new(packaged_backend());
    let scans = scan_series_chunked(&engine, &selected, CHUNK_DAYS);
    let mut validated = 0usize;
    let mut series_lines = Vec::with_capacity(scans.len());
    let (mut max_time_s, mut max_lon_arcsec) = (0.0_f64, 0.0_f64);
    for ((series, ceilings), scan) in selected.iter().zip(ceilings).zip(scans) {
        let SeriesOutcome { residuals, line } = compare_series(series, ceilings, &scan?)?;
        validated += residuals.matched;
        max_time_s = max_time_s.max(residuals.max_time_s);
        max_lon_arcsec = max_lon_arcsec.max(residuals.max_lon_arcsec);
        series_lines.push(line);
    }
    let floor = scope.floor().max(1);
    if validated < floor {
        return Err(StationsError::TooFewRowsValidated { validated, floor });
    }
    let summary_line = if scope == Scope::Asteroids {
        format!(
            "{}: {validated} stations validated across {} series vs Swiss Ephemeris SWIEPH (seas_18) \
             speed-zero corpus (station-for-station), max time {max_time_s:.1} s, max lon {max_lon_arcsec:.3}\"",
            scope.title(),
            series_lines.len(),
        )
    } else {
        format!(
            "{}: {validated} stations validated across {} series vs Swiss Ephemeris speed-zero corpus \
             (planets station-for-station; true node on stations separated by >= {SEPARATION_DAYS} d, \
             {TRUE_NODE_MIN_CLOSE_PERCENT} % of them within {TRUE_NODE_CLOSE_DAYS} d), \
             max time {max_time_s:.1} s, max lon {max_lon_arcsec:.3}\"",
            scope.title(),
            series_lines.len(),
        )
    };
    Ok(StationsReport {
        rows_validated: validated,
        series_lines,
        summary_line,
    })
}

/// The full gate: every corpus series (planets over 1900–2100 and the
/// 1990–2030 series), floor `MIN_ROWS_VALIDATED` (5542 stations). The
/// series scan in ten-year windows on a pool of one thread per core, so the
/// wall-clock is about 343 s of CPU spread over the cores: about 105 s on
/// the 4-core nightly runner (2026-10-04; 178 s with one thread per series,
/// where the longest series set the length). Run by `validate-stations` as
/// `mise run gate-stations` (its own nightly job and a `release-gate`
/// dependency) and by the opt-in `PLEIADES_FULL_STATIONS_GATE=1` test.
/// It then runs the four asteroid series (`Scope::Asteroids`, 1225
/// stations, floor `MIN_ROWS_VALIDATED_ASTEROIDS`) after the planet pool;
/// their cost is not yet measured on the nightly runner.
pub fn validate_stations_corpus() -> Result<StationsReport, StationsError> {
    let planets = validate(CORPUS_CSV, MANIFEST)?;
    let asteroids = validate_scoped(ASTEROID_CSV, ASTEROID_MANIFEST, Scope::Asteroids)?;
    Ok(planets.merged_with(asteroids))
}

/// The release-battery subset: verifies the checksum and row count of the
/// whole corpus, then compares only the `mean` and `sid` series (Mercury,
/// Mars, Saturn over 1990–2030), floor `MIN_ROWS_VALIDATED_MEAN_SID_SUBSET`
/// (734 stations). Fail-closed like the full gate, in about 24 s in the dev
/// profile (2026-10-02).
pub fn validate_stations_corpus_subset() -> Result<StationsReport, StationsError> {
    validate_scoped(CORPUS_CSV, MANIFEST, Scope::MeanSidSubset)
}

#[cfg(test)]
mod tests;
