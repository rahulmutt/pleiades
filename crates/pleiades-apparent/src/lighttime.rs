//! Light-time (planetary aberration) iteration: re-evaluate the geocentric
//! position at the retarded epoch t - τ until it converges.
//!
//! The first two positions come from the backend: at `t` and at `t − τ₁`. A
//! later retarded epoch `t − τₙ` that lies within a small fraction of that
//! spacing beyond the last queried epoch is read off the straight line through
//! the two queried positions instead of re-querying the backend (#247). For a
//! slow outer planet the third query would only move the retarded epoch by
//! `τ·ṙ/c ≈ 1e-5` d. On the VSOP87 backend over 1,200 monthly samples from
//! 2026, the interpolated position differs from the queried one by at most
//! 1.0e-9° (3.7 µas, Mercury) in longitude, and every planet costs two backend
//! queries per apparent sample instead of up to three.

use pleiades_types::{EclipticCoordinates, Instant, JulianDay, Latitude, Longitude};

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

/// Largest step beyond the last queried retardation, as a fraction of the
/// spacing between the two queried retardations, that is interpolated rather
/// than queried.
///
/// Linear interpolation through the queried epochs `t` and `t − τ₁` to
/// `t − (τ₁ + δ)` errs by about `½·a·τ₁·δ` for a geocentric angular
/// acceleration `a`. Real bodies have `δ/τ₁ = ṙ/c ≲ 3e-4`, so this bound always
/// admits them, and the error stays near 1e-9° (Neptune: `a` ≈ 6e-4 °/d² from
/// the Earth's annual parallax, `τ₁` ≈ 0.17 d, `δ` ≈ 2e-5 d). A larger step (a
/// synthetic or unreliable query) is queried.
const MAX_INTERPOLATION_STEP_RATIO: f64 = 1e-3;

/// A backend-queried geocentric position, as a Cartesian ecliptic vector in AU,
/// at retardation `tau` days.
#[derive(Clone, Copy)]
struct QueriedSample {
    tau: f64,
    position: [f64; 3],
}

impl QueriedSample {
    fn new(tau: f64, ecliptic: &EclipticCoordinates, distance_au: f64) -> Self {
        let (sin_lon, cos_lon) = ecliptic.longitude.degrees().to_radians().sin_cos();
        let (sin_lat, cos_lat) = ecliptic.latitude.degrees().to_radians().sin_cos();
        Self {
            tau,
            position: [
                distance_au * cos_lat * cos_lon,
                distance_au * cos_lat * sin_lon,
                distance_au * sin_lat,
            ],
        }
    }
}

/// The position at retardation `tau` on the line through two queried samples,
/// or `None` when `tau` lies further beyond `latest` than
/// [`MAX_INTERPOLATION_STEP_RATIO`] of their spacing.
fn interpolate(
    previous: QueriedSample,
    latest: QueriedSample,
    tau: f64,
) -> Option<EclipticCoordinates> {
    let spacing = latest.tau - previous.tau;
    if spacing == 0.0 || (tau - latest.tau).abs() > MAX_INTERPOLATION_STEP_RATIO * spacing.abs() {
        return None;
    }
    let fraction = (tau - previous.tau) / spacing;
    let [x, y, z]: [f64; 3] = std::array::from_fn(|axis| {
        previous.position[axis] + fraction * (latest.position[axis] - previous.position[axis])
    });
    let distance = (x * x + y * y + z * z).sqrt();
    Some(EclipticCoordinates::new(
        Longitude::from_degrees(y.atan2(x).to_degrees()),
        Latitude::from_degrees(z.atan2(x.hypot(y)).to_degrees()),
        Some(distance),
    ))
}

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
///
/// Once two positions have been queried, a further retarded epoch close to the
/// last queried one is interpolated between them instead of queried (see the
/// module docs). `iterations` counts every step, queried or interpolated.
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
    let mut last_is_queried = true;
    let mut previous_queried: Option<QueriedSample> = None;
    let mut latest_queried: Option<QueriedSample> = None;
    for step in 1..=max_iterations {
        let distance = last.distance_au.ok_or(ApparentLightTimeError::Apparent(
            ApparentPlaceError::MissingDistance,
        ))?;
        if last_is_queried {
            previous_queried = latest_queried;
            latest_queried = Some(QueriedSample::new(tau, &last, distance));
        }
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
        let interpolated = previous_queried
            .zip(latest_queried)
            .and_then(|(previous, latest)| interpolate(previous, latest, tau));
        last_is_queried = interpolated.is_none();
        last = match interpolated {
            Some(position) => position,
            None => {
                let retarded = Instant::new(JulianDay::from_days(base_jd - tau), instant.scale);
                query(retarded).map_err(ApparentLightTimeError::Query)?
            }
        };
    }
    Err(ApparentLightTimeError::Apparent(
        ApparentPlaceError::NonConvergentLightTime {
            iterations: max_iterations,
        },
    ))
}

#[cfg(test)]
mod tests;
