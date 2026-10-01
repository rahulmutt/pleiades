//! Fail-closed gate: `EventEngine::position_at(.., CrossingFrame::Heliocentric, ..)`
//! on the packaged backend vs the committed Swiss Ephemeris
//! `SEFLG_HELCTR | SEFLG_SPEED` reference corpus (Mercury–Pluto, 1900–2100),
//! for longitude, latitude, distance and their speeds (issue #89).
//!
//! The longitude residual carries the known light-time signature of
//! reconstructing the heliocentric vector from the backend's geocentric
//! vectors (see `pleiades-events` `tests/heliocentric.rs`), the same floor
//! `validate-crossings` measures for the heliocentric frame.

use crate::helio_position_thresholds::{PLANET_CEILINGS, PLUTO_CEILINGS};
use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, EventEngine, EventError};
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/helio-position-corpus/helio-position.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/helio-position-corpus/manifest.txt"
));

/// Fail-closed floor on validated rows (corpus is 25424 rows; both window
/// edges are inside the engine's window, so skips should be zero).
const MIN_ROWS_VALIDATED: usize = 25_400;

#[derive(Clone, Debug)]
struct Row {
    jd_tt: f64,
    body: CelestialBody,
    body_name: &'static str,
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
    lon_speed: f64,
    lat_speed: f64,
    dist_speed: f64,
}

#[derive(Debug)]
pub enum HelioPositionError {
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
        body: &'static str,
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        body: &'static str,
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for HelioPositionError {
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
            Self::CalculationFailed { body, jd_tt, reason } => {
                write!(f, "{body} calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { body, jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "{body} {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.12} want {want:.12} residual {residual:.6e} > ceiling {ceiling:.6e}"
            ),
        }
    }
}

impl std::error::Error for HelioPositionError {}

/// Largest absolute residuals seen for one body group across the corpus.
#[derive(Clone, Copy, Debug, Default)]
pub struct HelioMaxima {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub dist_rel: f64,
    pub lon_speed_arcsec_per_day: f64,
    pub lat_speed_arcsec_per_day: f64,
    pub dist_speed_au_per_day: f64,
}

#[derive(Debug)]
pub struct HelioPositionReport {
    pub rows_validated: usize,
    /// Rows skipped because the instant is outside the engine's window.
    pub rows_skipped_oor: usize,
    /// Mercury–Neptune.
    pub maxima: HelioMaxima,
    /// Pluto (served from a different source than the other planets).
    pub pluto_maxima: HelioMaxima,
    /// Mean signed longitude-speed residual (arcsec/day) over Jupiter–Neptune.
    /// A value near ±0.137 would mean the reference and the engine disagree on
    /// whether the speed includes the precession rate.
    pub outer_lon_speed_mean_signed_arcsec_per_day: f64,
    summary_line: String,
}

impl HelioPositionReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn body_from_name(name: &str) -> Option<(CelestialBody, &'static str)> {
    Some(match name {
        "Mercury" => (CelestialBody::Mercury, "Mercury"),
        "Venus" => (CelestialBody::Venus, "Venus"),
        "Mars" => (CelestialBody::Mars, "Mars"),
        "Jupiter" => (CelestialBody::Jupiter, "Jupiter"),
        "Saturn" => (CelestialBody::Saturn, "Saturn"),
        "Uranus" => (CelestialBody::Uranus, "Uranus"),
        "Neptune" => (CelestialBody::Neptune, "Neptune"),
        "Pluto" => (CelestialBody::Pluto, "Pluto"),
        _ => return None,
    })
}

