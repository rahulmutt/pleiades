//! Deterministic analytic Sun/Moon backend for downstream tests.
//! NOT for production: circular-orbit longitudes, fixed distances.

use crate::capabilities::BackendCapabilities;
use crate::claims::BodyClaim;
use crate::errors::{EphemerisError, EphemerisErrorKind};
use crate::identity::{AccuracyClass, BackendFamily, BackendId};
use crate::metadata::{BackendMetadata, BackendProvenance};
use crate::request::EphemerisRequest;
use crate::result::EphemerisResult;
use crate::traits::EphemerisBackend;
use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, Latitude, Longitude, TimeRange, TimeScale,
};

/// Sun and Moon move at constant ecliptic rates; both at `ref_longitude` at `jd0`.
#[derive(Clone, Copy, Debug)]
pub struct LinearSunMoon {
    jd0: f64,
    ref_longitude_deg: f64,
    sun_rate_deg_per_day: f64,
    moon_rate_deg_per_day: f64,
    moon_latitude_deg: f64,
    produce_coordinates: bool,
}

impl LinearSunMoon {
    /// New moon (Sun==Moon longitude, Moon on the ecliptic) at `jd0`.
    pub fn new_moon_at(jd0: f64) -> Self {
        Self {
            jd0,
            ref_longitude_deg: 100.0,
            sun_rate_deg_per_day: 0.985_647,
            moon_rate_deg_per_day: 13.176_396,
            moon_latitude_deg: 0.0,
            produce_coordinates: true,
        }
    }

    /// Backend that returns no ecliptic coordinates (drives the fail-closed path).
    pub fn empty() -> Self {
        let mut s = Self::new_moon_at(2_451_550.0);
        s.produce_coordinates = false;
        s
    }

    /// Returns a copy with the Moon's (constant) ecliptic latitude set to `degrees`.
    pub fn with_moon_latitude(mut self, degrees: f64) -> Self {
        self.moon_latitude_deg = degrees;
        self
    }
}

impl EphemerisBackend for LinearSunMoon {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("linear-sun-moon"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("deterministic analytic test backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tdb],
            body_claims: vec![
                BodyClaim::from(CelestialBody::Sun),
                BodyClaim::from(CelestialBody::Moon),
            ],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Sun | CelestialBody::Moon)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let backend_id = BackendId::new("linear-sun-moon");

        if !self.produce_coordinates {
            return Ok(EphemerisResult::new(
                backend_id,
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ));
        }

        let dt = req.instant.julian_day.days() - self.jd0;
        let (rate, latitude, distance_au) = match req.body {
            CelestialBody::Sun => (self.sun_rate_deg_per_day, 0.0_f64, 1.000_0_f64),
            CelestialBody::Moon => (
                self.moon_rate_deg_per_day,
                self.moon_latitude_deg,
                0.002_57_f64,
            ),
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    format!("linear-sun-moon does not support {}", req.body),
                ))
            }
        };

        let lon = Longitude::from_degrees(self.ref_longitude_deg + rate * dt);
        let coords =
            EclipticCoordinates::new(lon, Latitude::from_degrees(latitude), Some(distance_au));

        let mut result = EphemerisResult::new(
            backend_id,
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(coords);
        Ok(result)
    }
}
