//! Fail-closed cross-check of the engine's topocentric apparent-of-date longitudes
//! and latitudes against JPL Horizons goldens (quantity 31, observer site: Madrid).
//! Reads the committed CSV offline. Also asserts that Moon topocentric longitude
//! differs from geocentric by > 0.1° (no silent geocentric fallback).

#![forbid(unsafe_code)]

use core::fmt;

use pleiades_core::{
    Apparentness, CelestialBody, ChartEngine, ChartRequest, Instant, JulianDay, Latitude,
    Longitude, ObserverLocation, TimeScale,
};
use pleiades_data::PackagedDataBackend;

const GOLDENS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/topocentric-goldens.csv"
));
const GOLDENS_CHECKSUM: u64 = 4_726_259_366_376_230_117;

/// Madrid observer site used for all topocentric golden comparisons.
///
/// lat=+40.4168°, East-lon=356.2962° (= -3.7038°), elevation=650 m.
fn madrid_observer() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(40.4168),
        Longitude::from_degrees(-3.7038),
        Some(650.0),
    )
}

/// Summary of a successful topocentric-goldens validation run.
#[derive(Clone, Debug, PartialEq)]
pub struct TopocentricValidationReport {
    /// Number of rows validated (all must pass).
    pub rows_validated: usize,
    /// Maximum absolute wrap-aware residual seen across all rows, in arcseconds.
    pub max_lon_residual_arcsec: f64,
    /// Compact release-facing summary line.
    pub summary_line: String,
}

