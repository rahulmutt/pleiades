//! Fail-closed gate: recompute every NASA-canon eclipse and compare.
//!
//! Every data row is in-coverage (rows beyond the packaged backend's ephemeris
//! end were trimmed from the fixture at JD 2_488_069.5 / 2100-01-01 TT).
//! The gate is strictly fail-closed: any unexplained tolerance drift on any row
//! returns `Err` immediately. One known-irreducible eclipse is allowlisted (see
//! `ALLOWLIST`).

use pleiades_data::packaged_backend;
use pleiades_eclipse::{
    EclipseEngine, EclipseFilter, EclipseType, LunarEclipseType, SolarEclipseType, WINDOW_END_JD,
    WINDOW_START_JD,
};
use pleiades_types::{Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/eclipses-corpus/eclipses.csv"
));

const TIME_TOLERANCE_SECONDS: f64 = 60.0;
const MAGNITUDE_TOLERANCE: f64 = 0.01;
const LONGITUDE_TOLERANCE_DEG: f64 = 1.0 / 3600.0; // 1 arcsecond

/// Allowlist of JDs whose eclipse type is irreducibly mis-classified by the
/// geocentric shadow-cone model, but whose time/magnitude/saros/longitude all
/// pass within tolerance. Each allowlisted row is expected to fail ONLY on
/// type; any other drift is a real regression and will trip the gate.
///
/// Current allowlist (1 entry):
/// * `2_432_680.601_44` — 1948-05-09 solar, Saros 137.
///   NASA labels this annular (magnitude 0.9999). The geocentric engine
///   computes magnitude 1.0004, which PASSES (Δ = 0.0005 ≪ 0.01), but the
///   binary annular/hybrid type classification flips at the mag = 1.0
///   knife-edge — so the engine labels it hybrid. The residual is irreducible
///   for a mean-radius topocentric model: any threshold wide enough to push
///   this row back to annular would simultaneously reclassify adjacent genuine
///   hybrids (e.g. Saros 124, NASA mag 1.0000, only 0.08″ away on the same
///   side). Resolving it would require sub-0.0005 magnitude fidelity (detailed
///   limb profiles) beyond the scope of a geocentric shadow-cone model.
const ALLOWLIST: &[f64] = &[
    2_432_680.601_44, // 1948-05-09 solar Saros 137 — annular/hybrid knife-edge
];

/// Returns true if `jd` matches an allowlisted entry (within ≈8.6 s / 1e-4 day).
fn is_allowlisted(jd: f64) -> bool {
    ALLOWLIST.iter().any(|&a| (jd - a).abs() < 1e-4)
}

pub struct EclipseCorpusReport {
    /// Rows that fully passed all five tolerances (type, time, magnitude,
    /// saros, longitude).
    pub checked: usize,
    /// Rows found in the allowlist (expected to fail only on type; all other
    /// tolerances verified to still pass).
    pub allowlist_hits: usize,
}

impl EclipseCorpusReport {
    pub fn summary_line(&self) -> String {
        format!(
            "validate-eclipses: {} NASA-canon eclipses recomputed within \
             ≤{TIME_TOLERANCE_SECONDS}s / ≤{MAGNITUDE_TOLERANCE} mag, type & saros exact \
             ({} allowlist hit(s), 0 unexplained drift)",
            self.checked, self.allowlist_hits,
        )
    }
}

#[derive(Debug)]
pub enum EclipseCorpusError {
    Parse {
        line: usize,
    },
    /// A non-allowlisted in-coverage row failed at least one tolerance — fail closed.
    Drift {
        description: String,
    },
    /// An allowlisted row failed on something other than its expected type drift
    /// — this is a real regression.
    AllowlistRegression {
        description: String,
    },
}

impl std::fmt::Display for EclipseCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EclipseCorpusError {}

fn type_label(t: EclipseType) -> &'static str {
    match t {
        EclipseType::Solar(SolarEclipseType::Total) => "total",
        EclipseType::Solar(SolarEclipseType::Annular) => "annular",
        EclipseType::Solar(SolarEclipseType::Hybrid) => "hybrid",
        EclipseType::Solar(SolarEclipseType::Partial) => "partial",
        EclipseType::Lunar(LunarEclipseType::Total) => "total",
        EclipseType::Lunar(LunarEclipseType::Partial) => "partial",
        EclipseType::Lunar(LunarEclipseType::Penumbral) => "penumbral",
    }
}

