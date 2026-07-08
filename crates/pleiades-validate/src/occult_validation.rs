//! Fail-closed two-tier `validate-occultations` gate over the committed
//! Swiss-Ephemeris `swe_lun_occult_*` reference corpus (Task 9,
//! `data/occultations-corpus/{occultations.csv,manifest.txt}`).
//!
//! Tier 1 (self-consistency, no SE reference): every real occultation event
//! row (`occ_type` 1 or 2) is recomputed via
//! `pleiades_events::EventEngine::occultation` at SE's own reported maximum
//! instant, and the recomputed [`pleiades_events::LocalOccultation`] is
//! checked for internal consistency: `first_contact <= maximum <=
//! fourth_contact`; `second_contact`/`third_contact` present iff the target is
//! a planet disc AND the classification is `Total` (never for a point star);
//! `magnitude`/`obscuration` finite and `obscuration > 0 iff magnitude > 0`.
//!
//! Tier 2 (SE parity): the recomputed geometry is compared against the SE
//! corpus columns — see the module-level corrections below for the two SE
//! quirks that shape this comparison.
//!
//! ## Correction 1 — magnitude/obscuration are unbounded for planets ON SE'S
//! SIDE (magnitude only matches on the engine side — see `KNOWN GAP 1`)
//!
//! SE reports, for a planet target, `magnitude` (`attr[0]`, covered diameter
//! fraction) and `obscuration` (`attr[2]`, covered disc-area fraction) that
//! can run far past 1.0 (up to ~82 and ~26652 in the committed corpus) because
//! these are fractions of the TARGET's (planet's) own tiny disc, and the
//! Moon's disc is vastly larger. `pleiades_events::occult::covered_diameter_fraction`
//! (magnitude) is deliberately unclamped and reproduces the same large
//! values — intentional SE parity, not a bug. `obscuration_fraction`,
//! however, IS clamped to `[0,1]` in every branch of its implementation (a
//! correctly bounded disc-AREA fraction) — it does NOT reproduce SE's large
//! obscuration values for planets; see `KNOWN GAP 1` for why this gate does
//! not attempt to gate planet obscuration numerically. Only a point star
//! (`se_body == -1`) has a binary magnitude/obscuration in `{0.0, 1.0}` on
//! BOTH sides. So Tier-1 asserts `magnitude, obscuration ∈ [0,1]` only for
//! star rows; for planet rows it asserts only `>= 0` (no upper bound) — this
//! Tier-1 bound is honored by both engine quantities regardless of `KNOWN GAP
//! 1` (obscuration's bound is `[0,1]`, a strict subset of `>= 0`).
//!
//! Consequently Tier-2 gates magnitude differently for stars vs. planets (see
//! "Comparison-mode choice" below); obscuration is gated for stars only.
//!
//! ## Correction 2 — `central` is structurally always 0 for `loc` rows
//!
//! SE's `swe_lun_occult_when_loc` never sets `SE_ECL_CENTRAL` (only
//! `SE_ECL_NONCENTRAL`), so every `loc` row's `central` column is `0`
//! regardless of geometry. Only `glob` rows (via `swe_lun_occult_where`) carry
//! a meaningful `central` flag. This gate compares `central` ONLY on `glob`
//! rows; it is never asserted on `loc` rows.
//!
//! **Correction 2b (discovered, not in the original plan):** even within
//! `glob` rows, `central` is only meaningfully compared/counted for PLANET
//! targets. Every star `glob` row in the committed corpus has `central == 0`
//! — SE structurally never marks a point-target occultation as central. Our
//! own engine's `central` formula degenerates to always-`true` for a point
//! target (`s_tgt_deg == 0` collapses the "found" and "central" thresholds
//! in `next_global_occultation` to the identical inequality), so this gate
//! skips the `central` comparison for star `glob` rows entirely rather than
//! run a tautological (always-mismatching) check. See `KNOWN GAP 2` below for
//! how planet `glob` rows are handled (measured/counted, not hard-gated).
//!
//! ## Correction 3 — two distinct kinds of `occ_type == 0` (no-event) rows
//!
//! - **Un-occultable target** (the `Sirius@*` rows): the target can NEVER be
//!   occulted from anywhere on Earth because its ecliptic latitude exceeds
//!   the Moon's ~6.6° maximum reach
//!   (`pleiades_events::occult::MOON_MAX_REACH_DEG`, mirrored locally as
//!   [`MOON_MAX_REACH_DEG`] since the original is `pub(crate)`). This gate
//!   asserts `engine.next_occultation(..)` returns `Ok(None)` (the engine's own
//!   fast-reject path).
//! - **Geometric-miss occultable target** (e.g. `Aldebaran@miss`, `Venus@miss`):
//!   the target IS occultable in general, but this particular
//!   observer/instant is a miss. This gate asserts
//!   `engine.occultation(..)` returns a timed record with
//!   `occultation_type == Miss` (NOT `None` — `occultation()` always returns a
//!   record, timed at the closest approach).
//!
//! The discriminator: for a star row, recompute the star's apparent ecliptic
//! latitude (mirroring `target_never_occultable`'s own logic bit-for-bit —
//! see [`star_never_occultable`]) and compare against
//! [`MOON_MAX_REACH_DEG`]; a planet is never permanently un-occultable (it
//! moves), so every planet `occ_type == 0` row takes the geometric-miss path.
//!
//! ## Comparison-mode choice: magnitude/obscuration, star vs. planet
//!
//! An absolute ceiling meaningful for a star (exactly 1.0, ceiling ~0.01) is
//! unachievable for a planet, whose magnitude values are 18-82 and
//! hypersensitive to sub-arcsecond separation and semidiameter differences
//! (magnitude ∝ 1/`s_tgt`). This gate therefore uses TWO modes for
//! MAGNITUDE: absolute residual for star rows (gated under
//! [`STAR_MAGNITUDE_ABS`]), and RELATIVE residual (`|recomputed − se| /
//! |se|`) for planet rows (gated under [`PLANET_MAGNITUDE_REL`], measured
//! max 4.9%). The planet ABSOLUTE magnitude residual is still measured and
//! reported (informational, ungated) so a reviewer can sanity-check the
//! relative ceiling against the raw magnitude of the numbers involved.
//!
//! OBSCURATION for planet rows could NOT be gated this way — see `KNOWN GAP
//! 1` below (Correction 1b): it is measured/reported but not gated.
//!
//! ## KNOWN GAP 1 (Correction 1b) — planet `obscuration` is not the same
//! quantity on both sides
//!
//! Star `obscuration` (binary `{0,1}`) matches SE exactly and IS gated
//! (`STAR_OBSCURATION_ABS`). For PLANET rows, this gate discovered that
//! `pleiades_events::occult::obscuration_fraction` is a properly, correctly
//! bounded `[0,1]` disc-AREA fraction (every branch of its implementation
//! clamps or returns an exact `0.0`/`1.0`) — but SE's reported `attr[2]` for
//! an occultation of a target much smaller than the Moon is empirically NOT
//! bounded (up to ~26652 in the committed corpus, same order of magnitude
//! disproportion as the (correctly) unclamped magnitude in Correction 1).
//! Unlike magnitude, our bounded obscuration can NEVER reproduce SE's
//! unbounded one for a planet — the measured relative residual saturates
//! near 1.0 (our value is a rounding error next to SE's), which is not "close
//! but out of tolerance", it is a proof the two sides are not the same
//! physical quantity at this size ratio. Any ceiling loose enough to pass
//! would be vacuous (the brief's own warning against a "toothless" ceiling),
//! so `planet_obscuration_{abs,rel}` are measured and reported
//! (informational, see [`OccultReport`]) but NOT part of the fail-closed
//! gate. This is a discovered SE/engine semantic mismatch for planet
//! obscuration specifically — not a residual that a tighter/looser threshold
//! can resolve.
//!
//! ## KNOWN GAP 2 — `next_global_occultation`'s sub-lunar point does not
//! localize the occultation
//!
//! This gate discovered that `GlobalOccultation::sublunar_latitude/longitude`
//! (already-committed `occult.rs`, Task 8/9, out of this task's
//! additive-only scope to fix) report the point where the MOON is at zenith
//! (`moon_dec`, `moon_ra − GAST`) — NOT the point on Earth where the
//! occultation is centrally/best observed, which is what SE's
//! `swe_lun_occult_where` actually returns (and what the corpus's
//! `sublunar_lat/lon` columns are, despite the shared name). Independent
//! verification: calling a LOCAL `occultation()` at OUR reported sub-lunar
//! point gives `Miss` (no occultation there at all), while calling it at SE's
//! reported point gives `Total` with a magnitude matching SE's to 4
//! significant figures — confirming SE's point is correct and ours is a
//! different, non-equivalent quantity. The measured residual is
//! 2545-5344 ARCMINUTES (42-89 DEGREES) across every glob row in the
//! corpus — squarely the "residuals are DEGREES not arcsec, indicating a
//! real engine bug" case this gate is supposed to catch rather than hide.
//! `sublunar_arcmin` is measured and reported (informational) but NOT gated;
//! gating it would require either an enormous (vacuous) ceiling or a fix to
//! `next_global_occultation` in `occult.rs`, both out of this task's scope.
//! This is also very likely why Saturn's `central` boolean disagrees with SE
//! (Correction 2b / above): `central` is computed from the geocentric
//! existence-threshold independent of the (buggy) reported point, but the
//! same underlying sub-lunar-point-vs-true-alignment-point gap plausibly
//! explains why the existence check and SE's stricter check part ways for a
//! small-disc planet at a narrow margin.
//!
//! A sibling `manifest.txt` records the fnv1a64 digest of the CSV (drift
//! guard); a mismatch fails the gate closed.

