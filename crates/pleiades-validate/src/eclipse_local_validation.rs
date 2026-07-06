//! Fail-closed two-tier `validate-eclipses-local` gate over the committed
//! Swiss-Ephemeris local (per-observer) eclipse corpus (Task 10,
//! `data/eclipses-local-corpus/{sol-local.csv,lun-local.csv,manifest.txt}`).
//!
//! Tier 1 (self-consistency, no SE reference): every row is recomputed with
//! [`EclipseEngine::local_circumstances`] and the engine's own output is
//! checked for internal consistency — contact ordering
//! (`C1 ≤ max ≤ C4`; `P1 ≤ U1 ≤ U2 ≤ max ≤ U3 ≤ U4 ≤ P4` where each phase is
//! present), `0 ≤ magnitude`, `0 ≤ obscuration ≤ 1`, and `obscuration > 0` iff
//! `magnitude > 0`.
//!
//! Tier 2 (SE parity): each recomputed row is compared field-by-field to the
//! Swiss-Ephemeris columns under the per-category ceilings in
//! `crate::eclipse_local_thresholds`. Solar contact instants are topocentric
//! (observer-dependent); lunar contact instants are global (identical for every
//! observer — the corpus used `swe_lun_eclipse_when`, comparable to the
//! engine's global lunar contacts).
//!
//! ## Time base and the engine fixes this gate validates
//!
//! The Task-10 corpus is emitted in the engine's TT posture (columns
//! `se_*_jd_tt`, `TT = jd_ut + swe_deltat(jd_ut)`, matching the global corpus's
//! `greatest_eclipse_jd_tt`), and this gate compares the engine's native-TDB
//! instants DIRECTLY against those columns (no ΔT crossing). Two engine defects
//! in the local path (`pleiades-eclipse/src/local.rs::topo_sun_moon`) were fixed
//! for this gate to pass (see `eclipse_local_thresholds` for the full write-up):
//!
//! 1. **J2000→apparent-of-date frame.** `sample_sun_moon` returns Mean/J2000
//!    geocentric positions; `topo_sun_moon` now precesses (J2000→date) and adds
//!    nutation-in-longitude to both bodies before the of-date parallax/horizontal
//!    step. Frame-common to Sun and Moon, so the separation (hence eclipse
//!    timing/type/magnitude) is unchanged; it fixes the absolute az/alt.
//! 2. **ΔT-corrected (UT1) parallax rotation.** The topocentric parallax now
//!    rotates the observer offset with UT1 (`ut1_jd_from_tt`), so the
//!    observer-local greatest-eclipse / contact instants match SE's UT1-based
//!    `swe_sol_eclipse_when_loc` (previously biased +20..45 s).
//!
//! ## Atmosphere
//!
//! The corpus generator (`tools/se-eclipse-local-reference`) calls `swe_azalt`
//! with the fixed standard atmosphere `atpress = 1013.25` hPa, `attemp = 15 °C`
//! for every row (it does NOT auto-derive pressure from elevation). That is
//! exactly [`Atmosphere::default`], so this gate passes `Atmosphere::default()`
//! to the engine — the apparent-altitude residual is then a like-for-like
//! comparison of the two refraction models, not an atmosphere mismatch.
//!
//! A sibling `manifest.txt` records fnv1a64 digests of both CSVs (drift guard);
//! a mismatch fails the gate closed.

use crate::eclipse_local_thresholds::*;
use pleiades_apparent::{fnv1a64, Atmosphere};
use pleiades_data::packaged_backend;
use pleiades_eclipse::{
    Eclipse, EclipseEngine, EclipseFilter, EclipseKind, LocalCircumstances, LocalContact,
    LocalLunarCircumstances, LocalSolarCircumstances, LunarEclipseType, SolarEclipseType,
    WINDOW_START_JD,
};
use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
use std::collections::BTreeMap;

const SOLAR_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/eclipses-local-corpus/sol-local.csv"
));
const LUNAR_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/eclipses-local-corpus/lun-local.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/eclipses-local-corpus/manifest.txt"
));

/// Fixture counts pinned by the corpus (Task 10). Update when the corpus is
/// regenerated.
const EXPECTED_SOLAR_ROWS: usize = 29;
const EXPECTED_LUNAR_ROWS: usize = 20;

/// A solar row whose SE magnitude is within this of 1.0 (central limit) is
/// classified "grazing": its contact instants (especially C2/C3, which pinch
/// together as the disks reach internal tangency) are ill-conditioned and gated
/// under the wider `SOLAR_SECONDS_GRAZING` ceiling.
const GRAZING_MAGNITUDE_BAND: f64 = 0.02;

