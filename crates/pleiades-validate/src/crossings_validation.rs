//! Fail-closed two-tier gate over the committed SE crossing corpus.
//!
//! Tier 1 (self-consistency): every row's crossing is recomputed by the packaged
//! engine and compared to the committed `pleiades_jd_tdb` golden within
//! `SELF_CONSISTENCY_TOL_S` — tight teeth against any drift in engine output.
//! Tier 2 (SE parity): the engine's longitude at the SE crossing time is compared
//! to the target within a per-body arcsecond ceiling — honest, unamplified
//! agreement with Swiss Ephemeris across the Moshier-vs-VSOP87/ELP theory floor.
//! A sibling `manifest.txt` records an fnv1a64 digest of the CSV (drift guard).

use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingEngine, CrossingFrame};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/crossings-corpus/crossings.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/crossings-corpus/manifest.txt"
));

/// Fixture count pinned by the corpus test. Update when the corpus is regenerated.
/// Test-only: the runtime row check compares against the manifest's `rows:` field.
#[cfg(test)]
const EXPECTED_ROWS: usize = 82;

/// Tier-1 self-consistency ceiling: the engine is deterministic, so a recompute
/// matches the committed golden to the bit unless engine output changed. Set a
/// small factor above the root-finder's 0.5 s bisection tolerance.
const SELF_CONSISTENCY_TOL_S: f64 = 1.0;

// Tier-2 per-body arcsecond ceilings — MEASURED from the committed corpus and set
// to ~1.4x each group's measured maximum. These are cross-theory floors (SE
// Moshier vs engine VSOP87/ELP), not engine error; cf. validate-lilith accepting
// an SE-vs-ours residual of ~306". Measured group maxima at the committed epochs:
// geo Sun 0.3", geo Moon 21.7", geo planets ≤20.7", helio ≤35.1" (Mercury).
const GEO_SUN_ARCSEC: f64 = 2.0;
const GEO_MOON_ARCSEC: f64 = 32.0;
// Every geocentric planet Mercury–Pluto shares this ceiling. Pluto's backend
// fallback (VSOP87 excludes Pluto) nonetheless agrees to ≈20.7" at the sampled
// reachable-target epochs — in line with the other planets — so it needs no wider
// declared boundary; it rides the shared planet ceiling honestly.
const GEO_PLANET_ARCSEC: f64 = 30.0;
// Every heliocentric planet Mercury–Pluto shares this ceiling (measured max 35.1"
// at Mercury; Pluto sits far inside at 3.5").
const HELIO_ARCSEC: f64 = 50.0;

#[derive(Debug)]
pub enum CrossingsCorpusError {
    /// Tier-1: a recomputed crossing drifted from the committed golden.
    SelfConsistencyExceeded {
        row: String,
        residual_s: f64,
        ceiling_s: f64,
    },
    /// Tier-2: engine longitude at the SE time exceeded the arcsecond ceiling.
    ParityExceeded {
        row: String,
        residual_arcsec: f64,
        ceiling_arcsec: f64,
    },
    /// The engine found no crossing for a fixture SE reports one for.
    Missing { row: String },
    /// Malformed corpus row.
    Schema { row: String },
    /// Malformed or missing manifest fields.
    Manifest(String),
    /// The committed CSV digest disagrees with the manifest.
    ChecksumMismatch { got: u64, want: u64 },
    /// Engine error.
    Engine(String),
}

impl std::fmt::Display for CrossingsCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CrossingsCorpusError {}

#[derive(Debug)]
pub struct CrossingsCorpusReport {
    pub checked: usize,
    pub max_self_consistency_s: f64,
    pub max_parity_arcsec: f64,
}

impl CrossingsCorpusReport {
    pub fn summary_line(&self) -> String {
        format!(
            "validate-crossings: {} SE crossing fixtures — Tier 1 self-consistency \
             max {:.3} s (ceiling {:.1} s), Tier 2 SE-parity max {:.1}\" (per-body arcsec ceilings)",
            self.checked, self.max_self_consistency_s, SELF_CONSISTENCY_TOL_S, self.max_parity_arcsec
        )
    }
}

fn wrap180_deg(d: f64) -> f64 {
    ((d + 180.0).rem_euclid(360.0)) - 180.0
}

fn parse_manifest() -> Result<(usize, u64), CrossingsCorpusError> {
    let mut rows = None;
    let mut checksum = None;
    for line in MANIFEST.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("rows:") {
            rows = Some(
                v.trim()
                    .parse::<usize>()
                    .map_err(|e| CrossingsCorpusError::Manifest(format!("rows: {e}")))?,
            );
        }
        for tok in line.split_whitespace() {
            if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(
                    v.parse::<u64>()
                        .map_err(|e| CrossingsCorpusError::Manifest(format!("checksum: {e}")))?,
                );
            }
        }
    }
    Ok((
        rows.ok_or_else(|| CrossingsCorpusError::Manifest("rows: missing".into()))?,
        checksum.ok_or_else(|| CrossingsCorpusError::Manifest("checksum= missing".into()))?,
    ))
}

