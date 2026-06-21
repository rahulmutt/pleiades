use super::apparentness::ApparentnessPolicySummary;
use super::delta_t::DeltaTPolicySummary;
use super::frame::{FramePolicySummary, FrameTreatmentSummary};
use super::native_sidereal::NativeSiderealPolicySummary;
use super::observer::ObserverPolicySummary;
use super::pluto_fallback::{PlutoFallbackSummary, PlutoFallbackSummaryValidationError};
use super::request::RequestPolicySummary;
use super::time_scale::TimeScalePolicySummary;
use super::utc::UtcConveniencePolicySummary;
use super::zodiac::ZodiacPolicySummary;
use super::{
    CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT, CURRENT_DELTA_T_POLICY_SUMMARY_TEXT,
    CURRENT_FRAME_POLICY_SUMMARY_TEXT, CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT,
    CURRENT_OBSERVER_POLICY_SUMMARY_TEXT, CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT,
    CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT, CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT,
    CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT, CURRENT_ZODIAC_POLICY_SUMMARY_TEXT,
};
use crate::errors::{format_display_list, EphemerisError, EphemerisErrorKind};
use crate::metadata::BackendMetadata;
use crate::request::EphemerisRequest;
use pleiades_types::{Apparentness, CoordinateFrame, TimeScale, ZodiacMode};

/// Returns the current unsupported-modes posture used by validation and release reporting.
pub const fn unsupported_modes_summary_for_report() -> &'static str {
    CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT
}

