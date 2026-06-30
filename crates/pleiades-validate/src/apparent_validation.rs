//! Fail-closed cross-check of the engine's apparent-of-date longitudes against
//! JPL Horizons goldens (quantity 31). Reads the committed CSV offline.

#![forbid(unsafe_code)]

use core::fmt;

use pleiades_core::{
    Apparentness, CelestialBody, ChartEngine, ChartRequest, Instant, JulianDay, TimeScale,
};
use pleiades_data::PackagedDataBackend;

const GOLDENS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/apparent-goldens.csv"
));
const GOLDENS_CHECKSUM: u64 = 2837317093320463005;

/// Summary of a successful apparent-goldens validation run.
#[derive(Clone, Debug, PartialEq)]
pub struct ApparentValidationReport {
    /// Number of rows validated (all must pass).
    pub rows_validated: usize,
    /// Maximum absolute wrap-aware residual seen across all rows, in arcseconds.
    pub max_residual_arcsec: f64,
    /// Compact release-facing summary line.
    pub summary_line: String,
}

impl ApparentValidationReport {
    /// Returns the compact release-facing summary line.
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

impl fmt::Display for ApparentValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

/// Errors produced by the apparent-goldens validation.
#[derive(Clone, Debug, PartialEq)]
pub enum ApparentValidationError {
    /// The goldens CSV checksum does not match the pinned value.
    ChecksumMismatch {
        /// The expected (pinned) checksum.
        expected: u64,
        /// The actual checksum computed from the file.
        actual: u64,
    },
    /// A CSV data row could not be parsed.
    MalformedRow {
        /// One-based row number (counting only data rows, not headers/comments).
        row: usize,
        /// The raw CSV line.
        line: String,
        /// Description of what was malformed.
        reason: String,
    },
    /// A body label in the CSV could not be resolved to a `CelestialBody`.
    UnknownBody {
        /// One-based data row number.
        row: usize,
        /// The unrecognised body label.
        label: String,
    },
    /// The chart engine returned an error for a row.
    ChartError {
        /// One-based data row number.
        row: usize,
        /// Body label from the CSV.
        body: String,
        /// Julian day from the CSV.
        jd: f64,
        /// Error message from the engine.
        message: String,
    },
    /// A body was requested in apparent mode but silently fell back to mean place.
    ///
    /// This indicates the engine did not produce genuine apparent output for the body,
    /// so comparing its longitude against a Horizons apparent golden would be meaningless.
    /// The gate fails closed rather than masking a silent regression.
    UnexpectedMeanFallback {
        /// One-based data row number.
        row: usize,
        /// Body label from the CSV.
        body: String,
        /// Julian day (TT) from the CSV.
        jd_tt: f64,
    },
    /// A chart longitude exceeded its per-row tolerance.
    ToleranceExceeded {
        /// One-based data row number.
        row: usize,
        /// Body label from the CSV.
        body: String,
        /// Julian day from the CSV.
        jd: f64,
        /// Engine output (apparent ecliptic longitude of date), degrees.
        got: f64,
        /// Horizons golden value, degrees.
        want: f64,
        /// Wrap-aware absolute difference, arcseconds.
        residual_arcsec: f64,
        /// Per-row tolerance, arcseconds.
        tolerance_arcsec: f64,
    },
}

impl fmt::Display for ApparentValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "apparent goldens checksum mismatch: expected {expected:#018x}, got {actual:#018x}"
                )
            }
            Self::MalformedRow { row, line, reason } => {
                write!(
                    f,
                    "apparent goldens row {row} is malformed ({reason}): {line:?}"
                )
            }
            Self::UnknownBody { row, label } => {
                write!(
                    f,
                    "apparent goldens row {row}: unknown body label {label:?}"
                )
            }
            Self::UnexpectedMeanFallback { row, body, jd_tt } => {
                write!(
                    f,
                    "apparent goldens row {row} ({body} @ JD {jd_tt}): \
                     body fell back to mean place — apparent provenance is absent; \
                     cannot validate a genuine apparent longitude against Horizons"
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
                    "apparent goldens row {row} ({body} @ JD {jd}): chart error: {message}"
                )
            }
            Self::ToleranceExceeded {
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
                    "apparent goldens row {row} ({body} @ JD {jd}): \
                     got {got:.7}\u{00b0}, want {want:.7}\u{00b0}, \
                     residual {residual_arcsec:.2}\u{2033} > tolerance {tolerance_arcsec:.1}\u{2033}"
                )
            }
        }
    }
}

