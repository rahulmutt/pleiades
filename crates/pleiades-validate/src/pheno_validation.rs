//! Fail-closed two-tier `validate-pheno` gate over the committed
//! Swiss-Ephemeris `swe_pheno` reference corpus (Task 5,
//! `data/pheno-corpus/{pheno.csv,manifest.txt}`).
//!
//! Tier 1 (self-consistency, no SE reference): every row is recomputed with
//! `pleiades_events::EventEngine::pheno` over the production-style backend
//! chain (`PackagedDataBackend`, `CompositeBackend(Vsop87, Elp)`,
//! `JplSnapshotBackend`, `FictitiousBackend`) and every recomputed row is
//! checked for internal consistency: finite outputs, phase angle in
//! `[0, 180]`, illuminated fraction in `[0, 1]`, elongation in `[0, 180]`,
//! diameter `>= 0`, and an apparent magnitude present for all ten majors.
//!
//! Tier 2 (SE parity): each recomputed row is compared against the SE
//! `swe_pheno` reference columns — phase-angle residual (arcsec), phase
//! (illuminated-fraction) residual (absolute), elongation residual (arcsec),
//! apparent-diameter residual (arcsec), and apparent-magnitude residual
//! (absolute, Saturn bucketed separately since its ring term is the widest)
//! — accumulated into per-metric maxima. `validate_pheno_corpus` gates every
//! metric's maxima fail-closed under `crate::pheno_thresholds`'s PROVISIONAL
//! (deliberately generous) ceilings; Task 7 measures the true maxima and
//! tightens them.
//!
//! Corpus scope (see the committed `manifest.txt` and the CSV's header
//! comment): SE 0-9 classical planets only (Sun, Moon, Mercury..Pluto).
//! Small bodies and fictitious bodies have no SE `swe_pheno` magnitude model
//! at all (`pleiades_events::magnitude::apparent_magnitude` returns `None`
//! for them) — the engine still serves them geometric phase/phase-angle/
//! elongation/diameter, but that path is entirely gate-unreferenced. See
//! `summary_line`'s coverage-bound sentence.
//!
//! Sun rows are a special case (§E2): SE structurally leaves the Sun's
//! phase/phase-angle/elongation attributes at zero (it skips the phase
//! blocks for the Sun in `swe_pheno`), and the engine matches
//! (`pleiades_events::pheno::EventEngine::pheno` returns zero for all three
//! on the Sun). So for `se_body == 0` rows this gate skips the normal
//! Tier-2 residual accumulation for phase angle/phase/elongation and instead
//! asserts BOTH sides are exactly zero — failing closed on any nonzero
//! value. The Sun's diameter and magnitude stay on the normal Tier-1 +
//! Tier-2 path.
//!
//! A sibling `manifest.txt` records the fnv1a64 digest of the CSV (drift
//! guard); a mismatch fails the gate closed.

use crate::pheno_thresholds::*;
use pleiades_apparent::fnv1a64;
use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{EventEngine, PhenoData};
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;
use std::collections::BTreeMap;

const CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/pheno-corpus/pheno.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/pheno-corpus/manifest.txt"
));
const CSV_FILE: &str = "pheno.csv";

/// Fixture row count pinned by the corpus (Task 5: 10 majors x 8 epochs).
/// Update when the corpus is regenerated.
pub const EXPECTED_ROWS: usize = 80;

#[derive(Debug)]
pub enum PhenoError {
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
        metric: &'static str,
        label: String,
        jd: f64,
        residual: f64,
        ceiling: f64,
    },
    /// Malformed manifest or corpus row (also covers a Tier-1
    /// self-consistency invariant failing on the recomputed row, the Sun
    /// exact-zero assertion failing, and an unrecognized `se_body` value —
    /// the parse is fail-closed since only SE 0-9 exist in the committed
    /// corpus).
    Parse { row: String },
    /// The engine errored while recomputing a row.
    Engine(String),
}

impl std::fmt::Display for PhenoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PhenoError {}

/// Summary of the measured per-metric maxima and checked-row count for the
/// gate.
#[derive(Debug, Default)]
pub struct PhenoReport {
    pub rows: usize,
    pub max_phase_angle_arcsec: f64,
    pub max_phase_abs: f64,
    pub max_elongation_arcsec: f64,
    pub max_diameter_arcsec: f64,
    pub max_magnitude_abs: f64,
    pub max_saturn_magnitude_abs: f64,
}

