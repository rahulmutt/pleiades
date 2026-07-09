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
//! instead of SE's actual central-observation point (residual 42-89 DEGREES;
//! resolved — see `KNOWN GAP 2` in `crate::occult_validation`). After the fix,
//! `sublunar_arcmin` collapsed to arcmin-scale (measured max 69.628' —
//! `Regulus`, glob row, event instant jd 2457741.256640); a subsequent code
//! review found the fixed 8-round sub-lunar minimizer still under-converged
//! the worst-conditioned rows, so `occult.rs` was tightened to iterate to a
//! convergence tolerance (bounded backstop), which dropped the measured max
//! further to 20.218' (`Antares`, glob row, jd 2453378.335434) on
//! 2026-07-08. `planet_obscuration_rel_grazing` (grazing-only planet
//! obscuration, newly comparable once the central point was independently
//! confirmed correct) measured max 4.934% (`Saturn@graze`, jd
//! 2452052.796691) and is also now gated. Both pinned to ~1.4× measured,
//! rounded up to a clean value.
//!
//! One metric remains deliberately NOT gated — see
//! `crate::occult_validation`'s module doc for the full diagnosis:
//! `planet_obscuration_{abs,rel}` (Total-inclusive; `KNOWN GAP 1` — SE's
//! `attr[2]` for a fully-covered planet is a different, coverage-depth-ratio
//! quantity a bounded `[0,1]` area fraction cannot and should not reach). The
//! `central` exact-bool comparison (`KNOWN GAP 2`, resolved — see `KNOWN GAP
//! 2` in `crate::occult_validation`) was formerly measured-but-ungated
//! (Saturn's 2/6 glob rows disagreed even after the sub-lunar-point fix) but
//! is now resolved and hard-gated (SP-6-FU): the engine ports SE's own
//! closed-form axis-pierce test rather than deriving `central` from
//! `occ_type`, collapsing the Saturn mismatch to 0/6 for both planet and
//! (newly measured) star `glob` rows.
//!
//! The `miss_classify_disagree` COUNT ceiling
//! (`crate::occult_validation::MAX_MISS_CLASSIFICATION_DISAGREEMENTS`, not a
//! constant in this module but cross-referenced here for completeness) is
//! not a residual-magnitude threshold like the ones above — it pins how
//! many sibling-anchored geometric-miss rows may disagree with SE, per
//! `KNOWN GAP 3` (RESOLVED, SP-6-FU) in `crate::occult_validation`. SP-6-FU
//! reconciled the gate's comparison with SE's own `when_loc` visibility
//! semantics (source-confirmed, `swecl.c:2700-2732`), dropping the measured
//! disagreement from 8/18 to 3/18 and tightening the pin to 3; the 3
//! residual rows are a scan-vs-5-instant visibility-sampling delta at the
//! horizon, not a magnitude residual this module's ~1.4×-measured
//! convention would apply to.

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
/// — resolved, see `KNOWN GAP 2` in `crate::occult_validation`) and SE's
/// `sublunar_lat/lon`, over all `glob` rows. A code review of Task 15 found
/// the sub-lunar minimizer's fixed 8-round golden-section coordinate descent
/// under-converged the worst-conditioned rows; `occult.rs`'s
/// `minimize_sublunar_point` was tightened to iterate to a convergence
/// tolerance (bounded by a 48-round backstop) instead. Measured max dropped
/// from 69.628' (`Regulus`, glob row, jd 2457741.256640) to 20.218'
/// (`Antares`, glob row, event instant jd 2453378.335434) on 2026-07-08,
/// post-convergence-fix.
pub const SUBLUNAR_ARCMIN: f64 = 30.0;
/// Covered-area fraction (obscuration) RELATIVE residual for PLANET GRAZING
/// (`occ_type == 1`) rows only — bucketed exactly like `CONTACT_SECONDS` vs
/// `CONTACT_SECONDS_GRAZING`. Unlike the Total-inclusive
/// `planet_obscuration_rel` (ungated, see `KNOWN GAP 1`), the Grazing-only
/// residual IS comparable in scale to the gated `PLANET_MAGNITUDE_REL`.
/// Measured max 4.934% (`Saturn@graze`, jd 2452052.796691) on 2026-07-08.
pub const PLANET_OBSCURATION_REL: f64 = 0.07;