impl TopocentricValidationReport {
    /// Returns the compact release-facing summary line.
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

impl fmt::Display for TopocentricValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

/// Errors produced by the topocentric-goldens validation.
#[derive(Clone, Debug, PartialEq)]
pub enum TopocentricValidationError {
    /// The goldens CSV checksum does not match the pinned value.
    ChecksumMismatch { expected: u64, actual: u64 },
    /// A CSV data row could not be parsed.
    MalformedRow {
        row: usize,
        line: String,
        reason: String,
    },
    /// A body label in the CSV could not be resolved to a `CelestialBody`.
    UnknownBody { row: usize, label: String },
    /// The chart engine returned an error for a row.
    ChartError {
        row: usize,
        body: String,
        jd: f64,
        message: String,
    },
    /// A body was requested in apparent mode but silently fell back to mean place.
    ///
    /// Topocentric corrections are only applied after apparent-place computation.
    /// If the body fell back to mean, any longitude comparison against a Horizons
    /// topocentric apparent golden is meaningless.  The gate fails closed.
    UnexpectedMeanFallback {
        row: usize,
        body: String,
        jd_tt: f64,
    },
    /// Topocentric provenance is absent — the body silently stayed geocentric.
    ///
    /// This guard fires when `placement.topocentric.is_none()` while `topocentric=true`
    /// was requested, indicating the chart-layer correction was silently skipped.
    UnexpectedGeocentricFallback {
        row: usize,
        body: String,
        jd_tt: f64,
    },
    /// A chart longitude exceeded its per-row tolerance.
    LongitudeToleranceExceeded {
        row: usize,
        body: String,
        jd: f64,
        got: f64,
        want: f64,
        residual_arcsec: f64,
        tolerance_arcsec: f64,
    },
    /// A chart latitude exceeded its per-row tolerance.
    LatitudeToleranceExceeded {
        row: usize,
        body: String,
        jd: f64,
        got: f64,
        want: f64,
        residual_arcsec: f64,
        tolerance_arcsec: f64,
    },
    /// The Moon topocentric longitude did not differ from geocentric by > 0.1°.
    MoonNotTopocentric {
        row: usize,
        jd: f64,
        topo_lon: f64,
        geo_lon: f64,
        diff_deg: f64,
    },
}

impl fmt::Display for TopocentricValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "topocentric goldens checksum mismatch: expected {expected:#018x}, got {actual:#018x}"
                )
            }
            Self::MalformedRow { row, line, reason } => {
                write!(
                    f,
                    "topocentric goldens row {row} is malformed ({reason}): {line:?}"
                )
            }
            Self::UnknownBody { row, label } => {
                write!(
                    f,
                    "topocentric goldens row {row}: unknown body label {label:?}"
                )
            }
            Self::UnexpectedMeanFallback { row, body, jd_tt } => {
                write!(
                    f,
                    "topocentric goldens row {row} ({body} @ JD {jd_tt}): \
                     body fell back to mean place — apparent provenance is absent; \
                     cannot validate a genuine topocentric apparent longitude against Horizons"
                )
            }
            Self::UnexpectedGeocentricFallback { row, body, jd_tt } => {
                write!(
                    f,
                    "topocentric goldens row {row} ({body} @ JD {jd_tt}): \
                     topocentric provenance is absent — the chart-layer topocentric correction \
                     was silently skipped; the longitude is still geocentric"
                )
            }
            Self::ChartError {
                row,
                body,
                jd,
                message,
            } => {
                write!(
                    f,
                    "topocentric goldens row {row} ({body} @ JD {jd}): chart error: {message}"
                )
            }
            Self::LongitudeToleranceExceeded {
                row,
                body,
                jd,
                got,
                want,
                residual_arcsec,
                tolerance_arcsec,
            } => {
                write!(
                    f,
                    "topocentric goldens row {row} ({body} @ JD {jd}): longitude \
                     got {got:.7}\u{00b0}, want {want:.7}\u{00b0}, \
                     residual {residual_arcsec:.2}\u{2033} > tolerance {tolerance_arcsec:.1}\u{2033}"
                )
            }
            Self::LatitudeToleranceExceeded {
                row,
                body,
                jd,
                got,
                want,
                residual_arcsec,
                tolerance_arcsec,
            } => {
                write!(
                    f,
                    "topocentric goldens row {row} ({body} @ JD {jd}): latitude \
                     got {got:.7}\u{00b0}, want {want:.7}\u{00b0}, \
                     residual {residual_arcsec:.2}\u{2033} > tolerance {tolerance_arcsec:.1}\u{2033}"
                )
            }
            Self::MoonNotTopocentric {
                row,
                jd,
                topo_lon,
                geo_lon,
                diff_deg,
            } => {
                write!(
                    f,
                    "topocentric goldens row {row} (Moon @ JD {jd}): \
                     topo lon {topo_lon:.6}\u{00b0} vs geo lon {geo_lon:.6}\u{00b0}: \
                     diff {diff_deg:.4}\u{00b0} \u{2264} 0.1\u{00b0} — \
                     topocentric correction not applied (silent geocentric fallback)"
                )
            }
        }
    }
}

impl std::error::Error for TopocentricValidationError {}

/// Resolves a CSV body label to a `CelestialBody`.
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

/// A parsed golden row.
struct GoldenRow {
    body_label: String,
    body: CelestialBody,
    jd_tt: f64,
    topo_longitude_deg: f64,
    topo_latitude_deg: f64,
    lon_tolerance_arcsec: f64,
    lat_tolerance_arcsec: f64,
}

