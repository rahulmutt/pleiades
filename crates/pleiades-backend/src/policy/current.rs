use crate::errors::{format_display_list, EphemerisError, EphemerisErrorKind};
use crate::metadata::BackendMetadata;
use crate::request::EphemerisRequest;
use pleiades_types::{Apparentness, CoordinateFrame, TimeScale, ZodiacMode};

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