#[derive(Debug)]
pub enum EclipseLocalError {
    /// Malformed or missing manifest fields.
    Manifest(String),
    /// A committed CSV's digest disagrees with the manifest.
    ChecksumMismatch {
        file: &'static str,
        got: u64,
        want: u64,
    },
    /// The manifest's row count for a file disagrees with the parsed corpus.
    RowCountMismatch {
        file: &'static str,
        expected: usize,
        got: usize,
    },
    /// Malformed corpus row.
    Schema { row: String },
    /// The engine could not find the eclipse anchoring a row.
    MissingEclipse { label: String },
    /// A Tier-1 self-consistency invariant failed on the recomputed row.
    SelfConsistency { label: String, detail: String },
    /// SE reports a contact instant the engine does not produce (structural
    /// disagreement, not a ceiling).
    ContactPresenceMismatch {
        label: String,
        contact: &'static str,
    },
    /// A Tier-2 residual exceeded its per-category ceiling.
    ToleranceExceeded {
        category: &'static str,
        label: String,
        residual: f64,
        ceiling: f64,
    },
    /// A Tier-2 exact comparison (type or visibility) disagreed.
    ExactMismatch {
        category: &'static str,
        label: String,
        engine: String,
        se: String,
    },
    /// Engine error surfaced while recomputing a row.
    Engine(String),
}

impl std::fmt::Display for EclipseLocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EclipseLocalError {}

/// Summary of the measured maxima and checked-row counts for the gate.
#[derive(Debug, Default)]
pub struct EclipseLocalReport {
    pub solar_rows: usize,
    pub lunar_rows: usize,
    /// Max solar contact/max instant residual (seconds), over ALL solar rows
    /// (grazing and non-grazing), against the SE topocentric contacts.
    pub max_solar_seconds: f64,
    /// Max lunar contact/max instant residual (seconds), against the SE global
    /// contacts.
    pub max_lunar_seconds: f64,
    /// Max solar magnitude (diameter fraction) absolute residual.
    pub max_magnitude: f64,
    /// Max solar obscuration (area fraction) absolute residual.
    pub max_obscuration: f64,
    /// Max azimuth-at-maximum residual (arcseconds), over solar and lunar rows.
    pub max_az_arcsec: f64,
    /// Max apparent-altitude-at-maximum residual (arcseconds), over solar and
    /// lunar rows.
    pub max_alt_arcsec: f64,
}

impl EclipseLocalReport {
    /// The gate passed iff every committed row was checked (a silently truncated
    /// corpus is a failure, not a pass). Every per-category ceiling is enforced
    /// fail-closed by [`validate_eclipse_local_corpus`], so reaching a report
    /// already implies every checked row was within ceiling.
    pub fn passed(&self) -> bool {
        self.solar_rows == EXPECTED_SOLAR_ROWS && self.lunar_rows == EXPECTED_LUNAR_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-eclipses-local: {} solar + {} lunar rows — max residuals: \
             solar {:.2}s, lunar {:.2}s, mag {:.4}, obsc {:.4}, az {:.3}\", alt {:.3}\"",
            self.solar_rows,
            self.lunar_rows,
            self.max_solar_seconds,
            self.max_lunar_seconds,
            self.max_magnitude,
            self.max_obscuration,
            self.max_az_arcsec,
            self.max_alt_arcsec,
        )
    }
}

/// All measured residual maxima over the committed corpus (finer-grained than
/// the public [`EclipseLocalReport`]: solar time residuals are split into
/// grazing/non-grazing so each can be gated under its own ceiling, and lunar
/// magnitude is tracked for its own ceiling). Tier-1 self-consistency and
/// structural checks are enforced during measurement (they never depend on the
/// numeric ceilings); ceiling gating is applied afterwards by
/// [`validate_eclipse_local_corpus`].
#[derive(Debug, Default)]
struct Measured {
    solar_rows: usize,
    lunar_rows: usize,
    max_solar_nongrazing_s: f64,
    max_solar_grazing_s: f64,
    max_lunar_s: f64,
    max_magnitude: f64,
    max_obscuration: f64,
    max_lunar_magnitude: f64,
    max_az_arcsec: f64,
    max_alt_arcsec: f64,
}