fn arcsec_ceiling_for(frame: CrossingFrame, body: &CelestialBody) -> f64 {
    match frame {
        CrossingFrame::Heliocentric => HELIO_ARCSEC,
        CrossingFrame::GeocentricApparentOfDate => match body {
            CelestialBody::Sun => GEO_SUN_ARCSEC,
            CelestialBody::Moon => GEO_MOON_ARCSEC,
            _ => GEO_PLANET_ARCSEC,
        },
        // `CrossingFrame` is `#[non_exhaustive]`; a future frame falls back to
        // the loose planet ceiling (this corpus only exercises geo/helio).
        _ => GEO_PLANET_ARCSEC,
    }
}

/// Validate a 7-column crossings CSV string. `validate_crossings_corpus` calls
/// this with the committed `CORPUS_CSV`; tests call it with crafted rows.
pub(crate) fn validate_crossings_csv(
    csv: &str,
) -> Result<CrossingsCorpusReport, CrossingsCorpusError> {
    let engine = CrossingEngine::new(packaged_backend());
    let mut checked = 0usize;
    let mut max_self = 0.0_f64;
    let mut max_parity = 0.0_f64;
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("frame,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 7 {
            return Err(CrossingsCorpusError::Schema {
                row: line.to_string(),
            });
        }
        let frame = match f[0] {
            "geo" => CrossingFrame::GeocentricApparentOfDate,
            "helio" => CrossingFrame::Heliocentric,
            _ => {
                return Err(CrossingsCorpusError::Schema {
                    row: line.to_string(),
                })
            }
        };
        let body = parse_body(f[1]).ok_or_else(|| CrossingsCorpusError::Schema {
            row: line.to_string(),
        })?;
        // The engine is forward-only; reject any non-`fwd` direction as a schema
        // error so a future non-forward fixture fails closed instead of being
        // silently treated as forward.
        if f[4].trim() != "fwd" {
            return Err(CrossingsCorpusError::Schema {
                row: line.to_string(),
            });
        }
        let target = f[2]
            .parse::<f64>()
            .map_err(|_| CrossingsCorpusError::Schema {
                row: line.to_string(),
            })?;
        let start_jd = f[3]
            .parse::<f64>()
            .map_err(|_| CrossingsCorpusError::Schema {
                row: line.to_string(),
            })?;
        let se_jd = f[5]
            .parse::<f64>()
            .map_err(|_| CrossingsCorpusError::Schema {
                row: line.to_string(),
            })?;
        let golden_jd = f[6]
            .parse::<f64>()
            .map_err(|_| CrossingsCorpusError::Schema {
                row: line.to_string(),
            })?;
        let after = Instant::new(JulianDay::from_days(start_jd), TimeScale::Tdb);

        // Tier 1: recompute vs committed golden.
        let got = engine
            .next_longitude_crossing(body.clone(), Longitude::from_degrees(target), frame, after)
            .map_err(|e| CrossingsCorpusError::Engine(e.to_string()))?
            .ok_or_else(|| CrossingsCorpusError::Missing {
                row: line.to_string(),
            })?;
        let residual_s = (got.instant.julian_day.days() - golden_jd).abs() * 86_400.0;
        if residual_s > SELF_CONSISTENCY_TOL_S {
            return Err(CrossingsCorpusError::SelfConsistencyExceeded {
                row: line.to_string(),
                residual_s,
                ceiling_s: SELF_CONSISTENCY_TOL_S,
            });
        }
        max_self = max_self.max(residual_s);

        // Tier 2: engine longitude at the SE time vs target, in arcseconds.
        let se_instant = Instant::new(JulianDay::from_days(se_jd), TimeScale::Tdb);
        let lambda = engine
            .longitude_at(body.clone(), frame, se_instant)
            .map_err(|e| CrossingsCorpusError::Engine(e.to_string()))?;
        let residual_arcsec = wrap180_deg(lambda.degrees() - target).abs() * 3600.0;
        let ceiling_arcsec = arcsec_ceiling_for(frame, &body);
        if residual_arcsec > ceiling_arcsec {
            return Err(CrossingsCorpusError::ParityExceeded {
                row: line.to_string(),
                residual_arcsec,
                ceiling_arcsec,
            });
        }
        max_parity = max_parity.max(residual_arcsec);
        checked += 1;
    }
    Ok(CrossingsCorpusReport {
        checked,
        max_self_consistency_s: max_self,
        max_parity_arcsec: max_parity,
    })
}

