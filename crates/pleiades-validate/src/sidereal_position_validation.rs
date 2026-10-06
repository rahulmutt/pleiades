//! Fail-closed gate: a mean sidereal `ChartEngine` chart on the packaged
//! backend vs the committed Swiss Ephemeris geometric sidereal reference
//! corpus (`SEFLG_SIDEREAL | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL |
//! SEFLG_SPEED`; the Sun, the Moon and Mercury–Pluto; four ayanamsas;
//! 1901–2100), for longitude, latitude and longitude speed (issue #164 (b)).
//!
//! Both sides are the geometric place on the mean ecliptic and equinox of
//! date less the mean ayanamsa, so the residual is the Moshier-vs-DE440
//! ephemeris difference plus the ayanamsa gate's residual for the class. A
//! chart that subtracted the ayanamsa from a J2000 longitude, as charts did
//! before #164, is off by the precession since J2000: 83′ at the first epoch.
//! See `sidereal_position_thresholds` for the basis of the ceilings.

use crate::sidereal_position_thresholds::{Ceilings, MOON_CEILINGS, PLANET_CEILINGS, SUN_CEILINGS};
use pleiades_apparent::fnv1a64;
use pleiades_core::{ChartEngine, ChartRequest};
use pleiades_data::packaged_backend;
use pleiades_types::{
    Apparentness, Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode,
};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/sidereal-position-corpus/sidereal-position.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/sidereal-position-corpus/manifest.txt"
));

/// Fail-closed floor on validated rows: every corpus row is inside the
/// packaged backend's window, so none may be skipped.
const MIN_ROWS_VALIDATED: usize = 2680;

#[derive(Clone, Debug)]
struct Row {
    jd_tt: f64,
    ayanamsa: Ayanamsa,
    ayanamsa_name: &'static str,
    body: CelestialBody,
    body_name: &'static str,
    lon_deg: f64,
    lat_deg: f64,
    lon_speed: f64,
}

#[derive(Debug)]
pub enum SiderealPositionError {
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
        ayanamsa: &'static str,
        body: &'static str,
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        ayanamsa: &'static str,
        body: &'static str,
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for SiderealPositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got} want {want}")
            }
            Self::ManifestDrift {
                rows_csv,
                rows_manifest,
            } => write!(
                f,
                "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}"
            ),
            Self::TooFewRowsValidated { validated, floor } => {
                write!(f, "only {validated} rows validated, floor is {floor}")
            }
            Self::CalculationFailed {
                ayanamsa,
                body,
                jd_tt,
                reason,
            } => write!(
                f,
                "{ayanamsa} {body} chart failed at jd_tt={jd_tt}: {reason}"
            ),
            Self::CeilingExceeded {
                ayanamsa,
                body,
                jd_tt,
                kind,
                got,
                want,
                residual,
                ceiling,
            } => write!(
                f,
                "{ayanamsa} {body} {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.12} want {want:.12} residual {residual:.6e} > ceiling {ceiling:.6e}"
            ),
        }
    }
}

impl std::error::Error for SiderealPositionError {}

/// Largest absolute residuals seen for one body class across the corpus.
#[derive(Clone, Copy, Debug, Default)]
pub struct SiderealMaxima {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub lon_speed_arcsec_per_day: f64,
}

#[derive(Debug)]
pub struct SiderealPositionReport {
    pub rows_validated: usize,
    pub sun_maxima: SiderealMaxima,
    pub moon_maxima: SiderealMaxima,
    /// Mercury–Pluto.
    pub planet_maxima: SiderealMaxima,
    summary_line: String,
}

