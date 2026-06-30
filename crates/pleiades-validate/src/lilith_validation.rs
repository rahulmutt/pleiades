//! Fail-closed gate: our of-date osculating True Apogee vs the committed Swiss
//! Ephemeris `SE_OSCU_APOG` reference corpus. Reproduces the exact chart path —
//! packaged-backend mean-J2000 apsis → `apparent_apsis_position` (precession +
//! nutation only) — and compares against SE within published ceilings.

use pleiades_apparent::{apparent_apsis_position, fnv1a64};
use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
use pleiades_data::PackagedDataBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/lilith-corpus/lilith.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/lilith-corpus/manifest.txt"
));

// Ceilings — set to ceil(measured_max * 1.5), measured 2026-06-30 over 3176 rows.
// The dominant residual is the Moshier(SE corpus) vs DE440(our packaged Moon) difference.
// max measured: lon 306.442", lat 53.007", dist_rel 1.56e-4
const LON_CEILING_ARCSEC: f64 = 460.0; // measured max 306.442"
const LAT_CEILING_ARCSEC: f64 = 80.0; // measured max 53.007"
const DIST_CEILING_REL: f64 = 2.34e-4; // measured max 1.56e-4

#[derive(Clone, Copy, Debug)]
struct LilithRow {
    jd_tt: f64,
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

#[derive(Debug)]
pub enum LilithCorpusError {
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
    CalculationFailed {
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for LilithCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got:016x} want {want:016x}")
            }
            Self::ManifestDrift { rows_csv, rows_manifest } => {
                write!(f, "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}")
            }
            Self::CalculationFailed { jd_tt, reason } => {
                write!(f, "calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "lilith {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.6} want {want:.6} residual {residual:.4} > ceiling {ceiling:.4}"
            ),
        }
    }
}

impl std::error::Error for LilithCorpusError {}

#[derive(Debug)]
pub struct LilithCorpusReport {
    pub rows_validated: usize,
    /// Rows skipped because the packaged backend's coverage boundary (±0.5 day motion
    /// probe) falls outside the supported range; typically ≤2 rows at 1900/2100 edges.
    pub rows_skipped_oor: usize,
    pub max_residual_lon_arcsec: f64,
    pub max_residual_lat_arcsec: f64,
    pub max_residual_dist_rel: f64,
    summary_line: String,
}

impl LilithCorpusReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn parse_corpus() -> Result<Vec<LilithRow>, LilithCorpusError> {
    let mut rows = Vec::new();
    for line in CORPUS_CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let mut it = line.split(',');
        let mut next = |name: &str| -> Result<f64, LilithCorpusError> {
            it.next()
                .ok_or_else(|| {
                    LilithCorpusError::MalformedRow(format!("{name} missing in {line}"))
                })?
                .parse::<f64>()
                .map_err(|e| LilithCorpusError::MalformedRow(format!("{name}: {e} in {line}")))
        };
        rows.push(LilithRow {
            jd_tt: next("jd_tt")?,
            lon_deg: next("lon")?,
            lat_deg: next("lat")?,
            dist_au: next("dist")?,
        });
    }
    Ok(rows)
}

fn parse_manifest_rows() -> Result<(usize, u64), LilithCorpusError> {
    let line = MANIFEST
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| LilithCorpusError::MalformedManifest("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| LilithCorpusError::MalformedManifest(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| LilithCorpusError::MalformedManifest(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| LilithCorpusError::MalformedManifest("rows= missing".into()))?,
        checksum.ok_or_else(|| LilithCorpusError::MalformedManifest("checksum= missing".into()))?,
    ))
}

fn wrap_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    let mut d = got_deg - want_deg;
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    (d * 3600.0).abs()
}