pub fn validate_crossings_corpus() -> Result<CrossingsCorpusReport, CrossingsCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest()?;
    let got = fnv1a64(CORPUS_CSV);
    if got != manifest_checksum {
        return Err(CrossingsCorpusError::ChecksumMismatch {
            got,
            want: manifest_checksum,
        });
    }
    let report = validate_crossings_csv(CORPUS_CSV)?;
    if report.checked != manifest_rows {
        return Err(CrossingsCorpusError::Manifest(format!(
            "manifest rows={manifest_rows} but corpus has {} data rows",
            report.checked
        )));
    }
    Ok(report)
}

fn parse_body(name: &str) -> Option<CelestialBody> {
    Some(match name {
        "Sun" => CelestialBody::Sun,
        "Moon" => CelestialBody::Moon,
        "Mercury" => CelestialBody::Mercury,
        "Venus" => CelestialBody::Venus,
        "Mars" => CelestialBody::Mars,
        "Jupiter" => CelestialBody::Jupiter,
        "Saturn" => CelestialBody::Saturn,
        "Uranus" => CelestialBody::Uranus,
        "Neptune" => CelestialBody::Neptune,
        "Pluto" => CelestialBody::Pluto,
        _ => return None,
    })
}

#[derive(Debug)]
pub struct CrossingsGateOutcome(pub Result<CrossingsCorpusReport, CrossingsCorpusError>);
impl CrossingsGateOutcome {
    pub fn passed(&self) -> bool {
        self.0.is_ok()
    }
}
pub fn run_crossings_gate() -> CrossingsGateOutcome {
    CrossingsGateOutcome(validate_crossings_corpus())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_crossings_passes_over_committed_corpus() {
        let report = validate_crossings_corpus().expect("gate should pass");
        // Pin the fixture count so a corpus that silently loses rows fails.
        assert_eq!(report.checked, EXPECTED_ROWS, "unexpected fixture count");
    }

    #[test]
    fn manifest_checksum_matches_corpus() {
        // Closes spec §7: the manifest's fnv1a64 must equal the live CSV digest.
        let (_rows, want) = parse_manifest().expect("manifest parses");
        assert_eq!(
            fnv1a64(CORPUS_CSV),
            want,
            "manifest checksum drifted from crossings.csv"
        );
    }

    #[test]
    fn tier1_catches_golden_drift() {
        // A row whose pleiades_jd_tdb golden is perturbed beyond the sub-second
        // self-consistency ceiling must fail closed.
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,pleiades_jd_tdb
geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,2416199.301931810
";
        let err = validate_crossings_csv(csv).unwrap_err();
        assert!(
            matches!(err, CrossingsCorpusError::SelfConsistencyExceeded { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn tier2_catches_longitude_drift() {
        // A target offset far from where the engine actually is at the SE time
        // must fail the arcsecond parity tier.
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,pleiades_jd_tdb
geo,Sun,10.000000,2416000.500000,fwd,2416195.301931810,PLEIADES
";
        // Fill the golden with the engine's real recompute so Tier 1 passes and
        // only Tier 2 can fire.
        let csv = fill_golden_for_test(csv);
        let err = validate_crossings_csv(&csv).unwrap_err();
        assert!(
            matches!(err, CrossingsCorpusError::ParityExceeded { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn non_forward_and_bad_arity_are_schema_errors() {
        let bad = "geo,Sun,0.0,2416000.5,bwd,2416195.3,2416195.3\n";
        assert!(matches!(
            validate_crossings_csv(bad).unwrap_err(),
            CrossingsCorpusError::Schema { .. }
        ));
        let short = "geo,Sun,0.0,2416000.5,fwd,2416195.3\n";
        assert!(matches!(
            validate_crossings_csv(short).unwrap_err(),
            CrossingsCorpusError::Schema { .. }
        ));
    }

    // Test helper: replace a literal `PLEIADES` golden placeholder with the
    // engine's real next-crossing time so a crafted row exercises Tier 2 alone.
    fn fill_golden_for_test(csv: &str) -> String {
        let engine = CrossingEngine::new(packaged_backend());
        let mut out = String::new();
        for line in csv.lines() {
            if let Some(idx) = line.find(",PLEIADES") {
                let f: Vec<&str> = line[..idx].split(',').collect();
                let frame = if f[0] == "geo" {
                    CrossingFrame::GeocentricApparentOfDate
                } else {
                    CrossingFrame::Heliocentric
                };
                let body = parse_body(f[1]).unwrap();
                let target = Longitude::from_degrees(f[2].parse::<f64>().unwrap());
                let after = Instant::new(
                    JulianDay::from_days(f[3].parse::<f64>().unwrap()),
                    TimeScale::Tdb,
                );
                let c = engine
                    .next_longitude_crossing(body, target, frame, after)
                    .unwrap()
                    .unwrap();
                out.push_str(&format!(
                    "{},{:.9}\n",
                    &line[..idx],
                    c.instant.julian_day.days()
                ));
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }
}
