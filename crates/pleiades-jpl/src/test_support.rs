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