impl Measured {
    fn into_report(self) -> EclipseLocalReport {
        EclipseLocalReport {
            solar_rows: self.solar_rows,
            lunar_rows: self.lunar_rows,
            max_solar_seconds: self.max_solar_nongrazing_s.max(self.max_solar_grazing_s),
            max_lunar_seconds: self.max_lunar_s,
            max_magnitude: self.max_magnitude,
            max_obscuration: self.max_obscuration,
            max_az_arcsec: self.max_az_arcsec,
            max_alt_arcsec: self.max_alt_arcsec,
        }
    }
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

/// Seconds between two Julian-day instants.
fn seconds(a: f64, b: f64) -> f64 {
    (a - b).abs() * 86_400.0
}

fn wrap180(d: f64) -> f64 {
    ((d + 180.0).rem_euclid(360.0)) - 180.0
}

/// Minimal circular azimuth difference in arcseconds (handles 0/360 wrap).
fn azimuth_arcsec(a: f64, b: f64) -> f64 {
    wrap180(a - b).abs() * 3600.0
}

/// Cross-track (on-sky) azimuth residual: the raw azimuth difference projected
/// onto the sky by `cos(altitude)`. Azimuth lines converge toward the zenith, so
/// a fixed angular error on the sky inflates the raw azimuth by `1/cos(alt)`;
/// eclipse maxima routinely fall near local noon with the eclipsed body high and
/// near the meridian, where that inflation is large (and physically meaningless).
/// Multiplying back by `cos(alt)` recovers the true angular error, matching how
/// SP-2b already treats horizontal parity, and avoids a giant flat raw-azimuth
/// ceiling that would mask real regressions at moderate altitudes.
fn azimuth_sky_arcsec(eng_az_deg: f64, se_az_deg: f64, alt_deg: f64) -> f64 {
    azimuth_arcsec(eng_az_deg, se_az_deg) * alt_deg.to_radians().cos().abs()
}

fn parse_manifest() -> Result<BTreeMap<String, (usize, u64)>, EclipseLocalError> {
    let mut map = BTreeMap::new();
    for line in MANIFEST.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let toks: Vec<&str> = rest.split_whitespace().collect();
        if toks.len() < 3 {
            return Err(EclipseLocalError::Manifest(format!(
                "malformed file line: {line}"
            )));
        }
        let name = toks[0].to_string();
        let mut rows = None;
        let mut checksum = None;
        for tok in &toks[1..] {
            if let Some(v) = tok.strip_prefix("rows=") {
                rows = Some(
                    v.parse::<usize>()
                        .map_err(|e| EclipseLocalError::Manifest(format!("rows: {e}")))?,
                );
            } else if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(
                    v.parse::<u64>()
                        .map_err(|e| EclipseLocalError::Manifest(format!("checksum: {e}")))?,
                );
            }
        }
        let rows =
            rows.ok_or_else(|| EclipseLocalError::Manifest(format!("rows= missing: {line}")))?;
        let checksum = checksum
            .ok_or_else(|| EclipseLocalError::Manifest(format!("checksum= missing: {line}")))?;
        map.insert(name, (rows, checksum));
    }
    if map.is_empty() {
        return Err(EclipseLocalError::Manifest(
            "no `file:` lines found in manifest".to_string(),
        ));
    }
    Ok(map)
}

fn check_checksum(
    manifest: &BTreeMap<String, (usize, u64)>,
    file: &'static str,
    csv: &str,
) -> Result<usize, EclipseLocalError> {
    let (rows, want) = *manifest
        .get(file)
        .ok_or_else(|| EclipseLocalError::Manifest(format!("manifest missing entry for {file}")))?;
    let got = fnv1a64(csv);
    if got != want {
        return Err(EclipseLocalError::ChecksumMismatch { file, got, want });
    }
    Ok(rows)
}

fn parse_f64(s: &str, row: &str) -> Result<f64, EclipseLocalError> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| EclipseLocalError::Schema {
            row: row.to_string(),
        })
}

/// Empty cell → `None` ("phase absent"); otherwise the parsed instant.
fn parse_opt_jd(s: &str, row: &str) -> Result<Option<f64>, EclipseLocalError> {
    if s.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(parse_f64(s, row)?))
    }
}

fn parse_i64(s: &str, row: &str) -> Result<i64, EclipseLocalError> {
    s.trim()
        .parse::<i64>()
        .map_err(|_| EclipseLocalError::Schema {
            row: row.to_string(),
        })
}

fn parse_bool01(s: &str, row: &str) -> Result<bool, EclipseLocalError> {
    match s.trim() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(EclipseLocalError::Schema {
            row: row.to_string(),
        }),
    }
}

fn observer_of(lat: f64, lon: f64, elev: f64) -> ObserverLocation {
    // The corpus generator always passes the literal elevation (including 0.0)
    // into `swe_azalt`/`swe_*_eclipse_when_loc`; mirror that exactly.
    ObserverLocation::new(
        Latitude::from_degrees(lat),
        Longitude::from_degrees(lon),
        Some(elev),
    )
}

/// Returns the eclipse anchoring a row: the (single) same-kind eclipse whose
/// greatest eclipse falls in the tight `[se_max − 1 d, se_max + 1 d]` window.
/// A ±1-day bracket is a safe margin — consecutive same-kind eclipses are ≥ ~1
/// synodic month apart, and a topocentric local maximum stays within a couple
/// of hours of the global greatest eclipse — and, unlike a seed-just-before-it
/// `next_eclipse` call (which scans forward to the 2100 window end for every
/// row), it keeps the syzygy scan bounded to two days.
fn find_eclipse<B>(
    engine: &EclipseEngine<B>,
    se_max_jd_tt: f64,
    filter: EclipseFilter,
    label: &str,
) -> Result<Eclipse, EclipseLocalError>
where
    B: pleiades_backend::EphemerisBackend,
{
    let start = tdb(se_max_jd_tt - 1.0);
    let end = tdb(se_max_jd_tt + 1.0);
    engine
        .eclipses_in_range(start, end, filter)
        .map_err(|e| EclipseLocalError::Engine(e.to_string()))?
        .into_iter()
        .next()
        .ok_or_else(|| EclipseLocalError::MissingEclipse {
            label: label.to_string(),
        })
}