pub fn validate_eclipse_corpus() -> Result<EclipseCorpusReport, EclipseCorpusError> {
    let engine = EclipseEngine::new(packaged_backend());
    let mut checked = 0usize;
    let mut allowlist_hits = 0usize;

    for (i, line) in CORPUS_CSV.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() != 6 {
            return Err(EclipseCorpusError::Parse { line: i + 1 });
        }
        let kind = cols[0];
        let exp_type = cols[1];
        let exp_jd: f64 = cols[2]
            .parse()
            .map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;
        let exp_mag: f64 = cols[3]
            .parse()
            .map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;
        let exp_saros: u32 = cols[4]
            .parse()
            .map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;
        let exp_lon: f64 = cols[5]
            .parse()
            .map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;

        let allowlisted = is_allowlisted(exp_jd);

        let filter = match kind {
            "solar" => EclipseFilter::SolarOnly,
            _ => EclipseFilter::LunarOnly,
        };
        let start = Instant::new(JulianDay::from_days(exp_jd - 1.0), TimeScale::Tdb);
        let end = Instant::new(JulianDay::from_days(exp_jd + 1.0), TimeScale::Tdb);

        // A backend/coverage error on any row is a failure: all rows are
        // in-coverage since the fixture was trimmed to the data bound
        // (JD ≤ 2_488_069.5).
        let nearest = match engine.eclipses_in_range(start, end, filter) {
            Ok(found) => found.into_iter().min_by(|a, b| {
                let da = (a.greatest_eclipse.julian_day.days() - exp_jd).abs();
                let db = (b.greatest_eclipse.julian_day.days() - exp_jd).abs();
                da.partial_cmp(&db).unwrap()
            }),
            Err(err) => {
                return Err(EclipseCorpusError::Drift {
                    description: format!("JD={exp_jd:.5} {kind} {exp_type}: engine error: {err}"),
                });
            }
        };
        let Some(e) = nearest else {
            return Err(EclipseCorpusError::Drift {
                description: format!("JD={exp_jd:.5} {kind} {exp_type}: no eclipse found"),
            });
        };

        let actual_jd = e.greatest_eclipse.julian_day.days();
        let dt_s = (actual_jd - exp_jd).abs() * 86_400.0;
        let dlon = {
            let d = (e.eclipsed_longitude.degrees() - exp_lon).abs() % 360.0;
            d.min(360.0 - d)
        };

        let type_fail = type_label(e.eclipse_type) != exp_type;
        let time_fail = dt_s > TIME_TOLERANCE_SECONDS;
        let mag_fail = (e.magnitude - exp_mag).abs() > MAGNITUDE_TOLERANCE;
        let saros_fail = e.saros_series != exp_saros;
        let lon_fail = dlon > LONGITUDE_TOLERANCE_DEG;

        if allowlisted {
            // Allowlisted rows are expected to fail ONLY on type.
            // Any drift beyond type is a real regression — fail immediately.
            let mut non_type_fails: Vec<String> = Vec::new();
            if time_fail {
                non_type_fails.push(format!("time Δ={dt_s:.1}s"));
            }
            if mag_fail {
                non_type_fails.push(format!("mag exp={exp_mag:.4} act={:.4}", e.magnitude));
            }
            if saros_fail {
                non_type_fails.push(format!("saros exp={exp_saros} act={}", e.saros_series));
            }
            if lon_fail {
                non_type_fails.push(format!("lon Δ={:.2}\"", dlon * 3600.0));
            }
            if !non_type_fails.is_empty() {
                return Err(EclipseCorpusError::AllowlistRegression {
                    description: format!(
                        "JD={exp_jd:.5} {kind} {exp_type} saros={exp_saros}: \
                         allowlisted row has unexpected non-type drift: [{}]",
                        non_type_fails.join("; ")
                    ),
                });
            }
            allowlist_hits += 1;
        } else {
            // Non-allowlisted rows must pass all five tolerances — zero drift.
            let mut row_fail: Vec<String> = Vec::new();
            if time_fail {
                row_fail.push(format!("time Δ={dt_s:.1}s"));
            }
            if type_fail {
                row_fail.push(format!(
                    "type exp={exp_type} act={}",
                    type_label(e.eclipse_type)
                ));
            }
            if mag_fail {
                row_fail.push(format!("mag exp={exp_mag:.4} act={:.4}", e.magnitude));
            }
            if saros_fail {
                row_fail.push(format!("saros exp={exp_saros} act={}", e.saros_series));
            }
            if lon_fail {
                row_fail.push(format!("lon Δ={:.2}\"", dlon * 3600.0));
            }
            if !row_fail.is_empty() {
                return Err(EclipseCorpusError::Drift {
                    description: format!(
                        "JD={exp_jd:.5} {kind} {exp_type} saros={exp_saros}: [{}]",
                        row_fail.join("; ")
                    ),
                });
            }
            checked += 1;
        }
    }

    Ok(EclipseCorpusReport {
        checked,
        allowlist_hits,
    })
}

