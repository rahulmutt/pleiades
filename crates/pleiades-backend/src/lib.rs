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

/// A coarse posture label for how a backend family is typically categorized in release summaries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendFamilyPosture {
    /// Formula-driven backends.
    Algorithmic,
    /// Reference-data and packaged-data backends.
    DataBacked,
    /// Multi-provider routing backends.
    Routing,
    /// Families that do not yet have a sharper public posture label.
    Other,
}

impl BackendFamilyPosture {
    /// Returns a stable human-readable label for the posture classification.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Algorithmic => "algorithmic",
            Self::DataBacked => "data-backed",
            Self::Routing => "routing",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for BackendFamilyPosture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
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

    /// Returns `true` when the backend is driven primarily by external source data.
    pub const fn is_data_backed(&self) -> bool {
        matches!(self, Self::ReferenceData | Self::CompressedData)
    }

    /// Returns `true` when the backend is formula-driven rather than data-backed.
    pub const fn is_algorithmic(&self) -> bool {
        matches!(self, Self::Algorithmic)
    }

    /// Returns `true` when the backend routes across multiple providers.
    pub const fn is_routing(&self) -> bool {
        matches!(self, Self::Composite)
    }

    /// Returns the coarse posture classification used in compact release summaries.
    pub const fn posture(&self) -> BackendFamilyPosture {
        if self.is_algorithmic() {
            BackendFamilyPosture::Algorithmic
        } else if self.is_data_backed() {
            BackendFamilyPosture::DataBacked
        } else if self.is_routing() {
            BackendFamilyPosture::Routing
        } else {
            BackendFamilyPosture::Other
        }
    }

    /// Returns a short posture label for release-facing summaries.
    pub const fn posture_label(&self) -> &'static str {
        self.posture().label()
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

    /// Returns a compact one-line rendering of the provenance summary.
    pub fn summary_line(&self) -> String {
        self.summary.clone()
    }
}

impl fmt::Display for BackendProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary)
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

impl BackendCapabilities {
    /// Returns a compact one-line rendering of the declared capability flags.
    pub fn summary_line(&self) -> String {
        format!(
            "geocentric={}; topocentric={}; apparent={}; mean={}; batch={}; native_sidereal={}",
            self.geocentric,
            self.topocentric,
            self.apparent,
            self.mean,
            self.batch,
            self.native_sidereal
        )
    }
}

impl fmt::Display for BackendCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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

impl BackendMetadata {
    /// Returns a compact one-line rendering of the backend metadata posture.
    pub fn summary_line(&self) -> String {
        format!(
            "id={}; version={}; family={}; family posture={}; accuracy={}; deterministic={}; offline={}; nominal range={}; time scales=[{}]; bodies=[{}]; frames=[{}]; capabilities=[{}]; provenance={}",
            self.id,
            self.version,
            self.family,
            self.family.posture_label(),
            self.accuracy,
            self.deterministic,
            self.offline,
            self.nominal_range,
            format_display_list(&self.supported_time_scales),
            format_display_list(&self.body_coverage),
            format_display_list(&self.supported_frames),
            self.capabilities.summary_line(),
            self.provenance.summary_line(),
        )
    }
}

