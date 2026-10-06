//! Fail-closed two-tier gate over the committed SE crossing corpus.
//!
//! Tier 1 (self-consistency): every row's crossing is recomputed by the packaged
//! engine and compared to the committed `pleiades_jd_tdb` golden within
//! `SELF_CONSISTENCY_TOL_S` — tight teeth against any drift in engine output.
//! Tier 2 (SE parity): the engine's longitude at the SE crossing time is compared
//! to the target within a per-body arcsecond ceiling — honest, unamplified
//! agreement with Swiss Ephemeris across the Moshier-vs-VSOP87/ELP theory floor.
//! A sibling `manifest.txt` records an fnv1a64 digest of the CSV (drift guard).
//! Rows carry a frame (`geo`, `helio`, `geo-mean`) and a zodiac (`tropical` or
//! an ayanamsa name).

use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine};
use pleiades_types::{
    Ayanamsa, CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode,
};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/crossings-corpus/crossings.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/crossings-corpus/manifest.txt"
));

/// Fixture count pinned by the corpus test. Update when the corpus is regenerated.
#[cfg(test)]
pub(crate) const EXPECTED_ROWS: usize = 169;

/// Tier-1 self-consistency ceiling: the engine is deterministic, so a recompute
/// matches the committed golden to the bit unless engine output changed. Set a
/// small factor above the root-finder's 0.5 s bisection tolerance.
const SELF_CONSISTENCY_TOL_S: f64 = 1.0;

// Tier-2 per-body arcsecond ceilings — MEASURED from the committed corpus and set
// to ceil(1.4x each body-class group max). Cross-theory (SE Moshier vs engine)
// floors, not engine error. Measured group maxima (the 86 tropical geo/helio rows,
// 2026-10-06): geo Sun 0.322", geo Moon 2.606", geo planets 0.697" (Pluto; 0.483"
// for Mercury-Neptune), helio 0.597" (Pluto; 0.453" for Mercury-Neptune). Pluto
// is held to the planet ceilings like every other body: this gate runs on the
// packaged backend, whose Pluto is fitted from JPL.
//
// Two earlier sets of maxima were reference or engine defects, not theory floors.
// Before #93 the geo Moon and planet groups measured 21.70" and 20.96": the
// double-counted ~20" aberration term. Before #163 the helio rows were generated
// without SEFLG_TRUEPOS, so Swiss Ephemeris retarded each planet by its
// heliocentric light-time, and the helio group measured 35.090" (Pluto 3.530")
// against ceilings of 50" and 5".
const GEO_SUN_ARCSEC: f64 = 1.0;
const GEO_MOON_ARCSEC: f64 = 4.0;
const GEO_PLANET_ARCSEC: f64 = 1.0;
const HELIO_ARCSEC: f64 = 1.0;

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

