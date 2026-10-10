//! Fail-closed two-tier `validate-nod-aps` gate over the committed
//! Swiss-Ephemeris `swe_nod_aps` reference corpora
//! (`data/nod-aps-corpus/{nod-aps.csv,manifest.txt}` for the planets and the
//! Moon, `data/nod-aps-corpus/{asteroids.csv,asteroids-manifest.txt}` for
//! Ceres, Pallas, Juno and Vesta, issue #160).
//!
//! Tier 1 (self-consistency, no SE reference): every row is recomputed with
//! `pleiades_events::EventEngine::nod_aps` over the production-style backend
//! chain (`PackagedDataBackend`, `CompositeBackend(Vsop87, Elp)`,
//! `JplSnapshotBackend`, `FictitiousBackend`) and every recomputed point is
//! checked for internal consistency: finite longitude/latitude/distance,
//! longitude in `[0, 360)`, latitude in `[-90, 90]`, distance `> 0`.
//!
//! Tier 2 (SE parity): each recomputed point is compared against the SE
//! `swe_nod_aps` reference columns — wrap-aware longitude residual, latitude
//! residual, relative distance residual, and longitude-speed residual — and
//! accumulated into per-category maxima. Categories are `MEAN_PLANET`,
//! `MEAN_MOON`, `OSCU_PLANET`, `OSCU_MOON`, `OSCU_ASTEROID`: mean vs
//! osculating (methods 2 and 4, heliocentric and barycentric, both count as
//! osculating per the plan), Moon vs asteroids vs everything else (Sun and the
//! eight classical planets share `*_PLANET`). `validate_nod_aps_corpus` gates
//! every category's maxima fail-closed under `crate::nod_aps_thresholds`'s
//! measured ceilings.
//!
//! Corpus scope (see the committed manifests and the CSVs' header comments):
//! `nod-aps.csv` holds the SE 0-9 classical planets (the mean set drops
//! Pluto, SE 9, which has no SE mean elements), from Moshier except the
//! barycentric rows; `asteroids.csv` holds osculating (method 2, fopoint 0)
//! rows for Ceres, Pallas, Juno and Vesta (SE 17-20) from SWIEPH with the
//! pinned `seas_18` file. Chiron and Pholus (SE 15-16) are not sampled, and
//! fictitious bodies (SE 40-58) are OUT OF SCOPE — this SE build's
//! `swe_nod_aps` does not implement fictitious bodies at all. See
//! `summary_line`'s coverage-bound sentence.
//!
//! Sun rows are a special case: SE structurally zeroes the Sun's asc/dsc
//! columns (all 12 fields across both points, for both mean and osculating
//! rows present in the corpus) because it forms the Sun's points from
//! Earth's mean elements, whose ascending node/inclination are zero, so the
//! node rows come out zero ("no nodes for earth", `swecl.c:5436-5439`).
//! **Done as of Task 9 (§R8):** the engine now zeroes the Sun's ascending/
//! descending points to match (`pleiades_events::nod_aps::mean_points_at`
//! and `osculating_points_at`), so for `se_body == 0` rows this gate skips
//! `check_tier1`/Tier-2 residual accumulation for the ascending/descending
//! points entirely and instead asserts BOTH sides are exactly zeroed — the
//! recomputed point (longitude, latitude, distance, and all three speeds)
//! and the SE corpus fields — failing closed on any nonzero value. Perihelion
//! and aphelion stay on the normal Tier-1 + Tier-2 path for the Sun.
//!
//! Each CSV's manifest records its fnv1a64 digest (drift guard); a mismatch
//! fails the gate closed.

use crate::nod_aps_thresholds::*;
use pleiades_apparent::fnv1a64;
use pleiades_backend::{CompositeBackend, EphemerisBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{ApsisConvention, EventEngine, NodApsMethod, NodApsPoint, NodesApsides};
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;
use std::collections::BTreeMap;

const CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/nod-aps-corpus/nod-aps.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/nod-aps-corpus/manifest.txt"
));
const CSV_FILE: &str = "nod-aps.csv";
const ASTEROID_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/nod-aps-corpus/asteroids.csv"
));
const ASTEROID_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/nod-aps-corpus/asteroids-manifest.txt"
));
const ASTEROID_CSV_FILE: &str = "asteroids.csv";

