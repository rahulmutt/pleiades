//! Compression codecs and artifact packing helpers for ephemeris data.
//!
//! The current implementation defines a small, deterministic artifact format
//! with explicit versioning, checksums, artifact capability profiles, and
//! quantized polynomial segments. Optional residual-correction channels are
//! also supported so denser artifacts can keep a compact base fit while
//! layering deterministic corrections on top. The format is intentionally
//! simple enough to audit while still exercising the same segmented lookup flow
//! and random-access body/segment helpers that later, denser artifacts will use.
//! See `spec/data-compression.md` for the stored-vs-derived output contract and
//! `docs/time-observer-policy.md` for the explicit packaged lookup policy that
//! keeps derived coordinates separate from observer-driven body-position modes.
//!
//! Enable the optional `serde` feature to serialize compressed artifacts for
//! inspection or interchange workflows.
//!
//! # Examples
//!
//! ```
//! use pleiades_compression::{ArtifactHeader, BodyArtifact, ChannelKind, CompressedArtifact,
//!     PolynomialChannel, Segment, ARTIFACT_VERSION};
//! use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
//!
//! let artifact = CompressedArtifact::new(
//!     ArtifactHeader::new("demo", "synthetic example data"),
//!     vec![BodyArtifact::new(
//!         CelestialBody::Sun,
//!         vec![Segment::new(
//!             Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
//!             Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
//!             vec![PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0)],
//!         )],
//!     )],
//! );
//! assert_eq!(artifact.header.version, ARTIFACT_VERSION);
//! ```

#![forbid(unsafe_code)]

use core::fmt;

pub(crate) mod artifact;
pub(crate) mod channels;
pub(crate) mod codec;
pub(crate) mod error;
pub(crate) mod format;
mod frame_recombine;

// ── Public re-exports (preserve exact prior public surface) ───────────────────

pub use artifact::CompressedArtifact;
pub use channels::{BodyArtifact, ChannelKind, PolynomialChannel, Segment, StoredFrame};
pub use error::{CompressionError, CompressionErrorKind};
pub use format::{
    ArtifactHeader, ArtifactOutput, ArtifactOutputSupport, ArtifactProfile,
    ArtifactProfileCoverageSummary, ArtifactResidualBodyCoverageSummary, EndianPolicy, SpeedPolicy,
};
pub use frame_recombine::{
    cartesian_au_to_ecliptic, ecliptic_to_cartesian_au, geocentric_from_heliocentric,
    heliocentric_from_geocentric,
};

// ── Crate-level constants ─────────────────────────────────────────────────────

/// Current artifact format version.
pub const ARTIFACT_VERSION: u16 = 6;

pub(crate) const ARTIFACT_MAGIC: [u8; 8] = *b"PLDEPHEM";

// ── Public formatting helper ──────────────────────────────────────────────────

/// Joins displayable values into a compact comma-separated list.
///
/// This helper is used by release-facing summaries that need a stable,
/// human-readable body or capability listing without introducing an extra
/// formatting dependency in downstream crates.
pub fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests;
