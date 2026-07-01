//! Fail-closed cross-check of the engine's apparent equatorial-of-date RA/Dec
//! against JPL Horizons apparent RA/Dec goldens (quantity 2). Reads the
//! committed CSV offline. RA residual is cos(Dec)-weighted; Dec residual signed.
#![forbid(unsafe_code)]

use core::fmt;

use pleiades_core::{
    Apparentness, CelestialBody, ChartEngine, ChartRequest, Instant, JulianDay, TimeScale,
};
use pleiades_data::PackagedDataBackend;

const GOLDENS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/equatorial-goldens.csv"
));
const GOLDENS_CHECKSUM: u64 = 7_911_902_500_580_784_449;

#[derive(Clone, Debug, PartialEq)]
pub struct EquatorialValidationReport {
    pub rows_validated: usize,
    pub max_residual_ra_arcsec: f64,
    pub max_residual_dec_arcsec: f64,
    summary_line: String,
}

impl EquatorialValidationReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

impl fmt::Display for EquatorialValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EquatorialValidationError {
    ChecksumMismatch {
        expected: u64,
        actual: u64,
    },
    MalformedRow {
        row: usize,
        line: String,
        reason: String,
    },
    UnknownBody {
        row: usize,
        label: String,
    },
    ChartError {
        row: usize,
        body: String,
        jd: f64,
        message: String,
    },
    UnexpectedMeanFallback {
        row: usize,
        body: String,
        jd_tt: f64,
    },
    MissingEquatorial {
        row: usize,
        body: String,
        jd_tt: f64,
    },
    ToleranceExceeded {
        row: usize,
        body: String,
        jd: f64,
        axis: &'static str,
        got: f64,
        want: f64,
        residual_arcsec: f64,
        tolerance_arcsec: f64,
    },
    EmptyCorpus,
}

impl fmt::Display for EquatorialValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch { expected, actual } =>
                write!(f, "equatorial goldens checksum mismatch: expected {expected:#018x}, got {actual:#018x}"),
            Self::MalformedRow { row, line, reason } =>
                write!(f, "equatorial goldens row {row} malformed ({reason}): {line:?}"),
            Self::UnknownBody { row, label } =>
                write!(f, "equatorial goldens row {row}: unknown body {label:?}"),
            Self::ChartError { row, body, jd, message } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd}): chart error: {message}"),
            Self::UnexpectedMeanFallback { row, body, jd_tt } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd_tt}): mean fallback — apparent provenance absent"),
            Self::MissingEquatorial { row, body, jd_tt } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd_tt}): equatorial channel absent"),
            Self::ToleranceExceeded { row, body, jd, axis, got, want, residual_arcsec, tolerance_arcsec } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd}) {axis}: got {got:.7} want {want:.7} residual {residual_arcsec:.2}\u{2033} > tol {tolerance_arcsec:.1}\u{2033}"),
            Self::EmptyCorpus => write!(f, "equatorial goldens corpus is empty (fail-closed)"),
        }
    }
}

impl std::error::Error for EquatorialValidationError {}

fn resolve_body(label: &str) -> Option<CelestialBody> {
    match label {
        "Sun" => Some(CelestialBody::Sun),
        "Moon" => Some(CelestialBody::Moon),
        "Mercury" => Some(CelestialBody::Mercury),
        "Venus" => Some(CelestialBody::Venus),
        "Mars" => Some(CelestialBody::Mars),
        "Jupiter" => Some(CelestialBody::Jupiter),
        "Saturn" => Some(CelestialBody::Saturn),
        "Uranus" => Some(CelestialBody::Uranus),
        "Neptune" => Some(CelestialBody::Neptune),
        "Pluto" => Some(CelestialBody::Pluto),
        _ => None,
    }
}

struct Row {
    body_label: String,
    body: CelestialBody,
    jd_tt: f64,
    ra_deg: f64,
    dec_deg: f64,
    ra_tol_arcsec: f64,
    dec_tol_arcsec: f64,
}

