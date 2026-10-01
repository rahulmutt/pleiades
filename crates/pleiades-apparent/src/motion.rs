//! Speed of a corrected place, from the speed of its base place.
//!
//! A backend reports the speed of the place it serves (the *base* place). A
//! place derived from it by a small, smooth correction — the apparent place
//! (precession, nutation, annual aberration, light-time), or a frame rotation
//! from J2000 to the equinox of date — moves at a slightly different rate. The
//! corrected speed is the base speed plus the rate of that correction
//! (corrected minus base place).
//!
//! Differencing the correction rather than the corrected place itself keeps
//! the accuracy of the backend's own speed: the correction is small and
//! smooth, so its finite difference carries a truncation error far below the
//! one a difference of the full position would (the Moon's longitude speed
//! alone would be off by about 3e-3 deg/day over the same span).

use pleiades_types::{EclipticCoordinates, Motion};

/// Half-span of the correction difference, in days. Matches the span
/// `pleiades-data` and `pleiades-events` use for their differenced speeds.
pub const HALF_SPAN_DAYS: f64 = 0.5;

/// Corrected minus base place at one instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Correction {
    /// Longitude correction in degrees, wrapped to `[-180, 180)`.
    longitude_deg: f64,
    /// Latitude correction in degrees.
    latitude_deg: f64,
    /// Distance correction in astronomical units, when both places carry a distance.
    distance_au: Option<f64>,
}

impl Correction {
    /// The correction that takes `base` to `corrected`.
    pub fn between(corrected: &EclipticCoordinates, base: &EclipticCoordinates) -> Self {
        Self {
            longitude_deg: wrap_signed(corrected.longitude.degrees() - base.longitude.degrees()),
            latitude_deg: corrected.latitude.degrees() - base.latitude.degrees(),
            distance_au: corrected
                .distance_au
                .zip(base.distance_au)
                .map(|(corrected, base)| corrected - base),
        }
    }
}

/// A correction and the Julian day it was evaluated at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorrectionSample {
    /// Julian day of the sample.
    pub julian_day: f64,
    /// Corrected minus base place at that Julian day.
    pub correction: Correction,
}

/// Returns the speed of the corrected place: `base` plus the rate of the
/// correction between two samples. Channels `base` leaves empty stay empty,
/// and the distance speed is unchanged when a sample has no distance.
///
/// The two samples must be at different instants: when their span is zero or
/// not finite the speed is unknown and every channel is `None`.
pub fn apparent_motion(
    base: Motion,
    earlier: &CorrectionSample,
    later: &CorrectionSample,
) -> Motion {
    let span_days = later.julian_day - earlier.julian_day;
    if !span_days.is_finite() || span_days == 0.0 {
        return Motion::new(None, None, None);
    }
    let (from, to) = (&earlier.correction, &later.correction);
    let longitude_rate = wrap_signed(to.longitude_deg - from.longitude_deg) / span_days;
    let latitude_rate = (to.latitude_deg - from.latitude_deg) / span_days;
    let distance_rate = to
        .distance_au
        .zip(from.distance_au)
        .map_or(0.0, |(to, from)| (to - from) / span_days);
    Motion::new(
        base.longitude_deg_per_day
            .map(|speed| speed + longitude_rate),
        base.latitude_deg_per_day.map(|speed| speed + latitude_rate),
        base.distance_au_per_day.map(|speed| speed + distance_rate),
    )
}

/// Wraps an angle difference in degrees to `[-180, 180)`.
fn wrap_signed(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests;