fn se_solar_type_str(code: i64) -> &'static str {
    match code {
        4 => "total",
        8 => "annular",
        16 => "partial",
        32 => "hybrid",
        0 => "none",
        _ => "unknown",
    }
}

fn engine_solar_type_str(s: &LocalSolarCircumstances) -> &'static str {
    // The engine represents "no eclipse for this observer" as a degenerate
    // partial record with magnitude exactly 0.0 (see `local::solar_local`), which
    // corresponds to SE's `local_type == 0` ("none-here").
    if s.magnitude <= 0.0 {
        return "none";
    }
    match s.local_type {
        SolarEclipseType::Total => "total",
        SolarEclipseType::Annular => "annular",
        SolarEclipseType::Hybrid => "hybrid",
        SolarEclipseType::Partial => "partial",
    }
}

fn se_lunar_type_str(code: i64) -> &'static str {
    match code {
        4 => "total",
        16 => "partial",
        64 => "penumbral",
        _ => "unknown",
    }
}

fn engine_lunar_type_str(l: &LocalLunarCircumstances) -> &'static str {
    match l.eclipse_type {
        LunarEclipseType::Total => "total",
        LunarEclipseType::Partial => "partial",
        LunarEclipseType::Penumbral => "penumbral",
    }
}

/// Compares an optional SE contact against the engine's optional contact,
/// returning the residual in seconds when SE provides one. A phase SE marks
/// absent (empty cell) is skipped; a phase SE provides but the engine omits is a
/// structural [`EclipseLocalError::ContactPresenceMismatch`].
fn contact_seconds(
    label: &str,
    contact: &'static str,
    se: Option<f64>,
    engine: Option<&LocalContact>,
) -> Result<Option<f64>, EclipseLocalError> {
    match (se, engine) {
        (Some(se_jd), Some(c)) => Ok(Some(seconds(c.instant.julian_day.days(), se_jd))),
        (Some(_), None) => Err(EclipseLocalError::ContactPresenceMismatch {
            label: label.to_string(),
            contact,
        }),
        (None, _) => Ok(None),
    }
}