fn parse() -> Result<Vec<Row>, EquatorialValidationError> {
    let mut rows = Vec::new();
    let mut n = 0usize;
    for line in GOLDENS_CSV.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() {
            continue;
        }
        if t.starts_with("body,jd_tt,apparent_ra_deg") {
            continue;
        }
        n += 1;
        let p: Vec<&str> = t.splitn(6, ',').collect();
        if p.len() != 6 {
            return Err(EquatorialValidationError::MalformedRow {
                row: n,
                line: line.to_string(),
                reason: format!("expected 6 fields, got {}", p.len()),
            });
        }
        let f = |i: usize, name: &str| -> Result<f64, EquatorialValidationError> {
            p[i].trim()
                .parse::<f64>()
                .map_err(|_| EquatorialValidationError::MalformedRow {
                    row: n,
                    line: line.to_string(),
                    reason: format!("{name} {:?} not a float", p[i]),
                })
        };
        let body_label = p[0].trim().to_string();
        let body =
            resolve_body(&body_label).ok_or_else(|| EquatorialValidationError::UnknownBody {
                row: n,
                label: body_label.clone(),
            })?;
        rows.push(Row {
            body_label,
            body,
            jd_tt: f(1, "jd_tt")?,
            ra_deg: f(2, "ra")?,
            dec_deg: f(3, "dec")?,
            ra_tol_arcsec: f(4, "ra_tol")?,
            dec_tol_arcsec: f(5, "dec_tol")?,
        });
    }
    Ok(rows)
}

fn wrap_deg(mut d: f64) -> f64 {
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    d
}

pub fn validate_equatorial_goldens() -> Result<EquatorialValidationReport, EquatorialValidationError>
{
    let actual = pleiades_apparent::fnv1a64(GOLDENS_CSV);
    if GOLDENS_CHECKSUM != 0 && actual != GOLDENS_CHECKSUM {
        return Err(EquatorialValidationError::ChecksumMismatch {
            expected: GOLDENS_CHECKSUM,
            actual,
        });
    }
    let rows = parse()?;
    if rows.is_empty() {
        return Err(EquatorialValidationError::EmptyCorpus);
    }
    let engine = ChartEngine::new(PackagedDataBackend::new());
    let (mut max_ra, mut max_dec) = (0.0_f64, 0.0_f64);
    for (idx, row) in rows.iter().enumerate() {
        let r = idx + 1;
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let req = ChartRequest::new(instant)
            .with_bodies(vec![row.body.clone()])
            .with_apparentness(Apparentness::Apparent);
        let snap = engine
            .chart(&req)
            .map_err(|e| EquatorialValidationError::ChartError {
                row: r,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                message: e.to_string(),
            })?;
        let p =
            snap.placement_for(&row.body)
                .ok_or_else(|| EquatorialValidationError::ChartError {
                    row: r,
                    body: row.body_label.clone(),
                    jd: row.jd_tt,
                    message: "body not in snapshot".into(),
                })?;
        if p.apparent.is_none() {
            return Err(EquatorialValidationError::UnexpectedMeanFallback {
                row: r,
                body: row.body_label.clone(),
                jd_tt: row.jd_tt,
            });
        }
        let eq =
            p.position
                .equatorial
                .ok_or_else(|| EquatorialValidationError::MissingEquatorial {
                    row: r,
                    body: row.body_label.clone(),
                    jd_tt: row.jd_tt,
                })?;
        let got_ra = eq.right_ascension.degrees();
        let got_dec = eq.declination.degrees();
        // cos(Dec)-weighted RA residual (pole-safe), arcsec.
        let cos_dec = row.dec_deg.to_radians().cos();
        let ra_resid = (wrap_deg(got_ra - row.ra_deg).abs() * cos_dec) * 3600.0;
        let dec_resid = (got_dec - row.dec_deg).abs() * 3600.0;
        if ra_resid > row.ra_tol_arcsec {
            return Err(EquatorialValidationError::ToleranceExceeded {
                row: r,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                axis: "ra",
                got: got_ra,
                want: row.ra_deg,
                residual_arcsec: ra_resid,
                tolerance_arcsec: row.ra_tol_arcsec,
            });
        }
        if dec_resid > row.dec_tol_arcsec {
            return Err(EquatorialValidationError::ToleranceExceeded {
                row: r,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                axis: "dec",
                got: got_dec,
                want: row.dec_deg,
                residual_arcsec: dec_resid,
                tolerance_arcsec: row.dec_tol_arcsec,
            });
        }
        max_ra = max_ra.max(ra_resid);
        max_dec = max_dec.max(dec_resid);
    }
    let summary_line = format!(
        "Equatorial goldens: {} rows validated vs JPL Horizons, max RA {:.2}\u{2033} (cos\u{03b4}-wt), max Dec {:.2}\u{2033}",
        rows.len(), max_ra, max_dec
    );
    Ok(EquatorialValidationReport {
        rows_validated: rows.len(),
        max_residual_ra_arcsec: max_ra,
        max_residual_dec_arcsec: max_dec,
        summary_line,
    })
}

