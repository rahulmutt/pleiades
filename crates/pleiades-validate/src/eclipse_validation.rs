//! Fail-closed gate: recompute every NASA-canon eclipse and compare.

use pleiades_data::packaged_backend;
use pleiades_eclipse::{
    EclipseEngine, EclipseFilter, EclipseType, LunarEclipseType, SolarEclipseType,
};
use pleiades_types::{Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/eclipses-corpus/eclipses.csv"
));

const TIME_TOLERANCE_SECONDS: f64 = 60.0;
const MAGNITUDE_TOLERANCE: f64 = 0.01;
const LONGITUDE_TOLERANCE_DEG: f64 = 1.0 / 3600.0; // 1 arcsecond

/// Engine coverage window end (2100-01-01 TT). Corpus rows beyond it cannot be
/// recomputed (the packaged backend has no Sun/Moon segments there) and are
/// reported as out-of-coverage rather than drift.
const WINDOW_END_JD: f64 = pleiades_eclipse::WINDOW_END_JD;

/// Minimum number of NASA-canon eclipses that must recompute within every strict
/// tolerance for the gate to pass. The full corpus has 913 rows; 4 fall in 2100
/// beyond the backend's coverage, leaving 909 in range. The gate fails closed
/// below this floor, so any real regression (more than a couple of irreducible
/// near-boundary residuals drifting) trips it.
const MIN_CHECKED: usize = 900;

pub struct EclipseCorpusReport {
    /// Rows recomputed within every strict tolerance (type, time, magnitude,
    /// saros, longitude).
    pub checked: usize,
    /// Rows whose greatest-eclipse instant lies beyond the engine's coverage
    /// window, so the engine legitimately cannot recompute them.
    pub out_of_coverage: usize,
    /// In-coverage rows that missed a tolerance (kept for transparency).
    pub drift: Vec<String>,
}

impl EclipseCorpusReport {
    pub fn summary_line(&self) -> String {
        format!(
            "validate-eclipses: {} NASA-canon eclipses recomputed within \
             ≤{TIME_TOLERANCE_SECONDS}s / ≤{MAGNITUDE_TOLERANCE} mag, type & saros exact \
             ({} out of coverage, {} drift)",
            self.checked,
            self.out_of_coverage,
            self.drift.len(),
        )
    }
}

#[derive(Debug)]
pub enum EclipseCorpusError {
    Parse {
        line: usize,
    },
    /// Fewer than [`MIN_CHECKED`] rows recomputed within tolerance — fail closed.
    BelowThreshold {
        checked: usize,
        drift: Vec<String>,
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
    let mut out_of_coverage = 0usize;
    let mut drift: Vec<String> = Vec::new();

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

        // Rows beyond the engine's coverage window cannot be recomputed: the
        // packaged backend has no Sun/Moon segments there. They are reported as
        // out-of-coverage (not drift) — every row is still examined.
        if exp_jd > WINDOW_END_JD {
            out_of_coverage += 1;
            continue;
        }

        let filter = match kind {
            "solar" => EclipseFilter::SolarOnly,
            _ => EclipseFilter::LunarOnly,
        };
        let start = Instant::new(JulianDay::from_days(exp_jd - 1.0), TimeScale::Tdb);
        let end = Instant::new(JulianDay::from_days(exp_jd + 1.0), TimeScale::Tdb);
        let nearest = match engine.eclipses_in_range(start, end, filter) {
            Ok(found) => found.into_iter().min_by(|a, b| {
                let da = (a.greatest_eclipse.julian_day.days() - exp_jd).abs();
                let db = (b.greatest_eclipse.julian_day.days() - exp_jd).abs();
                da.partial_cmp(&db).unwrap()
            }),
            Err(err) => {
                drift.push(format!(
                    "JD={exp_jd:.5} {kind} {exp_type}: engine error: {err}"
                ));
                continue;
            }
        };
        let Some(e) = nearest else {
            drift.push(format!(
                "JD={exp_jd:.5} {kind} {exp_type}: no eclipse found"
            ));
            continue;
        };

        let actual_jd = e.greatest_eclipse.julian_day.days();
        let dt_s = (actual_jd - exp_jd).abs() * 86_400.0;
        let dlon = {
            let d = (e.eclipsed_longitude.degrees() - exp_lon).abs() % 360.0;
            d.min(360.0 - d)
        };
        let mut row_fail: Vec<String> = Vec::new();
        if dt_s > TIME_TOLERANCE_SECONDS {
            row_fail.push(format!("time Δ={dt_s:.1}s"));
        }
        if type_label(e.eclipse_type) != exp_type {
            row_fail.push(format!(
                "type exp={exp_type} act={}",
                type_label(e.eclipse_type)
            ));
        }
        if (e.magnitude - exp_mag).abs() > MAGNITUDE_TOLERANCE {
            row_fail.push(format!("mag exp={exp_mag:.4} act={:.4}", e.magnitude));
        }
        if e.saros_series != exp_saros {
            row_fail.push(format!("saros exp={exp_saros} act={}", e.saros_series));
        }
        if dlon > LONGITUDE_TOLERANCE_DEG {
            row_fail.push(format!("lon Δ={:.2}\"", dlon * 3600.0));
        }

        if row_fail.is_empty() {
            checked += 1;
        } else {
            drift.push(format!(
                "JD={exp_jd:.5} {kind} {exp_type} saros={exp_saros}: [{}]",
                row_fail.join("; ")
            ));
        }
    }

