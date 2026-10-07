//! Fail-closed gate: `EventEngine::aspects_in_range` on the packaged backend
//! vs the committed Swiss Ephemeris exact-aspect reference corpus (issue #84).
//!
//! Swiss Ephemeris has no aspect finder; the corpus holds the instants at
//! which the difference of its own longitudes equals 0, 60, 90, 120 or 180
//! degrees (`tools/se-aspects-reference`). The engine must match it event
//! for event, including the pair-and-angle series in which neither side has
//! an event. The reference tool refuses to write a corpus in which a
//! separation turns within 30 arcseconds of an angle, so no corpus event
//! depends on the ephemeris. See `aspects_thresholds` for the ceilings.
//!
//! Tiers: the full gate (about 15 minutes as `mise run gate-aspects`, 880 s;
//! 18.5 minutes as the in-crate test, 1110.8 s; dev/test profile, 2026-10-02)
//! runs in its own nightly job and in `release-gate`; the `mean` subset
//! (about 16 s) runs in the release battery (`release-smoke`).

use crate::aspects_thresholds::{
    ceilings_for, Ceilings, MIN_ROWS_VALIDATED, MIN_ROWS_VALIDATED_MEAN_SUBSET,
};
use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine};
use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/aspects.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/manifest.txt"
));

/// The spans `tools/se-aspects-reference` scanned (Julian days, TT). The
/// full span is the engine's window less five days at each end, so neither
/// side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

/// The angles the reference tool scanned for every pair.
const ANGLES_DEG: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

const SECONDS_PER_DAY: f64 = 86_400.0;
const ARCSEC_PER_DEG: f64 = 3600.0;

/// Which corpus groups a run compares. The checksum and the manifest row
/// count are always verified against the whole corpus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scope {
    /// Every pair: `validate-aspects` and the opt-in gate test.
    Full,
    /// The `mean` group only, for the release battery
    /// (`run_all_numeric_gates`), where the full 1900–2100 scans are too slow.
    MeanSubset,
}

impl Scope {
    fn includes(self, group: Group) -> bool {
        match self {
            Self::Full => true,
            Self::MeanSubset => group == Group::Mean,
        }
    }

    fn floor(self) -> usize {
        match self {
            Self::Full => MIN_ROWS_VALIDATED,
            Self::MeanSubset => MIN_ROWS_VALIDATED_MEAN_SUBSET,
        }
    }

    /// How the summary line names the run.
    fn title(self) -> &'static str {
        match self {
            Self::Full => "Aspects gate",
            Self::MeanSubset => "Aspects gate (mean subset)",
        }
    }
}

/// The frame a corpus group was generated in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Group {
    /// Geocentric apparent, tropical.
    Geo,
    /// Geocentric mean of date, tropical.
    Mean,
    /// Heliocentric, geometric.
    Helio,
}

impl Group {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "geo" => Self::Geo,
            "mean" => Self::Mean,
            "helio" => Self::Helio,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Geo => "geo",
            Self::Mean => "mean",
            Self::Helio => "helio",
        }
    }

    fn reference(self) -> CrossingReference {
        match self {
            Self::Geo => CrossingFrame::GeocentricApparentOfDate.into(),
            Self::Mean => CrossingFrame::GeocentricMeanOfDate.into(),
            Self::Helio => CrossingFrame::Heliocentric.into(),
        }
    }
}

/// One pair the reference tool scanned, at every angle in [`ANGLES_DEG`].
#[derive(Clone, Copy, Debug)]
struct Pair {
    group: Group,
    first: &'static str,
    second: &'static str,
    span: (f64, f64),
}

const fn pair(group: Group, first: &'static str, second: &'static str, span: (f64, f64)) -> Pair {
    Pair {
        group,
        first,
        second,
        span,
    }
}

/// The corpus plan, mirroring `main` of `tools/se-aspects-reference`. A
/// pair-and-angle series with no corpus rows is still compared: the engine
/// must find nothing there either.
const PAIRS: [Pair; 11] = [
    pair(Group::Geo, "Sun", "Moon", SHORT_SPAN),
    pair(Group::Geo, "Sun", "Mercury", FULL_SPAN),
    pair(Group::Geo, "Mercury", "Venus", FULL_SPAN),
    pair(Group::Geo, "Venus", "Mars", FULL_SPAN),
    pair(Group::Geo, "Mars", "Jupiter", FULL_SPAN),
    pair(Group::Geo, "Mars", "Saturn", FULL_SPAN),
    pair(Group::Geo, "Jupiter", "Saturn", FULL_SPAN),
    pair(Group::Geo, "Saturn", "Pluto", FULL_SPAN),
    pair(Group::Mean, "Mercury", "Venus", SHORT_SPAN),
    pair(Group::Mean, "Mars", "Saturn", SHORT_SPAN),
    pair(Group::Helio, "Mars", "Jupiter", FULL_SPAN),
];