impl PhenoReport {
    /// The gate passed iff every committed row was checked (a silently
    /// truncated corpus is a failure, not a pass). Every ceiling is enforced
    /// fail-closed by [`validate_pheno_corpus`], so reaching a report already
    /// implies every checked row was within ceiling.
    pub fn passed(&self) -> bool {
        self.rows == EXPECTED_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-pheno: {} rows — max residuals: phase_angle {:.3}\" phase {:.4} elongation {:.3}\" diameter {:.3}\" magnitude {:.3} saturn_magnitude {:.3} — coverage bound: magnitude covers the ten majors only (asteroids/fictitious geometric-only, gate-unreferenced)",
            self.rows,
            self.max_phase_angle_arcsec,
            self.max_phase_abs,
            self.max_elongation_arcsec,
            self.max_diameter_arcsec,
            self.max_magnitude_abs,
            self.max_saturn_magnitude_abs,
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

/// All measured residual maxima over the committed corpus. Tier-1
/// self-consistency is enforced during measurement (it never depends on the
/// numeric ceilings); ceiling gating is applied afterwards by
/// [`validate_pheno_corpus`].
#[derive(Debug, Default)]
struct Measured {
    rows: usize,
    phase_angle: MetricMax,
    phase: MetricMax,
    elongation: MetricMax,
    diameter: MetricMax,
    magnitude: MetricMax,
    saturn_magnitude: MetricMax,
}

impl Measured {
    fn into_report(self) -> PhenoReport {
        PhenoReport {
            rows: self.rows,
            max_phase_angle_arcsec: self.phase_angle.value,
            max_phase_abs: self.phase.value,
            max_elongation_arcsec: self.elongation.value,
            max_diameter_arcsec: self.diameter.value,
            max_magnitude_abs: self.magnitude.value,
            max_saturn_magnitude_abs: self.saturn_magnitude.value,
        }
    }
}

/// SE body number -> `CelestialBody`. Only 0 (Sun), 1 (Moon), and 2..=9
/// (Mercury..Pluto) exist in the committed corpus (majors only, per the
/// Task-5 corpus scope). Any other value is a fail-closed parse error.
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

fn parse_manifest() -> Result<BTreeMap<String, (usize, u64)>, PhenoError> {
    let mut map = BTreeMap::new();
    for line in MANIFEST.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let toks: Vec<&str> = rest.split_whitespace().collect();
        if toks.len() < 3 {
            return Err(PhenoError::Parse {
                row: format!("malformed file line: {line}"),
            });
        }
        let name = toks[0].to_string();
        let mut rows = None;
        let mut checksum = None;
        for tok in &toks[1..] {
            if let Some(v) = tok.strip_prefix("rows=") {
                rows = Some(v.parse::<usize>().map_err(|e| PhenoError::Parse {
                    row: format!("rows: {e}"),
                })?);
            } else if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(v.parse::<u64>().map_err(|e| PhenoError::Parse {
                    row: format!("checksum: {e}"),
                })?);
            }
        }
        let rows = rows.ok_or_else(|| PhenoError::Parse {
            row: format!("rows= missing: {line}"),
        })?;
        let checksum = checksum.ok_or_else(|| PhenoError::Parse {
            row: format!("checksum= missing: {line}"),
        })?;
        map.insert(name, (rows, checksum));
    }
    if map.is_empty() {
        return Err(PhenoError::Parse {
            row: "no `file:` lines found in manifest".to_string(),
        });
    }
    Ok(map)
}

/// Looks up `file` in the manifest and compares `fnv1a64(csv)` against the
/// recorded checksum, fail-closed. Returns the manifest's declared row count
/// on success.
fn check_checksum(file: &'static str, csv: &str) -> Result<usize, PhenoError> {
    let manifest = parse_manifest()?;
    let (rows, want) = *manifest.get(file).ok_or_else(|| PhenoError::Parse {
        row: format!("manifest missing entry for {file}"),
    })?;
    let got = fnv1a64(csv);
    if got != want {
        return Err(PhenoError::ChecksumMismatch { file, got, want });
    }
    Ok(rows)
}

fn parse_f64(s: &str, row: &str) -> Result<f64, PhenoError> {
    s.trim().parse::<f64>().map_err(|_| PhenoError::Parse {
        row: row.to_string(),
    })
}

