//! Reference-snapshot accessors: canonical instant, covered bodies, epochs, and
//! the parsed reference fixture entries.

use pleiades_types::{Instant, JulianDay, TimeScale};

use crate::*;

/// Canonical JPL Horizons snapshot instant used by the reference backend.
pub const fn reference_instant() -> Instant {
    Instant::new(JulianDay::from_days(REFERENCE_EPOCH_JD), TimeScale::Tdb)
}

/// The narrow body set covered by the checked-in reference snapshot.
pub fn reference_bodies() -> &'static [pleiades_backend::CelestialBody] {
    snapshot_bodies()
}

/// The instants covered by the checked-in reference snapshot.
pub fn reference_epochs() -> &'static [Instant] {
    snapshot_instants()
}

/// Returns the parsed reference fixture entries.
pub fn reference_snapshot() -> &'static [SnapshotEntry] {
    snapshot_entries().unwrap_or(&[])
}