/// Parse the goldens CSV, skipping comment lines (`#`) and the header row.
/// Fails closed on any malformed or unparseable data row.
fn parse_goldens() -> Result<Vec<GoldenRow>, TopocentricValidationError> {
    let mut rows = Vec::new();
    let mut data_row = 0usize;

    for line in GOLDENS_CSV.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if trimmed == "body,jd_tt,topo_longitude_deg,topo_latitude_deg,lon_tolerance_arcsec,lat_tolerance_arcsec" {
            continue;
        }

        data_row += 1;
        let parts: Vec<&str> = trimmed.splitn(6, ',').collect();
        if parts.len() != 6 {
            return Err(TopocentricValidationError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("expected 6 comma-separated fields, got {}", parts.len()),
            });
        }

        let body_label = parts[0].trim().to_string();
        let jd_tt: f64 =
            parts[1]
                .trim()
                .parse()
                .map_err(|_| TopocentricValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("jd_tt {:?} is not a valid float", parts[1]),
                })?;
        let topo_longitude_deg: f64 =
            parts[2]
                .trim()
                .parse()
                .map_err(|_| TopocentricValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("topo_longitude_deg {:?} is not a valid float", parts[2]),
                })?;
        let topo_latitude_deg: f64 =
            parts[3]
                .trim()
                .parse()
                .map_err(|_| TopocentricValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("topo_latitude_deg {:?} is not a valid float", parts[3]),
                })?;
        let lon_tolerance_arcsec: f64 =
            parts[4]
                .trim()
                .parse()
                .map_err(|_| TopocentricValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("lon_tolerance_arcsec {:?} is not a valid float", parts[4]),
                })?;
        let lat_tolerance_arcsec: f64 =
            parts[5]
                .trim()
                .parse()
                .map_err(|_| TopocentricValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("lat_tolerance_arcsec {:?} is not a valid float", parts[5]),
                })?;

        let body =
            resolve_body(&body_label).ok_or_else(|| TopocentricValidationError::UnknownBody {
                row: data_row,
                label: body_label.clone(),
            })?;

        rows.push(GoldenRow {
            body_label,
            body,
            jd_tt,
            topo_longitude_deg,
            topo_latitude_deg,
            lon_tolerance_arcsec,
            lat_tolerance_arcsec,
        });
    }

    Ok(rows)
}