impl std::error::Error for ApparentValidationError {}

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
    apparent_longitude_deg: f64,
    tolerance_arcsec: f64,
}

/// Parse the goldens CSV, skipping comment lines (`#`) and the header row.
/// Fails closed on any malformed or unparseable data row.
fn parse_goldens() -> Result<Vec<GoldenRow>, ApparentValidationError> {
    let mut rows = Vec::new();
    let mut data_row = 0usize;

    for line in GOLDENS_CSV.lines() {
        let trimmed = line.trim();
        // Skip comment lines and blank lines.
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        // Skip the header row.
        if trimmed == "body,jd_tt,apparent_longitude_deg,tolerance_arcsec" {
            continue;
        }

        data_row += 1;
        let parts: Vec<&str> = trimmed.splitn(4, ',').collect();
        if parts.len() != 4 {
            return Err(ApparentValidationError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("expected 4 comma-separated fields, got {}", parts.len()),
            });
        }

        let body_label = parts[0].trim().to_string();
        let jd_tt: f64 =
            parts[1]
                .trim()
                .parse()
                .map_err(|_| ApparentValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("jd_tt {:?} is not a valid float", parts[1]),
                })?;
        let apparent_longitude_deg: f64 =
            parts[2]
                .trim()
                .parse()
                .map_err(|_| ApparentValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("apparent_longitude_deg {:?} is not a valid float", parts[2]),
                })?;
        let tolerance_arcsec: f64 =
            parts[3]
                .trim()
                .parse()
                .map_err(|_| ApparentValidationError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("tolerance_arcsec {:?} is not a valid float", parts[3]),
                })?;

        let body =
            resolve_body(&body_label).ok_or_else(|| ApparentValidationError::UnknownBody {
                row: data_row,
                label: body_label.clone(),
            })?;

        rows.push(GoldenRow {
            body_label,
            body,
            jd_tt,
            apparent_longitude_deg,
            tolerance_arcsec,
        });
    }

    Ok(rows)
}

