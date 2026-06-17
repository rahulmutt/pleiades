//! Broad public-data reader for arbitrary external JPL-style inputs.
//!
//! See `docs/superpowers/specs/2026-06-16-public-data-reader-design.md`.

pub mod ir;

pub use ir::{RawCorpus, RawEphemerisRecord, RawManifest};
