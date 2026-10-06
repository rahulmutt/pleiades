//! Shared helpers for the window-edge regression tests (issue #208).

use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use core::fmt::Debug;
use pleiades_backend::{
    AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, CoordinateFrame, EphemerisBackend, EphemerisError,
    EphemerisRequest, EphemerisResult,
};
use pleiades_types::{
    CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, Motion, TimeRange,
    TimeScale,
};

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

/// Mars turns retrograde here, one day after the window's start...
pub(crate) const FIRST_STATION_JD: f64 = WINDOW_START_JD + 1.0;
/// ...and direct here, one day before its end. Mars's station step is 2 days.
pub(crate) const LAST_STATION_JD: f64 = WINDOW_END_JD - 1.0;
/// Speed slope at each station, in degrees per day per day: large enough
/// that precession to the mean equinox of date (about 3.8e-5 deg/day) moves
/// each station by about 3 s.
const SLOPE: f64 = 1.0;

/// Serves Mars on a J2000 longitude whose speed is
/// `k * (t - FIRST) * (t - LAST)`, positive before the first station,
/// negative between, positive after.
pub(crate) struct StationingMars;

impl StationingMars {
    fn k() -> f64 {
        SLOPE / (LAST_STATION_JD - FIRST_STATION_JD)
    }
    /// Longitude (degrees) and speed (degrees per day) at `jd`.
    fn longitude_and_speed(jd: f64) -> (f64, f64) {
        let u = jd - FIRST_STATION_JD;
        let span = LAST_STATION_JD - FIRST_STATION_JD;
        let k = Self::k();
        let longitude = 100.0 + k * (u.powi(3) / 3.0 - span * u.powi(2) / 2.0);
        let speed = k * u * (u - span);
        (longitude, speed)
    }
}

impl EphemerisBackend for StationingMars {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("stationing-mars"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("test backend with stations at the window's ends"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tdb],
            body_claims: vec![BodyClaim::from(CelestialBody::Mars)],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }
    fn supports_body(&self, body: CelestialBody) -> bool {
        body == CelestialBody::Mars
    }
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = EphemerisResult::new(
            BackendId::new("stationing-mars"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        if req.body == CelestialBody::Mars {
            let (longitude, speed) = Self::longitude_and_speed(req.instant.julian_day.days());
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(longitude),
                Latitude::from_degrees(0.0),
                Some(1.5),
            ));
            result.motion = Some(Motion::new(Some(speed), Some(0.0), Some(0.0)));
        }
        Ok(result)
    }
}
