//! Backend contracts, metadata, and adapter helpers for ephemeris providers.
//!
//! This crate defines the shared request/response shape used by all backend
//! families. Concrete backends live in their own `pleiades-*` crates and
//! implement [`EphemerisBackend`].
//!
//! Enable the optional `serde` feature to serialize the shared request,
//! result, metadata, and error types used by backend implementations.
//!
//! # Examples
//!
//! ```
//! use pleiades_backend::{EphemerisBackend, EphemerisRequest, BackendMetadata, BackendId, BackendFamily,
//!     BackendProvenance, BackendCapabilities, AccuracyClass, TimeRange, EphemerisResult, EphemerisError,
//!     EphemerisErrorKind, Apparentness, QualityAnnotation};
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

use core::fmt;
use std::borrow::Cow;

pub use pleiades_types::{
    Angle, Apparentness, Ayanamsa, CelestialBody, CoordinateFrame, CustomAyanamsa, CustomBodyId,
    CustomHouseSystem, EclipticCoordinates, EquatorialCoordinates, HouseSystem, Instant, JulianDay,
    Latitude, Longitude, Motion, ObserverLocation, TimeRange, TimeScale, ZodiacMode,
};

/// Stable identifier for a backend implementation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BackendId(String);

impl BackendId {
    /// Creates a new backend identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BackendId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The high-level backend family.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendFamily {
    /// A formula-based or algorithmic backend.
    Algorithmic,
    /// A backend backed primarily by reference data.
    ReferenceData,
    /// A backend backed by compressed packaged artifacts.
    CompressedData,
    /// A backend that routes across multiple providers.
    Composite,
    /// A future or project-specific family.
    Other(String),
}

impl BackendFamily {
    /// Returns a stable human-readable label for the backend family.
    pub fn display_name(&self) -> Cow<'_, str> {
        match self {
            Self::Algorithmic => Cow::Borrowed("Algorithmic"),
            Self::ReferenceData => Cow::Borrowed("ReferenceData"),
            Self::CompressedData => Cow::Borrowed("CompressedData"),
            Self::Composite => Cow::Borrowed("Composite"),
            Self::Other(value) => Cow::Owned(format!("Other({value})")),
        }
    }
}

impl fmt::Display for BackendFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_name())
    }
}

/// A rough accuracy class for a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AccuracyClass {
    /// Exact or source-equivalent within the backend's documented model.
    Exact,
    /// High accuracy suitable for production use.
    High,
    /// Moderate accuracy.
    Moderate,
    /// Approximate or preliminary accuracy.
    Approximate,
    /// Accuracy class is unknown or not yet published.
    Unknown,
}

impl AccuracyClass {
    /// Returns a stable human-readable label for the accuracy class.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::High => "High",
            Self::Moderate => "Moderate",
            Self::Approximate => "Approximate",
            Self::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for AccuracyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Provenance summary for a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BackendProvenance {
    /// Short human-readable summary of the backend's source material.
    pub summary: String,
    /// External data or reference sources used by the backend.
    pub data_sources: Vec<String>,
}

impl BackendProvenance {
    /// Creates a new provenance summary.
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            data_sources: Vec::new(),
        }
    }
}

/// Capability flags for a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BackendCapabilities {
    /// Whether geocentric calculations are supported.
    pub geocentric: bool,
    /// Whether topocentric calculations are supported.
    pub topocentric: bool,
    /// Whether apparent values are supported.
    pub apparent: bool,
    /// Whether mean values are supported.
    pub mean: bool,
    /// Whether the backend can serve batch requests.
    pub batch: bool,
    /// Whether sidereal outputs are computed natively rather than derived above the backend.
    pub native_sidereal: bool,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        Self {
            geocentric: true,
            topocentric: false,
            apparent: true,
            mean: true,
            batch: true,
            native_sidereal: false,
        }
    }
}