/// Fixture row count pinned by the corpus (Task 7: 72 mean + 6
/// mean-fopoint + 80 osculating + 20 barycentric + 6 oscu-fopoint). Update
/// when the corpus is regenerated.
pub const EXPECTED_ROWS: usize = 184;

/// Asteroid fixture row count pinned by `asteroids.csv` (issue #160: Ceres,
/// Pallas, Juno, Vesta × the 8-epoch grid, osculating). Update when that
/// corpus is regenerated.
pub const EXPECTED_ASTEROID_ROWS: usize = 32;

#[derive(Debug)]
pub enum NodApsError {
    /// A committed CSV's digest disagrees with the manifest.
    ChecksumMismatch {
        file: &'static str,
        got: u64,
        want: u64,
    },
    /// A parsed corpus row count disagrees with its pin ([`EXPECTED_ROWS`]
    /// for `nod-aps.csv`, [`EXPECTED_ASTEROID_ROWS`] for `asteroids.csv`).
    RowCountMismatch {
        file: &'static str,
        expected: usize,
        got: usize,
    },
    /// A Tier-2 residual exceeded its ceiling.
    ToleranceExceeded {
        category: &'static str,
        label: String,
        jd: f64,
        residual: f64,
        ceiling: f64,
    },
    /// Malformed manifest or corpus row (also covers a Tier-1
    /// self-consistency invariant failing on the recomputed row, and an
    /// unrecognized `se_body`/`method`/`fopoint` value — the parse is
    /// fail-closed since only SE 0-9 and 17-20 / methods 1,2,4 / fopoint 0,1
    /// exist in the committed corpora).
    Parse { row: String },
    /// The engine errored while recomputing a row.
    Engine(String),
}

impl std::fmt::Display for NodApsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NodApsError {}

/// Per-category maxima over the four Tier-2 metrics.
#[derive(Debug, Default, Clone, Copy)]
pub struct CategoryMaxima {
    pub max_lon_arcsec: f64,
    pub max_lat_arcsec: f64,
    pub max_dist_rel: f64,
    pub max_lon_speed_deg_day: f64,
}

/// Summary of the measured maxima and checked-row count for the gate.
#[derive(Debug, Default)]
pub struct NodApsReport {
    /// Rows checked from `nod-aps.csv` (planets and the Moon).
    pub rows: usize,
    /// Rows checked from `asteroids.csv` (Ceres, Pallas, Juno, Vesta).
    pub asteroid_rows: usize,
    pub mean_planet: CategoryMaxima,
    pub mean_moon: CategoryMaxima,
    pub oscu_planet: CategoryMaxima,
    pub oscu_moon: CategoryMaxima,
    pub oscu_asteroid: CategoryMaxima,
}

impl NodApsReport {
    /// The gate passed iff every committed row was checked (a silently
    /// truncated corpus is a failure, not a pass). Every ceiling is enforced
    /// fail-closed by [`validate_nod_aps_corpus`], so reaching a report
    /// already implies every checked row was within ceiling.
    pub fn passed(&self) -> bool {
        self.rows == EXPECTED_ROWS && self.asteroid_rows == EXPECTED_ASTEROID_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-nod-aps: {} rows + {} asteroid rows — max residuals: MEAN_PLANET lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day; MEAN_MOON lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day; OSCU_PLANET lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day; OSCU_MOON lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day; OSCU_ASTEROID lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day — asteroids (Ceres, Pallas, Juno, Vesta): osculating, gated vs SWIEPH seas_18 rows; coverage bound: SE swe_nod_aps does not implement fictitious bodies (upstream-disabled), and offline chains cannot sample snapshot-only asteroids such as asteroid:99942-Apophis densely enough; fictitious and other asteroid nod_aps is engine-covered, gate-unreferenced",
            self.rows,
            self.asteroid_rows,
            self.mean_planet.max_lon_arcsec,
            self.mean_planet.max_lat_arcsec,
            self.mean_planet.max_dist_rel,
            self.mean_planet.max_lon_speed_deg_day,
            self.mean_moon.max_lon_arcsec,
            self.mean_moon.max_lat_arcsec,
            self.mean_moon.max_dist_rel,
            self.mean_moon.max_lon_speed_deg_day,
            self.oscu_planet.max_lon_arcsec,
            self.oscu_planet.max_lat_arcsec,
            self.oscu_planet.max_dist_rel,
            self.oscu_planet.max_lon_speed_deg_day,
            self.oscu_moon.max_lon_arcsec,
            self.oscu_moon.max_lat_arcsec,
            self.oscu_moon.max_dist_rel,
            self.oscu_moon.max_lon_speed_deg_day,
            self.oscu_asteroid.max_lon_arcsec,
            self.oscu_asteroid.max_lat_arcsec,
            self.oscu_asteroid.max_dist_rel,
            self.oscu_asteroid.max_lon_speed_deg_day,
        )
    }
}

