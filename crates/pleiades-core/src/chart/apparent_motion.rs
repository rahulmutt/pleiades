//! Speed of the apparent place for chart placements.
//!
//! A backend reports the speed of the mean place it serves. The apparent
//! place moves at a slightly different rate, because precession, nutation,
//! annual aberration and the light-time displacement all change with time.
//! The apparent speed is the backend's mean speed plus the rate of that
//! correction (apparent minus mean place).
//!
//! Differencing the correction rather than the apparent place itself keeps
//! the accuracy of the backend's own speed: the correction is small and
//! smooth, so its finite difference carries a truncation error far below the
//! one a difference of the full position would (the Moon's longitude speed
//! alone would be off by about 3e-3 deg/day over the same span).

use pleiades_types::{EclipticCoordinates, Motion};

/// Half-span of the correction difference, in days. Matches the span
/// `pleiades-data` and `pleiades-events` use for their differenced speeds.
pub(super) const HALF_SPAN_DAYS: f64 = 0.5;

/// Apparent minus mean place at one instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Correction {
    /// Longitude correction in degrees, wrapped to `[-180, 180)`.
    longitude_deg: f64,
    /// Latitude correction in degrees.
    latitude_deg: f64,
    /// Distance correction in astronomical units, when both places carry a distance.
    distance_au: Option<f64>,
}

impl Correction {
    /// The correction that takes `mean` to `apparent`.
    pub(super) fn between(apparent: &EclipticCoordinates, mean: &EclipticCoordinates) -> Self {
        Self {
            longitude_deg: wrap_signed(apparent.longitude.degrees() - mean.longitude.degrees()),
            latitude_deg: apparent.latitude.degrees() - mean.latitude.degrees(),
            distance_au: apparent
                .distance_au
                .zip(mean.distance_au)
                .map(|(apparent, mean)| apparent - mean),
        }
    }
}

/// A correction and the Julian day it was evaluated at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CorrectionSample {
    /// Julian day of the sample.
    pub(super) julian_day: f64,
    /// Apparent minus mean place at that Julian day.
    pub(super) correction: Correction,
}

/// Returns the speed of the apparent place: `mean` plus the rate of the
/// correction between two samples. Channels the backend left empty stay
/// empty, and the distance speed is unchanged when a sample has no distance.
pub(super) fn apparent_motion(
    mean: Motion,
    earlier: &CorrectionSample,
    later: &CorrectionSample,
) -> Motion {
    let span_days = later.julian_day - earlier.julian_day;
    let (from, to) = (&earlier.correction, &later.correction);
    let longitude_rate = wrap_signed(to.longitude_deg - from.longitude_deg) / span_days;
    let latitude_rate = (to.latitude_deg - from.latitude_deg) / span_days;
    let distance_rate = to
        .distance_au
        .zip(from.distance_au)
        .map_or(0.0, |(to, from)| (to - from) / span_days);
    Motion::new(
        mean.longitude_deg_per_day
            .map(|speed| speed + longitude_rate),
        mean.latitude_deg_per_day.map(|speed| speed + latitude_rate),
        mean.distance_au_per_day.map(|speed| speed + distance_rate),
    )
}

/// Wraps an angle difference in degrees to `[-180, 180)`.
fn wrap_signed(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests;