fn measure_solar<B>(engine: &EclipseEngine<B>, m: &mut Measured) -> Result<(), EclipseLocalError>
where
    B: pleiades_backend::EphemerisBackend,
{
    for line in SOLAR_CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 16 {
            return Err(EclipseLocalError::Schema {
                row: line.to_string(),
            });
        }
        let label = f[0];
        let lat = parse_f64(f[1], line)?;
        let lon = parse_f64(f[2], line)?;
        let elev = parse_f64(f[3], line)?;
        let se_max = parse_f64(f[4], line)?;
        let se_c1 = parse_opt_jd(f[5], line)?;
        let se_c2 = parse_opt_jd(f[6], line)?;
        let se_c3 = parse_opt_jd(f[7], line)?;
        let se_c4 = parse_opt_jd(f[8], line)?;
        let se_type = parse_i64(f[9], line)?;
        let se_mag = parse_f64(f[10], line)?;
        let se_obsc = parse_f64(f[11], line)?;
        let se_az = parse_f64(f[12], line)?;
        // f[13] = se_true_alt_deg — the engine exposes only the apparent
        // (refracted) contact altitude, so this gate compares against the
        // apparent-altitude column (f[14]).
        let se_app_alt = parse_f64(f[14], line)?;
        let se_visible = parse_bool01(f[15], line)?;

        let observer = observer_of(lat, lon, elev);
        let eclipse = find_eclipse(engine, se_max, EclipseFilter::SolarOnly, label)?;
        let local = engine
            .local_circumstances(&eclipse, &observer, Atmosphere::default())
            .map_err(|e| EclipseLocalError::Engine(e.to_string()))?;
        let LocalCircumstances::Solar(s) = local else {
            return Err(EclipseLocalError::SelfConsistency {
                label: label.to_string(),
                detail: "expected solar circumstances for a solar row".to_string(),
            });
        };

        // ---- Tier 1: self-consistency (no SE reference) ----
        let max_jd = s.maximum.instant.julian_day.days();
        let c1_jd = s.first_contact.instant.julian_day.days();
        let c4_jd = s.fourth_contact.instant.julian_day.days();
        let ordered = |a: f64, b: f64| a <= b + 1e-9;
        if !(ordered(c1_jd, max_jd) && ordered(max_jd, c4_jd)) {
            return Err(EclipseLocalError::SelfConsistency {
                label: label.to_string(),
                detail: format!("C1<=max<=C4 violated: {c1_jd} {max_jd} {c4_jd}"),
            });
        }
        if let Some(c2) = s.second_contact {
            let c2_jd = c2.instant.julian_day.days();
            if !(ordered(c1_jd, c2_jd) && ordered(c2_jd, max_jd)) {
                return Err(EclipseLocalError::SelfConsistency {
                    label: label.to_string(),
                    detail: format!("C1<=C2<=max violated: {c1_jd} {c2_jd} {max_jd}"),
                });
            }
        }
        if let Some(c3) = s.third_contact {
            let c3_jd = c3.instant.julian_day.days();
            if !(ordered(max_jd, c3_jd) && ordered(c3_jd, c4_jd)) {
                return Err(EclipseLocalError::SelfConsistency {
                    label: label.to_string(),
                    detail: format!("max<=C3<=C4 violated: {max_jd} {c3_jd} {c4_jd}"),
                });
            }
        }
        if s.magnitude < 0.0 {
            return Err(EclipseLocalError::SelfConsistency {
                label: label.to_string(),
                detail: format!("magnitude < 0: {}", s.magnitude),
            });
        }
        if !(0.0..=1.0).contains(&s.obscuration) {
            return Err(EclipseLocalError::SelfConsistency {
                label: label.to_string(),
                detail: format!("obscuration out of [0,1]: {}", s.obscuration),
            });
        }
        if (s.obscuration > 0.0) != (s.magnitude > 0.0) {
            return Err(EclipseLocalError::SelfConsistency {
                label: label.to_string(),
                detail: format!(
                    "obscuration>0 iff magnitude>0 violated: mag={} obsc={}",
                    s.magnitude, s.obscuration
                ),
            });
        }

        // ---- Tier 2: SE parity residuals + exact comparisons ----
        // Visibility parity is meaningful for every solar row.
        if s.any_phase_visible != se_visible {
            return Err(EclipseLocalError::ExactMismatch {
                category: "solar_visible",
                label: label.to_string(),
                engine: s.any_phase_visible.to_string(),
                se: se_visible.to_string(),
            });
        }
        if !se_visible {
            // Not locally visible: the corpus generator switched to
            // `swe_sol_eclipse_how` at the GLOBAL greatest-eclipse instant
            // (type 0 = "none-here", empty contacts, magnitude/obscuration read
            // there). The engine instead reports this observer's own
            // topocentric LOCAL circumstances (its below-horizon
            // minimum-separation instant, a real but unseen partial). These are
            // different quantities evaluated at different instants, so only the
            // visibility flag (asserted above, both "not visible") is compared;
            // type/magnitude/obscuration/contacts/az/alt are intentionally not.
            m.solar_rows += 1;
            continue;
        }

        let engine_type = engine_solar_type_str(&s);
        let se_type_str = se_solar_type_str(se_type);
        if engine_type != se_type_str {
            return Err(EclipseLocalError::ExactMismatch {
                category: "solar_type",
                label: label.to_string(),
                engine: engine_type.to_string(),
                se: se_type_str.to_string(),
            });
        }

        let grazing = (se_mag - 1.0).abs() < GRAZING_MAGNITUDE_BAND;
        let mut times = vec![seconds(max_jd, se_max)];
        for (tag, se, eng) in [
            ("C1", se_c1, Some(&s.first_contact)),
            ("C2", se_c2, s.second_contact.as_ref()),
            ("C3", se_c3, s.third_contact.as_ref()),
            ("C4", se_c4, Some(&s.fourth_contact)),
        ] {
            if let Some(res) = contact_seconds(label, tag, se, eng)? {
                times.push(res);
            }
        }
        let row_max_s = times.into_iter().fold(0.0_f64, f64::max);
        if grazing {
            m.max_solar_grazing_s = m.max_solar_grazing_s.max(row_max_s);
        } else {
            m.max_solar_nongrazing_s = m.max_solar_nongrazing_s.max(row_max_s);
        }
        m.max_magnitude = m.max_magnitude.max((s.magnitude - se_mag).abs());
        // For total eclipses SE's attr[2] "obscuration" exceeds 1.0 (it is not
        // clamped to the physical solar-area fraction: e.g. 1.118 for 2024 totals).
        // The engine reports the true area fraction, capped at 1.0 (100 % of the
        // Sun's disc covered). Compare against SE clamped to [0,1] so a total is a
        // like-for-like 1.0-vs-1.0 check; partials/annulars (se_obsc < 1) are
        // unaffected and already agree to ~5e-4.
        m.max_obscuration = m
            .max_obscuration
            .max((s.obscuration - se_obsc.clamp(0.0, 1.0)).abs());
        m.max_az_arcsec = m.max_az_arcsec.max(azimuth_sky_arcsec(
            s.maximum.azimuth_degrees,
            se_az,
            s.maximum.altitude_degrees,
        ));
        m.max_alt_arcsec = m
            .max_alt_arcsec
            .max((s.maximum.altitude_degrees - se_app_alt).abs() * 3600.0);
        m.solar_rows += 1;
    }
    Ok(())
}