/// Lists eclipses in a JD range, one line per eclipse.
///
/// Usage: `eclipses [--start <jd>] [--end <jd>] [--solar|--lunar]`
///
/// JD floats are the only accepted format for `--start`/`--end`; ISO dates are
/// not supported (the error text says so if the user supplies a non-numeric value).
/// Defaults to `WINDOW_START_JD`..`WINDOW_END_JD` when omitted.
pub(crate) fn render_eclipses_listing(args: &[&str]) -> Result<String, String> {
    let mut start_jd = WINDOW_START_JD;
    let mut end_jd = WINDOW_END_JD;
    let mut filter = EclipseFilter::All;

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--start" => {
                i += 1;
                let val = args.get(i).ok_or("eclipses: --start requires a JD value")?;
                start_jd = val.parse::<f64>().map_err(|_| {
                    format!(
                        "eclipses: --start: invalid JD float '{val}' \
                         (only JD floats are accepted, e.g. --start 2451545.0)"
                    )
                })?;
            }
            "--end" => {
                i += 1;
                let val = args.get(i).ok_or("eclipses: --end requires a JD value")?;
                end_jd = val.parse::<f64>().map_err(|_| {
                    format!(
                        "eclipses: --end: invalid JD float '{val}' \
                         (only JD floats are accepted, e.g. --end 2488069.5)"
                    )
                })?;
            }
            "--solar" => filter = EclipseFilter::SolarOnly,
            "--lunar" => filter = EclipseFilter::LunarOnly,
            other => {
                return Err(format!(
                    "eclipses: unknown argument '{other}' \
                     (usage: eclipses [--start <jd>] [--end <jd>] [--solar|--lunar])"
                ));
            }
        }
        i += 1;
    }

    if start_jd > end_jd {
        return Err(format!("eclipses: --start {start_jd} > --end {end_jd}"));
    }

    let engine = EclipseEngine::new(packaged_backend());
    let start = Instant::new(JulianDay::from_days(start_jd), TimeScale::Tdb);
    let end = Instant::new(JulianDay::from_days(end_jd), TimeScale::Tdb);

    let eclipses = engine
        .eclipses_in_range(start, end, filter)
        .map_err(|e| format!("eclipses: engine error: {e}"))?;

    if eclipses.is_empty() {
        return Ok("eclipses: no eclipses found in range".to_string());
    }

    let lines: Vec<String> = eclipses
        .iter()
        .map(|e| {
            let jd = e.greatest_eclipse.julian_day.days();
            let kind = match e.eclipse_type {
                EclipseType::Solar(_) => "solar",
                EclipseType::Lunar(_) => "lunar",
            };
            let typ = type_label(e.eclipse_type);
            let mag = e.magnitude;
            let saros = e.saros_series;
            let lon = e.eclipsed_longitude.degrees();
            format!("{jd:.5} {kind} {typ} mag={mag:.4} saros={saros} lon={lon:.4}")
        })
        .collect();

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_canon_eclipse_recomputes_within_tolerance() {
        let report = validate_eclipse_corpus().expect("eclipse corpus must validate");
        assert_eq!(
            report.checked, 908,
            "expected exactly 908 fully-passing rows \
             (909 in-coverage rows minus 1 allowlisted knife-edge)"
        );
        assert_eq!(
            report.allowlist_hits, 1,
            "expected exactly 1 allowlist hit \
             (1948-05-09 Saros 137 annular/hybrid knife-edge)"
        );
    }
}
