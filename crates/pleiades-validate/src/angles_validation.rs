//! SE-parity gate for the SP-1 angles & sidereal-time layer.
//!
//! The gate validates two independent computations against a committed Swiss
//! Ephemeris reference corpus (`data/houses-corpus/angles.csv`):
//!
//! 1. **Sidereal-time parity** — `pleiades_apparent::sidereal::sidereal_time`
//!    local-apparent / GAST vs SE `armc` and `sidtime_gast_hours`. This exercises
//!    the Task-1 GMST + equation-of-equinoxes math.
//! 2. **Geometry parity** — `pleiades_houses::chart_points_from_armc`, fed SE's
//!    own `armc` (isolating the geometry from any sidereal-time delta), vs SE's
//!    `vertex`, `equatorial_ascendant`, `coasc_koch`, `coasc_munkasey`,
//!    `polar_asc`. This exercises the Task-3 `asc_mc_from` chart-point formulas
//!    ported from `swehouse.c`.
//!
//! The gate fails closed on missing rows, checksum drift, schema drift, or any
//! residual over its per-point arcsecond ceiling. It requires no Swiss Ephemeris
//! at run time — a clean checkout validates the committed values tool-free.

#![forbid(unsafe_code)]

use core::fmt;

use pleiades_core::{Angle, JulianDay, Latitude, Longitude};
use pleiades_houses::chart_points_from_armc;

const ANGLES_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/houses-corpus/angles.csv"
));

const ANGLES_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/houses-corpus/manifest.txt"
));

// ── Per-point arcsecond ceilings ──────────────────────────────────────────────
//
// Set from the measured residuals over the committed corpus (see the Task-7
// report). The sidereal-time-derived quantities (ARMC / local-apparent sidereal
// time and GAST) carry the ~0.16" delta between pleiades' GMST + equation-of-
// equinoxes model and Swiss Ephemeris, so they use the repo-standard 1.0" angle
// ceiling. The `swehouse.c`-ported geometry points (vertex, equatorial ascendant,
// both co-ascendants, polar ascendant) all measure < 0.05" once `asc_mc_from`
// transcribes the exact SE formulas, so they use a tight 0.5" ceiling (≈10×
// headroom over the measured max for cross-platform float determinism).
//
// Measured maxima over the committed corpus (arcseconds):
//   armc 0.1628, gast 0.1638, vertex 0.0076, equasc 0.0046,
//   coasc-koch 0.0050, coasc-munkasey 0.0487, polar-asc 0.0050.

/// Ceiling for local-apparent sidereal time / ARMC parity, arcseconds.
const CEIL_ARMC_ARCSEC: f64 = 1.0;
/// Ceiling for Greenwich apparent sidereal time (GAST) parity, arcseconds.
const CEIL_GAST_ARCSEC: f64 = 1.0;
/// Ceiling for the vertex, arcseconds.
const CEIL_VERTEX_ARCSEC: f64 = 0.5;
/// Ceiling for the equatorial ascendant (East Point), arcseconds.
const CEIL_EQUASC_ARCSEC: f64 = 0.5;
/// Ceiling for the Koch co-ascendant, arcseconds.
const CEIL_COASC_KOCH_ARCSEC: f64 = 0.5;
/// Ceiling for the Munkasey co-ascendant, arcseconds.
const CEIL_COASC_MUNKASEY_ARCSEC: f64 = 0.5;
/// Ceiling for the Munkasey polar ascendant, arcseconds.
const CEIL_POLAR_ASC_ARCSEC: f64 = 0.5;

/// A single parsed row from the angles corpus CSV.
#[derive(Clone, Debug, PartialEq)]
struct AnglesRow {
    chart_id: String,
    jd_ut: f64,
    lat_deg: f64,
    lon_deg: f64,
    armc: f64,
    vertex: f64,
    equatorial_ascendant: f64,
    coasc_koch: f64,
    coasc_munkasey: f64,
    polar_asc: f64,
    sidtime_gast_hours: f64,
}