impl fmt::Display for BackendMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Errors returned when backend metadata fails the shared consistency checks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendMetadataValidationError {
    /// A required metadata field is blank or whitespace-padded.
    BlankField { field: &'static str },
    /// A required list field is empty.
    EmptyField { field: &'static str },
    /// A catalog-style list field contains a duplicate entry.
    DuplicateEntry { field: &'static str, value: String },
    /// The nominal range contains a non-finite Julian-day bound.
    NominalRangeNotFinite,
    /// The nominal range bounds use different time scales.
    NominalRangeScaleMismatch,
    /// The nominal range end precedes the start.
    NominalRangeOutOfOrder,
}

impl BackendMetadataValidationError {
    /// Returns a compact validation summary string.
    pub fn summary_line(&self) -> String {
        match self {
            Self::BlankField { field } => {
                format!("backend metadata field `{field}` is blank or whitespace-padded")
            }
            Self::EmptyField { field } => {
                format!("backend metadata field `{field}` must not be empty")
            }
            Self::DuplicateEntry { field, value } => {
                format!("backend metadata field `{field}` contains duplicate entry `{value}`")
            }
            Self::NominalRangeNotFinite => {
                "backend metadata nominal range must use finite Julian-day bounds".to_owned()
            }
            Self::NominalRangeScaleMismatch => {
                "backend metadata nominal range bounds must use the same time scale".to_owned()
            }
            Self::NominalRangeOutOfOrder => {
                "backend metadata nominal range end must not precede the start".to_owned()
            }
        }
    }
}

impl fmt::Display for BackendMetadataValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for BackendMetadataValidationError {}

impl BackendMetadata {
    /// Returns `Ok(())` when the metadata is internally consistent.
    ///
    /// The shared check keeps the release-facing backend inventory from
    /// silently advertising blank identifiers, duplicate coverage entries, or
    /// an invalid nominal range. It does not attempt to validate source-specific
    /// accuracy claims; those still belong to the backend crate that owns the
    /// data.
    pub fn validate(&self) -> Result<(), BackendMetadataValidationError> {
        validate_non_blank("id", self.id.as_str())?;
        validate_non_blank("version", &self.version)?;
        validate_non_blank("provenance summary", &self.provenance.summary)?;
        validate_non_blank_entries("provenance data sources", &self.provenance.data_sources)?;
        validate_unique_entries("provenance data sources", &self.provenance.data_sources)?;
        validate_non_empty_unique("supported time scales", &self.supported_time_scales)?;
        validate_non_empty_unique("body coverage", &self.body_coverage)?;
        validate_non_empty_unique("supported frames", &self.supported_frames)?;
        self.validate_nominal_range()?;
        Ok(())
    }

    fn validate_nominal_range(&self) -> Result<(), BackendMetadataValidationError> {
        match (self.nominal_range.start, self.nominal_range.end) {
            (Some(start), Some(end)) => {
                if !start.julian_day.days().is_finite() || !end.julian_day.days().is_finite() {
                    return Err(BackendMetadataValidationError::NominalRangeNotFinite);
                }
                if start.scale != end.scale {
                    return Err(BackendMetadataValidationError::NominalRangeScaleMismatch);
                }
                if start.julian_day.days() > end.julian_day.days() {
                    return Err(BackendMetadataValidationError::NominalRangeOutOfOrder);
                }
            }
            (Some(start), None) => {
                if !start.julian_day.days().is_finite() {
                    return Err(BackendMetadataValidationError::NominalRangeNotFinite);
                }
            }
            (None, Some(end)) => {
                if !end.julian_day.days().is_finite() {
                    return Err(BackendMetadataValidationError::NominalRangeNotFinite);
                }
            }
            (None, None) => {}
        }

        Ok(())
    }
}

fn validate_non_blank(
    field: &'static str,
    value: &str,
) -> Result<(), BackendMetadataValidationError> {
    if value.trim().is_empty() || value.trim() != value {
        Err(BackendMetadataValidationError::BlankField { field })
    } else {
        Ok(())
    }
}

fn validate_non_blank_entries<T: AsRef<str>>(
    field: &'static str,
    values: &[T],
) -> Result<(), BackendMetadataValidationError> {
    for value in values {
        let value = value.as_ref();
        if value.trim().is_empty() || value.trim() != value {
            return Err(BackendMetadataValidationError::BlankField { field });
        }
    }

    Ok(())
}

fn validate_unique_entries<T: fmt::Display + PartialEq>(
    field: &'static str,
    values: &[T],
) -> Result<(), BackendMetadataValidationError> {
    for (index, value) in values.iter().enumerate() {
        if values[..index].iter().any(|prior| prior == value) {
            return Err(BackendMetadataValidationError::DuplicateEntry {
                field,
                value: value.to_string(),
            });
        }
    }

    Ok(())
}

fn validate_non_empty_unique<T: fmt::Display + PartialEq>(
    field: &'static str,
    values: &[T],
) -> Result<(), BackendMetadataValidationError> {
    if values.is_empty() {
        return Err(BackendMetadataValidationError::EmptyField { field });
    }

    validate_unique_entries(field, values)
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
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::{Apparentness, EphemerisRequest};
    /// use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode};
    ///
    /// let request = EphemerisRequest::new(
    ///     CelestialBody::Mars,
    ///     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    /// );
    ///
    /// assert_eq!(request.frame, CoordinateFrame::Ecliptic);
    /// assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
    /// assert_eq!(request.apparent, Apparentness::Mean);
    /// assert!(request.observer.is_none());
    /// ```
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

    /// Returns a compact one-line rendering of the request shape.
    pub fn summary_line(&self) -> String {
        let observer = self
            .observer
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "geocentric".to_string());

        format!(
            "body={}; instant={}; frame={}; zodiac={}; apparent={}; observer={}",
            self.body, self.instant, self.frame, self.zodiac_mode, self.apparent, observer
        )
    }
}

impl fmt::Display for EphemerisRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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

impl QualityAnnotation {
    /// Returns a stable human-readable label for the quality annotation.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::Interpolated => "Interpolated",
            Self::Approximate => "Approximate",
            Self::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for QualityAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
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

impl fmt::Display for EphemerisErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            EphemerisErrorKind::UnsupportedBody => "UnsupportedBody",
            EphemerisErrorKind::UnsupportedCoordinateFrame => "UnsupportedCoordinateFrame",
            EphemerisErrorKind::UnsupportedTimeScale => "UnsupportedTimeScale",
            EphemerisErrorKind::InvalidObserver => "InvalidObserver",
            EphemerisErrorKind::OutOfRangeInstant => "OutOfRangeInstant",
            EphemerisErrorKind::MissingDataset => "MissingDataset",
            EphemerisErrorKind::NumericalFailure => "NumericalFailure",
            EphemerisErrorKind::InvalidRequest => "InvalidRequest",
        };
        f.write_str(label)
    }
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

