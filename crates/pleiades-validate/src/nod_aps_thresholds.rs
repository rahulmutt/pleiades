//! Provisional ceilings for the `validate-nod-aps` SE-parity gate (Task 8),
//! mirroring the measured-basis convention of `fictitious_thresholds` /
//! `eclipse_local_thresholds` — except these are NOT yet measured-basis.
//!
//! Task 8 wires the gate with deliberately GENEROUS, provisional ceilings so
//! it can be exercised end-to-end before the real per-category residual
//! maxima are known. Task 9 measures the actual maxima over the committed
//! 184-row corpus (`data/nod-aps-corpus/`) and tightens each constant to
//! ~1.4x the observed value, per the plan.
//!
//! Categories (see `crate::nod_aps_validation`'s module doc for the split):
//! `MEAN_PLANET`, `MEAN_MOON`, `OSCU_PLANET`, `OSCU_MOON` — mean vs
//! osculating (methods 2 and 4 both count as osculating), Moon vs
//! everything else (Sun and the eight classical planets).
//!
//! Metrics, per compared point (ascending node, descending node,
//! perihelion, aphelion-or-focus):
//! - `_LONGITUDE_ARCSEC`: wrap-aware ecliptic-longitude residual vs SE, arcsec.
//! - `_LATITUDE_ARCSEC`: ecliptic-latitude residual vs SE, arcsec.
//! - `_DISTANCE_REL`: `|Δd| / d_se`, relative.
//! - `_LON_SPEED_DEG_DAY`: `|Δ(dλ/dt)|` vs SE, degrees/day.

/// Provisional longitude ceiling, MEAN_PLANET (arcsec). Tightened in Task 9.
pub const MEAN_PLANET_LONGITUDE_ARCSEC: f64 = 3600.0;
/// Provisional latitude ceiling, MEAN_PLANET (arcsec). Tightened in Task 9.
pub const MEAN_PLANET_LATITUDE_ARCSEC: f64 = 3600.0;
/// Provisional distance ceiling, MEAN_PLANET (relative). Tightened in Task 9.
pub const MEAN_PLANET_DISTANCE_REL: f64 = 1e-2;
/// Provisional longitude-speed ceiling, MEAN_PLANET (deg/day). Tightened in Task 9.
pub const MEAN_PLANET_LON_SPEED_DEG_DAY: f64 = 1.0;

/// Provisional longitude ceiling, MEAN_MOON (arcsec). Tightened in Task 9.
pub const MEAN_MOON_LONGITUDE_ARCSEC: f64 = 3600.0;
/// Provisional latitude ceiling, MEAN_MOON (arcsec). Tightened in Task 9.
pub const MEAN_MOON_LATITUDE_ARCSEC: f64 = 3600.0;
/// Provisional distance ceiling, MEAN_MOON (relative). Tightened in Task 9.
pub const MEAN_MOON_DISTANCE_REL: f64 = 1e-2;
/// Provisional longitude-speed ceiling, MEAN_MOON (deg/day). Tightened in Task 9.
pub const MEAN_MOON_LON_SPEED_DEG_DAY: f64 = 1.0;

/// Provisional longitude ceiling, OSCU_PLANET (arcsec). Tightened in Task 9.
pub const OSCU_PLANET_LONGITUDE_ARCSEC: f64 = 3600.0;
/// Provisional latitude ceiling, OSCU_PLANET (arcsec). Tightened in Task 9.
pub const OSCU_PLANET_LATITUDE_ARCSEC: f64 = 3600.0;
/// Provisional distance ceiling, OSCU_PLANET (relative). Tightened in Task 9.
pub const OSCU_PLANET_DISTANCE_REL: f64 = 1e-2;
/// Provisional longitude-speed ceiling, OSCU_PLANET (deg/day). Tightened in Task 9.
pub const OSCU_PLANET_LON_SPEED_DEG_DAY: f64 = 1.0;

/// Provisional longitude ceiling, OSCU_MOON (arcsec). Tightened in Task 9.
pub const OSCU_MOON_LONGITUDE_ARCSEC: f64 = 3600.0;
/// Provisional latitude ceiling, OSCU_MOON (arcsec). Tightened in Task 9.
pub const OSCU_MOON_LATITUDE_ARCSEC: f64 = 3600.0;
/// Provisional distance ceiling, OSCU_MOON (relative). Tightened in Task 9.
pub const OSCU_MOON_DISTANCE_REL: f64 = 1e-2;
// provisional; ±0.5-day central-difference vs SE's ±dt-scale sampling of a
// fast-oscillating osculating lunar apse line; Task 9 pins measured
/// Provisional longitude-speed ceiling, OSCU_MOON (deg/day). Raised from the
/// generic 1.0 provisional ceiling (measured max 2.634 deg/day) — this is
/// still a provisional pre-Task-9 ceiling, not a final claim.
pub const OSCU_MOON_LON_SPEED_DEG_DAY: f64 = 5.0;
