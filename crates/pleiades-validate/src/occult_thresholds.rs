//! SE-parity ceilings for the `validate-occultations` gate (SP-6). Each is set
//! from the measured per-metric residual maximum × ~1.4, matching the
//! `pheno_thresholds`/`rise_trans_thresholds` convention, measured against the
//! committed `data/occultations-corpus/occultations.csv` (62 rows).
//!
//! `CONTACT_SECONDS`, `CONTACT_SECONDS_GRAZING`, `STAR_MAGNITUDE_ABS`,
//! `STAR_OBSCURATION_ABS`, and `PLANET_MAGNITUDE_REL` were measured/pinned on
//! 2026-07-08 (Task 9/11, before Task 15's engine fix below — that fix did
//! not change any of these five residuals, all measured from `occultation()`/
//! `next_occultation()` local circumstances, unaffected by
//! `next_global_occultation`'s sub-lunar-point bug).
//!
//! `SUBLUNAR_ARCMIN` and `PLANET_OBSCURATION_REL` were added on 2026-07-08
//! (Task 15, a corrective task) after fixing a confirmed engine bug in
//! `next_global_occultation` (`crates/pleiades-events/src/occult.rs`): it
//! used to report the Moon's geocentric zenith point as the sub-lunar point
//! instead of SE's actual central-observation point (residual 42-89 DEGREES,
//! see `crate::occult_validation`'s module doc `KNOWN GAP 2`). After the fix,
//! `sublunar_arcmin` collapsed to arcmin-scale (measured max 69.628' —
//! `Regulus`, glob row, event instant jd 2457741.256640) and is now gated
//! here; `planet_obscuration_rel_grazing` (grazing-only planet obscuration,
//! newly comparable once the central point was independently confirmed
//! correct) measured max 4.934% (`Saturn@graze`, jd 2452052.796691) and is
//! also now gated. Both pinned to ~1.4× measured, rounded up to a clean
//! value.
//!
//! Two related metrics remain deliberately NOT gated — see
//! `crate::occult_validation`'s module doc for the full diagnosis:
//! `planet_obscuration_{abs,rel}` (Total-inclusive; `KNOWN GAP 1` — SE's
//! `attr[2]` for a fully-covered planet is a different, coverage-depth-ratio
//! quantity a bounded `[0,1]` area fraction cannot and should not reach), and
//! the planet `central` exact-bool comparison (`KNOWN GAP 2` — Saturn's 2/6
//! glob rows still disagree even after the sub-lunar-point fix; diagnosed as
//! a conceptual difference in what "central" means, not a positional error).

/// Contact/maximum instant residual vs SE, seconds (well-conditioned: `Total`
/// `loc` rows and all `glob` rows). Measured max 46.44s (Regulus@center,
/// jd 2460882.894225485) on 2026-07-08.
pub const CONTACT_SECONDS: f64 = 65.0;
/// Contact/maximum instant residual near grazing/limb chords, seconds.
/// Grazing contact timing is inherently ill-conditioned (near-tangential
/// geometry: the relative closing rate perpendicular to the limb is small,
/// so a sub-arcminute positional difference amplifies into a much larger
/// timing difference — an expected consequence of the geometry, not a
/// bug). Measured max 710.03s (Saturn@graze, jd 2452052.268799137) on
/// 2026-07-08.
pub const CONTACT_SECONDS_GRAZING: f64 = 995.0;
/// Covered-diameter fraction (magnitude) absolute residual for a point-star
/// target (binary in `{0,1}` on the SE side — an absolute ceiling is
/// meaningful here). Measured max 0.0 (exact match on every star row) on
/// 2026-07-08; pinned to a small nonzero floor to keep the check meaningful
/// against float noise rather than gating at literal zero.
pub const STAR_MAGNITUDE_ABS: f64 = 1e-6;
/// Covered-area fraction (obscuration) absolute residual for a point-star
/// target. Measured max 0.0 (exact match on every star row) on 2026-07-08;
/// same float-noise floor as `STAR_MAGNITUDE_ABS`.
pub const STAR_OBSCURATION_ABS: f64 = 1e-6;
/// Covered-diameter fraction (magnitude) RELATIVE residual
/// (`|recomputed - se| / |se|`) for a planet target. SE's planet magnitude
/// values run into the tens (18-82 in the committed corpus, a fraction of
/// the planet's own tiny disc covered by the much larger Moon), so an
/// absolute ceiling tuned for a star (~1.0) is meaningless here; the
/// relative residual is the metric that is actually comparable in scale.
/// Measured max 4.89% (`Saturn@graze`, jd 2452052.268799137) on 2026-07-08.
pub const PLANET_MAGNITUDE_REL: f64 = 0.07;
/// Great-circle residual (arcmin) between the recomputed central-observation
/// point (`GlobalOccultation::sublunar_latitude/longitude`, fixed in Task 15
/// — see `crate::occult_validation`'s `KNOWN GAP 2`) and SE's
/// `sublunar_lat/lon`, over all `glob` rows. Measured max 69.628' (`Regulus`,
/// glob row, event instant jd 2457741.256640) on 2026-07-08, post-fix.
pub const SUBLUNAR_ARCMIN: f64 = 100.0;
/// Covered-area fraction (obscuration) RELATIVE residual for PLANET GRAZING
/// (`occ_type == 1`) rows only — bucketed exactly like `CONTACT_SECONDS` vs
/// `CONTACT_SECONDS_GRAZING`. Unlike the Total-inclusive
/// `planet_obscuration_rel` (ungated, see `KNOWN GAP 1`), the Grazing-only
/// residual IS comparable in scale to the gated `PLANET_MAGNITUDE_REL`.
/// Measured max 4.934% (`Saturn@graze`, jd 2452052.796691) on 2026-07-08.
pub const PLANET_OBSCURATION_REL: f64 = 0.07;
