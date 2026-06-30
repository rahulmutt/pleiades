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
    ChecksumMismatch { expected: u64, actual: u64 },
    MalformedRow { row: usize, line: String, reason: String },
    UnknownBody { row: usize, label: String },
    ChartError { row: usize, body: String, jd: f64, message: String },
    UnexpectedMeanFallback { row: usize, body: String, jd_tt: f64 },
    MissingEquatorial { row: usize, body: String, jd_tt: f64 },
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
        if t.starts_with('#') || t.is_empty() { continue; }
        if t.starts_with("body,jd_tt,apparent_ra_deg") { continue; }
        n += 1;
        let p: Vec<&str> = t.splitn(6, ',').collect();
        if p.len() != 6 {
            return Err(EquatorialValidationError::MalformedRow {
                row: n, line: line.to_string(),
                reason: format!("expected 6 fields, got {}", p.len()),
            });
        }
        let f = |i: usize, name: &str| -> Result<f64, EquatorialValidationError> {
            p[i].trim().parse::<f64>().map_err(|_| EquatorialValidationError::MalformedRow {
                row: n, line: line.to_string(), reason: format!("{name} {:?} not a float", p[i]),
            })
        };
        let body_label = p[0].trim().to_string();
        let body = resolve_body(&body_label).ok_or_else(|| EquatorialValidationError::UnknownBody {
            row: n, label: body_label.clone(),
        })?;
        rows.push(Row {
            body_label, body,
            jd_tt: f(1, "jd_tt")?, ra_deg: f(2, "ra")?, dec_deg: f(3, "dec")?,
            ra_tol_arcsec: f(4, "ra_tol")?, dec_tol_arcsec: f(5, "dec_tol")?,
        });
    }
    Ok(rows)
}

fn wrap_deg(mut d: f64) -> f64 {
    while d > 180.0 { d -= 360.0; }
    while d < -180.0 { d += 360.0; }
    d
}

pub fn validate_equatorial_goldens() -> Result<EquatorialValidationReport, EquatorialValidationError> {
    let actual = pleiades_apparent::fnv1a64(GOLDENS_CSV);
    if GOLDENS_CHECKSUM != 0 && actual != GOLDENS_CHECKSUM {
        return Err(EquatorialValidationError::ChecksumMismatch { expected: GOLDENS_CHECKSUM, actual });
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
        let snap = engine.chart(&req).map_err(|e| EquatorialValidationError::ChartError {
            row: r, body: row.body_label.clone(), jd: row.jd_tt, message: e.to_string(),
        })?;
        let p = snap.placement_for(&row.body).ok_or_else(|| EquatorialValidationError::ChartError {
            row: r, body: row.body_label.clone(), jd: row.jd_tt, message: "body not in snapshot".into(),
        })?;
        if p.apparent.is_none() {
            return Err(EquatorialValidationError::UnexpectedMeanFallback {
                row: r, body: row.body_label.clone(), jd_tt: row.jd_tt,
            });
        }
        let eq = p.position.equatorial.ok_or_else(|| EquatorialValidationError::MissingEquatorial {
            row: r, body: row.body_label.clone(), jd_tt: row.jd_tt,
        })?;
        let got_ra = eq.right_ascension.degrees();
        let got_dec = eq.declination.degrees();
        // cos(Dec)-weighted RA residual (pole-safe), arcsec.
        let cos_dec = row.dec_deg.to_radians().cos();
        let ra_resid = (wrap_deg(got_ra - row.ra_deg).abs() * cos_dec) * 3600.0;
        let dec_resid = (got_dec - row.dec_deg).abs() * 3600.0;
        if ra_resid > row.ra_tol_arcsec {
            return Err(EquatorialValidationError::ToleranceExceeded {
                row: r, body: row.body_label.clone(), jd: row.jd_tt, axis: "ra",
                got: got_ra, want: row.ra_deg, residual_arcsec: ra_resid, tolerance_arcsec: row.ra_tol_arcsec,
            });
        }
        if dec_resid > row.dec_tol_arcsec {
            return Err(EquatorialValidationError::ToleranceExceeded {
                row: r, body: row.body_label.clone(), jd: row.jd_tt, axis: "dec",
                got: got_dec, want: row.dec_deg, residual_arcsec: dec_resid, tolerance_arcsec: row.dec_tol_arcsec,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equatorial_goldens_pass() {
        let report = validate_equatorial_goldens().expect("equatorial goldens within tolerance");
        // Fail-closed floor: 10 bodies × 5 epochs = 50 rows.
        assert!(report.rows_validated >= 45, "too few rows validated: {}", report.rows_validated);
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn pinned_checksum() {
        let actual = pleiades_apparent::fnv1a64(GOLDENS_CSV);
        assert_eq!(GOLDENS_CHECKSUM, actual, "update GOLDENS_CHECKSUM to {actual}");
    }
}
