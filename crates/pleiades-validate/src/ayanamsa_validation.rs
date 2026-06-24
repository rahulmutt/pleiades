//! Fail-closed numeric-residual gate for the six release-claimed ayanamsa modes
//! against the committed Swiss Ephemeris reference corpus. Mirrors
//! `house_validation.rs`: parse → checksum → manifest drift → per-row residual
//! vs the per-mode-class ceiling. Pure-Rust; no SE or network dependency.
use core::fmt;

use pleiades_ayanamsa::sidereal_offset;
use pleiades_ayanamsa::thresholds::{ayanamsa_mode_ceiling, ayanamsa_mode_class};
use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/ayanamsa-corpus/ayanamsa.csv"
));
const CORPUS_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/ayanamsa-corpus/manifest.txt"
));

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaCorpusRow {
    pub(crate) mode_code: String,
    pub(crate) jd_tt: f64,
    pub(crate) se_ayanamsa_deg: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AyanamsaCorpusError {
    MalformedRow {
        row: usize,
        line: String,
        reason: String,
    },
    MalformedManifest {
        reason: String,
    },
    ChecksumMismatch {
        expected: u64,
        actual: u64,
    },
    ManifestDrift {
        field: String,
        expected: String,
        actual: String,
    },
    UnknownModeCode {
        row: usize,
        code: String,
    },
    CalculationFailed {
        row: usize,
        mode: String,
    },
    CeilingExceeded {
        row: usize,
        mode: String,
        got: f64,
        want: f64,
        residual_arcsec: f64,
        ceiling_arcsec: f64,
    },
}

impl fmt::Display for AyanamsaCorpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRow { row, line, reason } =>
                write!(f, "ayanamsa corpus row {row} is malformed ({reason}): {line:?}"),
            Self::MalformedManifest { reason } =>
                write!(f, "ayanamsa corpus manifest is malformed: {reason}"),
            Self::ChecksumMismatch { expected, actual } =>
                write!(f, "ayanamsa corpus checksum mismatch: expected {expected}, got {actual}"),
            Self::ManifestDrift { field, expected, actual } =>
                write!(f, "ayanamsa corpus manifest drift on field '{field}': expected {expected:?}, got {actual:?}"),
            Self::UnknownModeCode { row, code } =>
                write!(f, "ayanamsa corpus row {row}: unknown mode code {code:?}"),
            Self::CalculationFailed { row, mode } =>
                write!(f, "ayanamsa corpus row {row} ({mode}): sidereal_offset returned None"),
            Self::CeilingExceeded { row, mode, got, want, residual_arcsec, ceiling_arcsec } =>
                write!(f, "ayanamsa corpus row {row} ({mode}): residual {residual_arcsec:.3}\u{2033} > ceiling {ceiling_arcsec:.1}\u{2033} (got={got:.9}\u{00b0}, want={want:.9}\u{00b0})"),
        }
    }
}
impl std::error::Error for AyanamsaCorpusError {}

#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaCorpusReport {
    pub rows_validated: usize,
    pub modes_checked: usize,
    pub max_residual_arcsec: f64,
    pub summary_line: String,
}
impl AyanamsaCorpusReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}
impl fmt::Display for AyanamsaCorpusReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

pub(crate) fn parse_corpus(csv: &str) -> Result<Vec<AyanamsaCorpusRow>, AyanamsaCorpusError> {
    let mut rows = Vec::new();
    let mut data_row = 0usize;
    for line in csv.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("mode_code,") {
            continue;
        }
        data_row += 1;
        let parts: Vec<&str> = t.split(',').collect();
        if parts.len() != 3 {
            return Err(AyanamsaCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("expected 3 fields, got {}", parts.len()),
            });
        }
        let jd_tt = parts[1]
            .trim()
            .parse()
            .map_err(|_| AyanamsaCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("jd_tt {:?} is not a valid float", parts[1]),
            })?;
        let se_ayanamsa_deg =
            parts[2]
                .trim()
                .parse()
                .map_err(|_| AyanamsaCorpusError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("se_ayanamsa_deg {:?} is not a valid float", parts[2]),
                })?;
        rows.push(AyanamsaCorpusRow {
            mode_code: parts[0].trim().to_string(),
            jd_tt,
            se_ayanamsa_deg,
        });
    }
    Ok(rows)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaManifest {
    pub(crate) rows: usize,
    pub(crate) checksum: u64,
}