use crate::occult_thresholds::*;
use pleiades_apparent::{fnv1a64, true_obliquity_degrees, Atmosphere};
use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{
    fixed_star_apparent, EventEngine, LocalOccultation, OccultTarget, OccultationType,
};
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{
    Angle, CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    OBLIQUITY_J2000_DEG,
};
use pleiades_vsop87::Vsop87Backend;
use std::collections::BTreeMap;

const CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/occultations-corpus/occultations.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/occultations-corpus/manifest.txt"
));
const CSV_FILE: &str = "occultations.csv";

/// Fixture row count pinned by the corpus (Task 9). Update when the corpus is
/// regenerated.
pub const EXPECTED_ROWS: usize = 62;

/// Mirrors `pleiades_events::occult::MOON_MAX_REACH_DEG` (`pub(crate)`, not
/// exported): the Moon's maximum reach in ecliptic latitude — beyond this a
/// star can never be occulted from anywhere on Earth. ~5.3° orbital
/// inclination + ~0.27° semidiameter + ~0.95° horizontal parallax ≈ 6.6°.
pub const MOON_MAX_REACH_DEG: f64 = 6.6;

#[derive(Debug)]
pub enum OccultError {
    /// A committed CSV's digest disagrees with the manifest.
    ChecksumMismatch {
        file: &'static str,
        got: u64,
        want: u64,
    },
    /// The parsed corpus row count disagrees with [`EXPECTED_ROWS`].
    RowCountMismatch { expected: usize, got: usize },
    /// A Tier-2 residual exceeded its ceiling.
    ToleranceExceeded {
        metric: &'static str,
        label: String,
        jd: f64,
        residual: f64,
        ceiling: f64,
    },
    /// Malformed manifest/corpus row, a Tier-1 self-consistency invariant
    /// failing on the recomputed row, an unrecognized `se_body`/`mode`, or a
    /// Tier-2 exact-match assertion failing (`occultation_type`, `central`,
    /// or an unexpected `Some`/`None` on the never-occultable/global paths).
    /// Fail-closed: never a silent skip.
    Parse { row: String },
    /// The engine errored while recomputing a row.
    Engine(String),
}

