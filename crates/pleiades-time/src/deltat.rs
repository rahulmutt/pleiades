//! Delta-T (`ΔT = TT − UT1`): checksum-pinned observed table with linear
//! interpolation, plus a documented polynomial extrapolation beyond it.
//!
//! Beyond the observed table the Espenak–Meeus 2005–2050 future fit is reused
//! (extrapolated) all the way out to the 2100 horizon; it is increasingly
//! approximate past 2050, so every extrapolated value is tagged `Predicted`.

use std::sync::OnceLock;

use crate::error::CivilTimeError;
use crate::fnv1a64;

const DELTA_T_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/delta-t-observed.csv"
));

/// FNV-1a checksum of `data/delta-t-observed.csv`; pinned in Step 4.
const DELTA_T_CSV_CHECKSUM: u64 = 17446600357888055970; // pinned

/// JD of the last observed ΔT node (2020-01-01 00:00). Beyond this, ΔT is `Predicted`.
pub const OBSERVED_THROUGH_JD: f64 = 2458849.5;

/// Quality of a Delta-T value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeltaTQuality {
    /// Interpolated from the committed observation table.
    Observed,
    /// Extrapolated by the documented polynomial beyond the observed table.
    Predicted,
}

static DELTA_T_ROWS: OnceLock<Result<Vec<(f64, f64)>, CivilTimeError>> = OnceLock::new();

fn parse_table() -> Result<Vec<(f64, f64)>, CivilTimeError> {
    if fnv1a64(DELTA_T_CSV) != DELTA_T_CSV_CHECKSUM {
        return Err(CivilTimeError::StaleTimeData { kind: "delta-t" });
    }
    let mut rows = Vec::new();
    for line in DELTA_T_CSV.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(',');
        let year: f64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or(CivilTimeError::StaleTimeData { kind: "delta-t" })?;
        let dt: f64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or(CivilTimeError::StaleTimeData { kind: "delta-t" })?;
        rows.push((year, dt));
    }
    Ok(rows)
}

fn table() -> Result<&'static [(f64, f64)], CivilTimeError> {
    DELTA_T_ROWS
        .get_or_init(parse_table)
        .as_deref()
        .map_err(|e| *e)
}

/// Approximate decimal year from a Julian Day (good enough for ΔT, which varies slowly).
fn decimal_year(jd: f64) -> f64 {
    2000.0 + (jd - 2451545.0) / 365.25
}

/// Espenak–Meeus future extrapolation, using the 2005–2050 polynomial form.
/// This is the only extrapolator used beyond the observed table; it is reused
/// (extrapolated) out to the 2100 horizon and is increasingly approximate past
/// its 2050 fit range. See https://eclipse.gsfc.nasa.gov/SEcat5/deltatpoly.html
fn extrapolate(year: f64) -> f64 {
    let t = year - 2000.0;
    62.92 + 0.32217 * t + 0.005589 * t * t
}

/// Returns `(ΔT seconds, quality)` for a Julian Day. The orchestrator is
/// responsible for the 1900–2100 horizon; this function extrapolates past the
/// observed table without an upper bound.
pub fn delta_t(jd: f64) -> Result<(f64, DeltaTQuality), CivilTimeError> {
    let year = decimal_year(jd);
    let rows = table()?;
    let first = rows[0];
    if year <= first.0 {
        return Ok((first.1, DeltaTQuality::Observed));
    }
    if jd >= OBSERVED_THROUGH_JD {
        // Past the observed table -> predicted extrapolation.
        return Ok((extrapolate(year), DeltaTQuality::Predicted));
    }
    // Linear interpolation between bracketing observed nodes.
    for pair in rows.windows(2) {
        let (y0, d0) = pair[0];
        let (y1, d1) = pair[1];
        if year >= y0 && year <= y1 {
            let frac = (year - y0) / (y1 - y0);
            return Ok((d0 + frac * (d1 - d0), DeltaTQuality::Observed));
        }
    }
    unreachable!("year is between first and last nodes")
}

#[cfg(test)]
mod tests;