/// Fail-closed cross-check of the engine's topocentric apparent-of-date longitudes
/// and latitudes against JPL Horizons goldens (quantity 31).
///
/// For each row:
///   1. Builds a topocentric apparent chart using the packaged backend.
///   2. Asserts apparent provenance is present (not a silent mean fallback).
///   3. Asserts topocentric provenance is present (not a silent geocentric fallback).
///   4. For Moon rows, also builds the geocentric apparent chart and asserts
///      the topocentric longitude differs by > 0.1° (diurnal parallax is real).
///   5. Checks longitude and latitude residuals against their per-row tolerances.
pub fn validate_topocentric_goldens(
) -> Result<TopocentricValidationReport, TopocentricValidationError> {
    // Checksum gate.
    let actual_checksum = pleiades_apparent::fnv1a64(GOLDENS_CSV);
    if GOLDENS_CHECKSUM != 0 && actual_checksum != GOLDENS_CHECKSUM {
        return Err(TopocentricValidationError::ChecksumMismatch {
            expected: GOLDENS_CHECKSUM,
            actual: actual_checksum,
        });
    }

    let rows = parse_goldens()?;
    let backend = PackagedDataBackend::new();
    let engine = ChartEngine::new(backend);

    let mut max_lon_residual_arcsec: f64 = 0.0;

    for (idx, row) in rows.iter().enumerate() {
        let data_row = idx + 1;
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);

        // Build topocentric apparent chart.
        let topo_request = ChartRequest::new(instant)
            .with_bodies(vec![row.body.clone()])
            .with_apparentness(Apparentness::Apparent)
            .with_observer(madrid_observer())
            .with_topocentric(true);

        let snapshot =
            engine
                .chart(&topo_request)
                .map_err(|e| TopocentricValidationError::ChartError {
                    row: data_row,
                    body: row.body_label.clone(),
                    jd: row.jd_tt,
                    message: e.to_string(),
                })?;

        let placement = snapshot.placement_for(&row.body).ok_or_else(|| {
            TopocentricValidationError::ChartError {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                message: "body not found in chart snapshot".to_string(),
            }
        })?;

        // Fail closed if the body silently fell back to mean place.
        if placement.apparent.is_none() {
            return Err(TopocentricValidationError::UnexpectedMeanFallback {
                row: data_row,
                body: row.body_label.clone(),
                jd_tt: row.jd_tt,
            });
        }

        // Fail closed if the topocentric correction was silently skipped.
        // Note: Sun's topocentric shift is ~8.8 arcsec peak but still present.
        if placement.topocentric.is_none() {
            return Err(TopocentricValidationError::UnexpectedGeocentricFallback {
                row: data_row,
                body: row.body_label.clone(),
                jd_tt: row.jd_tt,
            });
        }

        let ecliptic = placement.position.ecliptic.as_ref().ok_or_else(|| {
            TopocentricValidationError::ChartError {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                message: "ecliptic coordinates not available in chart result".to_string(),
            }
        })?;

        let got_lon = ecliptic.longitude.degrees();
        let got_lat = ecliptic.latitude.degrees();

        // For Moon: assert topo lon differs from geo lon by > 0.1°.
        if row.body == CelestialBody::Moon {
            let geo_request = ChartRequest::new(instant)
                .with_bodies(vec![CelestialBody::Moon])
                .with_apparentness(Apparentness::Apparent)
                .with_observer(madrid_observer())
                .with_topocentric(false);

            let geo_snapshot =
                engine
                    .chart(&geo_request)
                    .map_err(|e| TopocentricValidationError::ChartError {
                        row: data_row,
                        body: row.body_label.clone(),
                        jd: row.jd_tt,
                        message: format!("geocentric chart error: {e}"),
                    })?;

            let geo_lon = geo_snapshot
                .placement_for(&CelestialBody::Moon)
                .and_then(|p| p.position.ecliptic.as_ref())
                .map(|e| e.longitude.degrees())
                .ok_or_else(|| TopocentricValidationError::ChartError {
                    row: data_row,
                    body: row.body_label.clone(),
                    jd: row.jd_tt,
                    message: "Moon geocentric ecliptic not available".to_string(),
                })?;

            let mut diff = (got_lon - geo_lon).abs();
            if diff > 180.0 {
                diff = 360.0 - diff;
            }
            if diff <= 0.1 {
                return Err(TopocentricValidationError::MoonNotTopocentric {
                    row: data_row,
                    jd: row.jd_tt,
                    topo_lon: got_lon,
                    geo_lon,
                    diff_deg: diff,
                });
            }
        }

        // Longitude residual.
        let mut lon_diff = (got_lon - row.topo_longitude_deg).abs();
        if lon_diff > 180.0 {
            lon_diff = 360.0 - lon_diff;
        }
        let lon_residual_arcsec = lon_diff * 3600.0;
        if lon_residual_arcsec > max_lon_residual_arcsec {
            max_lon_residual_arcsec = lon_residual_arcsec;
        }
        if lon_residual_arcsec > row.lon_tolerance_arcsec {
            return Err(TopocentricValidationError::LongitudeToleranceExceeded {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                got: got_lon,
                want: row.topo_longitude_deg,
                residual_arcsec: lon_residual_arcsec,
                tolerance_arcsec: row.lon_tolerance_arcsec,
            });
        }

        // Latitude residual.
        let lat_residual_arcsec = (got_lat - row.topo_latitude_deg).abs() * 3600.0;
        if lat_residual_arcsec > row.lat_tolerance_arcsec {
            return Err(TopocentricValidationError::LatitudeToleranceExceeded {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                got: got_lat,
                want: row.topo_latitude_deg,
                residual_arcsec: lat_residual_arcsec,
                tolerance_arcsec: row.lat_tolerance_arcsec,
            });
        }
    }

    let summary_line = format!(
        "Topocentric goldens: {} rows validated, max lon residual {:.2}\u{2033}",
        rows.len(),
        max_lon_residual_arcsec
    );

    Ok(TopocentricValidationReport {
        rows_validated: rows.len(),
        max_lon_residual_arcsec,
        summary_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_checksum() {
        assert_eq!(
            pleiades_apparent::fnv1a64(GOLDENS_CSV),
            GOLDENS_CHECKSUM,
            "checksum = {}",
            pleiades_apparent::fnv1a64(GOLDENS_CSV)
        );
    }

    #[test]
    fn topocentric_goldens_pass() {
        validate_topocentric_goldens().expect("topocentric goldens within tolerance");
    }
}