/// Fail-closed cross-check of the engine's apparent-of-date longitudes against
/// JPL Horizons goldens (quantity 31).
///
/// Loads the committed goldens CSV (checksum-gated), builds a default (apparent)
/// chart per row via the packaged backend + `ChartEngine`, and compares the
/// chart's apparent ecliptic longitude to the Horizons value with a wrap-aware
/// difference. Fails closed on checksum mismatch, malformed rows, unknown bodies,
/// chart errors, or any row exceeding its per-row tolerance.
pub fn validate_apparent_goldens() -> Result<ApparentValidationReport, ApparentValidationError> {
    // Checksum gate — detects drift between the committed CSV and the pinned value.
    let actual_checksum = pleiades_apparent::fnv1a64(GOLDENS_CSV);
    if GOLDENS_CHECKSUM != 0 && actual_checksum != GOLDENS_CHECKSUM {
        return Err(ApparentValidationError::ChecksumMismatch {
            expected: GOLDENS_CHECKSUM,
            actual: actual_checksum,
        });
    }

    let rows = parse_goldens()?;
    let backend = PackagedDataBackend::new();
    let engine = ChartEngine::new(backend);

    let mut max_residual_arcsec: f64 = 0.0;

    for (idx, row) in rows.iter().enumerate() {
        let data_row = idx + 1;
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let request = ChartRequest::new(instant)
            .with_bodies(vec![row.body.clone()])
            .with_apparentness(Apparentness::Apparent);

        let snapshot = engine
            .chart(&request)
            .map_err(|e| ApparentValidationError::ChartError {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                message: e.to_string(),
            })?;

        let placement = snapshot.placement_for(&row.body).ok_or_else(|| {
            ApparentValidationError::ChartError {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                message: "body not found in chart snapshot".to_string(),
            }
        })?;

        // Fail closed if the body silently fell back to mean place: `apparent` is `None`
        // when the engine did not produce genuine apparent-mode output.  Comparing a mean
        // longitude against a Horizons apparent golden would either pass or fail for the
        // wrong reason, masking a regression.
        if placement.apparent.is_none() {
            return Err(ApparentValidationError::UnexpectedMeanFallback {
                row: data_row,
                body: row.body_label.clone(),
                jd_tt: row.jd_tt,
            });
        }

        let ecliptic = placement.position.ecliptic.as_ref().ok_or_else(|| {
            ApparentValidationError::ChartError {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                message: "ecliptic coordinates not available in chart result".to_string(),
            }
        })?;

        let got = ecliptic.longitude.degrees();
        let want = row.apparent_longitude_deg;

        // Wrap-aware absolute difference.
        let mut d = (got - want).abs();
        if d > 180.0 {
            d = 360.0 - d;
        }
        let residual_arcsec = d * 3600.0;
        if residual_arcsec > max_residual_arcsec {
            max_residual_arcsec = residual_arcsec;
        }

        if residual_arcsec > row.tolerance_arcsec {
            return Err(ApparentValidationError::ToleranceExceeded {
                row: data_row,
                body: row.body_label.clone(),
                jd: row.jd_tt,
                got,
                want,
                residual_arcsec,
                tolerance_arcsec: row.tolerance_arcsec,
            });
        }
    }

    let summary_line = format!(
        "Apparent goldens: {} rows validated, max residual {:.2}\u{2033}",
        rows.len(),
        max_residual_arcsec
    );

    Ok(ApparentValidationReport {
        rows_validated: rows.len(),
        max_residual_arcsec,
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
    fn apparent_goldens_pass() {
        validate_apparent_goldens().expect("apparent goldens within tolerance");
    }

    #[test]
    fn sun_apparent_longitude_is_of_date_1950() {
        // Of-date apparent Sun at 1950-01-01 12:00 TT (JD 2433283.0).
        // Horizons Q31 confirms 280.5144° at this epoch. The engine produces
        // ~280.508°, within 0.008° of the target value of 280.5°.
        // The old J2000-framed (non-apparent) output was 281.22° — a full ~0.7°
        // away — so this test clearly distinguishes the of-date result from the
        // stale J2000 value. Tolerance is 0.05° (~180 arcsec), comfortably above
        // the ~22 arcsec packaged-data accuracy limit (see golden CSV header comments).
        let backend = PackagedDataBackend::new();
        let engine = ChartEngine::new(backend);
        let request = ChartRequest::new(Instant::new(
            JulianDay::from_days(2_433_283.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun])
        .with_apparentness(Apparentness::Apparent);
        let snapshot = engine.chart(&request).unwrap();
        let lon = snapshot
            .placement_for(&CelestialBody::Sun)
            .unwrap()
            .position
            .ecliptic
            .as_ref()
            .unwrap()
            .longitude
            .degrees();
        // The of-date apparent value is ~280.5° (Horizons 280.5144°); the old J2000 output
        // was 281.22° — a full ~0.7° away. Tolerance is 0.05° (not 0.15°).
        let mut d = (lon - 280.5_f64).abs();
        if d > 180.0 {
            d = 360.0 - d;
        }
        assert!(
            d < 0.05,
            "apparent Sun longitude {lon}\u{00b0} not within 0.05\u{00b0} of of-date \u{2248}280.5\u{00b0} (old J2000 was 281.22\u{00b0}); diff={d}\u{00b0}"
        );
    }
}
