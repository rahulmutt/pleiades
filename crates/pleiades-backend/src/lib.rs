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
use std::time::Duration;

pub use pleiades_types::{
    Angle, Apparentness, Ayanamsa, CelestialBody, CoordinateFrame, CustomAyanamsa, CustomBodyId,
    CustomDefinitionValidationError, CustomHouseSystem, EclipticCoordinates, EquatorialCoordinates,
    HouseSystem, Instant, JulianDay, Latitude, Longitude, Motion, ObserverLocation, TimeRange,
    TimeRangeValidationError, TimeScale, TimeScaleConversion, TimeScaleConversionError, ZodiacMode,
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

    /// Returns `Ok(())` when the provenance summary is internally consistent.
    ///
    /// The shared check keeps backend provenance metadata from silently
    /// carrying blank summary text or duplicate/whitespace-padded source
    /// labels. Empty source lists are allowed for synthesized or routing
    /// backends that do not have external data provenance to list.
    pub fn validate(&self) -> Result<(), BackendProvenanceValidationError> {
        validate_non_blank("provenance summary", &self.summary)
            .map_err(|_| BackendProvenanceValidationError::BlankSummary)?;

        for (index, source) in self.data_sources.iter().enumerate() {
            if source.trim().is_empty() || source.trim() != source {
                return Err(BackendProvenanceValidationError::BlankDataSource { index });
            }
        }

        validate_unique_entries("provenance data sources", &self.data_sources).map_err(|error| {
            match error {
                BackendMetadataValidationError::DuplicateEntry { value, .. } => {
                    BackendProvenanceValidationError::DuplicateDataSource { value }
                }
                _ => {
                    unreachable!("duplicate provenance sources should only fail via DuplicateEntry")
                }
            }
        })
    }
}

/// Errors returned when backend provenance metadata fails the shared consistency checks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendProvenanceValidationError {
    /// The summary text was blank or whitespace-padded.
    BlankSummary,
    /// A provenance source entry was blank or whitespace-padded.
    BlankDataSource {
        /// Zero-based position of the invalid source entry.
        index: usize,
    },
    /// A provenance source entry appeared more than once.
    DuplicateDataSource {
        /// The duplicated source label.
        value: String,
    },
}

impl BackendProvenanceValidationError {
    /// Returns a compact validation summary string.
    pub fn summary_line(&self) -> String {
        match self {
            Self::BlankSummary => {
                "backend provenance summary must not be blank or whitespace-padded".to_owned()
            }
            Self::BlankDataSource { index } => format!(
                "backend provenance data source at index {index} must not be blank or whitespace-padded"
            ),
            Self::DuplicateDataSource { value } => {
                format!("backend provenance data sources contain duplicate entry `{value}`")
            }
        }
    }
}

impl fmt::Display for BackendProvenanceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for BackendProvenanceValidationError {}

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

    /// Returns `Ok(())` when the capability flags describe at least one usable
    /// position mode and one usable value mode.
    pub fn validate(&self) -> Result<(), BackendCapabilitiesValidationError> {
        if !self.geocentric && !self.topocentric {
            return Err(BackendCapabilitiesValidationError::MissingPositionMode);
        }

        if !self.apparent && !self.mean {
            return Err(BackendCapabilitiesValidationError::MissingValueMode);
        }

        Ok(())
    }
}

/// Errors returned when the declared backend capabilities cannot describe a usable request shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendCapabilitiesValidationError {
    /// Neither geocentric nor topocentric position support was declared.
    MissingPositionMode,
    /// Neither apparent nor mean output support was declared.
    MissingValueMode,
}

impl BackendCapabilitiesValidationError {
    /// Returns a compact validation summary string.
    pub fn summary_line(&self) -> &'static str {
        match self {
            Self::MissingPositionMode => {
                "backend capabilities must support geocentric or topocentric positions"
            }
            Self::MissingValueMode => "backend capabilities must support mean or apparent output",
        }
    }
}

impl fmt::Display for BackendCapabilitiesValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