fn wrap180_deg(mut d: f64) -> f64 {
    d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
    d
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

// Measured group maxima for the mean-of-date and sidereal rows (169-row corpus,
// 2026-10-01): geo-mean Sun 0.309", geo-mean Moon 2.638", geo-mean planets 0.342";
// sidereal Sun 0.320", sidereal Moon 0.456", sidereal planets 0.420" (largest over
// Lahiri, TrueCitra, GalacticCenter, DeLuce and both geocentric places).
// Ceilings are ceil(1.4x each).
const GEO_MEAN_SUN_ARCSEC: f64 = 1.0;
const GEO_MEAN_MOON_ARCSEC: f64 = 4.0;
const GEO_MEAN_PLANET_ARCSEC: f64 = 1.0;
const SIDEREAL_SUN_ARCSEC: f64 = 1.0;
const SIDEREAL_MOON_ARCSEC: f64 = 1.0;
const SIDEREAL_PLANET_ARCSEC: f64 = 1.0;

fn arcsec_ceiling_for(reference: &CrossingReference, body: &CelestialBody) -> f64 {
    let sidereal = !matches!(reference.zodiac, ZodiacMode::Tropical);
    match (reference.frame, sidereal) {
        (CrossingFrame::Heliocentric, _) => HELIO_ARCSEC,
        (CrossingFrame::GeocentricApparentOfDate, false) => match body {
            CelestialBody::Sun => GEO_SUN_ARCSEC,
            CelestialBody::Moon => GEO_MOON_ARCSEC,
            _ => GEO_PLANET_ARCSEC,
        },
        (CrossingFrame::GeocentricMeanOfDate, false) => match body {
            CelestialBody::Sun => GEO_MEAN_SUN_ARCSEC,
            CelestialBody::Moon => GEO_MEAN_MOON_ARCSEC,
            _ => GEO_MEAN_PLANET_ARCSEC,
        },
        // Sidereal rows of either geocentric place.
        (_, true) => match body {
            CelestialBody::Sun => SIDEREAL_SUN_ARCSEC,
            CelestialBody::Moon => SIDEREAL_MOON_ARCSEC,
            _ => SIDEREAL_PLANET_ARCSEC,
        },
        // `CrossingFrame` is `#[non_exhaustive]`; a future frame falls back to
        // the planet ceiling until it gets rows of its own.
        _ => GEO_PLANET_ARCSEC,
    }
}

/// Reads a corpus row's `frame` and `zodiac` fields. `None` for an unknown
/// name.
pub(crate) fn parse_reference(frame: &str, zodiac: &str) -> Option<CrossingReference> {
    let frame = match frame {
        "geo" => CrossingFrame::GeocentricApparentOfDate,
        "helio" => CrossingFrame::Heliocentric,
        "geo-mean" => CrossingFrame::GeocentricMeanOfDate,
        _ => return None,
    };
    let ayanamsa = match zodiac {
        "tropical" => return Some(CrossingReference::tropical(frame)),
        "Lahiri" => Ayanamsa::Lahiri,
        "TrueCitra" => Ayanamsa::TrueCitra,
        "GalacticCenter" => Ayanamsa::GalacticCenter,
        "DeLuce" => Ayanamsa::DeLuce,
        _ => return None,
    };
    Some(CrossingReference::sidereal(frame, ayanamsa))
}

/// Validate an 8-column crossings CSV string. `validate_crossings_corpus` calls
/// this with the committed `CORPUS_CSV`; tests call it with crafted rows.
pub(crate) fn validate_crossings_csv(
    csv: &str,
) -> Result<CrossingsCorpusReport, CrossingsCorpusError> {
    let engine = EventEngine::new(packaged_backend());
    let mut checked = 0usize;
    let mut max_self = 0.0_f64;
    let mut max_parity = 0.0_f64;
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("frame,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 8 {
            return Err(CrossingsCorpusError::Schema {
                row: line.to_string(),
            });
        }
        let reference =
            parse_reference(f[0], f[6].trim()).ok_or_else(|| CrossingsCorpusError::Schema {
                row: line.to_string(),
            })?;
        let body = parse_body(f[1]).ok_or_else(|| CrossingsCorpusError::Schema {
            row: line.to_string(),
        })?;
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
        let golden_jd = f[7]
            .parse::<f64>()
            .map_err(|_| CrossingsCorpusError::Schema {
                row: line.to_string(),
            })?;
        let after = Instant::new(JulianDay::from_days(start_jd), TimeScale::Tdb);

        // Tier 1: recompute vs committed golden.
        let got = engine
            .next_longitude_crossing(
                body.clone(),
                Longitude::from_degrees(target),
                &reference,
                after,
            )
            .map_err(|e| CrossingsCorpusError::Engine(e.to_string()))?
            .ok_or_else(|| CrossingsCorpusError::Missing {
                row: line.to_string(),
            })?;
        let residual_s = (got.instant.julian_day.days() - golden_jd).abs() * 86_400.0;
        if !residual_s.is_finite() || residual_s > SELF_CONSISTENCY_TOL_S {
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
            .longitude_at(body.clone(), &reference, se_instant)
            .map_err(|e| CrossingsCorpusError::Engine(e.to_string()))?;
        let residual_arcsec = wrap180_deg(lambda.degrees() - target).abs() * 3600.0;
        let ceiling_arcsec = arcsec_ceiling_for(&reference, &body);
        if !residual_arcsec.is_finite() || residual_arcsec > ceiling_arcsec {
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
        let report = crate::tests::test_support::crossings_gate_report()
            .as_ref()
            .expect("gate should pass");
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
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac,pleiades_jd_tdb
geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,tropical,2416199.301931810
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
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac,pleiades_jd_tdb
geo,Sun,10.000000,2416000.500000,fwd,2416195.301931810,tropical,PLEIADES
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
        let bad = "geo,Sun,0.0,2416000.5,bwd,2416195.3,tropical,2416195.3\n";
        assert!(matches!(
            validate_crossings_csv(bad).unwrap_err(),
            CrossingsCorpusError::Schema { .. }
        ));
        let short = "geo,Sun,0.0,2416000.5,fwd,2416195.3,tropical\n";
        assert!(matches!(
            validate_crossings_csv(short).unwrap_err(),
            CrossingsCorpusError::Schema { .. }
        ));
    }

    #[test]
    fn parse_reference_reads_frames_and_zodiacs() {
        use pleiades_types::{Ayanamsa, ZodiacMode};
        let geo = parse_reference("geo", "tropical").unwrap();
        assert_eq!(
            geo,
            CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate)
        );
        let mean = parse_reference("geo-mean", "Lahiri").unwrap();
        assert_eq!(mean.frame, CrossingFrame::GeocentricMeanOfDate);
        assert_eq!(
            mean.zodiac,
            ZodiacMode::Sidereal {
                ayanamsa: Ayanamsa::Lahiri
            }
        );
        for name in ["TrueCitra", "GalacticCenter", "DeLuce"] {
            assert!(parse_reference("geo", name).is_some(), "{name}");
        }
        assert!(parse_reference("geo", "Nonesuch").is_none());
        assert!(parse_reference("lunar", "tropical").is_none());
        // The heliocentric frame takes a sidereal zodiac since issue #106.
        assert_eq!(
            parse_reference("helio", "Lahiri"),
            Some(CrossingReference::sidereal(
                CrossingFrame::Heliocentric,
                Ayanamsa::Lahiri
            ))
        );
    }

    #[test]
    fn unknown_zodiac_is_a_schema_error() {
        let csv = "geo,Sun,0.0,2416000.5,fwd,2416195.3,Nonesuch,2416195.3\n";
        assert!(matches!(
            validate_crossings_csv(csv).unwrap_err(),
            CrossingsCorpusError::Schema { .. }
        ));
    }

    #[test]
    fn tier2_honours_the_zodiac_column() {
        // The SE time below is the tropical 0° crossing. Labelled Lahiri, the
        // engine's sidereal longitude there is about 24° short of the target.
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac,pleiades_jd_tdb
geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,Lahiri,PLEIADES
";
        let csv = fill_golden_for_test(csv);
        let err = validate_crossings_csv(&csv).unwrap_err();
        assert!(
            matches!(err, CrossingsCorpusError::ParityExceeded { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn tier2_catches_a_light_time_retarded_heliocentric_reference() {
        // The Saturn row as it stood before issue #163, generated without
        // SEFLG_TRUEPOS: 4750 s late, about 6.6" of Saturn's longitude. The 50"
        // ceiling of the time let it through; the measured one does not.
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac,pleiades_jd_tdb
helio,Saturn,0.000000,2426000.500000,fwd,2428751.149103666,tropical,PLEIADES
";
        let csv = fill_golden_for_test(csv);
        match validate_crossings_csv(&csv).unwrap_err() {
            CrossingsCorpusError::ParityExceeded {
                residual_arcsec,
                ceiling_arcsec,
                ..
            } => {
                assert!((6.0..7.5).contains(&residual_arcsec), "{residual_arcsec}");
                assert_eq!(ceiling_arcsec, HELIO_ARCSEC);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn nan_golden_residual_fails_closed() {
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac,pleiades_jd_tdb
geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,tropical,NaN
";
        let err = validate_crossings_csv(csv).unwrap_err();
        assert!(
            matches!(err, CrossingsCorpusError::SelfConsistencyExceeded { .. }),
            "{err:?}"
        );
    }

    // Test helper: replace a literal `PLEIADES` golden placeholder with the
    // engine's real next-crossing time so a crafted row exercises Tier 2 alone.
    fn fill_golden_for_test(csv: &str) -> String {
        let engine = EventEngine::new(packaged_backend());
        let mut out = String::new();
        for line in csv.lines() {
            if let Some(idx) = line.find(",PLEIADES") {
                let f: Vec<&str> = line[..idx].split(',').collect();
                let reference = parse_reference(f[0], f[6]).unwrap();
                let body = parse_body(f[1]).unwrap();
                let target = Longitude::from_degrees(f[2].parse::<f64>().unwrap());
                let after = Instant::new(
                    JulianDay::from_days(f[3].parse::<f64>().unwrap()),
                    TimeScale::Tdb,
                );
                let c = engine
                    .next_longitude_crossing(body, target, reference, after)
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

    /// Diagnostic: Tier-2 SE-parity maximum per (frame, body) group, so the
    /// per-group ceilings can be re-measured. Run with
    /// `cargo test -p pleiades-validate crossings_validation::tests::measure_per_group_parity -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_per_group_parity() {
        let engine = EventEngine::new(packaged_backend());
        let mut max_by_group: std::collections::BTreeMap<String, (f64, String)> =
            std::collections::BTreeMap::new();
        for line in CORPUS_CSV.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("frame,") {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            let reference = parse_reference(f[0], f[6].trim()).expect("reference");
            let body = parse_body(f[1]).expect("known body");
            let target: f64 = f[2].parse().expect("target");
            let se_jd: f64 = f[5].parse().expect("se jd");
            let se_instant = Instant::new(JulianDay::from_days(se_jd), TimeScale::Tdb);
            let lambda = engine
                .longitude_at(body.clone(), &reference, se_instant)
                .expect("longitude_at");
            let residual_arcsec = wrap180_deg(lambda.degrees() - target).abs() * 3600.0;
            let group = format!("{}/{}/{}", f[0], f[6].trim(), f[1]);
            let entry = max_by_group.entry(group).or_insert((0.0, String::new()));
            if residual_arcsec > entry.0 {
                *entry = (residual_arcsec, line.to_string());
            }
        }
        for (group, (max, row)) in &max_by_group {
            eprintln!("max {group} {max:.3}\" on row: {row}");
        }
    }
}
