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
//! Tiers: the full gate (`mise run gate-aspects`: planets, asteroids and
//! fictitious bodies; see
//! [`validate_aspects_corpus`]) and the lunar-point gate (`mise run
//! gate-aspects-lunar`; see [`validate_lunar_point_aspects_corpus`]; split
//! out because the combined step took 15m40s nightly on 2026-10-17) each run
//! in their own nightly job and in `release-gate`; the `mean` subset (about 16 s) runs in the release
//! battery (`release-smoke`), after a scan-free checksum and row-count check
//! of the asteroid and lunar-point corpora and a check of the graze-window
//! file against its manifest and the corpus (issue #254).
//!
//! Asteroids (issue #168): Sun–Ceres, Sun–Pallas, Sun–Juno, Sun–Vesta,
//! Mars–Vesta and Jupiter–Ceres over 1900–2100 and Moon–Ceres over 1990–2030
//! are compared event for event against `asteroids.csv`, generated with
//! SWIEPH and seas_18 by the same tool's `--asteroids` mode, in the full gate
//! only (the release battery checks only that file's manifest). A Sun or Moon
//! pair's separation never turns; Mars–Vesta and Jupiter–Ceres turn 188 and
//! 315 times, so the asteroids' retrograde loops reach the gate (issue #253).
//! None of their turns comes within 209" of an angle, and no two events of
//! one angle fall within one 2-day search step.
//!
//! Lunar points (issue #168): the true node (against the Sun, Mars, Jupiter,
//! Saturn, Uranus, Neptune and Pluto), the osculating (true) apogee (Sun,
//! Jupiter, Saturn, Pluto), the mean node (the seven planets of the true
//! node) and the mean apogee (Sun, Jupiter, Saturn, Pluto) over 1900–2100 are
//! compared event for event against `lunar-points.csv`, generated with SWIEPH
//! by `tools/se-aspects-reference --lunar-points`, in the full gate only (the
//! release battery checks only that file's manifest too, and
//! `lunar-points-grazes.csv`'s). The tool's graze margin applies: 3" (12" for the osculating apogee).
//! Wherever a pair's separation turns within the tool's graze margin of a
//! level it records a graze window in `lunar-points-grazes.csv`; there an
//! event pair may exist in one ephemeris only, so the gate checks that the
//! engine's and the corpus's event counts in the window both have the
//! parity of the window's recorded SE count, compares equal counts without
//! ceilings (reporting their residuals), and compares every event outside a
//! window strictly. See `grazes`.
//!
//! Fictitious bodies (issue #168): the 19 bodies with the Sun, and Cupido to
//! Poseidon and Transpluto with Saturn (28 pairs, 38733 events), over
//! 1900–2100 in the geometric mean-of-date frame, are compared event for event
//! against `fictitious.csv`, generated with SWIEPH and a pinned `seorbel.txt`
//! by `tools/se-aspects-reference --fictitious`, in the full gate (and alone
//! as `validate-aspects-fictitious`). The corpus has no graze windows. The
//! apparent frame is not gated: SE's light-time correction for fictitious
//! bodies jumps by up to about 5" at isolated instants.

use crate::aspects_thresholds::{
    ceilings_for, Ceilings, MIN_ROWS_VALIDATED, MIN_ROWS_VALIDATED_ASTEROIDS,
    MIN_ROWS_VALIDATED_FICTITIOUS, MIN_ROWS_VALIDATED_LUNAR_POINTS, MIN_ROWS_VALIDATED_MEAN_SUBSET,
};
use pleiades_apparent::fnv1a64;
use pleiades_backend::{EphemerisBackend, RoutingBackend};
use pleiades_data::{packaged_backend, PackagedDataBackend};
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine};
use pleiades_fict::FictitiousBackend;
use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};

mod grazes;