    /// Returns a compact one-line rendering of the backend error.
    pub fn summary_line(&self) -> String {
        format!("{}: {}", self.kind, self.message)
    }
}

impl fmt::Display for EphemerisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for EphemerisError {}

fn format_display_list<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Compact summary of the current shared request-policy posture.
///
/// # Example
///
/// ```
/// use pleiades_backend::RequestPolicySummary;
///
/// let summary = RequestPolicySummary::current();
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert!(summary.summary_line().contains("time-scale="));
/// assert!(summary.summary_line().contains("observer="));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestPolicySummary {
    /// Time-scale policy wording.
    pub time_scale: &'static str,
    /// Observer policy wording.
    pub observer: &'static str,
    /// Apparentness policy wording.
    pub apparentness: &'static str,
    /// Frame policy wording.
    pub frame: &'static str,
}

impl RequestPolicySummary {
    /// Returns the current shared request-policy posture.
    pub const fn current() -> Self {
        current_request_policy_summary()
    }

    /// Returns a compact one-line rendering of the shared request-policy posture.
    pub fn summary_line(&self) -> String {
        format!(
            "time-scale={}; observer={}; apparentness={}; frame={}",
            self.time_scale, self.observer, self.apparentness, self.frame
        )
    }
}

impl fmt::Display for RequestPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for the shared request-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestPolicySummaryValidationError {
    /// A summary field is out of sync with the current request-policy posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for RequestPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the request-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for RequestPolicySummaryValidationError {}

impl RequestPolicySummary {
    /// Returns `Ok(())` when the shared request-policy wording still matches the current posture.
    pub fn validate(&self) -> Result<(), RequestPolicySummaryValidationError> {
        let current = current_request_policy_summary();
        for (field, value, expected) in [
            ("time_scale", self.time_scale, current.time_scale),
            ("observer", self.observer, current.observer),
            ("apparentness", self.apparentness, current.apparentness),
            ("frame", self.frame, current.frame),
        ] {
            if value != expected {
                return Err(RequestPolicySummaryValidationError::FieldOutOfSync { field });
            }
        }

        Ok(())
    }
}

/// Compact summary of a backend's frame-treatment posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameTreatmentSummary {
    summary: &'static str,
}

impl FrameTreatmentSummary {
    /// Creates a new frame-treatment summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the frame-treatment posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }
}

impl fmt::Display for FrameTreatmentSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Returns the current shared request-policy posture used by validation and reports.
pub const fn current_request_policy_summary() -> RequestPolicySummary {
    RequestPolicySummary {
        time_scale: "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T model",
        observer: "chart houses use observer locations; body requests stay geocentric; geocentric-only backends reject observer-bearing requests",
        apparentness: "current first-party backends accept mean geometric output only; apparent requests are rejected unless a backend explicitly advertises support",
        frame: "ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported",
    }
}