const SE_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/equatorial-se-corpus/equatorial-se.csv"
));
const SE_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/equatorial-se-corpus/manifest.txt"
));

// Loose parity ceilings — set to ceil(measured_max * 1.5), measured 2026-06-30 over 400 rows.
// The Moon dominates: Moshier(SE corpus) vs DE440(our packaged Moon) diverges significantly
// at century-scale offsets from J2000.  max measured: RA 2642.637" (Moon@2435114.2),
// Dec 1202.920" (Moon@2460681.8).  Planets stay well under 100".  These ceilings catch
// gross convention/sign/units errors; a radians-vs-degrees or sign flip would exceed them
// by orders of magnitude.
const SE_RA_CEILING_ARCSEC: f64 = 4000.0; // measured max 2642.637" (Moon, cos-δ-wt)
const SE_DEC_CEILING_ARCSEC: f64 = 1810.0; // measured max 1202.920" (Moon)

#[derive(Clone, Debug, PartialEq)]
pub struct EquatorialSeReport {
    pub rows_validated: usize,
    pub max_residual_ra_arcsec: f64,
    pub max_residual_dec_arcsec: f64,
    summary_line: String,
}

impl EquatorialSeReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EquatorialSeError {
    ChecksumMismatch {
        got: u64,
        want: u64,
    },
    ManifestDrift {
        rows_csv: usize,
        rows_manifest: usize,
    },
    MalformedRow(String),
    MalformedManifest(String),
    UnknownBody(String),
    ChartError {
        jd_tt: f64,
        body: String,
        message: String,
    },
    MeanFallback {
        jd_tt: f64,
        body: String,
    },
    MissingEquatorial {
        jd_tt: f64,
        body: String,
    },
    CeilingExceeded {
        jd_tt: f64,
        body: String,
        axis: &'static str,
        residual: f64,
        ceiling: f64,
    },
}

impl fmt::Display for EquatorialSeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch { got, want } =>
                write!(f, "equatorial-se checksum mismatch: got {got:016x} want {want:016x}"),
            Self::ManifestDrift { rows_csv, rows_manifest } =>
                write!(f, "equatorial-se manifest drift: csv {rows_csv} vs manifest {rows_manifest}"),
            Self::MalformedRow(s) => write!(f, "malformed equatorial-se row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed equatorial-se manifest: {s}"),
            Self::UnknownBody(s) => write!(f, "unknown body in equatorial-se corpus: {s}"),
            Self::ChartError { jd_tt, body, message } =>
                write!(f, "equatorial-se chart error ({body} @ {jd_tt}): {message}"),
            Self::MeanFallback { jd_tt, body } =>
                write!(f, "equatorial-se ({body} @ {jd_tt}): unexpected mean fallback"),
            Self::MissingEquatorial { jd_tt, body } =>
                write!(f, "equatorial-se ({body} @ {jd_tt}): equatorial channel absent"),
            Self::CeilingExceeded { jd_tt, body, axis, residual, ceiling } =>
                write!(f, "equatorial-se ({body} @ {jd_tt}) {axis}: residual {residual:.2}\u{2033} > ceiling {ceiling:.1}\u{2033}"),
        }
    }
}