/// The angles-slice metadata parsed from the corpus manifest.
#[derive(Clone, Debug, PartialEq)]
struct AnglesManifest {
    reference_engine: String,
    rows: usize,
    checksum: u64,
}

/// Errors produced while parsing or validating the angles corpus.
#[derive(Clone, Debug, PartialEq)]
pub enum AnglesCorpusError {
    /// A CSV data row could not be parsed.
    MalformedRow {
        row: usize,
        line: String,
        reason: String,
    },
    /// The manifest text could not be parsed (missing angles slice, etc.).
    MalformedManifest { reason: String },
    /// The corpus CSV checksum does not match the manifest pin.
    ChecksumMismatch { expected: u64, actual: u64 },
    /// A manifest field does not match the corpus shape (e.g. row count).
    ManifestDrift {
        field: String,
        expected: String,
        actual: String,
    },
    /// A recomputed value exceeded its per-point arcsecond ceiling.
    CeilingExceeded {
        row: usize,
        chart_id: String,
        point: &'static str,
        got: f64,
        want: f64,
        residual_arcsec: f64,
        ceiling_arcsec: f64,
    },
    /// `chart_points_from_armc` returned an error for a corpus row.
    ChartPointsFailed {
        row: usize,
        chart_id: String,
        reason: String,
    },
    /// The true obliquity of date could not be computed for a corpus row.
    ObliquityFailed {
        row: usize,
        chart_id: String,
        reason: String,
    },
}

impl fmt::Display for AnglesCorpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRow { row, line, reason } => {
                write!(f, "angles corpus row {row} is malformed ({reason}): {line:?}")
            }
            Self::MalformedManifest { reason } => {
                write!(f, "angles corpus manifest is malformed: {reason}")
            }
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "angles corpus checksum mismatch: expected {expected}, got {actual}"
            ),
            Self::ManifestDrift {
                field,
                expected,
                actual,
            } => write!(
                f,
                "angles corpus manifest drift on field '{field}': expected {expected:?}, got {actual:?}"
            ),
            Self::CeilingExceeded {
                row,
                chart_id,
                point,
                got,
                want,
                residual_arcsec,
                ceiling_arcsec,
            } => write!(
                f,
                "angles corpus row {row} ({chart_id} {point}): residual {residual_arcsec:.4}\u{2033} > ceiling {ceiling_arcsec:.1}\u{2033} (got={got:.6}\u{00b0}, want={want:.6}\u{00b0})"
            ),
            Self::ChartPointsFailed {
                row,
                chart_id,
                reason,
            } => write!(
                f,
                "angles corpus row {row} ({chart_id}): chart_points_from_armc failed: {reason}"
            ),
            Self::ObliquityFailed {
                row,
                chart_id,
                reason,
            } => write!(
                f,
                "angles corpus row {row} ({chart_id}): true obliquity of date failed: {reason}"
            ),
        }
    }
}

impl std::error::Error for AnglesCorpusError {}

/// Summary report produced by [`validate_angles_corpus`].
#[derive(Clone, Debug, PartialEq)]
pub struct AnglesCorpusReport {
    /// Number of corpus rows validated.
    pub rows_validated: usize,
    /// Reference engine field from the manifest.
    pub reference_engine: String,
    /// Maximum ARMC / local-apparent sidereal-time residual, arcseconds.
    pub max_armc_arcsec: f64,
    /// Maximum GAST residual, arcseconds.
    pub max_gast_arcsec: f64,
    /// Maximum vertex residual, arcseconds.
    pub max_vertex_arcsec: f64,
    /// Maximum equatorial-ascendant residual, arcseconds.
    pub max_equasc_arcsec: f64,
    /// Maximum Koch co-ascendant residual, arcseconds.
    pub max_coasc_koch_arcsec: f64,
    /// Maximum Munkasey co-ascendant residual, arcseconds.
    pub max_coasc_munkasey_arcsec: f64,
    /// Maximum Munkasey polar-ascendant residual, arcseconds.
    pub max_polar_asc_arcsec: f64,
    /// Compact one-line summary.
    pub summary_line: String,
}