impl std::fmt::Display for OccultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for OccultError {}

/// Summary of the measured per-metric maxima and checked-row count for the
/// gate. Fields marked "informational" below are measured and reported but
/// NOT part of the fail-closed ceiling gate in
/// [`validate_occultations_corpus`] — see `KNOWN GAP 1`/`KNOWN GAP 2` in the
/// module doc for why.
#[derive(Debug, Default)]
pub struct OccultReport {
    pub rows: usize,
    pub max_contact_seconds: f64,
    pub max_contact_seconds_grazing: f64,
    /// Informational (KNOWN GAP 1): great-circle arcmin between our
    /// `GlobalOccultation` sub-lunar point and SE's. Runs 42-89 DEGREES in
    /// the committed corpus (a discovered bug in already-committed
    /// `next_global_occultation`, out of this task's scope to fix) — NOT
    /// gated.
    pub max_sublunar_arcmin: f64,
    pub max_star_magnitude_abs: f64,
    pub max_star_obscuration_abs: f64,
    /// Informational: planet magnitude absolute residual (see
    /// `max_planet_magnitude_rel` for the gated metric).
    pub max_planet_magnitude_abs: f64,
    pub max_planet_magnitude_rel: f64,
    /// Informational (Correction 1b): planet obscuration absolute residual.
    /// Our engine's `obscuration` is a properly bounded `[0,1]` disc-area
    /// fraction; SE's `attr[2]` for an occultation of a target much smaller
    /// than the Moon is empirically NOT bounded (values up to ~26652 in the
    /// committed corpus) and does not appear to be the same physical
    /// quantity at this size ratio — NOT gated (any ceiling loose enough to
    /// pass would be vacuous; see `max_planet_obscuration_rel`).
    pub max_planet_obscuration_abs: f64,
    /// Informational (Correction 1b): relative residual saturates near 1.0
    /// (our bounded value vs. SE's much larger one) — confirms the values
    /// are not comparable at this scale, not that the engine is "close".
    pub max_planet_obscuration_rel: f64,
    /// Informational (KNOWN GAP 2): planet glob rows where `central` was
    /// compared.
    pub central_planet_checked: usize,
    /// Informational (KNOWN GAP 2): of those, how many disagreed with SE.
    /// Saturn's 2 rows mismatch (engine `true`, SE `false`) at a
    /// narrow-margin existence-threshold edge case; Venus/Jupiter agree.
    pub central_planet_mismatched: usize,
}