/// One metric's running maximum, tagged with the offending row's label and
/// `jd_tt` so a ceiling violation can be reported precisely.
#[derive(Debug, Clone)]
struct MetricMax {
    value: f64,
    label: String,
    jd: f64,
}

impl Default for MetricMax {
    fn default() -> Self {
        MetricMax {
            value: 0.0,
            label: String::new(),
            jd: f64::NAN,
        }
    }
}

impl MetricMax {
    fn observe(&mut self, value: f64, label: &str, jd: f64) {
        // A NaN residual never compares `Greater`, so treat it as a new max
        // explicitly — it must never be silently dropped (checked again,
        // fail-closed, at gate time via `residual.is_finite()`).
        let is_new_max = match value.partial_cmp(&self.value) {
            Some(std::cmp::Ordering::Greater) => true,
            Some(_) => false,
            None => true,
        };
        if is_new_max {
            self.value = value;
            self.label = label.to_string();
            self.jd = jd;
        }
    }
}

/// One category's four tracked metric maxima.
#[derive(Debug, Default, Clone)]
struct CategoryTrack {
    lon_arcsec: MetricMax,
    lat_arcsec: MetricMax,
    dist_rel: MetricMax,
    lon_speed_deg_day: MetricMax,
}

impl CategoryTrack {
    fn to_maxima(&self) -> CategoryMaxima {
        CategoryMaxima {
            max_lon_arcsec: self.lon_arcsec.value,
            max_lat_arcsec: self.lat_arcsec.value,
            max_dist_rel: self.dist_rel.value,
            max_lon_speed_deg_day: self.lon_speed_deg_day.value,
        }
    }
}

/// All measured residual maxima over the committed corpora, split into the
/// five categories so each can be gated under its own ceilings. Tier-1
/// self-consistency is enforced during measurement (it never depends on the
/// numeric ceilings); ceiling gating is applied afterwards by
/// [`validate_nod_aps_corpus`].
#[derive(Debug, Default)]
struct Measured {
    rows: usize,
    asteroid_rows: usize,
    mean_planet: CategoryTrack,
    mean_moon: CategoryTrack,
    oscu_planet: CategoryTrack,
    oscu_moon: CategoryTrack,
    oscu_asteroid: CategoryTrack,
}

impl Measured {
    fn into_report(self) -> NodApsReport {
        NodApsReport {
            rows: self.rows,
            asteroid_rows: self.asteroid_rows,
            mean_planet: self.mean_planet.to_maxima(),
            mean_moon: self.mean_moon.to_maxima(),
            oscu_planet: self.oscu_planet.to_maxima(),
            oscu_moon: self.oscu_moon.to_maxima(),
            oscu_asteroid: self.oscu_asteroid.to_maxima(),
        }
    }

