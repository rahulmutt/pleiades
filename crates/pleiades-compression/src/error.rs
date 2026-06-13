//! Compression error types.

use core::fmt;

/// Error categories for compression and artifact parsing.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CompressionErrorKind {
    /// The artifact magic header did not match.
    InvalidMagic,
    /// The artifact version is not supported.
    UnsupportedVersion,
    /// The checksum did not match the payload.
    ChecksumMismatch,
    /// The payload ended unexpectedly.
    Truncated,
    /// The artifact contents were malformed.
    InvalidFormat,
    /// The artifact declared an unsupported byte-order policy.
    UnsupportedEndianPolicy,
    /// The requested body was not present.
    MissingBody,
    /// A required channel was absent.
    MissingChannel,
    /// The requested instant was outside the available segments.
    OutOfRangeInstant,
    /// The instant used an unsupported time scale.
    UnsupportedTimeScale,
    /// A coefficient exceeded the supported integer quantization range.
    QuantizationOverflow,
}

/// A structured compression error.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompressionError {
    /// Error category.
    pub kind: CompressionErrorKind,
    /// Human-readable explanation.
    pub message: String,
}

impl CompressionError {
    /// Creates a new compression error.
    pub fn new(kind: CompressionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for CompressionError {}