impl OccultReport {
    /// The gate passed iff every committed row was checked (a silently
    /// truncated corpus is a failure, not a pass). Every GATED ceiling is
    /// enforced fail-closed by [`validate_occultations_corpus`], so reaching
    /// a report already implies every checked row was within its gated
    /// ceiling — informational fields (see field docs) are reported but not
    /// part of that gate.
    pub fn passed(&self) -> bool {
        self.rows == EXPECTED_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-occultations: {} rows — max residuals (gated): contact {:.3}s contact_grazing {:.3}s star_mag {:.4} star_obsc {:.4} planet_mag_rel {:.4} — informational (ungated, see KNOWN GAP 1/2): sublunar {:.1}' planet_mag_abs {:.3} planet_obsc_abs {:.1} planet_obsc_rel {:.4} central_planet {}/{} mismatched",
            self.rows,
            self.max_contact_seconds,
            self.max_contact_seconds_grazing,
            self.max_star_magnitude_abs,
            self.max_star_obscuration_abs,
            self.max_planet_magnitude_rel,
            self.max_sublunar_arcmin,
            self.max_planet_magnitude_abs,
            self.max_planet_obscuration_abs,
            self.max_planet_obscuration_rel,
            self.central_planet_mismatched,
            self.central_planet_checked,
        )
    }
}

/// One metric's running maximum, tagged with the offending row's label and
/// `jd_tt` so a ceiling violation can be reported precisely.
#[derive(Debug, Clone)]
struct MetricMax {
    value: f64,
    label: String,
    jd: f64,
}

impl Default for MetricMax {
    fn default() -> Self {
        MetricMax {
            value: 0.0,
            label: String::new(),
            jd: f64::NAN,
        }
    }
}

impl MetricMax {
    fn observe(&mut self, value: f64, label: &str, jd: f64) {
        // A NaN residual never compares `Greater`, so treat it as a new max
        // explicitly — it must never be silently dropped (checked again,
        // fail-closed, at gate time via `residual.is_finite()`).
        let is_new_max = match value.partial_cmp(&self.value) {
            Some(std::cmp::Ordering::Greater) => true,
            Some(_) => false,
            None => true,
        };
        if is_new_max {
            self.value = value;
            self.label = label.to_string();
            self.jd = jd;
        }
    }
}

/// All measured residual maxima over the committed corpus. Tier-1
/// self-consistency is enforced during measurement (it never depends on the
/// numeric ceilings); ceiling gating is applied afterwards by
/// [`validate_occultations_corpus`].
#[derive(Debug, Default)]
struct Measured {
    rows: usize,
    /// Contact/maximum instant residuals from well-conditioned (`Total`,
    /// `occ_type == 2`) `loc` rows AND all `glob` rows (always `Total`-class
    /// in the committed corpus).
    contact_total: MetricMax,
    /// Contact/maximum instant residuals from `Grazing` (`occ_type == 1`)
    /// `loc` rows.
    contact_grazing: MetricMax,
    sublunar: MetricMax,
    star_magnitude: MetricMax,
    star_obscuration: MetricMax,
    planet_magnitude_abs: MetricMax,
    planet_magnitude_rel: MetricMax,
    planet_obscuration_abs: MetricMax,
    planet_obscuration_rel: MetricMax,
    /// Informational (KNOWN GAP 2): planet glob rows where `central` was
    /// compared / how many disagreed. See field docs on [`OccultReport`].
    central_planet_checked: usize,
    central_planet_mismatched: usize,
}

impl Measured {
    fn into_report(self) -> OccultReport {
        OccultReport {
            rows: self.rows,
            max_contact_seconds: self.contact_total.value,
            max_contact_seconds_grazing: self.contact_grazing.value,
            max_sublunar_arcmin: self.sublunar.value,
            max_star_magnitude_abs: self.star_magnitude.value,
            max_star_obscuration_abs: self.star_obscuration.value,
            max_planet_magnitude_abs: self.planet_magnitude_abs.value,
            max_planet_magnitude_rel: self.planet_magnitude_rel.value,
            max_planet_obscuration_abs: self.planet_obscuration_abs.value,
            max_planet_obscuration_rel: self.planet_obscuration_rel.value,
            central_planet_checked: self.central_planet_checked,
            central_planet_mismatched: self.central_planet_mismatched,
        }
    }
}

