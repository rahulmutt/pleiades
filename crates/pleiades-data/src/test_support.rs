//! Shared test setup helpers for the `pleiades-data` test suite.
//!
//! Builders and fixtures extracted from repeated arrange blocks in tests so
//! every test file can import them with `use crate::test_support::*;`.

use crate::*;

/// Build a TT-scale [`Instant`] from a Julian-day number.
pub(crate) fn instant_tt(days: f64) -> Instant {
    Instant::new(JulianDay::from_days(days), TimeScale::Tt)
}
