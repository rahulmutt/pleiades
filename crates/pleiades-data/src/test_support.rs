//! Shared test setup helpers for the `pleiades-data` test suite.
//!
//! Builders and fixtures extracted from repeated arrange blocks in tests so
//! every test file can import them with `use crate::test_support::*;`.

use crate::*;

/// Build an [`EclipticCoordinates`] from plain degree/AU values.
///
/// Shorthand for the verbose four-line construction sites scattered throughout
/// the split-fraction and segment tests.
pub(crate) fn ecl(lon_deg: f64, lat_deg: f64, dist_au: f64) -> EclipticCoordinates {
    EclipticCoordinates::new(
        pleiades_backend::Longitude::from_degrees(lon_deg),
        pleiades_backend::Latitude::from_degrees(lat_deg),
        Some(dist_au),
    )
}

/// Build a TT-scale [`Instant`] from a Julian-day number.
pub(crate) fn instant_tt(days: f64) -> Instant {
    Instant::new(JulianDay::from_days(days), TimeScale::Tt)
}

/// Build a unit-span [`Segment`] (JD 0 → 1, single longitude channel) that
/// serves as the "current segment" baseline in the `moon_residual_search_*`
/// tests.
pub(crate) fn unit_segment() -> Segment {
    Segment::new(
        instant_tt(0.0),
        instant_tt(1.0),
        vec![PolynomialChannel::new(ChannelKind::Longitude, 0, vec![0.0])],
    )
}

/// Build the "current error" baseline used in `moon_residual_search_*` tests:
/// a [`PackagedArtifactSegmentFitError`] with every channel error at `10.0`.
pub(crate) fn baseline_fit_error() -> PackagedArtifactSegmentFitError {
    PackagedArtifactSegmentFitError {
        longitude_degrees: 10.0,
        latitude_degrees: 10.0,
        distance_au: 10.0,
    }
}
