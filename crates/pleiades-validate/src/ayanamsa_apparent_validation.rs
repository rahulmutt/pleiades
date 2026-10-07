//! Fail-closed gate: pleiades' apparent-star ayanamsa correction
//! ([`pleiades_core::apparent_star_ayanamsa_correction`], apparent minus mean
//! ayanamsa) vs the committed Swiss Ephemeris corpus (issue #164 (c)).
//!
//! Swiss Ephemeris' side is the ayanamsa with `SEFLG_MOSEPH | SEFLG_NONUT`
//! (the apparent, nutation-free anchor-star place `SEFLG_SIDEREAL` uses)
//! less the ayanamsa with `SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL`
//! added (the geometric place: the mean ayanamsa). Nine star-anchored modes
//! carry 60 rows spread over 1900–2100 and 50 rows within a day of ten
//! conjunctions of the anchor star with the Sun, where light deflection
//! peaks; five galactic-equator modes Swiss Ephemeris does not aberrate carry
//! 10 rows each, held to a zero correction: a corpus row of exactly zero
//! requires pleiades to report no correction at all (`None`), and any other
//! row requires one (issue #226). See `ayanamsa_apparent_thresholds`
//! for the basis of the ceilings.

use crate::ayanamsa_apparent_thresholds::{CONJUNCTION_CEILING_ARCSEC, UNIFORM_CEILING_ARCSEC};
use pleiades_apparent::fnv1a64;
use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/ayanamsa-apparent-corpus/ayanamsa-apparent.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/ayanamsa-apparent-corpus/manifest.txt"
));

/// Fail-closed floor on validated rows. No row is skipped, so this is the
/// committed corpus' size: the checksum and row count only tie the corpus to
/// its manifest, and a corpus regenerated with a mode dropped, manifest and
/// all, must still fail.
const MIN_ROWS_VALIDATED: usize = 1040;

#[derive(Clone, Copy, Debug, PartialEq)]
enum RowClass {
    Uniform,
    Conjunction,
}

impl RowClass {
    fn name(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Conjunction => "conjunction",
        }
    }

    fn ceiling_arcsec(self) -> f64 {
        match self {
            Self::Uniform => UNIFORM_CEILING_ARCSEC,
            Self::Conjunction => CONJUNCTION_CEILING_ARCSEC,
        }
    }
}

#[derive(Clone, Debug)]
struct Row {
    mode: Ayanamsa,
    mode_name: String,
    jd_tt: f64,
    class: RowClass,
    se_arcsec: f64,
}

#[derive(Debug)]
pub enum AyanamsaApparentError {
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
    CeilingExceeded {
        mode: String,
        jd_tt: f64,
        class: &'static str,
        residual: f64,
        ceiling: f64,
    },
    /// Swiss Ephemeris and pleiades disagree on whether the mode takes a
    /// correction at all: a zero corpus row with a correction, or a nonzero
    /// one without.
    CorrectionPresence {
        mode: String,
        jd_tt: f64,
        swiss_ephemeris_corrects: bool,
    },
}

impl std::fmt::Display for AyanamsaApparentError {
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
            Self::CeilingExceeded {
                mode,
                jd_tt,
                class,
                residual,
                ceiling,
            } => write!(
                f,
                "{mode} {class} apparent-star correction ceiling exceeded at jd_tt={jd_tt}: residual {residual:.6}\" > ceiling {ceiling}\""
            ),
            Self::CorrectionPresence {
                mode,
                jd_tt,
                swiss_ephemeris_corrects,
            } => write!(
                f,
                "{mode} at jd_tt={jd_tt}: Swiss Ephemeris {} the apparent-star correction, pleiades {}",
                if *swiss_ephemeris_corrects { "applies" } else { "does not apply" },
                if *swiss_ephemeris_corrects { "does not" } else { "does" }
            ),
        }
    }
}

impl std::error::Error for AyanamsaApparentError {}

#[derive(Debug)]
pub struct AyanamsaApparentReport {
    pub rows_validated: usize,
    pub uniform_max_arcsec: f64,
    pub conjunction_max_arcsec: f64,
    summary_line: String,
}

impl AyanamsaApparentReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn ayanamsa_from_name(name: &str) -> Option<Ayanamsa> {
    Some(match name {
        "TrueCitra" => Ayanamsa::TrueCitra,
        "TrueRevati" => Ayanamsa::TrueRevati,
        "TruePushya" => Ayanamsa::TruePushya,
        "TrueMula" => Ayanamsa::TrueMula,
        "TrueSheoran" => Ayanamsa::TrueSheoran,
        "GalacticCenter" => Ayanamsa::GalacticCenter,
        "GalacticCenterRgilbrand" => Ayanamsa::GalacticCenterRgilbrand,
        "GalacticCenterMulaWilhelm" => Ayanamsa::GalacticCenterMulaWilhelm,
        "GalacticCenterCochrane" => Ayanamsa::GalacticCenterCochrane,
        "GalacticEquatorIau1958" => Ayanamsa::GalacticEquatorIau1958,
        "GalacticEquatorTrue" => Ayanamsa::GalacticEquatorTrue,
        "GalacticEquatorMula" => Ayanamsa::GalacticEquatorMula,
        "GalacticCenterMardyks" => Ayanamsa::GalacticCenterMardyks,
        "GalacticEquatorFiorenza" => Ayanamsa::GalacticEquatorFiorenza,
        _ => return None,
    })
}

