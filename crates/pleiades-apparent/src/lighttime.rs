//! Light-time (planetary aberration) iteration: re-evaluate the geocentric
//! position at the retarded epoch t - τ until it converges.

use pleiades_types::{EclipticCoordinates, Instant, JulianDay};

use crate::error::{ApparentLightTimeError, ApparentPlaceError};

/// Light travel time across one AU, in days (≈ 499.0047 s).
pub const LIGHT_TIME_DAYS_PER_AU: f64 = 0.005_775_518_3;

/// Convergence threshold on the retardation, in days (≈ 0.04 s).
const CONVERGENCE_DAYS: f64 = 5e-7;

/// Maximum plausible light-time retardation, in days.
///
/// Pluto at aphelion is ~49 AU → light-time ≈ 0.28 days. This cap of 10 days
/// (≈ 1730 AU) is far above any real solar-system body handled by the engine
/// and far below the ~167-day garbage value emitted for 433-Eros at 1900 when
/// the packaged distance channel is unreliable. Exceeding this cap is treated
/// as a non-convergent result (fail-closed).
const MAX_PLAUSIBLE_LIGHT_TIME_DAYS: f64 = 10.0;

/// A light-time-corrected geocentric position and the retardation used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTimePosition {
    /// Geocentric ecliptic position at the retarded epoch.
    pub ecliptic: EclipticCoordinates,
    /// Light-time retardation applied, in days.
    pub light_time_days: f64,
    /// Iterations taken to converge.
    pub iterations: u8,
}

/// Iterates t - τ until the retardation converges. `query` returns the
/// geocentric ecliptic position (with `distance_au`) at a given instant.
pub fn apparent_via_light_time<F, E>(
    instant: Instant,
    max_iterations: u8,
    mut query: F,
) -> Result<LightTimePosition, ApparentLightTimeError<E>>
where
    F: FnMut(Instant) -> Result<EclipticCoordinates, E>,
{
    let base_jd = instant.julian_day.days();
    let mut tau = 0.0_f64;
    let mut last = query(instant).map_err(ApparentLightTimeError::Query)?;
    for step in 1..=max_iterations {
        let distance = last.distance_au.ok_or(ApparentLightTimeError::Apparent(
            ApparentPlaceError::MissingDistance,
        ))?;
        let new_tau = distance * LIGHT_TIME_DAYS_PER_AU;
        if !new_tau.is_finite() {
            return Err(ApparentLightTimeError::Apparent(
                ApparentPlaceError::NonFiniteCorrection {
                    stage: "light-time",
                },
            ));
        }
        if new_tau > MAX_PLAUSIBLE_LIGHT_TIME_DAYS {
            return Err(ApparentLightTimeError::Apparent(
                ApparentPlaceError::NonConvergentLightTime { iterations: step },
            ));
        }
        if (new_tau - tau).abs() < CONVERGENCE_DAYS {
            return Ok(LightTimePosition {
                ecliptic: last,
                light_time_days: new_tau,
                iterations: step,
            });
        }
        tau = new_tau;
        let retarded = Instant::new(JulianDay::from_days(base_jd - tau), instant.scale);
        last = query(retarded).map_err(ApparentLightTimeError::Query)?;
    }
    Err(ApparentLightTimeError::Apparent(
        ApparentPlaceError::NonConvergentLightTime {
            iterations: max_iterations,
        },
    ))
}

#[cfg(test)]
mod tests;
