//! Measured-basis ceilings for the `validate-nod-aps` SE-parity gate,
//! mirroring the measured-basis convention of `fictitious_thresholds` /
//! `eclipse_local_thresholds` / `lilith_validation`. Pinned in Task 9 from
//! the actual per-category residual maxima measured over the committed
//! 184-row corpus (`data/nod-aps-corpus/`, 2026-07-07), each ceiling set to
//! ~1.4× the observed maximum, rounded up to a clean value — replacing the
//! Task 8 provisional (deliberately generous) placeholders.
//!
//! Categories (see `crate::nod_aps_validation`'s module doc for the split):
//! `MEAN_PLANET`, `MEAN_MOON`, `OSCU_PLANET`, `OSCU_MOON` — mean vs
//! osculating (methods 2 and 4 both count as osculating), Moon vs
//! everything else (Sun and the eight classical planets). Sun ascending/
//! descending rows are excluded from these ceilings entirely: since Task 9
//! (§R8) they are asserted exactly zero (both engine and SE side) rather
//! than gated against a residual ceiling — see `nod_aps_validation`'s
//! `assert_sun_node_zeroed`.
//!
//! Metrics, per compared point (ascending node, descending node,
//! perihelion, aphelion-or-focus):
//! - `_LONGITUDE_ARCSEC`: wrap-aware ecliptic-longitude residual vs SE, arcsec.
//! - `_LATITUDE_ARCSEC`: ecliptic-latitude residual vs SE, arcsec.
//! - `_DISTANCE_REL`: `|Δd| / d_se`, relative.
//! - `_LON_SPEED_DEG_DAY`: `|Δ(dλ/dt)|` vs SE, degrees/day.
//!
//! §R2 note: MEAN_MOON's measured longitude max came in at 0.561″ — far
//! below the plan's 120″ Moon-mean-correction trigger — so that follow-up
//! does not fire; no follow-up issue is needed for this category.

/// Measured max 0.658″ on 2026-07-07 corpus; ceiling ~1.4×.
pub const MEAN_PLANET_LONGITUDE_ARCSEC: f64 = 1.0;
/// Measured max 1.528″ on 2026-07-07 corpus; ceiling ~1.4×.
pub const MEAN_PLANET_LATITUDE_ARCSEC: f64 = 2.2;
/// Measured max 2.095e-5 on 2026-07-07 corpus; ceiling ~1.4×.
pub const MEAN_PLANET_DISTANCE_REL: f64 = 3.0e-5;
/// Measured max 0.1799 deg/day on 2026-07-07 corpus; ceiling ~1.4×.
pub const MEAN_PLANET_LON_SPEED_DEG_DAY: f64 = 0.3;

/// Measured max 0.561″ on 2026-07-07 corpus; ceiling ~1.4×. Far below the
/// plan's 120″ §R2 Moon-mean-correction trigger — that follow-up does not
/// fire for this corpus.
pub const MEAN_MOON_LONGITUDE_ARCSEC: f64 = 0.8;
/// Measured max 0.0385″ on 2026-07-07 corpus; ceiling ~1.4×.
pub const MEAN_MOON_LATITUDE_ARCSEC: f64 = 0.06;
/// Measured max 1.121e-6 on 2026-07-07 corpus; ceiling ~1.4×.
pub const MEAN_MOON_DISTANCE_REL: f64 = 2.0e-6;
/// Measured max 3.43e-6 deg/day on 2026-07-07 corpus; ceiling ~1.4×.
pub const MEAN_MOON_LON_SPEED_DEG_DAY: f64 = 5.0e-6;

/// Measured max 1415.477″ on 2026-07-07 corpus; ceiling ~1.4×. The max is a
/// heliocentric (method=2) Neptune perihelion row, geometrically amplified:
/// Neptune's small eccentricity (e≈0.0086) divides a legitimate
/// ~arcsecond-class cross-ephemeris state difference by e (1/e≈100×) to
/// place the perihelion direction — nodes on the same rows agree to ~7″.
pub const OSCU_PLANET_LONGITUDE_ARCSEC: f64 = 2000.0;
/// Measured max 16.292″ on 2026-07-07 corpus; ceiling ~1.4×.
pub const OSCU_PLANET_LATITUDE_ARCSEC: f64 = 25.0;
/// Measured max 3.030e-3 on 2026-07-07 corpus; ceiling ~1.4×.
pub const OSCU_PLANET_DISTANCE_REL: f64 = 5.0e-3;
/// Measured max 0.1444 deg/day on 2026-07-07 corpus; ceiling ~1.4×.
pub const OSCU_PLANET_LON_SPEED_DEG_DAY: f64 = 0.25;

/// Measured max 3402.602″ on 2026-07-07 corpus; ceiling ~1.4×. Cross-theory
/// (our ELP/DE440 lunar backend vs SE Moshier) full-6-D osculating apse
/// line; same family as `lilith_validation`'s accepted ELP-vs-SE difference
/// (its ceiling 460″/measured 306″), larger here because this is a full
/// osculating state+speed rather than precession-only. Confirmed
/// apse-concentrated by a per-point decomposition of the OSCU_MOON rows: the
/// worst row (jd 2477476.5) has ascending/descending residuals of only
/// 14.108″ while perihelion/aphelion carry the full 3402.602″ — the same
/// eccentricity/theory signature as the OSCU_PLANET Neptune case above, not
/// a lunar-osculating bug.
pub const OSCU_MOON_LONGITUDE_ARCSEC: f64 = 5000.0;
/// Measured max 72.933″ on 2026-07-07 corpus; ceiling ~1.4×.
pub const OSCU_MOON_LATITUDE_ARCSEC: f64 = 110.0;
/// Measured max 8.033e-4 on 2026-07-07 corpus; ceiling ~1.4×.
pub const OSCU_MOON_DISTANCE_REL: f64 = 1.5e-3;
/// Measured max 2.634 deg/day on 2026-07-07 corpus; ceiling ~1.4× (≈3.69,
/// rounded to 4.0). Same cross-theory osculating-apse-line mechanism as the
/// longitude ceiling above; dropped from the Task 8 provisional 5.0 now
/// that the true maximum is measured.
pub const OSCU_MOON_LON_SPEED_DEG_DAY: f64 = 4.0;
