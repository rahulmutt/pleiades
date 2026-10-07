//! Fail-closed gate: the packaged backend's mean lunar node, mean apogee and
//! mean perigee vs the committed Swiss Ephemeris `SE_MEAN_NODE` /
//! `SE_MEAN_APOG` reference corpus. Reproduces the exact chart path —
//! packaged-backend J2000 point → `apparent_apsis_position` (precession +
//! nutation in longitude only) — and compares against SE within published
//! ceilings. Sibling of `true_node_validation` (issue #90).
//!
//! Swiss Ephemeris has no mean-perigee body; the perigee is gated on the same
//! rows as the point antipodal to the mean apogee (longitude + 180°, latitude
//! negated) at distance `a(1−e)`.

use pleiades_apparent::{apparent_apsis_position, fnv1a64};
use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
use pleiades_data::PackagedDataBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/mean-lunar-corpus/mean-lunar.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/mean-lunar-corpus/manifest.txt"
));

/// Swiss Ephemeris `MOON_MEAN_ECC`, used only to derive the perigee reference
/// distance from the corpus apogee distance.
const SE_MOON_MEAN_ECC: f64 = 0.054_900_489;

/// Fail-closed floor on validated rows (corpus is 3177 rows; the first row
/// sits on the coverage boundary and is served, so skips should be zero).
const MIN_ROWS_VALIDATED: usize = 3170;

// Ceilings — 1.5 × the largest measured maximum across node/apogee/perigee,
// rounded up to two significant figures (floors 0.01" / 1e-9), measured
// 2026-10-01 over 3177 rows. The residual is the difference between the Meeus
// mean-element polynomials and Swiss Ephemeris' mean lunar elements.
// max measured: lon 0.5705", lat 0.0395", dist_rel 1.02e-10
const LON_CEILING_ARCSEC: f64 = 0.86; // measured max 0.5705"
const LAT_CEILING_ARCSEC: f64 = 0.060; // measured max 0.0395"
const DIST_CEILING_REL: f64 = 1.0e-9; // measured max 1.02e-10 (floor applies)

#[derive(Clone, Copy, Debug)]
struct Reference {
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

#[derive(Clone, Copy, Debug)]
struct MeanLunarRow {
    jd_tt: f64,
    node: Reference,
    apogee: Reference,
}

impl MeanLunarRow {
    fn perigee(&self) -> Reference {
        Reference {
            lon_deg: (self.apogee.lon_deg + 180.0).rem_euclid(360.0),
            lat_deg: -self.apogee.lat_deg,
            dist_au: self.apogee.dist_au * (1.0 - SE_MOON_MEAN_ECC) / (1.0 + SE_MOON_MEAN_ECC),
        }
    }
}

#[derive(Debug)]
pub enum MeanLunarCorpusError {
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
        point: &'static str,
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        point: &'static str,
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for MeanLunarCorpusError {
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
            Self::TooFewRowsValidated { validated, floor } => {
                write!(f, "only {validated} rows validated, floor is {floor}")
            }
            Self::CalculationFailed { point, jd_tt, reason } => {
                write!(f, "{point} calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { point, jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "{point} {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.9} want {want:.9} residual {residual:.6} > ceiling {ceiling:.6}"
            ),
        }
    }
}

impl std::error::Error for MeanLunarCorpusError {}

/// Largest residuals seen for one point across the corpus.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChannelMaxima {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub dist_rel: f64,
}

#[derive(Debug)]
pub struct MeanLunarCorpusReport {
    pub rows_validated: usize,
    /// Rows skipped because the instant is outside the packaged window.
    pub rows_skipped_oor: usize,
    pub node: ChannelMaxima,
    pub apogee: ChannelMaxima,
    pub perigee: ChannelMaxima,
    summary_line: String,
}