impl AnglesCorpusReport {
    /// Returns the compact one-line summary.
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

impl fmt::Display for AnglesCorpusReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

/// Wraps [`validate_angles_corpus`] with a boolean pass/fail predicate for use
/// in the gate test.
#[derive(Debug)]
pub struct AnglesGateOutcome(pub Result<AnglesCorpusReport, AnglesCorpusError>);

impl AnglesGateOutcome {
    /// Returns `true` when the gate passed over the committed corpus.
    pub fn passed(&self) -> bool {
        self.0.is_ok()
    }
}

/// Runs the angles SE-parity gate and returns a pass/fail outcome.
pub fn run_angles_gate() -> AnglesGateOutcome {
    AnglesGateOutcome(validate_angles_corpus())
}

/// Circular absolute residual in arcseconds between two degree values.
fn wrap_arcsec(got: f64, want: f64) -> f64 {
    let mut d = (got - want).abs();
    if d > 180.0 {
        d = 360.0 - d;
    }
    d * 3600.0
}

/// Parses the `role=angles` slice from the corpus manifest.
fn parse_angles_manifest(text: &str) -> Result<AnglesManifest, AnglesCorpusError> {
    let mut reference_engine: Option<String> = None;
    let mut rows: Option<usize> = None;
    let mut checksum: Option<u64> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#Reference-Engine:") {
            reference_engine = Some(rest.trim().to_string());
            continue;
        }
        if trimmed.starts_with("slice ") && trimmed.contains("role=angles") {
            for token in trimmed.split_whitespace() {
                if let Some(val) = token.strip_prefix("rows=") {
                    rows = Some(val.parse::<usize>().map_err(|_| {
                        AnglesCorpusError::MalformedManifest {
                            reason: format!("angles rows value {val:?} is not a valid usize"),
                        }
                    })?);
                } else if let Some(val) = token.strip_prefix("checksum=") {
                    checksum = Some(val.parse::<u64>().map_err(|_| {
                        AnglesCorpusError::MalformedManifest {
                            reason: format!("angles checksum value {val:?} is not a valid u64"),
                        }
                    })?);
                }
            }
        }
    }

    let reference_engine =
        reference_engine.ok_or_else(|| AnglesCorpusError::MalformedManifest {
            reason: "#Reference-Engine comment not found".to_string(),
        })?;
    let rows = rows.ok_or_else(|| AnglesCorpusError::MalformedManifest {
        reason: "rows= key not found in angles slice line".to_string(),
    })?;
    let checksum = checksum.ok_or_else(|| AnglesCorpusError::MalformedManifest {
        reason: "checksum= key not found in angles slice line".to_string(),
    })?;

    Ok(AnglesManifest {
        reference_engine,
        rows,
        checksum,
    })
}