/// Nominal backend metadata.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BackendMetadata {
    /// Stable backend identifier.
    pub id: BackendId,
    /// Human-readable backend version.
    pub version: String,
    /// Backend family.
    pub family: BackendFamily,
    /// Provenance summary.
    pub provenance: BackendProvenance,
    /// Nominal supported time range.
    pub nominal_range: TimeRange,
    /// Time scales the backend can accept.
    pub supported_time_scales: Vec<TimeScale>,
    /// Supported body coverage.
    pub body_coverage: Vec<CelestialBody>,
    /// Supported coordinate frames.
    pub supported_frames: Vec<CoordinateFrame>,
    /// Declared capabilities.
    pub capabilities: BackendCapabilities,
    /// Published accuracy class.
    pub accuracy: AccuracyClass,
    /// Whether repeated queries are deterministic.
    pub deterministic: bool,
    /// Whether the backend runs fully offline.
    pub offline: bool,
}

/// A backend request.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct EphemerisRequest {
    /// Requested body.
    pub body: CelestialBody,
    /// Requested instant.
    pub instant: Instant,
    /// Optional observer location for topocentric calculations.
    pub observer: Option<ObserverLocation>,
    /// Requested coordinate frame.
    pub frame: CoordinateFrame,
    /// Requested zodiac mode.
    pub zodiac_mode: ZodiacMode,
    /// Whether apparent or mean values are preferred.
    pub apparent: Apparentness,
}

impl EphemerisRequest {
    /// Creates a new request with sensible defaults for a tropical geocentric query.
    ///
    /// The default apparentness is mean geometric output so a bare request stays
    /// compatible with the current mean-only first-party backends.
    pub fn new(body: CelestialBody, instant: Instant) -> Self {
        Self {
            body,
            instant,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        }
    }
}

/// Quality annotation for a backend result.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum QualityAnnotation {
    /// Exact or source-equivalent data.
    Exact,
    /// Interpolated from source samples.
    Interpolated,
    /// Approximate but still useful.
    Approximate,
    /// Quality is not yet published.
    Unknown,
}

/// A backend result containing the requested coordinates where available.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct EphemerisResult {
    /// Backend that produced the result.
    pub backend_id: BackendId,
    /// Body that was queried.
    pub body: CelestialBody,
    /// Instant that was queried.
    pub instant: Instant,
    /// Coordinate frame of the result.
    pub frame: CoordinateFrame,
    /// Zodiac mode of the result.
    pub zodiac_mode: ZodiacMode,
    /// Whether apparent or mean values were requested.
    pub apparent: Apparentness,
    /// Ecliptic coordinates when available.
    pub ecliptic: Option<EclipticCoordinates>,
    /// Equatorial coordinates when available.
    pub equatorial: Option<EquatorialCoordinates>,
    /// Apparent motion when available.
    pub motion: Option<Motion>,
    /// Quality annotation for the result.
    pub quality: QualityAnnotation,
}

impl EphemerisResult {
    /// Creates an empty result shell with the request metadata filled in.
    pub fn new(
        backend_id: BackendId,
        body: CelestialBody,
        instant: Instant,
        frame: CoordinateFrame,
        zodiac_mode: ZodiacMode,
        apparent: Apparentness,
    ) -> Self {
        Self {
            backend_id,
            body,
            instant,
            frame,
            zodiac_mode,
            apparent,
            ecliptic: None,
            equatorial: None,
            motion: None,
            quality: QualityAnnotation::Unknown,
        }
    }
}

/// Error categories for backend queries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum EphemerisErrorKind {
    /// The requested body is not supported.
    UnsupportedBody,
    /// The requested coordinate frame is not supported.
    UnsupportedCoordinateFrame,
    /// The requested time scale is not supported.
    UnsupportedTimeScale,
    /// The observer parameters are invalid for the calculation.
    InvalidObserver,
    /// The instant lies outside the backend's nominal range.
    OutOfRangeInstant,
    /// Required data is missing.
    MissingDataset,
    /// The backend encountered a numerical failure.
    NumericalFailure,
    /// The request is malformed or internally inconsistent.
    InvalidRequest,
}

/// A structured backend error.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EphemerisError {
    /// The error category.
    pub kind: EphemerisErrorKind,
    /// Human-readable error message.
    pub message: String,
}