impl SiderealPositionReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn body_from_name(name: &str) -> Option<(CelestialBody, &'static str)> {
    Some(match name {
        "Sun" => (CelestialBody::Sun, "Sun"),
        "Moon" => (CelestialBody::Moon, "Moon"),
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

fn ayanamsa_from_name(name: &str) -> Option<(Ayanamsa, &'static str)> {
    Some(match name {
        "Lahiri" => (Ayanamsa::Lahiri, "Lahiri"),
        "TrueCitra" => (Ayanamsa::TrueCitra, "TrueCitra"),
        "GalacticCenter" => (Ayanamsa::GalacticCenter, "GalacticCenter"),
        "DeLuce" => (Ayanamsa::DeLuce, "DeLuce"),
        _ => return None,
    })
}

fn parse_corpus(csv: &str) -> Result<Vec<Row>, SiderealPositionError> {
    let malformed = |what: String| SiderealPositionError::MalformedRow(what);
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 6 {
            return Err(malformed(format!(
                "expected 6 fields, got {} in {line}",
                fields.len()
            )));
        }
        let num = |i: usize| -> Result<f64, SiderealPositionError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let (ayanamsa, ayanamsa_name) = ayanamsa_from_name(fields[1])
            .ok_or_else(|| malformed(format!("unknown ayanamsa {} in {line}", fields[1])))?;
        let (body, body_name) = body_from_name(fields[2])
            .ok_or_else(|| malformed(format!("unknown body {} in {line}", fields[2])))?;
        rows.push(Row {
            jd_tt: num(0)?,
            ayanamsa,
            ayanamsa_name,
            body,
            body_name,
            lon_deg: num(3)?,
            lat_deg: num(4)?,
            lon_speed: num(5)?,
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), SiderealPositionError> {
    let malformed = |what: String| SiderealPositionError::MalformedManifest(what);
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| malformed("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| malformed(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| malformed(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| malformed("rows= missing".into()))?,
        checksum.ok_or_else(|| malformed("checksum= missing".into()))?,
    ))
}

fn wrap_deg(got_deg: f64, want_deg: f64) -> f64 {
    (got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0
}

fn validate(csv: &str, manifest: &str) -> Result<SiderealPositionReport, SiderealPositionError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(SiderealPositionError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(SiderealPositionError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let engine = ChartEngine::new(packaged_backend());
    let mut sun_maxima = SiderealMaxima::default();
    let mut moon_maxima = SiderealMaxima::default();
    let mut planet_maxima = SiderealMaxima::default();
    let mut validated = 0usize;

    for row in &rows {
        let failed = |reason: String| SiderealPositionError::CalculationFailed {
            ayanamsa: row.ayanamsa_name,
            body: row.body_name,
            jd_tt: row.jd_tt,
            reason,
        };
        // The corpus epoch is TT; the chart reads the Julian day as TDB. The
        // two differ by under 2 ms, far below every ceiling here.
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tdb);
        let request = ChartRequest::new(instant)
            .with_bodies(vec![row.body.clone()])
            .with_apparentness(Apparentness::Mean)
            .with_zodiac_mode(ZodiacMode::Sidereal {
                ayanamsa: row.ayanamsa.clone(),
            });
        let chart = engine.chart(&request).map_err(|e| failed(e.to_string()))?;
        let position = &chart
            .placement_for(&row.body)
            .ok_or_else(|| failed("body not placed".into()))?
            .position;
        let ecliptic = position
            .ecliptic
            .ok_or_else(|| failed("no ecliptic coordinates".into()))?;
        let got_lon_speed = position
            .motion
            .and_then(|motion| motion.longitude_deg_per_day)
            .ok_or_else(|| failed("no longitude speed".into()))?;
        let got_lon = ecliptic.longitude.degrees();
        let got_lat = ecliptic.latitude.degrees();

        let (ceilings, maxima): (Ceilings, &mut SiderealMaxima) = match row.body {
            CelestialBody::Sun => (SUN_CEILINGS, &mut sun_maxima),
            CelestialBody::Moon => (MOON_CEILINGS, &mut moon_maxima),
            _ => (PLANET_CEILINGS, &mut planet_maxima),
        };
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
                "longitude_speed_arcsec_per_day",
                got_lon_speed,
                row.lon_speed,
                ((got_lon_speed - row.lon_speed) * 3600.0).abs(),
                ceilings.lon_speed_arcsec_per_day,
            ),
        ];
        for (kind, got, want, residual, ceiling) in checks {
            // A NaN residual must fail closed too.
            if residual.is_nan() || residual > ceiling {
                return Err(SiderealPositionError::CeilingExceeded {
                    ayanamsa: row.ayanamsa_name,
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
        maxima.lon_arcsec = maxima.lon_arcsec.max(checks[0].3);
        maxima.lat_arcsec = maxima.lat_arcsec.max(checks[1].3);
        maxima.lon_speed_arcsec_per_day = maxima.lon_speed_arcsec_per_day.max(checks[2].3);
        validated += 1;
    }
    let floor = MIN_ROWS_VALIDATED.min(manifest_rows);
    if validated < floor {
        return Err(SiderealPositionError::TooFewRowsValidated { validated, floor });
    }

    let class = |m: &SiderealMaxima| {
        format!(
            "lon {:.3}\" lat {:.3}\" speed lon {:.4}\"/d",
            m.lon_arcsec, m.lat_arcsec, m.lon_speed_arcsec_per_day
        )
    };
    let summary_line = format!(
        "Sidereal-position gate: {validated} mean sidereal chart placements validated vs Swiss Ephemeris \
         SEFLG_SIDEREAL|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_SPEED, \
         Sun max {}; Moon max {}; Mercury-Pluto max {}",
        class(&sun_maxima),
        class(&moon_maxima),
        class(&planet_maxima),
    );
    Ok(SiderealPositionReport {
        rows_validated: validated,
        sun_maxima,
        moon_maxima,
        planet_maxima,
        summary_line,
    })
}

pub fn validate_sidereal_position_corpus() -> Result<SiderealPositionReport, SiderealPositionError>
{
    validate(CORPUS_CSV, MANIFEST)
}

#[cfg(test)]
mod tests;