    fn category_mut(&mut self, category: Category) -> &mut CategoryTrack {
        match category {
            Category::MeanPlanet => &mut self.mean_planet,
            Category::MeanMoon => &mut self.mean_moon,
            Category::OscuPlanet => &mut self.oscu_planet,
            Category::OscuMoon => &mut self.oscu_moon,
            Category::OscuAsteroid => &mut self.oscu_asteroid,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Category {
    MeanPlanet,
    MeanMoon,
    OscuPlanet,
    OscuMoon,
    OscuAsteroid,
}

impl Category {
    fn name(self) -> &'static str {
        match self {
            Category::MeanPlanet => "MEAN_PLANET",
            Category::MeanMoon => "MEAN_MOON",
            Category::OscuPlanet => "OSCU_PLANET",
            Category::OscuMoon => "OSCU_MOON",
            Category::OscuAsteroid => "OSCU_ASTEROID",
        }
    }
}

/// SE body number -> `CelestialBody`. Only 0–9 (Sun, Moon, Mercury..Pluto)
/// and the asteroids 17–20 (Ceres, Pallas, Juno, Vesta) exist in the
/// committed corpora (no Chiron/Pholus, no fictitious bodies). Any other
/// value is a fail-closed parse error.
fn body_from_se(se_body: i64) -> Option<CelestialBody> {
    match se_body {
        0 => Some(CelestialBody::Sun),
        1 => Some(CelestialBody::Moon),
        2 => Some(CelestialBody::Mercury),
        3 => Some(CelestialBody::Venus),
        4 => Some(CelestialBody::Mars),
        5 => Some(CelestialBody::Jupiter),
        6 => Some(CelestialBody::Saturn),
        7 => Some(CelestialBody::Uranus),
        8 => Some(CelestialBody::Neptune),
        9 => Some(CelestialBody::Pluto),
        17 => Some(CelestialBody::Ceres),
        18 => Some(CelestialBody::Pallas),
        19 => Some(CelestialBody::Juno),
        20 => Some(CelestialBody::Vesta),
        _ => None,
    }
}

fn method_from_se(method: i64) -> Option<NodApsMethod> {
    match method {
        1 => Some(NodApsMethod::Mean),
        2 => Some(NodApsMethod::Osculating),
        4 => Some(NodApsMethod::OsculatingBarycentric),
        _ => None,
    }
}

fn convention_from_fopoint(fopoint: i64) -> Option<ApsisConvention> {
    match fopoint {
        0 => Some(ApsisConvention::Aphelion),
        1 => Some(ApsisConvention::SecondFocus),
        _ => None,
    }
}

fn category_for(body: &CelestialBody, method: NodApsMethod) -> Category {
    let is_asteroid = matches!(
        body,
        CelestialBody::Ceres | CelestialBody::Pallas | CelestialBody::Juno | CelestialBody::Vesta
    );
    if is_asteroid {
        return Category::OscuAsteroid;
    }
    let is_mean = method == NodApsMethod::Mean;
    let is_moon = *body == CelestialBody::Moon;
    match (is_mean, is_moon) {
        (true, true) => Category::MeanMoon,
        (true, false) => Category::MeanPlanet,
        (false, true) => Category::OscuMoon,
        (false, false) => Category::OscuPlanet,
    }
}

fn wrap180(d: f64) -> f64 {
    ((d + 180.0).rem_euclid(360.0)) - 180.0
}

fn parse_manifest(manifest: &str) -> Result<BTreeMap<String, (usize, u64)>, NodApsError> {
    crate::corpus_manifest::file_entries(manifest)
        .map_err(|e| NodApsError::Parse { row: e.to_string() })
}

/// Looks up `file` in `manifest` and compares `fnv1a64(csv)` against the
/// recorded checksum, fail-closed. Returns the manifest's declared row count
/// on success.
fn check_checksum(manifest: &str, file: &'static str, csv: &str) -> Result<usize, NodApsError> {
    let manifest = parse_manifest(manifest)?;
    let (rows, want) = *manifest.get(file).ok_or_else(|| NodApsError::Parse {
        row: format!("manifest missing entry for {file}"),
    })?;
    let got = fnv1a64(csv);
    if got != want {
        return Err(NodApsError::ChecksumMismatch { file, got, want });
    }
    Ok(rows)
}

fn parse_f64(s: &str, row: &str) -> Result<f64, NodApsError> {
    s.trim().parse::<f64>().map_err(|_| NodApsError::Parse {
        row: row.to_string(),
    })
}

fn parse_i64(s: &str, row: &str) -> Result<i64, NodApsError> {
    s.trim().parse::<i64>().map_err(|_| NodApsError::Parse {
        row: row.to_string(),
    })
}

/// One parsed SE reference point (6 columns): lon, lat, dist, dlon, dlat,
/// ddist. Only `dlon` feeds the Tier-2 speed ceiling (per plan/brief); `dlat`
/// and `ddist` are otherwise unused except for the Sun exact-zero check
/// below.
#[derive(Clone, Copy, Debug)]
struct SePoint {
    lon: f64,
    lat: f64,
    dist: f64,
    dlon: f64,
    dlat: f64,
    ddist: f64,
}

fn parse_point(f: &[&str], row: &str) -> Result<SePoint, NodApsError> {
    debug_assert_eq!(f.len(), 6);
    Ok(SePoint {
        lon: parse_f64(f[0], row)?,
        lat: parse_f64(f[1], row)?,
        dist: parse_f64(f[2], row)?,
        dlon: parse_f64(f[3], row)?,
        dlat: parse_f64(f[4], row)?,
        ddist: parse_f64(f[5], row)?,
    })
}

/// Asserts a Sun ascending/descending point is EXACTLY zeroed on both sides:
/// the recomputed `NodApsPoint` (longitude, latitude, distance, all three
/// speeds) and the SE corpus point (all six fields). SE zeroes the Sun's
/// node columns structurally (`swecl.c:5436-5439`, "no nodes for earth"),
/// and the engine now matches (§R8, Task 9) — any nonzero value on either
/// side means the zeroing broke or the corpus drifted, so this fails closed
/// rather than silently comparing residuals against a zero reference.
fn assert_sun_node_zeroed(
    label: &str,
    jd: f64,
    point_name: &str,
    engine: &NodApsPoint,
    se: &SePoint,
) -> Result<(), NodApsError> {
    let engine_zero = engine.longitude_deg == 0.0
        && engine.latitude_deg == 0.0
        && engine.distance_au == 0.0
        && engine.longitude_speed_deg_per_day == 0.0
        && engine.latitude_speed_deg_per_day == 0.0
        && engine.distance_speed_au_per_day == 0.0;
    let se_zero = se.lon == 0.0
        && se.lat == 0.0
        && se.dist == 0.0
        && se.dlon == 0.0
        && se.dlat == 0.0
        && se.ddist == 0.0;
    if !engine_zero || !se_zero {
        return Err(NodApsError::Parse {
            row: format!(
                "{label} [{point_name}] (jd {jd}): Sun node not exactly zeroed — engine {engine:?} se {se:?}"
            ),
        });
    }
    Ok(())
}

/// Checks Tier-1 self-consistency for one recomputed point. Returns an
/// error identifying the offending row/point on failure.
fn check_tier1(label: &str, point_name: &str, p: &NodApsPoint) -> Result<(), NodApsError> {
    if !(p.longitude_deg.is_finite()
        && p.latitude_deg.is_finite()
        && p.distance_au.is_finite()
        && p.longitude_speed_deg_per_day.is_finite())
    {
        return Err(NodApsError::Parse {
            row: format!("{label} {point_name}: non-finite output {p:?}"),
        });
    }
    if !(0.0..360.0).contains(&p.longitude_deg) {
        return Err(NodApsError::Parse {
            row: format!(
                "{label} {point_name}: longitude out of [0,360): {}",
                p.longitude_deg
            ),
        });
    }
    if !(-90.0..=90.0).contains(&p.latitude_deg) {
        return Err(NodApsError::Parse {
            row: format!(
                "{label} {point_name}: latitude out of [-90,90]: {}",
                p.latitude_deg
            ),
        });
    }
    if p.distance_au <= 0.0 {
        return Err(NodApsError::Parse {
            row: format!("{label} {point_name}: distance not > 0: {}", p.distance_au),
        });
    }
    Ok(())
}

/// Accumulates one point's Tier-2 residuals into `track`, tagging each
/// metric's running maximum with `label`/`jd` for precise error reporting.
fn accumulate_tier2(
    track: &mut CategoryTrack,
    label: &str,
    jd: f64,
    point_name: &str,
    engine: &NodApsPoint,
    se: &SePoint,
) {
    let tag = format!("{label} [{point_name}]");
    let lon_res = wrap180(engine.longitude_deg - se.lon).abs() * 3600.0;
    let lat_res = (engine.latitude_deg - se.lat).abs() * 3600.0;
    let dist_res = if se.dist != 0.0 {
        (engine.distance_au - se.dist).abs() / se.dist
    } else {
        (engine.distance_au - se.dist).abs()
    };
    let speed_res = (engine.longitude_speed_deg_per_day - se.dlon).abs();

    track.lon_arcsec.observe(lon_res, &tag, jd);
    track.lat_arcsec.observe(lat_res, &tag, jd);
    track.dist_rel.observe(dist_res, &tag, jd);
    track.lon_speed_deg_day.observe(speed_res, &tag, jd);
}

/// One parsed corpus row: the SE inputs mapped to engine types, plus the
/// four SE reference points.
#[derive(Clone, Debug)]
struct CorpusRow {
    label: String,
    se_body: i64,
    body: CelestialBody,
    method: NodApsMethod,
    convention: ApsisConvention,
    jd_tt: f64,
    asc: SePoint,
    dsc: SePoint,
    peri: SePoint,
    apo: SePoint,
}

/// Parses every data row of a corpus CSV (29 columns; `#` comments and the
/// header are skipped), fail-closed on any malformed or unrecognized value.
fn parse_rows(csv: &str) -> Result<Vec<CorpusRow>, NodApsError> {
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 29 {
            return Err(NodApsError::Parse {
                row: line.to_string(),
            });
        }
        let se_body = parse_i64(f[1], line)?;
        let se_method = parse_i64(f[2], line)?;
        let se_fopoint = parse_i64(f[3], line)?;
        rows.push(CorpusRow {
            label: f[0].to_string(),
            se_body,
            body: body_from_se(se_body).ok_or_else(|| NodApsError::Parse {
                row: format!("{line} (unrecognized se_body {se_body})"),
            })?,
            method: method_from_se(se_method).ok_or_else(|| NodApsError::Parse {
                row: format!("{line} (unrecognized method {se_method})"),
            })?,
            convention: convention_from_fopoint(se_fopoint).ok_or_else(|| NodApsError::Parse {
                row: format!("{line} (unrecognized fopoint {se_fopoint})"),
            })?,
            jd_tt: parse_f64(f[4], line)?,
            asc: parse_point(&f[5..11], line)?,
            dsc: parse_point(&f[11..17], line)?,
            peri: parse_point(&f[17..23], line)?,
            apo: parse_point(&f[23..29], line)?,
        });
    }
    Ok(rows)
}

/// Recomputes one corpus row, enforces Tier-1 self-consistency, and
/// accumulates its Tier-2 residuals into the row's category.
fn check_row<B: EphemerisBackend>(
    engine: &EventEngine<B>,
    row: &CorpusRow,
    m: &mut Measured,
) -> Result<(), NodApsError> {
    let label = row.label.as_str();
    let jd_tt = row.jd_tt;
    let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
    let NodesApsides {
        ascending,
        descending,
        perihelion,
        aphelion,
        ..
    } = engine
        .nod_aps(row.body.clone(), instant, row.method, row.convention)
        .map_err(|e| NodApsError::Engine(format!("{label} ({jd_tt}): {e}")))?;

    // ---- Tier 1: self-consistency (no SE reference) ----
    // Sun ascending/descending are excluded from `check_tier1`: the
    // engine now zeroes them (§R8), so its `distance > 0` invariant
    // would legitimately fail on a Sun row — they get their own exact-
    // zero assertion below instead.
    let is_sun = row.se_body == 0;
    if !is_sun {
        check_tier1(label, "ascending", &ascending)?;
        check_tier1(label, "descending", &descending)?;
    }
    check_tier1(label, "perihelion", &perihelion)?;
    check_tier1(label, "aphelion", &aphelion)?;

    // ---- Tier 2: SE parity residuals ----
    let track = m.category_mut(category_for(&row.body, row.method));
    if is_sun {
        // SE zeroes the Sun node columns (Earth elements have no node,
        // swecl.c:5436-5439) and the engine now matches (§R8, Task 9):
        // assert both sides are exactly zero rather than running the
        // normal residual-vs-ceiling comparison.
        assert_sun_node_zeroed(label, jd_tt, "ascending", &ascending, &row.asc)?;
        assert_sun_node_zeroed(label, jd_tt, "descending", &descending, &row.dsc)?;
    } else {
        accumulate_tier2(track, label, jd_tt, "ascending", &ascending, &row.asc);
        accumulate_tier2(track, label, jd_tt, "descending", &descending, &row.dsc);
    }
    accumulate_tier2(track, label, jd_tt, "perihelion", &perihelion, &row.peri);
    accumulate_tier2(track, label, jd_tt, "aphelion", &aphelion, &row.apo);
    Ok(())
}

/// Verifies a corpus CSV against its own manifest, parses it, and pins its
/// row count, all fail-closed.
fn load_corpus(
    manifest: &str,
    file: &'static str,
    csv: &str,
    expected: usize,
) -> Result<Vec<CorpusRow>, NodApsError> {
    check_checksum(manifest, file, csv)?;
    let rows = parse_rows(csv)?;
    if rows.len() != expected {
        return Err(NodApsError::RowCountMismatch {
            file,
            expected,
            got: rows.len(),
        });
    }
    Ok(rows)
}

/// Runs the checksum guards, parses both corpora, recomputes every row via a
/// freshly built production-style `EventEngine`, enforces Tier-1
/// self-consistency, and accumulates every Tier-2 residual maximum per
/// category. Numeric ceiling gating is NOT applied here (that is
/// [`validate_nod_aps_corpus`]'s job) — so this succeeds regardless of the
/// ceiling constants.
fn measure() -> Result<Measured, NodApsError> {
    let planet_rows = load_corpus(MANIFEST, CSV_FILE, CSV, EXPECTED_ROWS)?;
    let asteroid_rows = load_corpus(
        ASTEROID_MANIFEST,
        ASTEROID_CSV_FILE,
        ASTEROID_CSV,
        EXPECTED_ASTEROID_ROWS,
    )?;

    let backend = RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]);
    let engine = EventEngine::new(backend);
    let mut m = Measured::default();

    for row in &planet_rows {
        check_row(&engine, row, &mut m)?;
        m.rows += 1;
    }
    for row in &asteroid_rows {
        check_row(&engine, row, &mut m)?;
        m.asteroid_rows += 1;
    }
    Ok(m)
}