impl EphemerisError {
    /// Creates a new structured backend error.
    pub fn new(kind: EphemerisErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for EphemerisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for EphemerisError {}

fn format_debug_list<T: fmt::Debug>(values: &[T]) -> String {
    values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Validates the request-shape policy shared by the current first-party backends.
///
/// This helper checks the request against the backend's published time-scale,
/// frame, and apparentness capabilities. It leaves body-specific, observer,
/// and zodiac-mode validation to the concrete backend so implementations can
/// keep their own source-specific error messages while sharing the common
/// policy guardrails.
pub fn validate_request_policy(
    req: &EphemerisRequest,
    backend_label: &str,
    supported_time_scales: &[TimeScale],
    supported_frames: &[CoordinateFrame],
    supports_apparent: bool,
) -> Result<(), EphemerisError> {
    if !supported_time_scales.contains(&req.instant.scale) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedTimeScale,
            format!(
                "{backend_label} expects one of [{}] for request instants",
                format_debug_list(supported_time_scales)
            ),
        ));
    }

    if !supported_frames.contains(&req.frame) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedCoordinateFrame,
            format!(
                "{backend_label} only returns [{}] coordinates",
                format_debug_list(supported_frames)
            ),
        ));
    }

    if req.apparent == Apparentness::Apparent && !supports_apparent {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "{backend_label} currently returns mean geometric coordinates only; apparent corrections are not implemented"
            ),
        ));
    }

    Ok(())
}

/// Validates the zodiac-mode policy shared by the current first-party backends.
///
/// Current first-party backends that do not advertise native sidereal support
/// should call this after higher-priority request checks so sidereal requests
/// fail with a structured [`EphemerisErrorKind::InvalidRequest`] error rather
/// than being silently coerced to tropical coordinates.
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
                format_debug_list(supported_zodiac_modes)
            )
        };

        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            message,
        ));
    }

    Ok(())
}

/// Validates the observer policy shared by the current first-party backends.
///
/// Geocentric-only backends should call this after any higher-priority request
/// checks they want to preserve so observer-bearing requests fail with a
/// structured [`EphemerisErrorKind::InvalidObserver`] error.
pub fn validate_observer_policy(
    req: &EphemerisRequest,
    backend_label: &str,
    supports_topocentric: bool,
) -> Result<(), EphemerisError> {
    if req.observer.is_some() && !supports_topocentric {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidObserver,
            format!(
                "{backend_label} is geocentric only; topocentric positions are not implemented"
            ),
        ));
    }

    Ok(())
}

/// The shared backend contract.
///
/// Implementations must support one-request/one-result queries. Batch querying
/// is provided as a default all-or-error adapter that fail-fast stops on the
/// first structured error so callers can build chart-style workflows without
/// hand-rolling request loops.
pub trait EphemerisBackend: Send + Sync {
    /// Returns backend metadata.
    fn metadata(&self) -> BackendMetadata;

    /// Returns whether the backend supports the requested body.
    fn supports_body(&self, body: CelestialBody) -> bool;

    /// Computes a single ephemeris result.
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError>;

    /// Computes multiple ephemeris results.
    fn positions(&self, reqs: &[EphemerisRequest]) -> Result<Vec<EphemerisResult>, EphemerisError> {
        reqs.iter().map(|req| self.position(req)).collect()
    }
}

/// A simple composite backend that routes requests to one of two providers.
///
/// The primary backend is consulted first. If it does not advertise support for
/// the requested body, the secondary backend is tried instead.
#[derive(Debug)]
pub struct CompositeBackend<A, B> {
    primary: A,
    secondary: B,
}

impl<A, B> CompositeBackend<A, B> {
    /// Creates a new routing backend.
    pub const fn new(primary: A, secondary: B) -> Self {
        Self { primary, secondary }
    }

    /// Returns the primary backend.
    pub const fn primary(&self) -> &A {
        &self.primary
    }