fn parse_corpus(csv: &str) -> Result<Vec<Row>, HelioPositionError> {
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 8 {
            return Err(HelioPositionError::MalformedRow(format!(
                "expected 8 fields, got {} in {line}",
                fields.len()
            )));
        }
        let num = |i: usize| -> Result<f64, HelioPositionError> {
            let v = fields[i].parse::<f64>().map_err(|e| {
                HelioPositionError::MalformedRow(format!("field {i}: {e} in {line}"))
            })?;
            if v.is_finite() {
                Ok(v)
            } else {
                Err(HelioPositionError::MalformedRow(format!(
                    "field {i} is not finite in {line}"
                )))
            }
        };
        let (body, body_name) = body_from_name(fields[1]).ok_or_else(|| {
            HelioPositionError::MalformedRow(format!("unknown body {} in {line}", fields[1]))
        })?;
        let dist_au = num(4)?;
        if dist_au <= 0.0 {
            return Err(HelioPositionError::MalformedRow(format!(
                "non-positive distance in {line}"
            )));
        }
        rows.push(Row {
            jd_tt: num(0)?,
            body,
            body_name,
            lon_deg: num(2)?,
            lat_deg: num(3)?,
            dist_au,
            lon_speed: num(5)?,
            lat_speed: num(6)?,
            dist_speed: num(7)?,
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), HelioPositionError> {
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| HelioPositionError::MalformedManifest("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| HelioPositionError::MalformedManifest(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum =
                Some(v.parse::<u64>().map_err(|e| {
                    HelioPositionError::MalformedManifest(format!("checksum: {e}"))
                })?);
        }
    }
    Ok((
        rows.ok_or_else(|| HelioPositionError::MalformedManifest("rows= missing".into()))?,
        checksum
            .ok_or_else(|| HelioPositionError::MalformedManifest("checksum= missing".into()))?,
    ))
}

fn wrap_deg(got_deg: f64, want_deg: f64) -> f64 {
    (got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0
}

fn validate(csv: &str, manifest: &str) -> Result<HelioPositionReport, HelioPositionError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(HelioPositionError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(HelioPositionError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let engine = EventEngine::new(packaged_backend());
    let mut maxima = HelioMaxima::default();
    let mut pluto_maxima = HelioMaxima::default();
    let mut validated = 0usize;
    let mut skipped_oor = 0usize;
    let (mut outer_sum, mut outer_count) = (0.0_f64, 0usize);

    for row in &rows {
        let failed = |reason: String| HelioPositionError::CalculationFailed {
            body: row.body_name,
            jd_tt: row.jd_tt,
            reason,
        };
        // The corpus epoch is TT; the engine reads the Julian day as TDB. The
        // two differ by under 2 ms, far below every ceiling here.
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tdb);
        let position =
            match engine.position_at(row.body.clone(), CrossingFrame::Heliocentric, instant) {
                Ok(position) => position,
                Err(EventError::OutOfWindow { .. }) => {
                    skipped_oor += 1;
                    continue;
                }
                Err(e) => return Err(failed(e.to_string())),
            };
        let got_dist = position
            .ecliptic
            .distance_au
            .ok_or_else(|| failed("no distance".into()))?;
        let speed = |value: Option<f64>, name: &str| {
            value.ok_or_else(|| failed(format!("no {name} speed")))
        };
        let got_lon_speed = speed(position.motion.longitude_deg_per_day, "longitude")?;
        let got_lat_speed = speed(position.motion.latitude_deg_per_day, "latitude")?;
        let got_dist_speed = speed(position.motion.distance_au_per_day, "distance")?;
        let got_lon = position.ecliptic.longitude.degrees();
        let got_lat = position.ecliptic.latitude.degrees();

        let is_pluto = row.body == CelestialBody::Pluto;
        let ceilings = if is_pluto {
            PLUTO_CEILINGS
        } else {
            PLANET_CEILINGS
        };
        let lon_speed_signed = (got_lon_speed - row.lon_speed) * 3600.0;
        let checks = [
            (
                "longitude_arcsec",
                got_lon,
                row.lon_deg,
                (wrap_deg(got_lon, row.lon_deg) * 3600.0).abs(),
                ceilings.lon_arcsec,
            ),
            (
                "latitude_arcsec",
                got_lat,
                row.lat_deg,
                ((got_lat - row.lat_deg) * 3600.0).abs(),
                ceilings.lat_arcsec,
            ),
            (
                "distance_rel",
                got_dist,
                row.dist_au,
                ((got_dist - row.dist_au) / row.dist_au).abs(),
                ceilings.dist_rel,
            ),
            (
                "longitude_speed_arcsec_per_day",
                got_lon_speed,
                row.lon_speed,
                lon_speed_signed.abs(),
                ceilings.lon_speed_arcsec_per_day,
            ),
            (
                "latitude_speed_arcsec_per_day",
                got_lat_speed,
                row.lat_speed,
                ((got_lat_speed - row.lat_speed) * 3600.0).abs(),
                ceilings.lat_speed_arcsec_per_day,
            ),
            (
                "distance_speed_au_per_day",
                got_dist_speed,
                row.dist_speed,
                (got_dist_speed - row.dist_speed).abs(),
                ceilings.dist_speed_au_per_day,
            ),
        ];
        for (kind, got, want, residual, ceiling) in checks {
            // A NaN residual must fail closed too.
            if residual.is_nan() || residual > ceiling {
                return Err(HelioPositionError::CeilingExceeded {
                    body: row.body_name,
                    jd_tt: row.jd_tt,
                    kind,
                    got,
                    want,
                    residual,
                    ceiling,
                });
            }
        }
        let group = if is_pluto {
            &mut pluto_maxima
        } else {
            &mut maxima
        };
        group.lon_arcsec = group.lon_arcsec.max(checks[0].3);
        group.lat_arcsec = group.lat_arcsec.max(checks[1].3);
        group.dist_rel = group.dist_rel.max(checks[2].3);
        group.lon_speed_arcsec_per_day = group.lon_speed_arcsec_per_day.max(checks[3].3);
        group.lat_speed_arcsec_per_day = group.lat_speed_arcsec_per_day.max(checks[4].3);
        group.dist_speed_au_per_day = group.dist_speed_au_per_day.max(checks[5].3);
        if matches!(
            row.body,
            CelestialBody::Jupiter
                | CelestialBody::Saturn
                | CelestialBody::Uranus
                | CelestialBody::Neptune
        ) {
            outer_sum += lon_speed_signed;
            outer_count += 1;
        }
        validated += 1;
    }
    let floor = MIN_ROWS_VALIDATED.min(manifest_rows);
    if validated < floor {
        return Err(HelioPositionError::TooFewRowsValidated { validated, floor });
    }
    let outer_mean = if outer_count == 0 {
        0.0
    } else {
        outer_sum / outer_count as f64
    };

    let group = |m: &HelioMaxima| {
        format!(
            "lon {:.3}\" lat {:.3}\" dist {:.2e} rel, speed lon {:.4}\"/d lat {:.4}\"/d dist {:.2e} AU/d",
            m.lon_arcsec, m.lat_arcsec, m.dist_rel,
            m.lon_speed_arcsec_per_day, m.lat_speed_arcsec_per_day, m.dist_speed_au_per_day
        )
    };
    let summary_line = format!(
        "Helio-position gate: {validated} rows validated ({skipped_oor} oor-skipped) vs Swiss Ephemeris SEFLG_HELCTR|SEFLG_SPEED, \
         Mercury-Neptune max {}; Pluto max {}; outer-planet mean signed lon speed {:+.4}\"/d",
        group(&maxima),
        group(&pluto_maxima),
        outer_mean,
    );
    Ok(HelioPositionReport {
        rows_validated: validated,
        rows_skipped_oor: skipped_oor,
        maxima,
        pluto_maxima,
        outer_lon_speed_mean_signed_arcsec_per_day: outer_mean,
        summary_line,
    })
}

pub fn validate_helio_position_corpus() -> Result<HelioPositionReport, HelioPositionError> {
    validate(CORPUS_CSV, MANIFEST)
}

#[cfg(test)]
mod tests;