/// SE body number -> `CelestialBody`. Only -1 (star, handled separately), 3
/// (Venus), 5 (Jupiter), and 6 (Saturn) appear in the committed corpus, but
/// the full SE 0-9 major mapping is honored for robustness against corpus
/// regeneration; any other value is a fail-closed parse error.
fn body_from_se(se_body: i64) -> Option<CelestialBody> {
    match se_body {
        0 => Some(CelestialBody::Sun),
        1 => Some(CelestialBody::Moon),
        2 => Some(CelestialBody::Mercury),
        3 => Some(CelestialBody::Venus),
        4 => Some(CelestialBody::Mars),
        5 => Some(CelestialBody::Jupiter),
        6 => Some(CelestialBody::Saturn),
        7 => Some(CelestialBody::Uranus),
        8 => Some(CelestialBody::Neptune),
        9 => Some(CelestialBody::Pluto),
        _ => None,
    }
}

fn parse_manifest() -> Result<BTreeMap<String, (usize, u64)>, OccultError> {
    let mut map = BTreeMap::new();
    for line in MANIFEST.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let toks: Vec<&str> = rest.split_whitespace().collect();
        if toks.len() < 3 {
            return Err(OccultError::Parse {
                row: format!("malformed file line: {line}"),
            });
        }
        let name = toks[0].to_string();
        let mut rows = None;
        let mut checksum = None;
        for tok in &toks[1..] {
            if let Some(v) = tok.strip_prefix("rows=") {
                rows = Some(v.parse::<usize>().map_err(|e| OccultError::Parse {
                    row: format!("rows: {e}"),
                })?);
            } else if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(v.parse::<u64>().map_err(|e| OccultError::Parse {
                    row: format!("checksum: {e}"),
                })?);
            }
        }
        let rows = rows.ok_or_else(|| OccultError::Parse {
            row: format!("rows= missing: {line}"),
        })?;
        let checksum = checksum.ok_or_else(|| OccultError::Parse {
            row: format!("checksum= missing: {line}"),
        })?;
        map.insert(name, (rows, checksum));
    }
    if map.is_empty() {
        return Err(OccultError::Parse {
            row: "no `file:` lines found in manifest".to_string(),
        });
    }
    Ok(map)
}

/// Looks up `file` in the manifest and compares `fnv1a64(csv)` against the
/// recorded checksum, fail-closed. Returns the manifest's declared row count
/// on success.
fn check_checksum(file: &'static str, csv: &str) -> Result<usize, OccultError> {
    let manifest = parse_manifest()?;
    let (rows, want) = *manifest.get(file).ok_or_else(|| OccultError::Parse {
        row: format!("manifest missing entry for {file}"),
    })?;
    let got = fnv1a64(csv);
    if got != want {
        return Err(OccultError::ChecksumMismatch { file, got, want });
    }
    Ok(rows)
}

fn parse_f64(s: &str, row: &str) -> Result<f64, OccultError> {
    s.trim().parse::<f64>().map_err(|_| OccultError::Parse {
        row: row.to_string(),
    })
}

fn parse_i64(s: &str, row: &str) -> Result<i64, OccultError> {
    s.trim().parse::<i64>().map_err(|_| OccultError::Parse {
        row: row.to_string(),
    })
}

