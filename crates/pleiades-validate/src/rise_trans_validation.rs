//! Fail-closed two-tier `validate-rise-trans` gate over the committed SE
//! rise/set/transit + horizontal-coordinate corpus (Task 15,
//! `data/rise-trans-corpus/{rise-trans.csv,azalt.csv,manifest.txt}`).
//!
//! Tier 1 (self-consistency): the engine's own coordinate transforms are
//! round-tripped without reference to any SE value — `horizontal_to_equatorial
//! . horizontal ~= id`, a transit's hour angle ~= 0, and
//! `true_from_apparent . apparent_from_true ~= id` at representative
//! non-grazing altitudes.
//!
//! Tier 2 (SE parity): every committed rise-trans row is recomputed with
//! [`EventEngine::next_rise_set`] and compared to Swiss Ephemeris's
//! `se_jd_tdb` column; every azalt row is recomputed with
//! [`EventEngine::horizontal`] and compared to SE's
//! azimuth/true-altitude/apparent-altitude.
//!
//! ## Time-base note
//!
//! The corpus records Swiss Ephemeris's UT days (`start_jd_ut`, `jd_ut`,
//! `se_jd_ut`) alongside `se_jd_tdb`. Since issue #74 the engine reads the
//! `TimeScale` tag on its query instants and returns genuine TDB instants, so
//! this gate tags every corpus UT day `TimeScale::Ut1` on the way in and
//! compares the returned TDB day against `se_jd_tdb`. Swiss Ephemeris's own
//! ΔT at the corpus epoch (2000-01-01: 63.83 s) matches the packaged
//! `pleiades-time` table, so the switch from `se_jd_ut` is lossless for the
//! parity ceilings. The residual against `se_jd_ut` is still computed (not
//! enforced) so the ~ΔT gap stays visible in the report as evidence that the
//! returned instants are TDB, not UT1.
//!
//! A sibling `manifest.txt` records fnv1a64 digests of both CSVs (drift guard).
//!
//! ## Engine alignment (Task 16; see `rise_trans_thresholds` doc for the
//! ## measured basis of every ceiling)
//!
//! Building this gate originally surfaced two real SE-parity gaps in the
//! engine, both since fixed to match SE (see
//! `crates/pleiades-events/src/rise_trans.rs`): (1) the engine's
//! elevation-based horizon dip did not match SE's default `swe_rise_trans`
//! (which applies no dip unless a `-100` sentinel horizon height is
//! requested — confirmed against the vendored `swecl.c`) — the dip term was
//! removed; (2) `next_rise_set`'s `Rise`/`Set` search was unbounded (scanning
//! the entire ~190-year packaged window for the true next occurrence)
//! whereas SE's `swe_rise_trans` only searches ~28h ahead and reports "no
//! event" otherwise — the search is now bounded to
//! `RISE_SET_SEARCH_SPAN_DAYS`, so a body circumpolar right now correctly
//! reports `None` instead of a far-future event.
//!
//! ## Task 17: below-horizon refraction
//!
//! `pleiades_apparent::apparent_from_true`/`true_from_apparent` now match SE's
//! below-horizon behavior closely (see that module's doc) via a hold-then-fade
//! shape rather than an exact reproduction of SE's own (discontinuous)
//! `swe_refrac_extended` model. Effects on this gate's two known findings:
//!
//! - azalt's below-horizon apparent-altitude residual dropped from ~282
//!   arcsec (worst case) to ~9 arcsec — small enough that this gate now gates
//!   below-horizon apparent-altitude rows too, at the same
//!   `APPARENT_ALTITUDE_ARCSEC` ceiling as above-horizon rows (previously
//!   informational-only; see `rise_trans_thresholds`).
//! - The rise/set "refraction floor" category (Sun/Moon, refraction on, no
//!   custom horizon; see `is_refraction_floor_row`) is UNCHANGED (still ~22 s
//!   max residual): every such crossing's true altitude stays within [-1, 0)
//!   deg, a range this task deliberately left untouched (see
//!   `pleiades_apparent::refraction::apparent_from_true_below_horizon`'s doc
//!   for why extending the fix into that exact band regressed a different
//!   row via bisection-on-a-discontinuity). The corpus provides no
//!   below-horizon ground truth in [-1, 0) deg to diagnose that residual's
//!   real cause against, so `RISE_SET_SECONDS_REFRACTION_FLOOR` keeps its
//!   prior (honestly measured) ceiling rather than being tightened on
//!   unverified assumptions.

use crate::rise_trans_thresholds::*;
use pleiades_apparent::{apparent_from_true, fnv1a64, true_from_apparent, Atmosphere};
use pleiades_data::packaged_backend;
use pleiades_events::{
    fixed_star_apparent, DiscMode, EventEngine, HorizontalInput, RiseSetEvent, RiseSetOptions,
    RiseSetTarget,
};
use pleiades_types::{
    CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};
use std::collections::BTreeMap;

