//! Broad public-data reader for arbitrary external JPL-style inputs.
//!
//! See `docs/superpowers/specs/2026-06-16-public-data-reader-design.md`.

pub mod detect;
pub mod error;
pub mod ir;
pub(crate) mod normalize;
pub mod profile;

pub use error::{Attribute, IngestError};
pub use ir::{RawCorpus, RawEphemerisRecord, RawManifest};
pub use profile::{Center, ExpectedProfile, IngestProvenance, Provenance, Units};
