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
//!     CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, TimeScaleConversion,
//!     EphemerisError, EphemerisErrorKind, current_api_stability_profile};
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
//! let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
//! assert_eq!(policy.summary_line(), "source=UT1; target=TT; offset_seconds=64.184 s");
//!
//! let request = EphemerisRequest::new(
//!     CelestialBody::Sun,
//!     Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
//! );
//! let result = engine.position(&request).expect("demo backend should succeed");
//! assert_eq!(result.backend_id.as_str(), "demo");
//!
//! let metadata = engine.validated_metadata().expect("demo metadata should be valid");
//! assert_eq!(metadata.id.as_str(), "demo");
//!
//! let posture = current_api_stability_profile();
//! assert!(posture.summary.contains("stable consumer surface"));
//! ```

#![forbid(unsafe_code)]

mod api_stability;
mod chart;
mod compatibility;
mod release_profiles;

pub use api_stability::{
    current_api_stability_profile, current_api_stability_profile_id, ApiStabilityProfile,
    ApiStabilityProfileValidationError, CURRENT_API_STABILITY_PROFILE_ID,
};
pub use chart::{
    default_chart_bodies, sidereal_longitude, AspectDefinition, AspectKind, AspectMatch,
    BodyPlacement, ChartRequest, ChartSnapshot, HouseSummary, MotionSummary, ObserverPolicy,
    SignSummary,
};
pub use compatibility::{
    current_compatibility_profile, current_compatibility_profile_id,
    validate_custom_definition_labels, CompatibilityProfile, CURRENT_COMPATIBILITY_PROFILE_ID,
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
    Latitude, Longitude, Motion, MotionDirection, ObserverLocation, TimeRange,
    TimeRangeValidationError, TimeScale, TimeScaleConversion, TimeScaleConversionError, ZodiacMode,
    ZodiacSign, SECONDS_PER_DAY,
};
pub use release_profiles::{
    current_release_profile_identifiers, ReleaseProfileIdentifiers,
    ReleaseProfileIdentifiersValidationError,
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
    ///
    /// Call [`validated_metadata`](Self::validated_metadata) when you need the
    /// shared consistency checks that confirm the metadata is not advertising
    /// blank identifiers, duplicate catalog entries, or an invalid nominal
    /// range.
    pub fn metadata(&self) -> BackendMetadata {
        self.backend.metadata()
    }

    /// Returns backend metadata after applying the shared consistency checks.
    pub fn validated_metadata(
        &self,
    ) -> Result<BackendMetadata, pleiades_backend::BackendMetadataValidationError> {
        let metadata = self.backend.metadata();
        metadata.validate()?;
        Ok(metadata)
    }

    /// Validates a chart request against the backend metadata without querying positions.
    ///
    /// This is the façade-level counterpart to
    /// [`ChartRequest::validate_against_metadata`], which lets callers preflight
    /// chart shape, house-observer policy, zodiac routing, frame support, and
    /// body coverage directly from the engine when they want to separate
    /// validation from chart assembly. The backend metadata is validated first so
    /// the request preflight fails closed if the backend inventory itself drifts
    /// out of consistency.
    pub fn validate_chart_request(&self, request: &ChartRequest) -> Result<(), EphemerisError> {
        let metadata = self.validated_metadata().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("backend metadata failed validation: {error}"),
            )
        })?;

        request.validate_against_metadata(&metadata)
    }

    /// Validates a batch of chart requests against the backend metadata without querying positions.
    ///
    /// This convenience mirrors [`validate_chart_request`](Self::validate_chart_request) while
    /// preserving the first failing request's 1-based index in the returned error message. It does
    /// not normalize request instants, so a supported batch may legitimately mix TT and TDB chart
    /// requests when the backend metadata allows both scales. It is useful for callers that want to
    /// preflight a chart corpus before dispatching a sequence of chart assemblies or validation
    /// runs.
    pub fn validate_chart_requests(&self, requests: &[ChartRequest]) -> Result<(), EphemerisError> {
        let metadata = self.validated_metadata().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("backend metadata failed validation: {error}"),
            )
        })?;

        for (index, request) in requests.iter().enumerate() {
            request
                .validate_against_metadata(&metadata)
                .map_err(|error| {
                    EphemerisError::new(
                        error.kind,
                        format!(
                            "chart request #{} failed validation: {}",
                            index + 1,
                            error.message
                        ),
                    )
                })?;
        }

        Ok(())
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

    struct RestrictedPolicyBackend;

    impl EphemerisBackend for RestrictedPolicyBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("restricted"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("restricted policy test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    apparent: false,
                    ..BackendCapabilities::default()
                },
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
                BackendId::new("restricted"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            ))
        }
    }

    struct MixedScaleBackend;

    impl EphemerisBackend for MixedScaleBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("mixed-scale"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("mixed-scale test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
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

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("mixed-scale"),
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
        assert_eq!(engine.validated_metadata().unwrap().id.as_str(), "simple");
        assert_eq!(
            engine.position(&request).unwrap().backend_id.as_str(),
            "simple"
        );
    }

    #[test]
    fn validate_chart_request_reuses_backend_metadata_guardrails() {
        let engine = ChartEngine::new(SimpleBackend);
        let instant = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);

        let supported = ChartRequest::new(instant).with_bodies(vec![CelestialBody::Sun]);
        assert!(engine.validate_chart_request(&supported).is_ok());

        let unsupported = ChartRequest::new(instant);
        let error = engine
            .validate_chart_request(&unsupported)
            .expect_err("chart request should reject unsupported bodies before assembly");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        assert!(error.message.contains("simple does not support Moon"));

        let house_request =
            ChartRequest::new(instant).with_house_system(crate::HouseSystem::WholeSign);
        let error = engine
            .validate_chart_request(&house_request)
            .expect_err("house requests should require an observer location");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            error.message,
            "house placement requires an observer location"
        );
    }

    #[test]
    fn validate_chart_requests_prefixes_batch_failures() {
        let engine = ChartEngine::new(SimpleBackend);
        let instant = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let requests = [
            ChartRequest::new(instant).with_bodies(vec![CelestialBody::Sun]),
            ChartRequest::new(instant).with_house_system(crate::HouseSystem::WholeSign),
        ];

        let error = engine
            .validate_chart_requests(&requests)
            .expect_err("batch chart validation should reject missing observers");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            error.message,
            "chart request #2 failed validation: house placement requires an observer location"
        );
    }

    #[test]
    fn validate_chart_requests_prefixes_unsupported_time_scale_failures() {
        let engine = ChartEngine::new(RestrictedPolicyBackend);
        let tt = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let utc = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc);
        let requests = [
            ChartRequest::new(tt).with_bodies(vec![CelestialBody::Sun]),
            ChartRequest::new(utc).with_bodies(vec![CelestialBody::Sun]),
        ];

        let error = engine
            .validate_chart_requests(&requests)
            .expect_err("batch chart validation should reject unsupported time scales");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
        assert_eq!(
            error.message,
            "chart request #2 failed validation: restricted expects one of [TT] for request instants"
        );
    }

    #[test]
    fn validate_chart_requests_prefixes_apparentness_failures() {
        let engine = ChartEngine::new(RestrictedPolicyBackend);
        let instant = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let requests = [
            ChartRequest::new(instant).with_bodies(vec![CelestialBody::Sun]),
            ChartRequest::new(instant)
                .with_bodies(vec![CelestialBody::Sun])
                .with_apparentness(Apparentness::Apparent),
        ];

        let error = engine
            .validate_chart_requests(&requests)
            .expect_err("batch chart validation should reject unsupported apparentness");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            error.message,
            "chart request #2 failed validation: restricted currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );
    }

    #[test]
    fn validate_chart_requests_prefixes_topocentric_house_failures() {
        let engine = ChartEngine::new(SimpleBackend);
        let instant = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let requests = [
            ChartRequest::new(instant).with_bodies(vec![CelestialBody::Sun]),
            ChartRequest::new(instant).with_house_system(crate::HouseSystem::Topocentric),
        ];

        let error = engine
            .validate_chart_requests(&requests)
            .expect_err("batch chart validation should reject missing topocentric observers");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            error.message,
            "chart request #2 failed validation: house placement requires an observer location"
        );
    }

    #[test]
    fn validate_chart_requests_preserves_mixed_supported_time_scales() {
        let engine = ChartEngine::new(MixedScaleBackend);
        let tt = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let tdb = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb);
        let requests = [
            ChartRequest::new(tt).with_bodies(vec![CelestialBody::Sun]),
            ChartRequest::new(tdb).with_bodies(vec![CelestialBody::Moon]),
        ];

        engine
            .validate_chart_requests(&requests)
            .expect("batch chart validation should accept mixed TT/TDB requests when supported");
        assert_eq!(requests[0].instant.scale, TimeScale::Tt);
        assert_eq!(requests[1].instant.scale, TimeScale::Tdb);
    }

    #[test]
    fn validated_metadata_rejects_duplicate_body_coverage() {
        struct InvalidMetadataBackend;

        impl EphemerisBackend for InvalidMetadataBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("invalid-metadata"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("invalid metadata test backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt],
                    body_coverage: vec![CelestialBody::Sun, CelestialBody::Sun],
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

            fn position(
                &self,
                request: &EphemerisRequest,
            ) -> Result<EphemerisResult, EphemerisError> {
                Ok(EphemerisResult::new(
                    BackendId::new("invalid-metadata"),
                    request.body.clone(),
                    request.instant,
                    request.frame,
                    request.zodiac_mode.clone(),
                    request.apparent,
                ))
            }
        }

        let engine = ChartEngine::new(InvalidMetadataBackend);
        let error = engine
            .validated_metadata()
            .expect_err("duplicate metadata coverage should be rejected");

        assert!(matches!(
            error,
            pleiades_backend::BackendMetadataValidationError::DuplicateEntry {
                field: "body coverage",
                ref value,
            } if value == "Sun"
        ));
        let error_text = error.to_string();
        assert!(error_text
            .contains("backend metadata field `body coverage` contains duplicate entry `Sun`"));

        let chart_request =
            ChartRequest::new(Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt))
                .with_bodies(vec![CelestialBody::Sun]);

        let validation_error = engine
            .validate_chart_request(&chart_request)
            .expect_err("invalid backend metadata should reject chart preflights");
        assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(validation_error
            .message
            .contains("backend metadata failed validation: backend metadata field `body coverage` contains duplicate entry `Sun`"));

        let chart_error = engine
            .chart(&chart_request)
            .expect_err("invalid backend metadata should reject chart assembly");
        assert_eq!(chart_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(chart_error
            .message
            .contains("backend metadata failed validation: backend metadata field `body coverage` contains duplicate entry `Sun`"));
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

        assert!(
            release_notes.contains("selected asteroid coverage"),
            "release notes should mention selected asteroid coverage"
        );
        assert!(
            release_notes.contains("asteroid:433-Eros"),
            "release notes should mention the source-backed custom asteroid"
        );
        assert!(release_notes.contains("Equal (MC) house system"));
        assert!(release_notes.contains("Equal (1=Aries) house system"));
    }

    #[test]
    fn profile_summary_spells_out_the_true_galactic_equator_batch() {
        let profile = current_compatibility_profile();

        assert!(profile.summary.contains("Galactic Equator (True)"));
        assert!(profile.summary.contains("True galactic equator"));
        assert!(profile.summary.contains("Galactic equator true"));
    }

    #[test]
    fn profile_identifiers_are_re_exported_from_the_facade() {
        let release_profiles = current_release_profile_identifiers();

        assert_eq!(
            CURRENT_COMPATIBILITY_PROFILE_ID,
            current_compatibility_profile_id()
        );
        assert_eq!(
            CURRENT_API_STABILITY_PROFILE_ID,
            current_api_stability_profile_id()
        );
        assert_eq!(
            release_profiles.compatibility_profile_id,
            current_compatibility_profile_id()
        );
        assert_eq!(
            release_profiles.api_stability_profile_id,
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

    #[test]
    fn time_scale_conversion_is_re_exported_from_the_facade() {
        let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);

        assert_eq!(
            policy.summary_line(),
            "source=UT1; target=TT; offset_seconds=64.184 s"
        );
        assert_eq!(policy.to_string(), policy.summary_line());
    }
}