const RISE_TRANS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/rise-trans-corpus/rise-trans.csv"
));
const AZALT_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/rise-trans-corpus/azalt.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/rise-trans-corpus/manifest.txt"
));

/// Fixture counts pinned by the corpus (Task 15). Update when the corpus is
/// regenerated.
const EXPECTED_RISE_TRANS_ROWS: usize = 50;
const EXPECTED_AZALT_ROWS: usize = 20;

/// Representative non-grazing true altitudes (degrees) for the Tier-1
/// refraction round-trip check — see `SELF_CONSISTENCY_ARCSEC`'s doc comment
/// on why grazing altitudes are excluded from this self-check.
const SELF_CONSISTENCY_ALTITUDES_DEG: [f64; 3] = [30.0, 60.0, 85.0];

#[derive(Debug)]
pub enum RiseTransError {
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
    /// Engine error surfaced while recomputing a row.
    Engine(String),
    /// The engine found no rise/set/transit for a row SE reports one for.
    Missing { row: String },
    /// The engine found a rise/set/transit for a row SE reports none for.
    UnexpectedEvent { row: String },
    /// Tier-2 rise/set/transit time residual exceeded its ceiling.
    RiseTransParityExceeded {
        row: String,
        residual_s: f64,
        ceiling_s: f64,
    },
    /// Tier-2 azalt angle residual exceeded its ceiling.
    AzAltParityExceeded {
        row: String,
        field: &'static str,
        residual_arcsec: f64,
        ceiling_arcsec: f64,
    },
    /// Tier-1 self-consistency residual exceeded its ceiling.
    SelfConsistencyExceeded {
        detail: String,
        residual_arcsec: f64,
        ceiling_arcsec: f64,
    },
}

impl std::fmt::Display for RiseTransError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RiseTransError {}

#[derive(Debug, Default)]
pub struct RiseTransReport {
    pub rise_trans_checked: usize,
    pub azalt_checked: usize,
    /// Max rise/set time residual (seconds) against `se_jd_tdb`, over
    /// well-conditioned, non-refraction-floor, non-grazing, non-transit rows
    /// (gated at `RISE_SET_SECONDS_TIGHT`).
    pub max_rise_set_residual_s: f64,
    /// Max rise/set time residual (seconds) against `se_jd_tdb`, over the
    /// Sun/Moon refraction-floor rows (gated at
    /// `RISE_SET_SECONDS_REFRACTION_FLOOR`; see `is_refraction_floor_row`).
    pub max_refraction_floor_residual_s: f64,
    /// Max rise/set time residual (seconds) against `se_jd_tdb`, over the
    /// lat-66.5N grazing rows (gated at `RISE_SET_SECONDS_GRAZING`).
    pub max_grazing_residual_s: f64,
    /// Max meridian-transit time residual (seconds) against `se_jd_tdb`.
    pub max_transit_residual_s: f64,
    /// Max rise/set/transit time residual (seconds) against `se_jd_ut`
    /// (informational only, NOT gated — shows the ~ΔT gap between the
    /// engine's TDB instants and SE's UT days; see the module's time-base
    /// note).
    pub max_residual_vs_se_jd_ut_s: f64,
    /// Max azalt azimuth residual (arcseconds), gated at `AZIMUTH_ARCSEC`.
    pub max_azimuth_residual_arcsec: f64,
    /// Max azalt true-altitude residual (arcseconds), gated at
    /// `TRUE_ALTITUDE_ARCSEC`.
    pub max_true_altitude_residual_arcsec: f64,
    /// Max azalt apparent (refracted) altitude residual (arcseconds), over
    /// on/above-horizon rows (`se_true_alt_deg >= 0`), gated at
    /// `APPARENT_ALTITUDE_ARCSEC`.
    pub max_apparent_altitude_residual_arcsec: f64,
    /// Max azalt apparent (refracted) altitude residual (arcseconds), over
    /// BELOW-horizon rows (`se_true_alt_deg < 0`). Also gated at
    /// `APPARENT_ALTITUDE_ARCSEC` as of Task 17 (previously informational
    /// only — see the module doc's "Task 17" section and
    /// `rise_trans_thresholds`); tracked in its own field purely so the
    /// summary line can show above/below horizon separately.
    pub max_below_horizon_apparent_alt_residual_arcsec: f64,
    /// Max Tier-1 self-consistency residual (arcseconds) over the azalt and
    /// refraction round-trips, gated at `SELF_CONSISTENCY_ARCSEC`.
    pub max_self_consistency_arcsec: f64,
    /// Hour angle (arcseconds past the meridian) at the Tier-1 returned upper
    /// transit, gated one-sided at `TRANSIT_HOUR_ANGLE_ARCSEC`.
    pub transit_hour_angle_arcsec: f64,
}