fn measure_lunar<B>(engine: &EclipseEngine<B>, m: &mut Measured) -> Result<(), EclipseLocalError>
where
    B: pleiades_backend::EphemerisBackend,
{
    for line in LUNAR_CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 18 {
            return Err(EclipseLocalError::Schema {
                row: line.to_string(),
            });
        }
        let label = f[0];
        let lat = parse_f64(f[1], line)?;
        let lon = parse_f64(f[2], line)?;
        let elev = parse_f64(f[3], line)?;
        let se_max = parse_f64(f[4], line)?;
        let se_p1 = parse_opt_jd(f[5], line)?;
        let se_u1 = parse_opt_jd(f[6], line)?;
        let se_u2 = parse_opt_jd(f[7], line)?;
        let se_u3 = parse_opt_jd(f[8], line)?;
        let se_u4 = parse_opt_jd(f[9], line)?;
        let se_p4 = parse_opt_jd(f[10], line)?;
        let se_type = parse_i64(f[11], line)?;
        let se_umbral = parse_f64(f[12], line)?;
        let se_penumbral = parse_f64(f[13], line)?;
        let se_az = parse_f64(f[14], line)?;
        // f[15] = se_true_alt_deg — see solar note; compare apparent (f[16]).
        let se_app_alt = parse_f64(f[16], line)?;
        let se_visible = parse_bool01(f[17], line)?;

        let observer = observer_of(lat, lon, elev);
        let eclipse = find_eclipse(engine, se_max, EclipseFilter::LunarOnly, label)?;
        let local = engine
            .local_circumstances(&eclipse, &observer, Atmosphere::default())
            .map_err(|e| EclipseLocalError::Engine(e.to_string()))?;
        let LocalCircumstances::Lunar(l) = local else {
            return Err(EclipseLocalError::SelfConsistency {
                label: label.to_string(),
                detail: "expected lunar circumstances for a lunar row".to_string(),
            });
        };

        // ---- Tier 1: self-consistency (P1<=U1<=U2<=max<=U3<=U4<=P4) ----
        let max_jd = l.maximum.instant.julian_day.days();
        let p1_jd = l.penumbral_begin.instant.julian_day.days();
        let p4_jd = l.penumbral_end.instant.julian_day.days();
        let jd = |c: &LocalContact| c.instant.julian_day.days();
        let ordered = |a: f64, b: f64| a <= b + 1e-9;
        // Build the present-phase instant chain and assert monotonicity.
        let mut chain = vec![("P1", p1_jd)];
        if let Some(u1) = l.partial_begin {
            chain.push(("U1", jd(&u1)));
        }
        if let Some(u2) = l.total_begin {
            chain.push(("U2", jd(&u2)));
        }
        chain.push(("max", max_jd));
        if let Some(u3) = l.total_end {
            chain.push(("U3", jd(&u3)));
        }
        if let Some(u4) = l.partial_end {
            chain.push(("U4", jd(&u4)));
        }
        chain.push(("P4", p4_jd));
        for w in chain.windows(2) {
            if !ordered(w[0].1, w[1].1) {
                return Err(EclipseLocalError::SelfConsistency {
                    label: label.to_string(),
                    detail: format!("{} <= {} violated: {} {}", w[0].0, w[1].0, w[0].1, w[1].1),
                });
            }
        }
        if l.umbral_magnitude < 0.0 || l.penumbral_magnitude < 0.0 {
            return Err(EclipseLocalError::SelfConsistency {
                label: label.to_string(),
                detail: format!(
                    "negative magnitude: umbral={} penumbral={}",
                    l.umbral_magnitude, l.penumbral_magnitude
                ),
            });
        }

        // ---- Tier 2: SE parity residuals + exact comparisons ----
        let engine_type = engine_lunar_type_str(&l);
        let se_type_str = se_lunar_type_str(se_type);
        if engine_type != se_type_str {
            return Err(EclipseLocalError::ExactMismatch {
                category: "lunar_type",
                label: label.to_string(),
                engine: engine_type.to_string(),
                se: se_type_str.to_string(),
            });
        }
        // SE's lunar visibility is "Moon above the horizon (apparent alt > 0) at
        // MAXIMUM"; the engine's matching field is the maximum contact's own
        // `visible` flag (its `any_phase_visible` uses a different, any-phase
        // definition).
        if l.maximum.visible != se_visible {
            return Err(EclipseLocalError::ExactMismatch {
                category: "lunar_visible",
                label: label.to_string(),
                engine: l.maximum.visible.to_string(),
                se: se_visible.to_string(),
            });
        }

        let mut times = vec![seconds(max_jd, se_max)];
        for (tag, se, eng) in [
            ("P1", se_p1, Some(&l.penumbral_begin)),
            ("U1", se_u1, l.partial_begin.as_ref()),
            ("U2", se_u2, l.total_begin.as_ref()),
            ("U3", se_u3, l.total_end.as_ref()),
            ("U4", se_u4, l.partial_end.as_ref()),
            ("P4", se_p4, Some(&l.penumbral_end)),
        ] {
            if let Some(res) = contact_seconds(label, tag, se, eng)? {
                times.push(res);
            }
        }
        m.max_lunar_s = m.max_lunar_s.max(times.into_iter().fold(0.0_f64, f64::max));
        m.max_lunar_magnitude = m
            .max_lunar_magnitude
            .max((l.umbral_magnitude - se_umbral).abs())
            .max((l.penumbral_magnitude - se_penumbral).abs());
        m.max_az_arcsec = m.max_az_arcsec.max(azimuth_sky_arcsec(
            l.maximum.azimuth_degrees,
            se_az,
            l.maximum.altitude_degrees,
        ));
        m.max_alt_arcsec = m
            .max_alt_arcsec
            .max((l.maximum.altitude_degrees - se_app_alt).abs() * 3600.0);
        m.lunar_rows += 1;
    }
    Ok(())
}