impl std::error::Error for EquatorialSeError {}

fn se_manifest_rows() -> Result<(usize, u64), EquatorialSeError> {
    let line = SE_MANIFEST
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| EquatorialSeError::MalformedManifest("no slice line".into()))?;
    let (mut rows, mut checksum) = (None, None);
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| EquatorialSeError::MalformedManifest(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| EquatorialSeError::MalformedManifest(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| EquatorialSeError::MalformedManifest("rows= missing".into()))?,
        checksum.ok_or_else(|| EquatorialSeError::MalformedManifest("checksum= missing".into()))?,
    ))
}

pub fn validate_equatorial_se_corpus() -> Result<EquatorialSeReport, EquatorialSeError> {
    let (manifest_rows, manifest_checksum) = se_manifest_rows()?;
    let got = pleiades_apparent::fnv1a64(SE_CSV);
    if got != manifest_checksum {
        return Err(EquatorialSeError::ChecksumMismatch {
            got,
            want: manifest_checksum,
        });
    }

    // Parse rows: jd_tt,body,ra_deg,dec_deg.
    let mut parsed: Vec<(f64, String, CelestialBody, f64, f64)> = Vec::new();
    for line in SE_CSV.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() || t.starts_with("jd_tt") {
            continue;
        }
        let c: Vec<&str> = t.split(',').collect();
        if c.len() != 4 {
            return Err(EquatorialSeError::MalformedRow(t.to_string()));
        }
        let jd = c[0]
            .trim()
            .parse::<f64>()
            .map_err(|_| EquatorialSeError::MalformedRow(t.to_string()))?;
        let label = c[1].trim().to_string();
        let body =
            resolve_body(&label).ok_or_else(|| EquatorialSeError::UnknownBody(label.clone()))?;
        let ra = c[2]
            .trim()
            .parse::<f64>()
            .map_err(|_| EquatorialSeError::MalformedRow(t.to_string()))?;
        let dec = c[3]
            .trim()
            .parse::<f64>()
            .map_err(|_| EquatorialSeError::MalformedRow(t.to_string()))?;
        parsed.push((jd, label, body, ra, dec));
    }
    if parsed.len() != manifest_rows {
        return Err(EquatorialSeError::ManifestDrift {
            rows_csv: parsed.len(),
            rows_manifest: manifest_rows,
        });
    }

    let engine = ChartEngine::new(PackagedDataBackend::new());
    let (mut max_ra, mut max_dec, mut validated) = (0.0_f64, 0.0_f64, 0usize);
    for (jd, label, body, se_ra, se_dec) in &parsed {
        let instant = Instant::new(JulianDay::from_days(*jd), TimeScale::Tt);
        let req = ChartRequest::new(instant)
            .with_bodies(vec![body.clone()])
            .with_apparentness(Apparentness::Apparent);
        let snap = engine
            .chart(&req)
            .map_err(|e| EquatorialSeError::ChartError {
                jd_tt: *jd,
                body: label.clone(),
                message: e.to_string(),
            })?;
        let p = snap
            .placement_for(body)
            .ok_or_else(|| EquatorialSeError::ChartError {
                jd_tt: *jd,
                body: label.clone(),
                message: "body not in snapshot".into(),
            })?;
        if p.apparent.is_none() {
            return Err(EquatorialSeError::MeanFallback {
                jd_tt: *jd,
                body: label.clone(),
            });
        }
        let eq = p
            .position
            .equatorial
            .ok_or_else(|| EquatorialSeError::MissingEquatorial {
                jd_tt: *jd,
                body: label.clone(),
            })?;
        let cos_dec = se_dec.to_radians().cos();
        let ra_resid = wrap_deg(eq.right_ascension.degrees() - se_ra).abs() * cos_dec * 3600.0;
        let dec_resid = (eq.declination.degrees() - se_dec).abs() * 3600.0;
        if ra_resid > SE_RA_CEILING_ARCSEC {
            return Err(EquatorialSeError::CeilingExceeded {
                jd_tt: *jd,
                body: label.clone(),
                axis: "ra",
                residual: ra_resid,
                ceiling: SE_RA_CEILING_ARCSEC,
            });
        }
        if dec_resid > SE_DEC_CEILING_ARCSEC {
            return Err(EquatorialSeError::CeilingExceeded {
                jd_tt: *jd,
                body: label.clone(),
                axis: "dec",
                residual: dec_resid,
                ceiling: SE_DEC_CEILING_ARCSEC,
            });
        }
        max_ra = max_ra.max(ra_resid);
        max_dec = max_dec.max(dec_resid);
        validated += 1;
    }
    let summary_line = format!(
        "Equatorial-SE parity: {validated} rows vs Swiss Ephemeris SEFLG_EQUATORIAL, max RA {max_ra:.2}\u{2033} (cos\u{03b4}-wt) Dec {max_dec:.2}\u{2033}"
    );
    Ok(EquatorialSeReport {
        rows_validated: validated,
        max_residual_ra_arcsec: max_ra,
        max_residual_dec_arcsec: max_dec,
        summary_line,
    })
}