fn parse_i64(s: &str, row: &str) -> Result<i64, PhenoError> {
    s.trim().parse::<i64>().map_err(|_| PhenoError::Parse {
        row: row.to_string(),
    })
}

/// Asserts a Sun row's phase/phase-angle/elongation are EXACTLY zeroed on
/// both sides: the recomputed [`PhenoData`] and the SE corpus columns. SE
/// structurally leaves these at zero for the Sun (§E2), and the engine
/// matches — any nonzero value on either side means the zeroing broke or the
/// corpus drifted, so this fails closed rather than silently comparing
/// residuals against a zero reference.
fn assert_sun_phase_zeroed(
    label: &str,
    jd: f64,
    engine: &PhenoData,
    se_phase_angle: f64,
    se_phase: f64,
    se_elongation: f64,
) -> Result<(), PhenoError> {
    let engine_zero = engine.phase_angle_deg == 0.0
        && engine.phase_fraction == 0.0
        && engine.elongation_deg == 0.0;
    let se_zero = se_phase_angle == 0.0 && se_phase == 0.0 && se_elongation == 0.0;
    if !engine_zero || !se_zero {
        return Err(PhenoError::Parse {
            row: format!(
                "{label} (jd {jd}): Sun phase not exactly zeroed — engine phase_angle={} phase={} elongation={}, se phase_angle={se_phase_angle} phase={se_phase} elongation={se_elongation}",
                engine.phase_angle_deg, engine.phase_fraction, engine.elongation_deg
            ),
        });
    }
    Ok(())
}

/// Checks Tier-1 self-consistency for one recomputed row. Returns an error
/// identifying the offending row on failure.
fn check_tier1(label: &str, p: &PhenoData) -> Result<(), PhenoError> {
    if !(p.phase_angle_deg.is_finite()
        && p.phase_fraction.is_finite()
        && p.elongation_deg.is_finite()
        && p.apparent_diameter_deg.is_finite())
    {
        return Err(PhenoError::Parse {
            row: format!("{label}: non-finite output {p:?}"),
        });
    }
    if !(0.0..=180.0).contains(&p.phase_angle_deg) {
        return Err(PhenoError::Parse {
            row: format!("{label}: phase_angle out of [0,180]: {}", p.phase_angle_deg),
        });
    }
    if !(0.0..=1.0).contains(&p.phase_fraction) {
        return Err(PhenoError::Parse {
            row: format!("{label}: phase out of [0,1]: {}", p.phase_fraction),
        });
    }
    if !(0.0..=180.0).contains(&p.elongation_deg) {
        return Err(PhenoError::Parse {
            row: format!("{label}: elongation out of [0,180]: {}", p.elongation_deg),
        });
    }
    if p.apparent_diameter_deg < 0.0 {
        return Err(PhenoError::Parse {
            row: format!("{label}: diameter not >= 0: {}", p.apparent_diameter_deg),
        });
    }
    if p.apparent_magnitude.is_none() {
        return Err(PhenoError::Parse {
            row: format!("{label}: magnitude missing for a major body"),
        });
    }
    Ok(())
}