/// Runs the checksum guard, parses both CSVs, recomputes every row via the
/// packaged engine, enforces Tier-1 self-consistency + structural + exact
/// (type/visibility) checks, and accumulates every Tier-2 residual maximum.
/// Numeric ceiling gating is NOT applied here (that is
/// [`validate_eclipse_local_corpus`]'s job) — so this succeeds regardless of the
/// ceiling constants.
fn measure() -> Result<Measured, EclipseLocalError> {
    let manifest = parse_manifest()?;
    let want_solar = check_checksum(&manifest, "sol-local.csv", SOLAR_CSV)?;
    let want_lunar = check_checksum(&manifest, "lun-local.csv", LUNAR_CSV)?;

    let engine = EclipseEngine::new(packaged_backend());
    let mut m = Measured::default();
    measure_solar(&engine, &mut m)?;
    measure_lunar(&engine, &mut m)?;

    if m.solar_rows != want_solar {
        return Err(EclipseLocalError::RowCountMismatch {
            file: "sol-local.csv",
            expected: want_solar,
            got: m.solar_rows,
        });
    }
    if m.lunar_rows != want_lunar {
        return Err(EclipseLocalError::RowCountMismatch {
            file: "lun-local.csv",
            expected: want_lunar,
            got: m.lunar_rows,
        });
    }
    Ok(m)
}

/// Tier-1 (self-consistency) only: checksum guard + per-row recompute + contact
/// ordering / magnitude / obscuration invariants, with NO Tier-2 ceiling gating.
/// Passes on the committed corpus independently of the threshold constants.
pub fn run_tier1_only() -> Result<EclipseLocalReport, EclipseLocalError> {
    Ok(measure()?.into_report())
}