fn body_from_name(name: &str) -> Option<CelestialBody> {
    Some(match name {
        "Sun" => CelestialBody::Sun,
        "Moon" => CelestialBody::Moon,
        "Mercury" => CelestialBody::Mercury,
        "Venus" => CelestialBody::Venus,
        "Mars" => CelestialBody::Mars,
        "Jupiter" => CelestialBody::Jupiter,
        "Saturn" => CelestialBody::Saturn,
        "Pluto" => CelestialBody::Pluto,
        _ => return None,
    })
}

/// One exact aspect, from either side of the comparison.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Found {
    jd: f64,
    first_lon_deg: f64,
    second_lon_deg: f64,
}

/// A corpus event: the reference instant and longitudes, and the pair's
/// relative longitude speed there.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Expected {
    found: Found,
    rel_speed_deg_per_day: f64,
}

/// One corpus row, with its pair and angle as indices into [`PAIRS`] and
/// [`ANGLES_DEG`].
#[derive(Clone, Copy, Debug)]
struct Row {
    pair: usize,
    angle: usize,
    expected: Expected,
}

#[derive(Debug)]
pub enum AspectsError {
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
    /// The engine and the corpus disagree on how many times an aspect is exact.
    CountMismatch {
        series: String,
        got: usize,
        want: usize,
    },
    /// The engine and the corpus disagree on which body is ahead.
    SideMismatch {
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
}

impl std::fmt::Display for AspectsError {
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
                write!(f, "only {validated} aspects validated, floor is {floor}")
            }
            Self::CalculationFailed { series, reason } => {
                write!(f, "{series} aspect search failed: {reason}")
            }
            Self::CountMismatch { series, got, want } => write!(
                f,
                "{series}: engine found {got} exact aspects, corpus has {want}"
            ),
            Self::SideMismatch {
                series,
                index,
                jd_tt,
            } => write!(
                f,
                "{series}: aspect {index} near jd_tt={jd_tt} is on the other side in the corpus"
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
        }
    }
}

impl std::error::Error for AspectsError {}

