//! House-calculation error types.

use core::fmt;

/// Error categories for house calculations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum HouseErrorKind {
    /// The selected house system is catalogued but not yet implemented.
    UnsupportedHouseSystem,
    /// The observer latitude is outside the mathematically valid range.
    InvalidLatitude,
    /// The observer longitude was not finite.
    InvalidLongitude,
    /// The observer elevation was not finite when a topocentric correction was requested.
    InvalidElevation,
    /// The supplied obliquity override was not finite.
    InvalidObliquity,
    /// The calculation failed for a numerical reason.
    NumericalFailure,
}

impl fmt::Display for HouseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::UnsupportedHouseSystem => "UnsupportedHouseSystem",
            Self::InvalidLatitude => "InvalidLatitude",
            Self::InvalidLongitude => "InvalidLongitude",
            Self::InvalidElevation => "InvalidElevation",
            Self::InvalidObliquity => "InvalidObliquity",
            Self::NumericalFailure => "NumericalFailure",
        };
        f.write_str(label)
    }
}

/// A structured house-calculation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseError {
    /// Error category.
    pub kind: HouseErrorKind,
    /// Human-readable message.
    pub message: String,
}

impl HouseError {
    /// Creates a new structured house error.
    pub fn new(kind: HouseErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for HouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for HouseError {}