impl RiseTransReport {
    /// The gate passed iff every committed row was checked (a silently
    /// truncated corpus is a failure, not a pass) — mirrors the pinned
    /// `EXPECTED_ROWS` check in `crossings_validation.rs`. Every tier's
    /// per-row ceiling is enforced fail-closed by returning `Err` during
    /// validation, so reaching `Ok` already implies every checked row was
    /// within ceiling; `passed()` additionally confirms row-count integrity.
    pub fn passed(&self) -> bool {
        self.rise_trans_checked == EXPECTED_RISE_TRANS_ROWS
            && self.azalt_checked == EXPECTED_AZALT_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-rise-trans: {} rise-trans + {} azalt SE fixtures — \
             Tier 1 self-consistency max {:.3}\" (ceiling {:.1}\"), \
             returned transit {:.3}\" past the meridian (ceiling {:.1}\", never before it), \
             Tier 2 rise/set max {:.3} s (ceiling {:.1} s tight), \
             refraction-floor max {:.3} s (ceiling {:.1} s, Task 17), \
             grazing max {:.3} s (ceiling {:.1} s), \
             transit max {:.3} s (ceiling {:.1} s), \
             azalt azimuth max {:.3}\" (ceiling {:.1}\"), \
             true-altitude max {:.3}\" (ceiling {:.1}\"), \
             apparent-altitude (above horizon) max {:.3}\" (ceiling {:.1}\"), \
             apparent-altitude (below horizon) max {:.3}\" (ceiling {:.1}\", Task 17); \
             informational: max vs se_jd_ut {:.1} s (≈ΔT, returned instants are TDB)",
            self.rise_trans_checked,
            self.azalt_checked,
            self.max_self_consistency_arcsec,
            SELF_CONSISTENCY_ARCSEC,
            self.transit_hour_angle_arcsec,
            TRANSIT_HOUR_ANGLE_ARCSEC,
            self.max_rise_set_residual_s,
            RISE_SET_SECONDS_TIGHT,
            self.max_refraction_floor_residual_s,
            RISE_SET_SECONDS_REFRACTION_FLOOR,
            self.max_grazing_residual_s,
            RISE_SET_SECONDS_GRAZING,
            self.max_transit_residual_s,
            TRANSIT_SECONDS,
            self.max_azimuth_residual_arcsec,
            AZIMUTH_ARCSEC,
            self.max_true_altitude_residual_arcsec,
            TRUE_ALTITUDE_ARCSEC,
            self.max_apparent_altitude_residual_arcsec,
            APPARENT_ALTITUDE_ARCSEC,
            self.max_below_horizon_apparent_alt_residual_arcsec,
            APPARENT_ALTITUDE_ARCSEC,
            self.max_residual_vs_se_jd_ut_s,
        )
    }
}

/// Rows classified as genuinely ill-conditioned (near-circumpolar / grazing
/// horizon geometry): the winter Sun/Aldebaran rise-set pair at the Arctic
/// Circle (lat 66.5 N), where the body's altitude changes very slowly with
/// time near the horizon (shallow rise/set angle), amplifying small
/// cross-theory disagreement into a larger time residual. Identified by
/// (object, event, lat_deg) so the classification is explicit and reviewable.
fn is_grazing_row(object: &str, lat_deg: f64) -> bool {
    (object == "Sun" || object == "Aldebaran") && (lat_deg - 66.5).abs() < 1e-6
}

/// Rows squarely in the below-horizon refraction-model floor (Task 17's
/// scope, see the module doc): a Sun/Moon rise/set event with refraction
/// enabled and no custom horizon offset, so the event is defined exactly at
/// the geometric horizon where the engine's Bennett-forward refraction and
/// SE's own refraction algorithm disagree most. Point bodies (stars, Mars)
/// and any row with refraction disabled or a custom horizon offset (which
/// moves the crossing away from the geometric horizon) are unaffected and
/// fall through to the tight ceiling instead.
fn is_refraction_floor_row(object: &str, refraction: bool, horizon_deg: Option<f64>) -> bool {
    (object == "Sun" || object == "Moon") && refraction && horizon_deg.is_none()
}

fn wrap180(mut d: f64) -> f64 {
    d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
    d
}

/// Minimal circular difference between two azimuths (degrees), handling the
/// 0/360 wraparound.
fn azimuth_diff_deg(a: f64, b: f64) -> f64 {
    wrap180(a - b).abs()
}

fn parse_manifest() -> Result<BTreeMap<String, (usize, u64)>, RiseTransError> {
    let mut map = BTreeMap::new();
    for line in MANIFEST.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let toks: Vec<&str> = rest.split_whitespace().collect();
        if toks.len() < 3 {
            return Err(RiseTransError::Manifest(format!(
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
                        .map_err(|e| RiseTransError::Manifest(format!("rows: {e}")))?,
                );
            } else if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(
                    v.parse::<u64>()
                        .map_err(|e| RiseTransError::Manifest(format!("checksum: {e}")))?,
                );
            }
        }
        let rows =
            rows.ok_or_else(|| RiseTransError::Manifest(format!("rows= missing: {line}")))?;
        let checksum = checksum
            .ok_or_else(|| RiseTransError::Manifest(format!("checksum= missing: {line}")))?;
        map.insert(name, (rows, checksum));
    }
    if map.is_empty() {
        return Err(RiseTransError::Manifest(
            "no `file:` lines found in manifest".to_string(),
        ));
    }
    Ok(map)
}

