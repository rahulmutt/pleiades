//! Fail-closed gate: our of-date osculating True Node vs the committed Swiss
//! Ephemeris `SE_TRUE_NODE` reference corpus. Reproduces the exact chart path —
//! backend mean-J2000 node → `apparent_apsis_position` (precession + nutation
//! in longitude only) — and compares against SE within published ceilings.
//! Sibling of `lilith_validation` (issue #58).
//!
//! Two channels are measured over the same corpus: the release-grade packaged
//! node (`PackagedDataBackend`, DE440 Moon state) under the release ceilings,
//! and the constrained-tier ELP node (`ElpBackend`, compact Moon series) under
//! its own looser ceilings (issue #127). Either channel over its ceiling fails
//! the gate.

use pleiades_apparent::{apparent_apsis_position, fnv1a64};
use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/true-node-corpus/true-node.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/true-node-corpus/manifest.txt"
));

// Ceilings — set to ceil(measured_max * 1.5), measured 2026-09-26 over 3177 rows.
// The dominant residual is the Moshier(SE corpus) vs DE440(our packaged Moon)
// difference amplified by 1/sin(i) in the node direction (i ≈ 5.1°). Latitude
// is ~0 by construction (nodes lie on the ecliptic and only longitude gets
// precession/nutation applied), so the measured max is floating-point noise.
// max measured: lon 52.851", lat 6.30e-11", dist_rel 1.5809e-4
const LON_CEILING_ARCSEC: f64 = 80.0; // measured max 52.851"
const LAT_CEILING_ARCSEC: f64 = 1.0; // measured max 6.30e-11" (fp noise; lat is 0 by construction)
const DIST_CEILING_REL: f64 = 2.38e-4; // measured max 1.5809e-4

// ELP channel ceilings — ceil(measured_max * 1.5), measured 2026-10-04 over the
// same 3177 rows (issue #127). The residual is set by the truncated Meeus Moon
// series feeding the differenced velocity, amplified by 1/sin(i); it has no
// drift across the window (p50 0.19', p90 0.51', p99 0.84'). The Meeus Ch. 47
// periodic-term node this channel replaced measured 17.37' max on this corpus.
// max measured: lon 78.0" (1.30'), lat 6.87e-11", dist_rel 4.189e-4
const ELP_LON_CEILING_ARCSEC: f64 = 120.0; // measured max 78.0"
const ELP_LAT_CEILING_ARCSEC: f64 = 1.0; // measured max 6.87e-11" (fp noise)
const ELP_DIST_CEILING_REL: f64 = 6.4e-4; // measured max 4.189e-4

/// Fail-closed floor on validated rows per channel (corpus is 3177 rows; every
/// row lies inside the packaged window, so skips should be zero).
const MIN_ROWS_VALIDATED: usize = 3170;

/// Per-channel ceilings for one true-node backend.
#[derive(Clone, Copy)]
struct Ceilings {
    lon_arcsec: f64,
    lat_arcsec: f64,
    dist_rel: f64,
}

const PACKAGED_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: LON_CEILING_ARCSEC,
    lat_arcsec: LAT_CEILING_ARCSEC,
    dist_rel: DIST_CEILING_REL,
};

const ELP_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: ELP_LON_CEILING_ARCSEC,
    lat_arcsec: ELP_LAT_CEILING_ARCSEC,
    dist_rel: ELP_DIST_CEILING_REL,
};

#[derive(Clone, Copy, Debug)]
struct TrueNodeRow {
    jd_tt: f64,
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

#[derive(Debug)]
pub enum TrueNodeCorpusError {
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
        /// Which channel exceeded: `"packaged"` or `"elp"`.
        backend: &'static str,
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
    TooFewRowsValidated {
        /// Which channel fell short: `"packaged"` or `"elp"`.
        backend: &'static str,
        validated: usize,
        floor: usize,
    },
}

impl std::fmt::Display for TrueNodeCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got} want {want}")
            }
            Self::ManifestDrift { rows_csv, rows_manifest } => {
                write!(f, "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}")
            }
            Self::CalculationFailed { jd_tt, reason } => {
                write!(f, "calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { backend, jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "true-node ({backend}) {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.6} want {want:.6} residual {residual:.4} > ceiling {ceiling:.4}"
            ),
            Self::TooFewRowsValidated { backend, validated, floor } => write!(
                f,
                "true-node ({backend}): only {validated} rows validated, floor is {floor}"
            ),
        }
    }
}

impl std::error::Error for TrueNodeCorpusError {}

