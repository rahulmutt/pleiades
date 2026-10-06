//! Shared helpers for the window-edge regression tests (issue #208).

use crate::error::EventError;
use core::fmt::Debug;
use pleiades_types::{Instant, JulianDay, TimeScale};

/// A TDB instant.
pub(crate) fn tdb(julian_day: f64) -> Instant {
    Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb)
}

/// The Julian day an `OutOfWindow` error names; panics on anything else.
pub(crate) fn out_of_window_jd<T: Debug>(result: Result<T, EventError>) -> f64 {
    match result {
        Err(EventError::OutOfWindow { julian_day }) => julian_day,
        other => panic!("expected OutOfWindow, got {other:?}"),
    }
}
