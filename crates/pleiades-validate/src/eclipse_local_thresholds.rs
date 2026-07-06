//! Per-category parity ceilings for `validate-eclipses-local`, mirroring the
//! measured-basis convention of `rise_trans_thresholds`. Each constant is set
//! from the MEASURED maximum residual over the committed corpus (~1.4× the
//! observed max, rounded clean), not guessed.
//!
//! All maxima were measured (Task 11) AFTER the two engine fixes in
//! `pleiades-eclipse/src/local.rs::topo_sun_moon` landed:
//!
//! 1. **Apparent-of-date frame.** The backend samples Mean/J2000 geocentric
//!    Sun/Moon (see `ephemeris::sample_sun_moon`); `topo_sun_moon` now carries
//!    both bodies to apparent ecliptic-of-date (precession J2000→date + nutation
//!    in longitude, frame-common to both so the Sun−Moon separation is
//!    unchanged) before the diurnal-parallax / horizontal step, which already
//!    used of-date obliquity and sidereal time. This removes the ~0.28°/20 yr
//!    precession offset that had corrupted absolute az/alt.
//! 2. **ΔT-corrected (UT1) parallax rotation.** Diurnal parallax rotates the
//!    observer's offset with the true Earth orientation (a function of UT1), so
//!    `topo_sun_moon` converts the dynamical instant to UT1 (`ut1_jd_from_tt`)
//!    before taking sidereal time. Without it the parallax was rotated ~ΔT≈69 s
//!    off, biasing the observer-local greatest-eclipse / contact instants
//!    (found by minimizing the topocentric separation) systematically +20..+45 s
//!    versus SE's UT1-based `swe_sol_eclipse_when_loc`. This collapsed the
//!    non-grazing solar contact residual from ~114 s to ~16 s.
//!
//! The final horizontal-frame rotation in `body_horizontal` keeps the engine's
//! numeric-instant ("J as UT1") sidereal convention, matching SP-2b
//! rise/set/transit and the SE az/alt reference generation.
//!
//! Measured maxima (committed corpus, 29 solar + 20 lunar rows):
//! solar non-grazing 16.1 s, solar grazing 65.0 s (2017 KansasCity C2 at
//! magnitude 1.0002 — internal tangency, near-zero totality, the intended
//! ill-conditioned grazing case), lunar 5.0 s, solar magnitude 1.1e-3, solar
//! obscuration 1.1e-3 (SE clamped to [0,1]; see the gate), lunar magnitude
//! 7.1e-4, azimuth-on-sky 91.0″, apparent altitude 81.0″.

/// Contact/max instant parity ceiling for well-conditioned solar rows (seconds
/// of time). Measured max 16.1 s. Solar contacts near the central limit widen;
/// see `SOLAR_SECONDS_GRAZING`.
pub const SOLAR_SECONDS: f64 = 23.0;
/// Contact/max instant ceiling for grazing / central-limit solar rows (seconds).
/// Measured max 65.0 s (C2/C3 pinch at magnitude ≈ 1, near-zero totality).
pub const SOLAR_SECONDS_GRAZING: f64 = 95.0;
/// Lunar contact/max instant ceiling (seconds). Global instants. Measured 5.0 s.
pub const LUNAR_SECONDS: f64 = 7.0;
/// Solar magnitude (diameter fraction) absolute ceiling. Measured 1.1e-3.
pub const MAGNITUDE_ABS: f64 = 0.002;
/// Solar obscuration (area fraction) absolute ceiling. Measured 1.1e-3 (SE's
/// attr[2] clamped to [0,1] for total eclipses; see `measure_solar`).
pub const OBSCURATION_ABS: f64 = 0.002;
/// Lunar umbral/penumbral magnitude absolute ceiling. Measured 7.1e-4.
pub const LUNAR_MAGNITUDE_ABS: f64 = 0.001;
/// Azimuth parity ceiling: cross-track (on-sky) residual `Δaz·cos(alt)`, in
/// arcseconds. Measured max 91.0″ (raw azimuth reached ~249″ on 2024-total rows
/// whose maxima sit near local noon with the Sun high and near the meridian,
/// where azimuth is intrinsically ill-conditioned — see `azimuth_sky_arcsec`).
pub const AZIMUTH_ARCSEC: f64 = 130.0;
/// Apparent-altitude parity ceiling (arcseconds). Measured max 81.0″.
pub const ALTITUDE_ARCSEC: f64 = 120.0;
