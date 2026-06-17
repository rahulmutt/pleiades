//! Broad public-data reader for arbitrary external JPL-style inputs.
//!
//! See `docs/superpowers/specs/2026-06-16-public-data-reader-design.md`.

pub mod detect;
pub mod error;
pub mod format;
pub mod ir;
pub(crate) mod normalize;
pub mod profile;

use std::path::Path;

pub use detect::{detect_format, InputFormat};
pub use error::{Attribute, IngestError};
pub use ir::{RawCorpus, RawEphemerisRecord, RawManifest};
pub use profile::{Center, ExpectedProfile, IngestProvenance, Provenance, Units};

/// A successfully ingested external corpus plus its provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct PublicCorpus {
    /// The normalized corpus, ready for the existing validation/comparison surfaces.
    pub corpus: crate::backend::SnapshotCorpus,
    /// Per-attribute provenance (Read vs Asserted) and source labels.
    pub provenance: IngestProvenance,
}

/// Reads external bytes, auto-detecting the format, into a `PublicCorpus`.
pub fn read_public_corpus(
    bytes: &[u8],
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let format = detect::detect_format(bytes)?;
    read_public_corpus_as(bytes, format, expected)
}

/// Reads external bytes with an explicit format (bypassing detection).
pub fn read_public_corpus_as(
    bytes: &[u8],
    format: InputFormat,
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let text = String::from_utf8_lossy(bytes);
    let raw = format::parse_to_ir(format, &text)?;
    let (corpus, provenance) = normalize::normalize(raw, expected)?;
    Ok(PublicCorpus { corpus, provenance })
}

/// Reads an external corpus from a file path.
pub fn read_public_corpus_from_path(
    path: impl AsRef<Path>,
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let bytes = std::fs::read(path.as_ref()).map_err(|error| IngestError::Malformed {
        format: InputFormat::GenericCsv,
        line: 0,
        detail: format!("could not read {}: {error}", path.as_ref().display()),
    })?;
    read_public_corpus(&bytes, expected)
}