impl MeanLunarCorpusReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn parse_corpus(csv: &str) -> Result<Vec<MeanLunarRow>, MeanLunarCorpusError> {
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 7 {
            return Err(MeanLunarCorpusError::MalformedRow(format!(
                "expected 7 fields, got {} in {line}",
                fields.len()
            )));
        }
        let num = |i: usize| -> Result<f64, MeanLunarCorpusError> {
            let v = fields[i].parse::<f64>().map_err(|e| {
                MeanLunarCorpusError::MalformedRow(format!("field {i}: {e} in {line}"))
            })?;
            if v.is_finite() {
                Ok(v)
            } else {
                Err(MeanLunarCorpusError::MalformedRow(format!(
                    "field {i} is not finite in {line}"
                )))
            }
        };
        rows.push(MeanLunarRow {
            jd_tt: num(0)?,
            node: Reference {
                lon_deg: num(1)?,
                lat_deg: num(2)?,
                dist_au: num(3)?,
            },
            apogee: Reference {
                lon_deg: num(4)?,
                lat_deg: num(5)?,
                dist_au: num(6)?,
            },
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), MeanLunarCorpusError> {
    crate::corpus_manifest::slice_entry(manifest)
        .map_err(|e| MeanLunarCorpusError::MalformedManifest(e.to_string()))
}

fn wrap_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    ((got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0).abs() * 3600.0
}

/// Residuals of one packaged point against its reference, or `None` when the
/// instant is outside the packaged window.
fn check_point(
    backend: &PackagedDataBackend,
    body: CelestialBody,
    point: &'static str,
    jd_tt: f64,
    want: Reference,
    maxima: &mut ChannelMaxima,
) -> Result<Option<()>, MeanLunarCorpusError> {
    let failed = |reason: String| MeanLunarCorpusError::CalculationFailed {
        point,
        jd_tt,
        reason,
    };
    let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
    let mean = match backend.position(&EphemerisRequest::new(body, instant)) {
        Ok(r) => r.ecliptic.ok_or_else(|| failed("no ecliptic".into()))?,
        Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => return Ok(None),
        Err(e) => return Err(failed(e.to_string())),
    };
    let apparent = apparent_apsis_position(instant, mean).map_err(|e| failed(format!("{e:?}")))?;
    let got_lon = apparent.ecliptic.longitude.degrees();
    let got_lat = apparent.ecliptic.latitude.degrees();
    let got_dist = apparent
        .ecliptic
        .distance_au
        .ok_or_else(|| failed("no distance".into()))?;

    let checks = [
        (
            "longitude_arcsec",
            got_lon,
            want.lon_deg,
            wrap_arcsec(got_lon, want.lon_deg),
            LON_CEILING_ARCSEC,
        ),
        (
            "latitude_arcsec",
            got_lat,
            want.lat_deg,
            ((got_lat - want.lat_deg) * 3600.0).abs(),
            LAT_CEILING_ARCSEC,
        ),
        (
            "distance_rel",
            got_dist,
            want.dist_au,
            ((got_dist - want.dist_au) / want.dist_au).abs(),
            DIST_CEILING_REL,
        ),
    ];
    for (kind, got, want, residual, ceiling) in checks {
        // A NaN residual must fail closed too.
        if residual.is_nan() || residual > ceiling {
            return Err(MeanLunarCorpusError::CeilingExceeded {
                point,
                jd_tt,
                kind,
                got,
                want,
                residual,
                ceiling,
            });
        }
    }
    maxima.lon_arcsec = maxima.lon_arcsec.max(checks[0].3);
    maxima.lat_arcsec = maxima.lat_arcsec.max(checks[1].3);
    maxima.dist_rel = maxima.dist_rel.max(checks[2].3);
    Ok(Some(()))
}

fn validate(csv: &str, manifest: &str) -> Result<MeanLunarCorpusReport, MeanLunarCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(MeanLunarCorpusError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(MeanLunarCorpusError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let backend = PackagedDataBackend::new();
    let (mut node, mut apogee, mut perigee) = (
        ChannelMaxima::default(),
        ChannelMaxima::default(),
        ChannelMaxima::default(),
    );
    let mut validated = 0usize;
    let mut skipped_oor = 0usize;

    for row in &rows {
        let n = check_point(
            &backend,
            CelestialBody::MeanNode,
            "mean-node",
            row.jd_tt,
            row.node,
            &mut node,
        )?;
        let a = check_point(
            &backend,
            CelestialBody::MeanApogee,
            "mean-apogee",
            row.jd_tt,
            row.apogee,
            &mut apogee,
        )?;
        let p = check_point(
            &backend,
            CelestialBody::MeanPerigee,
            "mean-perigee",
            row.jd_tt,
            row.perigee(),
            &mut perigee,
        )?;
        if n.is_some() && a.is_some() && p.is_some() {
            validated += 1;
        } else {
            skipped_oor += 1;
        }
    }
    if validated < MIN_ROWS_VALIDATED.min(manifest_rows) {
        return Err(MeanLunarCorpusError::TooFewRowsValidated {
            validated,
            floor: MIN_ROWS_VALIDATED.min(manifest_rows),
        });
    }

    let summary_line = format!(
        "Mean-lunar-points gate: {validated} rows validated ({skipped_oor} oor-skipped) vs Swiss Ephemeris SE_MEAN_NODE/SE_MEAN_APOG, \
         node max lon {:.4}\" lat {:.4}\" dist {:.2e} rel; apogee max lon {:.4}\" lat {:.4}\" dist {:.2e} rel; \
         perigee max lon {:.4}\" lat {:.4}\" dist {:.2e} rel",
        node.lon_arcsec, node.lat_arcsec, node.dist_rel,
        apogee.lon_arcsec, apogee.lat_arcsec, apogee.dist_rel,
        perigee.lon_arcsec, perigee.lat_arcsec, perigee.dist_rel,
    );
    Ok(MeanLunarCorpusReport {
        rows_validated: validated,
        rows_skipped_oor: skipped_oor,
        node,
        apogee,
        perigee,
        summary_line,
    })
}

pub fn validate_mean_lunar_points_corpus() -> Result<MeanLunarCorpusReport, MeanLunarCorpusError> {
    validate(CORPUS_CSV, MANIFEST)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_lunar_points_gate_passes_within_ceilings() {
        let report = validate_mean_lunar_points_corpus().expect("mean-lunar-points gate passes");
        assert!(report.rows_validated >= MIN_ROWS_VALIDATED);
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn tampered_corpus_fails_the_checksum() {
        let tampered = CORPUS_CSV.replacen("2415020.5,", "2415020.5, ", 1);
        assert!(matches!(
            validate(&tampered, MANIFEST),
            Err(MeanLunarCorpusError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn manifest_row_count_drift_fails_closed() {
        let checksum = fnv1a64(CORPUS_CSV);
        let manifest = format!(
            "slice mean-lunar file=mean-lunar.csv role=mean-lunar rows=3176 checksum={checksum}"
        );
        assert!(matches!(
            validate(CORPUS_CSV, &manifest),
            Err(MeanLunarCorpusError::ManifestDrift {
                rows_csv: 3177,
                rows_manifest: 3176
            })
        ));
    }

    #[test]
    fn manifest_without_a_slice_line_is_rejected() {
        assert!(matches!(
            validate(CORPUS_CSV, "rows=3177"),
            Err(MeanLunarCorpusError::MalformedManifest(_))
        ));
    }

    #[test]
    fn short_and_non_finite_rows_are_rejected() {
        for bad in [
            "2451545.0,1.0,0.0",
            "2451545.0,NaN,0.0,0.0025,1.0,1.0,0.0027",
        ] {
            let manifest = format!("slice x rows=1 checksum={}", fnv1a64(bad));
            assert!(
                matches!(
                    validate(bad, &manifest),
                    Err(MeanLunarCorpusError::MalformedRow(_))
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_shifted_reference_exceeds_the_ceiling() {
        // One real row with the node longitude moved by 0.01° (36″).
        let line = CORPUS_CSV
            .lines()
            .find(|l| l.starts_with("2451544.5,") || l.starts_with("2451546.5,"))
            .or_else(|| CORPUS_CSV.lines().find(|l| l.starts_with("245")))
            .expect("a data row near J2000");
        let mut f: Vec<String> = line.split(',').map(str::to_string).collect();
        let lon: f64 = f[1].parse().unwrap();
        f[1] = format!("{:.9}", (lon + 0.01).rem_euclid(360.0));
        let csv = f.join(",");
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
        assert!(matches!(
            validate(&csv, &manifest),
            Err(MeanLunarCorpusError::CeilingExceeded {
                point: "mean-node",
                kind: "longitude_arcsec",
                ..
            })
        ));
    }

    #[test]
    fn elp_and_packaged_mean_node_agree_at_floating_point_scale() {
        use pleiades_elp::ElpBackend;
        let elp = ElpBackend::new();
        let packaged = PackagedDataBackend::new();
        let mut max_diff = 0.0_f64;
        for jd in [2_415_021.5, 2_451_545.0, 2_461_041.5, 2_488_000.5] {
            let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
            let request = EphemerisRequest::new(CelestialBody::MeanNode, instant);
            let ecliptic = |backend: &dyn EphemerisBackend| {
                backend
                    .position(&request)
                    .ok()
                    .and_then(|r| r.ecliptic)
                    .expect("mean node ecliptic")
            };
            let (e, p) = (ecliptic(&elp), ecliptic(&packaged));
            let dlon =
                (e.longitude.degrees() - p.longitude.degrees() + 180.0).rem_euclid(360.0) - 180.0;
            let dlat = e.latitude.degrees() - p.latitude.degrees();
            max_diff = max_diff.max(dlon.abs()).max(dlat.abs());
        }
        eprintln!("elp vs packaged MeanNode max difference: {max_diff:e} deg");
        assert!(max_diff <= 1e-8, "max difference {max_diff:e} deg");
    }
}