fn parse_corpus(csv: &str) -> Result<Vec<Row>, AspectsError> {
    let malformed = |what: String| AspectsError::MalformedRow(what);
    let mut rows = Vec::new();
    // The last instant seen in each pair-and-angle series.
    let mut last_jd = [[f64::NEG_INFINITY; ANGLES_DEG.len()]; PAIRS.len()];
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 8 {
            return Err(malformed(format!(
                "expected 8 fields, got {} in {line}",
                fields.len()
            )));
        }
        let group = Group::from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown group {} in {line}", fields[0])))?;
        let pair = PAIRS
            .iter()
            .position(|p| p.group == group && p.first == fields[1] && p.second == fields[2])
            .ok_or_else(|| {
                malformed(format!(
                    "pair {}-{} is not in the {} corpus plan in {line}",
                    fields[1], fields[2], fields[0]
                ))
            })?;
        let num = |i: usize| -> Result<f64, AspectsError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let angle_deg = num(3)?;
        let angle = ANGLES_DEG
            .iter()
            .position(|known| *known == angle_deg)
            .ok_or_else(|| {
                malformed(format!(
                    "angle {angle_deg} is not in the corpus plan in {line}"
                ))
            })?;
        let expected = Expected {
            found: Found {
                jd: num(4)?,
                first_lon_deg: num(5)?,
                second_lon_deg: num(6)?,
            },
            rel_speed_deg_per_day: num(7)?,
        };
        if last_jd[pair][angle] >= expected.found.jd {
            return Err(malformed(format!("rows are not ascending at {line}")));
        }
        last_jd[pair][angle] = expected.found.jd;
        rows.push(Row {
            pair,
            angle,
            expected,
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), AspectsError> {
    crate::corpus_manifest::slice_entry(manifest)
        .map_err(|e| AspectsError::MalformedManifest(e.to_string()))
}

/// What a comparison measured.
#[derive(Clone, Copy, Debug, Default)]
struct Residuals {
    /// Events compared.
    matched: usize,
    max_sep_arcsec: f64,
    max_time_s: f64,
    max_lon_arcsec: f64,
    /// Sum of engine − corpus time over the compared events.
    sum_signed_time_s: f64,
}

impl Residuals {
    fn absorb(&mut self, other: Residuals) {
        self.matched += other.matched;
        self.max_sep_arcsec = self.max_sep_arcsec.max(other.max_sep_arcsec);
        self.max_time_s = self.max_time_s.max(other.max_time_s);
        self.max_lon_arcsec = self.max_lon_arcsec.max(other.max_lon_arcsec);
        self.sum_signed_time_s += other.sum_signed_time_s;
    }

    fn mean_signed_time_s(&self) -> f64 {
        if self.matched == 0 {
            0.0
        } else {
            self.sum_signed_time_s / self.matched as f64
        }
    }
}

fn wrap180(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

fn lon_residual_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    wrap180(got_deg - want_deg).abs() * ARCSEC_PER_DEG
}

/// Whether the first body is ahead of the second.
fn first_is_ahead(found: &Found) -> bool {
    wrap180(found.first_lon_deg - found.second_lon_deg) > 0.0
}

/// The two lists must agree event for event: the same length and, for an
/// angle strictly between 0 and 180 degrees, the same side in order; then
/// every residual within its ceiling. A NaN residual fails closed.
fn compare_exact(
    label: &str,
    angle_deg: f64,
    engine: &[Found],
    corpus: &[Expected],
    ceilings: Ceilings,
) -> Result<Residuals, AspectsError> {
    if engine.len() != corpus.len() {
        return Err(AspectsError::CountMismatch {
            series: label.to_string(),
            got: engine.len(),
            want: corpus.len(),
        });
    }
    let has_side = angle_deg > 0.0 && angle_deg < 180.0;
    let mut residuals = Residuals::default();
    for (index, (got, want)) in engine.iter().zip(corpus).enumerate() {
        if has_side && first_is_ahead(got) != first_is_ahead(&want.found) {
            return Err(AspectsError::SideMismatch {
                series: label.to_string(),
                index,
                jd_tt: want.found.jd,
            });
        }
        let signed_time_s = (got.jd - want.found.jd) * SECONDS_PER_DAY;
        let sep_arcsec =
            (got.jd - want.found.jd).abs() * want.rel_speed_deg_per_day.abs() * ARCSEC_PER_DEG;
        let first_arcsec = lon_residual_arcsec(got.first_lon_deg, want.found.first_lon_deg);
        let second_arcsec = lon_residual_arcsec(got.second_lon_deg, want.found.second_lon_deg);
        let checks = [
            ("separation_arcsec", sep_arcsec, ceilings.sep_arcsec),
            ("first_longitude_arcsec", first_arcsec, ceilings.lon_arcsec),
            (
                "second_longitude_arcsec",
                second_arcsec,
                ceilings.lon_arcsec,
            ),
        ];
        for (kind, residual, ceiling) in checks {
            if residual.is_nan() || residual > ceiling {
                return Err(AspectsError::CeilingExceeded {
                    series: label.to_string(),
                    jd_tt: want.found.jd,
                    kind,
                    residual,
                    ceiling,
                });
            }
        }
        residuals.absorb(Residuals {
            matched: 1,
            max_sep_arcsec: sep_arcsec,
            max_time_s: signed_time_s.abs(),
            max_lon_arcsec: first_arcsec.max(second_arcsec),
            sum_signed_time_s: signed_time_s,
        });
    }
    Ok(residuals)
}

fn check_floor(validated: usize, floor: usize) -> Result<(), AspectsError> {
    let floor = floor.max(1);
    if validated < floor {
        return Err(AspectsError::TooFewRowsValidated { validated, floor });
    }
    Ok(())
}

#[derive(Debug)]
pub struct AspectsReport {
    /// Exact aspects compared against the corpus.
    pub rows_validated: usize,
    pair_lines: Vec<String>,
    summary_line: String,
}

impl AspectsReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }

    /// One line per corpus pair with its counts per angle and its measured
    /// maxima; the basis for the ceilings in `aspects_thresholds`.
    pub fn pair_lines(&self) -> &[String] {
        &self.pair_lines
    }
}

fn validate(csv: &str, manifest: &str) -> Result<AspectsReport, AspectsError> {
    validate_scoped(csv, manifest, Scope::Full)
}

