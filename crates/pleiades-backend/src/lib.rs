//! Backend contracts, metadata, and adapter helpers for ephemeris providers.
//!
//! This crate defines the shared request/response shape used by all backend
//! families. Concrete backends live in their own `pleiades-*` crates and
//! implement [`EphemerisBackend`].
//!
//! Enable the optional `serde` feature to serialize the shared request,
//! result, metadata, and error types used by backend implementations.
//!
//! The current time-scale, observer, apparentness, and frame policy is
//! documented in `docs/time-observer-policy.md` so the direct backend contract
//! and the façade-level request helpers stay in sync. Direct batch callers
//! should pair that policy with `validate_requests_against_metadata()` so the
//! same explicit contract is checked before a slice of requests is dispatched.
//!
//! # Examples
//!
//! ```
//! use pleiades_backend::{EphemerisBackend, EphemerisRequest, BackendMetadata, BackendId, BackendFamily,
//!     BackendProvenance, BackendCapabilities, AccuracyClass, TimeRange, EphemerisResult, EphemerisError,
//!     EphemerisErrorKind, Apparentness, QualityAnnotation, BodyClaim};
//! use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, Latitude,
//!     Longitude, TimeScale, ZodiacMode};
//!
//! struct ToyBackend;
//!
//! impl EphemerisBackend for ToyBackend {
//!     fn metadata(&self) -> BackendMetadata {
//!         BackendMetadata {
//!             id: BackendId::new("toy"),
//!             version: "0.1.0".to_string(),
//!             family: BackendFamily::Algorithmic,
//!             provenance: BackendProvenance { summary: "example backend".to_string(), data_sources: vec![] },
//!             nominal_range: TimeRange::new(None, None),
//!             supported_time_scales: vec![TimeScale::Tt],
//!             body_claims: vec![BodyClaim::from(CelestialBody::Sun)],
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
//!             return Err(EphemerisError::new(EphemerisErrorKind::UnsupportedBody, "only the Sun is supported"));
//!         }
//!
//!         Ok(EphemerisResult::new(
//!             BackendId::new("toy"),
//!             request.body.clone(),
//!             request.instant,
//!             request.frame,
//!             request.zodiac_mode.clone(),
//!             request.apparent,
//!         ))
//!     }
//! }
//! ```

#![forbid(unsafe_code)]

mod capabilities;
mod claims;
mod errors;
mod identity;
mod metadata;
mod policy;
mod release_posture;
mod request;
mod result;
mod traits;
mod validation;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(any(test, feature = "test-backend"))]
pub mod test_backend;

pub use pleiades_types::{
    Angle, Apparentness, Ayanamsa, CelestialBody, CelestialBodyClass, CoordinateFrame,
    CoordinateValidationError, CustomAyanamsa, CustomBodyId, CustomDefinitionValidationError,
    CustomHouseSystem, EclipticCoordinates, EquatorialCoordinates, HouseSystem, Instant, JulianDay,
    Latitude, Longitude, Motion, MotionValidationError, ObserverLocation, TimeRange,
    TimeRangeValidationError, TimeScale, TimeScaleConversion, TimeScaleConversionError, ZodiacMode,
};

pub use capabilities::{BackendCapabilities, BackendCapabilitiesValidationError};
pub use claims::{BodyClaim, BodyClaimTier, ClaimEvidence};
pub use errors::{EphemerisError, EphemerisErrorKind};
pub use identity::{AccuracyClass, BackendFamily, BackendFamilyPosture, BackendId};
pub use metadata::{
    merge_body_claims, BackendMetadata, BackendMetadataValidationError, BackendProvenance,
    BackendProvenanceValidationError,
};
pub use policy::current::{
    apparentness_policy_summary_for_report, current_apparentness_policy_summary,
    current_delta_t_policy_summary, current_frame_policy_summary,
    current_native_sidereal_policy_summary, current_observer_policy_summary,
    current_pluto_fallback_summary, current_request_policy_summary,
    current_time_scale_policy_summary, current_utc_convenience_policy_summary,
    current_zodiac_policy_summary, delta_t_policy_summary_for_report, frame_policy_summary_details,
    frame_policy_summary_for_report, frame_treatment_summary_for_report,
    native_sidereal_policy_summary_for_report, observer_policy_summary_for_report,
    pluto_fallback_summary_for_report, request_policy_summary_for_report,
    request_semantics_summary_for_report, time_scale_policy_summary_for_report,
    unsupported_modes_summary_for_report, utc_convenience_policy_summary_for_report,
    validate_observer_policy, validate_request_against_metadata, validate_request_policy,
    validate_requests_against_metadata, validate_zodiac_policy,
    validated_apparentness_policy_summary_for_report, validated_delta_t_policy_summary_for_report,
    validated_frame_policy_summary_for_report, validated_frame_treatment_summary_for_report,
    validated_native_sidereal_policy_summary_for_report,
    validated_observer_policy_summary_for_report, validated_pluto_fallback_summary_line_for_report,
    validated_request_policy_summary_for_report, validated_request_semantics_summary_for_report,
    validated_time_scale_policy_summary_for_report,
    validated_utc_convenience_policy_summary_for_report,
    validated_zodiac_policy_summary_for_report, zodiac_policy_summary_for_report,
};
pub use policy::{
    ApparentnessPolicySummary, ApparentnessPolicySummaryValidationError, DeltaTPolicySummary,
    DeltaTPolicySummaryValidationError, FramePolicySummary, FramePolicySummaryValidationError,
    FrameTreatmentSummary, FrameTreatmentSummaryValidationError, NativeSiderealPolicySummary,
    NativeSiderealPolicySummaryValidationError, ObserverPolicySummary,
    ObserverPolicySummaryValidationError, PlutoFallbackSummary,
    PlutoFallbackSummaryValidationError, RequestPolicySummary, RequestPolicySummaryValidationError,
    TimeScalePolicySummary, TimeScalePolicySummaryValidationError, UtcConveniencePolicySummary,
    UtcConveniencePolicySummaryValidationError, ZodiacPolicySummary,
    ZodiacPolicySummaryValidationError, CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT,
    CURRENT_DELTA_T_POLICY_SUMMARY_TEXT, CURRENT_FRAME_POLICY_SUMMARY_TEXT,
    CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT, CURRENT_OBSERVER_POLICY_SUMMARY_TEXT,
    CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT, CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT,
    CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT, CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT,
    CURRENT_ZODIAC_POLICY_SUMMARY_TEXT,
};
pub use release_posture::ReleasePosture;
pub use request::EphemerisRequest;
pub use result::{EphemerisResult, EphemerisResultValidationError, QualityAnnotation};
pub use traits::{CompositeBackend, EphemerisBackend, RoutingBackend};
