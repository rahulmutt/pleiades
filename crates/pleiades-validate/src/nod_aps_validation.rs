//! Fail-closed two-tier `validate-nod-aps` gate over the committed
//! Swiss-Ephemeris `swe_nod_aps` reference corpus (Task 8,
//! `data/nod-aps-corpus/{nod-aps.csv,manifest.txt}`).
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
//! `MEAN_MOON`, `OSCU_PLANET`, `OSCU_MOON`: mean vs osculating (methods 2 and
//! 4, heliocentric and barycentric, both count as osculating per the plan),
//! Moon vs everything else (Sun and the eight classical planets share
//! `*_PLANET`). `validate_nod_aps_corpus` gates every category's maxima
//! fail-closed under `crate::nod_aps_thresholds`'s provisional ceilings
//! (tightened in Task 9 once the true maxima are measured).
//!
//! Corpus scope (see the committed `manifest.txt` and the CSV's header
//! comment): SE 0-9 classical planets only (the mean set drops Pluto, SE 9,
//! which has no SE mean elements). Small bodies (SE 15-20) and fictitious
//! bodies (SE 40-58) are OUT OF SCOPE — this SE build's `swe_nod_aps` does
//! not implement fictitious bodies at all, and the reference generator never
//! sampled small bodies. See `summary_line`'s coverage-bound sentence.
//!
//! Sun rows are a special case: SE structurally zeroes the Sun's asc/dsc
//! columns (all 18 fields across both points, for both methods present in
//! the corpus) because the Earth-orbit "node" has no defined ascending/
//! descending crossing in SE's convention (`swecl.c`). Our engine does not
//! yet zero its Sun-node output to match (that lands in Task 9, per plan
//! §R8), so for `se_body == 0` rows this gate runs Tier-1 invariants on all
//! four recomputed points as usual but restricts Tier-2 residual comparison
//! to the perihelion and aphelion points only.
//!
//! A sibling `manifest.txt` records the fnv1a64 digest of the CSV (drift
//! guard); a mismatch fails the gate closed.

use crate::nod_aps_thresholds::*;
use pleiades_apparent::fnv1a64;
use pleiades_backend::{CompositeBackend, RoutingBackend};
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

/// Fixture row count pinned by the corpus (Task 7: 72 mean + 6
/// mean-fopoint + 80 osculating + 20 barycentric + 6 oscu-fopoint). Update
/// when the corpus is regenerated.
pub const EXPECTED_ROWS: usize = 184;

#[derive(Debug)]
pub enum NodApsError {
    /// A committed CSV's digest disagrees with the manifest.
    ChecksumMismatch {
        file: &'static str,
        got: u64,
        want: u64,
    },
    /// The parsed corpus row count disagrees with [`EXPECTED_ROWS`].
    RowCountMismatch { expected: usize, got: usize },
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
    /// fail-closed since only SE 0-9 / methods 1,2,4 / fopoint 0,1 exist in
    /// the committed corpus).
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
    pub rows: usize,
    pub mean_planet: CategoryMaxima,
    pub mean_moon: CategoryMaxima,
    pub oscu_planet: CategoryMaxima,
    pub oscu_moon: CategoryMaxima,
}