use grazes::{check_grazes, split_windows, windows_of, GrazeWindow, WindowStats};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/aspects.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/manifest.txt"
));
const ASTEROID_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/asteroids.csv"
));
const ASTEROID_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/asteroids-manifest.txt"
));
const LUNAR_POINT_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/lunar-points.csv"
));
const LUNAR_POINT_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/lunar-points-manifest.txt"
));
const LUNAR_POINT_GRAZES_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/lunar-points-grazes.csv"
));
const LUNAR_POINT_GRAZES_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/lunar-points-grazes-manifest.txt"
));
const FICTITIOUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/fictitious.csv"
));
const FICTITIOUS_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/fictitious-manifest.txt"
));
const FICTITIOUS_GRAZES_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/fictitious-grazes.csv"
));
const FICTITIOUS_GRAZES_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/fictitious-grazes-manifest.txt"
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
    /// The asteroid corpus (`asteroids.csv`, Swiss Ephemeris SWIEPH with
    /// seas_18): the [`ASTEROID_PAIRS`]. Part of the full gate only (issue
    /// #168).
    Asteroids,
    /// The lunar-point corpus (`lunar-points.csv`, Swiss Ephemeris SWIEPH): the
    /// true node, osculating apogee, mean node and mean apogee against the
    /// [`LUNAR_POINT_PAIRS`], with graze windows compared by count parity
    /// (`lunar-points-grazes.csv`). Part of the full gate only (issue #168).
    LunarPoints,
    /// The fictitious-body corpus (`fictitious.csv`, Swiss Ephemeris SWIEPH with
    /// `seorbel.txt`, mean-of-date frame): the 19 fictitious bodies against the
    /// Sun and nine of them against Saturn ([`FICTITIOUS_PAIRS`]), served by the
    /// fictitious backend routed beside the packaged one, with graze windows
    /// compared by count parity (`fictitious-grazes.csv`). Part of the full gate
    /// and its own command (issue #168).
    Fictitious,
}

impl Scope {
    fn includes(self, group: Group) -> bool {
        match self {
            Self::Full => true,
            Self::MeanSubset => group == Group::Mean,
            Self::Asteroids | Self::LunarPoints | Self::Fictitious => true,
        }
    }

    fn floor(self) -> usize {
        match self {
            Self::Full => MIN_ROWS_VALIDATED,
            Self::MeanSubset => MIN_ROWS_VALIDATED_MEAN_SUBSET,
            Self::Asteroids => MIN_ROWS_VALIDATED_ASTEROIDS,
            Self::LunarPoints => MIN_ROWS_VALIDATED_LUNAR_POINTS,
            Self::Fictitious => MIN_ROWS_VALIDATED_FICTITIOUS,
        }
    }

