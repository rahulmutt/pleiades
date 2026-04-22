//! Backend contracts, metadata, and adapter helpers for ephemeris providers.
//!
//! This crate defines the shared request/response shape used by all backend
//! families. Concrete backends live in their own `pleiades-*` crates and
//! implement [`EphemerisBackend`].
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

pub use pleiades_types::{
    Angle, Apparentness, Ayanamsa, CelestialBody, CoordinateFrame, CustomAyanamsa, CustomBodyId,
    CustomHouseSystem, EclipticCoordinates, EquatorialCoordinates, HouseSystem, Instant, JulianDay,
    Latitude, Longitude, Motion, ObserverLocation, TimeRange, TimeScale, ZodiacMode,
};

/// Stable identifier for a backend implementation.
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

/// A rough accuracy class for a backend.
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

/// Provenance summary for a backend.
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
    pub fn new(body: CelestialBody, instant: Instant) -> Self {
        Self {
            body,
            instant,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Apparent,
        }
    }
}

/// Quality annotation for a backend result.
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

/// The shared backend contract.
///
/// Implementations must support one-request/one-result queries. Batch querying
/// is provided as a default all-or-error adapter so callers can build chart-style
/// workflows without hand-rolling request loops.
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn request_defaults_are_sensible() {
        let request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );

        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.apparent, Apparentness::Apparent);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
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
    fn metadata_is_reported() {
        let backend = ToyBackend;
        let metadata = backend.metadata();
        assert_eq!(metadata.id.as_str(), "toy");
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
    }
}