impl std::error::Error for BackendCapabilitiesValidationError {}

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
    /// The declared capability flags are internally inconsistent.
    InvalidCapabilities {
        /// The invalid field name.
        field: &'static str,
        /// A short description of the capability mismatch.
        message: &'static str,
    },
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
            Self::InvalidCapabilities { field, message } => {
                format!("backend metadata field `{field}` is invalid: {message}")
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
        self.provenance.validate().map_err(|error| match error {
            BackendProvenanceValidationError::BlankSummary => {
                BackendMetadataValidationError::BlankField {
                    field: "provenance summary",
                }
            }
            BackendProvenanceValidationError::BlankDataSource { .. } => {
                BackendMetadataValidationError::BlankField {
                    field: "provenance data sources",
                }
            }
            BackendProvenanceValidationError::DuplicateDataSource { value } => {
                BackendMetadataValidationError::DuplicateEntry {
                    field: "provenance data sources",
                    value,
                }
            }
        })?;
        validate_non_empty_unique("supported time scales", &self.supported_time_scales)?;
        validate_non_empty_unique("body coverage", &self.body_coverage)?;
        validate_non_empty_unique("supported frames", &self.supported_frames)?;
        self.capabilities.validate().map_err(|error| match error {
            BackendCapabilitiesValidationError::MissingPositionMode => {
                BackendMetadataValidationError::InvalidCapabilities {
                    field: "capabilities",
                    message: error.summary_line(),
                }
            }
            BackendCapabilitiesValidationError::MissingValueMode => {
                BackendMetadataValidationError::InvalidCapabilities {
                    field: "capabilities",
                    message: error.summary_line(),
                }
            }
        })?;
        self.validate_nominal_range()?;
        Ok(())
    }

    fn validate_nominal_range(&self) -> Result<(), BackendMetadataValidationError> {
        match self.nominal_range.validate() {
            Ok(()) => Ok(()),
            Err(TimeRangeValidationError::NonFiniteBound { .. }) => {
                Err(BackendMetadataValidationError::NominalRangeNotFinite)
            }
            Err(TimeRangeValidationError::ScaleMismatch { .. }) => {
                Err(BackendMetadataValidationError::NominalRangeScaleMismatch)
            }
            Err(TimeRangeValidationError::OutOfOrder { .. }) => {
                Err(BackendMetadataValidationError::NominalRangeOutOfOrder)
            }
        }
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

    /// Validates any custom request identifiers embedded in this request.
    ///
    /// Built-in bodies and sidereal labels are always accepted. Custom bodies
    /// and custom ayanamsas are validated through their structured descriptor
    /// records so malformed user-defined entries fail before request dispatch.
    pub fn validate_custom_definitions(&self) -> Result<(), EphemerisError> {
        self.body
            .validate()
            .map_err(|error| map_custom_definition_error("request body", error))?;

        if let ZodiacMode::Sidereal { ayanamsa } = &self.zodiac_mode {
            ayanamsa
                .validate()
                .map_err(|error| map_custom_definition_error("sidereal ayanamsa", error))?;
        }

        Ok(())
    }

    /// Replaces the request instant with a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::with_time_scale_offset`].
    /// It preserves the rest of the request shape while letting direct backend
    /// callers stage explicit Delta T or TDB offsets before dispatch.
    pub fn with_instant_time_scale_offset(
        mut self,
        target_scale: TimeScale,
        offset_seconds: f64,
    ) -> Self {
        self.instant = self
            .instant
            .with_time_scale_offset(target_scale, offset_seconds);
        self
    }

    /// Replaces the request instant with a caller-supplied offset after validation.
    ///
    /// This is the checked counterpart to [`EphemerisRequest::with_instant_time_scale_offset`].
    /// It rejects non-finite offsets and mismatched source scales before the
    /// request is retagged, which keeps the backend-level convenience available
    /// in a release-grade form.
    pub fn with_instant_time_scale_offset_checked(
        mut self,
        target_scale: TimeScale,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .with_time_scale_offset_checked(target_scale, offset_seconds)?;
        Ok(self)
    }

    /// Applies a caller-supplied time-scale conversion policy to the request instant.
    ///
    /// This is the generic counterpart to the source-specific offset helpers.
    /// It keeps the explicit source, target, and offset choice available as a
    /// typed policy object while preserving the rest of the request shape.
    pub fn with_time_scale_conversion(
        mut self,
        conversion: TimeScaleConversion,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = conversion.apply(self.instant)?;
        Ok(self)
    }

    /// Converts the request instant from TT to TDB using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_tt`].
    pub fn with_tdb_from_tt(mut self, offset: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt(offset)?;
        Ok(self)
    }

    /// Converts the request instant from TT to TDB using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_tt_signed`].
    pub fn with_tdb_from_tt_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from TDB to TT using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_tdb`].
    pub fn with_tt_from_tdb(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb(offset_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from TDB to TT using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_tdb_signed`].
    pub fn with_tt_from_tdb_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TT using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_ut1`].
    pub fn with_tt_from_ut1(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1(delta_t)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TT using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_ut1_signed`].
    pub fn with_tt_from_ut1_signed(
        mut self,
        delta_t_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1_signed(delta_t_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from UTC to TT using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_utc`].
    pub fn with_tt_from_utc(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc(delta_t)?;
        Ok(self)
    }

    /// Converts the request instant from UTC to TT using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_utc_signed`].
    pub fn with_tt_from_utc_signed(
        mut self,
        delta_t_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc_signed(delta_t_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TDB using caller-supplied TT-UT1 and TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_ut1`].
    pub fn with_tdb_from_ut1(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_ut1(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TDB using caller-supplied TT-UT1 and signed TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_ut1_signed`].
    pub fn with_tdb_from_ut1_signed(
        mut self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .tdb_from_ut1_signed(tt_offset, tdb_offset_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from UTC to TDB using caller-supplied TT-UTC and TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_utc`].
    pub fn with_tdb_from_utc(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_utc(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the request instant from UTC to TDB using caller-supplied TT-UTC and signed TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_utc_signed`].
    pub fn with_tdb_from_utc_signed(
        mut self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .tdb_from_utc_signed(tt_offset, tdb_offset_seconds)?;
        Ok(self)
    }

    /// Validates a caller-supplied time-scale conversion policy without mutating the request.
    ///
    /// This mirrors [`TimeScaleConversion::validate`] at the backend-request
    /// layer so direct backend callers can preflight the explicit source/target/
    /// offset contract before choosing whether to apply it.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::EphemerisRequest;
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale, TimeScaleConversion};
    ///
    /// let request = EphemerisRequest::new(
    ///     CelestialBody::Sun,
    ///     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
    /// );
    /// let policy = TimeScaleConversion::new(TimeScale::Utc, TimeScale::Tt, 64.184);
    ///
    /// assert!(request.validate_time_scale_conversion(policy).is_ok());
    /// ```
    pub fn validate_time_scale_conversion(
        &self,
        conversion: TimeScaleConversion,
    ) -> Result<(), TimeScaleConversionError> {
        self.instant.validate_time_scale_conversion(conversion)
    }

    /// Validates this request against backend metadata.
    ///
    /// This is the method-form counterpart to [`validate_request_against_metadata`].
    /// It keeps direct backend callers from having to import the free helper when
    /// they want to preflight a request before dispatch.
    pub fn validate_against_metadata(
        &self,
        metadata: &BackendMetadata,
    ) -> Result<(), EphemerisError> {
        validate_request_against_metadata(self, metadata)
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

    /// Returns a compact one-line rendering of the backend result.
    ///
    /// The summary keeps the request-shape metadata alongside the available
    /// coordinate, motion, and quality fields so callers can compare a backend
    /// result without drilling into each optional channel manually.
    pub fn summary_line(&self) -> String {
        format!(
            "backend={}; body={}; instant={}; frame={}; zodiac={}; apparent={}; quality={}; ecliptic={}; equatorial={}; motion={}",
            self.backend_id,
            self.body,
            self.instant,
            self.frame,
            self.zodiac_mode,
            self.apparent,
            self.quality,
            format_optional_ecliptic_coordinates(self.ecliptic.as_ref()),
            format_optional_equatorial_coordinates(self.equatorial.as_ref()),
            format_optional_motion(self.motion.as_ref()),
        )
    }
}

impl fmt::Display for EphemerisResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn format_optional_ecliptic_coordinates(value: Option<&EclipticCoordinates>) -> String {
    value
        .map(|coordinates| {
            let distance = coordinates
                .distance_au
                .map(|distance| format!("{distance} AU"))
                .unwrap_or_else(|| "n/a".to_string());

            format!(
                "longitude={}, latitude={}, distance={}",
                coordinates.longitude, coordinates.latitude, distance
            )
        })
        .unwrap_or_else(|| "absent".to_string())
}

fn format_optional_equatorial_coordinates(value: Option<&EquatorialCoordinates>) -> String {
    value
        .map(|coordinates| {
            let distance = coordinates
                .distance_au
                .map(|distance| format!("{distance} AU"))
                .unwrap_or_else(|| "n/a".to_string());

            format!(
                "right_ascension={}, declination={}, distance={}",
                coordinates.right_ascension, coordinates.declination, distance
            )
        })
        .unwrap_or_else(|| "absent".to_string())
}

fn format_optional_motion(value: Option<&Motion>) -> String {
    value
        .map(|motion| {
            let longitude_speed = motion
                .longitude_deg_per_day
                .map(|speed| format!("{speed} deg/day"))
                .unwrap_or_else(|| "n/a".to_string());
            let latitude_speed = motion
                .latitude_deg_per_day
                .map(|speed| format!("{speed} deg/day"))
                .unwrap_or_else(|| "n/a".to_string());
            let distance_speed = motion
                .distance_au_per_day
                .map(|speed| format!("{speed} AU/day"))
                .unwrap_or_else(|| "n/a".to_string());

            format!(
                "longitude_speed={}, latitude_speed={}, distance_speed={}",
                longitude_speed, latitude_speed, distance_speed
            )
        })
        .unwrap_or_else(|| "absent".to_string())
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

fn map_custom_definition_error(
    subject: &'static str,
    error: CustomDefinitionValidationError,
) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("{subject} is invalid: {error}"),
    )
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

/// Validation error for a frame-treatment summary that drifted away from a compact release-facing line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameTreatmentSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
}

impl fmt::Display for FrameTreatmentSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("frame-treatment summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("frame-treatment summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("frame-treatment summary contains a line break"),
        }
    }
}