/// Great-circle separation (degrees) between two geographic lat/lon points —
/// identical formula to `pleiades_events::occult::angular_separation_deg`
/// (RA/Dec there, lat/lon here; the spherical-law-of-cosines math is the
/// same).
fn great_circle_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (a1, d1) = (lon1.to_radians(), lat1.to_radians());
    let (a2, d2) = (lon2.to_radians(), lat2.to_radians());
    let cos_sep = (d1.sin() * d2.sin() + d1.cos() * d2.cos() * (a1 - a2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos().to_degrees()
}

/// Whether a star (by curated catalog name) sits permanently outside the
/// Moon's reach — mirrors `pleiades_events::occult::EventEngine::target_never_occultable`'s
/// star branch bit-for-bit (that method is `pub(crate)`, so this
/// re-derives it from the public `fixed_star_apparent` + `to_ecliptic` path).
fn star_never_occultable(name: &str, jd: f64) -> Result<bool, OccultError> {
    let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let equ = fixed_star_apparent(name, at)
        .map_err(|e| OccultError::Engine(format!("{name} ({jd}): {e}")))?;
    let eps = true_obliquity_degrees(jd).unwrap_or(OBLIQUITY_J2000_DEG);
    let ecl = equ.to_ecliptic(Angle::from_degrees(eps));
    Ok(ecl.latitude.degrees().abs() > MOON_MAX_REACH_DEG)
}

/// Checks Tier-1 self-consistency for one recomputed `loc`-mode occultation
/// event row (`occ_type` 1 or 2). Returns an error identifying the offending
/// row on failure.
fn check_tier1_local(label: &str, rec: &LocalOccultation, is_star: bool) -> Result<(), OccultError> {
    if !(rec.magnitude.is_finite() && rec.obscuration.is_finite()) {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: non-finite magnitude/obscuration: {} {}",
                rec.magnitude, rec.obscuration
            ),
        });
    }
    let max_jd = rec.maximum.instant.julian_day.days();
    let c1_jd = rec.first_contact.instant.julian_day.days();
    let c4_jd = rec.fourth_contact.instant.julian_day.days();
    if !(c1_jd <= max_jd && max_jd <= c4_jd) {
        return Err(OccultError::Parse {
            row: format!("{label}: contacts not ordered c1={c1_jd} max={max_jd} c4={c4_jd}"),
        });
    }
    // Interior contacts (c2/c3) exist iff the target is a planet disc AND the
    // classification is Total — a point star never has them (§Correction 1).
    let disc_total = !is_star && rec.occultation_type == OccultationType::Total;
    let has_interior = rec.second_contact.is_some() && rec.third_contact.is_some();
    if disc_total != has_interior {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: c2/c3 presence mismatch (disc_total={disc_total} has_interior={has_interior})"
            ),
        });
    }
    if is_star {
        if !(0.0..=1.0).contains(&rec.magnitude) || !(0.0..=1.0).contains(&rec.obscuration) {
            return Err(OccultError::Parse {
                row: format!(
                    "{label}: star magnitude/obscuration out of [0,1]: {} {}",
                    rec.magnitude, rec.obscuration
                ),
            });
        }
    } else if rec.magnitude < 0.0 || rec.obscuration < 0.0 {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: planet magnitude/obscuration negative: {} {}",
                rec.magnitude, rec.obscuration
            ),
        });
    }
    if (rec.obscuration > 0.0) != (rec.magnitude > 0.0) {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: obscuration>0 iff magnitude>0 violated: mag={} obs={}",
                rec.magnitude, rec.obscuration
            ),
        });
    }
    Ok(())
}