pub(crate) fn parse_manifest(text: &str) -> Result<AyanamsaManifest, AyanamsaCorpusError> {
    let mut rows = None;
    let mut checksum = None;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("slice ") {
            for tok in t.split_whitespace() {
                if let Some(v) = tok.strip_prefix("rows=") {
                    rows = Some(
                        v.parse()
                            .map_err(|_| AyanamsaCorpusError::MalformedManifest {
                                reason: format!("rows value {v:?} is not a valid usize"),
                            })?,
                    );
                } else if let Some(v) = tok.strip_prefix("checksum=") {
                    checksum =
                        Some(
                            v.parse()
                                .map_err(|_| AyanamsaCorpusError::MalformedManifest {
                                    reason: format!("checksum value {v:?} is not a valid u64"),
                                })?,
                        );
                }
            }
        }
    }
    Ok(AyanamsaManifest {
        rows: rows.ok_or(AyanamsaCorpusError::MalformedManifest {
            reason: "rows= not found".into(),
        })?,
        checksum: checksum.ok_or(AyanamsaCorpusError::MalformedManifest {
            reason: "checksum= not found".into(),
        })?,
    })
}

fn mode_for_code(code: &str) -> Option<Ayanamsa> {
    match code {
        "Lahiri" => Some(Ayanamsa::Lahiri),
        "Raman" => Some(Ayanamsa::Raman),
        "Krishnamurti" => Some(Ayanamsa::Krishnamurti),
        "FaganBradley" => Some(Ayanamsa::FaganBradley),
        "TrueChitra" => Some(Ayanamsa::TrueChitra),
        "TrueCitra" => Some(Ayanamsa::TrueCitra),
        _ => None,
    }
}

fn wrap_arcsec(got: f64, want: f64) -> f64 {
    let mut d = (got - want).abs();
    if d > 180.0 {
        d = 360.0 - d;
    }
    d * 3600.0
}