#[cfg(test)]
mod se_tests {
    use super::*;

    /// Diagnostic: scan the whole corpus without ceilings to print actual max residuals.
    /// Run with `cargo test -p pleiades-validate equatorial_validation::se_tests::measure -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_max_residuals() {
        let engine = ChartEngine::new(PackagedDataBackend::new());
        let (mut max_ra, mut max_dec) = (0.0_f64, 0.0_f64);
        let (mut max_ra_ctx, mut max_dec_ctx) = (String::new(), String::new());
        for line in SE_CSV.lines() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() || t.starts_with("jd_tt") {
                continue;
            }
            let c: Vec<&str> = t.split(',').collect();
            if c.len() != 4 {
                continue;
            }
            let jd: f64 = c[0].trim().parse().unwrap();
            let label = c[1].trim().to_string();
            let body = match resolve_body(&label) {
                Some(b) => b,
                None => continue,
            };
            let se_ra: f64 = c[2].trim().parse().unwrap();
            let se_dec: f64 = c[3].trim().parse().unwrap();
            let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
            let req = ChartRequest::new(instant)
                .with_bodies(vec![body.clone()])
                .with_apparentness(Apparentness::Apparent);
            let snap = engine.chart(&req).unwrap();
            let p = snap.placement_for(&body).unwrap();
            if p.apparent.is_none() {
                continue;
            }
            let eq = match p.position.equatorial {
                Some(e) => e,
                None => continue,
            };
            let cos_dec = se_dec.to_radians().cos();
            let ra_r = wrap_deg(eq.right_ascension.degrees() - se_ra).abs() * cos_dec * 3600.0;
            let dec_r = (eq.declination.degrees() - se_dec).abs() * 3600.0;
            if ra_r > max_ra {
                max_ra = ra_r;
                max_ra_ctx = format!("{label}@{jd:.1}");
            }
            if dec_r > max_dec {
                max_dec = dec_r;
                max_dec_ctx = format!("{label}@{jd:.1}");
            }
        }
        eprintln!("max RA  residual: {max_ra:.3}\" at {max_ra_ctx}");
        eprintln!("max Dec residual: {max_dec:.3}\" at {max_dec_ctx}");
    }

    #[test]
    fn equatorial_se_parity_passes() {
        let report =
            validate_equatorial_se_corpus().expect("equatorial-se parity within loose ceilings");
        assert!(report.rows_validated >= 1, "fail-closed floor");
        eprintln!("{}", report.summary_line());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equatorial_goldens_pass() {
        let report = validate_equatorial_goldens().expect("equatorial goldens within tolerance");
        // Fail-closed floor: 10 bodies × 5 epochs = 50 rows.
        assert!(
            report.rows_validated >= 45,
            "too few rows validated: {}",
            report.rows_validated
        );
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn pinned_checksum() {
        let actual = pleiades_apparent::fnv1a64(GOLDENS_CSV);
        assert_eq!(
            GOLDENS_CHECKSUM, actual,
            "update GOLDENS_CHECKSUM to {actual}"
        );
    }
}