/// Parses the angles corpus CSV. Schema (verbatim from the Task-6 harness):
/// `chart_id,jd_ut,lat_deg,lon_deg,armc,vertex,equatorial_ascendant,coasc_koch,coasc_munkasey,polar_asc,sidtime_gast_hours`.
///
/// Fails closed: any malformed data row returns `Err(MalformedRow)`.
fn parse_angles_corpus(csv: &str) -> Result<Vec<AnglesRow>, AnglesCorpusError> {
    let mut rows = Vec::new();
    let mut data_row = 0usize;

    for line in csv.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.starts_with("chart_id,") {
            continue;
        }
        data_row += 1;
        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() != 11 {
            return Err(AnglesCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("expected 11 comma-separated fields, got {}", parts.len()),
            });
        }
        let parse_f64 = |idx: usize, name: &str| -> Result<f64, AnglesCorpusError> {
            parts[idx]
                .trim()
                .parse()
                .map_err(|_| AnglesCorpusError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("{name} {:?} is not a valid float", parts[idx]),
                })
        };
        rows.push(AnglesRow {
            chart_id: parts[0].trim().to_string(),
            jd_ut: parse_f64(1, "jd_ut")?,
            lat_deg: parse_f64(2, "lat_deg")?,
            lon_deg: parse_f64(3, "lon_deg")?,
            armc: parse_f64(4, "armc")?,
            vertex: parse_f64(5, "vertex")?,
            equatorial_ascendant: parse_f64(6, "equatorial_ascendant")?,
            coasc_koch: parse_f64(7, "coasc_koch")?,
            coasc_munkasey: parse_f64(8, "coasc_munkasey")?,
            polar_asc: parse_f64(9, "polar_asc")?,
            sidtime_gast_hours: parse_f64(10, "sidtime_gast_hours")?,
        });
    }

    Ok(rows)
}

