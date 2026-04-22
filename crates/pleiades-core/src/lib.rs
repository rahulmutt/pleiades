//! High-level façade that combines backend queries with astrology-domain logic.
//!
//! This crate stays intentionally thin in stage 2: it re-exports the stable
//! shared types and provides a small backend wrapper that keeps orchestration in
//! one place without hiding the lower-level contracts.
//!
//! # Examples
//!
//! ```
//! use pleiades_core::{ChartEngine, EphemerisBackend, EphemerisRequest, EphemerisResult, BackendMetadata,
//!     BackendId, BackendFamily, BackendProvenance, BackendCapabilities, AccuracyClass, TimeRange,
//!     CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, EphemerisError,
//!     EphemerisErrorKind};
//!
//! struct DemoBackend;
//!
//! impl EphemerisBackend for DemoBackend {
//!     fn metadata(&self) -> BackendMetadata {
//!         BackendMetadata {
//!             id: BackendId::new("demo"),
//!             version: "0.1.0".to_string(),
//!             family: BackendFamily::Algorithmic,
//!             provenance: BackendProvenance::new("demo backend"),
//!             nominal_range: TimeRange::new(None, None),
//!             supported_time_scales: vec![TimeScale::Tt],
//!             body_coverage: vec![CelestialBody::Sun],
//!             supported_frames: vec![CoordinateFrame::Ecliptic],
//!             capabilities: BackendCapabilities::default(),
//!             accuracy: AccuracyClass::Approximate,
//!             deterministic: true,
//!             offline: true,
//!         }
//!     }
//!
//!     fn supports_body(&self, body: CelestialBody) -> bool {
//!         body == CelestialBody::Sun
//!     }
//!
//!     fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
//!         if request.body != CelestialBody::Sun {
//!             return Err(EphemerisError::new(EphemerisErrorKind::UnsupportedBody, "unsupported body"));
//!         }
//!
//!         Ok(EphemerisResult::new(
//!             BackendId::new("demo"),
//!             request.body.clone(),
//!             request.instant,
//!             request.frame,
//!             request.zodiac_mode.clone(),
//!             request.apparent,
//!         ))
//!     }
//! }
//!
//! let engine = ChartEngine::new(DemoBackend);
//! let request = EphemerisRequest::new(
//!     CelestialBody::Sun,
//!     Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
//! );
//! let result = engine.position(&request).expect("demo backend should succeed");
//! assert_eq!(result.backend_id.as_str(), "demo");
//! ```

#![forbid(unsafe_code)]

pub use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, QualityAnnotation,
};
pub use pleiades_types::{
    Angle, Ayanamsa, CelestialBody, CoordinateFrame, CustomAyanamsa, CustomBodyId,
    CustomHouseSystem, EclipticCoordinates, EquatorialCoordinates, HouseSystem, Instant, JulianDay,
    Latitude, Longitude, Motion, ObserverLocation, TimeRange, TimeScale, ZodiacMode,
};

/// A thin façade around a backend implementation.
#[derive(Debug)]
pub struct ChartEngine<B> {
    backend: B,
}

impl<B> ChartEngine<B> {
    /// Creates a new chart engine around the given backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Returns a shared reference to the underlying backend.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Returns the wrapped backend, consuming the façade.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

impl<B: EphemerisBackend> ChartEngine<B> {
    /// Returns the backend metadata.
    pub fn metadata(&self) -> BackendMetadata {
        self.backend.metadata()
    }

    /// Returns whether the backend supports a body.
    pub fn supports_body(&self, body: CelestialBody) -> bool {
        self.backend.supports_body(body)
    }

    /// Queries a single body position.
    pub fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.backend.position(request)
    }

    /// Queries multiple body positions.
    pub fn positions(
        &self,
        requests: &[EphemerisRequest],
    ) -> Result<Vec<EphemerisResult>, EphemerisError> {
        self.backend.positions(requests)
    }
}

impl<B> From<B> for ChartEngine<B> {
    fn from(backend: B) -> Self {
        Self::new(backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleBackend;

    impl EphemerisBackend for SimpleBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("simple"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("simple test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("simple"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            ))
        }
    }

    #[test]
    fn chart_engine_delegates_to_backend() {
        let engine = ChartEngine::new(SimpleBackend);
        let request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );

        assert!(engine.supports_body(CelestialBody::Sun));
        assert_eq!(engine.metadata().id.as_str(), "simple");
        assert_eq!(
            engine.position(&request).unwrap().backend_id.as_str(),
            "simple"
        );
    }
}