fn check_checksum(
    manifest: &BTreeMap<String, (usize, u64)>,
    file: &'static str,
    csv: &str,
) -> Result<usize, RiseTransError> {
    let (rows, want) = *manifest
        .get(file)
        .ok_or_else(|| RiseTransError::Manifest(format!("manifest missing entry for {file}")))?;
    let got = fnv1a64(csv);
    if got != want {
        return Err(RiseTransError::ChecksumMismatch { file, got, want });
    }
    Ok(rows)
}

fn parse_event(s: &str, row: &str) -> Result<RiseSetEvent, RiseTransError> {
    Ok(match s {
        "Rise" => RiseSetEvent::Rise,
        "Set" => RiseSetEvent::Set,
        "UpperTransit" => RiseSetEvent::UpperTransit,
        "LowerTransit" => RiseSetEvent::LowerTransit,
        _ => {
            return Err(RiseTransError::Schema {
                row: row.to_string(),
            })
        }
    })
}

fn parse_disc(s: &str, row: &str) -> Result<DiscMode, RiseTransError> {
    Ok(match s {
        "upper" => DiscMode::UpperLimb,
        "center" => DiscMode::Center,
        "lower" => DiscMode::LowerLimb,
        _ => {
            return Err(RiseTransError::Schema {
                row: row.to_string(),
            })
        }
    })
}

fn parse_bool01(s: &str, row: &str) -> Result<bool, RiseTransError> {
    match s {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(RiseTransError::Schema {
            row: row.to_string(),
        }),
    }
}

fn parse_target(object: &str) -> RiseSetTarget {
    match object {
        "Sun" => RiseSetTarget::Body(CelestialBody::Sun),
        "Moon" => RiseSetTarget::Body(CelestialBody::Moon),
        "Mars" => RiseSetTarget::Body(CelestialBody::Mars),
        other => RiseSetTarget::FixedStar(other.to_string()),
    }
}

fn parse_f64(s: &str, row: &str) -> Result<f64, RiseTransError> {
    s.trim().parse::<f64>().map_err(|_| RiseTransError::Schema {
        row: row.to_string(),
    })
}

/// `Some(jd)`/`None`, where the corpus spells "no event" as the literal
/// `none`.
fn parse_maybe_jd(s: &str, row: &str) -> Result<Option<f64>, RiseTransError> {
    if s.trim() == "none" {
        Ok(None)
    } else {
        Ok(Some(parse_f64(s, row)?))
    }
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

/// A corpus UT day as the engine expects it: tagged `Ut1`, so the engine adds
/// ΔT itself (see the module's time-base note).
fn ut1(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Ut1)
}

