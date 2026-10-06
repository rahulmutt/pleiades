//! Shared test setup for the crate's co-located unit tests.
//!
//! Builders and constructors reused across the relocated test suites live
//! here so individual test modules can share arrange code instead of
//! repeating it.

#![allow(unused_imports)]

use pleiades_backend::{
    Apparentness, CelestialBody, EphemerisBackend, EphemerisRequest, QualityAnnotation,
};
use pleiades_types::{CoordinateFrame, Instant, JulianDay, TimeScale};

use crate::JplSnapshotBackend;

/// Constructs the checked-in JPL snapshot backend used across the suite.
pub(crate) fn backend() -> JplSnapshotBackend {
    JplSnapshotBackend::new()
}

/// A mean, tropical, geocentric ecliptic request at a TDB Julian day.
pub(crate) fn mean_request(body: CelestialBody, julian_day: f64) -> EphemerisRequest {
    EphemerisRequest {
        body,
        instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: pleiades_types::ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    }
}