#[derive(Debug)]
pub struct TrueNodeCorpusReport {
    /// Packaged (release-grade) channel: rows compared.
    pub rows_validated: usize,
    /// Rows skipped because the packaged backend reported the node's position
    /// outside its coverage window. Zero on the committed corpus; the gate fails
    /// if fewer than `MIN_ROWS_VALIDATED` rows are validated.
    pub rows_skipped_oor: usize,
    pub max_residual_lon_arcsec: f64,
    pub max_residual_lat_arcsec: f64,
    pub max_residual_dist_rel: f64,
    /// ELP (constrained-tier) channel: rows compared. The ELP backend has no
    /// coverage window, so every corpus row is compared.
    pub elp_rows_validated: usize,
    pub elp_max_residual_lon_arcsec: f64,
    pub elp_max_residual_lat_arcsec: f64,
    pub elp_max_residual_dist_rel: f64,
    summary_line: String,
}

/// Residual maxima of one backend's true node over the corpus.
struct ChannelMeasurement {
    rows_validated: usize,
    rows_skipped_oor: usize,
    max_lon_arcsec: f64,
    max_lat_arcsec: f64,
    max_dist_rel: f64,
}

impl TrueNodeCorpusReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn parse_corpus() -> Result<Vec<TrueNodeRow>, TrueNodeCorpusError> {
    let mut rows = Vec::new();
    for line in CORPUS_CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let mut it = line.split(',');
        let mut next = |name: &str| -> Result<f64, TrueNodeCorpusError> {
            it.next()
                .ok_or_else(|| {
                    TrueNodeCorpusError::MalformedRow(format!("{name} missing in {line}"))
                })?
                .parse::<f64>()
                .map_err(|e| TrueNodeCorpusError::MalformedRow(format!("{name}: {e} in {line}")))
        };
        rows.push(TrueNodeRow {
            jd_tt: next("jd_tt")?,
            lon_deg: next("lon")?,
            lat_deg: next("lat")?,
            dist_au: next("dist")?,
        });
    }
    Ok(rows)
}

