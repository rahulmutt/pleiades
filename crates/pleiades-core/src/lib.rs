//! High-level façade that combines backend queries with astrology-domain logic.
//!
//! Later stages layer in catalog, compatibility, and release-profile
//! information while keeping the façade intentionally thin. It still delegates
//! query execution to a backend, but it now also exposes the versioned
//! compatibility profile and API stability posture used by the CLI,
//! validation reports, and release notes.
//!
//! # Examples
//!
//! ```
//! use pleiades_core::{ChartEngine, EphemerisBackend, EphemerisRequest, EphemerisResult, BackendMetadata,
//!     BackendId, BackendFamily, BackendProvenance, BackendCapabilities, AccuracyClass, TimeRange,
//!     CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, EphemerisError,
//!     EphemerisErrorKind, current_api_stability_profile};
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
//!
//! let posture = current_api_stability_profile();
//! assert!(posture.summary.contains("stable consumer surface"));
//! ```

#![forbid(unsafe_code)]

mod api_stability;
mod chart;
mod compatibility;

pub use api_stability::{
    current_api_stability_profile, current_api_stability_profile_id, ApiStabilityProfile,
    CURRENT_API_STABILITY_PROFILE_ID,
};
pub use chart::{
    default_chart_bodies, sidereal_longitude, AspectDefinition, AspectKind, AspectMatch,
    BodyPlacement, ChartRequest, ChartSnapshot, HouseSummary, MotionSummary, SignSummary,
};
pub use compatibility::{
    current_compatibility_profile, current_compatibility_profile_id, CompatibilityProfile,
    CURRENT_COMPATIBILITY_PROFILE_ID,
};
pub use pleiades_ayanamsa::{
    baseline_ayanamsas, built_in_ayanamsas, descriptor as ayanamsa_descriptor, release_ayanamsas,
    resolve_ayanamsa, AyanamsaDescriptor,
};
pub use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, CompositeBackend, EphemerisBackend, EphemerisError, EphemerisErrorKind,
    EphemerisRequest, EphemerisResult, QualityAnnotation, RoutingBackend,
};
pub use pleiades_houses::{
    baseline_house_systems, calculate_houses, descriptor as house_system_descriptor,
    house_for_longitude, resolve_house_system, HouseAngles, HouseError, HouseErrorKind,
    HouseRequest, HouseSnapshot, HouseSystemDescriptor,
};
pub use pleiades_types::{
    Angle, Ayanamsa, CelestialBody, CoordinateFrame, CustomAyanamsa, CustomBodyId,
    CustomHouseSystem, EclipticCoordinates, EquatorialCoordinates, HouseSystem, Instant, JulianDay,
    Latitude, Longitude, Motion, MotionDirection, ObserverLocation, TimeRange, TimeScale,
    ZodiacMode, ZodiacSign,
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
            .baseline_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Topocentric"));
        assert!(profile
            .baseline_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Fagan/Bradley"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Equal (1=Aries)"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Sripati"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Horizon/Azimuth"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "APC"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Gauquelin sectors"));
    }

    #[test]
    fn release_notes_cover_release_catalog_entries() {
        let profile = current_compatibility_profile();
        let release_notes = profile.release_notes.join("\n");

        for entry in profile.release_house_systems {
            assert!(
                release_notes.contains(entry.canonical_name),
                "release notes should mention house-system {}",
                entry.canonical_name
            );
        }

        for entry in profile.release_ayanamsas {
            assert!(
                release_notes.contains(entry.canonical_name),
                "release notes should mention ayanamsa {}",
                entry.canonical_name
            );
        }
    }

    #[test]
    fn profile_identifiers_are_re_exported_from_the_facade() {
        assert_eq!(
            CURRENT_COMPATIBILITY_PROFILE_ID,
            current_compatibility_profile_id()
        );
        assert_eq!(
            CURRENT_API_STABILITY_PROFILE_ID,
            current_api_stability_profile_id()
        );
        assert_eq!(
            current_compatibility_profile_id(),
            current_compatibility_profile().profile_id
        );
        assert_eq!(
            current_api_stability_profile_id(),
            current_api_stability_profile().profile_id
        );
    }
}
