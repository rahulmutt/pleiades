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
const GOLDENS_CHECKSUM: u64 = 1_799_772_837_221_551_337;

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

/// One body's parity ceilings: `(label, RA ceiling, Dec ceiling)` in
/// arcseconds, RA weighted by cos δ.
type SeCeilings = (&'static str, f64, f64);

/// Per-body parity ceilings, each about 1.5 times the maximum measured
/// 2026-10-05 over the body's 40 rows (in the trailing comment, RA / Dec).
/// The residual is Moshier (the corpus) against the DE440-sourced packaged
/// backend, so it is a cross-theory floor, not an accuracy claim; the Horizons
/// gate holds sub-arcsecond accuracy.
///
/// Until issue #171 one pair of ceilings, 4000″ / 1810″, covered every body.
/// They were sized to a 2643″ Moon residual that was an artefact: the corpus
/// stored its `.25` / `.75` day epochs with one decimal, so half the rows
/// were compared 0.05 day from the instant Swiss Ephemeris computed, and a
/// 1000″ regression in any planet would have passed.
const SE_CEILINGS_ARCSEC: [SeCeilings; 10] = [
    ("Sun", 0.45, 0.08),     // 0.300 / 0.049
    ("Moon", 2.4, 2.1),      // 1.562 / 1.369
    ("Mercury", 0.50, 0.12), // 0.327 / 0.077
    ("Venus", 0.72, 0.19),   // 0.477 / 0.123
    ("Mars", 0.60, 0.37),    // 0.399 / 0.246
    ("Jupiter", 0.56, 0.21), // 0.371 / 0.137
    ("Saturn", 1.3, 1.1),    // 0.804 / 0.688
    ("Uranus", 0.58, 0.26),  // 0.385 / 0.170
    ("Neptune", 3.3, 0.91),  // 2.181 / 0.604
    ("Pluto", 1.3, 0.81),    // 0.861 / 0.535
];

/// The `(RA, Dec)` ceilings for a corpus body label. A body without ceilings
/// is an error, never an unchecked row.
fn se_ceilings(label: &str) -> Result<(f64, f64), EquatorialSeError> {
    SE_CEILINGS_ARCSEC
        .iter()
        .find(|(body, _, _)| *body == label)
        .map(|&(_, ra, dec)| (ra, dec))
        .ok_or_else(|| EquatorialSeError::UnknownBody(label.to_string()))
}

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
                write!(f, "equatorial-se ({body} @ {jd_tt}) {axis}: residual {residual:.3}\u{2033} > ceiling {ceiling:.2}\u{2033}"),
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
        let (ra_ceiling, dec_ceiling) = se_ceilings(label)?;
        if ra_resid > ra_ceiling {
            return Err(EquatorialSeError::CeilingExceeded {
                jd_tt: *jd,
                body: label.clone(),
                axis: "ra",
                residual: ra_resid,
                ceiling: ra_ceiling,
            });
        }
        if dec_resid > dec_ceiling {
            return Err(EquatorialSeError::CeilingExceeded {
                jd_tt: *jd,
                body: label.clone(),
                axis: "dec",
                residual: dec_resid,
                ceiling: dec_ceiling,
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

    /// The corpus rows as `(jd_tt text, body label)`.
    fn corpus_rows() -> Vec<(&'static str, &'static str)> {
        SE_CSV
            .lines()
            .map(str::trim)
            .filter(|line| !(line.starts_with('#') || line.is_empty() || line.starts_with("jd_tt")))
            .map(|line| {
                let mut columns = line.split(',');
                (
                    columns.next().expect("jd_tt column"),
                    columns.next().expect("body column"),
                )
            })
            .collect()
    }

    // Issue #171: the reference tool steps 1826.25 days from JD 2415025.5 and
    // once printed that with one decimal, so half the corpus named an instant
    // 0.05 day from the one Swiss Ephemeris computed.
    #[test]
    fn corpus_epochs_are_the_instants_the_reference_tool_computed() {
        let rows = corpus_rows();
        assert_eq!(rows.len(), 400);
        for (jd_text, body) in rows {
            let jd: f64 = jd_text.parse().expect("jd_tt parses");
            let steps = (jd - 2_415_025.5) / 1_826.25;
            assert_eq!(steps, steps.round(), "{body} at {jd_text}");
            assert!((0.0..40.0).contains(&steps), "{body} at {jd_text}");
        }
    }

    #[test]
    fn every_corpus_body_has_its_own_ceilings() {
        for (_, body) in corpus_rows() {
            se_ceilings(body).unwrap_or_else(|error| panic!("{error}"));
        }
        assert_eq!(
            se_ceilings("Ceres"),
            Err(EquatorialSeError::UnknownBody("Ceres".to_string()))
        );
    }

    #[test]
    fn no_ceiling_would_pass_an_arcminute_regression() {
        // The point of per-body ceilings: the old global pair let a 1000″
        // planet error through.
        for (body, ra, dec) in SE_CEILINGS_ARCSEC {
            assert!(ra > 0.0 && ra < 4.0, "{body} RA ceiling {ra}");
            assert!(dec > 0.0 && dec < 4.0, "{body} Dec ceiling {dec}");
        }
    }

    /// Diagnostic: scan the whole corpus without ceilings to print actual max residuals.
    /// Run with `cargo test -p pleiades-validate equatorial_validation::se_tests::measure -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_max_residuals() {
        let engine = ChartEngine::new(PackagedDataBackend::new());
        let (mut max_ra, mut max_dec) = (0.0_f64, 0.0_f64);
        let (mut max_ra_ctx, mut max_dec_ctx) = (String::new(), String::new());
        let mut by_body = std::collections::BTreeMap::<String, (f64, f64)>::new();
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
            let body_max = by_body.entry(label.clone()).or_default();
            *body_max = (body_max.0.max(ra_r), body_max.1.max(dec_r));
            if ra_r > max_ra {
                max_ra = ra_r;
                max_ra_ctx = format!("{label}@{jd}");
            }
            if dec_r > max_dec {
                max_dec = dec_r;
                max_dec_ctx = format!("{label}@{jd}");
            }
        }
        for (label, (ra, dec)) in &by_body {
            eprintln!("{label}: max RA {ra:.3}\" Dec {dec:.3}\"");
        }
        eprintln!("max RA  residual: {max_ra:.3}\" at {max_ra_ctx}");
        eprintln!("max Dec residual: {max_dec:.3}\" at {max_dec_ctx}");
    }

    #[test]
    fn equatorial_se_parity_passes() {
        let report =
            validate_equatorial_se_corpus().expect("equatorial-se parity within its ceilings");
        assert_eq!(report.rows_validated, 400);
        // The largest per-body ceilings bound the report's maxima.
        assert!(report.max_residual_ra_arcsec < 3.3);
        assert!(report.max_residual_dec_arcsec < 2.1);
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

    /// Diagnostic: per-body maximum cos(Dec)-weighted RA and Dec residual against
    /// the Horizons goldens. Run with
    /// `cargo test -p pleiades-validate equatorial_validation::tests::measure_goldens_per_body -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_goldens_per_body() {
        let rows = parse().expect("goldens parse");
        let engine = ChartEngine::new(PackagedDataBackend::new());
        let mut max_by_body: std::collections::BTreeMap<String, (f64, f64, f64, f64)> =
            std::collections::BTreeMap::new();
        for row in &rows {
            let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
            let req = ChartRequest::new(instant)
                .with_bodies(vec![row.body.clone()])
                .with_apparentness(Apparentness::Apparent);
            let snap = engine.chart(&req).expect("chart");
            let p = snap.placement_for(&row.body).expect("placement");
            let eq = p.position.equatorial.expect("equatorial");
            let cos_dec = row.dec_deg.to_radians().cos();
            let ra_resid =
                (wrap_deg(eq.right_ascension.degrees() - row.ra_deg).abs() * cos_dec) * 3600.0;
            let dec_resid = (eq.declination.degrees() - row.dec_deg).abs() * 3600.0;
            eprintln!(
                "{},{},{ra_resid:.3},{dec_resid:.3}",
                row.body_label, row.jd_tt
            );
            let entry = max_by_body
                .entry(row.body_label.clone())
                .or_insert((0.0, row.jd_tt, 0.0, row.jd_tt));
            if ra_resid > entry.0 {
                entry.0 = ra_resid;
                entry.1 = row.jd_tt;
            }
            if dec_resid > entry.2 {
                entry.2 = dec_resid;
                entry.3 = row.jd_tt;
            }
        }
        for (body, (ra, ra_jd, dec, dec_jd)) in &max_by_body {
            eprintln!("max {body} RA {ra:.3}\" at jd {ra_jd}; Dec {dec:.3}\" at jd {dec_jd}");
        }
    }
}
