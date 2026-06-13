use core::fmt;
use pleiades_types::CustomDefinitionValidationError;

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
    /// The request asks for topocentric observer support the backend does not implement.
    UnsupportedObserver,
    /// The instant lies outside the backend's nominal range.
    OutOfRangeInstant,
    /// Required data is missing.
    MissingDataset,
    /// The backend encountered a numerical failure.
    NumericalFailure,
    /// The request asks for a value mode the backend does not implement.
    UnsupportedApparentness,
    /// The request asks for a zodiac mode the backend does not implement.
    UnsupportedZodiacMode,
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
            EphemerisErrorKind::UnsupportedObserver => "UnsupportedObserver",
            EphemerisErrorKind::OutOfRangeInstant => "OutOfRangeInstant",
            EphemerisErrorKind::MissingDataset => "MissingDataset",
            EphemerisErrorKind::NumericalFailure => "NumericalFailure",
            EphemerisErrorKind::UnsupportedApparentness => "UnsupportedApparentness",
            EphemerisErrorKind::UnsupportedZodiacMode => "UnsupportedZodiacMode",
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

pub(crate) fn format_display_list<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn map_custom_definition_error(
    subject: &'static str,
    error: CustomDefinitionValidationError,
) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("{subject} is invalid: {error}"),
    )
}
