//! Pure-Rust reader for JPL DE binary SPK (`.bsp`) ephemeris kernels.
//!
//! Parses the DAF container and SPK segment types 2, 3, 1, and 21, evaluates
//! target-relative ICRF states, and reduces them to geocentric ecliptic
//! coordinates consistent with the rest of the workspace (mean geometric).

pub mod asteroid_roster;
pub mod backend;
pub(crate) mod bytes;
pub(crate) mod chain;
pub mod corpus_manifest;
pub mod corpus_spec;
pub(crate) mod daf;
pub mod generate;
pub mod object_spk;
pub(crate) mod pool;
pub(crate) mod segment;

pub use backend::{SpkBackend, SpkBackendBuilder};
pub use generate::{
    build_manifest, generate_corpus_csv, generate_slice, regenerate_holdout_slice_csv,
    CorpusRequest, GeneratedSlice,
};

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod cross_check_tests;

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
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// Random-access byte source: a slice in tests, a buffered file in production.
pub trait ReadAt {
    /// Total length in bytes.
    fn len(&self) -> usize;
    /// Returns `len` bytes starting at `offset`, or `Truncated` if out of range.
    fn read_at(&self, offset: usize, len: usize) -> Result<&[u8], SpkError>;
    /// Convenience: true when empty.
    // Conventional companion to `len`; part of the trait's public shape even
    // when no current caller needs it.
    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ReadAt for [u8] {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
    fn read_at(&self, offset: usize, len: usize) -> Result<&[u8], SpkError> {
        let end = offset.checked_add(len).ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::Truncated,
                format!("read of {len} bytes at {offset} overflowed a usize"),
            )
        })?;
        self.get(offset..end).ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::Truncated,
                format!(
                    "read of {len} bytes at {offset} exceeds slice length {}",
                    <[u8]>::len(self)
                ),
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
        assert_eq!(
            data.read_at(2, 5).unwrap_err().kind,
            SpkErrorKind::Truncated
        );
    }

    #[test]
    fn read_at_rejects_offset_len_overflow_without_panicking() {
        let data: &[u8] = &[1, 2, 3, 4];
        let err = data
            .read_at(usize::MAX, 8)
            .expect_err("offset + len overflow must return Truncated, not panic");
        assert_eq!(err.kind, SpkErrorKind::Truncated);
    }
}