/// Runs the checksum guard, parses the corpus, recomputes every row via a
/// freshly built production-style `EventEngine`, enforces Tier-1
/// self-consistency, and accumulates every Tier-2 residual maximum. Numeric
/// ceiling gating is NOT applied here (that is [`validate_pheno_corpus`]'s
/// job) — so this succeeds regardless of the ceiling constants.
fn measure() -> Result<Measured, PhenoError> {
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
        if f.len() != 8 {
            return Err(PhenoError::Parse {
                row: line.to_string(),
            });
        }
        let label = f[0];
        let se_body = parse_i64(f[1], line)?;
        let jd_tt = parse_f64(f[2], line)?;
        let se_phase_angle = parse_f64(f[3], line)?;
        let se_phase = parse_f64(f[4], line)?;
        let se_elongation = parse_f64(f[5], line)?;
        let se_diameter_deg = parse_f64(f[6], line)?;
        let se_magnitude = parse_f64(f[7], line)?;

        let body = body_from_se(se_body).ok_or_else(|| PhenoError::Parse {
            row: format!("{line} (unrecognized se_body {se_body})"),
        })?;

        let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tdb);
        let data = engine
            .pheno(body.clone(), instant)
            .map_err(|e| PhenoError::Engine(format!("{label} ({jd_tt}): {e}")))?;

        // ---- Tier 1: self-consistency (no SE reference) ----
        check_tier1(label, &data)?;

        // ---- Tier 2: SE parity residuals ----
        let is_sun = se_body == 0;
        if is_sun {
            // SE leaves the Sun's phase/phase-angle/elongation attributes at
            // zero (§E2) and the engine matches: assert both sides are
            // exactly zero rather than running the normal residual-vs-
            // ceiling comparison.
            assert_sun_phase_zeroed(label, jd_tt, &data, se_phase_angle, se_phase, se_elongation)?;
        } else {
            let phase_angle_res = (data.phase_angle_deg - se_phase_angle).abs() * 3600.0;
            let phase_res = (data.phase_fraction - se_phase).abs();
            let elongation_res = (data.elongation_deg - se_elongation).abs() * 3600.0;
            m.phase_angle.observe(phase_angle_res, label, jd_tt);
            m.phase.observe(phase_res, label, jd_tt);
            m.elongation.observe(elongation_res, label, jd_tt);
        }

        let diameter_res = (data.apparent_diameter_deg - se_diameter_deg).abs() * 3600.0;
        m.diameter.observe(diameter_res, label, jd_tt);

        let magnitude = data.apparent_magnitude.ok_or_else(|| PhenoError::Parse {
            row: format!("{label}: magnitude missing for a major body"),
        })?;
        let magnitude_res = (magnitude - se_magnitude).abs();
        if body == CelestialBody::Saturn {
            m.saturn_magnitude.observe(magnitude_res, label, jd_tt);
        } else {
            m.magnitude.observe(magnitude_res, label, jd_tt);
        }

        m.rows += 1;
    }

    if m.rows != EXPECTED_ROWS {
        return Err(PhenoError::RowCountMismatch {
            expected: EXPECTED_ROWS,
            got: m.rows,
        });
    }
    Ok(m)
}

/// Tier-1 (self-consistency) only: checksum guard + per-row recompute +
/// finite/range invariants, with NO Tier-2 ceiling gating. Passes on the
/// committed corpus independently of the threshold constants.
pub fn run_pheno_tier1_only() -> Result<PhenoReport, PhenoError> {
    Ok(measure()?.into_report())
}

fn check_metric(metric: &'static str, tracked: &MetricMax, ceiling: f64) -> Result<(), PhenoError> {
    let residual = tracked.value;
    if !residual.is_finite() || residual > ceiling {
        return Err(PhenoError::ToleranceExceeded {
            metric,
            label: tracked.label.clone(),
            jd: tracked.jd,
            residual,
            ceiling,
        });
    }
    Ok(())
}

/// Full two-tier gate: Tier-1 self-consistency (via `measure`) plus Tier-2
/// SE parity gated under the provisional ceilings in
/// `crate::pheno_thresholds`. Fails closed on any exceeded ceiling.
pub fn validate_pheno_corpus() -> Result<PhenoReport, PhenoError> {
    let m = measure()?;

    check_metric("phase_angle_arcsec", &m.phase_angle, PHASE_ANGLE_ARCSEC)?;
    check_metric("phase_abs", &m.phase, PHASE_FRACTION_ABS)?;
    check_metric("elongation_arcsec", &m.elongation, ELONGATION_ARCSEC)?;
    check_metric("diameter_arcsec", &m.diameter, DIAMETER_ARCSEC)?;
    check_metric("magnitude_abs", &m.magnitude, MAGNITUDE_ABS)?;
    check_metric(
        "saturn_magnitude_abs",
        &m.saturn_magnitude,
        SATURN_MAGNITUDE_ABS,
    )?;

    Ok(m.into_report())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_row_count_is_pinned() {
        let report = run_pheno_tier1_only().expect("tier1 passes");
        assert_eq!(report.rows, EXPECTED_ROWS);
    }

    #[test]
    fn checksum_drift_fails_closed() {
        assert!(check_checksum("pheno.csv", "mutated,body\n").is_err());
    }

    #[test]
    fn gate_passes_on_committed_corpus() {
        validate_pheno_corpus().expect("pheno gate passes");
    }

    #[test]
    fn unrecognized_se_body_fails_closed() {
        assert!(body_from_se(15).is_none());
        assert!(body_from_se(20).is_none());
        assert!(body_from_se(40).is_none());
        assert!(body_from_se(58).is_none());
    }
}
