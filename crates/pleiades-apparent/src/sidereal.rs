//! Sidereal time (GMST/GAST, Greenwich and local) for the of-date chart layer.
//!
//! Sidereal time is a function of UT1 (Earth rotation), not TT/TDB. The
//! `Instant`'s Julian Day is used as supplied by [`sidereal_time`]; pass a
//! UT1-scale instant for rigorous results (see `docs/time-observer-policy.md`).
//! Callers holding a TT/TDB instant convert it with [`ut1_instant`] first —
//! that adapter is the single sanctioned place where the `Instant`'s time-scale
//! tag drives a ΔT correction, so the house layer and any other Earth-rotation
//! consumer apply the same policy.

use pleiades_time::CivilTimeError;
use pleiades_types::{Angle, Instant, JulianDay, Longitude, TimeScale};

use crate::nutation::{mean_obliquity_degrees, nutation};

/// Greenwich Mean Sidereal Time in degrees (unnormalized).
///
/// Delegates to `pleiades_time::gmst_degrees_raw` — the single source of the
/// GMST polynomial — so the coefficients live in exactly one crate. Still
/// returns the unnormalized value; `sidereal_time` normalizes downstream.
pub fn greenwich_mean_sidereal_time_degrees(jd: f64) -> f64 {
    pleiades_time::gmst_degrees_raw(jd)
}

/// Equation of the equinoxes in degrees from its two inputs: `Δψ · cos(ε_true)`.
///
/// `delta_psi_deg` is nutation-in-longitude and `true_obliquity_deg` the true
/// obliquity, both in degrees. Single source of the equation-of-equinoxes
/// formula, shared by `equation_of_equinoxes_degrees` and the chart layer's
/// topocentric-LAST path (`pleiades-core`).
pub fn equation_of_equinoxes(delta_psi_deg: f64, true_obliquity_deg: f64) -> f64 {
    delta_psi_deg * true_obliquity_deg.to_radians().cos()
}

/// Equation of the equinoxes in degrees: `Δψ · cos(ε_true)`.
///
/// Falls back to `0.0` if the nutation table is unavailable (a development-time
/// artifact — a stale checksum — not a runtime condition), matching the prior
/// behavior in `pleiades-houses`.
pub fn equation_of_equinoxes_degrees(jd: f64) -> f64 {
    nutation(jd)
        .map(|n| {
            let delta_psi_deg = n.delta_psi_arcsec / 3600.0;
            let true_obl_deg = mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0;
            equation_of_equinoxes(delta_psi_deg, true_obl_deg)
        })
        .unwrap_or(0.0)
}

/// Sidereal time in mean/apparent form, at Greenwich and locally.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SiderealTime {
    /// Greenwich Mean Sidereal Time, degrees `[0,360)`.
    pub gmst_deg: f64,
    /// Greenwich Apparent Sidereal Time (GMST + equation of equinoxes), degrees `[0,360)`.
    pub gast_deg: f64,
    /// Local Mean Sidereal Time (GMST + east longitude), degrees `[0,360)`.
    pub local_mean_deg: f64,
    /// Local Apparent Sidereal Time (GAST + east longitude), degrees `[0,360)`.
    pub local_apparent_deg: f64,
}

impl SiderealTime {
    /// GMST in hours `[0,24)`.
    pub fn gmst_hours(&self) -> f64 {
        self.gmst_deg / 15.0
    }
    /// GAST in hours `[0,24)`.
    pub fn gast_hours(&self) -> f64 {
        self.gast_deg / 15.0
    }
    /// Local mean sidereal time in hours `[0,24)`.
    pub fn local_mean_hours(&self) -> f64 {
        self.local_mean_deg / 15.0
    }
    /// Local apparent sidereal time in hours `[0,24)`.
    pub fn local_apparent_hours(&self) -> f64 {
        self.local_apparent_deg / 15.0
    }
}

/// Re-expresses an instant on the UT1 scale for Earth-rotation quantities.
///
/// Sidereal time (and everything built on it: ARMC, house cusps, hour
/// angles) is a function of UT1. This adapter honours the `Instant`'s
/// time-scale tag:
///
/// - `Tt` and `Tdb`: subtract ΔT from `pleiades-time` and retag `Ut1`. TDB is
///   treated as TT (they differ by under 2 ms, far below sidereal-time
///   precision).
/// - `Ut1`: returned unchanged.
/// - `Utc`: retagged `Ut1` without shifting the day. |UT1 − UTC| is kept
///   under 0.9 s by leap seconds, about 0.004° of sidereal time, and this
///   crate carries no DUT1 table.
///
/// # Errors
///
/// Fails when the packaged ΔT table cannot be read (a stale checksum — a
/// development-time artifact, not a runtime condition), or when a time scale
/// this adapter has no conversion for is supplied (`TimeScale` is
/// `#[non_exhaustive]`).
///
/// The error type is `pleiades_time::CivilTimeError` from the `pleiades-time`
/// release this crate pins (0.7 and later); a caller that matches on it must
/// depend on the same `pleiades-time` release.
pub fn ut1_instant(instant: Instant) -> Result<Instant, CivilTimeError> {
    let jd_ut1 = match instant.scale {
        TimeScale::Tt | TimeScale::Tdb => pleiades_time::ut1_jd_from_tt(instant.julian_day.days())?,
        TimeScale::Ut1 | TimeScale::Utc => instant.julian_day.days(),
        other => {
            return Err(CivilTimeError::UnsupportedScale {
                source: other,
                target: TimeScale::Ut1,
            })
        }
    };
    Ok(Instant::new(JulianDay::from_days(jd_ut1), TimeScale::Ut1))
}

/// Computes sidereal time for an instant and observer east longitude.
///
/// The Julian Day is consumed as supplied, whatever its `TimeScale` tag;
/// convert TT/TDB instants with [`ut1_instant`] first.
pub fn sidereal_time(instant: Instant, observer_longitude: Longitude) -> SiderealTime {
    let jd = instant.julian_day.days();
    let gmst = greenwich_mean_sidereal_time_degrees(jd);
    let ee = equation_of_equinoxes_degrees(jd);
    let lon = observer_longitude.degrees();
    let norm = |d: f64| Angle::from_degrees(d).normalized_0_360().degrees();
    SiderealTime {
        gmst_deg: norm(gmst),
        gast_deg: norm(gmst + ee),
        local_mean_deg: norm(gmst + lon),
        local_apparent_deg: norm(gmst + ee + lon),
    }
}

#[cfg(test)]
mod tests;
