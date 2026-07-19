//! Nutation in longitude (Δψ) and obliquity (Δε) from the truncated IAU-1980
//! series, plus the mean obliquity of the ecliptic. Δψ is the term the chart
//! layer applies to ecliptic longitude; Δε is exposed for equatorial callers.

use crate::error::ApparentPlaceError;
use crate::fnv1a64;

const NUTATION_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/nutation-iau1980.csv"
));

/// FNV-1a checksum of `data/nutation-iau1980.csv`; pinned in Step 4.
const NUTATION_CSV_CHECKSUM: u64 = 13689944370085337276;

/// Nutation components in arcseconds.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nutation {
    /// Nutation in longitude (Δψ), arcseconds.
    pub delta_psi_arcsec: f64,
    /// Nutation in obliquity (Δε), arcseconds.
    pub delta_eps_arcsec: f64,
}

struct Term {
    multipliers: [f64; 5],
    psi_a: f64,
    psi_b: f64,
    eps_c: f64,
    eps_d: f64,
}

fn table() -> Result<Vec<Term>, ApparentPlaceError> {
    if fnv1a64(NUTATION_CSV) != NUTATION_CSV_CHECKSUM {
        return Err(ApparentPlaceError::StaleModelData { kind: "nutation" });
    }
    let mut terms = Vec::new();
    for line in NUTATION_CSV.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<f64> = line
            .split(',')
            .map(|s| s.trim().parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ApparentPlaceError::StaleModelData { kind: "nutation" })?;
        if cols.len() != 9 {
            return Err(ApparentPlaceError::StaleModelData { kind: "nutation" });
        }
        terms.push(Term {
            multipliers: [cols[0], cols[1], cols[2], cols[3], cols[4]],
            psi_a: cols[5],
            psi_b: cols[6],
            eps_c: cols[7],
            eps_d: cols[8],
        });
    }
    Ok(terms)
}

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - 2_451_545.0) / 36_525.0
}

/// Fundamental arguments (Meeus 22.x), returned in degrees.
fn fundamental_arguments(t: f64) -> [f64; 5] {
    let d = 297.85036 + 445_267.111_480 * t - 0.001_914_2 * t * t + t * t * t / 189_474.0;
    let m = 357.52772 + 35_999.050_340 * t - 0.000_160_3 * t * t - t * t * t / 300_000.0;
    let m1 = 134.96298 + 477_198.867_398 * t + 0.008_697_2 * t * t + t * t * t / 56_250.0;
    let f = 93.27191 + 483_202.017_538 * t - 0.003_682_5 * t * t + t * t * t / 327_270.0;
    let om = 125.04452 - 1_934.136_261 * t + 0.002_070_8 * t * t + t * t * t / 450_000.0;
    [d, m, m1, f, om]
}

/// Mean obliquity of the ecliptic (Meeus 22.2), degrees.
pub fn mean_obliquity_degrees(jd_tt: f64) -> f64 {
    let t = julian_centuries(jd_tt);
    // 23°26'21.448" expressed in arcseconds, minus the polynomial.
    let eps_arcsec = 84_381.448 - 46.8150 * t - 0.000_59 * t * t + 0.001_813 * t * t * t;
    eps_arcsec / 3600.0
}

/// Nutation in longitude and obliquity for a TT Julian Day.
pub fn nutation(jd_tt: f64) -> Result<Nutation, ApparentPlaceError> {
    let t = julian_centuries(jd_tt);
    let args = fundamental_arguments(t);
    let terms = table()?;
    let mut psi = 0.0_f64; // in 0.0001"
    let mut eps = 0.0_f64; // in 0.0001"
    for term in &terms {
        let mut arg_deg = 0.0;
        for (i, mult) in term.multipliers.iter().enumerate() {
            arg_deg += mult * args[i];
        }
        let arg = arg_deg.to_radians();
        psi += (term.psi_a + term.psi_b * t) * arg.sin();
        eps += (term.eps_c + term.eps_d * t) * arg.cos();
    }
    let delta_psi_arcsec = psi * 0.0001;
    let delta_eps_arcsec = eps * 0.0001;
    if !delta_psi_arcsec.is_finite() || !delta_eps_arcsec.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "nutation" });
    }
    Ok(Nutation {
        delta_psi_arcsec,
        delta_eps_arcsec,
    })
}

#[cfg(test)]
mod tests;