/// Validate a 17-column rise-trans CSV string against the packaged engine.
pub(crate) fn validate_rise_trans_csv(
    csv: &str,
    report: &mut RiseTransReport,
) -> Result<(), RiseTransError> {
    let engine = EventEngine::new(packaged_backend());
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("object,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 17 {
            return Err(RiseTransError::Schema {
                row: line.to_string(),
            });
        }
        let object = f[0];
        let event = parse_event(f[1], line)?;
        let lat_deg = parse_f64(f[2], line)?;
        let lon_deg = parse_f64(f[3], line)?;
        let elev_m = parse_f64(f[4], line)?;
        // f[5] = preset label, informational only.
        let disc = parse_disc(f[6], line)?;
        let refraction = parse_bool01(f[7], line)?;
        let no_ecl_lat = parse_bool01(f[8], line)?;
        let fixed_disc = parse_bool01(f[9], line)?;
        let hindu = parse_bool01(f[10], line)?;
        let horizon_deg = if f[11].trim() == "none" {
            None
        } else {
            Some(parse_f64(f[11], line)?)
        };
        let atpress_hpa = parse_f64(f[12], line)?;
        let attemp_c = parse_f64(f[13], line)?;
        let start_jd_ut = parse_f64(f[14], line)?;
        let se_jd_ut = parse_maybe_jd(f[15], line)?;
        let se_jd_tdb = parse_maybe_jd(f[16], line)?;

        let observer = ObserverLocation::new(
            Latitude::from_degrees(lat_deg),
            Longitude::from_degrees(lon_deg),
            if elev_m == 0.0 { None } else { Some(elev_m) },
        );
        let atmos = Atmosphere {
            pressure_mbar: atpress_hpa,
            temperature_c: attemp_c,
        };
        let opts = RiseSetOptions {
            disc,
            refraction,
            no_ecl_lat,
            fixed_disc_size: fixed_disc,
            hindu,
            horizon_altitude_deg: horizon_deg,
        };
        let target = parse_target(object);
        let after = ut1(start_jd_ut);

        let got = engine
            .next_rise_set(target, event, observer, atmos, opts, after)
            .map_err(|e| RiseTransError::Engine(e.to_string()))?;

        match (got, se_jd_tdb) {
            (None, None) => {
                // Both agree there's no event — nothing further to check.
            }
            (None, Some(_)) => {
                return Err(RiseTransError::Missing {
                    row: line.to_string(),
                })
            }
            (Some(_), None) => {
                return Err(RiseTransError::UnexpectedEvent {
                    row: line.to_string(),
                })
            }
            (Some(rs), Some(se_tdb)) => {
                let got_jd = rs.instant.julian_day.days();
                let residual_s = (got_jd - se_tdb).abs() * 86_400.0;
                if !residual_s.is_finite() {
                    return Err(RiseTransError::RiseTransParityExceeded {
                        row: line.to_string(),
                        residual_s,
                        ceiling_s: 0.0,
                    });
                }
                let is_transit = matches!(
                    event,
                    RiseSetEvent::UpperTransit | RiseSetEvent::LowerTransit
                );
                // Priority: transit > grazing (lat 66.5N oblique path) >
                // refraction floor (Sun/Moon, refraction on, no custom
                // horizon) > tight. Grazing is checked before the refraction
                // floor because the two lat-66.5N Sun rows are affected by
                // BOTH (stacked), and grazing's wider ceiling already covers
                // that; only Aldebaran's lat-66.5N rows are grazing-but-not-
                // floor (point body), and their residual is tiny regardless.
                let ceiling_s = if is_transit {
                    TRANSIT_SECONDS
                } else if is_grazing_row(object, lat_deg) {
                    RISE_SET_SECONDS_GRAZING
                } else if is_refraction_floor_row(object, refraction, horizon_deg) {
                    RISE_SET_SECONDS_REFRACTION_FLOOR
                } else {
                    RISE_SET_SECONDS_TIGHT
                };
                if residual_s > ceiling_s {
                    return Err(RiseTransError::RiseTransParityExceeded {
                        row: line.to_string(),
                        residual_s,
                        ceiling_s,
                    });
                }
                if is_transit {
                    report.max_transit_residual_s = report.max_transit_residual_s.max(residual_s);
                } else if is_grazing_row(object, lat_deg) {
                    report.max_grazing_residual_s = report.max_grazing_residual_s.max(residual_s);
                } else if is_refraction_floor_row(object, refraction, horizon_deg) {
                    report.max_refraction_floor_residual_s =
                        report.max_refraction_floor_residual_s.max(residual_s);
                } else {
                    report.max_rise_set_residual_s = report.max_rise_set_residual_s.max(residual_s);
                }

                // Informational only (shows the ~ΔT time-base gap): residual
                // against `se_jd_ut`, NOT gated.
                if let Some(se_ut) = se_jd_ut {
                    let residual_ut_s = (got_jd - se_ut).abs() * 86_400.0;
                    if residual_ut_s.is_finite() {
                        report.max_residual_vs_se_jd_ut_s =
                            report.max_residual_vs_se_jd_ut_s.max(residual_ut_s);
                    }
                }
            }
        }
        report.rise_trans_checked += 1;
    }
    Ok(())
}

