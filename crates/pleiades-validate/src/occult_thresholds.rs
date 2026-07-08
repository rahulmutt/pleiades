//! SE-parity ceilings for the `validate-occultations` gate (SP-6). Each is set
//! from the measured per-metric residual maximum × ~1.4, matching the
//! `pheno_thresholds`/`rise_trans_thresholds` convention, measured against the
//! committed `data/occultations-corpus/occultations.csv` (62 rows) on
//! 2026-07-08.
//!
//! Two metrics from the original plan (`sublunar_arcmin`, the planet
//! `obscuration` residual) are deliberately NOT represented here as gated
//! ceilings — see `crate::occult_validation`'s module doc, `KNOWN GAP 1`
//! (planet `obscuration` is not the same physical quantity on the SE vs.
//! engine side — any ceiling loose enough to pass is vacuous) and
//! `KNOWN GAP 2` (`next_global_occultation`'s sub-lunar point is off by
//! 42-89 DEGREES from SE's, a discovered bug in already-committed
//! `occult.rs` that is out of this task's additive-only scope to fix).
//! Gating either would either always vacuously pass or require modifying
//! code outside this task's scope, so both are measured and reported
//! (informational, see `OccultReport`) but not gated here.

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