/// Full two-tier gate: Tier-1 self-consistency (via `measure`) plus Tier-2 SE
/// parity gated under every per-category ceiling in
/// `crate::eclipse_local_thresholds`. Fails closed on any exceeded ceiling.
pub fn validate_eclipse_local_corpus() -> Result<EclipseLocalReport, EclipseLocalError> {
    let m = measure()?;

    // Gate each measured maximum against its category ceiling, fail-closed.
    let checks: [(&'static str, f64, f64); 8] = [
        ("solar_seconds", m.max_solar_nongrazing_s, SOLAR_SECONDS),
        (
            "solar_seconds_grazing",
            m.max_solar_grazing_s,
            SOLAR_SECONDS_GRAZING,
        ),
        ("lunar_seconds", m.max_lunar_s, LUNAR_SECONDS),
        ("magnitude", m.max_magnitude, MAGNITUDE_ABS),
        ("obscuration", m.max_obscuration, OBSCURATION_ABS),
        (
            "lunar_magnitude",
            m.max_lunar_magnitude,
            LUNAR_MAGNITUDE_ABS,
        ),
        ("azimuth_arcsec", m.max_az_arcsec, AZIMUTH_ARCSEC),
        ("altitude_arcsec", m.max_alt_arcsec, ALTITUDE_ARCSEC),
    ];
    for (category, residual, ceiling) in checks {
        if !residual.is_finite() || residual > ceiling {
            return Err(EclipseLocalError::ToleranceExceeded {
                category,
                label: "corpus-max".to_string(),
                residual,
                ceiling,
            });
        }
    }
    Ok(m.into_report())
}

/// Prints the next locally-visible eclipse's per-observer circumstances for a
/// given observer, mirroring `eclipse_validation::render_eclipses_listing`'s
/// arg-parsing and one-line-per-result formatting shape.
///
/// Usage: `eclipse-local --lat <deg> --lon <deg> [--elev <m>] [--after <jd>] [--solar|--lunar]`
///
/// JD floats are the only accepted format for `--after` (ISO dates are not
/// supported); defaults to [`WINDOW_START_JD`] when omitted. `--elev` defaults
/// to `0.0` meters. `--lat`/`--lon` are required.
pub fn render_eclipse_local_listing(args: &[&str]) -> Result<String, String> {
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut elev = 0.0_f64;
    let mut after_jd = WINDOW_START_JD;
    let mut filter = EclipseFilter::All;

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--lat" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("eclipse-local: --lat requires a degree value")?;
                lat =
                    Some(val.parse::<f64>().map_err(|_| {
                        format!("eclipse-local: --lat: invalid degree value '{val}'")
                    })?);
            }
            "--lon" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("eclipse-local: --lon requires a degree value")?;
                lon =
                    Some(val.parse::<f64>().map_err(|_| {
                        format!("eclipse-local: --lon: invalid degree value '{val}'")
                    })?);
            }
            "--elev" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("eclipse-local: --elev requires a meter value")?;
                elev = val
                    .parse::<f64>()
                    .map_err(|_| format!("eclipse-local: --elev: invalid meter value '{val}'"))?;
            }
            "--after" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("eclipse-local: --after requires a JD value")?;
                after_jd = val.parse::<f64>().map_err(|_| {
                    format!(
                        "eclipse-local: --after: invalid JD float '{val}' \
                         (only JD floats are accepted, e.g. --after 2451545.0)"
                    )
                })?;
            }
            "--solar" => filter = EclipseFilter::SolarOnly,
            "--lunar" => filter = EclipseFilter::LunarOnly,
            other => {
                return Err(format!(
                    "eclipse-local: unknown argument '{other}' (usage: eclipse-local --lat <deg> \
                     --lon <deg> [--elev <m>] [--after <jd>] [--solar|--lunar])"
                ));
            }
        }
        i += 1;
    }

    let lat = lat.ok_or("eclipse-local: --lat is required")?;
    let lon = lon.ok_or("eclipse-local: --lon is required")?;
    let observer = observer_of(lat, lon, elev);

    let engine = EclipseEngine::new(packaged_backend());
    let after = tdb(after_jd);
    let found = engine
        .next_local_eclipse(after, &observer, filter, Atmosphere::default())
        .map_err(|e| format!("eclipse-local: engine error: {e}"))?;

    let Some((eclipse, local)) = found else {
        return Ok(
            "eclipse-local: no locally-visible eclipse found after the given instant".to_string(),
        );
    };

    let jd = eclipse.greatest_eclipse.julian_day.days();
    let kind = match eclipse.kind {
        EclipseKind::Solar => "solar",
        EclipseKind::Lunar => "lunar",
    };
    // `next_local_eclipse` already filters to `any_phase_visible == true`
    // (see `is_locally_visible`), so that flag would print as a constant
    // `true` here; `max.visible` (visibility AT the same maximum instant
    // whose az/alt is printed) is the informative, non-constant pairing.
    let (typ, mag, max) = match &local {
        LocalCircumstances::Solar(s) => (engine_solar_type_str(s), s.magnitude, s.maximum),
        LocalCircumstances::Lunar(l) => (engine_lunar_type_str(l), l.umbral_magnitude, l.maximum),
    };
    let max_jd = max.instant.julian_day.days();

    Ok(format!(
        "{jd:.5} {kind} {typ} mag={mag:.4} max_jd={max_jd:.5} \
         az={:.3} alt={:.3} visible_at_max={}",
        max.azimuth_degrees, max.altitude_degrees, max.visible,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier1_self_consistency_holds() {
        // Recompute every row; assert contact ordering and magnitude/obscuration
        // bounds. Independent of the Step-1 ceilings, so it passes immediately.
        let report =
            run_tier1_only().expect("tier-1 self-consistency must hold on the committed corpus");
        assert!(report.solar_rows > 0 && report.lunar_rows > 0);
        assert_eq!(report.solar_rows, EXPECTED_SOLAR_ROWS);
        assert_eq!(report.lunar_rows, EXPECTED_LUNAR_ROWS);
    }

    #[test]
    fn manifest_row_counts_are_pinned() {
        let manifest = parse_manifest().expect("manifest parses");
        assert_eq!(manifest["sol-local.csv"].0, EXPECTED_SOLAR_ROWS);
        assert_eq!(manifest["lun-local.csv"].0, EXPECTED_LUNAR_ROWS);
    }

    #[test]
    fn manifest_checksum_drift_fails_closed() {
        let manifest = parse_manifest().expect("manifest parses");
        assert_eq!(fnv1a64(SOLAR_CSV), manifest["sol-local.csv"].1);
        assert_eq!(fnv1a64(LUNAR_CSV), manifest["lun-local.csv"].1);
    }

    #[test]
    fn gate_passes_on_committed_corpus() {
        let outcome = validate_eclipse_local_corpus();
        assert!(outcome.is_ok(), "gate errored: {:?}", outcome.err());
        let report = outcome.unwrap();
        assert!(report.passed(), "eclipse-local gate failed: {report:?}");
    }
}
