use crate::systems::*;
use pleiades_types::{JulianDay, Latitude, TimeScale};

pub(super) fn observer() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(0.0),
        Longitude::from_degrees(0.0),
        None,
    )
}

pub(super) fn sample_request(system: HouseSystem) -> HouseRequest {
    HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        observer(),
        system,
    )
}

pub(super) fn assert_close_degrees(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "expected {expected}, got {actual}"
    );
}

/// Builds a `HouseAngles` from an ascendant/midheaven pair with their
/// opposites derived automatically. Shared across the great-circle, sector,
/// and sunshine family test files, which all construct geometries this way.
pub(super) fn gc_angles(asc: f64, mc: f64) -> HouseAngles {
    HouseAngles {
        ascendant: Longitude::from_degrees(asc),
        descendant: Longitude::from_degrees(asc + 180.0),
        midheaven: Longitude::from_degrees(mc),
        imum_coeli: Longitude::from_degrees(mc + 180.0),
    }
}

pub(super) fn test_asc_mc(angles: HouseAngles) -> AscMc {
    AscMc {
        ascendant: angles.ascendant,
        midheaven: angles.midheaven,
        descendant: angles.descendant,
        imum_coeli: angles.imum_coeli,
        armc: angles.midheaven,
        vertex: angles.ascendant,
        antivertex: angles.descendant,
        equatorial_ascendant: angles.ascendant,
        coascendant_koch: angles.ascendant,
        coascendant_munkasey: angles.ascendant,
        polar_ascendant: angles.descendant,
    }
}