pub fn validate_lilith_corpus() -> Result<LilithCorpusReport, LilithCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest_rows()?;
    let got_checksum = fnv1a64(CORPUS_CSV);
    if got_checksum != manifest_checksum {
        return Err(LilithCorpusError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus()?;
    if rows.len() != manifest_rows {
        return Err(LilithCorpusError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let backend = PackagedDataBackend::new();
    let mut max_lon = 0.0_f64;
    let mut max_lat = 0.0_f64;
    let mut max_dist = 0.0_f64;
    let mut validated = 0usize;
    let mut skipped_oor = 0usize;

    for row in &rows {
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let mean_result =
            backend.position(&EphemerisRequest::new(CelestialBody::TrueApogee, instant));
        let mean = match mean_result {
            Ok(r) => r
                .ecliptic
                .ok_or_else(|| LilithCorpusError::CalculationFailed {
                    jd_tt: row.jd_tt,
                    reason: "no ecliptic".into(),
                })?,
            Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => {
                // The packaged backend's ±0.5-day motion probe falls outside the
                // 1900–2100 coverage window at the first/last rows. Skip silently;
                // at most a handful of boundary rows are affected.
                skipped_oor += 1;
                continue;
            }
            Err(e) => {
                return Err(LilithCorpusError::CalculationFailed {
                    jd_tt: row.jd_tt,
                    reason: e.to_string(),
                })
            }
        };
        let apparent = apparent_apsis_position(instant, mean).map_err(|e| {
            LilithCorpusError::CalculationFailed {
                jd_tt: row.jd_tt,
                reason: format!("{e:?}"),
            }
        })?;

        let our_lon = apparent.ecliptic.longitude.degrees();
        let our_lat = apparent.ecliptic.latitude.degrees();
        let our_dist = apparent.ecliptic.distance_au.unwrap_or(0.0);

        let resid_lon = wrap_arcsec(our_lon, row.lon_deg);
        let resid_lat = ((our_lat - row.lat_deg) * 3600.0).abs();
        let resid_dist = if row.dist_au != 0.0 {
            ((our_dist - row.dist_au) / row.dist_au).abs()
        } else {
            0.0
        };

        if resid_lon > LON_CEILING_ARCSEC {
            return Err(LilithCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "longitude_arcsec",
                got: our_lon,
                want: row.lon_deg,
                residual: resid_lon,
                ceiling: LON_CEILING_ARCSEC,
            });
        }
        if resid_lat > LAT_CEILING_ARCSEC {
            return Err(LilithCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "latitude_arcsec",
                got: our_lat,
                want: row.lat_deg,
                residual: resid_lat,
                ceiling: LAT_CEILING_ARCSEC,
            });
        }
        if resid_dist > DIST_CEILING_REL {
            return Err(LilithCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "distance_rel",
                got: our_dist,
                want: row.dist_au,
                residual: resid_dist,
                ceiling: DIST_CEILING_REL,
            });
        }

        max_lon = max_lon.max(resid_lon);
        max_lat = max_lat.max(resid_lat);
        max_dist = max_dist.max(resid_dist);
        validated += 1;
    }

    let summary_line = format!(
        "Lilith gate: {validated} rows validated ({skipped_oor} oor-skipped) vs Swiss Ephemeris SE_OSCU_APOG, max lon {max_lon:.3}\" lat {max_lat:.3}\" dist {max_dist:.2e} rel"
    );
    Ok(LilithCorpusReport {
        rows_validated: validated,
        rows_skipped_oor: skipped_oor,
        max_residual_lon_arcsec: max_lon,
        max_residual_lat_arcsec: max_lat,
        max_residual_dist_rel: max_dist,
        summary_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lilith_gate_passes_within_ceilings() {
        let report = validate_lilith_corpus().expect("lilith gate passes");
        assert!(report.rows_validated > 0);
        // Fail-closed floor: the OOR-skip is a single-boundary-row accommodation, not
        // a license to silently validate almost nothing. Cap the skipped rows and
        // require the corpus is overwhelmingly validated, so a future regression that
        // mass-skips rows fails the gate instead of passing vacuously.
        assert!(
            report.rows_skipped_oor <= 5,
            "too many oor-skipped rows: {} (expected the boundary handful); check coverage/probe logic",
            report.rows_skipped_oor
        );
        assert!(
            report.rows_validated >= 3170,
            "too few rows validated: {} (corpus is 3177 rows)",
            report.rows_validated
        );
        // Print measured maxima so the ceilings can be tightened.
        eprintln!("{}", report.summary_line());
    }
}