/// Tier-1 (self-consistency) only: checksum guard + per-row recompute +
/// finite/range invariants, with NO Tier-2 ceiling gating. Passes on the
/// committed corpus independently of the threshold constants.
pub fn run_nod_aps_tier1_only() -> Result<NodApsReport, NodApsError> {
    Ok(measure()?.into_report())
}

fn check_category(
    category: Category,
    track: &CategoryTrack,
    ceilings: (f64, f64, f64, f64),
) -> Result<(), NodApsError> {
    let (lon_ceiling, lat_ceiling, dist_ceiling, speed_ceiling) = ceilings;
    let checks: [(&'static str, &MetricMax, f64); 4] = [
        ("longitude_arcsec", &track.lon_arcsec, lon_ceiling),
        ("latitude_arcsec", &track.lat_arcsec, lat_ceiling),
        ("distance_rel", &track.dist_rel, dist_ceiling),
        ("lon_speed_deg_day", &track.lon_speed_deg_day, speed_ceiling),
    ];
    for (metric, tracked, ceiling) in checks {
        let residual = tracked.value;
        if !residual.is_finite() || residual > ceiling {
            return Err(NodApsError::ToleranceExceeded {
                category: category.name(),
                label: format!("{metric}: {}", tracked.label),
                jd: tracked.jd,
                residual,
                ceiling,
            });
        }
    }
    Ok(())
}

/// Full two-tier gate: Tier-1 self-consistency (via `measure`) plus Tier-2
/// SE parity gated under the measured ceilings in
/// `crate::nod_aps_thresholds`, per category. Fails closed on any exceeded
/// ceiling.
pub fn validate_nod_aps_corpus() -> Result<NodApsReport, NodApsError> {
    let m = measure()?;

    check_category(
        Category::MeanPlanet,
        &m.mean_planet,
        (
            MEAN_PLANET_LONGITUDE_ARCSEC,
            MEAN_PLANET_LATITUDE_ARCSEC,
            MEAN_PLANET_DISTANCE_REL,
            MEAN_PLANET_LON_SPEED_DEG_DAY,
        ),
    )?;
    check_category(
        Category::MeanMoon,
        &m.mean_moon,
        (
            MEAN_MOON_LONGITUDE_ARCSEC,
            MEAN_MOON_LATITUDE_ARCSEC,
            MEAN_MOON_DISTANCE_REL,
            MEAN_MOON_LON_SPEED_DEG_DAY,
        ),
    )?;
    check_category(
        Category::OscuPlanet,
        &m.oscu_planet,
        (
            OSCU_PLANET_LONGITUDE_ARCSEC,
            OSCU_PLANET_LATITUDE_ARCSEC,
            OSCU_PLANET_DISTANCE_REL,
            OSCU_PLANET_LON_SPEED_DEG_DAY,
        ),
    )?;
    check_category(
        Category::OscuMoon,
        &m.oscu_moon,
        (
            OSCU_MOON_LONGITUDE_ARCSEC,
            OSCU_MOON_LATITUDE_ARCSEC,
            OSCU_MOON_DISTANCE_REL,
            OSCU_MOON_LON_SPEED_DEG_DAY,
        ),
    )?;
    check_category(
        Category::OscuAsteroid,
        &m.oscu_asteroid,
        (
            OSCU_ASTEROID_LONGITUDE_ARCSEC,
            OSCU_ASTEROID_LATITUDE_ARCSEC,
            OSCU_ASTEROID_DISTANCE_REL,
            OSCU_ASTEROID_LON_SPEED_DEG_DAY,
        ),
    )?;

    Ok(m.into_report())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_row_count_is_pinned() {
        let report = run_nod_aps_tier1_only().expect("tier1 passes");
        assert_eq!(report.rows, EXPECTED_ROWS);
        assert_eq!(report.asteroid_rows, EXPECTED_ASTEROID_ROWS);
        assert!(report.passed());
    }

    #[test]
    fn checksum_drift_fails_closed() {
        assert!(check_checksum(MANIFEST, "nod-aps.csv", "mutated,body\n").is_err());
    }

    #[test]
    fn gate_passes_on_committed_corpus() {
        validate_nod_aps_corpus().expect("nod-aps gate passes");
    }

    #[test]
    fn unrecognized_se_body_fails_closed() {
        assert!(body_from_se(15).is_none(), "Chiron is not in the corpus");
        assert!(body_from_se(16).is_none(), "Pholus is not in the corpus");
        assert!(body_from_se(21).is_none());
        assert!(body_from_se(40).is_none());
        assert!(body_from_se(58).is_none());
    }

    #[test]
    fn asteroid_se_bodies_map_to_ceres_through_vesta() {
        assert_eq!(body_from_se(17), Some(CelestialBody::Ceres));
        assert_eq!(body_from_se(18), Some(CelestialBody::Pallas));
        assert_eq!(body_from_se(19), Some(CelestialBody::Juno));
        assert_eq!(body_from_se(20), Some(CelestialBody::Vesta));
    }

    #[test]
    fn asteroid_rows_parse_and_count_32() {
        let rows = parse_rows(ASTEROID_CSV).expect("asteroid corpus parses");
        assert_eq!(rows.len(), EXPECTED_ASTEROID_ROWS);
        assert_eq!(EXPECTED_ASTEROID_ROWS, 32);
        assert!(rows.iter().all(|r| r.method == NodApsMethod::Osculating));
    }

    #[test]
    fn asteroid_corpus_tampering_fails_closed() {
        let tampered = ASTEROID_CSV.replacen("Ceres,17,", "Ceres,17, ", 1);
        assert_ne!(tampered, ASTEROID_CSV);
        assert!(matches!(
            check_checksum(ASTEROID_MANIFEST, ASTEROID_CSV_FILE, &tampered),
            Err(NodApsError::ChecksumMismatch { .. })
        ));
        check_checksum(ASTEROID_MANIFEST, ASTEROID_CSV_FILE, ASTEROID_CSV)
            .expect("committed asteroid corpus matches its manifest");
    }

    #[test]
    fn asteroid_rows_fall_in_the_asteroid_category() {
        for body in [
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
        ] {
            assert_eq!(
                category_for(&body, NodApsMethod::Osculating),
                Category::OscuAsteroid
            );
        }
    }

    #[test]
    fn asteroid_row_count_drift_fails_closed() {
        // Drop the last data line and re-pin the checksum, so only the row
        // count can reject it.
        let trimmed = ASTEROID_CSV.trim_end_matches('\n');
        let (csv, _) = trimmed.rsplit_once('\n').expect("multi-line corpus");
        let csv = format!("{csv}\n");
        let manifest = format!(
            "file: {ASTEROID_CSV_FILE} rows={} checksum={}\n",
            EXPECTED_ASTEROID_ROWS - 1,
            fnv1a64(&csv)
        );
        assert!(matches!(
            load_corpus(&manifest, ASTEROID_CSV_FILE, &csv, EXPECTED_ASTEROID_ROWS),
            Err(NodApsError::RowCountMismatch {
                file: "asteroids.csv",
                ..
            })
        ));
    }

    #[test]
    fn asteroid_ceiling_breach_fails_closed() {
        let mut track = CategoryTrack::default();
        track.lon_arcsec.observe(39.0, "Ceres", 2_451_545.0);
        let result = check_category(
            Category::OscuAsteroid,
            &track,
            (
                OSCU_ASTEROID_LONGITUDE_ARCSEC,
                OSCU_ASTEROID_LATITUDE_ARCSEC,
                OSCU_ASTEROID_DISTANCE_REL,
                OSCU_ASTEROID_LON_SPEED_DEG_DAY,
            ),
        );
        assert!(matches!(
            result,
            Err(NodApsError::ToleranceExceeded {
                category: "OSCU_ASTEROID",
                ..
            })
        ));
    }
}