/// Runs the angles SE-parity gate over the committed corpus.
///
/// Steps:
/// 1. Verify the embedded CSV checksum against the manifest `role=angles` pin.
/// 2. Verify the row count matches the manifest.
/// 3. For each row: check sidereal-time parity (ARMC + GAST) and geometry parity
///    (vertex, equatorial ascendant, both co-ascendants, polar ascendant) against
///    per-point arcsecond ceilings.
///
/// Returns an [`AnglesCorpusReport`] on success, or the first
/// [`AnglesCorpusError`] on failure (fail-closed).
pub fn validate_angles_corpus() -> Result<AnglesCorpusReport, AnglesCorpusError> {
    // 1. Checksum gate.
    let manifest = parse_angles_manifest(ANGLES_MANIFEST)?;
    let actual_checksum = pleiades_apparent::fnv1a64(ANGLES_CSV);
    if actual_checksum != manifest.checksum {
        return Err(AnglesCorpusError::ChecksumMismatch {
            expected: manifest.checksum,
            actual: actual_checksum,
        });
    }

    // 2. Row-count gate.
    let rows = parse_angles_corpus(ANGLES_CSV)?;
    if rows.len() != manifest.rows {
        return Err(AnglesCorpusError::ManifestDrift {
            field: "rows".into(),
            expected: manifest.rows.to_string(),
            actual: rows.len().to_string(),
        });
    }

    let mut max_armc = 0.0_f64;
    let mut max_gast = 0.0_f64;
    let mut max_vertex = 0.0_f64;
    let mut max_equasc = 0.0_f64;
    let mut max_koch = 0.0_f64;
    let mut max_munkasey = 0.0_f64;
    let mut max_polar = 0.0_f64;

    for (idx, row) in rows.iter().enumerate() {
        let data_row = idx + 1;

        // (a) Sidereal-time parity. Match the house gate's TT time-scale
        //     convention for corpus jd_ut instants.
        let instant = pleiades_core::Instant::new(
            JulianDay::from_days(row.jd_ut),
            pleiades_core::TimeScale::Tt,
        );
        let sid = pleiades_apparent::sidereal::sidereal_time(
            instant,
            Longitude::from_degrees(row.lon_deg),
        );
        // Local apparent sidereal time == ARMC (right ascension of the MC).
        let armc_resid = wrap_arcsec(sid.local_apparent_deg, row.armc);
        max_armc = max_armc.max(armc_resid);
        if armc_resid > CEIL_ARMC_ARCSEC {
            return Err(AnglesCorpusError::CeilingExceeded {
                row: data_row,
                chart_id: row.chart_id.clone(),
                point: "armc/local-apparent-sidereal-time",
                got: sid.local_apparent_deg,
                want: row.armc,
                residual_arcsec: armc_resid,
                ceiling_arcsec: CEIL_ARMC_ARCSEC,
            });
        }
        let gast_want = (row.sidtime_gast_hours * 15.0).rem_euclid(360.0);
        let gast_resid = wrap_arcsec(sid.gast_deg, gast_want);
        max_gast = max_gast.max(gast_resid);
        if gast_resid > CEIL_GAST_ARCSEC {
            return Err(AnglesCorpusError::CeilingExceeded {
                row: data_row,
                chart_id: row.chart_id.clone(),
                point: "gast",
                got: sid.gast_deg,
                want: gast_want,
                residual_arcsec: gast_resid,
                ceiling_arcsec: CEIL_GAST_ARCSEC,
            });
        }

        // (b) Geometry parity. Feed SE's own ARMC to isolate the chart-point
        //     geometry from any sidereal-time delta. Use the true obliquity of
        //     date (mean + Δε) that chart_points uses internally by default.
        let obliquity_deg = pleiades_apparent::true_obliquity_degrees(row.jd_ut).map_err(|e| {
            AnglesCorpusError::ObliquityFailed {
                row: data_row,
                chart_id: row.chart_id.clone(),
                reason: e.to_string(),
            }
        })?;
        let points = chart_points_from_armc(
            Longitude::from_degrees(row.armc),
            Latitude::from_degrees(row.lat_deg),
            Angle::from_degrees(obliquity_deg),
        )
        .map_err(|e| AnglesCorpusError::ChartPointsFailed {
            row: data_row,
            chart_id: row.chart_id.clone(),
            reason: e.to_string(),
        })?;

        for (point, got, want, ceiling, max) in [
            (
                "vertex",
                points.vertex.degrees(),
                row.vertex,
                CEIL_VERTEX_ARCSEC,
                &mut max_vertex,
            ),
            (
                "equatorial_ascendant",
                points.equatorial_ascendant.degrees(),
                row.equatorial_ascendant,
                CEIL_EQUASC_ARCSEC,
                &mut max_equasc,
            ),
            (
                "coascendant_koch",
                points.coascendant_koch.degrees(),
                row.coasc_koch,
                CEIL_COASC_KOCH_ARCSEC,
                &mut max_koch,
            ),
            (
                "coascendant_munkasey",
                points.coascendant_munkasey.degrees(),
                row.coasc_munkasey,
                CEIL_COASC_MUNKASEY_ARCSEC,
                &mut max_munkasey,
            ),
            (
                "polar_ascendant",
                points.polar_ascendant.degrees(),
                row.polar_asc,
                CEIL_POLAR_ASC_ARCSEC,
                &mut max_polar,
            ),
        ] {
            let resid = wrap_arcsec(got, want);
            *max = max.max(resid);
            if resid > ceiling {
                return Err(AnglesCorpusError::CeilingExceeded {
                    row: data_row,
                    chart_id: row.chart_id.clone(),
                    point,
                    got,
                    want,
                    residual_arcsec: resid,
                    ceiling_arcsec: ceiling,
                });
            }
        }
    }

    let summary_line = format!(
        "Angles gate: {} rows, max residuals (\u{2033}): armc {:.4}, gast {:.4}, vertex {:.4}, equasc {:.4}, coasc-koch {:.4}, coasc-munkasey {:.4}, polar-asc {:.4}; reference {}",
        rows.len(),
        max_armc,
        max_gast,
        max_vertex,
        max_equasc,
        max_koch,
        max_munkasey,
        max_polar,
        manifest.reference_engine,
    );

    Ok(AnglesCorpusReport {
        rows_validated: rows.len(),
        reference_engine: manifest.reference_engine,
        max_armc_arcsec: max_armc,
        max_gast_arcsec: max_gast,
        max_vertex_arcsec: max_vertex,
        max_equasc_arcsec: max_equasc,
        max_coasc_koch_arcsec: max_koch,
        max_coasc_munkasey_arcsec: max_munkasey,
        max_polar_asc_arcsec: max_polar,
        summary_line,
    })
}