fn parse_rows(csv: &str) -> Result<Vec<Row>, AyanamsaApparentError> {
    let malformed = |what: String| AyanamsaApparentError::MalformedRow(what);
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("mode,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 4 {
            return Err(malformed(format!(
                "expected 4 fields, got {} in {line}",
                fields.len()
            )));
        }
        let num = |i: usize| -> Result<f64, AyanamsaApparentError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let mode = ayanamsa_from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown mode {} in {line}", fields[0])))?;
        let class = match fields[2] {
            "uniform" => RowClass::Uniform,
            "conjunction" => RowClass::Conjunction,
            other => return Err(malformed(format!("unknown class {other} in {line}"))),
        };
        rows.push(Row {
            mode,
            mode_name: fields[0].to_string(),
            jd_tt: num(1)?,
            class,
            se_arcsec: num(3)?,
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), AyanamsaApparentError> {
    let malformed = |what: String| AyanamsaApparentError::MalformedManifest(what);
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

/// |pleiades − Swiss Ephemeris|, arcsec. Swiss Ephemeris writes exactly zero
/// for a mode it does not aberrate; pleiades must then report no correction
/// (`None`), and must report one for every other row.
fn residual_arcsec(row: &Row) -> Result<f64, AyanamsaApparentError> {
    let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
    let ours = pleiades_core::apparent_star_ayanamsa_correction(&row.mode, instant);
    let swiss_ephemeris_corrects = row.se_arcsec != 0.0;
    match ours {
        Some(angle) if swiss_ephemeris_corrects => {
            Ok((angle.degrees() * 3600.0 - row.se_arcsec).abs())
        }
        None if !swiss_ephemeris_corrects => Ok(0.0),
        _ => Err(AyanamsaApparentError::CorrectionPresence {
            mode: row.mode_name.clone(),
            jd_tt: row.jd_tt,
            swiss_ephemeris_corrects,
        }),
    }
}

fn validate(csv: &str, manifest: &str) -> Result<AyanamsaApparentReport, AyanamsaApparentError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(AyanamsaApparentError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_rows(csv)?;
    if rows.len() != manifest_rows {
        return Err(AyanamsaApparentError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let mut uniform_max_arcsec = 0.0f64;
    let mut conjunction_max_arcsec = 0.0f64;
    let mut validated = 0usize;
    for row in &rows {
        let residual = residual_arcsec(row)?;
        let ceiling = row.class.ceiling_arcsec();
        // A NaN residual must fail closed too.
        if residual.is_nan() || residual > ceiling {
            return Err(AyanamsaApparentError::CeilingExceeded {
                mode: row.mode_name.clone(),
                jd_tt: row.jd_tt,
                class: row.class.name(),
                residual,
                ceiling,
            });
        }
        let max = match row.class {
            RowClass::Uniform => &mut uniform_max_arcsec,
            RowClass::Conjunction => &mut conjunction_max_arcsec,
        };
        *max = max.max(residual);
        validated += 1;
    }
    let summary_line = format!(
        "Ayanamsa-apparent gate: {validated} Swiss Ephemeris apparent-star ayanamsa corrections \
         validated (9 star-anchored modes, 5 unaberrated); max residual {uniform_max_arcsec:.3}\" \
         uniform (ceiling {UNIFORM_CEILING_ARCSEC:.2}\"), {conjunction_max_arcsec:.3}\" near solar \
         conjunction (ceiling {CONJUNCTION_CEILING_ARCSEC:.2}\")"
    );
    Ok(AyanamsaApparentReport {
        rows_validated: validated,
        uniform_max_arcsec,
        conjunction_max_arcsec,
        summary_line,
    })
}

/// [`validate`], then fail closed if fewer than `floor` rows were validated.
fn validate_with_floor(
    csv: &str,
    manifest: &str,
    floor: usize,
) -> Result<AyanamsaApparentReport, AyanamsaApparentError> {
    let report = validate(csv, manifest)?;
    if report.rows_validated < floor {
        return Err(AyanamsaApparentError::TooFewRowsValidated {
            validated: report.rows_validated,
            floor,
        });
    }
    Ok(report)
}

pub fn validate_ayanamsa_apparent_corpus() -> Result<AyanamsaApparentReport, AyanamsaApparentError>
{
    validate_with_floor(CORPUS_CSV, MANIFEST, MIN_ROWS_VALIDATED)
}

#[cfg(test)]
mod tests;