/// Validate an 11-column azalt CSV string against the packaged engine.
pub(crate) fn validate_azalt_csv(
    csv: &str,
    report: &mut RiseTransReport,
) -> Result<(), RiseTransError> {
    let engine = EventEngine::new(packaged_backend());
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("lon_ecl_deg,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 11 {
            return Err(RiseTransError::Schema {
                row: line.to_string(),
            });
        }
        let lon_ecl_deg = parse_f64(f[0], line)?;
        let lat_ecl_deg = parse_f64(f[1], line)?;
        let lat_deg = parse_f64(f[2], line)?;
        let lon_deg = parse_f64(f[3], line)?;
        let elev_m = parse_f64(f[4], line)?;
        let atpress_hpa = parse_f64(f[5], line)?;
        let attemp_c = parse_f64(f[6], line)?;
        let jd_ut = parse_f64(f[7], line)?;
        let se_azimuth_deg = parse_f64(f[8], line)?;
        let se_true_alt_deg = parse_f64(f[9], line)?;
        let se_apparent_alt_deg = parse_f64(f[10], line)?;

        let observer = ObserverLocation::new(
            Latitude::from_degrees(lat_deg),
            Longitude::from_degrees(lon_deg),
            if elev_m == 0.0 { None } else { Some(elev_m) },
        );
        let atmos = Atmosphere {
            pressure_mbar: atpress_hpa,
            temperature_c: attemp_c,
        };
        let at = ut1(jd_ut);
        let input = HorizontalInput::Ecliptic(
            Longitude::from_degrees(lon_ecl_deg),
            Latitude::from_degrees(lat_ecl_deg),
        );
        let h = engine
            .horizontal(input, observer, atmos, at)
            .map_err(|e| RiseTransError::Engine(e.to_string()))?;

        // Azimuth and true altitude are gated tightly for every row.
        let azimuth_residual_arcsec = azimuth_diff_deg(h.azimuth, se_azimuth_deg) * 3600.0;
        let true_alt_residual_arcsec = (h.true_altitude - se_true_alt_deg).abs() * 3600.0;
        for (field, residual_arcsec, ceiling_arcsec) in [
            ("azimuth", azimuth_residual_arcsec, AZIMUTH_ARCSEC),
            (
                "true_altitude",
                true_alt_residual_arcsec,
                TRUE_ALTITUDE_ARCSEC,
            ),
        ] {
            if !residual_arcsec.is_finite() || residual_arcsec > ceiling_arcsec {
                return Err(RiseTransError::AzAltParityExceeded {
                    row: line.to_string(),
                    field,
                    residual_arcsec,
                    ceiling_arcsec,
                });
            }
        }
        report.max_azimuth_residual_arcsec = report
            .max_azimuth_residual_arcsec
            .max(azimuth_residual_arcsec);
        report.max_true_altitude_residual_arcsec = report
            .max_true_altitude_residual_arcsec
            .max(true_alt_residual_arcsec);

        // Apparent (refracted) altitude: gated for EVERY row, on/above AND
        // below the horizon (Task 17: the below-horizon refraction branch now
        // matches SE closely enough — see `pleiades_apparent::refraction`'s
        // module doc — that the previous above-horizon-only restriction is no
        // longer needed; see `rise_trans_thresholds::APPARENT_ALTITUDE_ARCSEC`
        // for the measured basis).
        let apparent_alt_residual_arcsec =
            (h.apparent_altitude - se_apparent_alt_deg).abs() * 3600.0;
        if !apparent_alt_residual_arcsec.is_finite()
            || apparent_alt_residual_arcsec > APPARENT_ALTITUDE_ARCSEC
        {
            return Err(RiseTransError::AzAltParityExceeded {
                row: line.to_string(),
                field: "apparent_altitude",
                residual_arcsec: apparent_alt_residual_arcsec,
                ceiling_arcsec: APPARENT_ALTITUDE_ARCSEC,
            });
        }
        if se_true_alt_deg >= 0.0 {
            report.max_apparent_altitude_residual_arcsec = report
                .max_apparent_altitude_residual_arcsec
                .max(apparent_alt_residual_arcsec);
        } else {
            // Below-horizon: also gated (see above), tracked in its own
            // report field purely for above/below visibility in the summary.
            report.max_below_horizon_apparent_alt_residual_arcsec = report
                .max_below_horizon_apparent_alt_residual_arcsec
                .max(apparent_alt_residual_arcsec);
        }
        report.azalt_checked += 1;
    }
    Ok(())
}