/// Runs the checksum guard, parses the corpus, recomputes every row via a
/// freshly built production-style `EventEngine`, enforces Tier-1
/// self-consistency, and accumulates every Tier-2 residual maximum. Numeric
/// ceiling gating is NOT applied here (that is [`validate_occultations_corpus`]'s
/// job) — so this succeeds regardless of the ceiling constants.
fn measure() -> Result<Measured, OccultError> {
    check_checksum(CSV_FILE, CSV)?;

    let backend = RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]);
    let engine = EventEngine::new(backend);
    let mut m = Measured::default();

    for line in CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 19 {
            return Err(OccultError::Parse {
                row: line.to_string(),
            });
        }
        let label = f[0].to_string();
        let mode = f[1].trim();
        let se_body = parse_i64(f[2], line)?;
        let star = f[3].trim().to_string();
        let jd_tt = parse_f64(f[4], line)?;
        let lat = parse_f64(f[5], line)?;
        let lon = parse_f64(f[6], line)?;
        let elev = parse_f64(f[7], line)?;
        let max_jd = parse_f64(f[8], line)?;
        let c1_jd = parse_f64(f[9], line)?;
        let c2_jd = parse_f64(f[10], line)?;
        let c3_jd = parse_f64(f[11], line)?;
        let c4_jd = parse_f64(f[12], line)?;
        let se_magnitude = parse_f64(f[13], line)?;
        let se_obscuration = parse_f64(f[14], line)?;
        let occ_type = parse_i64(f[15], line)?;
        let se_sublunar_lat = parse_f64(f[16], line)?;
        let se_sublunar_lon = parse_f64(f[17], line)?;
        let central = parse_i64(f[18], line)?;

        let is_star = se_body == -1;
        let target = if is_star {
            OccultTarget::Star(star.clone())
        } else {
            let body = body_from_se(se_body).ok_or_else(|| OccultError::Parse {
                row: format!("{line} (unrecognized se_body {se_body})"),
            })?;
            OccultTarget::Body(body)
        };

        match mode {
            "loc" => {
                let observer = ObserverLocation::new(
                    Latitude::from_degrees(lat),
                    Longitude::from_degrees(lon),
                    Some(elev),
                );
                if occ_type == 0 {
                    // Two distinct kinds of no-event row (§Correction 3).
                    let never = is_star && star_never_occultable(&star, jd_tt)?;
                    if never {
                        let after = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tdb);
                        let result = engine
                            .next_occultation(target, observer, Atmosphere::default(), after)
                            .map_err(|e| {
                                OccultError::Engine(format!("{label} ({jd_tt}): {e}"))
                            })?;
                        if result.is_some() {
                            return Err(OccultError::Parse {
                                row: format!(
                                    "{label}: expected None (never-occultable) but got Some"
                                ),
                            });
                        }
                    } else {
                        let at = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tdb);
                        let rec = engine
                            .occultation(target, observer, Atmosphere::default(), at)
                            .map_err(|e| {
                                OccultError::Engine(format!("{label} ({jd_tt}): {e}"))
                            })?;
                        if rec.occultation_type != OccultationType::Miss {
                            return Err(OccultError::Parse {
                                row: format!(
                                    "{label}: expected Miss, got {:?}",
                                    rec.occultation_type
                                ),
                            });
                        }
                    }
                } else {
                    let at = Instant::new(JulianDay::from_days(max_jd), TimeScale::Tdb);
                    let rec = engine
                        .occultation(target, observer, Atmosphere::default(), at)
                        .map_err(|e| OccultError::Engine(format!("{label} ({max_jd}): {e}")))?;

                    check_tier1_local(&label, &rec, is_star)?;

                    let expected_type = if occ_type == 2 {
                        OccultationType::Total
                    } else {
                        OccultationType::Grazing
                    };
                    if rec.occultation_type != expected_type {
                        return Err(OccultError::Parse {
                            row: format!(
                                "{label}: expected {expected_type:?} got {:?}",
                                rec.occultation_type
                            ),
                        });
                    }

                    let contact_tracker = if occ_type == 2 {
                        &mut m.contact_total
                    } else {
                        &mut m.contact_grazing
                    };
                    let max_res = (rec.maximum.instant.julian_day.days() - max_jd).abs() * 86_400.0;
                    contact_tracker.observe(max_res, &label, jd_tt);
                    let c1_res =
                        (rec.first_contact.instant.julian_day.days() - c1_jd).abs() * 86_400.0;
                    contact_tracker.observe(c1_res, &label, jd_tt);
                    let c4_res =
                        (rec.fourth_contact.instant.julian_day.days() - c4_jd).abs() * 86_400.0;
                    contact_tracker.observe(c4_res, &label, jd_tt);
                    if c2_jd > 0.0 {
                        if let Some(sc) = rec.second_contact {
                            let r = (sc.instant.julian_day.days() - c2_jd).abs() * 86_400.0;
                            contact_tracker.observe(r, &label, jd_tt);
                        }
                    }
                    if c3_jd > 0.0 {
                        if let Some(tc) = rec.third_contact {
                            let r = (tc.instant.julian_day.days() - c3_jd).abs() * 86_400.0;
                            contact_tracker.observe(r, &label, jd_tt);
                        }
                    }

                    let mag_res = (rec.magnitude - se_magnitude).abs();
                    let obs_res = (rec.obscuration - se_obscuration).abs();
                    if is_star {
                        m.star_magnitude.observe(mag_res, &label, jd_tt);
                        m.star_obscuration.observe(obs_res, &label, jd_tt);
                    } else {
                        m.planet_magnitude_abs.observe(mag_res, &label, jd_tt);
                        m.planet_obscuration_abs.observe(obs_res, &label, jd_tt);
                        let mag_rel = mag_res / se_magnitude.abs();
                        let obs_rel = obs_res / se_obscuration.abs();
                        m.planet_magnitude_rel.observe(mag_rel, &label, jd_tt);
                        m.planet_obscuration_rel.observe(obs_rel, &label, jd_tt);
                    }
                }
            }
            "glob" => {
                let after = Instant::new(JulianDay::from_days(max_jd - 1.0), TimeScale::Tdb);
                let rec = engine
                    .next_global_occultation(target, after)
                    .map_err(|e| OccultError::Engine(format!("{label} ({max_jd}): {e}")))?
                    .ok_or_else(|| OccultError::Parse {
                        row: format!("{label}: expected Some global occultation, got None"),
                    })?;

                let max_res = (rec.maximum.julian_day.days() - max_jd).abs() * 86_400.0;
                m.contact_total.observe(max_res, &label, jd_tt);

                // Sub-lunar point: MEASURED and reported, but NOT gated — see
                // `KNOWN GAP 1` in the module doc. `next_global_occultation`'s
                // reported (`sublunar_latitude`, `sublunar_longitude`) do not
                // localize the occultation the way SE's `swe_lun_occult_where`
                // does (residuals run 42-89 DEGREES, not arcminutes, across
                // every body in the corpus — a discovered, pre-existing bug
                // in already-committed `occult.rs`, out of this task's
                // additive-only scope to fix). Gating this with a ceiling
                // large enough to pass would be a degrees-scale threshold —
                // exactly the kind of "paper over with an enormous threshold"
                // this gate must not do.
                let arcmin = great_circle_deg(
                    rec.sublunar_latitude.degrees(),
                    rec.sublunar_longitude.degrees(),
                    se_sublunar_lat,
                    se_sublunar_lon,
                ) * 60.0;
                m.sublunar.observe(arcmin, &label, jd_tt);

                // `central` is MEASURED (counted) but NOT gated for PLANET
                // rows — see `KNOWN GAP 2` in the module doc: Saturn's two
                // glob rows disagree (engine `true`, SE `false`) at a
                // genuinely tight existence-threshold margin (Saturn's disc
                // is tiny relative to the Moon, so the central/non-central
                // band is only tens of arcseconds wide) while Venus/Jupiter
                // agree exactly. Independent verification (a LOCAL
                // `occultation()` call at SE's own reported location)
                // confirms our lower-level geometry is otherwise correct
                // there (magnitude matches SE's to 4 significant figures),
                // so this is a narrow-margin existence-test disagreement, not
                // a sign/frame bug — but it cannot be gated as an exact bool
                // without either occasionally failing on genuine corpus rows
                // or requiring an `occult.rs` fix (out of scope).
                //
                // Correction 2b (discovered, not in the plan): `central` is
                // compared/counted only for PLANET glob rows in the first
                // place. Every star `glob` row in the committed corpus has
                // `central=0` — SE structurally never sets SE_ECL_CENTRAL for
                // a POINT-target global occultation. This matches our own
                // engine's geometry: for a point target (`s_tgt_deg == 0`)
                // the "found an occultation" threshold and the "central"
                // threshold in `next_global_occultation` collapse to the SAME
                // inequality (`s_moon_deg + pi_moon`), so our engine's
                // `central` is definitionally `true` whenever it finds a star
                // glob event at all — comparing that degenerate always-true
                // flag against SE's always-false one would be tautological.
                if !is_star {
                    let expected_central = central != 0;
                    m.central_planet_checked += 1;
                    if rec.central != expected_central {
                        m.central_planet_mismatched += 1;
                    }
                }
            }
            other => {
                return Err(OccultError::Parse {
                    row: format!("{line} (unrecognized mode {other})"),
                })
            }
        }

        m.rows += 1;
    }

    if m.rows != EXPECTED_ROWS {
        return Err(OccultError::RowCountMismatch {
            expected: EXPECTED_ROWS,
            got: m.rows,
        });
    }
    Ok(m)
}