/// Runs the fail-closed ayanamsa numeric-residual gate over the committed corpus.
pub fn validate_ayanamsa_corpus() -> Result<AyanamsaCorpusReport, AyanamsaCorpusError> {
    let actual = pleiades_apparent::fnv1a64(CORPUS_CSV);
    let manifest = parse_manifest(CORPUS_MANIFEST)?;
    if actual != manifest.checksum {
        return Err(AyanamsaCorpusError::ChecksumMismatch {
            expected: manifest.checksum,
            actual,
        });
    }
    let rows = parse_corpus(CORPUS_CSV)?;
    if rows.len() != manifest.rows {
        return Err(AyanamsaCorpusError::ManifestDrift {
            field: "rows".into(),
            expected: manifest.rows.to_string(),
            actual: rows.len().to_string(),
        });
    }
    // Completeness: all six gated modes present.
    for code in [
        "Lahiri",
        "Raman",
        "Krishnamurti",
        "FaganBradley",
        "TrueChitra",
        "TrueCitra",
    ] {
        if !rows.iter().any(|r| r.mode_code == code) {
            return Err(AyanamsaCorpusError::ManifestDrift {
                field: "completeness".into(),
                expected: format!("rows for {code}"),
                actual: "missing".into(),
            });
        }
    }
    let mut max_residual_arcsec = 0.0_f64;
    let mut modes = std::collections::BTreeSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let data_row = idx + 1;
        let mode =
            mode_for_code(&row.mode_code).ok_or_else(|| AyanamsaCorpusError::UnknownModeCode {
                row: data_row,
                code: row.mode_code.clone(),
            })?;
        modes.insert(row.mode_code.clone());
        let class =
            ayanamsa_mode_class(&mode).ok_or_else(|| AyanamsaCorpusError::UnknownModeCode {
                row: data_row,
                code: row.mode_code.clone(),
            })?;
        let ceiling = ayanamsa_mode_ceiling(class);
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let got = sidereal_offset(&mode, instant)
            .ok_or_else(|| AyanamsaCorpusError::CalculationFailed {
                row: data_row,
                mode: row.mode_code.clone(),
            })?
            .degrees();
        let resid = wrap_arcsec(got, row.se_ayanamsa_deg);
        if resid > max_residual_arcsec {
            max_residual_arcsec = resid;
        }
        if resid > ceiling.offset_arcsec {
            return Err(AyanamsaCorpusError::CeilingExceeded {
                row: data_row,
                mode: row.mode_code.clone(),
                got,
                want: row.se_ayanamsa_deg,
                residual_arcsec: resid,
                ceiling_arcsec: ceiling.offset_arcsec,
            });
        }
    }
    let summary_line =
        format!(
        "Ayanamsa gate: {} rows, {} modes validated vs Swiss Ephemeris, max residual {:.3}\u{2033}",
        rows.len(), modes.len(), max_residual_arcsec
    );
    Ok(AyanamsaCorpusReport {
        rows_validated: rows.len(),
        modes_checked: modes.len(),
        max_residual_arcsec,
        summary_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_ayanamsa::thresholds::AyanamsaModeClass;

    #[test]
    fn gate_passes_over_committed_corpus() {
        let report = validate_ayanamsa_corpus().expect("ayanamsa gate should pass");
        assert_eq!(report.modes_checked, 6);
        assert!(report.summary_line().starts_with("Ayanamsa gate"));
    }

    #[test]
    fn checksum_drift_fails_closed() {
        // The committed checksum must equal the recomputed CSV checksum.
        let manifest = parse_manifest(CORPUS_MANIFEST).unwrap();
        assert_eq!(manifest.checksum, pleiades_apparent::fnv1a64(CORPUS_CSV));
    }

    #[test]
    fn malformed_row_fails_closed() {
        let err =
            parse_corpus("mode_code,jd_tt,se_ayanamsa_deg\nLahiri,not_a_float,23.0\n").unwrap_err();
        assert!(matches!(err, AyanamsaCorpusError::MalformedRow { .. }));
    }

    #[test]
    fn unknown_mode_code_fails_closed() {
        assert!(mode_for_code("Bogus").is_none());
    }

    /// Prove the gate fails closed on a residual that exceeds the mode-class ceiling.
    ///
    /// This tests the ceiling-comparison logic directly using the in-module helpers,
    /// without injecting rows into the committed corpus. A synthetic pair where
    /// `got` is 10° off from `want` produces a 36000″ residual, which clearly
    /// exceeds the 2.0″ OffsetDefined ceiling and the 1.0″ TrueStar ceiling.
    ///
    /// This exercises the same `wrap_arcsec(got, want) > ceiling.offset_arcsec`
    /// condition that `validate_ayanamsa_corpus()` evaluates per-row, proving
    /// the gate would produce `CeilingExceeded` on a corrupted reference value.
    #[test]
    fn ceiling_exceeded_logic_fails_closed_on_large_residual() {
        // A synthetic 10° error produces 36 000″ — far above any mode-class ceiling.
        let got = 23.0_f64;
        let want = 33.0_f64; // 10° off
        let residual = wrap_arcsec(got, want);
        assert_eq!(residual, 36_000.0, "10° difference should be 36 000″");

        // Verify the residual exceeds both mode-class ceilings.
        let offset_ceiling = ayanamsa_mode_ceiling(AyanamsaModeClass::OffsetDefined);
        let truestar_ceiling = ayanamsa_mode_ceiling(AyanamsaModeClass::TrueStar);
        assert!(
            residual > offset_ceiling.offset_arcsec,
            "36 000″ should exceed OffsetDefined ceiling {:.1}″",
            offset_ceiling.offset_arcsec
        );
        assert!(
            residual > truestar_ceiling.offset_arcsec,
            "36 000″ should exceed TrueStar ceiling {:.1}″",
            truestar_ceiling.offset_arcsec
        );

        // Confirm a small residual (well within tolerance) does NOT exceed the ceiling,
        // so the test is non-vacuous in both directions.
        let small_residual = wrap_arcsec(23.0, 23.0 + 0.0001); // ~0.36″
        assert!(
            small_residual < offset_ceiling.offset_arcsec,
            "0.36″ should be within OffsetDefined ceiling"
        );
    }
}
