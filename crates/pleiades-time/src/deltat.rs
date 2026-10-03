//! Delta-T (`ΔT = TT − UT1`) in three tiers by epoch:
//!
//! 1. **Observed** — the checksum-pinned observation table (decade nodes
//!    1900–2020), linearly interpolated.
//! 2. **Leap-second bound** — from the 2020 node while the leap-second table
//!    is authoritative: `ΔT = TT − UTC − DUT1` with `TT − UTC = 32.184 s +
//!    (TAI − UTC)` known exactly and `DUT1` taken as zero, so the value is
//!    within `|DUT1| < 0.9 s` of truth. The observed table gives 69.4 s at the
//!    2020 node and the bound 69.184 s, a 0.2 s step of the same order as the
//!    table's own interpolation error.
//! 3. **Predicted** — beyond the leap horizon, the Espenak–Meeus 2005–2050
//!    polynomial anchored to the leap-second bound at the horizon so ΔT is
//!    continuous there; it is reused (extrapolated) out to the 2100 horizon
//!    and is increasingly approximate past 2050.

use std::sync::OnceLock;

use crate::error::CivilTimeError;
use crate::fnv1a64;
use crate::leap;

/// TT − TAI, in seconds (fixed by definition).
pub(crate) const TT_MINUS_TAI: f64 = 32.184;

const DELTA_T_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/delta-t-observed.csv"
));

/// FNV-1a checksum of `data/delta-t-observed.csv`; pinned in Step 4.
const DELTA_T_CSV_CHECKSUM: u64 = 17446600357888055970; // pinned

/// JD of the last observed ΔT node (2020-01-01 00:00). From this day on, ΔT is
/// `LeapSecondBound` while the leap-second table is authoritative
/// (`leap::VALID_THROUGH_JD`) and `Predicted` beyond that.
pub const OBSERVED_THROUGH_JD: f64 = 2458849.5;

/// Quality of a Delta-T value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeltaTQuality {
    /// Interpolated from the committed observation table.
    Observed,
    /// `32.184 s + (TAI − UTC)` from the leap-second table with DUT1 taken as
    /// zero; within 0.9 s of the true value.
    LeapSecondBound,
    /// Extrapolated by the documented polynomial beyond the leap-second table.
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
/// This is the only extrapolator used beyond the leap-second table; it is
/// reused (extrapolated) out to the 2100 horizon and is increasingly
/// approximate past its 2050 fit range. Callers anchor it (see
/// [`past_observed_table`]) rather than using its absolute value. See
/// https://eclipse.gsfc.nasa.gov/SEcat5/deltatpoly.html
fn extrapolate(year: f64) -> f64 {
    let t = year - 2000.0;
    62.92 + 0.32217 * t + 0.005589 * t * t
}

/// `ΔT` implied by a leap-second count with DUT1 taken as zero.
fn leap_second_bound(tai_minus_utc: i32) -> f64 {
    TT_MINUS_TAI + f64::from(tai_minus_utc)
}

/// ΔT from the 2020 node on: the leap-second bound while the leap table is
/// authoritative, then the polynomial anchored to the bound at the horizon.
///
/// The leap lookup takes a UTC day but `jd` may be TT-scale (see
/// `ut1_jd_from_tt`); the two differ by ΔT ≈ 69 s, which only matters within
/// 69 s of a leap epoch, where ΔT itself steps by a full second.
fn past_observed_table(jd: f64) -> Result<(f64, DeltaTQuality), CivilTimeError> {
    if let Some(tai_minus_utc) = leap::tai_minus_utc(jd)? {
        return Ok((
            leap_second_bound(tai_minus_utc),
            DeltaTQuality::LeapSecondBound,
        ));
    }
    let horizon_jd = leap::VALID_THROUGH_JD;
    let anchor = leap_second_bound(leap::last_tai_minus_utc()?);
    let predicted = anchor + extrapolate(decimal_year(jd)) - extrapolate(decimal_year(horizon_jd));
    Ok((predicted, DeltaTQuality::Predicted))
}

/// Returns `(ΔT seconds, quality)` for a Julian Day. The orchestrator is
/// responsible for the 1900–2100 horizon; this function extrapolates past the
/// leap-second table without an upper bound.
pub fn delta_t(jd: f64) -> Result<(f64, DeltaTQuality), CivilTimeError> {
    let year = decimal_year(jd);
    let rows = table()?;
    let first = rows[0];
    if year <= first.0 {
        return Ok((first.1, DeltaTQuality::Observed));
    }
    if jd >= OBSERVED_THROUGH_JD {
        return past_observed_table(jd);
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