/// Returns the current shared time-scale policy used by validation and reports.
pub const fn current_time_scale_policy_summary() -> TimeScalePolicySummary {
    TimeScalePolicySummary::new(CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared Delta T policy used by validation and reports.
pub const fn current_delta_t_policy_summary() -> DeltaTPolicySummary {
    DeltaTPolicySummary::new(CURRENT_DELTA_T_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared UTC-convenience policy used by validation and reports.
pub const fn current_utc_convenience_policy_summary() -> UtcConveniencePolicySummary {
    UtcConveniencePolicySummary::new(CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT)
}

/// Returns the UTC-convenience policy posture used by validation and release reporting.
pub const fn utc_convenience_policy_summary_for_report() -> UtcConveniencePolicySummary {
    current_utc_convenience_policy_summary()
}

/// Returns the validated UTC-convenience policy summary line used by validation and release reporting.
pub fn validated_utc_convenience_policy_summary_for_report() -> String {
    match current_utc_convenience_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("UTC convenience policy unavailable ({error})"),
    }
}

/// Returns the current shared observer policy used by validation and reports.
pub const fn current_observer_policy_summary() -> ObserverPolicySummary {
    ObserverPolicySummary::new(CURRENT_OBSERVER_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared apparentness policy used by validation and reports.
pub const fn current_apparentness_policy_summary() -> ApparentnessPolicySummary {
    ApparentnessPolicySummary::new(CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared request-policy posture used by validation and reports.
pub const fn current_request_policy_summary() -> RequestPolicySummary {
    RequestPolicySummary {
        time_scale: CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT,
        observer: CURRENT_OBSERVER_POLICY_SUMMARY_TEXT,
        apparentness: CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT,
        frame: CURRENT_FRAME_POLICY_SUMMARY_TEXT,
    }
}

/// Returns the current shared frame-policy posture used by validation and reports.
pub const fn current_frame_policy_summary() -> FramePolicySummary {
    FramePolicySummary::new(CURRENT_FRAME_POLICY_SUMMARY_TEXT)
}

/// Returns the current native sidereal policy used by validation and reports.
pub const fn current_native_sidereal_policy_summary() -> NativeSiderealPolicySummary {
    NativeSiderealPolicySummary::new(CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT)
}

/// Returns the native sidereal policy posture used by validation and release reporting.
pub const fn native_sidereal_policy_summary_for_report() -> NativeSiderealPolicySummary {
    current_native_sidereal_policy_summary()
}

/// Returns the validated native sidereal policy summary line used by validation and release reporting.
pub fn validated_native_sidereal_policy_summary_for_report() -> String {
    match current_native_sidereal_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("native sidereal policy unavailable ({error})"),
    }
}

/// Returns the current zodiac posture used by validation and reports.
pub const fn current_zodiac_policy_summary() -> ZodiacPolicySummary {
    ZodiacPolicySummary::new(CURRENT_ZODIAC_POLICY_SUMMARY_TEXT)
}

/// Returns the validated zodiac policy summary line used by validation and release reporting.
pub fn validated_zodiac_policy_summary_for_report() -> String {
    match current_zodiac_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("zodiac policy unavailable ({error})"),
    }
}

/// Returns the current Pluto fallback posture used by validation and reports.
pub const fn current_pluto_fallback_summary() -> PlutoFallbackSummary {
    PlutoFallbackSummary::new(CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT)
}

/// Returns the Pluto fallback posture used by validation and release reporting.
pub const fn pluto_fallback_summary_for_report() -> PlutoFallbackSummary {
    current_pluto_fallback_summary()
}

/// Returns the validated Pluto fallback summary line used by validation and release reporting.
pub fn validated_pluto_fallback_summary_line_for_report(
) -> Result<&'static str, PlutoFallbackSummaryValidationError> {
    current_pluto_fallback_summary().validated_summary_line()
}

/// Returns the request-policy posture used by validation and release reporting.
pub const fn request_policy_summary_for_report() -> RequestPolicySummary {
    current_request_policy_summary()
}

/// Returns the validated request-policy summary line used by validation and release reporting.
pub fn validated_request_policy_summary_for_report() -> String {
    match current_request_policy_summary().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("request policy unavailable ({error})"),
    }
}

/// Returns the validated request-semantics summary line used by validation and release reporting.
///
/// This is a backend-layer alias for [`validated_request_policy_summary_for_report()`]
/// so callers that use the request-semantics vocabulary can share the same
/// guarded compact line without reinterpreting the compact report wording.
pub fn validated_request_semantics_summary_for_report() -> String {
    validated_request_policy_summary_for_report()
}

/// Returns the request-semantics posture used by validation and release reporting.
///
/// This is a backend-layer alias for [`request_policy_summary_for_report()`]
/// so callers that use the request-semantics vocabulary can share the same
/// typed summary without reinterpreting the compact report wording.
pub const fn request_semantics_summary_for_report() -> RequestPolicySummary {
    request_policy_summary_for_report()
}

/// Returns the observer-policy posture used by validation and release reporting.
pub const fn observer_policy_summary_for_report() -> ObserverPolicySummary {
    current_observer_policy_summary()
}

/// Returns the validated observer-policy summary line used by validation and release reporting.
pub fn validated_observer_policy_summary_for_report() -> String {
    match current_observer_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("observer policy unavailable ({error})"),
    }
}

/// Returns the apparentness-policy posture used by validation and release reporting.
pub const fn apparentness_policy_summary_for_report() -> ApparentnessPolicySummary {
    current_apparentness_policy_summary()
}

/// Returns the validated apparentness-policy summary line used by validation and release reporting.
pub fn validated_apparentness_policy_summary_for_report() -> String {
    match current_apparentness_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("apparentness policy unavailable ({error})"),
    }
}

/// Validates the request-shape policy shared by the current first-party backends.
///
/// This helper checks the request against the backend's published time-scale,
/// frame, and mean/apparent value-mode capabilities. It leaves body-specific,
/// observer, and zodiac-mode validation to the concrete backend so
/// implementations can keep their own source-specific error messages while
/// sharing the common policy guardrails.
pub fn validate_request_policy(
    req: &EphemerisRequest,
    backend_label: &str,
    supported_time_scales: &[TimeScale],
    supported_frames: &[CoordinateFrame],
    supports_mean: bool,
    supports_apparent: bool,
) -> Result<(), EphemerisError> {
    if !supported_time_scales.contains(&req.instant.scale) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedTimeScale,
            format!(
                "{backend_label} expects one of [{}] for request instants",
                format_display_list(supported_time_scales)
            ),
        ));
    }

    if !supported_frames.contains(&req.frame) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedCoordinateFrame,
            format!(
                "{backend_label} only returns [{}] coordinates",
                format_display_list(supported_frames)
            ),
        ));
    }

    match req.apparent {
        Apparentness::Mean if !supports_mean => {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedApparentness,
                format!(
                    "{backend_label} currently returns apparent coordinates only; mean geometric coordinates are not implemented"
                ),
            ));
        }
        Apparentness::Apparent if !supports_apparent => {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedApparentness,
                format!(
                    "{backend_label} currently returns mean geometric coordinates only; apparent corrections are not implemented"
                ),
            ));
        }
        _ => {}
    }

    Ok(())
}

pub(crate) fn validate_request_observer_location(
    req: &EphemerisRequest,
) -> Result<(), EphemerisError> {
    if let Some(observer) = &req.observer {
        observer.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidObserver,
                format!("request received invalid observer location: {error}"),
            )
        })?;
    }

    Ok(())
}