    /// Returns the secondary backend.
    pub const fn secondary(&self) -> &B {
        &self.secondary
    }
}

impl<A: EphemerisBackend, B: EphemerisBackend> EphemerisBackend for CompositeBackend<A, B> {
    fn metadata(&self) -> BackendMetadata {
        let primary = self.primary.metadata();
        let secondary = self.secondary.metadata();
        BackendMetadata {
            id: BackendId::new(format!(
                "composite:{}+{}",
                primary.id.as_str(),
                secondary.id.as_str()
            )),
            version: primary.version.clone(),
            family: BackendFamily::Composite,
            provenance: BackendProvenance {
                summary: format!(
                    "Composite routing backend combining {} and {}.",
                    primary.provenance.summary, secondary.provenance.summary
                ),
                data_sources: combine_sources(
                    &primary.provenance.data_sources,
                    &secondary.provenance.data_sources,
                ),
            },
            nominal_range: intersect_ranges(primary.nominal_range, secondary.nominal_range),
            supported_time_scales: intersect_strings(
                &primary.supported_time_scales,
                &secondary.supported_time_scales,
            ),
            body_coverage: combine_bodies(&primary.body_coverage, &secondary.body_coverage),
            supported_frames: intersect_strings(
                &primary.supported_frames,
                &secondary.supported_frames,
            ),
            capabilities: BackendCapabilities {
                geocentric: primary.capabilities.geocentric && secondary.capabilities.geocentric,
                topocentric: primary.capabilities.topocentric && secondary.capabilities.topocentric,
                apparent: primary.capabilities.apparent && secondary.capabilities.apparent,
                mean: primary.capabilities.mean && secondary.capabilities.mean,
                batch: primary.capabilities.batch && secondary.capabilities.batch,
                native_sidereal: primary.capabilities.native_sidereal
                    && secondary.capabilities.native_sidereal,
            },
            accuracy: min_accuracy(primary.accuracy, secondary.accuracy),
            deterministic: primary.deterministic && secondary.deterministic,
            offline: primary.offline && secondary.offline,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.primary.supports_body(body.clone()) || self.secondary.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let primary_supports = self.primary.supports_body(req.body.clone());
        let secondary_supports = self.secondary.supports_body(req.body.clone());

        if primary_supports {
            match self.primary.position(req) {
                Ok(result) => Ok(result),
                Err(error) if secondary_supports && should_fallback_to_secondary(&error.kind) => {
                    self.secondary.position(req)
                }
                Err(error) => Err(error),
            }
        } else if secondary_supports {
            self.secondary.position(req)
        } else {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "no backend in the composite router supports the requested body",
            ))
        }
    }
}

/// A routing backend that can chain any number of providers.
///
/// The router queries providers in priority order and falls back to later
/// backends when the earlier ones report a retryable routing error. This makes
/// it convenient to compose packaged, algorithmic, and reference-data
/// backends without nesting multiple binary composites.
#[derive(Default)]
pub struct RoutingBackend {
    backends: Vec<Box<dyn EphemerisBackend>>,
}

impl RoutingBackend {
    /// Creates a new routing backend from a prioritized list of providers.
    pub fn new(backends: Vec<Box<dyn EphemerisBackend>>) -> Self {
        Self { backends }
    }

    /// Returns the configured provider chain.
    pub fn backends(&self) -> &[Box<dyn EphemerisBackend>] {
        &self.backends
    }

    /// Returns `true` if no providers are configured.
    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }
}