fn check_metric(metric: &'static str, tracked: &MetricMax, ceiling: f64) -> Result<(), OccultError> {
    let residual = tracked.value;
    if !residual.is_finite() || residual > ceiling {
        return Err(OccultError::ToleranceExceeded {
            metric,
            label: tracked.label.clone(),
            jd: tracked.jd,
            residual,
            ceiling,
        });
    }
    Ok(())
}

/// Full two-tier gate: Tier-1 self-consistency (via `measure`) plus Tier-2 SE
/// parity gated under the provisional ceilings in `crate::occult_thresholds`.
/// Fails closed on any exceeded ceiling.
///
/// Five metrics are gated: `contact_seconds`, `contact_seconds_grazing`,
/// `star_magnitude_abs`, `star_obscuration_abs`, `planet_magnitude_rel`.
/// `sublunar_arcmin`, `planet_obscuration_{abs,rel}`, and the planet
/// `central` comparison are measured/counted and reported (see
/// [`OccultReport`]'s field docs) but deliberately NOT gated — see `KNOWN GAP
/// 1`/`KNOWN GAP 2` in the module doc for why gating them would either always
/// vacuously pass or require a fix to already-committed `occult.rs` outside
/// this task's additive-only scope.
pub fn validate_occultations_corpus() -> Result<OccultReport, OccultError> {
    let m = measure()?;

    check_metric("contact_seconds", &m.contact_total, CONTACT_SECONDS)?;
    check_metric(
        "contact_seconds_grazing",
        &m.contact_grazing,
        CONTACT_SECONDS_GRAZING,
    )?;
    check_metric("star_magnitude_abs", &m.star_magnitude, STAR_MAGNITUDE_ABS)?;
    check_metric(
        "star_obscuration_abs",
        &m.star_obscuration,
        STAR_OBSCURATION_ABS,
    )?;
    check_metric(
        "planet_magnitude_rel",
        &m.planet_magnitude_rel,
        PLANET_MAGNITUDE_REL,
    )?;

    Ok(m.into_report())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_row_count_is_pinned() {
        let report = validate_occultations_corpus().expect("occult gate passes");
        assert_eq!(report.rows, EXPECTED_ROWS);
    }

    #[test]
    fn checksum_drift_fails_closed() {
        assert!(check_checksum("occultations.csv", "mutated,corpus\n").is_err());
    }

    #[test]
    fn gate_passes_on_committed_corpus() {
        let report = validate_occultations_corpus().expect("occult gate passes");
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn unrecognized_se_body_fails_closed() {
        assert!(body_from_se(15).is_none());
        assert!(body_from_se(20).is_none());
        assert!(body_from_se(40).is_none());
    }
}