impl std::error::Error for FrameTreatmentSummaryValidationError {}

impl FrameTreatmentSummary {
    /// Creates a new frame-treatment summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the frame-treatment posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns `Ok(())` when the summary still contains a compact canonical line.
    pub fn validate(&self) -> Result<(), FrameTreatmentSummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(FrameTreatmentSummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(FrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(FrameTreatmentSummaryValidationError::EmbeddedLineBreak)
        } else {
            Ok(())
        }
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
/// This convenience helper combines the shared request-shape checks with custom
/// body and sidereal descriptor validation, body coverage, tropical-only zodiac
/// routing for backends that do not advertise native sidereal support, and
/// topocentric capability validation. The shared metadata model still does not
/// capture per-ayanamsa sidereal catalog breadth, so callers that need
/// finer-grained sidereal routing must keep that logic at the backend or façade
/// layer. Routing backends are treated as a special case: they still preflight
/// custom definitions and body coverage here, but they defer the broader
/// time-scale, frame, zodiac, apparentness, and observer checks to the selected
/// provider because their aggregate metadata is intentionally conservative.
pub fn validate_request_against_metadata(
    req: &EphemerisRequest,
    metadata: &BackendMetadata,
) -> Result<(), EphemerisError> {
    req.validate_custom_definitions()?;

    if !metadata.family.is_routing() {
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
    }

    if !metadata.body_coverage.contains(&req.body) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedBody,
            format!("{} does not support {}", metadata.id, req.body),
        ));
    }

    Ok(())
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
///     BackendId, BackendMetadata, BackendProvenance, EphemerisErrorKind, EphemerisRequest,
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
///     body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
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
/// assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
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
        if let Err(error) = validate_request_against_metadata(req, metadata) {
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
    ///
    /// The default adapter calls [`Self::position`] for each request in order and
    /// preserves each request's own instant and time-scale label exactly as
    /// supplied, so mixed TT/TDB batches remain mixed in the returned results
    /// instead of being normalized to a batch-wide scale.
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
    use std::time::Duration;

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
        metadata.provenance.summary = " ".to_string();

        let error = metadata
            .validate()
            .expect_err("blank provenance summaries should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata field `provenance summary` is blank or whitespace-padded"
        );
        assert_eq!(error.to_string(), error.summary_line());

        metadata.provenance.summary = "example backend".to_string();
        metadata.provenance.data_sources = vec![" source A".to_string()];

        let error = metadata
            .validate()
            .expect_err("whitespace-padded provenance sources should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata field `provenance data sources` is blank or whitespace-padded"
        );
        assert_eq!(error.to_string(), error.summary_line());

        metadata.provenance.data_sources = vec!["source A".to_string()];
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

        metadata.nominal_range = TimeRange::new(
            Some(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            )),
            Some(Instant::new(
                JulianDay::from_days(2_451_546.0),
                TimeScale::Tt,
            )),
        );
        metadata.capabilities = BackendCapabilities {
            geocentric: false,
            topocentric: false,
            apparent: false,
            mean: false,
            batch: true,
            native_sidereal: false,
        };

        let error = metadata
            .validate()
            .expect_err("capability flags without a position or value mode should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata field `capabilities` is invalid: backend capabilities must support geocentric or topocentric positions"
        );
        assert_eq!(error.to_string(), error.summary_line());

        metadata.capabilities = BackendCapabilities::default();
        metadata.nominal_range = TimeRange::new(
            Some(Instant::new(
                JulianDay::from_days(f64::INFINITY),
                TimeScale::Tt,
            )),
            Some(Instant::new(
                JulianDay::from_days(2_451_546.0),
                TimeScale::Tt,
            )),
        );

        let error = metadata
            .validate()
            .expect_err("non-finite nominal-range bounds should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend metadata nominal range must use finite Julian-day bounds"
        );
        assert_eq!(error.to_string(), error.summary_line());
    }

    #[test]
    fn backend_capabilities_validation_rejects_missing_position_or_value_modes() {
        let mut capabilities = BackendCapabilities::default();
        assert!(capabilities.validate().is_ok());

        capabilities.geocentric = false;
        capabilities.topocentric = false;
        let error = capabilities
            .validate()
            .expect_err("capabilities without a position mode should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend capabilities must support geocentric or topocentric positions"
        );
        assert_eq!(error.to_string(), error.summary_line());

        capabilities.geocentric = true;
        capabilities.topocentric = false;
        capabilities.apparent = false;
        capabilities.mean = false;
        let error = capabilities
            .validate()
            .expect_err("capabilities without a value mode should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend capabilities must support mean or apparent output"
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
    fn ephemeris_result_has_a_compact_display() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let mut result = EphemerisResult::new(
            BackendId::new("toy"),
            CelestialBody::Sun,
            instant,
            CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Mean,
        );
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(12.5),
            Latitude::from_degrees(-3.25),
            Some(1.234),
        ));
        result.equatorial = Some(EquatorialCoordinates::new(
            Angle::from_degrees(98.0),
            Latitude::from_degrees(0.5),
            None,
        ));
        result.motion = Some(Motion::new(Some(0.1), Some(-0.2), Some(0.003)));
        result.quality = QualityAnnotation::Exact;

        assert_eq!(result.to_string(), result.summary_line());
        assert_eq!(
            result.summary_line(),
            "backend=toy; body=Sun; instant=JD 2451545 TT; frame=Ecliptic; zodiac=Tropical; apparent=Mean; quality=Exact; ecliptic=longitude=12.5°, latitude=-3.25°, distance=1.234 AU; equatorial=right_ascension=98°, declination=0.5°, distance=n/a; motion=longitude_speed=0.1 deg/day, latitude_speed=-0.2 deg/day, distance_speed=0.003 AU/day"
        );
        assert!(result.summary_line().contains("backend=toy"));
        assert!(result.summary_line().contains("quality=Exact"));
        assert!(result.summary_line().contains("ecliptic=longitude=12.5°"));
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
        assert!(provenance.validate().is_ok());
    }

    #[test]
    fn backend_provenance_validation_rejects_blank_summary_and_duplicate_sources() {
        let mut provenance = BackendProvenance {
            summary: " ".to_string(),
            data_sources: vec!["source A".to_string(), "source A".to_string()],
        };

        let error = provenance
            .validate()
            .expect_err("blank provenance summaries should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend provenance summary must not be blank or whitespace-padded"
        );
        assert_eq!(error.to_string(), error.summary_line());

        provenance.summary = "toy backend".to_string();
        provenance.data_sources = vec![" source A".to_string()];

        let error = provenance
            .validate()
            .expect_err("whitespace-padded provenance sources should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend provenance data source at index 0 must not be blank or whitespace-padded"
        );
        assert_eq!(error.to_string(), error.summary_line());

        provenance.data_sources = vec!["source A".to_string(), "source A".to_string()];

        let error = provenance
            .validate()
            .expect_err("duplicate provenance sources should fail validation");
        assert_eq!(
            error.summary_line(),
            "backend provenance data sources contain duplicate entry `source A`"
        );
        assert_eq!(error.to_string(), error.summary_line());
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
        assert_eq!(summary.validate(), Ok(()));
        assert!(summary.summary_line().contains("mean-obliquity"));
    }

    #[test]
    fn frame_treatment_summary_rejects_blank_summary_text() {
        let summary = FrameTreatmentSummary::new("   ");

        assert_eq!(
            summary.validate(),
            Err(FrameTreatmentSummaryValidationError::BlankSummary)
        );
    }

    #[test]
    fn frame_treatment_summary_rejects_whitespace_padded_summary_text() {
        let summary = FrameTreatmentSummary::new(
            " geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform ",
        );

        assert_eq!(
            summary.validate(),
            Err(FrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
        );
    }

    #[test]
    fn frame_treatment_summary_rejects_embedded_line_breaks() {
        let summary = FrameTreatmentSummary::new(
            "geocentric ecliptic inputs;\nequatorial coordinates are derived with a mean-obliquity transform",
        );

        assert_eq!(
            summary.validate(),
            Err(FrameTreatmentSummaryValidationError::EmbeddedLineBreak)
        );
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
        let conversion = TimeScaleConversion::new(TimeScale::Utc, TimeScale::Tt, 64.184);
        assert!(time_scale_request
            .validate_time_scale_conversion(conversion)
            .is_ok());
        let converted = time_scale_request
            .clone()
            .with_time_scale_conversion(conversion)
            .expect("UTC request should convert with the caller-supplied policy");
        assert_eq!(converted.instant.scale, TimeScale::Tt);
        assert_eq!(converted.body, time_scale_request.body);
        assert_eq!(converted.observer, time_scale_request.observer);
        assert_eq!(converted.frame, time_scale_request.frame);
        assert_eq!(converted.zodiac_mode, time_scale_request.zodiac_mode);
        assert_eq!(converted.apparent, time_scale_request.apparent);

        let checked_offset = time_scale_request
            .clone()
            .with_instant_time_scale_offset_checked(TimeScale::Tt, 64.184)
            .expect("UTC request should accept the checked offset helper");
        assert_eq!(checked_offset.instant.scale, TimeScale::Tt);
        assert_eq!(checked_offset.body, time_scale_request.body);
        assert_eq!(checked_offset.observer, time_scale_request.observer);
        assert_eq!(checked_offset.frame, time_scale_request.frame);
        assert_eq!(checked_offset.zodiac_mode, time_scale_request.zodiac_mode);
        assert_eq!(checked_offset.apparent, time_scale_request.apparent);

        let tt_from_tdb_request = EphemerisRequest {
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
            ..time_scale_request.clone()
        };
        let tt_from_tdb = tt_from_tdb_request
            .clone()
            .with_tt_from_tdb(-0.001_657)
            .expect("TDB request should convert back to TT with a caller-supplied offset");
        assert_eq!(tt_from_tdb.instant.scale, TimeScale::Tt);
        assert_eq!(tt_from_tdb.body, tt_from_tdb_request.body);
        assert_eq!(tt_from_tdb.observer, tt_from_tdb_request.observer);
        assert_eq!(tt_from_tdb.frame, tt_from_tdb_request.frame);
        assert_eq!(tt_from_tdb.zodiac_mode, tt_from_tdb_request.zodiac_mode);
        assert_eq!(tt_from_tdb.apparent, tt_from_tdb_request.apparent);

        let tt_from_tdb_signed = tt_from_tdb_request
            .clone()
            .with_tt_from_tdb_signed(-0.001_657)
            .expect("TDB request should convert back to TT with a signed offset");
        assert_eq!(tt_from_tdb_signed.instant.scale, TimeScale::Tt);
        assert_eq!(tt_from_tdb_signed.body, tt_from_tdb_request.body);
        assert_eq!(tt_from_tdb_signed.observer, tt_from_tdb_request.observer);
        assert_eq!(tt_from_tdb_signed.frame, tt_from_tdb_request.frame);
        assert_eq!(
            tt_from_tdb_signed.zodiac_mode,
            tt_from_tdb_request.zodiac_mode
        );
        assert_eq!(tt_from_tdb_signed.apparent, tt_from_tdb_request.apparent);

        let tt_from_tdb_unsigned = tt_from_tdb_request
            .clone()
            .with_tt_from_tdb(0.001_657)
            .expect("TDB request should convert back to TT with a duration offset");
        assert_eq!(tt_from_tdb_unsigned.instant.scale, TimeScale::Tt);
        assert_eq!(tt_from_tdb_unsigned.body, tt_from_tdb_request.body);
        assert_eq!(tt_from_tdb_unsigned.observer, tt_from_tdb_request.observer);
        assert_eq!(tt_from_tdb_unsigned.frame, tt_from_tdb_request.frame);
        assert_eq!(
            tt_from_tdb_unsigned.zodiac_mode,
            tt_from_tdb_request.zodiac_mode
        );
        assert_eq!(tt_from_tdb_unsigned.apparent, tt_from_tdb_request.apparent);

        let tt_from_ut1_request = EphemerisRequest {
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Ut1),
            ..time_scale_request.clone()
        };
        let tt_from_ut1 = tt_from_ut1_request
            .clone()
            .with_tt_from_ut1_signed(64.184)
            .expect("UT1 request should convert to TT with a signed offset");
        assert_eq!(tt_from_ut1.instant.scale, TimeScale::Tt);
        assert_eq!(tt_from_ut1.body, tt_from_ut1_request.body);
        assert_eq!(tt_from_ut1.observer, tt_from_ut1_request.observer);
        assert_eq!(tt_from_ut1.frame, tt_from_ut1_request.frame);
        assert_eq!(tt_from_ut1.zodiac_mode, tt_from_ut1_request.zodiac_mode);
        assert_eq!(tt_from_ut1.apparent, tt_from_ut1_request.apparent);

        let tt_from_ut1_unsigned = tt_from_ut1_request
            .clone()
            .with_tt_from_ut1(Duration::from_secs_f64(64.184))
            .expect("UT1 request should convert to TT with a duration offset");
        assert_eq!(tt_from_ut1_unsigned.instant.scale, TimeScale::Tt);
        assert_eq!(tt_from_ut1_unsigned.body, tt_from_ut1_request.body);
        assert_eq!(tt_from_ut1_unsigned.observer, tt_from_ut1_request.observer);
        assert_eq!(tt_from_ut1_unsigned.frame, tt_from_ut1_request.frame);
        assert_eq!(
            tt_from_ut1_unsigned.zodiac_mode,
            tt_from_ut1_request.zodiac_mode
        );
        assert_eq!(tt_from_ut1_unsigned.apparent, tt_from_ut1_request.apparent);

        let tt_from_utc_request = EphemerisRequest {
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
            ..time_scale_request.clone()
        };
        let tt_from_utc = tt_from_utc_request
            .clone()
            .with_tt_from_utc_signed(64.184)
            .expect("UTC request should convert to TT with a signed offset");
        assert_eq!(tt_from_utc.instant.scale, TimeScale::Tt);
        assert_eq!(tt_from_utc.body, tt_from_utc_request.body);
        assert_eq!(tt_from_utc.observer, tt_from_utc_request.observer);
        assert_eq!(tt_from_utc.frame, tt_from_utc_request.frame);
        assert_eq!(tt_from_utc.zodiac_mode, tt_from_utc_request.zodiac_mode);
        assert_eq!(tt_from_utc.apparent, tt_from_utc_request.apparent);

        let tdb_from_tt_request = EphemerisRequest {
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
            ..time_scale_request.clone()
        };
        let tdb_from_tt = tdb_from_tt_request
            .clone()
            .with_tdb_from_tt_signed(-0.001_657)
            .expect("TT request should convert to TDB with a signed offset");
        assert_eq!(tdb_from_tt.instant.scale, TimeScale::Tdb);
        assert_eq!(tdb_from_tt.body, tdb_from_tt_request.body);
        assert_eq!(tdb_from_tt.observer, tdb_from_tt_request.observer);
        assert_eq!(tdb_from_tt.frame, tdb_from_tt_request.frame);
        assert_eq!(tdb_from_tt.zodiac_mode, tdb_from_tt_request.zodiac_mode);
        assert_eq!(tdb_from_tt.apparent, tdb_from_tt_request.apparent);

        let tdb_from_tt_unsigned = tdb_from_tt_request
            .clone()
            .with_tdb_from_tt(Duration::from_secs_f64(0.001_657))
            .expect("TT request should convert to TDB with a duration offset");
        assert_eq!(tdb_from_tt_unsigned.instant.scale, TimeScale::Tdb);
        assert_eq!(tdb_from_tt_unsigned.body, tdb_from_tt_request.body);
        assert_eq!(tdb_from_tt_unsigned.observer, tdb_from_tt_request.observer);
        assert_eq!(tdb_from_tt_unsigned.frame, tdb_from_tt_request.frame);
        assert_eq!(
            tdb_from_tt_unsigned.zodiac_mode,
            tdb_from_tt_request.zodiac_mode
        );
        assert_eq!(tdb_from_tt_unsigned.apparent, tdb_from_tt_request.apparent);

        let tdb_from_ut1_request = EphemerisRequest {
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Ut1),
            ..time_scale_request.clone()
        };
        let tdb_from_ut1 = tdb_from_ut1_request
            .clone()
            .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
            .expect("UT1 request should convert to TDB with caller-supplied offsets");
        assert_eq!(tdb_from_ut1.instant.scale, TimeScale::Tdb);
        assert_eq!(tdb_from_ut1.body, tdb_from_ut1_request.body);
        assert_eq!(tdb_from_ut1.observer, tdb_from_ut1_request.observer);
        assert_eq!(tdb_from_ut1.frame, tdb_from_ut1_request.frame);
        assert_eq!(tdb_from_ut1.zodiac_mode, tdb_from_ut1_request.zodiac_mode);
        assert_eq!(tdb_from_ut1.apparent, tdb_from_ut1_request.apparent);

        let tdb_from_ut1_unsigned = tdb_from_ut1_request
            .clone()
            .with_tdb_from_ut1(
                Duration::from_secs_f64(64.184),
                Duration::from_secs_f64(0.001_657),
            )
            .expect("UT1 request should convert to TDB with duration offsets");
        assert_eq!(tdb_from_ut1_unsigned.instant.scale, TimeScale::Tdb);
        assert_eq!(tdb_from_ut1_unsigned.body, tdb_from_ut1_request.body);
        assert_eq!(
            tdb_from_ut1_unsigned.observer,
            tdb_from_ut1_request.observer
        );
        assert_eq!(tdb_from_ut1_unsigned.frame, tdb_from_ut1_request.frame);
        assert_eq!(
            tdb_from_ut1_unsigned.zodiac_mode,
            tdb_from_ut1_request.zodiac_mode
        );
        assert_eq!(
            tdb_from_ut1_unsigned.apparent,
            tdb_from_ut1_request.apparent
        );

        let tdb_from_utc_request = EphemerisRequest {
            instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
            ..time_scale_request.clone()
        };
        let tdb_from_utc = tdb_from_utc_request
            .clone()
            .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
            .expect("UTC request should convert to TDB with caller-supplied offsets");
        assert_eq!(tdb_from_utc.instant.scale, TimeScale::Tdb);
        assert_eq!(tdb_from_utc.body, tdb_from_utc_request.body);
        assert_eq!(tdb_from_utc.observer, tdb_from_utc_request.observer);
        assert_eq!(tdb_from_utc.frame, tdb_from_utc_request.frame);
        assert_eq!(tdb_from_utc.zodiac_mode, tdb_from_utc_request.zodiac_mode);
        assert_eq!(tdb_from_utc.apparent, tdb_from_utc_request.apparent);

        let tdb_from_utc_unsigned = tdb_from_utc_request
            .clone()
            .with_tdb_from_utc(
                Duration::from_secs_f64(64.184),
                Duration::from_secs_f64(0.001_657),
            )
            .expect("UTC request should convert to TDB with duration offsets");
        assert_eq!(tdb_from_utc_unsigned.instant.scale, TimeScale::Tdb);
        assert_eq!(tdb_from_utc_unsigned.body, tdb_from_utc_request.body);
        assert_eq!(
            tdb_from_utc_unsigned.observer,
            tdb_from_utc_request.observer
        );
        assert_eq!(tdb_from_utc_unsigned.frame, tdb_from_utc_request.frame);
        assert_eq!(
            tdb_from_utc_unsigned.zodiac_mode,
            tdb_from_utc_request.zodiac_mode
        );
        assert_eq!(
            tdb_from_utc_unsigned.apparent,
            tdb_from_utc_request.apparent
        );

        let error = time_scale_request
            .clone()
            .with_instant_time_scale_offset_checked(TimeScale::Tt, f64::NAN)
            .expect_err("non-finite offsets should be rejected at the request layer");
        assert_eq!(error, TimeScaleConversionError::NonFiniteOffset);

        let error = time_scale_request
            .validate_time_scale_conversion(TimeScaleConversion::new(
                TimeScale::Tt,
                TimeScale::Tt,
                64.184,
            ))
            .expect_err("mismatched source scales should fail validation before retagging");
        assert_eq!(
            error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Tt,
                actual: TimeScale::Utc
            }
        );
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

        let invalid_custom_body_request = EphemerisRequest {
            body: CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid",
                " 433-Eros ",
            )),
            ..frame_request.clone()
        };
        let invalid_custom_body_error = validate_request_against_metadata(
            &invalid_custom_body_request,
            &BackendMetadata {
                body_coverage: vec![CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                    "asteroid",
                    " 433-Eros ",
                ))],
                ..metadata.clone()
            },
        )
        .expect_err("custom body identifiers should validate before metadata dispatch");
        assert_eq!(
            invalid_custom_body_error.kind,
            EphemerisErrorKind::InvalidRequest
        );
        assert!(invalid_custom_body_error
            .message
            .contains("request body is invalid: custom body id designation must not have leading or trailing whitespace"));

        let invalid_custom_ayanamsa_request = EphemerisRequest {
            zodiac_mode: ZodiacMode::Sidereal {
                ayanamsa: pleiades_types::Ayanamsa::Custom(pleiades_types::CustomAyanamsa {
                    name: "  ".to_string(),
                    description: Some("local calibration".to_string()),
                    epoch: Some(pleiades_types::JulianDay::from_days(2451545.0)),
                    offset_degrees: Some(pleiades_types::Angle::from_degrees(24.0)),
                }),
            },
            ..frame_request.clone()
        };
        let invalid_custom_ayanamsa_error =
            validate_request_against_metadata(&invalid_custom_ayanamsa_request, &sidereal_metadata)
                .expect_err("custom ayanamsas should validate before sidereal request dispatch");
        assert_eq!(
            invalid_custom_ayanamsa_error.kind,
            EphemerisErrorKind::InvalidRequest
        );
        assert!(invalid_custom_ayanamsa_error
            .message
            .contains("sidereal ayanamsa is invalid: custom ayanamsa name must not be blank"));

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
        fn assert_field_out_of_sync(
            mut summary: RequestPolicySummary,
            field: &'static str,
            mutate: impl FnOnce(&mut RequestPolicySummary),
        ) {
            mutate(&mut summary);

            let error = summary
                .validate()
                .expect_err("stale request-policy wording should fail validation");

            assert_eq!(
                error,
                RequestPolicySummaryValidationError::FieldOutOfSync { field }
            );
            assert_eq!(
                error.to_string(),
                format!(
                    "the request-policy summary field `{field}` is out of sync with the current posture"
                )
            );
        }

        let current = current_request_policy_summary();

        assert_field_out_of_sync(current, "time_scale", |summary| {
            summary.time_scale =
                "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers";
        });
        assert_field_out_of_sync(current, "observer", |summary| {
            summary.observer = "chart houses use observer locations; body requests stay geocentric";
        });
        assert_field_out_of_sync(current, "apparentness", |summary| {
            summary.apparentness = "current first-party backends accept mean geometric output only";
        });
        assert_field_out_of_sync(current, "frame", |summary| {
            summary.frame = "ecliptic body positions are the default request shape";
        });
    }

    #[test]
    fn validate_requests_against_metadata_rejects_unsupported_batch_backends() {
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
                batch: false,
                ..BackendCapabilities::default()
            },
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };
        let request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );

        let error = validate_requests_against_metadata(&[request], &metadata).expect_err(
            "batch requests should be rejected when the backend does not advertise batch support",
        );
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(error.message, "toy backend does not support batch requests");
    }

    #[test]
    fn routing_metadata_defers_request_shape_checks_to_the_selected_provider() {
        struct RejectingSunBackend;

        impl EphemerisBackend for RejectingSunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("rejecting-sun"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("rejecting Sun backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt],
                    body_coverage: vec![CelestialBody::Sun],
                    supported_frames: vec![CoordinateFrame::Ecliptic],
                    capabilities: BackendCapabilities {
                        batch: false,
                        apparent: false,
                        topocentric: false,
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

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                if req.observer.is_some() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidObserver,
                        "rejecting Sun backend is geocentric only",
                    ));
                }

                if req.apparent == Apparentness::Apparent {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        "rejecting Sun backend only returns mean geometric coordinates",
                    ));
                }

                Ok(EphemerisResult::new(
                    BackendId::new("rejecting-sun"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        struct AcceptingSunBackend;

        impl EphemerisBackend for AcceptingSunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("accepting-sun"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("accepting Sun backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt],
                    body_coverage: vec![CelestialBody::Sun],
                    supported_frames: vec![CoordinateFrame::Ecliptic],
                    capabilities: BackendCapabilities {
                        batch: true,
                        apparent: true,
                        topocentric: true,
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

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                Ok(EphemerisResult::new(
                    BackendId::new("accepting-sun"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let routing = RoutingBackend::new(vec![
            Box::new(RejectingSunBackend),
            Box::new(AcceptingSunBackend),
        ]);
        let mut request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        request.observer = Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(12.5),
            Some(0.0),
        ));
        request.apparent = Apparentness::Apparent;

        let metadata = routing.metadata();
        assert!(metadata.family.is_routing());
        assert!(!metadata.capabilities.batch);

        validate_requests_against_metadata(&[request.clone()], &metadata)
            .expect("routing metadata should defer request-shape checks to the selected provider");

        let result = routing
            .positions(&[request])
            .expect("routing should recover through the secondary provider");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].backend_id.as_str(), "accepting-sun");
    }

    #[test]
    fn routing_backend_batch_metadata_defers_observer_and_apparentness_checks_to_the_selected_provider(
    ) {
        struct RejectingSunBackend;

        impl EphemerisBackend for RejectingSunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("rejecting-sun-batch"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("rejecting Sun batch backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                    body_coverage: vec![CelestialBody::Sun],
                    supported_frames: vec![CoordinateFrame::Ecliptic],
                    capabilities: BackendCapabilities {
                        batch: false,
                        apparent: false,
                        topocentric: false,
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

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                if req.observer.is_some() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidObserver,
                        "rejecting Sun batch backend is geocentric only",
                    ));
                }

                if req.apparent == Apparentness::Apparent {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        "rejecting Sun batch backend only returns mean geometric coordinates",
                    ));
                }

                Ok(EphemerisResult::new(
                    BackendId::new("rejecting-sun-batch"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        struct AcceptingSunBackend;

        impl EphemerisBackend for AcceptingSunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("accepting-sun-batch"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("accepting Sun batch backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                    body_coverage: vec![CelestialBody::Sun],
                    supported_frames: vec![CoordinateFrame::Ecliptic],
                    capabilities: BackendCapabilities {
                        batch: true,
                        apparent: true,
                        topocentric: true,
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

            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                Ok(EphemerisResult::new(
                    BackendId::new("accepting-sun-batch"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let routing = RoutingBackend::new(vec![
            Box::new(RejectingSunBackend),
            Box::new(AcceptingSunBackend),
        ]);
        let mut geocentric_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        geocentric_request.observer = Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(12.5),
            Some(0.0),
        ));
        let mut apparent_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
        );
        apparent_request.apparent = Apparentness::Apparent;

        let metadata = routing.metadata();
        assert!(metadata.family.is_routing());
        assert!(!metadata.capabilities.batch);

        validate_requests_against_metadata(&[geocentric_request.clone(), apparent_request.clone()], &metadata)
            .expect("routing metadata should defer observer and apparentness checks to the selected provider");

        let results = routing
            .positions(&[geocentric_request, apparent_request])
            .expect("routing should recover through the secondary provider for batch requests");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].backend_id.as_str(), "accepting-sun-batch");
        assert_eq!(results[1].backend_id.as_str(), "accepting-sun-batch");
        assert_eq!(results[0].instant.scale, TimeScale::Tt);
        assert_eq!(results[1].instant.scale, TimeScale::Tdb);
    }

    #[test]
    fn validate_requests_against_metadata_rejects_sidereal_requests_with_batch_index_prefix() {
        let metadata = BackendMetadata {
            id: BackendId::new("toy backend"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("toy backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![CelestialBody::Sun],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };
        let tropical_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        let sidereal_request = EphemerisRequest {
            zodiac_mode: ZodiacMode::Sidereal {
                ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
            },
            ..tropical_request.clone()
        };

        let error =
            validate_requests_against_metadata(&[tropical_request, sidereal_request], &metadata)
                .expect_err("the batch helper should preserve sidereal request failures");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            error.message,
            "batch request 2: toy backend currently exposes tropical coordinates only"
        );
    }

    #[test]
    fn validate_requests_against_metadata_rejects_apparent_requests_with_batch_index_prefix() {
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
                apparent: false,
                ..BackendCapabilities::default()
            },
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };
        let mean_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        let apparent_request = EphemerisRequest {
            apparent: Apparentness::Apparent,
            ..mean_request.clone()
        };

        let error =
            validate_requests_against_metadata(&[mean_request, apparent_request], &metadata)
                .expect_err("the batch helper should preserve apparentness failures");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            error.message,
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );
    }

    #[test]
    fn validate_requests_against_metadata_rejects_topocentric_requests_with_batch_index_prefix() {
        let metadata = BackendMetadata {
            id: BackendId::new("toy backend"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("toy backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![CelestialBody::Sun],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };
        let geocentric_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        let topocentric_request = EphemerisRequest {
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(51.5),
                Longitude::from_degrees(-0.1),
                Some(45.0),
            )),
            ..geocentric_request.clone()
        };
        let topocentric_summary = topocentric_request
            .observer
            .as_ref()
            .expect("observer should be present")
            .summary_line();

        let error = validate_requests_against_metadata(
            &[geocentric_request, topocentric_request],
            &metadata,
        )
        .expect_err("the batch helper should preserve observer failures");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error
            .message
            .contains("batch request 2: toy backend is geocentric only"));
        assert!(error.message.contains(&topocentric_summary));
    }

    #[test]
    fn validate_requests_against_metadata_fails_fast_on_the_first_invalid_request() {
        let metadata = BackendMetadata {
            id: BackendId::new("toy backend"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("toy backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![CelestialBody::Sun],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        };
        let valid_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        let invalid_request = EphemerisRequest {
            body: CelestialBody::Mars,
            ..valid_request.clone()
        };

        let error =
            validate_requests_against_metadata(&[valid_request, invalid_request], &metadata)
                .expect_err("the batch helper should stop at the first unsupported body");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        assert_eq!(
            error.message,
            "batch request 2: toy backend does not support Mars"
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

    #[test]
    fn batch_query_preserves_mixed_time_scales() {
        struct MixedScaleBackend {
            calls: AtomicUsize,
        }

        impl EphemerisBackend for MixedScaleBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("mixed-scale"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("mixed-scale test backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
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
                    "mixed-scale test backend",
                    &[TimeScale::Tt, TimeScale::Tdb],
                    &[CoordinateFrame::Ecliptic],
                    false,
                )?;

                Ok(EphemerisResult::new(
                    BackendId::new("mixed-scale"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        let backend = MixedScaleBackend {
            calls: AtomicUsize::new(0),
        };
        let tt_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );
        let tdb_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
        );

        let results = backend
            .positions(&[tt_request, tdb_request])
            .expect("batch requests should preserve mixed time-scale labels");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].instant.scale, TimeScale::Tt);
        assert_eq!(results[1].instant.scale, TimeScale::Tdb);
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

    #[test]
    fn routing_backend_batch_positions_preserve_mixed_time_scales_after_fallback() {
        struct FailingSunBackend;

        impl EphemerisBackend for FailingSunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("fail-sun-batch"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("failing Sun batch backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
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
                    EphemerisErrorKind::InvalidRequest,
                    "retry with the next provider",
                ))
            }
        }

        struct RecoverySunBackend {
            calls: AtomicUsize,
        }

        impl EphemerisBackend for RecoverySunBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("recovery-sun-batch"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("recovery Sun batch backend"),
                    nominal_range: TimeRange::new(None, None),
                    supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
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
                Ok(EphemerisResult::new(
                    BackendId::new("recovery-sun-batch"),
                    req.body.clone(),
                    req.instant,
                    req.frame,
                    req.zodiac_mode.clone(),
                    req.apparent,
                ))
            }
        }

        struct MoonBackend;

        impl EphemerisBackend for MoonBackend {
            fn metadata(&self) -> BackendMetadata {
                BackendMetadata {
                    id: BackendId::new("moon-batch"),
                    version: "0.1.0".to_string(),
                    family: BackendFamily::Algorithmic,
                    provenance: BackendProvenance::new("moon batch backend"),
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
                    BackendId::new("moon-batch"),
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
            Box::new(RecoverySunBackend {
                calls: AtomicUsize::new(0),
            }),
            Box::new(MoonBackend),
        ]);
        let tt_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );
        let tdb_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
        );
        let moon_request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        );

        let results = routing
            .positions(&[tt_request.clone(), tdb_request.clone(), moon_request.clone()])
            .expect("routing should preserve mixed batch scales while falling back to the secondary provider");

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].backend_id.as_str(), "recovery-sun-batch");
        assert_eq!(results[1].backend_id.as_str(), "recovery-sun-batch");
        assert_eq!(results[2].backend_id.as_str(), "moon-batch");
        assert_eq!(results[0].instant.scale, TimeScale::Tt);
        assert_eq!(results[1].instant.scale, TimeScale::Tdb);
        assert_eq!(results[2].instant.scale, TimeScale::Tt);
    }
}
