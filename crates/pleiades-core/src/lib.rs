//! High-level façade that combines backend queries with astrology-domain logic.
//!
//! Stage 3 starts layering in catalog and compatibility information while
//! keeping the façade intentionally thin. It still delegates query execution to
//! a backend, but it now also exposes the versioned compatibility profile used
//! by the CLI and release notes.
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

mod chart;
mod compatibility;

pub use chart::{
    default_chart_bodies, sidereal_longitude, BodyPlacement, ChartRequest, ChartSnapshot,
};
pub use compatibility::{current_compatibility_profile, CompatibilityProfile};
pub use pleiades_ayanamsa::{
    baseline_ayanamsas, descriptor as ayanamsa_descriptor, resolve_ayanamsa, AyanamsaDescriptor,
};
pub use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, CompositeBackend, EphemerisBackend, EphemerisError, EphemerisErrorKind,
    EphemerisRequest, EphemerisResult, QualityAnnotation,
};
pub use pleiades_houses::{
    baseline_house_systems, descriptor as house_system_descriptor, resolve_house_system,
    HouseSystemDescriptor,
};
pub use pleiades_types::{
    Angle, Ayanamsa, CelestialBody, CoordinateFrame, CustomAyanamsa, CustomBodyId,
    CustomHouseSystem, EclipticCoordinates, EquatorialCoordinates, HouseSystem, Instant, JulianDay,
    Latitude, Longitude, Motion, ObserverLocation, TimeRange, TimeScale, ZodiacMode, ZodiacSign,
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

    #[test]
    fn compatibility_profile_surfaces_current_baseline() {
        let profile = current_compatibility_profile();
        assert!(profile
            .house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Topocentric"));
        assert!(profile
            .ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Fagan/Bradley"));
    }
}