impl NodApsReport {
    /// The gate passed iff every committed row was checked (a silently
    /// truncated corpus is a failure, not a pass). Every ceiling is enforced
    /// fail-closed by [`validate_nod_aps_corpus`], so reaching a report
    /// already implies every checked row was within ceiling.
    pub fn passed(&self) -> bool {
        self.rows == EXPECTED_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-nod-aps: {} rows — max residuals: MEAN_PLANET lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day; MEAN_MOON lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day; OSCU_PLANET lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day; OSCU_MOON lon {:.3}\" lat {:.3}\" dist {:.2e} rel speed {:.4} deg/day — coverage bound: SE swe_nod_aps implements neither fictitious bodies (upstream-disabled) nor offline small-body sampling; fictitious/asteroid nod_aps is engine-covered, gate-unreferenced",
            self.rows,
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

/// All measured residual maxima over the committed corpus, split into the
/// four categories so each can be gated under its own ceilings. Tier-1
/// self-consistency is enforced during measurement (it never depends on the
/// numeric ceilings); ceiling gating is applied afterwards by
/// [`validate_nod_aps_corpus`].
#[derive(Debug, Default)]
struct Measured {
    rows: usize,
    mean_planet: CategoryTrack,
    mean_moon: CategoryTrack,
    oscu_planet: CategoryTrack,
    oscu_moon: CategoryTrack,
}

impl Measured {
    fn into_report(self) -> NodApsReport {
        NodApsReport {
            rows: self.rows,
            mean_planet: self.mean_planet.to_maxima(),
            mean_moon: self.mean_moon.to_maxima(),
            oscu_planet: self.oscu_planet.to_maxima(),
            oscu_moon: self.oscu_moon.to_maxima(),
        }
    }

    fn category_mut(&mut self, category: Category) -> &mut CategoryTrack {
        match category {
            Category::MeanPlanet => &mut self.mean_planet,
            Category::MeanMoon => &mut self.mean_moon,
            Category::OscuPlanet => &mut self.oscu_planet,
            Category::OscuMoon => &mut self.oscu_moon,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Category {
    MeanPlanet,
    MeanMoon,
    OscuPlanet,
    OscuMoon,
}

impl Category {
    fn name(self) -> &'static str {
        match self {
            Category::MeanPlanet => "MEAN_PLANET",
            Category::MeanMoon => "MEAN_MOON",
            Category::OscuPlanet => "OSCU_PLANET",
            Category::OscuMoon => "OSCU_MOON",
        }
    }
}

/// SE body number -> `CelestialBody`. Only 0 (Sun), 1 (Moon), and 2..=9
/// (Mercury..Pluto) exist in the committed corpus (controller-mandated scope
/// cut: no small bodies, no fictitious bodies). Any other value is a
/// fail-closed parse error.
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

fn parse_manifest() -> Result<BTreeMap<String, (usize, u64)>, NodApsError> {
    let mut map = BTreeMap::new();
    for line in MANIFEST.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let toks: Vec<&str> = rest.split_whitespace().collect();
        if toks.len() < 3 {
            return Err(NodApsError::Parse {
                row: format!("malformed file line: {line}"),
            });
        }
        let name = toks[0].to_string();
        let mut rows = None;
        let mut checksum = None;
        for tok in &toks[1..] {
            if let Some(v) = tok.strip_prefix("rows=") {
                rows = Some(v.parse::<usize>().map_err(|e| NodApsError::Parse {
                    row: format!("rows: {e}"),
                })?);
            } else if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(v.parse::<u64>().map_err(|e| NodApsError::Parse {
                    row: format!("checksum: {e}"),
                })?);
            }
        }
        let rows = rows.ok_or_else(|| NodApsError::Parse {
            row: format!("rows= missing: {line}"),
        })?;
        let checksum = checksum.ok_or_else(|| NodApsError::Parse {
            row: format!("checksum= missing: {line}"),
        })?;
        map.insert(name, (rows, checksum));
    }
    if map.is_empty() {
        return Err(NodApsError::Parse {
            row: "no `file:` lines found in manifest".to_string(),
        });
    }
    Ok(map)
}

/// Looks up `file` in the manifest and compares `fnv1a64(csv)` against the
/// recorded checksum, fail-closed. Returns the manifest's declared row count
/// on success.
fn check_checksum(file: &'static str, csv: &str) -> Result<usize, NodApsError> {
    let manifest = parse_manifest()?;
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
/// ddist.
#[derive(Clone, Copy, Debug)]
struct SePoint {
    lon: f64,
    lat: f64,
    dist: f64,
    dlon: f64,
}

fn parse_point(f: &[&str], row: &str) -> Result<SePoint, NodApsError> {
    debug_assert_eq!(f.len(), 6);
    Ok(SePoint {
        lon: parse_f64(f[0], row)?,
        lat: parse_f64(f[1], row)?,
        dist: parse_f64(f[2], row)?,
        dlon: parse_f64(f[3], row)?,
        // dlat (f[4]) and ddist (f[5]) are parsed for schema completeness but
        // not compared — only longitude speed is gated (per plan/brief).
    })
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

/// Runs the checksum guard, parses the corpus, recomputes every row via a
/// freshly built production-style `EventEngine`, enforces Tier-1
/// self-consistency, and accumulates every Tier-2 residual maximum per
/// category. Numeric ceiling gating is NOT applied here (that is
/// [`validate_nod_aps_corpus`]'s job) — so this succeeds regardless of the
/// ceiling constants.
fn measure() -> Result<Measured, NodApsError> {
    check_checksum(CSV_FILE, CSV)?;

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

    for line in CSV.lines() {
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
        let label = f[0];
        let se_body = parse_i64(f[1], line)?;
        let se_method = parse_i64(f[2], line)?;
        let se_fopoint = parse_i64(f[3], line)?;
        let jd_tt = parse_f64(f[4], line)?;
        let asc = parse_point(&f[5..11], line)?;
        let dsc = parse_point(&f[11..17], line)?;
        let peri = parse_point(&f[17..23], line)?;
        let apo = parse_point(&f[23..29], line)?;

        let body = body_from_se(se_body).ok_or_else(|| NodApsError::Parse {
            row: format!("{line} (unrecognized se_body {se_body})"),
        })?;
        let method = method_from_se(se_method).ok_or_else(|| NodApsError::Parse {
            row: format!("{line} (unrecognized method {se_method})"),
        })?;
        let convention = convention_from_fopoint(se_fopoint).ok_or_else(|| NodApsError::Parse {
            row: format!("{line} (unrecognized fopoint {se_fopoint})"),
        })?;

        let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
        let NodesApsides {
            ascending,
            descending,
            perihelion,
            aphelion,
            ..
        } = engine
            .nod_aps(body.clone(), instant, method, convention)
            .map_err(|e| NodApsError::Engine(format!("{label} ({jd_tt}): {e}")))?;

        // ---- Tier 1: self-consistency (no SE reference), all four points ----
        check_tier1(label, "ascending", &ascending)?;
        check_tier1(label, "descending", &descending)?;
        check_tier1(label, "perihelion", &perihelion)?;
        check_tier1(label, "aphelion", &aphelion)?;

        // ---- Tier 2: SE parity residuals ----
        let category = category_for(&body, method);
        let track = m.category_mut(category);
        let is_sun = se_body == 0;
        if !is_sun {
            // SE zeroes the Sun node columns (Earth elements have no node,
            // swecl.c); engine-side zeroing + exact node comparison lands
            // with Task 9 (§R8).
            accumulate_tier2(track, label, jd_tt, "ascending", &ascending, &asc);
            accumulate_tier2(track, label, jd_tt, "descending", &descending, &dsc);
        }
        accumulate_tier2(track, label, jd_tt, "perihelion", &perihelion, &peri);
        accumulate_tier2(track, label, jd_tt, "aphelion", &aphelion, &apo);

        m.rows += 1;
    }

    if m.rows != EXPECTED_ROWS {
        return Err(NodApsError::RowCountMismatch {
            expected: EXPECTED_ROWS,
            got: m.rows,
        });
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
/// SE parity gated under the provisional ceilings in
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

    Ok(m.into_report())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_row_count_is_pinned() {
        let report = run_nod_aps_tier1_only().expect("tier1 passes");
        assert_eq!(report.rows, EXPECTED_ROWS);
    }

    #[test]
    fn checksum_drift_fails_closed() {
        assert!(check_checksum("nod-aps.csv", "mutated,body\n").is_err());
    }

    #[test]
    fn gate_passes_on_committed_corpus() {
        validate_nod_aps_corpus().expect("nod-aps gate passes");
    }

    #[test]
    fn unrecognized_se_body_fails_closed() {
        assert!(body_from_se(15).is_none());
        assert!(body_from_se(20).is_none());
        assert!(body_from_se(40).is_none());
        assert!(body_from_se(58).is_none());
    }
}