/// Returns the request-policy posture used by validation and release reporting.
pub const fn request_policy_summary_for_report() -> RequestPolicySummary {
    current_request_policy_summary()
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

/// Validates a direct backend request against the published backend metadata.
///
/// This convenience helper combines the shared request-shape checks with body
/// coverage, tropical-only zodiac routing for backends that do not advertise
/// native sidereal support, and topocentric capability validation. The shared
/// metadata model still does not capture per-ayanamsa sidereal catalog breadth,
/// so callers that need finer-grained sidereal routing must keep that logic at
/// the backend or façade layer.
pub fn validate_request_against_metadata(
    req: &EphemerisRequest,
    metadata: &BackendMetadata,
) -> Result<(), EphemerisError> {
    validate_request_policy(
        req,
        metadata.id.as_str(),
        &metadata.supported_time_scales,
        &metadata.supported_frames,
        metadata.capabilities.apparent,
    )?;

    if !metadata.capabilities.native_sidereal {
        validate_zodiac_policy(req, metadata.id.as_str(), &[ZodiacMode::Tropical])?;
    }

    validate_observer_policy(req, metadata.id.as_str(), metadata.capabilities.topocentric)?;

    if !metadata.body_coverage.contains(&req.body) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedBody,
            format!("{} does not support {}", metadata.id, req.body),
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
                format_display_list(supported_zodiac_modes)
            )
        };

        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            message,
        ));
    }

    Ok(())
}

/// Returns the compact report wording for the current time-scale policy.
pub const fn time_scale_policy_summary_for_report() -> &'static str {
    current_request_policy_summary().time_scale
}

/// Returns the compact report wording for the current observer policy.
pub const fn observer_policy_summary_for_report() -> &'static str {
    current_request_policy_summary().observer
}

/// Returns the compact report wording for the current apparentness policy.
pub const fn apparentness_policy_summary_for_report() -> &'static str {
    current_request_policy_summary().apparentness
}