/// Validates a direct backend request against the published backend metadata.
///
/// This convenience helper combines the shared request-shape checks with custom
/// body and sidereal descriptor validation, observer-location validation, body
/// coverage, tropical-only zodiac routing for backends that do not advertise
/// native sidereal support, and topocentric capability validation. The shared
/// metadata model still does not capture per-ayanamsa sidereal catalog breadth,
/// so callers that need finer-grained sidereal routing must keep that logic at
/// the backend or façade layer. Routing backends are treated as a special case:
/// they still preflight custom definitions and body coverage here, but they
/// defer the broader time-scale, frame, zodiac, and value-mode capability
/// checks to the selected provider because their aggregate metadata is
/// intentionally conservative.
pub fn validate_request_against_metadata(
    req: &EphemerisRequest,
    metadata: &BackendMetadata,
) -> Result<(), EphemerisError> {
    metadata.validate_request(req)
}

/// Validates a batch of direct backend requests against backend metadata.
///
/// The helper first checks whether the backend advertises batch support and then
/// validates each request with [`validate_request_against_metadata`], failing
/// fast on the first unsupported request shape. The returned error message
/// prefixes the failing request's 1-based batch index so callers can correlate
/// the structured error with the slice position that triggered it. Batch
/// requests preserve sidereal, apparentness, observer, and body-coverage
/// failures with the same index prefix so callers can pinpoint the invalid slice
/// entry without losing the underlying request policy details. Routing backends
/// are treated conservatively here too: the aggregate metadata only gates the
/// body coverage, while the routed providers remain responsible for the
/// provider-specific batch and request-shape checks.
///
/// # Example
///
/// ```
/// use pleiades_backend::{
///     validate_requests_against_metadata, AccuracyClass, BackendCapabilities, BackendFamily,
///     BackendId, BackendMetadata, BackendProvenance, BodyClaim, EphemerisErrorKind,
///     EphemerisRequest,
/// };
/// use pleiades_types::{
///     CelestialBody, CoordinateFrame, Instant, JulianDay, Latitude, Longitude,
///     ObserverLocation, TimeRange, TimeScale,
/// };
///
/// let metadata = BackendMetadata {
///     id: BackendId::new("toy backend"),
///     version: "0.1.0".to_string(),
///     family: BackendFamily::Algorithmic,
///     provenance: BackendProvenance::new("toy backend"),
///     nominal_range: TimeRange::new(None, None),
///     supported_time_scales: vec![TimeScale::Tt],
///     body_claims: vec![BodyClaim::from(CelestialBody::Sun), BodyClaim::from(CelestialBody::Moon)],
///     supported_frames: vec![CoordinateFrame::Ecliptic],
///     capabilities: BackendCapabilities::default(),
///     accuracy: AccuracyClass::Approximate,
///     deterministic: true,
///     offline: true,
/// };
/// let requests = [
///     EphemerisRequest::new(
///         CelestialBody::Sun,
///         Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
///     ),
///     EphemerisRequest::new(
///         CelestialBody::Moon,
///         Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
///     ),
/// ];
///
/// assert!(validate_requests_against_metadata(&requests, &metadata).is_ok());
///
/// let mixed_scale_metadata = BackendMetadata {
///     supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
///     ..metadata.clone()
/// };
/// let mixed_scale_requests = [
///     EphemerisRequest::new(
///         CelestialBody::Sun,
///         Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
///     ),
///     EphemerisRequest::new(
///         CelestialBody::Moon,
///         Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
///     ),
/// ];
/// assert!(validate_requests_against_metadata(&mixed_scale_requests, &mixed_scale_metadata).is_ok());
///
/// let mut batchless_metadata = metadata.clone();
/// batchless_metadata.capabilities.batch = false;
/// let error = validate_requests_against_metadata(&requests, &batchless_metadata)
///     .expect_err("batch support should be required before dispatch");
/// assert_eq!(error.message, "toy backend does not support batch requests");
///
/// let observer_requests = [
///     EphemerisRequest::new(
///         CelestialBody::Sun,
///         Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
///     ),
///     EphemerisRequest {
///         observer: Some(ObserverLocation::new(
///             Latitude::from_degrees(51.5),
///             Longitude::from_degrees(12.5),
///             Some(0.0),
///         )),
///         ..EphemerisRequest::new(
///             CelestialBody::Moon,
///             Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
///         )
///     },
/// ];
/// let error = validate_requests_against_metadata(&observer_requests, &metadata)
///     .expect_err("observer-bearing batch requests should preserve the indexed observer failure");
/// assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
/// assert!(error.message.contains("batch request 2:"));
/// ```
pub fn validate_requests_against_metadata(
    reqs: &[EphemerisRequest],
    metadata: &BackendMetadata,
) -> Result<(), EphemerisError> {
    if !metadata.family.is_routing() && !metadata.capabilities.batch {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("{} does not support batch requests", metadata.id),
        ));
    }

    for (index, req) in reqs.iter().enumerate() {
        if let Err(error) = metadata.validate_request(req) {
            return Err(EphemerisError::new(
                error.kind,
                format!("batch request {}: {}", index + 1, error.message),
            ));
        }
    }

    Ok(())
}