    /// How the summary line names the run.
    fn title(self) -> &'static str {
        match self {
            Self::Full => "Aspects gate",
            Self::MeanSubset => "Aspects gate (mean subset)",
            Self::Asteroids => "Asteroid aspects",
            Self::LunarPoints => "Lunar-point aspects",
            Self::Fictitious => "Fictitious-body aspects",
        }
    }

    /// The corpus plan this scope's corpus was generated from.
    fn pairs(self) -> &'static [Pair] {
        match self {
            Self::Full | Self::MeanSubset => &PAIRS,
            Self::Asteroids => &ASTEROID_PAIRS,
            Self::LunarPoints => &LUNAR_POINT_PAIRS,
            Self::Fictitious => &FICTITIOUS_PAIRS,
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

/// The asteroid corpus plan, mirroring the `--asteroids` mode of
/// `tools/se-aspects-reference`.
const ASTEROID_PAIRS: [Pair; 7] = [
    pair(Group::Geo, "Sun", "Ceres", FULL_SPAN),
    pair(Group::Geo, "Sun", "Pallas", FULL_SPAN),
    pair(Group::Geo, "Sun", "Juno", FULL_SPAN),
    pair(Group::Geo, "Sun", "Vesta", FULL_SPAN),
    pair(Group::Geo, "Moon", "Ceres", SHORT_SPAN),
    pair(Group::Geo, "Mars", "Vesta", FULL_SPAN),
    pair(Group::Geo, "Jupiter", "Ceres", FULL_SPAN),
];

/// The lunar-point corpus plan, mirroring the `--lunar-points` mode of
/// `tools/se-aspects-reference`.
const LUNAR_POINT_PAIRS: [Pair; 22] = [
    pair(Group::Geo, "TrueNode", "Sun", FULL_SPAN),
    pair(Group::Geo, "TrueNode", "Mars", FULL_SPAN),
    pair(Group::Geo, "TrueNode", "Jupiter", FULL_SPAN),
    pair(Group::Geo, "TrueNode", "Saturn", FULL_SPAN),
    pair(Group::Geo, "TrueNode", "Uranus", FULL_SPAN),
    pair(Group::Geo, "TrueNode", "Neptune", FULL_SPAN),
    pair(Group::Geo, "TrueNode", "Pluto", FULL_SPAN),
    pair(Group::Geo, "TrueApogee", "Sun", FULL_SPAN),
    pair(Group::Geo, "TrueApogee", "Jupiter", FULL_SPAN),
    pair(Group::Geo, "TrueApogee", "Saturn", FULL_SPAN),
    pair(Group::Geo, "TrueApogee", "Pluto", FULL_SPAN),
    pair(Group::Geo, "MeanNode", "Sun", FULL_SPAN),
    pair(Group::Geo, "MeanNode", "Mars", FULL_SPAN),
    pair(Group::Geo, "MeanNode", "Jupiter", FULL_SPAN),
    pair(Group::Geo, "MeanNode", "Saturn", FULL_SPAN),
    pair(Group::Geo, "MeanNode", "Uranus", FULL_SPAN),
    pair(Group::Geo, "MeanNode", "Neptune", FULL_SPAN),
    pair(Group::Geo, "MeanNode", "Pluto", FULL_SPAN),
    pair(Group::Geo, "MeanApogee", "Sun", FULL_SPAN),
    pair(Group::Geo, "MeanApogee", "Jupiter", FULL_SPAN),
    pair(Group::Geo, "MeanApogee", "Saturn", FULL_SPAN),
    pair(Group::Geo, "MeanApogee", "Pluto", FULL_SPAN),
];

/// The fictitious-body corpus plan, mirroring the `--fictitious` mode of
/// `tools/se-aspects-reference`.
const FICTITIOUS_PAIRS: [Pair; 28] = [
    pair(Group::Mean, "Cupido", "Sun", FULL_SPAN),
    pair(Group::Mean, "Hades", "Sun", FULL_SPAN),
    pair(Group::Mean, "Zeus", "Sun", FULL_SPAN),
    pair(Group::Mean, "Kronos", "Sun", FULL_SPAN),
    pair(Group::Mean, "Apollon", "Sun", FULL_SPAN),
    pair(Group::Mean, "Admetos", "Sun", FULL_SPAN),
    pair(Group::Mean, "Vulkanus", "Sun", FULL_SPAN),
    pair(Group::Mean, "Poseidon", "Sun", FULL_SPAN),
    pair(Group::Mean, "Transpluto", "Sun", FULL_SPAN),
    pair(Group::Mean, "Nibiru", "Sun", FULL_SPAN),
    pair(Group::Mean, "Harrington", "Sun", FULL_SPAN),
    pair(Group::Mean, "NeptuneLeverrier", "Sun", FULL_SPAN),
    pair(Group::Mean, "NeptuneAdams", "Sun", FULL_SPAN),
    pair(Group::Mean, "PlutoLowell", "Sun", FULL_SPAN),
    pair(Group::Mean, "PlutoPickering", "Sun", FULL_SPAN),
    pair(Group::Mean, "Vulcan", "Sun", FULL_SPAN),
    pair(Group::Mean, "WhiteMoon", "Sun", FULL_SPAN),
    pair(Group::Mean, "Proserpina", "Sun", FULL_SPAN),
    pair(Group::Mean, "Waldemath", "Sun", FULL_SPAN),
    pair(Group::Mean, "Cupido", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Hades", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Zeus", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Kronos", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Apollon", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Admetos", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Vulkanus", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Poseidon", "Saturn", FULL_SPAN),
    pair(Group::Mean, "Transpluto", "Saturn", FULL_SPAN),
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
        "Uranus" => CelestialBody::Uranus,
        "Neptune" => CelestialBody::Neptune,
        "Pluto" => CelestialBody::Pluto,
        "Ceres" => CelestialBody::Ceres,
        "Pallas" => CelestialBody::Pallas,
        "Juno" => CelestialBody::Juno,
        "Vesta" => CelestialBody::Vesta,
        "TrueNode" => CelestialBody::TrueNode,
        "TrueApogee" => CelestialBody::TrueApogee,
        "MeanNode" => CelestialBody::MeanNode,
        "MeanApogee" => CelestialBody::MeanApogee,
        "Cupido" => CelestialBody::Cupido,
        "Hades" => CelestialBody::Hades,
        "Zeus" => CelestialBody::Zeus,
        "Kronos" => CelestialBody::Kronos,
        "Apollon" => CelestialBody::Apollon,
        "Admetos" => CelestialBody::Admetos,
        "Vulkanus" => CelestialBody::Vulkanus,
        "Poseidon" => CelestialBody::Poseidon,
        "Transpluto" => CelestialBody::Transpluto,
        "Nibiru" => CelestialBody::Nibiru,
        "Harrington" => CelestialBody::Harrington,
        "NeptuneLeverrier" => CelestialBody::NeptuneLeverrier,
        "NeptuneAdams" => CelestialBody::NeptuneAdams,
        "PlutoLowell" => CelestialBody::PlutoLowell,
        "PlutoPickering" => CelestialBody::PlutoPickering,
        "Vulcan" => CelestialBody::Vulcan,
        "WhiteMoon" => CelestialBody::WhiteMoon,
        "Proserpina" => CelestialBody::Proserpina,
        "Waldemath" => CelestialBody::Waldemath,
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

/// One corpus row, with its pair and angle as indices into the corpus plan
/// it was parsed against ([`PAIRS`], [`ASTEROID_PAIRS`], [`LUNAR_POINT_PAIRS`] or [`FICTITIOUS_PAIRS`]) and into
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
    /// The grazes file is malformed (issue #168 (c)).
    MalformedGrazes(String),
    /// A graze window's recorded SE count disagrees with the corpus.
    GrazeEventCount {
        pair: usize,
        angle_deg: f64,
        lo: f64,
        hi: f64,
        recorded: usize,
        corpus: usize,
    },
    /// A graze window holds a count of the wrong parity on one side.
    GrazeWindowParity {
        series: String,
        lo: f64,
        hi: f64,
        side: &'static str,
        count: usize,
        se_events: usize,
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
            Self::MalformedGrazes(s) => write!(f, "malformed grazes row: {s}"),
            Self::GrazeEventCount {
                pair,
                angle_deg,
                lo,
                hi,
                recorded,
                corpus,
            } => write!(
                f,
                "graze window [{lo}, {hi}] of pair {pair} at {angle_deg} degrees records {recorded} SE events, the corpus has {corpus}"
            ),
            Self::GrazeWindowParity {
                series,
                lo,
                hi,
                side,
                count,
                se_events,
            } => write!(
                f,
                "{series}: graze window [{lo}, {hi}] holds {count} {side} events; its parity must match the {se_events} SE events'"
            ),
        }
    }
}

impl std::error::Error for AspectsError {}

/// Parses `csv` against the corpus plan `pairs`: a row whose pair is not in
/// the plan is malformed.
fn parse_corpus(csv: &str, pairs: &[Pair]) -> Result<Vec<Row>, AspectsError> {
    let malformed = |what: String| AspectsError::MalformedRow(what);
    let mut rows = Vec::new();
    // The last instant seen in each pair-and-angle series.
    let mut last_jd = vec![[f64::NEG_INFINITY; ANGLES_DEG.len()]; pairs.len()];
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
        let pair = pairs
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

/// The residuals of one engine event against its corpus event: separation
/// (arcseconds), signed time (engine − corpus, seconds), and the first and
/// second body's longitude (arcseconds).
fn residual_parts(got: &Found, want: &Expected) -> (f64, f64, f64, f64) {
    let signed_time_s = (got.jd - want.found.jd) * SECONDS_PER_DAY;
    let sep_arcsec =
        (got.jd - want.found.jd).abs() * want.rel_speed_deg_per_day.abs() * ARCSEC_PER_DEG;
    let first_arcsec = lon_residual_arcsec(got.first_lon_deg, want.found.first_lon_deg);
    let second_arcsec = lon_residual_arcsec(got.second_lon_deg, want.found.second_lon_deg);
    (sep_arcsec, signed_time_s, first_arcsec, second_arcsec)
}

/// [`residual_parts`] with the two longitude residuals folded into their
/// maximum: `(sep_arcsec, signed_time_s, max_lon_arcsec)`.
fn residuals_of(got: &Found, want: &Expected) -> (f64, f64, f64) {
    let (sep, time_s, first, second) = residual_parts(got, want);
    (sep, time_s, max_nan(first, second))
}

/// The larger of two values, NaN if either is NaN (`f64::max` would drop it),
/// so report-only statistics make a NaN residual visible.
fn max_nan(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
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
        let (sep_arcsec, signed_time_s, first_arcsec, second_arcsec) = residual_parts(got, want);
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

    /// Chains the planet, asteroid and true-node reports in order: rows
    /// summed, pair lines in order, summary lines joined with "; ".
    fn merged_with(self, other: AspectsReport) -> AspectsReport {
        let mut pair_lines = self.pair_lines;
        pair_lines.extend(other.pair_lines);
        AspectsReport {
            rows_validated: self.rows_validated + other.rows_validated,
            pair_lines,
            summary_line: format!("{}; {}", self.summary_line, other.summary_line),
        }
    }
}

fn validate(csv: &str, manifest: &str) -> Result<AspectsReport, AspectsError> {
    validate_scoped(csv, manifest, Scope::Full, None)
}

/// Ties a corpus to its manifest without scanning: the checksum, the parse
/// against `pairs` and the row count. Returns the parsed rows.
fn check_manifest(csv: &str, manifest: &str, pairs: &[Pair]) -> Result<Vec<Row>, AspectsError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(AspectsError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv, pairs)?;
    if rows.len() != manifest_rows {
        return Err(AspectsError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }
    Ok(rows)
}

/// `grazes` is the graze-window file and its manifest, for the lunar-point
/// and fictitious scopes; every other scope passes `None`.
fn validate_scoped(
    csv: &str,
    manifest: &str,
    scope: Scope,
    grazes: Option<(&str, &str)>,
) -> Result<AspectsReport, AspectsError> {
    let rows = check_manifest(csv, manifest, scope.pairs())?;
    let windows = match grazes {
        Some((grazes_csv, grazes_manifest)) => {
            check_grazes(grazes_csv, grazes_manifest, scope.pairs(), &rows)?
        }
        None => Vec::new(),
    };

    // The fictitious bodies are served by their own backend, routed beside the
    // packaged one for the planets; every other scope needs the packaged one.
    match scope {
        Scope::Fictitious => {
            let engine = EventEngine::new(RoutingBackend::new(vec![
                Box::new(PackagedDataBackend::new()),
                Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
            ]));
            compare_scope(&engine, scope, &rows, &windows)
        }
        _ => compare_scope(
            &EventEngine::new(packaged_backend()),
            scope,
            &rows,
            &windows,
        ),
    }
}

/// Compares every in-scope pair of `rows` (and the graze `windows`) with the
/// events `engine` finds.
fn compare_scope<B: EphemerisBackend>(
    engine: &EventEngine<B>,
    scope: Scope,
    rows: &[Row],
    windows: &[GrazeWindow],
) -> Result<AspectsReport, AspectsError> {
    // The corpus epoch is TT; the engine reads the Julian day as TDB. The two
    // differ by under 2 ms, far below every ceiling here.
    let tdb = |jd: f64| Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let mut total = Residuals::default();
    let mut pair_lines = Vec::new();
    let in_scope = scope
        .pairs()
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
        let mut window_stats = WindowStats::default();
        let mut counts = Vec::new();
        let angles = ANGLES_DEG.map(Angle::from_degrees);
        let events = engine
            .aspects_in_range_for_angles(
                first.clone(),
                second.clone(),
                &angles,
                pair.group.reference(),
                tdb(pair.span.0),
                tdb(pair.span.1),
            )
            .map_err(|e| failed(&pair_label, e.to_string()))?;
        for (angle_index, angle_deg) in ANGLES_DEG.iter().enumerate() {
            let label = format!("{pair_label} {angle_deg:.0}");
            let corpus: Vec<Expected> = rows
                .iter()
                .filter(|row| row.pair == pair_index && row.angle == angle_index)
                .map(|row| row.expected)
                .collect();
            let found: Vec<Found> = events
                .iter()
                .filter(|event| event.angle.degrees() == *angle_deg)
                .map(|event| Found {
                    jd: event.instant.julian_day.days(),
                    first_lon_deg: event.first_longitude.degrees(),
                    second_lon_deg: event.second_longitude.degrees(),
                })
                .collect();
            counts.push(format!("{angle_deg:.0}°: {}", found.len()));
            let (found, corpus, stats) = split_windows(
                &label,
                &windows_of(windows, pair_index, angle_index),
                &found,
                &corpus,
            )?;
            window_stats.absorb(stats);
            residuals.absorb(compare_exact(
                &label, *angle_deg, &found, &corpus, ceilings,
            )?);
        }
        let mut line = format!(
            "{pair_label}: {} compared (engine events found {}), max sep {:.3}\", max time {:.1} s, mean signed time {:+.1} s, max lon {:.3}\"",
            residuals.matched,
            counts.join(", "),
            residuals.max_sep_arcsec,
            residuals.max_time_s,
            residuals.mean_signed_time_s(),
            residuals.max_lon_arcsec,
        );
        if matches!(scope, Scope::LunarPoints | Scope::Fictitious) {
            line.push_str(&format!(
                ", graze windows {} ({} differing, {} events compared inside, max sep {:.3}\", max time {:.1} s, max lon {:.3}\")",
                window_stats.windows,
                window_stats.differing,
                window_stats.compared,
                window_stats.max_sep_arcsec,
                window_stats.max_time_s,
                window_stats.max_lon_arcsec,
            ));
        }
        pair_lines.push(line);
        total.absorb(residuals);
    }
    check_floor(total.matched, scope.floor())?;
    let source = match scope {
        Scope::Asteroids => "Swiss Ephemeris SWIEPH (seas_18) corpus",
        Scope::LunarPoints => "Swiss Ephemeris SWIEPH corpus",
        Scope::Fictitious => "Swiss Ephemeris SWIEPH corpus (mean of date)",
        Scope::Full | Scope::MeanSubset => "Swiss Ephemeris corpus",
    };
    let summary_line = format!(
        "{}: {} exact aspects validated across {} pairs vs {source} (event for event at 0, 60, 90, 120 and 180 degrees), max separation residual {:.3}\", max time {:.1} s, max lon {:.3}\"",
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

/// The full planet, asteroid and fictitious-body gate: every planet corpus pair
/// at every angle (floor `MIN_ROWS_VALIDATED`, 10359 events), then the asteroid
/// pairs (floor `MIN_ROWS_VALIDATED_ASTEROIDS`, 10118 events), then the
/// fictitious-body pairs in the mean-of-date frame (floor
/// `MIN_ROWS_VALIDATED_FICTITIOUS`, 38733 events; see
/// [`validate_fictitious_aspects_corpus`]). The lunar points run
/// separately, in [`validate_lunar_point_aspects_corpus`]. Run by
/// `validate-aspects` / `mise run gate-aspects` (its own nightly job and a
/// `release-gate` dependency) and by the opt-in `PLEIADES_FULL_ASPECTS_GATE=1` test.
/// Measured on the 4-core nightly runner with the planet and asteroid passes
/// (five nightly runs at 3f5849db6, 2026-10-11..15, warm build): the gate
/// command takes 77–151 s (median 114 s) and the whole `aspects-gate` job
/// 2m00s–3m07s, against an 80-min step and 90-min job timeout. The gate does not time its
/// passes; the planets-only command took 82 s at b23c6ca39 (2026-10-10, the
/// run before #251), so the asteroid pass (`Scope::Asteroids`, seven pairs)
/// is roughly 30 s there, less than the run-to-run spread above. On the
/// 24-core dev box under load it took 54.5 s in the release profile
/// (2026-10-10). With the true node merged in, the nightly runner measured
/// the combined gate at 5m58s (2026-10-16, run 38971941852); with the apogee
/// and mean points it measured 15m40s for the step (2026-10-17, run
/// 39039081162), over the 15-minute threshold, so the lunar points were split
/// out; on its own this gate's step then took 53 s (run 39041130704). With the
/// fictitious pass the step took 1m35s (branch run 39052564875, 2026-10-17),
/// so that pass costs about 42 s there (32.3 s locally in release at load
/// 11-12), and it stays in this gate rather than splitting out.
/// The first measurements, 880 s
/// as the command and 1110.8 s as the in-crate test (dev/test profile,
/// 2026-10-02, planet pairs only), predate the event-engine speedups since
/// and are why the gate left nightly `test-full`.
pub fn validate_aspects_corpus() -> Result<AspectsReport, AspectsError> {
    let planets = validate(CORPUS_CSV, MANIFEST)?;
    let asteroids = validate_scoped(ASTEROID_CSV, ASTEROID_MANIFEST, Scope::Asteroids, None)?;
    let fictitious = validate_fictitious_aspects_corpus()?;
    Ok(planets.merged_with(asteroids).merged_with(fictitious))
}

/// The fictitious-body gate: the 19 fictitious bodies against the Sun and
/// Cupido to Poseidon and Transpluto against Saturn (28 pairs), in the
/// mean-of-date frame (SWIEPH with `seorbel.txt`), floor
/// `MIN_ROWS_VALIDATED_FICTITIOUS` (every corpus row outside a graze window).
/// Inside a graze window only the parity of the event counts is checked. Run
/// by `validate-aspects-fictitious` and, with the planets and asteroids, by
/// `validate-aspects`.
pub fn validate_fictitious_aspects_corpus() -> Result<AspectsReport, AspectsError> {
    validate_scoped(
        FICTITIOUS_CSV,
        FICTITIOUS_MANIFEST,
        Scope::Fictitious,
        Some((FICTITIOUS_GRAZES_CSV, FICTITIOUS_GRAZES_MANIFEST)),
    )
}

/// The lunar-point gate: the 22 pairs of the true node and mean node (against
/// the Sun, Mars, Jupiter, Saturn, Uranus, Neptune and Pluto) and of the true
/// apogee and mean apogee (Sun, Jupiter, Saturn, Pluto), floor
/// `MIN_ROWS_VALIDATED_LUNAR_POINTS` (22596 events outside the graze windows;
/// SWIEPH, 3" graze margin, 12" for the osculating apogee). Inside a graze
/// window only the parity of the event counts is checked. Run by
/// `validate-aspects-lunar` / `mise run gate-aspects-lunar` (its own nightly
/// job and a `release-gate` dependency) and by the opt-in
/// `PLEIADES_FULL_ASPECTS_GATE=1` test. The combined planet, asteroid and
/// lunar step took 15m40s on the nightly runner before the split (2026-10-17,
/// run 39039081162); after it, on a branch run of the same day (run
/// 39041130704), this gate's step took 15m53s and the planet and asteroid
/// gate's 53 s.
pub fn validate_lunar_point_aspects_corpus() -> Result<AspectsReport, AspectsError> {
    validate_scoped(
        LUNAR_POINT_CSV,
        LUNAR_POINT_MANIFEST,
        Scope::LunarPoints,
        Some((LUNAR_POINT_GRAZES_CSV, LUNAR_POINT_GRAZES_MANIFEST)),
    )
}

/// The release-battery subset: verifies the checksum and row count of the
/// whole corpus, then compares only the `mean` group (Mercury–Venus and
/// Mars–Saturn over 1990–2030), floor `MIN_ROWS_VALIDATED_MEAN_SUBSET`.
/// Fail-closed like the full gate. 372 events, about 16 s in the dev/test
/// profile (2026-10-02).
///
/// Before that it verifies the asteroid corpus's checksum and row count
/// against its manifest, with no engine scan (issue #254), so a hand-edited
/// `asteroids.csv` or a stale manifest fails the blocking tier rather than
/// only the nightly full gate. The lunar-point corpus (`lunar-points.csv`) is
/// checked the same way, and the graze-window file (`lunar-points-grazes.csv`)
/// is checked against its manifest and the lunar-point corpus. The
/// fictitious-body corpus and its graze windows are checked the same way.
pub fn validate_aspects_corpus_subset() -> Result<AspectsReport, AspectsError> {
    validate_subset(
        ASTEROID_CSV,
        ASTEROID_MANIFEST,
        LUNAR_POINT_CSV,
        LUNAR_POINT_MANIFEST,
        LUNAR_POINT_GRAZES_CSV,
        LUNAR_POINT_GRAZES_MANIFEST,
        FICTITIOUS_CSV,
        FICTITIOUS_MANIFEST,
        FICTITIOUS_GRAZES_CSV,
        FICTITIOUS_GRAZES_MANIFEST,
    )
}

/// [`validate_aspects_corpus_subset`] with the asteroid, lunar-point,
/// fictitious-body and graze-window files passed in, so a test can hand it a tampered file or
/// manifest.
// One (corpus, manifest) pair per checked file is the point of this seam: a test
// hands it one tampered file among the otherwise committed ones.
#[allow(clippy::too_many_arguments)]
fn validate_subset(
    asteroid_csv: &str,
    asteroid_manifest: &str,
    lunar_csv: &str,
    lunar_manifest: &str,
    grazes_csv: &str,
    grazes_manifest: &str,
    fictitious_csv: &str,
    fictitious_manifest: &str,
    fictitious_grazes_csv: &str,
    fictitious_grazes_manifest: &str,
) -> Result<AspectsReport, AspectsError> {
    check_manifest(asteroid_csv, asteroid_manifest, &ASTEROID_PAIRS)?;
    let lunar_rows = check_manifest(lunar_csv, lunar_manifest, &LUNAR_POINT_PAIRS)?;
    check_grazes(grazes_csv, grazes_manifest, &LUNAR_POINT_PAIRS, &lunar_rows)?;
    let fictitious_rows = check_manifest(fictitious_csv, fictitious_manifest, &FICTITIOUS_PAIRS)?;
    check_grazes(
        fictitious_grazes_csv,
        fictitious_grazes_manifest,
        &FICTITIOUS_PAIRS,
        &fictitious_rows,
    )?;
    validate_scoped(CORPUS_CSV, MANIFEST, Scope::MeanSubset, None)
}

#[cfg(test)]
mod tests;