/// Returns the compact report wording for the current frame policy.
pub const fn frame_policy_summary_for_report() -> &'static str {
    current_request_policy_summary().frame
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
/// structured [`EphemerisErrorKind::InvalidObserver`] error.
pub fn validate_observer_policy(
    req: &EphemerisRequest,
    backend_label: &str,
    supports_topocentric: bool,
) -> Result<(), EphemerisError> {
    if !supports_topocentric {
        if let Some(observer) = req.observer.as_ref() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidObserver,
                format!(
                    "{backend_label} is geocentric only; topocentric positions are not implemented for {}",
                    observer.summary_line()
                ),
            ));
        }
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

        assert!(BackendFamily::ReferenceData.is_data_backed());
        assert!(BackendFamily::CompressedData.is_data_backed());
        assert!(!BackendFamily::Algorithmic.is_data_backed());
        assert!(BackendFamily::Algorithmic.is_algorithmic());
        assert!(BackendFamily::Composite.is_routing());
        assert_eq!(
            BackendFamily::Algorithmic.posture().to_string(),
            "algorithmic"
        );
        assert_eq!(
            BackendFamily::ReferenceData.posture().to_string(),
            "data-backed"
        );
        assert_eq!(
            BackendFamily::CompressedData.posture().to_string(),
            "data-backed"
        );
        assert_eq!(BackendFamily::Composite.posture().to_string(), "routing");
        assert_eq!(
            BackendFamily::Other("custom".to_string())
                .posture()
                .to_string(),
            "other"
        );
        assert_eq!(BackendFamily::Algorithmic.posture_label(), "algorithmic");
        assert_eq!(BackendFamily::ReferenceData.posture_label(), "data-backed");
        assert_eq!(BackendFamily::CompressedData.posture_label(), "data-backed");
        assert_eq!(BackendFamily::Composite.posture_label(), "routing");
        assert_eq!(
            BackendFamily::Other("custom".to_string()).posture_label(),
            "other"
        );

        assert_eq!(AccuracyClass::Exact.to_string(), "Exact");
        assert_eq!(AccuracyClass::High.to_string(), "High");
        assert_eq!(AccuracyClass::Moderate.to_string(), "Moderate");
        assert_eq!(AccuracyClass::Approximate.to_string(), "Approximate");
        assert_eq!(AccuracyClass::Unknown.to_string(), "Unknown");

        assert_eq!(QualityAnnotation::Exact.to_string(), "Exact");
        assert_eq!(QualityAnnotation::Interpolated.to_string(), "Interpolated");
        assert_eq!(QualityAnnotation::Approximate.to_string(), "Approximate");
        assert_eq!(QualityAnnotation::Unknown.to_string(), "Unknown");

        assert_eq!(
            EphemerisErrorKind::UnsupportedBody.to_string(),
            "UnsupportedBody"
        );
        assert_eq!(
            EphemerisErrorKind::UnsupportedCoordinateFrame.to_string(),
            "UnsupportedCoordinateFrame"
        );
        assert_eq!(
            EphemerisErrorKind::UnsupportedTimeScale.to_string(),
            "UnsupportedTimeScale"
        );
        assert_eq!(
            EphemerisErrorKind::InvalidObserver.to_string(),
            "InvalidObserver"
        );
        assert_eq!(
            EphemerisErrorKind::OutOfRangeInstant.to_string(),
            "OutOfRangeInstant"
        );
        assert_eq!(
            EphemerisErrorKind::MissingDataset.to_string(),
            "MissingDataset"
        );
        assert_eq!(
            EphemerisErrorKind::NumericalFailure.to_string(),
            "NumericalFailure"
        );
        assert_eq!(
            EphemerisErrorKind::InvalidRequest.to_string(),
            "InvalidRequest"
        );

        let error = EphemerisError::new(EphemerisErrorKind::InvalidRequest, "example failure");
        assert_eq!(error.summary_line(), "InvalidRequest: example failure");
        assert_eq!(error.to_string(), error.summary_line());
    }

    #[test]
    fn request_policy_summary_has_a_compact_display() {
        let summary = RequestPolicySummary::current();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            "time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T model; observer=chart houses use observer locations; body requests stay geocentric; geocentric-only backends reject observer-bearing requests; apparentness=current first-party backends accept mean geometric output only; apparent requests are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported"
        );
        assert!(summary.summary_line().contains("time-scale="));
        assert!(summary.summary_line().contains("observer="));
        assert!(summary.summary_line().contains("apparentness="));
        assert!(summary.summary_line().contains("frame="));
        assert!(summary.validate().is_ok());
    }

    #[test]
    fn request_policy_summary_validate_rejects_blank_fields() {
        let mut summary = RequestPolicySummary::current();
        summary.frame = " ";

        let error = summary
            .validate()
            .expect_err("blank policy prose should fail validation");
        assert_eq!(
            error.to_string(),
            "the request-policy summary field `frame` is out of sync with the current posture"
        );
    }

    #[test]
    fn backend_metadata_has_a_compact_display() {
        let metadata = BackendMetadata {
            id: BackendId::new("toy"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("example backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
            supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };

        assert_eq!(metadata.to_string(), metadata.summary_line());
        assert!(metadata.summary_line().contains("id=toy"));
        assert!(metadata.summary_line().contains("version=0.1.0"));
        assert!(metadata.summary_line().contains("family=Algorithmic"));
        assert!(metadata
            .summary_line()
            .contains("family posture=algorithmic"));
        assert!(metadata.summary_line().contains("accuracy=Approximate"));
        assert!(metadata.summary_line().contains("deterministic=true"));
        assert!(metadata.summary_line().contains("offline=true"));
        assert!(metadata.summary_line().contains("time scales=[TT, TDB]"));
        assert!(metadata.summary_line().contains("bodies=[Sun, Moon]"));
        assert!(metadata
            .summary_line()
            .contains("frames=[Ecliptic, Equatorial]"));
        assert!(metadata.summary_line().contains("capabilities=["));
        assert!(metadata
            .summary_line()
            .contains("provenance=example backend"));
        assert!(metadata.validate().is_ok());
    }

    #[test]
    fn backend_metadata_validation_rejects_blank_and_duplicate_fields() {
        let mut metadata = BackendMetadata {
            id: BackendId::new(" "),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("example backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tt],
            body_coverage: vec![CelestialBody::Sun, CelestialBody::Sun],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };

        let error = metadata
            .validate()
            .expect_err("blank backend ids should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata field `id` is blank or whitespace-padded"
        );
        assert_eq!(error.to_string(), error.summary_line());

        metadata.id = BackendId::new("toy");
        metadata.supported_time_scales = vec![TimeScale::Tt];
        metadata.body_coverage = vec![CelestialBody::Sun];
        metadata.supported_frames = vec![CoordinateFrame::Ecliptic, CoordinateFrame::Ecliptic];

        let error = metadata
            .validate()
            .expect_err("duplicate supported frames should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata field `supported frames` contains duplicate entry `Ecliptic`"
        );
        assert_eq!(error.to_string(), error.summary_line());

        metadata.supported_frames = vec![CoordinateFrame::Ecliptic];
        metadata.provenance.data_sources = vec!["source A".to_string(), "source A".to_string()];

        let error = metadata
            .validate()
            .expect_err("duplicate provenance sources should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata field `provenance data sources` contains duplicate entry `source A`"
        );
        assert_eq!(error.to_string(), error.summary_line());

        metadata.provenance.data_sources = vec!["source A".to_string()];
        metadata.nominal_range = TimeRange::new(
            Some(Instant::new(
                JulianDay::from_days(2_451_546.0),
                TimeScale::Tt,
            )),
            Some(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            )),
        );

        let error = metadata
            .validate()
            .expect_err("out-of-order nominal ranges should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata nominal range end must not precede the start"
        );
        assert_eq!(error.to_string(), error.summary_line());

        metadata.nominal_range = TimeRange::new(
            Some(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            )),
            Some(Instant::new(
                JulianDay::from_days(2_451_546.0),
                TimeScale::Tdb,
            )),
        );

        let error = metadata
            .validate()
            .expect_err("mixed nominal-range scales should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata nominal range bounds must use the same time scale"
        );
        assert_eq!(error.to_string(), error.summary_line());
    }

    #[test]
    fn ephemeris_request_has_a_compact_display() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let request = EphemerisRequest::new(CelestialBody::Mars, instant);
        let request = EphemerisRequest {
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(51.5),
                Longitude::from_degrees(-0.1),
                None,
            )),
            ..request
        };

        assert_eq!(request.to_string(), request.summary_line());
        assert_eq!(
            request.summary_line(),
            "body=Mars; instant=JD 2451545 TT; frame=Ecliptic; zodiac=Tropical; apparent=Mean; observer=latitude=51.5°, longitude=359.9°, elevation=n/a"
        );
        assert!(request.summary_line().contains("body=Mars"));
        assert!(request.summary_line().contains("observer="));
    }

    #[test]
    fn backend_provenance_summary_has_a_compact_display() {
        let provenance = BackendProvenance {
            summary: "toy backend for tests".to_string(),
            data_sources: vec!["source A".to_string(), "source B".to_string()],
        };

        assert_eq!(provenance.to_string(), provenance.summary_line());
        assert_eq!(provenance.summary_line(), "toy backend for tests");
        assert!(provenance.summary_line().contains("toy backend for tests"));
    }

    #[test]
    fn backend_capabilities_summary_has_a_compact_display() {
        let capabilities = BackendCapabilities::default();

        assert_eq!(capabilities.to_string(), capabilities.summary_line());
        assert_eq!(
            capabilities.summary_line(),
            "geocentric=true; topocentric=false; apparent=true; mean=true; batch=true; native_sidereal=false"
        );
        assert!(capabilities.summary_line().contains("geocentric="));
        assert!(capabilities.summary_line().contains("topocentric="));
        assert!(capabilities.summary_line().contains("apparent="));
        assert!(capabilities.summary_line().contains("native_sidereal="));
    }

    #[test]
    fn frame_treatment_summary_has_a_compact_display() {
        let summary = FrameTreatmentSummary::new(
            "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform",
        );

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.summary_line(), "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform");
        assert!(summary.summary_line().contains("mean-obliquity"));
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
        assert_eq!(
            error.message,
            "toy backend expects one of [TT] for request instants"
        );

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
        assert_eq!(
            error.message,
            "toy backend only returns [Ecliptic] coordinates"
        );

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

        let metadata = BackendMetadata {
            id: BackendId::new("toy backend"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("toy backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![CelestialBody::Sun],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };

        let frame_error = validate_request_against_metadata(&frame_request, &metadata)
            .expect_err("equatorial requests should still be rejected through metadata preflight");
        assert_eq!(
            frame_error.kind,
            EphemerisErrorKind::UnsupportedCoordinateFrame
        );
        assert_eq!(
            frame_error.message,
            "toy backend only returns [Ecliptic] coordinates"
        );

        let topocentric_request = EphemerisRequest {
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(51.5),
                Longitude::from_degrees(-0.1),
                Some(45.0),
            )),
            apparent: Apparentness::Mean,
            ..apparent_request.clone()
        };
        let error = validate_request_against_metadata(&topocentric_request, &metadata)
            .expect_err("metadata preflight should reject observer-bearing geocentric requests");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error.message.contains("toy backend is geocentric only"));
        assert!(error.message.contains(
            &topocentric_request
                .observer
                .as_ref()
                .unwrap()
                .summary_line()
        ));

        let unsupported_body_request = EphemerisRequest {
            body: CelestialBody::Mars,
            frame: CoordinateFrame::Ecliptic,
            apparent: Apparentness::Mean,
            observer: None,
            ..frame_request.clone()
        };
        let error = validate_request_against_metadata(&unsupported_body_request, &metadata)
            .expect_err("metadata preflight should reject bodies outside the declared coverage");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        assert_eq!(error.message, "toy backend does not support Mars");

        let sidereal_request = EphemerisRequest {
            zodiac_mode: ZodiacMode::Sidereal {
                ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
            },
            frame: CoordinateFrame::Ecliptic,
            apparent: Apparentness::Mean,
            observer: None,
            ..frame_request.clone()
        };
        let error = validate_request_against_metadata(&sidereal_request, &metadata)
            .expect_err("sidereal requests should be rejected when metadata stays tropical-only");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("tropical coordinates only"));

        let sidereal_metadata = BackendMetadata {
            capabilities: BackendCapabilities {
                native_sidereal: true,
                ..metadata.capabilities.clone()
            },
            ..metadata.clone()
        };
        assert!(validate_request_against_metadata(&sidereal_request, &sidereal_metadata).is_ok());

        let error =
            validate_zodiac_policy(&sidereal_request, "toy backend", &[ZodiacMode::Tropical])
                .expect_err(
                    "sidereal requests should be rejected when only tropical output is supported",
                );
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("tropical coordinates only"));
        let request_policy = current_request_policy_summary();
        assert_eq!(
            request_policy.time_scale,
            "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T model"
        );
        assert_eq!(
            request_policy.observer,
            "chart houses use observer locations; body requests stay geocentric; geocentric-only backends reject observer-bearing requests"
        );
        assert_eq!(
            request_policy.apparentness,
            "current first-party backends accept mean geometric output only; apparent requests are rejected unless a backend explicitly advertises support"
        );
        assert_eq!(
            request_policy.frame,
            "ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported"
        );
        assert_eq!(
            time_scale_policy_summary_for_report(),
            request_policy.time_scale
        );
        assert_eq!(
            observer_policy_summary_for_report(),
            request_policy.observer
        );
        assert_eq!(
            apparentness_policy_summary_for_report(),
            request_policy.apparentness
        );
        assert_eq!(frame_policy_summary_for_report(), request_policy.frame);
        assert_eq!(
            zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]),
            "tropical only"
        );

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
        assert!(error.message.contains("toy backend is geocentric only"));
        assert!(error
            .message
            .contains(&observer_request.observer.as_ref().unwrap().summary_line()));
    }

    #[test]
    fn request_policy_summary_validation_rejects_stale_field_text() {
        let mut summary = current_request_policy_summary();
        summary.time_scale =
            "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers";

        let error = summary
            .validate()
            .expect_err("stale request-policy wording should fail validation");

        assert_eq!(
            error,
            RequestPolicySummaryValidationError::FieldOutOfSync {
                field: "time_scale"
            }
        );
        assert_eq!(
            error.to_string(),
            "the request-policy summary field `time_scale` is out of sync with the current posture"
        );
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

    #[test]
    fn batch_query_preserves_observer_request_rejection() {
        struct GeocentricOnlyBackend {
            calls: AtomicUsize,
        }

        impl EphemerisBackend for GeocentricOnlyBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("geocentric-only"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("geocentric-only test backend"),
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

                validate_observer_policy(req, "geocentric-only test backend", false)?;

                Ok(EphemerisResult::new(
                    BackendId::new("geocentric-only"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let backend = GeocentricOnlyBackend {
            calls: AtomicUsize::new(0),
        };
        let geocentric_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );
        let topocentric_request = EphemerisRequest {
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(51.5),
                Longitude::from_degrees(-0.1),
                Some(45.0),
            )),
            ..geocentric_request.clone()
        };

        let error = backend
            .positions(&[geocentric_request, topocentric_request])
            .expect_err("batch requests should preserve observer rejections");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
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