impl EphemerisBackend for RoutingBackend {
    fn metadata(&self) -> BackendMetadata {
        let backends: Vec<&dyn EphemerisBackend> = self
            .backends
            .iter()
            .map(|backend| backend.as_ref())
            .collect();
        let metadatas: Vec<BackendMetadata> =
            backends.iter().map(|backend| backend.metadata()).collect();

        if metadatas.is_empty() {
            return BackendMetadata {
                id: BackendId::new("routing:empty"),
                version: "routing[none]".to_string(),
                family: BackendFamily::Composite,
                provenance: BackendProvenance::new("Routing backend with no configured providers."),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: Vec::new(),
                body_coverage: Vec::new(),
                supported_frames: Vec::new(),
                capabilities: BackendCapabilities {
                    geocentric: false,
                    topocentric: false,
                    apparent: false,
                    mean: false,
                    batch: false,
                    native_sidereal: false,
                },
                accuracy: AccuracyClass::Unknown,
                deterministic: true,
                offline: true,
            };
        }

        let mut id_parts = Vec::with_capacity(metadatas.len());
        let mut version_parts = Vec::with_capacity(metadatas.len());
        let mut provenance_parts = Vec::with_capacity(metadatas.len());
        let mut data_sources = Vec::new();
        let mut nominal_range = metadatas[0].nominal_range;
        let mut supported_time_scales = metadatas[0].supported_time_scales.clone();
        let mut body_coverage = metadatas[0].body_coverage.clone();
        let mut supported_frames = metadatas[0].supported_frames.clone();
        let mut capabilities = metadatas[0].capabilities.clone();
        let mut accuracy = metadatas[0].accuracy;
        let mut deterministic = metadatas[0].deterministic;
        let mut offline = metadatas[0].offline;

        for metadata in &metadatas {
            id_parts.push(metadata.id.as_str().to_string());
            version_parts.push(metadata.version.clone());
            provenance_parts.push(metadata.provenance.summary.clone());
            data_sources = combine_sources(&data_sources, &metadata.provenance.data_sources);
            nominal_range = intersect_ranges(nominal_range, metadata.nominal_range);
            supported_time_scales =
                intersect_strings(&supported_time_scales, &metadata.supported_time_scales);
            body_coverage = combine_bodies(&body_coverage, &metadata.body_coverage);
            supported_frames = intersect_strings(&supported_frames, &metadata.supported_frames);
            capabilities.geocentric &= metadata.capabilities.geocentric;
            capabilities.topocentric &= metadata.capabilities.topocentric;
            capabilities.apparent &= metadata.capabilities.apparent;
            capabilities.mean &= metadata.capabilities.mean;
            capabilities.batch &= metadata.capabilities.batch;
            capabilities.native_sidereal &= metadata.capabilities.native_sidereal;
            accuracy = min_accuracy(accuracy, metadata.accuracy);
            deterministic &= metadata.deterministic;
            offline &= metadata.offline;
        }

        BackendMetadata {
            id: BackendId::new(format!("routing:{}", id_parts.join("+"))),
            version: format!("routing[{}]", version_parts.join("+")),
            family: BackendFamily::Composite,
            provenance: BackendProvenance {
                summary: format!(
                    "Routing backend combining {} provider(s): {}.",
                    metadatas.len(),
                    provenance_parts.join("; ")
                ),
                data_sources,
            },
            nominal_range,
            supported_time_scales,
            body_coverage,
            supported_frames,
            capabilities,
            accuracy,
            deterministic,
            offline,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.backends
            .iter()
            .any(|backend| backend.supports_body(body.clone()))
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut saw_support = false;
        let mut last_retryable_error = None;

        for backend in &self.backends {
            if !backend.supports_body(req.body.clone()) {
                continue;
            }

            saw_support = true;
            match backend.position(req) {
                Ok(result) => return Ok(result),
                Err(error) if should_fallback_to_secondary(&error.kind) => {
                    last_retryable_error = Some(error);
                }
                Err(error) => return Err(error),
            }
        }

        if let Some(error) = last_retryable_error {
            Err(error)
        } else if saw_support {
            Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "configured providers could not satisfy the requested body and request shape",
            ))
        } else {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "no backend in the routing chain supports the requested body",
            ))
        }
    }
}

fn combine_sources(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut combined = primary.to_vec();
    for source in secondary {
        if !combined.iter().any(|existing| existing == source) {
            combined.push(source.clone());
        }
    }
    combined
}

fn combine_bodies(primary: &[CelestialBody], secondary: &[CelestialBody]) -> Vec<CelestialBody> {
    let mut combined = primary.to_vec();
    for body in secondary {
        if !combined.contains(body) {
            combined.push(body.clone());
        }
    }
    combined
}

