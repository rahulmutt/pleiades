//! Pure-Rust reader for JPL DE binary SPK (`.bsp`) ephemeris kernels.
//!
//! Parses the DAF container and SPK segment types 2, 3, 1, and 21, evaluates
//! target-relative ICRF states, and reduces them to geocentric ecliptic
//! coordinates consistent with the rest of the workspace (mean geometric).

pub(crate) mod bytes;
pub(crate) mod chain;
pub(crate) mod daf;
pub(crate) mod pool;
pub(crate) mod segment;

#[cfg(test)]
pub(crate) mod test_support;

/// Error kinds for SPK kernel reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpkErrorKind {
    /// The bytes are too short for the structure being read.
    Truncated,
    /// The DAF identification word or layout was not recognised.
    BadHeader,
    /// The endianness marker was neither LTL-IEEE nor BIG-IEEE.
    UnknownEndianness,
    /// An SPK segment used a data type this reader does not implement.
    UnsupportedSegmentType,
    /// A requested epoch is outside every segment for the body.
    OutOfCoverage,
    /// No segment chain connects the body to the requested center.
    NoChain,
    /// A numerical failure occurred during state evaluation (e.g. a zero
    /// modified-difference stepsize in a Type 1 / Type 21 record).
    NumericalFailure,
    /// Underlying I/O failed.
    Io,
}

/// An SPK reading error with a human-readable message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpkError {
    /// The category of failure.
    pub kind: SpkErrorKind,
    /// A human-readable explanation.
    pub message: String,
}

impl SpkError {
    /// Builds a new error.
    pub fn new(kind: SpkErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }
}

/// Random-access byte source: a slice in tests, a buffered file in production.
pub trait ReadAt {
    /// Total length in bytes.
    fn len(&self) -> usize;
    /// Returns `len` bytes starting at `offset`, or `Truncated` if out of range.
    fn read_at(&self, offset: usize, len: usize) -> Result<&[u8], SpkError>;
    /// Convenience: true when empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ReadAt for [u8] {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
    fn read_at(&self, offset: usize, len: usize) -> Result<&[u8], SpkError> {
        self.get(offset..offset + len).ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::Truncated,
                format!("read of {len} bytes at {offset} exceeds slice length {}", self.len()),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_at_returns_subslice_and_truncation_error() {
        let data: &[u8] = &[1, 2, 3, 4];
        assert_eq!(data.read_at(1, 2).unwrap(), &[2, 3]);
        assert_eq!(data.read_at(2, 5).unwrap_err().kind, SpkErrorKind::Truncated);
    }
}
