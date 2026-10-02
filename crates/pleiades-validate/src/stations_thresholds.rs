//! Measured-basis ceilings for the `validate-stations` gate.

// Consumed by the gate entry point added next.
#![allow(dead_code)]

/// Ceilings for one body: time between the engine's station and the
/// reference's, and the longitude difference at the station.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    pub(crate) time_s: f64,
    pub(crate) lon_arcsec: f64,
}

/// A true-node station is "separated" when its nearest neighbouring station
/// in its own list is at least this many days away. Closer pairs are grazes
/// of the speed against zero whose existence depends on the ephemeris, and
/// are not compared.
pub(crate) const SEPARATION_DAYS: f64 = 2.0;

/// Fail-closed floor on compared stations. Set in Task 5 from the measured
/// count.
pub(crate) const MIN_ROWS_VALIDATED: usize = 0;

/// Not yet measured: Task 5 replaces every value from the gate's own output.
const UNMEASURED: Ceilings = Ceilings {
    time_s: f64::INFINITY,
    lon_arcsec: f64::INFINITY,
};

/// Ceilings by corpus body name, or `None` for a body the gate does not cover.
pub(crate) fn ceilings_for(body_name: &str) -> Option<Ceilings> {
    match body_name {
        "Mercury" | "Venus" | "Mars" | "Jupiter" | "Saturn" | "Uranus" | "Neptune" | "Pluto"
        | "TrueNode" => Some(UNMEASURED),
        _ => None,
    }
}