fn intersect_strings<T: Clone + PartialEq>(primary: &[T], secondary: &[T]) -> Vec<T> {
    primary
        .iter()
        .filter(|value| secondary.contains(value))
        .cloned()
        .collect()
}

fn intersect_ranges(primary: TimeRange, secondary: TimeRange) -> TimeRange {
    let start = match (primary.start, secondary.start) {
        (Some(a), Some(b)) => Some(if a.julian_day.days() >= b.julian_day.days() {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    let end = match (primary.end, secondary.end) {
        (Some(a), Some(b)) => Some(if a.julian_day.days() <= b.julian_day.days() {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    TimeRange::new(start, end)
}

fn min_accuracy(primary: AccuracyClass, secondary: AccuracyClass) -> AccuracyClass {
    use AccuracyClass::*;

    match (primary, secondary) {
        (Unknown, _) | (_, Unknown) => Unknown,
        (Approximate, _) | (_, Approximate) => Approximate,
        (Moderate, _) | (_, Moderate) => Moderate,
        (High, _) | (_, High) => High,
        (Exact, Exact) => Exact,
    }
}

fn should_fallback_to_secondary(kind: &EphemerisErrorKind) -> bool {
    matches!(
        kind,
        EphemerisErrorKind::UnsupportedBody
            | EphemerisErrorKind::UnsupportedCoordinateFrame
            | EphemerisErrorKind::UnsupportedTimeScale
            | EphemerisErrorKind::InvalidObserver
            | EphemerisErrorKind::MissingDataset
            | EphemerisErrorKind::InvalidRequest
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn family_and_accuracy_labels_are_stable() {
        assert_eq!(BackendFamily::Algorithmic.to_string(), "Algorithmic");
        assert_eq!(BackendFamily::ReferenceData.to_string(), "ReferenceData");
        assert_eq!(BackendFamily::CompressedData.to_string(), "CompressedData");
        assert_eq!(BackendFamily::Composite.to_string(), "Composite");
        assert_eq!(
            BackendFamily::Other("custom".to_string()).to_string(),
            "Other(custom)"
        );

        assert_eq!(AccuracyClass::Exact.to_string(), "Exact");
        assert_eq!(AccuracyClass::High.to_string(), "High");
        assert_eq!(AccuracyClass::Moderate.to_string(), "Moderate");
        assert_eq!(AccuracyClass::Approximate.to_string(), "Approximate");
        assert_eq!(AccuracyClass::Unknown.to_string(), "Unknown");
    }

    struct ToyBackend;

    impl EphemerisBackend for ToyBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("toy"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("toy backend for tests"),
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

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            if req.body != CelestialBody::Sun {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "only the Sun is supported",
                ));
            }

            let mut result = EphemerisResult::new(
                BackendId::new("toy"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            );
            result.quality = QualityAnnotation::Exact;
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(120.0),
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }
    }

    #[test]
    fn request_policy_helpers_reject_unsupported_shapes() {
        let time_scale_request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(51.5),
                Longitude::from_degrees(-0.1),
                Some(45.0),
            )),
            frame: CoordinateFrame::Equatorial,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Apparent,
        };
        let error = validate_request_policy(
            &time_scale_request,
            "toy backend",
            &[TimeScale::Tt],
            &[CoordinateFrame::Ecliptic],
            false,
        )
        .expect_err("UTC should be rejected when only TT is supported");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);

        let frame_request = EphemerisRequest {
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
            observer: None,
            frame: CoordinateFrame::Equatorial,
            apparent: Apparentness::Mean,
            ..time_scale_request.clone()
        };
        let error = validate_request_policy(
            &frame_request,
            "toy backend",
            &[TimeScale::Tt],
            &[CoordinateFrame::Ecliptic],
            false,
        )
        .expect_err("equatorial frame should be rejected when only ecliptic is supported");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedCoordinateFrame);

        let apparent_request = EphemerisRequest {
            frame: CoordinateFrame::Ecliptic,
            apparent: Apparentness::Apparent,
            observer: None,
            ..frame_request.clone()
        };
        let error = validate_request_policy(
            &apparent_request,
            "toy backend",
            &[TimeScale::Tt],
            &[CoordinateFrame::Ecliptic],
            false,
        )
        .expect_err("apparent requests should be rejected when only mean output is supported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);

        let sidereal_request = EphemerisRequest {
            zodiac_mode: ZodiacMode::Sidereal {
                ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
            },
            ..apparent_request.clone()
        };
        let error =
            validate_zodiac_policy(&sidereal_request, "toy backend", &[ZodiacMode::Tropical])
                .expect_err(
                    "sidereal requests should be rejected when only tropical output is supported",
                );
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("tropical coordinates only"));

        let observer_request = EphemerisRequest {
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(51.5),
                Longitude::from_degrees(-0.1),
                Some(45.0),
            )),
            ..apparent_request.clone()
        };
        let error = validate_observer_policy(&observer_request, "toy backend", false)
            .expect_err("topocentric requests should be rejected when unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    }

    #[test]
    fn batch_query_preserves_apparent_request_rejection() {
        struct MeanOnlyBackend {
            calls: AtomicUsize,
        }

        impl EphemerisBackend for MeanOnlyBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("mean-only"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("mean-only test backend"),
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

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                self.calls.fetch_add(1, Ordering::SeqCst);

                validate_request_policy(
                    req,
                    "mean-only test backend",
                    &[TimeScale::Tt],
                    &[CoordinateFrame::Ecliptic],
                    false,
                )?;

                Ok(EphemerisResult::new(
                    BackendId::new("mean-only"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let backend = MeanOnlyBackend {
            calls: AtomicUsize::new(0),
        };
        let mean_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );
        let apparent_request = EphemerisRequest {
            apparent: Apparentness::Apparent,
            ..mean_request.clone()
        };

        let error = backend
            .positions(&[mean_request, apparent_request])
            .expect_err("batch requests should preserve apparentness rejections");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_preserves_requests_and_results() {
        let request = EphemerisRequest {
            body: CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(51.5),
                Longitude::from_degrees(-0.1),
                Some(45.0),
            )),
            frame: CoordinateFrame::Equatorial,
            zodiac_mode: ZodiacMode::Sidereal {
                ayanamsa: Ayanamsa::Lahiri,
            },
            apparent: Apparentness::Mean,
        };
        let request_roundtrip: EphemerisRequest = serde_json::from_value(
            serde_json::to_value(&request).expect("request should serialize"),
        )
        .expect("request should deserialize");
        assert_eq!(request_roundtrip, request);

        let mut result = EphemerisResult::new(
            BackendId::new("toy"),
            CelestialBody::Moon,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
            CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        result.quality = QualityAnnotation::Interpolated;
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(123.0),
            Latitude::from_degrees(2.5),
            Some(1.0),
        ));
        result.motion = Some(Motion::new(Some(0.12), Some(-0.01), None));

        let result_roundtrip: EphemerisResult =
            serde_json::from_value(serde_json::to_value(&result).expect("result should serialize"))
                .expect("result should deserialize");
        assert_eq!(result_roundtrip, result);
    }

    #[test]
    fn batch_query_uses_single_query_behavior() {
        let backend = ToyBackend;
        let request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );

        let result = backend
            .positions(&[request])
            .expect("toy backend should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].quality, QualityAnnotation::Exact);
        assert!(result[0].ecliptic.is_some());
    }

    #[test]
    fn batch_query_short_circuits_on_the_first_error() {
        struct CountingBackend {
            calls: AtomicUsize,
        }

        impl EphemerisBackend for CountingBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("counting"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("counting test backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt],
                    body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
                    supported_frames: vec![CoordinateFrame::Ecliptic],
                    capabilities: BackendCapabilities::default(),
                    accuracy: AccuracyClass::Approximate,
                    deterministic: true,
                    offline: true,
                }
            }

            fn supports_body(&self, _body: CelestialBody) -> bool {
                true
            }

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                self.calls.fetch_add(1, Ordering::SeqCst);

                if req.body == CelestialBody::Moon {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::UnsupportedBody,
                        "Moon requests fail in the counting backend",
                    ));
                }

                Ok(EphemerisResult::new(
                    BackendId::new("counting"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let backend = CountingBackend {
            calls: AtomicUsize::new(0),
        };
        let requests = [
            EphemerisRequest::new(
                CelestialBody::Sun,
                Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
            ),
            EphemerisRequest::new(
                CelestialBody::Moon,
                Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
            ),
            EphemerisRequest::new(
                CelestialBody::Sun,
                Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
            ),
        ];

        let error = backend
            .positions(&requests)
            .expect_err("batch requests should fail fast on the first error");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn metadata_is_reported() {
        let backend = ToyBackend;
        let metadata = backend.metadata();
        assert_eq!(metadata.id.as_str(), "toy");
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
    }

    #[test]
    fn composite_backend_routes_by_body() {
        struct MoonBackend;

        impl EphemerisBackend for MoonBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("moon"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("moon backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt],
                    body_coverage: vec![CelestialBody::Moon],
                    supported_frames: vec![CoordinateFrame::Ecliptic],
                    capabilities: BackendCapabilities::default(),
                    accuracy: AccuracyClass::Approximate,
                    deterministic: true,
                    offline: true,
                }
            }

            fn supports_body(&self, body: CelestialBody) -> bool {
                body == CelestialBody::Moon
            }

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                Ok(EphemerisResult::new(
                    BackendId::new("moon"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let composite = CompositeBackend::new(ToyBackend, MoonBackend);
        let sun_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );
        let moon_request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );

        assert_eq!(
            composite
                .position(&sun_request)
                .unwrap()
                .backend_id
                .as_str(),
            "toy"
        );
        assert_eq!(
            composite
                .position(&moon_request)
                .unwrap()
                .backend_id
                .as_str(),
            "moon"
        );
    }

    #[test]
    fn routing_backend_tries_later_providers_and_merges_metadata() {
        struct FailingSunBackend;

        impl EphemerisBackend for FailingSunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("fail-sun"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("failing Sun backend"),
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

            fn position(&self, _req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedCoordinateFrame,
                    "retry with the next provider",
                ))
            }
        }

        struct RecoverySunBackend;

        impl EphemerisBackend for RecoverySunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("recovery-sun"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("recovery Sun backend"),
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

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                let mut result = EphemerisResult::new(
                    BackendId::new("recovery-sun"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                );
                result.quality = QualityAnnotation::Exact;
                result.ecliptic = Some(EclipticCoordinates::new(
                    Longitude::from_degrees(10.0),
                    Latitude::from_degrees(0.0),
                    Some(1.0),
                ));
                Ok(result)
            }
        }

        struct MoonBackend;

        impl EphemerisBackend for MoonBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("moon"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("moon backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt],
                    body_coverage: vec![CelestialBody::Moon],
                    supported_frames: vec![CoordinateFrame::Ecliptic],
                    capabilities: BackendCapabilities::default(),
                    accuracy: AccuracyClass::Approximate,
                    deterministic: true,
                    offline: true,
                }
            }

            fn supports_body(&self, body: CelestialBody) -> bool {
                body == CelestialBody::Moon
            }

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                Ok(EphemerisResult::new(
                    BackendId::new("moon"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let routing = RoutingBackend::new(vec![
            Box::new(FailingSunBackend),
            Box::new(RecoverySunBackend),
            Box::new(MoonBackend),
        ]);
        let sun_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );
        let moon_request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );

        let metadata = routing.metadata();
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
        assert!(metadata.body_coverage.contains(&CelestialBody::Moon));
        assert!(metadata.provenance.summary.contains("3 provider(s)"));

        assert_eq!(
            routing.position(&sun_request).unwrap().backend_id.as_str(),
            "recovery-sun"
        );
        assert_eq!(
            routing.position(&moon_request).unwrap().backend_id.as_str(),
            "moon"
        );
    }
}