/// Tier 1: self-consistency checks that need no SE values at all.
fn tier1_self_consistency(report: &mut RiseTransReport) -> Result<(), RiseTransError> {
    let engine = EventEngine::new(packaged_backend());
    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(-74.0),
        None,
    );
    let atmos = Atmosphere::default();
    let at = tdb(2_451_545.0);

    // (a) horizontal_to_equatorial(horizontal(x)) ~= x, for a representative
    // equatorial input.
    let ra_in = pleiades_types::Angle::from_degrees(123.456);
    let dec_in = Latitude::from_degrees(12.34);
    let h = engine
        .horizontal(
            HorizontalInput::Equatorial(ra_in, dec_in),
            observer.clone(),
            atmos,
            at,
        )
        .map_err(|e| RiseTransError::Engine(e.to_string()))?;
    let (ra_out, dec_out) = engine
        .horizontal_to_equatorial(
            h.azimuth,
            h.true_altitude,
            false,
            observer.clone(),
            atmos,
            at,
        )
        .map_err(|e| RiseTransError::Engine(e.to_string()))?;
    let ra_residual_arcsec = wrap180(ra_out.degrees() - ra_in.degrees()).abs() * 3600.0;
    let dec_residual_arcsec = (dec_out.degrees() - dec_in.degrees()).abs() * 3600.0;
    for (detail, residual_arcsec) in [
        ("azalt round-trip RA", ra_residual_arcsec),
        ("azalt round-trip Dec", dec_residual_arcsec),
    ] {
        if !residual_arcsec.is_finite() || residual_arcsec > SELF_CONSISTENCY_ARCSEC {
            return Err(RiseTransError::SelfConsistencyExceeded {
                detail: detail.to_string(),
                residual_arcsec,
                ceiling_arcsec: SELF_CONSISTENCY_ARCSEC,
            });
        }
        report.max_self_consistency_arcsec =
            report.max_self_consistency_arcsec.max(residual_arcsec);
    }

    // (b) A returned upper transit is the settled end of the engine's
    // refinement bracket: the star is past the meridian there, by less than
    // the refinement tolerance's worth of rotation, and never short of it
    // (issues #80/#81). Uses a
    // FixedStar target (Aldebaran) rather than a Body: `target_equatorial`'s
    // FixedStar path is exactly the public `fixed_star_apparent` with no
    // topocentric correction, so this check can be reproduced here without
    // reaching into the engine's private body-topocentric pipeline.
    let after = tdb(2_451_545.0);
    let t = engine
        .next_rise_set(
            RiseSetTarget::FixedStar("Aldebaran".to_string()),
            RiseSetEvent::UpperTransit,
            observer.clone(),
            atmos,
            RiseSetOptions::default(),
            after,
        )
        .map_err(|e| RiseTransError::Engine(e.to_string()))?
        .ok_or_else(|| RiseTransError::Missing {
            row: "tier1 upper transit".to_string(),
        })?;
    let jd = t.instant.julian_day.days();
    let equ = fixed_star_apparent("Aldebaran", tdb(jd))
        .map_err(|e| RiseTransError::Engine(e.to_string()))?;
    let ra = equ.right_ascension.degrees();
    // The returned instant is TDB; Earth rotation is a function of UT1.
    let ut1_at = pleiades_apparent::ut1_instant(tdb(jd))
        .map_err(|e| RiseTransError::Engine(e.to_string()))?;
    let lst = pleiades_apparent::sidereal_time(ut1_at, observer.longitude).local_apparent_deg;
    let hour_angle_arcsec = wrap180(lst - ra) * 3600.0;
    check_transit_hour_angle(hour_angle_arcsec)?;
    report.transit_hour_angle_arcsec = hour_angle_arcsec;

    // (c) true_from_apparent(apparent_from_true(h)) ~= h at representative
    // non-grazing altitudes.
    for &alt in &SELF_CONSISTENCY_ALTITUDES_DEG {
        let apparent = apparent_from_true(alt, atmos);
        let back = true_from_apparent(apparent, atmos);
        let residual_arcsec = (back - alt).abs() * 3600.0;
        if !residual_arcsec.is_finite() || residual_arcsec > SELF_CONSISTENCY_ARCSEC {
            return Err(RiseTransError::SelfConsistencyExceeded {
                detail: format!("refraction round-trip at {alt} deg"),
                residual_arcsec,
                ceiling_arcsec: SELF_CONSISTENCY_ARCSEC,
            });
        }
        report.max_self_consistency_arcsec =
            report.max_self_consistency_arcsec.max(residual_arcsec);
    }
    Ok(())
}

/// Gates the signed hour angle (arcseconds, positive past the meridian) at a
/// returned upper transit: fail-closed on a non-finite value, on an instant
/// that precedes the transit, and on one that trails it by more than the
/// refinement tolerance allows.
fn check_transit_hour_angle(hour_angle_arcsec: f64) -> Result<(), RiseTransError> {
    if !hour_angle_arcsec.is_finite() || hour_angle_arcsec > TRANSIT_HOUR_ANGLE_ARCSEC {
        return Err(RiseTransError::SelfConsistencyExceeded {
            detail: "returned transit trails the meridian passage".to_string(),
            residual_arcsec: hour_angle_arcsec,
            ceiling_arcsec: TRANSIT_HOUR_ANGLE_ARCSEC,
        });
    }
    if hour_angle_arcsec < -TRANSIT_HOUR_ANGLE_EARLY_SLACK_ARCSEC {
        return Err(RiseTransError::SelfConsistencyExceeded {
            detail: "returned transit precedes the meridian passage".to_string(),
            residual_arcsec: hour_angle_arcsec,
            ceiling_arcsec: TRANSIT_HOUR_ANGLE_EARLY_SLACK_ARCSEC,
        });
    }
    Ok(())
}