/// Validates the zodiac-mode policy shared by the current first-party backends.
///
/// Current first-party backends that do not advertise native sidereal support
/// should call this after higher-priority request checks so sidereal requests
/// fail with a structured [`EphemerisErrorKind::UnsupportedZodiacMode`] error
/// rather than being silently coerced to tropical coordinates.
pub fn validate_zodiac_policy(
    req: &EphemerisRequest,
    backend_label: &str,
    supported_zodiac_modes: &[ZodiacMode],
) -> Result<(), EphemerisError> {
    if !supported_zodiac_modes.contains(&req.zodiac_mode) {
        let message = if supported_zodiac_modes.len() == 1
            && supported_zodiac_modes[0] == ZodiacMode::Tropical
        {
            format!("{backend_label} currently exposes tropical coordinates only")
        } else {
            format!(
                "{backend_label} currently exposes [{}] zodiac coordinates only",
                format_display_list(supported_zodiac_modes)
            )
        };

        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedZodiacMode,
            message,
        ));
    }

    Ok(())
}

/// Returns the compact report wording for the current time-scale policy.
pub const fn time_scale_policy_summary_for_report() -> TimeScalePolicySummary {
    current_time_scale_policy_summary()
}

/// Returns the validated compact time-scale policy summary line used by validation and release reporting.
pub fn validated_time_scale_policy_summary_for_report() -> String {
    match current_time_scale_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("time-scale policy unavailable ({error})"),
    }
}

/// Returns the compact report wording for the current Delta T policy.
pub const fn delta_t_policy_summary_for_report() -> DeltaTPolicySummary {
    current_delta_t_policy_summary()
}

/// Returns the validated compact Delta T policy summary line used by validation and release reporting.
pub fn validated_delta_t_policy_summary_for_report() -> String {
    match current_delta_t_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("Delta T policy unavailable ({error})"),
    }
}

/// Returns the compact report wording for the current frame policy.
pub const fn frame_policy_summary_for_report() -> &'static str {
    current_frame_policy_summary().summary_line()
}

/// Returns the compact report wording for the current frame policy after validation.
pub fn validated_frame_policy_summary_for_report() -> String {
    match current_frame_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("frame policy unavailable ({error})"),
    }
}

/// Returns the compact typed frame-policy posture for reporting.
pub const fn frame_policy_summary_details() -> FramePolicySummary {
    current_frame_policy_summary()
}

/// Returns the compact typed frame-treatment posture for reporting.
pub const fn frame_treatment_summary_for_report() -> FrameTreatmentSummary {
    FrameTreatmentSummary::new(current_request_policy_summary().frame)
}

/// Returns the current frame-treatment posture after validation.
pub fn validated_frame_treatment_summary_for_report() -> String {
    match frame_treatment_summary_for_report().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("frame treatment unavailable ({error})"),
    }
}

/// Formats the zodiac-mode policy shared by the current first-party backends.
pub fn zodiac_policy_summary_for_report(supported_zodiac_modes: &[ZodiacMode]) -> String {
    if supported_zodiac_modes.len() == 1 && supported_zodiac_modes[0] == ZodiacMode::Tropical {
        "tropical only".to_string()
    } else {
        format!(
            "zodiac modes=[{}]",
            format_display_list(supported_zodiac_modes)
        )
    }
}

/// Validates the observer policy shared by the current first-party backends.
///
/// Geocentric-only backends should call this after any higher-priority request
/// checks they want to preserve so observer-bearing requests fail with a
/// structured [`EphemerisErrorKind::UnsupportedObserver`] error.
pub fn validate_observer_policy(
    req: &EphemerisRequest,
    backend_label: &str,
    supports_topocentric: bool,
) -> Result<(), EphemerisError> {
    if let Some(observer) = req.observer.as_ref() {
        observer.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidObserver,
                format!(
                    "{backend_label} received invalid observer location for {}: {error}",
                    observer.summary_line(),
                ),
            )
        })?;

        if !supports_topocentric {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedObserver,
                format!(
                    "{backend_label} is geocentric only; topocentric positions are not implemented for {}",
                    observer.summary_line()
                ),
            ));
        }
    }

    Ok(())
}