fn parse_manifest_rows() -> Result<(usize, u64), TrueNodeCorpusError> {
    crate::corpus_manifest::slice_entry(MANIFEST)
        .map_err(|e| TrueNodeCorpusError::MalformedManifest(e.to_string()))
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

/// Compares one backend's `TrueNode`, taken through `apparent_apsis_position`,
/// against every corpus row under `ceilings`. `OutOfRangeInstant` rows are
/// skipped, but the channel fails unless at least `MIN_ROWS_VALIDATED` rows
/// (or every row, for a smaller corpus) are validated; every other failure is
/// an error.
fn measure_channel<B: EphemerisBackend>(
    backend: &B,
    label: &'static str,
    rows: &[TrueNodeRow],
    ceilings: Ceilings,
) -> Result<ChannelMeasurement, TrueNodeCorpusError> {
    let mut max_lon = 0.0_f64;
    let mut max_lat = 0.0_f64;
    let mut max_dist = 0.0_f64;
    let mut validated = 0usize;
    let mut skipped_oor = 0usize;

    for row in rows {
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let mean = match backend.position(&EphemerisRequest::new(CelestialBody::TrueNode, instant))
        {
            Ok(r) => r
                .ecliptic
                .ok_or_else(|| TrueNodeCorpusError::CalculationFailed {
                    jd_tt: row.jd_tt,
                    reason: format!("{label}: no ecliptic"),
                })?,
            Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => {
                // Only a position outside the backend's window lands here: the
                // packaged motion probe degrades to `None` channels instead of
                // erroring. The floor below bounds how many rows may skip.
                skipped_oor += 1;
                continue;
            }
            Err(e) => {
                return Err(TrueNodeCorpusError::CalculationFailed {
                    jd_tt: row.jd_tt,
                    reason: format!("{label}: {e}"),
                })
            }
        };
        let apparent = apparent_apsis_position(instant, mean).map_err(|e| {
            TrueNodeCorpusError::CalculationFailed {
                jd_tt: row.jd_tt,
                reason: format!("{label}: {e:?}"),
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

        if resid_lon > ceilings.lon_arcsec {
            return Err(TrueNodeCorpusError::CeilingExceeded {
                backend: label,
                jd_tt: row.jd_tt,
                kind: "longitude_arcsec",
                got: our_lon,
                want: row.lon_deg,
                residual: resid_lon,
                ceiling: ceilings.lon_arcsec,
            });
        }
        if resid_lat > ceilings.lat_arcsec {
            return Err(TrueNodeCorpusError::CeilingExceeded {
                backend: label,
                jd_tt: row.jd_tt,
                kind: "latitude_arcsec",
                got: our_lat,
                want: row.lat_deg,
                residual: resid_lat,
                ceiling: ceilings.lat_arcsec,
            });
        }
        if resid_dist > ceilings.dist_rel {
            return Err(TrueNodeCorpusError::CeilingExceeded {
                backend: label,
                jd_tt: row.jd_tt,
                kind: "distance_rel",
                got: our_dist,
                want: row.dist_au,
                residual: resid_dist,
                ceiling: ceilings.dist_rel,
            });
        }

        max_lon = max_lon.max(resid_lon);
        max_lat = max_lat.max(resid_lat);
        max_dist = max_dist.max(resid_dist);
        validated += 1;
    }

    let floor = MIN_ROWS_VALIDATED.min(rows.len());
    if validated < floor {
        return Err(TrueNodeCorpusError::TooFewRowsValidated {
            backend: label,
            validated,
            floor,
        });
    }

    Ok(ChannelMeasurement {
        rows_validated: validated,
        rows_skipped_oor: skipped_oor,
        max_lon_arcsec: max_lon,
        max_lat_arcsec: max_lat,
        max_dist_rel: max_dist,
    })
}

pub fn validate_true_node_corpus() -> Result<TrueNodeCorpusReport, TrueNodeCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest_rows()?;
    let got_checksum = fnv1a64(CORPUS_CSV);
    if got_checksum != manifest_checksum {
        return Err(TrueNodeCorpusError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus()?;
    if rows.len() != manifest_rows {
        return Err(TrueNodeCorpusError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let packaged = measure_channel(
        &PackagedDataBackend::new(),
        "packaged",
        &rows,
        PACKAGED_CEILINGS,
    )?;
    let elp = measure_channel(&ElpBackend::new(), "elp", &rows, ELP_CEILINGS)?;

    let summary_line = format!(
        "True-node gate: {} rows validated ({} oor-skipped) vs Swiss Ephemeris SE_TRUE_NODE, max lon {:.3}\" lat {:.3}\" dist {:.2e} rel; ELP osculating node {} rows, max lon {:.3}\" lat {:.3}\" dist {:.2e} rel",
        packaged.rows_validated,
        packaged.rows_skipped_oor,
        packaged.max_lon_arcsec,
        packaged.max_lat_arcsec,
        packaged.max_dist_rel,
        elp.rows_validated,
        elp.max_lon_arcsec,
        elp.max_lat_arcsec,
        elp.max_dist_rel,
    );
    Ok(TrueNodeCorpusReport {
        rows_validated: packaged.rows_validated,
        rows_skipped_oor: packaged.rows_skipped_oor,
        max_residual_lon_arcsec: packaged.max_lon_arcsec,
        max_residual_lat_arcsec: packaged.max_lat_arcsec,
        max_residual_dist_rel: packaged.max_dist_rel,
        elp_rows_validated: elp.rows_validated,
        elp_max_residual_lon_arcsec: elp.max_lon_arcsec,
        elp_max_residual_lat_arcsec: elp.max_lat_arcsec,
        elp_max_residual_dist_rel: elp.max_dist_rel,
        summary_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::AlwaysOutOfRange;

    #[test]
    fn channel_that_skips_every_row_fails_the_gate() {
        // #64: the floor must live in the gate, not only in this test module,
        // so release-gate and the CLI cannot pass on a mass of skipped rows.
        let rows = parse_corpus().expect("corpus parses");
        let err = measure_channel(&AlwaysOutOfRange::new(), "stub", &rows, PACKAGED_CEILINGS)
            .err()
            .expect("a channel that validates no rows must fail the gate");
        let message = err.to_string();
        assert!(
            message.contains("0 rows validated, floor is 3170"),
            "{message}"
        );
    }

    #[test]
    fn true_node_gate_passes_within_ceilings() {
        // The gate itself enforces the validated-row floor (see
        // `channel_that_skips_every_row_fails_the_gate`); every committed row
        // lies inside the packaged window, so none is skipped.
        let report = validate_true_node_corpus().expect("true-node gate passes");
        assert_eq!(report.rows_skipped_oor, 0);
        assert_eq!(report.elp_rows_validated, report.rows_validated);
        // Print measured maxima so the ceilings can be set/tightened.
        eprintln!("{}", report.summary_line());
    }
}