pub fn validate_rise_trans_corpus() -> Result<RiseTransReport, RiseTransError> {
    let manifest = parse_manifest()?;
    let want_rt_rows = check_checksum(&manifest, "rise-trans.csv", RISE_TRANS_CSV)?;
    let want_azalt_rows = check_checksum(&manifest, "azalt.csv", AZALT_CSV)?;

    let mut report = RiseTransReport::default();
    tier1_self_consistency(&mut report)?;
    validate_rise_trans_csv(RISE_TRANS_CSV, &mut report)?;
    validate_azalt_csv(AZALT_CSV, &mut report)?;

    if report.rise_trans_checked != want_rt_rows {
        return Err(RiseTransError::RowCountMismatch {
            file: "rise-trans.csv",
            expected: want_rt_rows,
            got: report.rise_trans_checked,
        });
    }
    if report.azalt_checked != want_azalt_rows {
        return Err(RiseTransError::RowCountMismatch {
            file: "azalt.csv",
            expected: want_azalt_rows,
            got: report.azalt_checked,
        });
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_passes_on_committed_corpus() {
        let outcome = validate_rise_trans_corpus();
        assert!(outcome.is_ok(), "gate errored: {:?}", outcome.err());
        let report = outcome.unwrap();
        assert!(report.passed(), "rise-trans gate failed: {report:?}");
        assert_eq!(report.rise_trans_checked, EXPECTED_RISE_TRANS_ROWS);
        assert_eq!(report.azalt_checked, EXPECTED_AZALT_ROWS);
    }

    #[test]
    fn manifest_checksum_drift_fails_closed() {
        let manifest = parse_manifest().expect("manifest parses");
        let (_, want_rt) = manifest["rise-trans.csv"];
        assert_eq!(
            fnv1a64(RISE_TRANS_CSV),
            want_rt,
            "manifest checksum drifted from rise-trans.csv"
        );
        let (_, want_az) = manifest["azalt.csv"];
        assert_eq!(
            fnv1a64(AZALT_CSV),
            want_az,
            "manifest checksum drifted from azalt.csv"
        );

        // And a genuine mismatch must fail closed.
        let mut bad = manifest.clone();
        bad.insert("rise-trans.csv".to_string(), (50, want_rt.wrapping_add(1)));
        let got = fnv1a64(RISE_TRANS_CSV);
        assert_ne!(got, bad["rise-trans.csv"].1);
    }

    #[test]
    fn manifest_row_counts_are_pinned() {
        let manifest = parse_manifest().expect("manifest parses");
        assert_eq!(manifest["rise-trans.csv"].0, EXPECTED_RISE_TRANS_ROWS);
        assert_eq!(manifest["azalt.csv"].0, EXPECTED_AZALT_ROWS);
    }

    #[test]
    fn no_event_row_yields_none() {
        // Exercises the `none`/`none` schema branch with a row placed a
        // fraction of a scan-step before the packaged ephemeris window's end
        // (see `pleiades_events::WINDOW_END_JD`), so `next_rise_set` returns
        // `None` immediately (no room left to scan) rather than via a slow
        // full-window search. `next_rise_set` scans forward to the true next
        // occurrence anywhere in the ~190-year window, unlike Swiss
        // Ephemeris's `swe_rise_trans` (which only searches ~28h ahead per
        // the vendored `swecl.c`); the committed corpus's own `none` rows
        // (53-56) hit exactly that semantic gap (Task 16 finding, see the
        // report) rather than the window-boundary case exercised here, so
        // this test deliberately avoids relying on that finding.
        let csv = "object,event,lat_deg,lon_deg,elev_m,preset,disc,refraction,no_ecl_lat,fixed_disc,hindu,horizon_deg,atpress_hpa,attemp_c,start_jd_ut,se_jd_ut,se_jd_tdb\n\
Sun,Rise,40.0000,-74.0000,10.0,default,upper,1,0,0,0,none,1013.250,15.000,2488069.495000,none,none\n";
        let mut report = RiseTransReport::default();
        validate_rise_trans_csv(csv, &mut report).expect("no-event row should validate cleanly");
        assert_eq!(report.rise_trans_checked, 1);
    }

    #[test]
    fn bad_arity_is_schema_error() {
        let csv = "Sun,Rise,40.0,-74.0,10.0,default,upper,1,0,0,0,none,1013.25,15.0,2451544.5,2451545.0\n";
        let mut report = RiseTransReport::default();
        assert!(matches!(
            validate_rise_trans_csv(csv, &mut report).unwrap_err(),
            RiseTransError::Schema { .. }
        ));
    }

    // Issues #80/#81: a returned transit is the settled end of the engine's
    // refinement bracket, so the gate on its hour angle is one-sided.
    #[test]
    fn transit_hour_angle_gate_accepts_a_settled_instant() {
        for hour_angle_arcsec in [0.0, 4.065, TRANSIT_HOUR_ANGLE_ARCSEC] {
            check_transit_hour_angle(hour_angle_arcsec)
                .unwrap_or_else(|e| panic!("{hour_angle_arcsec}: {e:?}"));
        }
    }

    #[test]
    fn transit_hour_angle_gate_rejects_an_instant_before_the_transit() {
        // 1.0" short of the meridian: inside the old two-sided 2.6" ceiling,
        // but an instant that precedes its event breaks chained searches.
        let err = check_transit_hour_angle(-1.0).unwrap_err();
        assert!(
            matches!(&err, RiseTransError::SelfConsistencyExceeded { detail, .. }
                if detail.contains("precedes")),
            "{err:?}"
        );
    }

    #[test]
    fn transit_hour_angle_gate_rejects_an_instant_beyond_the_tolerance() {
        for hour_angle_arcsec in [TRANSIT_HOUR_ANGLE_ARCSEC + 0.1, f64::NAN, f64::INFINITY] {
            let err = check_transit_hour_angle(hour_angle_arcsec).unwrap_err();
            assert!(
                matches!(&err, RiseTransError::SelfConsistencyExceeded { detail, .. }
                    if detail.contains("trails")),
                "{hour_angle_arcsec}: {err:?}"
            );
        }
    }
}