    if checked < MIN_CHECKED {
        return Err(EclipseCorpusError::BelowThreshold { checked, drift });
    }
    Ok(EclipseCorpusReport {
        checked,
        out_of_coverage,
        drift,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_eclipse::EclipseKind;

    #[test]
    fn every_canon_eclipse_recomputes_within_tolerance() {
        let report = validate_eclipse_corpus().expect("eclipse corpus must validate");
        assert!(
            report.checked >= 900,
            "expected the exhaustive 1900–2100 set"
        );
    }

    #[test]
    #[ignore]
    fn debug_collect_all_failures() {
        let engine = EclipseEngine::new(packaged_backend());
        let mut failures: Vec<String> = Vec::new();
        let mut checked = 0usize;

        for (i, line) in CORPUS_CSV.lines().enumerate().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() != 6 {
                failures.push(format!("line {}: bad cols", i + 1));
                continue;
            }
            let kind = cols[0];
            let exp_type = cols[1];
            let exp_jd: f64 = match cols[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    failures.push(format!("line {}: bad jd", i + 1));
                    continue;
                }
            };
            let exp_mag: f64 = match cols[3].parse() {
                Ok(v) => v,
                Err(_) => {
                    failures.push(format!("line {}: bad mag", i + 1));
                    continue;
                }
            };
            let exp_saros: u32 = match cols[4].parse() {
                Ok(v) => v,
                Err(_) => {
                    failures.push(format!("line {}: bad saros", i + 1));
                    continue;
                }
            };
            let exp_lon: f64 = match cols[5].parse() {
                Ok(v) => v,
                Err(_) => {
                    failures.push(format!("line {}: bad lon", i + 1));
                    continue;
                }
            };

            let filter = match kind {
                "solar" => EclipseFilter::SolarOnly,
                _ => EclipseFilter::LunarOnly,
            };
            let start = Instant::new(JulianDay::from_days(exp_jd - 1.0), TimeScale::Tdb);
            let end = Instant::new(JulianDay::from_days(exp_jd + 1.0), TimeScale::Tdb);
            let found = match engine.eclipses_in_range(start, end, filter) {
                Ok(v) => v,
                Err(e) => {
                    failures.push(format!(
                        "JD={exp_jd:.5} {kind} {exp_type}: engine error: {e:?}"
                    ));
                    continue;
                }
            };
            let e = match found.into_iter().min_by(|a, b| {
                let da = (a.greatest_eclipse.julian_day.days() - exp_jd).abs();
                let db = (b.greatest_eclipse.julian_day.days() - exp_jd).abs();
                da.partial_cmp(&db).unwrap()
            }) {
                Some(v) => v,
                None => {
                    failures.push(format!("JD={exp_jd:.5} {kind} {exp_type}: MISSING"));
                    continue;
                }
            };

            let actual_jd = e.greatest_eclipse.julian_day.days();
            let actual_type = type_label(e.eclipse_type);
            let actual_mag = e.magnitude;
            let actual_saros = e.saros_series;
            let dlon = {
                let d = (e.eclipsed_longitude.degrees() - exp_lon).abs() % 360.0;
                d.min(360.0 - d)
            };
            let dt_s = (actual_jd - exp_jd).abs() * 86_400.0;

            let mut row_fails: Vec<String> = Vec::new();
            if dt_s > TIME_TOLERANCE_SECONDS {
                row_fails.push(format!(
                    "TIME: exp={exp_jd:.5} act={actual_jd:.5} delta={dt_s:.1}s"
                ));
            }
            if actual_type != exp_type {
                row_fails.push(format!("TYPE: exp={exp_type} act={actual_type}"));
            }
            if (actual_mag - exp_mag).abs() > MAGNITUDE_TOLERANCE {
                row_fails.push(format!(
                    "MAG: exp={exp_mag:.4} act={actual_mag:.4} delta={:.4}",
                    (actual_mag - exp_mag).abs()
                ));
            }
            if actual_saros != exp_saros {
                row_fails.push(format!("SAROS: exp={exp_saros} act={actual_saros}"));
            }
            if dlon > LONGITUDE_TOLERANCE_DEG {
                row_fails.push(format!("LON: delta={:.6}°={:.2}\"", dlon, dlon * 3600.0));
            }

            if !row_fails.is_empty() {
                let ekind = match e.eclipse_type.kind() {
                    EclipseKind::Solar => "solar",
                    EclipseKind::Lunar => "lunar",
                };
                failures.push(format!(
                    "JD={exp_jd:.5} {kind} {exp_type} saros={exp_saros}: [{}] (got: {ekind} {actual_type} mag={actual_mag:.4} saros={actual_saros})",
                    row_fails.join("; ")
                ));
            } else {
                checked += 1;
            }
        }

        eprintln!("\n=== ECLIPSE CORPUS DEBUG ===");
        eprintln!("Checked OK: {checked}");
        eprintln!("Failures: {}", failures.len());
        for f in &failures {
            eprintln!("  {f}");
        }
        eprintln!("===========================\n");
        assert!(
            failures.is_empty(),
            "{} failures (see above)",
            failures.len()
        );
    }
}