fn validate_scoped(csv: &str, manifest: &str, scope: Scope) -> Result<AspectsReport, AspectsError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(AspectsError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(AspectsError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let engine = EventEngine::new(packaged_backend());
    // The corpus epoch is TT; the engine reads the Julian day as TDB. The two
    // differ by under 2 ms, far below every ceiling here.
    let tdb = |jd: f64| Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let mut total = Residuals::default();
    let mut pair_lines = Vec::new();
    let in_scope = PAIRS
        .iter()
        .enumerate()
        .filter(|(_, pair)| scope.includes(pair.group));
    for (pair_index, pair) in in_scope {
        let name = format!("{}-{}", pair.first, pair.second);
        let pair_label = format!("{} {name}", pair.group.name());
        let failed = |series: &str, reason: String| AspectsError::CalculationFailed {
            series: series.to_string(),
            reason,
        };
        let ceilings = ceilings_for(&name)
            .ok_or_else(|| failed(&pair_label, "no ceilings for this pair".into()))?;
        let body = |body_name: &str| {
            body_from_name(body_name)
                .ok_or_else(|| failed(&pair_label, format!("unknown body {body_name}")))
        };
        let (first, second) = (body(pair.first)?, body(pair.second)?);
        let mut residuals = Residuals::default();
        let mut counts = Vec::new();
        for (angle_index, angle_deg) in ANGLES_DEG.iter().enumerate() {
            let label = format!("{pair_label} {angle_deg:.0}");
            let corpus: Vec<Expected> = rows
                .iter()
                .filter(|row| row.pair == pair_index && row.angle == angle_index)
                .map(|row| row.expected)
                .collect();
            let found: Vec<Found> = engine
                .aspects_in_range(
                    first.clone(),
                    second.clone(),
                    Angle::from_degrees(*angle_deg),
                    pair.group.reference(),
                    tdb(pair.span.0),
                    tdb(pair.span.1),
                )
                .map_err(|e| failed(&label, e.to_string()))?
                .into_iter()
                .map(|event| Found {
                    jd: event.instant.julian_day.days(),
                    first_lon_deg: event.first_longitude.degrees(),
                    second_lon_deg: event.second_longitude.degrees(),
                })
                .collect();
            residuals.absorb(compare_exact(
                &label, *angle_deg, &found, &corpus, ceilings,
            )?);
            counts.push(format!("{angle_deg:.0}°: {}", found.len()));
        }
        pair_lines.push(format!(
            "{pair_label}: {} compared ({}), max sep {:.3}\", max time {:.1} s, mean signed time {:+.1} s, max lon {:.3}\"",
            residuals.matched,
            counts.join(", "),
            residuals.max_sep_arcsec,
            residuals.max_time_s,
            residuals.mean_signed_time_s(),
            residuals.max_lon_arcsec,
        ));
        total.absorb(residuals);
    }
    check_floor(total.matched, scope.floor())?;
    let summary_line = format!(
        "{}: {} exact aspects validated across {} pairs vs Swiss Ephemeris corpus (event for event at 0, 60, 90, 120 and 180 degrees), max separation residual {:.3}\", max time {:.1} s, max lon {:.3}\"",
        scope.title(),
        total.matched,
        pair_lines.len(),
        total.max_sep_arcsec,
        total.max_time_s,
        total.max_lon_arcsec,
    );
    Ok(AspectsReport {
        rows_validated: total.matched,
        pair_lines,
        summary_line,
    })
}

/// The full gate: every corpus pair at every angle, floor
/// `MIN_ROWS_VALIDATED` (10359 events). Run by `validate-aspects` /
/// `mise run gate-aspects` (its own nightly job and a `release-gate`
/// dependency) and by the opt-in `PLEIADES_FULL_ASPECTS_GATE=1` test. About
/// 880 s (15 min) as the command and 1110.8 s (18.5 min) as the in-crate
/// test, dev/test profile (2026-10-02); too slow for nightly `test-full`.
pub fn validate_aspects_corpus() -> Result<AspectsReport, AspectsError> {
    validate(CORPUS_CSV, MANIFEST)
}

/// The release-battery subset: verifies the checksum and row count of the
/// whole corpus, then compares only the `mean` group (Mercury–Venus and
/// Mars–Saturn over 1990–2030), floor `MIN_ROWS_VALIDATED_MEAN_SUBSET`.
/// Fail-closed like the full gate. 372 events, about 16 s in the dev/test
/// profile (2026-10-02).
pub fn validate_aspects_corpus_subset() -> Result<AspectsReport, AspectsError> {
    validate_scoped(CORPUS_CSV, MANIFEST, Scope::MeanSubset)
}

#[cfg(test)]
mod tests;
