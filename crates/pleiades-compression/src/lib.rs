//! Compression codecs and artifact packing helpers for ephemeris data.
//!
//! The current implementation defines a small, deterministic artifact format
//! with explicit versioning, checksums, artifact capability profiles, and
//! quantized polynomial segments. Optional residual-correction channels are
//! also supported so denser artifacts can keep a compact base fit while
//! layering deterministic corrections on top. The format is intentionally
//! simple enough to audit while still exercising the same segmented lookup flow
//! and random-access body/segment helpers that later, denser artifacts will use.
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
use std::collections::HashMap;

use pleiades_types::{
    Angle, CelestialBody, CustomBodyId, EclipticCoordinates, EquatorialCoordinates, Instant,
    JulianDay, Latitude, Longitude, TimeScale,
};

/// Current artifact format version.
pub const ARTIFACT_VERSION: u16 = 4;

const ARTIFACT_MAGIC: [u8; 8] = *b"PLDEPHEM";

/// Describes the byte-order policy encoded by a compressed artifact.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum EndianPolicy {
    /// The artifact stores its numeric fields in little-endian byte order.
    LittleEndian,
}

impl EndianPolicy {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::LittleEndian => "little-endian",
        }
    }
}

impl fmt::Display for EndianPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Describes the non-body metadata stored in a compressed artifact.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactHeader {
    /// Format version.
    pub version: u16,
    /// Human-readable generation label.
    pub generation_label: String,
    /// Human-readable provenance/source summary.
    pub source: String,
    /// Explicit byte-order policy for the stored numeric fields.
    pub endian_policy: EndianPolicy,
    /// Artifact capability profile describing stored, derived, and unsupported outputs.
    pub profile: ArtifactProfile,
}

impl ArtifactHeader {
    /// Creates a new header using the current artifact version, an explicit
    /// little-endian byte-order policy, and a conservative ecliptic-only profile.
    pub fn new(generation_label: impl Into<String>, source: impl Into<String>) -> Self {
        Self::with_profile(
            generation_label,
            source,
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        )
    }

    /// Creates a new header using the current artifact version, an explicit
    /// little-endian byte-order policy, and an explicit profile.
    pub fn with_profile(
        generation_label: impl Into<String>,
        source: impl Into<String>,
        profile: ArtifactProfile,
    ) -> Self {
        Self::with_profile_and_endian(
            generation_label,
            source,
            EndianPolicy::LittleEndian,
            profile,
        )
    }

    /// Creates a new header with an explicit byte-order policy and profile.
    pub fn with_profile_and_endian(
        generation_label: impl Into<String>,
        source: impl Into<String>,
        endian_policy: EndianPolicy,
        profile: ArtifactProfile,
    ) -> Self {
        Self {
            version: ARTIFACT_VERSION,
            generation_label: generation_label.into(),
            source: source.into(),
            endian_policy,
            profile,
        }
    }

    /// Returns a compact one-line summary of the byte order and capability
    /// profile encoded by this header.
    pub fn summary(&self) -> String {
        self.summary_line()
    }

    /// Returns a compact one-line summary of the byte order and capability
    /// profile encoded by this header.
    pub fn summary_line(&self) -> String {
        format!("byte order: {}; {}", self.endian_policy, self.profile)
    }

    /// Validates that the header's version and provenance fields are populated
    /// with canonical, non-whitespace-padded text.
    ///
    /// The codec already enforces these checks at encode/decode time, but
    /// exposing the validation step directly lets artifact generators and
    /// release tooling fail before writing or reusing an invalid header.
    pub fn validate(&self) -> Result<(), CompressionError> {
        if self.version != ARTIFACT_VERSION {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!(
                    "artifact header version {} does not match the current format version {}",
                    self.version, ARTIFACT_VERSION
                ),
            ));
        }

        validate_canonical_header_text("artifact header generation label", &self.generation_label)?;
        validate_canonical_header_text("artifact header source", &self.source)?;

        self.profile.validate()
    }

    /// Returns the header summary annotated with how many bodies share it.
    pub fn summary_for_body_count(&self, body_count: usize) -> String {
        format!(
            "{}; applies to {} bundled bodies",
            self.summary_line(),
            body_count
        )
    }
}

/// Artifact-level output semantics for fields that are not raw segment channels.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ArtifactOutput {
    /// Ecliptic coordinates assembled from longitude, latitude, and distance channels.
    EclipticCoordinates,
    /// Equatorial coordinates reconstructed from ecliptic coordinates and obliquity policy.
    EquatorialCoordinates,
    /// Apparent longitude/latitude corrections such as light-time, aberration, or nutation.
    ApparentCorrections,
    /// Topocentric coordinates reconstructed for a terrestrial observer.
    TopocentricCoordinates,
    /// Sidereal coordinates derived from tropical coordinates and ayanamsa policy.
    SiderealCoordinates,
    /// Longitude/latitude/radial speed values.
    Motion,
}

impl ArtifactOutput {
    /// Returns all built-in artifact outputs in a stable declaration order.
    pub const fn all() -> [Self; 6] {
        [
            Self::EclipticCoordinates,
            Self::EquatorialCoordinates,
            Self::ApparentCorrections,
            Self::TopocentricCoordinates,
            Self::SiderealCoordinates,
            Self::Motion,
        ]
    }

    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::EclipticCoordinates => "EclipticCoordinates",
            Self::EquatorialCoordinates => "EquatorialCoordinates",
            Self::ApparentCorrections => "ApparentCorrections",
            Self::TopocentricCoordinates => "TopocentricCoordinates",
            Self::SiderealCoordinates => "SiderealCoordinates",
            Self::Motion => "Motion",
        }
    }
}

impl fmt::Display for ArtifactOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Describes how a high-level artifact output is represented by the profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ArtifactOutputSupport {
    /// The output is reconstructed deterministically from stored data.
    Derived,
    /// The output is explicitly unsupported by the profile.
    Unsupported,
    /// The output is neither stored nor explicitly declared by the profile.
    Unlisted,
}

impl ArtifactOutputSupport {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Derived => "derived",
            Self::Unsupported => "unsupported",
            Self::Unlisted => "unlisted",
        }
    }
}

impl fmt::Display for ArtifactOutputSupport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl fmt::Display for ArtifactProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl fmt::Display for ArtifactHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn validate_canonical_header_text(field: &str, value: &str) -> Result<(), CompressionError> {
    if value.trim().is_empty() {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!("{field} must not be blank"),
        ));
    }

    if value != value.trim() {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!("{field} must not include surrounding whitespace"),
        ));
    }

    Ok(())
}

/// Declares how motion/speed values are represented by an artifact.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum SpeedPolicy {
    /// The artifact does not provide speed values.
    Unsupported,
    /// Speeds are stored as direct channels.
    Stored,
    /// Speeds are derived analytically from fitted segment derivatives.
    FittedDerivative,
    /// Speeds are approximated numerically from neighboring decoded samples.
    NumericalDifference,
}

impl SpeedPolicy {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unsupported => "Unsupported",
            Self::Stored => "Stored",
            Self::FittedDerivative => "FittedDerivative",
            Self::NumericalDifference => "NumericalDifference",
        }
    }

    /// Returns how motion/speed output is represented when this policy is used.
    pub const fn motion_output_support(self) -> ArtifactOutputSupport {
        match self {
            Self::Unsupported => ArtifactOutputSupport::Unsupported,
            Self::Stored | Self::FittedDerivative | Self::NumericalDifference => {
                ArtifactOutputSupport::Derived
            }
        }
    }
}

impl fmt::Display for SpeedPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Capability/profile metadata for a compressed artifact.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactProfile {
    /// Coordinate channels stored directly in each applicable segment.
    pub stored_channels: Vec<ChannelKind>,
    /// Higher-level outputs that decoders may derive deterministically from stored data.
    pub derived_outputs: Vec<ArtifactOutput>,
    /// Outputs explicitly unsupported by this artifact profile.
    pub unsupported_outputs: Vec<ArtifactOutput>,
    /// Motion/speed representation policy.
    pub speed_policy: SpeedPolicy,
}

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

fn format_bracketed_labels<T: fmt::Display>(values: &[T]) -> String {
    format!("[{}]", join_display(values))
}

impl ArtifactProfile {
    /// Creates a profile from explicit fields.
    pub fn new(
        stored_channels: Vec<ChannelKind>,
        derived_outputs: Vec<ArtifactOutput>,
        unsupported_outputs: Vec<ArtifactOutput>,
        speed_policy: SpeedPolicy,
    ) -> Self {
        Self {
            stored_channels,
            derived_outputs,
            unsupported_outputs,
            speed_policy,
        }
    }

    /// Validates that the profile does not contain duplicate or conflicting entries.
    ///
    /// The codec performs the same checks when encoding or decoding artifacts,
    /// but exposing the validation step directly lets artifact generators fail
    /// before serialization if they assemble an invalid capability profile.
    pub fn validate(&self) -> Result<(), CompressionError> {
        validate_artifact_profile(self)
    }

    /// Returns a compact one-line summary of the stored, derived, unsupported,
    /// and speed-policy capabilities encoded by this profile.
    pub fn summary(&self) -> String {
        self.summary_line()
    }

    /// Returns a compact one-line summary of the stored, derived, unsupported,
    /// and speed-policy capabilities encoded by this profile.
    pub fn summary_line(&self) -> String {
        format!(
            "stored channels: {}; derived outputs: {}; unsupported outputs: {}; speed policy: {}",
            format_bracketed_labels(&self.stored_channels),
            format_bracketed_labels(&self.derived_outputs),
            format_bracketed_labels(&self.unsupported_outputs),
            self.speed_policy,
        )
    }

    /// Returns the capability summary annotated with how many bodies share it.
    pub fn summary_for_body_count(&self, body_count: usize) -> String {
        format!(
            "{}; applies to {} bundled bodies",
            self.summary_line(),
            body_count
        )
    }

    /// Returns the capability summary annotated with how many bodies share it.
    pub fn summary_line_with_body_count(&self, body_count: usize) -> String {
        self.summary_for_body_count(body_count)
    }

    /// Returns the compact support entries used by the output-support summary.
    pub fn output_support_entries_summary_line(&self) -> String {
        ArtifactOutput::all()
            .into_iter()
            .map(|output| format!("{output}={}", self.output_support(output)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Returns a compact one-line summary of each artifact output's support state.
    pub fn output_support_summary_line(&self) -> String {
        format!(
            "output support: {}",
            self.output_support_entries_summary_line()
        )
    }

    /// Returns how a high-level output is represented by this profile.
    pub fn output_support(&self, output: ArtifactOutput) -> ArtifactOutputSupport {
        if self.derived_outputs.contains(&output) {
            ArtifactOutputSupport::Derived
        } else if self.unsupported_outputs.contains(&output) {
            ArtifactOutputSupport::Unsupported
        } else {
            ArtifactOutputSupport::Unlisted
        }
    }

    /// Returns whether the profile can reconstruct the requested output.
    pub fn supports_output(&self, output: ArtifactOutput) -> bool {
        matches!(self.output_support(output), ArtifactOutputSupport::Derived)
    }

    /// Returns whether the profile explicitly marks the output unsupported.
    pub fn is_unsupported_output(&self, output: ArtifactOutput) -> bool {
        matches!(
            self.output_support(output),
            ArtifactOutputSupport::Unsupported
        )
    }

    /// Returns the current conservative profile: ecliptic longitude, latitude,
    /// and distance are stored directly; ecliptic coordinates are reconstructed
    /// from those channels, equatorial coordinates are derived from the stored
    /// ecliptic coordinates and mean-obliquity policy, and motion or richer
    /// coordinate modes remain unsupported.
    pub fn ecliptic_longitude_latitude_distance() -> Self {
        Self::new(
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu,
            ],
            vec![
                ArtifactOutput::EclipticCoordinates,
                ArtifactOutput::EquatorialCoordinates,
            ],
            vec![
                ArtifactOutput::ApparentCorrections,
                ArtifactOutput::TopocentricCoordinates,
                ArtifactOutput::SiderealCoordinates,
                ArtifactOutput::Motion,
            ],
            SpeedPolicy::Unsupported,
        )
    }
}

/// Structured body coverage attached to an artifact capability profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactProfileCoverageSummary {
    /// Number of bundled bodies that share the profile.
    pub body_count: usize,
    /// Bodies bundled under the profile.
    pub bodies: Vec<CelestialBody>,
    /// Capability profile encoded by the artifact.
    pub profile: ArtifactProfile,
}

impl ArtifactProfileCoverageSummary {
    /// Creates a profile coverage summary from the profile and bundled bodies.
    pub fn new(profile: ArtifactProfile, bodies: Vec<CelestialBody>) -> Self {
        let body_count = bodies.len();
        Self {
            body_count,
            bodies,
            profile,
        }
    }

    /// Validates that the summary's body count matches the bundled body list and
    /// that the embedded artifact profile is internally consistent.
    pub fn validate(&self) -> Result<(), CompressionError> {
        self.profile.validate()?;
        if self.bodies.is_empty() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "artifact profile coverage bundled body list must not be empty",
            ));
        }
        validate_unique_values("artifact profile coverage bundled bodies", &self.bodies)?;
        if self.body_count != self.bodies.len() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "artifact profile coverage body count does not match bundled body list",
            ));
        }

        Ok(())
    }

    /// Returns the capability summary annotated with how many bodies share it.
    pub fn summary_line(&self) -> String {
        self.profile.summary_for_body_count(self.body_count)
    }

    /// Returns the capability summary annotated with how many bodies share it
    /// and lists the bundled bodies explicitly.
    pub fn summary_line_with_bodies(&self) -> String {
        format!(
            "{}; bundled bodies: {}",
            self.summary_line(),
            join_display(&self.bodies)
        )
    }
}

impl fmt::Display for ArtifactProfileCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// The kind of ecliptic channel carried by a segment.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum ChannelKind {
    /// Ecliptic longitude in degrees.
    Longitude,
    /// Ecliptic latitude in degrees.
    Latitude,
    /// Radius vector or distance in astronomical units.
    DistanceAu,
}

impl ChannelKind {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Longitude => "Longitude",
            Self::Latitude => "Latitude",
            Self::DistanceAu => "DistanceAu",
        }
    }
}

impl fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Quantized polynomial coefficients for one channel of a time segment.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PolynomialChannel {
    /// Channel kind.
    pub kind: ChannelKind,
    /// Decimal scale exponent used when quantizing coefficients.
    pub scale_exponent: u8,
    /// Polynomial coefficients in ascending power order, expressed in native channel units.
    pub coefficients: Vec<f64>,
}

impl PolynomialChannel {
    /// Creates a channel from already-normalized polynomial coefficients.
    pub fn new(kind: ChannelKind, scale_exponent: u8, coefficients: Vec<f64>) -> Self {
        Self {
            kind,
            scale_exponent,
            coefficients,
        }
    }

    /// Creates a linear channel from endpoint values over the normalized interval `[0, 1]`.
    pub fn linear(kind: ChannelKind, scale_exponent: u8, start: f64, end: f64) -> Self {
        Self::new(kind, scale_exponent, vec![start, end - start])
    }

    /// Validates that the channel coefficients are finite before encoding or lookup.
    pub fn validate(&self) -> Result<(), CompressionError> {
        for (index, coefficient) in self.coefficients.iter().enumerate() {
            if !coefficient.is_finite() {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    format!(
                        "polynomial channel {:?} contains a non-finite coefficient at index {index}",
                        self.kind
                    ),
                ));
            }
        }

        Ok(())
    }

    fn evaluate(&self, x: f64) -> f64 {
        let mut result = 0.0;
        let mut power = 1.0;
        for coefficient in &self.coefficients {
            result += coefficient * power;
            power *= x;
        }
        result
    }
}

/// A single time segment for a specific body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// Inclusive segment start.
    pub start: Instant,
    /// Inclusive segment end.
    pub end: Instant,
    /// Quantized polynomial channels.
    pub channels: Vec<PolynomialChannel>,
    /// Optional residual-correction channels layered on top of the base fit.
    pub residual_channels: Vec<PolynomialChannel>,
}

impl Segment {
    /// Creates a new segment.
    pub fn new(start: Instant, end: Instant, channels: Vec<PolynomialChannel>) -> Self {
        Self {
            start,
            end,
            channels,
            residual_channels: Vec::new(),
        }
    }

    /// Creates a new segment with optional residual-correction channels.
    pub fn with_residual_channels(
        start: Instant,
        end: Instant,
        channels: Vec<PolynomialChannel>,
        residual_channels: Vec<PolynomialChannel>,
    ) -> Self {
        Self {
            start,
            end,
            channels,
            residual_channels,
        }
    }

    /// Validates the segment metadata before the segment is stored or encoded.
    ///
    /// Stored and residual channels must be unique and ordered canonically by
    /// channel kind so deterministic encoding stays stable across builders.
    pub fn validate(&self) -> Result<(), CompressionError> {
        validate_segment(self)
    }

    /// Returns a compact one-line summary of the segment span and channel mix.
    pub fn summary_line(&self) -> String {
        let stored_channels = self
            .channels
            .iter()
            .map(|channel| channel.kind)
            .collect::<Vec<_>>();
        let residual_channels = self
            .residual_channels
            .iter()
            .map(|channel| channel.kind)
            .collect::<Vec<_>>();

        format!(
            "start: {}; end: {}; stored channels: {}; residual channels: {}",
            self.start,
            self.end,
            format_bracketed_labels(&stored_channels),
            format_bracketed_labels(&residual_channels),
        )
    }

    fn contains(&self, instant: Instant) -> bool {
        self.start.scale == instant.scale
            && self.end.scale == instant.scale
            && self.start.julian_day.days() <= instant.julian_day.days()
            && instant.julian_day.days() <= self.end.julian_day.days()
    }

    fn span_days(&self) -> f64 {
        self.end.julian_day.days() - self.start.julian_day.days()
    }

    fn channel(&self, kind: ChannelKind) -> Option<&PolynomialChannel> {
        self.channels.iter().find(|channel| channel.kind == kind)
    }

    fn residual_channel(&self, kind: ChannelKind) -> Option<&PolynomialChannel> {
        self.residual_channels
            .iter()
            .find(|channel| channel.kind == kind)
    }

    fn evaluate_channel(&self, kind: ChannelKind, x: f64) -> Result<f64, CompressionError> {
        let base = self
            .channel(kind)
            .map(|channel| channel.evaluate(x))
            .ok_or_else(|| {
                CompressionError::new(
                    CompressionErrorKind::MissingChannel,
                    format!("missing {kind:?} channel"),
                )
            })?;

        let residual = self
            .residual_channel(kind)
            .map(|channel| channel.evaluate(x))
            .unwrap_or(0.0);

        Ok(base + residual)
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// All segments for a single body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BodyArtifact {
    /// Body identifier.
    pub body: CelestialBody,
    /// Time segments for the body.
    pub segments: Vec<Segment>,
}

impl BodyArtifact {
    /// Creates a new body artifact.
    pub fn new(body: CelestialBody, segments: Vec<Segment>) -> Self {
        Self { body, segments }
    }

    /// Validates the body's segment metadata.
    ///
    /// This checks each segment's internal invariants, ensures that segments
    /// using the same time scale are ordered and non-overlapping, and rejects
    /// duplicate stored or residual channels before lookup or encoding.
    pub fn validate(&self) -> Result<(), CompressionError> {
        for segment in &self.segments {
            segment.validate()?;
        }

        validate_body_segments(&self.segments)
    }

    /// Returns a compact one-line summary of the body's segment coverage.
    pub fn summary_line(&self) -> String {
        let residual_segment_count = self
            .segments
            .iter()
            .filter(|segment| !segment.residual_channels.is_empty())
            .count();

        format!(
            "body: {}; segments: {}; residual-bearing segments: {}",
            self.body,
            self.segments.len(),
            residual_segment_count,
        )
    }

    /// Returns the segment covering the requested instant, if any.
    ///
    /// When two adjacent segments both include the same boundary instant, the
    /// later segment wins. This keeps shared segment edges deterministic for
    /// piecewise artifacts that use inclusive endpoints.
    pub fn segment_at(&self, instant: Instant) -> Option<&Segment> {
        self.segments
            .iter()
            .rev()
            .find(|segment| segment.contains(instant))
    }
}

impl fmt::Display for BodyArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A compressed ephemeris artifact.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct CompressedArtifact {
    /// File header metadata.
    pub header: ArtifactHeader,
    /// Checksum over the payload bytes.
    pub checksum: u64,
    /// Body series stored in the artifact.
    pub bodies: Vec<BodyArtifact>,
}

impl CompressedArtifact {
    /// Creates a new artifact with an unset checksum.
    pub fn new(header: ArtifactHeader, bodies: Vec<BodyArtifact>) -> Self {
        Self {
            header,
            checksum: 0,
            bodies,
        }
    }

    /// Validates the in-memory artifact before encoding or regeneration.
    ///
    /// This checks the header metadata, header profile, duplicate body entries,
    /// and every body segment's metadata. It is useful for generators that want
    /// to fail fast before writing a deterministic binary payload.
    pub fn validate(&self) -> Result<(), CompressionError> {
        self.header.validate()?;
        validate_body_artifacts(&self.bodies)?;
        for body in &self.bodies {
            body.validate()?;
        }

        Ok(())
    }

    /// Returns the body artifact for the requested body, if present.
    pub fn body_artifact(&self, body: &CelestialBody) -> Option<&BodyArtifact> {
        self.bodies.iter().find(|series| &series.body == body)
    }

    /// Returns the body segment covering the requested instant.
    pub fn segment_for(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<&Segment, CompressionError> {
        let series = self.body_artifact(body).ok_or_else(|| {
            CompressionError::new(
                CompressionErrorKind::MissingBody,
                format!("no packed data exists for {body:?}"),
            )
        })?;

        series.segment_at(instant).ok_or_else(|| {
            CompressionError::new(
                CompressionErrorKind::OutOfRangeInstant,
                format!("no packed segment covers {body:?} at {instant:?}"),
            )
        })
    }

    /// Returns the on-disk checksum for this artifact.
    pub fn checksum(&self) -> Result<u64, CompressionError> {
        Ok(fnv1a64(&self.encode_payload()?))
    }

    /// Encodes the artifact as a deterministic binary blob.
    pub fn encode(&self) -> Result<Vec<u8>, CompressionError> {
        let payload = self.encode_payload()?;
        let checksum = fnv1a64(&payload);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ARTIFACT_MAGIC);
        write_u16(&mut bytes, self.header.version);
        write_u64(&mut bytes, checksum);
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }

    /// Decodes an artifact from a binary blob.
    ///
    /// Checksum mismatches are rejected with [`CompressionErrorKind::ChecksumMismatch`]
    /// before the payload is parsed.
    pub fn decode(bytes: &[u8]) -> Result<Self, CompressionError> {
        let mut cursor = Cursor::new(bytes);
        let magic = cursor.read_array::<8>()?;
        if magic != ARTIFACT_MAGIC {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidMagic,
                "compressed artifact magic header did not match",
            ));
        }

        let version = cursor.read_u16()?;
        if version != ARTIFACT_VERSION {
            return Err(CompressionError::new(
                CompressionErrorKind::UnsupportedVersion,
                format!("artifact version {version} is not supported"),
            ));
        }

        let checksum = cursor.read_u64()?;
        let payload = cursor.remaining();
        if fnv1a64(payload) != checksum {
            return Err(CompressionError::new(
                CompressionErrorKind::ChecksumMismatch,
                "compressed artifact checksum did not match",
            ));
        }

        let mut payload_cursor = Cursor::new(payload);
        let header = ArtifactHeader {
            version,
            generation_label: payload_cursor.read_string()?,
            source: payload_cursor.read_string()?,
            endian_policy: decode_endian_policy(payload_cursor.read_u8()?)?,
            profile: decode_artifact_profile(&mut payload_cursor)?,
        };
        let body_count = payload_cursor.read_u16()? as usize;
        let mut bodies = Vec::with_capacity(body_count);
        for _ in 0..body_count {
            let body = decode_body(&mut payload_cursor)?;
            bodies.push(body);
        }

        if !payload_cursor.is_finished() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "compressed artifact contained trailing bytes",
            ));
        }

        let artifact = Self {
            header,
            checksum,
            bodies,
        };
        artifact.validate()?;

        Ok(artifact)
    }

    /// Returns the ecliptic coordinates for a body at a given instant.
    ///
    /// The artifact profile must advertise `EclipticCoordinates` as a derived
    /// output before this helper will serve the result.
    pub fn lookup_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, CompressionError> {
        self.require_output_support(ArtifactOutput::EclipticCoordinates)?;

        if !matches!(instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(CompressionError::new(
                CompressionErrorKind::UnsupportedTimeScale,
                "packaged lookup only accepts TT or TDB instants",
            ));
        }

        let segment = self.segment_for(body, instant)?;

        let span = segment.span_days();
        let x = if span == 0.0 {
            0.0
        } else {
            (instant.julian_day.days() - segment.start.julian_day.days()) / span
        };

        let longitude = segment.evaluate_channel(ChannelKind::Longitude, x)?;
        let latitude = segment.evaluate_channel(ChannelKind::Latitude, x)?;
        let distance_au = segment.evaluate_channel(ChannelKind::DistanceAu, x)?;

        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(longitude),
            Latitude::from_degrees(latitude),
            Some(distance_au),
        ))
    }

    /// Returns equatorial coordinates reconstructed from the stored ecliptic channels.
    ///
    /// This keeps the artifact format focused on the stored channels while still allowing
    /// the runtime to reconstruct a derived coordinate family when the caller supplies the
    /// geometric obliquity used for the mean-obliquity frame rotation. The artifact profile
    /// must advertise `EquatorialCoordinates` as a derived output before this helper will
    /// serve the result.
    pub fn lookup_equatorial(
        &self,
        body: &CelestialBody,
        instant: Instant,
        obliquity: Angle,
    ) -> Result<EquatorialCoordinates, CompressionError> {
        self.require_output_support(ArtifactOutput::EquatorialCoordinates)?;
        Ok(self
            .lookup_ecliptic(body, instant)?
            .to_equatorial(obliquity))
    }

    fn require_output_support(&self, output: ArtifactOutput) -> Result<(), CompressionError> {
        if self.header.profile.supports_output(output) {
            Ok(())
        } else {
            Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("artifact profile does not derive {output}"),
            ))
        }
    }

    fn encode_payload(&self) -> Result<Vec<u8>, CompressionError> {
        self.validate()?;

        let mut bytes = Vec::new();
        write_string(&mut bytes, &self.header.generation_label);
        write_string(&mut bytes, &self.header.source);
        write_u8(&mut bytes, encode_endian_policy(self.header.endian_policy));
        encode_artifact_profile(&mut bytes, &self.header.profile)?;
        write_u16(&mut bytes, self.bodies.len() as u16);
        for body in &self.bodies {
            encode_body(&mut bytes, body)?;
        }
        Ok(bytes)
    }
}

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

fn encode_endian_policy(policy: EndianPolicy) -> u8 {
    match policy {
        EndianPolicy::LittleEndian => 0,
    }
}

fn decode_endian_policy(value: u8) -> Result<EndianPolicy, CompressionError> {
    match value {
        0 => Ok(EndianPolicy::LittleEndian),
        other => Err(CompressionError::new(
            CompressionErrorKind::UnsupportedEndianPolicy,
            format!("artifact byte-order policy {other} is not supported"),
        )),
    }
}

fn encode_artifact_profile(
    bytes: &mut Vec<u8>,
    profile: &ArtifactProfile,
) -> Result<(), CompressionError> {
    profile.validate()?;

    write_u8(bytes, profile.stored_channels.len() as u8);
    for channel in &profile.stored_channels {
        write_u8(bytes, encode_channel_kind(*channel));
    }

    write_u8(bytes, profile.derived_outputs.len() as u8);
    for output in &profile.derived_outputs {
        write_u8(bytes, encode_artifact_output(*output));
    }

    write_u8(bytes, profile.unsupported_outputs.len() as u8);
    for output in &profile.unsupported_outputs {
        write_u8(bytes, encode_artifact_output(*output));
    }

    write_u8(bytes, encode_speed_policy(profile.speed_policy));
    Ok(())
}

fn decode_artifact_profile(cursor: &mut Cursor<'_>) -> Result<ArtifactProfile, CompressionError> {
    let stored_channel_count = cursor.read_u8()? as usize;
    let mut stored_channels = Vec::with_capacity(stored_channel_count);
    for _ in 0..stored_channel_count {
        stored_channels.push(decode_channel_kind(cursor.read_u8()?)?);
    }

    let derived_output_count = cursor.read_u8()? as usize;
    let mut derived_outputs = Vec::with_capacity(derived_output_count);
    for _ in 0..derived_output_count {
        derived_outputs.push(decode_artifact_output(cursor.read_u8()?)?);
    }

    let unsupported_output_count = cursor.read_u8()? as usize;
    let mut unsupported_outputs = Vec::with_capacity(unsupported_output_count);
    for _ in 0..unsupported_output_count {
        unsupported_outputs.push(decode_artifact_output(cursor.read_u8()?)?);
    }

    let speed_policy = decode_speed_policy(cursor.read_u8()?)?;
    let profile = ArtifactProfile::new(
        stored_channels,
        derived_outputs,
        unsupported_outputs,
        speed_policy,
    );
    profile.validate()?;
    Ok(profile)
}

fn validate_artifact_profile(profile: &ArtifactProfile) -> Result<(), CompressionError> {
    validate_unique_values("artifact profile stored channels", &profile.stored_channels)?;
    validate_unique_values("artifact profile derived outputs", &profile.derived_outputs)?;
    validate_unique_values(
        "artifact profile unsupported outputs",
        &profile.unsupported_outputs,
    )?;
    validate_disjoint_values(
        "artifact profile derived outputs",
        &profile.derived_outputs,
        "artifact profile unsupported outputs",
        &profile.unsupported_outputs,
    )?;
    validate_coordinate_output_policy(profile)?;
    validate_motion_policy(profile)?;
    Ok(())
}

fn validate_coordinate_output_policy(profile: &ArtifactProfile) -> Result<(), CompressionError> {
    let needs_coordinate_channels = profile.derived_outputs.iter().any(|output| {
        matches!(
            *output,
            ArtifactOutput::EclipticCoordinates
                | ArtifactOutput::EquatorialCoordinates
                | ArtifactOutput::ApparentCorrections
                | ArtifactOutput::TopocentricCoordinates
                | ArtifactOutput::SiderealCoordinates
        )
    });

    if needs_coordinate_channels
        && (!profile.stored_channels.contains(&ChannelKind::Longitude)
            || !profile.stored_channels.contains(&ChannelKind::Latitude)
            || !profile.stored_channels.contains(&ChannelKind::DistanceAu))
    {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            "artifact profile derived coordinate outputs require Longitude, Latitude, and DistanceAu in stored channels",
        ));
    }

    Ok(())
}

fn validate_motion_policy(profile: &ArtifactProfile) -> Result<(), CompressionError> {
    let motion_support = profile.speed_policy.motion_output_support();
    match motion_support {
        ArtifactOutputSupport::Derived => {
            if !profile.derived_outputs.contains(&ArtifactOutput::Motion) {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    format!(
                        "artifact profile speed policy {} requires Motion to be listed in derived outputs",
                        profile.speed_policy
                    ),
                ));
            }
        }
        ArtifactOutputSupport::Unsupported => {
            if !profile
                .unsupported_outputs
                .contains(&ArtifactOutput::Motion)
            {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    format!(
                        "artifact profile speed policy {} requires Motion to be listed in unsupported outputs",
                        profile.speed_policy
                    ),
                ));
            }
        }
        ArtifactOutputSupport::Unlisted => {
            unreachable!("motion support is always derived or unsupported")
        }
    }

    Ok(())
}

fn validate_body_artifacts(bodies: &[BodyArtifact]) -> Result<(), CompressionError> {
    for (index, body) in bodies.iter().enumerate() {
        if bodies[..index].iter().any(|other| other.body == body.body) {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("compressed artifact contains duplicate body entry {body:?}"),
            ));
        }
    }

    Ok(())
}

fn validate_unique_values<T: fmt::Display + Eq>(
    label: &str,
    values: &[T],
) -> Result<(), CompressionError> {
    for (index, value) in values.iter().enumerate() {
        if values[..index].contains(value) {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("{label} contains duplicate {value} entry"),
            ));
        }
    }

    Ok(())
}

fn validate_disjoint_values<T: fmt::Display + Eq>(
    left_label: &str,
    left: &[T],
    right_label: &str,
    right: &[T],
) -> Result<(), CompressionError> {
    for value in left {
        if right.contains(value) {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("{left_label} and {right_label} both include {value}"),
            ));
        }
    }

    Ok(())
}

fn encode_body(bytes: &mut Vec<u8>, body: &BodyArtifact) -> Result<(), CompressionError> {
    encode_celestial_body(bytes, &body.body)?;
    write_u16(bytes, body.segments.len() as u16);
    for segment in &body.segments {
        encode_segment(bytes, segment)?;
    }
    Ok(())
}

fn decode_body(cursor: &mut Cursor<'_>) -> Result<BodyArtifact, CompressionError> {
    let body = decode_celestial_body(cursor)?;
    let segment_count = cursor.read_u16()? as usize;
    let mut segments = Vec::with_capacity(segment_count);
    for _ in 0..segment_count {
        segments.push(decode_segment(cursor)?);
    }
    Ok(BodyArtifact { body, segments })
}

fn encode_segment(bytes: &mut Vec<u8>, segment: &Segment) -> Result<(), CompressionError> {
    validate_segment(segment)?;

    encode_instant(bytes, segment.start);
    encode_instant(bytes, segment.end);
    write_u8(bytes, segment.channels.len() as u8);
    for channel in &segment.channels {
        encode_polynomial_channel(bytes, channel)?;
    }
    write_u8(bytes, segment.residual_channels.len() as u8);
    for channel in &segment.residual_channels {
        encode_polynomial_channel(bytes, channel)?;
    }
    Ok(())
}

fn decode_segment(cursor: &mut Cursor<'_>) -> Result<Segment, CompressionError> {
    let start = decode_instant(cursor)?;
    let end = decode_instant(cursor)?;
    let channel_count = cursor.read_u8()? as usize;
    let mut channels = Vec::with_capacity(channel_count);
    for _ in 0..channel_count {
        channels.push(decode_polynomial_channel(cursor)?);
    }
    let residual_channel_count = cursor.read_u8()? as usize;
    let mut residual_channels = Vec::with_capacity(residual_channel_count);
    for _ in 0..residual_channel_count {
        residual_channels.push(decode_polynomial_channel(cursor)?);
    }
    let segment = Segment::with_residual_channels(start, end, channels, residual_channels);
    validate_segment(&segment)?;
    Ok(segment)
}

fn validate_segment(segment: &Segment) -> Result<(), CompressionError> {
    if !segment.start.julian_day.days().is_finite() {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            "segment start must be finite",
        ));
    }

    if !segment.end.julian_day.days().is_finite() {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            "segment end must be finite",
        ));
    }

    if segment.end.julian_day.days() < segment.start.julian_day.days() {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            "segment end precedes segment start",
        ));
    }

    if segment.start.scale != segment.end.scale {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            "segment start and end use different time scales",
        ));
    }

    for channel in segment
        .channels
        .iter()
        .chain(segment.residual_channels.iter())
    {
        channel.validate()?;
    }

    let stored_channels = segment
        .channels
        .iter()
        .map(|channel| channel.kind)
        .collect::<Vec<_>>();
    let residual_channels = segment
        .residual_channels
        .iter()
        .map(|channel| channel.kind)
        .collect::<Vec<_>>();

    validate_unique_values("segment stored channels", &stored_channels)?;
    validate_unique_values("segment residual channels", &residual_channels)?;
    validate_channel_kind_order("segment stored channels", &stored_channels)?;
    validate_channel_kind_order("segment residual channels", &residual_channels)?;

    for residual_channel in &segment.residual_channels {
        if segment.channel(residual_channel.kind).is_none() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!(
                    "segment residual channels require a matching stored channel for {:?}",
                    residual_channel.kind
                ),
            ));
        }
    }

    Ok(())
}

fn validate_channel_kind_order(field: &str, kinds: &[ChannelKind]) -> Result<(), CompressionError> {
    for pair in kinds.windows(2) {
        if pair[0] as u8 > pair[1] as u8 {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!(
                    "{field} must be ordered by channel kind; found {:?} before {:?}",
                    pair[0], pair[1]
                ),
            ));
        }
    }

    Ok(())
}

fn validate_body_segments(segments: &[Segment]) -> Result<(), CompressionError> {
    let mut last_segment_end_by_scale: HashMap<TimeScale, f64> = HashMap::new();

    for segment in segments {
        let scale = segment.start.scale;
        let start = segment.start.julian_day.days();
        let end = segment.end.julian_day.days();

        if let Some(previous_end) = last_segment_end_by_scale.get(&scale) {
            if start < *previous_end {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    format!(
                        "body segments for {scale} must be ordered and non-overlapping; segment starting at {start} precedes the previous segment end {previous_end}"
                    ),
                ));
            }
        }

        last_segment_end_by_scale.insert(scale, end);
    }

    Ok(())
}

fn encode_instant(bytes: &mut Vec<u8>, instant: Instant) {
    write_f64(bytes, instant.julian_day.days());
    write_u8(bytes, encode_time_scale(instant.scale));
}

fn decode_instant(cursor: &mut Cursor<'_>) -> Result<Instant, CompressionError> {
    let julian_day = cursor.read_f64()?;
    let scale = decode_time_scale(cursor.read_u8()?)?;
    Ok(Instant::new(JulianDay::from_days(julian_day), scale))
}

fn encode_polynomial_channel(
    bytes: &mut Vec<u8>,
    channel: &PolynomialChannel,
) -> Result<(), CompressionError> {
    channel.validate()?;

    write_u8(bytes, encode_channel_kind(channel.kind));
    write_u8(bytes, channel.scale_exponent);
    write_u8(bytes, channel.coefficients.len() as u8);
    let scale = 10f64.powi(channel.scale_exponent as i32);
    for coefficient in &channel.coefficients {
        let scaled = (*coefficient * scale).round();
        if !scaled.is_finite() || scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
            return Err(CompressionError::new(
                CompressionErrorKind::QuantizationOverflow,
                "a polynomial coefficient exceeded the supported quantization range",
            ));
        }
        write_i64(bytes, scaled as i64);
    }
    Ok(())
}

fn decode_polynomial_channel(
    cursor: &mut Cursor<'_>,
) -> Result<PolynomialChannel, CompressionError> {
    let kind = decode_channel_kind(cursor.read_u8()?)?;
    let scale_exponent = cursor.read_u8()?;
    let coefficient_count = cursor.read_u8()? as usize;
    let scale = 10f64.powi(scale_exponent as i32);
    let mut coefficients = Vec::with_capacity(coefficient_count);
    for _ in 0..coefficient_count {
        coefficients.push(cursor.read_i64()? as f64 / scale);
    }
    Ok(PolynomialChannel::new(kind, scale_exponent, coefficients))
}

fn encode_celestial_body(
    bytes: &mut Vec<u8>,
    body: &CelestialBody,
) -> Result<(), CompressionError> {
    match body {
        CelestialBody::Sun => write_u8(bytes, 0),
        CelestialBody::Moon => write_u8(bytes, 1),
        CelestialBody::Mercury => write_u8(bytes, 2),
        CelestialBody::Venus => write_u8(bytes, 3),
        CelestialBody::Mars => write_u8(bytes, 4),
        CelestialBody::Jupiter => write_u8(bytes, 5),
        CelestialBody::Saturn => write_u8(bytes, 6),
        CelestialBody::Uranus => write_u8(bytes, 7),
        CelestialBody::Neptune => write_u8(bytes, 8),
        CelestialBody::Pluto => write_u8(bytes, 9),
        CelestialBody::MeanNode => write_u8(bytes, 10),
        CelestialBody::TrueNode => write_u8(bytes, 11),
        CelestialBody::MeanApogee => write_u8(bytes, 12),
        CelestialBody::TrueApogee => write_u8(bytes, 13),
        CelestialBody::MeanPerigee => write_u8(bytes, 20),
        CelestialBody::TruePerigee => write_u8(bytes, 21),
        CelestialBody::Ceres => write_u8(bytes, 14),
        CelestialBody::Pallas => write_u8(bytes, 15),
        CelestialBody::Juno => write_u8(bytes, 16),
        CelestialBody::Vesta => write_u8(bytes, 17),
        CelestialBody::Custom(custom) => {
            write_u8(bytes, 255);
            write_string(bytes, &custom.catalog);
            write_string(bytes, &custom.designation);
        }
        _ => {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "unsupported celestial body variant in compressed artifact",
            ))
        }
    }
    Ok(())
}

fn decode_celestial_body(cursor: &mut Cursor<'_>) -> Result<CelestialBody, CompressionError> {
    Ok(match cursor.read_u8()? {
        0 => CelestialBody::Sun,
        1 => CelestialBody::Moon,
        2 => CelestialBody::Mercury,
        3 => CelestialBody::Venus,
        4 => CelestialBody::Mars,
        5 => CelestialBody::Jupiter,
        6 => CelestialBody::Saturn,
        7 => CelestialBody::Uranus,
        8 => CelestialBody::Neptune,
        9 => CelestialBody::Pluto,
        10 => CelestialBody::MeanNode,
        11 => CelestialBody::TrueNode,
        12 => CelestialBody::MeanApogee,
        13 => CelestialBody::TrueApogee,
        14 => CelestialBody::Ceres,
        15 => CelestialBody::Pallas,
        16 => CelestialBody::Juno,
        17 => CelestialBody::Vesta,
        20 => CelestialBody::MeanPerigee,
        21 => CelestialBody::TruePerigee,
        255 => {
            let catalog = cursor.read_string()?;
            let designation = cursor.read_string()?;
            CelestialBody::Custom(CustomBodyId::new(catalog, designation))
        }
        other => {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("unknown body tag {other}"),
            ))
        }
    })
}

fn encode_channel_kind(kind: ChannelKind) -> u8 {
    match kind {
        ChannelKind::Longitude => 0,
        ChannelKind::Latitude => 1,
        ChannelKind::DistanceAu => 2,
    }
}

fn decode_channel_kind(tag: u8) -> Result<ChannelKind, CompressionError> {
    match tag {
        0 => Ok(ChannelKind::Longitude),
        1 => Ok(ChannelKind::Latitude),
        2 => Ok(ChannelKind::DistanceAu),
        other => Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!("unknown channel kind tag {other}"),
        )),
    }
}

fn encode_artifact_output(output: ArtifactOutput) -> u8 {
    match output {
        ArtifactOutput::EclipticCoordinates => 0,
        ArtifactOutput::EquatorialCoordinates => 1,
        ArtifactOutput::ApparentCorrections => 2,
        ArtifactOutput::TopocentricCoordinates => 3,
        ArtifactOutput::SiderealCoordinates => 4,
        ArtifactOutput::Motion => 5,
    }
}

fn decode_artifact_output(tag: u8) -> Result<ArtifactOutput, CompressionError> {
    match tag {
        0 => Ok(ArtifactOutput::EclipticCoordinates),
        1 => Ok(ArtifactOutput::EquatorialCoordinates),
        2 => Ok(ArtifactOutput::ApparentCorrections),
        3 => Ok(ArtifactOutput::TopocentricCoordinates),
        4 => Ok(ArtifactOutput::SiderealCoordinates),
        5 => Ok(ArtifactOutput::Motion),
        other => Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!("unknown artifact output tag {other}"),
        )),
    }
}

fn encode_speed_policy(policy: SpeedPolicy) -> u8 {
    match policy {
        SpeedPolicy::Unsupported => 0,
        SpeedPolicy::Stored => 1,
        SpeedPolicy::FittedDerivative => 2,
        SpeedPolicy::NumericalDifference => 3,
    }
}

fn decode_speed_policy(tag: u8) -> Result<SpeedPolicy, CompressionError> {
    match tag {
        0 => Ok(SpeedPolicy::Unsupported),
        1 => Ok(SpeedPolicy::Stored),
        2 => Ok(SpeedPolicy::FittedDerivative),
        3 => Ok(SpeedPolicy::NumericalDifference),
        other => Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!("unknown speed policy tag {other}"),
        )),
    }
}

fn encode_time_scale(scale: TimeScale) -> u8 {
    match scale {
        TimeScale::Utc => 0,
        TimeScale::Ut1 => 1,
        TimeScale::Tt => 2,
        TimeScale::Tdb => 3,
        _ => 255,
    }
}

fn decode_time_scale(tag: u8) -> Result<TimeScale, CompressionError> {
    match tag {
        0 => Ok(TimeScale::Utc),
        1 => Ok(TimeScale::Ut1),
        2 => Ok(TimeScale::Tt),
        3 => Ok(TimeScale::Tdb),
        other => Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!("unknown time-scale tag {other}"),
        )),
    }
}

fn write_string(bytes: &mut Vec<u8>, value: &str) {
    write_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_i64(bytes: &mut Vec<u8>, value: i64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_f64(bytes: &mut Vec<u8>, value: f64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.offset..]
    }

    fn is_finished(&self) -> bool {
        self.offset >= self.bytes.len()
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], CompressionError> {
        let bytes = self.read_exact(N)?;
        let mut array = [0u8; N];
        array.copy_from_slice(bytes);
        Ok(array)
    }

    fn read_u8(&mut self) -> Result<u8, CompressionError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16, CompressionError> {
        Ok(u16::from_le_bytes(self.read_array()?))
    }

    fn read_u32(&mut self) -> Result<u32, CompressionError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    fn read_u64(&mut self) -> Result<u64, CompressionError> {
        Ok(u64::from_le_bytes(self.read_array()?))
    }

    fn read_i64(&mut self) -> Result<i64, CompressionError> {
        Ok(i64::from_le_bytes(self.read_array()?))
    }

    fn read_f64(&mut self) -> Result<f64, CompressionError> {
        Ok(f64::from_le_bytes(self.read_array()?))
    }

    fn read_string(&mut self) -> Result<String, CompressionError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|error| {
            CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("compressed artifact string was not valid UTF-8: {error}"),
            )
        })
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], CompressionError> {
        let end = self.offset.checked_add(len).ok_or_else(|| {
            CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "compressed artifact length overflowed",
            )
        })?;
        if end > self.bytes.len() {
            return Err(CompressionError::new(
                CompressionErrorKind::Truncated,
                "compressed artifact ended unexpectedly",
            ));
        }
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_preserves_structure() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("demo", "unit test fixture"),
            vec![BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, -1.0, 1.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 1.0, 2.0),
                    ],
                )],
            )],
        );

        let encoded = artifact.encode().expect("artifact should encode");
        let decoded = CompressedArtifact::decode(&encoded).expect("artifact should decode");
        assert_eq!(decoded.header.version, ARTIFACT_VERSION);
        assert_eq!(decoded.header.generation_label, "demo");
        assert_eq!(decoded.header.source, "unit test fixture");
        assert_eq!(decoded.header.endian_policy, EndianPolicy::LittleEndian);
        assert_eq!(
            decoded.header.profile.stored_channels,
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu
            ]
        );
        assert_eq!(
            decoded.header.profile.speed_policy,
            SpeedPolicy::Unsupported
        );
        assert!(decoded
            .header
            .profile
            .derived_outputs
            .contains(&ArtifactOutput::EclipticCoordinates));
        assert!(decoded
            .header
            .profile
            .unsupported_outputs
            .contains(&ArtifactOutput::Motion));
        assert_eq!(decoded.bodies.len(), 1);
        assert_eq!(decoded.bodies[0].body, CelestialBody::Sun);
        assert_eq!(
            decoded.checksum,
            artifact.checksum().expect("checksum should compute")
        );
    }

    #[test]
    fn decode_rejects_checksum_corruption() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("demo", "tamper check fixture"),
            vec![BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        10.0,
                        11.0,
                    )],
                )],
            )],
        );

        let mut encoded = artifact.encode().expect("artifact should encode");
        let last_index = encoded.len() - 1;
        encoded[last_index] ^= 0x01;

        let error =
            CompressedArtifact::decode(&encoded).expect_err("tampered artifact should fail");
        assert_eq!(error.kind, CompressionErrorKind::ChecksumMismatch);
    }

    #[test]
    fn explicit_profile_roundtrip_preserves_stored_derived_and_unsupported_outputs() {
        let profile = ArtifactProfile::new(
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu,
            ],
            vec![ArtifactOutput::EclipticCoordinates, ArtifactOutput::Motion],
            vec![
                ArtifactOutput::EquatorialCoordinates,
                ArtifactOutput::TopocentricCoordinates,
            ],
            SpeedPolicy::FittedDerivative,
        );
        let artifact = CompressedArtifact::new(
            ArtifactHeader::with_profile("profile demo", "unit test profile", profile.clone()),
            Vec::new(),
        );

        let decoded = CompressedArtifact::decode(
            &artifact
                .encode()
                .expect("artifact should encode with profile"),
        )
        .expect("artifact should decode with profile");

        assert_eq!(decoded.header.profile, profile);
        assert_eq!(decoded.header.endian_policy, EndianPolicy::LittleEndian);
    }

    #[test]
    fn artifact_profile_reports_output_support_statuses() {
        let profile = ArtifactProfile::ecliptic_longitude_latitude_distance();

        assert_eq!(
            profile.output_support(ArtifactOutput::EclipticCoordinates),
            ArtifactOutputSupport::Derived
        );
        assert_eq!(
            profile.output_support(ArtifactOutput::EquatorialCoordinates),
            ArtifactOutputSupport::Derived
        );
        assert_eq!(
            profile.output_support(ArtifactOutput::Motion),
            ArtifactOutputSupport::Unsupported
        );
        assert_eq!(
            profile.output_support(ArtifactOutput::SiderealCoordinates),
            ArtifactOutputSupport::Unsupported
        );
        assert_eq!(
            profile.speed_policy.motion_output_support(),
            ArtifactOutputSupport::Unsupported
        );
        assert!(profile.supports_output(ArtifactOutput::EclipticCoordinates));
        assert!(profile.supports_output(ArtifactOutput::EquatorialCoordinates));
        assert!(!profile.is_unsupported_output(ArtifactOutput::EquatorialCoordinates));
        assert!(profile.is_unsupported_output(ArtifactOutput::SiderealCoordinates));

        assert_eq!(
            SpeedPolicy::Stored.motion_output_support(),
            ArtifactOutputSupport::Derived
        );
        assert_eq!(
            SpeedPolicy::FittedDerivative.motion_output_support(),
            ArtifactOutputSupport::Derived
        );
        assert_eq!(
            SpeedPolicy::NumericalDifference.motion_output_support(),
            ArtifactOutputSupport::Derived
        );

        let unlisted_profile = ArtifactProfile::new(
            vec![ChannelKind::Longitude],
            Vec::new(),
            Vec::new(),
            SpeedPolicy::Unsupported,
        );
        assert_eq!(
            unlisted_profile.output_support(ArtifactOutput::Motion),
            ArtifactOutputSupport::Unlisted
        );
        assert!(!unlisted_profile.supports_output(ArtifactOutput::Motion));
        assert!(!unlisted_profile.is_unsupported_output(ArtifactOutput::Motion));
    }

    #[test]
    fn artifact_validation_helpers_reject_invalid_profiles_and_segments() {
        let invalid_profile = ArtifactProfile::new(
            vec![ChannelKind::Longitude, ChannelKind::Longitude],
            vec![ArtifactOutput::EclipticCoordinates],
            vec![ArtifactOutput::EquatorialCoordinates],
            SpeedPolicy::Unsupported,
        );
        let profile_error = invalid_profile
            .validate()
            .expect_err("duplicate profile entries should be rejected");
        assert_eq!(profile_error.kind, CompressionErrorKind::InvalidFormat);

        let motion_policy_mismatch = ArtifactProfile::new(
            vec![ChannelKind::Longitude],
            vec![ArtifactOutput::EclipticCoordinates],
            vec![ArtifactOutput::Motion],
            SpeedPolicy::Stored,
        );
        let motion_policy_error = motion_policy_mismatch
            .validate()
            .expect_err("stored motion support should require Motion in derived outputs");
        assert_eq!(
            motion_policy_error.kind,
            CompressionErrorKind::InvalidFormat
        );

        let unsupported_motion_mismatch = ArtifactProfile::new(
            vec![ChannelKind::Longitude],
            vec![ArtifactOutput::EclipticCoordinates],
            Vec::new(),
            SpeedPolicy::Unsupported,
        );
        let unsupported_motion_error = unsupported_motion_mismatch
            .validate()
            .expect_err("unsupported motion policy should require Motion in unsupported outputs");
        assert_eq!(
            unsupported_motion_error.kind,
            CompressionErrorKind::InvalidFormat
        );

        let derived_coordinate_channel_mismatch = ArtifactProfile::new(
            vec![ChannelKind::Longitude, ChannelKind::Latitude],
            vec![ArtifactOutput::EquatorialCoordinates],
            vec![ArtifactOutput::Motion],
            SpeedPolicy::Unsupported,
        );
        let derived_coordinate_channel_error = derived_coordinate_channel_mismatch
            .validate()
            .expect_err(
            "derived coordinate outputs should require longitude, latitude, and distance channels",
        );
        assert_eq!(
            derived_coordinate_channel_error.kind,
            CompressionErrorKind::InvalidFormat
        );
        assert!(format!("{derived_coordinate_channel_error}").contains(
            "derived coordinate outputs require Longitude, Latitude, and DistanceAu in stored channels"
        ));

        let derived_coordinate_channel_missing_latitude = ArtifactProfile::new(
            vec![ChannelKind::Longitude, ChannelKind::DistanceAu],
            vec![ArtifactOutput::EquatorialCoordinates],
            vec![ArtifactOutput::Motion],
            SpeedPolicy::Unsupported,
        );
        let derived_coordinate_channel_missing_latitude_error =
            derived_coordinate_channel_missing_latitude
                .validate()
                .expect_err("derived coordinate outputs should require latitude as well");
        assert_eq!(
            derived_coordinate_channel_missing_latitude_error.kind,
            CompressionErrorKind::InvalidFormat
        );
        assert!(format!("{derived_coordinate_channel_missing_latitude_error}").contains(
            "derived coordinate outputs require Longitude, Latitude, and DistanceAu in stored channels"
        ));

        let invalid_segment_body = BodyArtifact::new(
            CelestialBody::Moon,
            vec![Segment::new(
                Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 1.0),
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 2.0, 3.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 4.0, 5.0),
                ],
            )],
        );
        let segment_error = invalid_segment_body
            .validate()
            .expect_err("duplicate segment channels should be rejected");
        assert_eq!(segment_error.kind, CompressionErrorKind::InvalidFormat);

        let invalid_artifact = CompressedArtifact::new(
            ArtifactHeader::new("duplicate body demo", "unit test duplicate body validation"),
            vec![
                BodyArtifact::new(
                    CelestialBody::Sun,
                    vec![Segment::new(
                        Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                        Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                        vec![PolynomialChannel::linear(
                            ChannelKind::Longitude,
                            9,
                            10.0,
                            11.0,
                        )],
                    )],
                ),
                BodyArtifact::new(
                    CelestialBody::Sun,
                    vec![Segment::new(
                        Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                        Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                        vec![PolynomialChannel::linear(
                            ChannelKind::Longitude,
                            9,
                            12.0,
                            13.0,
                        )],
                    )],
                ),
            ],
        );
        let artifact_error = invalid_artifact
            .validate()
            .expect_err("duplicate bodies should be rejected");
        assert_eq!(artifact_error.kind, CompressionErrorKind::InvalidFormat);

        let blank_header = ArtifactHeader::with_profile(
            "  ",
            "unit test blank header",
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        );
        let blank_header_error = blank_header
            .validate()
            .expect_err("blank generation labels should be rejected");
        assert_eq!(blank_header_error.kind, CompressionErrorKind::InvalidFormat);

        let blank_source_header = ArtifactHeader::with_profile(
            "blank source demo",
            "",
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        );
        let blank_source_error = blank_source_header
            .validate()
            .expect_err("blank sources should be rejected");
        assert_eq!(blank_source_error.kind, CompressionErrorKind::InvalidFormat);

        let padded_header = ArtifactHeader::with_profile(
            " padded demo ",
            "unit test padded header",
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        );
        let padded_header_error = padded_header
            .validate()
            .expect_err("padded generation labels should be rejected");
        assert_eq!(
            padded_header_error.kind,
            CompressionErrorKind::InvalidFormat
        );

        let padded_source_header = ArtifactHeader::with_profile(
            "padded source demo",
            " unit test padded source ",
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        );
        let padded_source_error = padded_source_header
            .validate()
            .expect_err("padded sources should be rejected");
        assert_eq!(
            padded_source_error.kind,
            CompressionErrorKind::InvalidFormat
        );

        let blank_header_artifact = CompressedArtifact::new(
            ArtifactHeader::with_profile(
                " ",
                "unit test blank header artifact",
                ArtifactProfile::ecliptic_longitude_latitude_distance(),
            ),
            Vec::new(),
        );
        let blank_header_artifact_error = blank_header_artifact
            .validate()
            .expect_err("artifact validation should reject blank header metadata");
        assert_eq!(
            blank_header_artifact_error.kind,
            CompressionErrorKind::InvalidFormat
        );

        let blank_header_encode_error = CompressedArtifact::new(
            ArtifactHeader::with_profile(
                " ",
                "unit test blank header encode",
                ArtifactProfile::ecliptic_longitude_latitude_distance(),
            ),
            Vec::new(),
        )
        .encode()
        .expect_err("artifact encoding should reject blank header metadata");
        assert_eq!(
            blank_header_encode_error.kind,
            CompressionErrorKind::InvalidFormat
        );
    }

    #[test]
    fn artifact_encoding_rejects_duplicate_profile_entries() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::with_profile(
                "duplicate profile demo",
                "unit test duplicate profile entries",
                ArtifactProfile::new(
                    vec![ChannelKind::Longitude, ChannelKind::Longitude],
                    vec![ArtifactOutput::EclipticCoordinates],
                    vec![ArtifactOutput::EquatorialCoordinates],
                    SpeedPolicy::Unsupported,
                ),
            ),
            Vec::new(),
        );

        let error = artifact
            .encode()
            .expect_err("duplicate profile entries should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    }

    #[test]
    fn artifact_encoding_rejects_duplicate_segment_channels() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new(
                "duplicate segment demo",
                "unit test duplicate segment channels",
            ),
            vec![BodyArtifact::new(
                CelestialBody::Moon,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 1.0),
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 2.0, 3.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 4.0, 5.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                    ],
                )],
            )],
        );

        let error = artifact
            .encode()
            .expect_err("duplicate segment channels should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    }

    #[test]
    fn artifact_encoding_rejects_mismatched_segment_scales() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new(
                "mismatched segment scales demo",
                "unit test mismatched segment scales",
            ),
            vec![BodyArtifact::new(
                CelestialBody::Moon,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tdb),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 1.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 2.0, 3.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                    ],
                )],
            )],
        );

        let error = artifact
            .encode()
            .expect_err("mismatched segment scales should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    }

    #[test]
    fn artifact_decoding_rejects_duplicate_body_entries() {
        let profile = ArtifactProfile::ecliptic_longitude_latitude_distance();
        let mut payload = Vec::new();
        write_string(&mut payload, "duplicate body decode demo");
        write_string(&mut payload, "unit test duplicate body decode");
        write_u8(
            &mut payload,
            encode_endian_policy(EndianPolicy::LittleEndian),
        );
        encode_artifact_profile(&mut payload, &profile).expect("profile should encode");
        write_u16(&mut payload, 2);
        encode_celestial_body(&mut payload, &CelestialBody::Sun).expect("Sun should encode");
        write_u16(&mut payload, 0);
        encode_celestial_body(&mut payload, &CelestialBody::Sun).expect("Sun should encode");
        write_u16(&mut payload, 0);

        let checksum = fnv1a64(&payload);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ARTIFACT_MAGIC);
        write_u16(&mut bytes, ARTIFACT_VERSION);
        write_u64(&mut bytes, checksum);
        bytes.extend_from_slice(&payload);

        let error = CompressedArtifact::decode(&bytes)
            .expect_err("duplicate body entries should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    }

    #[test]
    fn residual_channels_are_applied_during_lookup() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("residual demo", "unit test residual channels"),
            vec![BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::with_residual_channels(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0),
                        PolynomialChannel::new(ChannelKind::Latitude, 9, vec![1.0]),
                        PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![2.0]),
                    ],
                    vec![
                        PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.25]),
                        PolynomialChannel::new(ChannelKind::Latitude, 9, vec![0.0]),
                        PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![-0.1]),
                    ],
                )],
            )],
        );

        let lookup = artifact
            .lookup_ecliptic(
                &CelestialBody::Sun,
                Instant::new(pleiades_types::JulianDay::from_days(0.5), TimeScale::Tt),
            )
            .expect("residual-corrected lookup should succeed");

        assert!((lookup.longitude.degrees() - 10.75).abs() < 1e-12);
        assert!((lookup.latitude.degrees() - 1.0).abs() < 1e-12);
        assert!((lookup.distance_au.unwrap() - 1.9).abs() < 1e-12);
    }

    #[test]
    fn residual_channels_roundtrip_through_the_codec() {
        let segment = Segment::with_residual_channels(
            Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
            Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
            ],
            vec![
                PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![-0.25]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![0.125]),
            ],
        );
        segment
            .validate()
            .expect("segment metadata should validate");
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("residual roundtrip demo", "unit test residual roundtrip"),
            vec![BodyArtifact::new(CelestialBody::Sun, vec![segment.clone()])],
        );

        let decoded = CompressedArtifact::decode(
            &artifact
                .encode()
                .expect("artifact with residual channels should encode"),
        )
        .expect("artifact with residual channels should decode");

        assert_eq!(decoded.bodies[0].segments[0], segment);
    }

    #[test]
    fn segment_summary_line_reports_stored_and_residual_channels() {
        let segment = Segment::with_residual_channels(
            Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
            Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
            ],
            vec![
                PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![-0.25]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![0.125]),
            ],
        );

        let expected = format!(
            "start: {}; end: {}; stored channels: [Longitude, Latitude, DistanceAu]; residual channels: [Longitude, Latitude, DistanceAu]",
            segment.start, segment.end
        );
        assert_eq!(segment.summary_line(), expected);
        assert_eq!(segment.to_string(), expected);
    }

    #[test]
    fn body_artifact_summary_line_reports_body_and_residual_segments() {
        let body_artifact = BodyArtifact::new(
            CelestialBody::Sun,
            vec![
                Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0),
                        PolynomialChannel::new(ChannelKind::Latitude, 9, vec![1.0]),
                        PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![2.0]),
                    ],
                ),
                Segment::with_residual_channels(
                    Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                        PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
                        PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
                    ],
                    vec![PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5])],
                ),
            ],
        );

        let expected = "body: Sun; segments: 2; residual-bearing segments: 1";
        assert_eq!(body_artifact.summary_line(), expected);
        assert_eq!(body_artifact.to_string(), expected);
    }

    #[test]
    fn segment_validate_rejects_duplicate_residual_channels() {
        let segment = Segment::with_residual_channels(
            Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
            Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
            ],
            vec![
                PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
                PolynomialChannel::new(ChannelKind::Longitude, 9, vec![-0.25]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![0.125]),
            ],
        );

        let error = segment
            .validate()
            .expect_err("duplicate residual channels should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error.to_string().contains("segment residual channels"));
    }

    #[test]
    fn segment_validate_rejects_unsorted_stored_channels() {
        let segment = Segment::new(
            Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
            Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
            ],
        );

        let error = segment
            .validate()
            .expect_err("stored channels should be ordered canonically by channel kind");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error
            .to_string()
            .contains("segment stored channels must be ordered by channel kind"));
    }

    #[test]
    fn segment_validate_rejects_residual_channels_without_matching_stored_channels() {
        let segment = Segment::with_residual_channels(
            Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
            Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
            ],
            vec![
                PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![-0.25]),
            ],
        );

        let error = segment
            .validate()
            .expect_err("residual channels should require matching stored channels");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error
            .to_string()
            .contains("segment residual channels require a matching stored channel"));
    }

    #[test]
    fn segment_validate_rejects_non_finite_bounds() {
        let segment = Segment::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(f64::INFINITY),
                TimeScale::Tt,
            ),
            Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
            ],
        );

        let error = segment
            .validate()
            .expect_err("non-finite segment bounds should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error.message.contains("segment start must be finite"));
    }

    #[test]
    fn segment_validate_rejects_non_finite_channel_coefficients() {
        let segment = Segment::new(
            Instant::new(pleiades_types::JulianDay::from_days(10.0), TimeScale::Tt),
            Instant::new(pleiades_types::JulianDay::from_days(11.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                PolynomialChannel::new(ChannelKind::Latitude, 9, vec![f64::NAN]),
                PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
            ],
        );

        let error = segment
            .validate()
            .expect_err("non-finite channel coefficients should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error
            .message
            .contains("polynomial channel Latitude contains a non-finite coefficient at index 0"));
    }

    #[test]
    fn body_validate_rejects_overlapping_segments_for_the_same_scale() {
        let body = BodyArtifact::new(
            CelestialBody::Moon,
            vec![
                Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 20.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                    ],
                ),
                Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(1.5), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(3.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 30.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.3, 0.4),
                    ],
                ),
            ],
        );

        let error = body
            .validate()
            .expect_err("overlapping same-scale segments should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error.message.contains("ordered and non-overlapping"));
    }

    #[test]
    fn body_validate_allows_shared_boundary_segments_for_the_same_scale() {
        let body = BodyArtifact::new(
            CelestialBody::Moon,
            vec![
                Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 10.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                    ],
                ),
                Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 30.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.3, 0.4),
                    ],
                ),
            ],
        );

        assert!(body.validate().is_ok());
    }

    #[test]
    fn explicit_endian_policy_roundtrip_preserves_header_metadata() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::with_profile_and_endian(
                "endian demo",
                "unit test endian policy",
                EndianPolicy::LittleEndian,
                ArtifactProfile::ecliptic_longitude_latitude_distance(),
            ),
            Vec::new(),
        );

        let decoded = CompressedArtifact::decode(
            &artifact
                .encode()
                .expect("artifact should encode with explicit endian policy"),
        )
        .expect("artifact should decode with explicit endian policy");

        assert_eq!(decoded.header.endian_policy, EndianPolicy::LittleEndian);
        assert_eq!(decoded.header.generation_label, "endian demo");
    }

    #[test]
    fn artifact_profile_summary_lists_capability_fields() {
        let profile = ArtifactProfile::new(
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu,
            ],
            vec![
                ArtifactOutput::EclipticCoordinates,
                ArtifactOutput::EquatorialCoordinates,
            ],
            vec![
                ArtifactOutput::ApparentCorrections,
                ArtifactOutput::TopocentricCoordinates,
                ArtifactOutput::SiderealCoordinates,
                ArtifactOutput::Motion,
            ],
            SpeedPolicy::Unsupported,
        );
        let coverage = ArtifactProfileCoverageSummary::new(
            profile.clone(),
            vec![CelestialBody::Sun, CelestialBody::Moon],
        );

        assert_eq!(
            profile.summary(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
        assert_eq!(
            profile.summary_line(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
        assert_eq!(
            profile.summary_for_body_count(11),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies"
        );
        assert_eq!(
            profile.summary_line_with_body_count(11),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies"
        );
        assert_eq!(
            profile.to_string(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
        assert_eq!(
            profile.output_support_entries_summary_line(),
            "EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported"
        );
        assert_eq!(
            profile.output_support_summary_line(),
            "output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported"
        );
        assert_eq!(coverage.body_count, 2);
        assert_eq!(
            coverage.bodies,
            vec![CelestialBody::Sun, CelestialBody::Moon]
        );
        assert_eq!(coverage.profile, profile);
        assert_eq!(
            coverage.summary_line(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies"
        );
        assert_eq!(
            coverage.summary_line_with_bodies(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies; bundled bodies: Sun, Moon"
        );
        assert_eq!(
            coverage.to_string(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies"
        );
        coverage
            .validate()
            .expect("coverage summary should validate");

        let header = ArtifactHeader::with_profile_and_endian(
            "demo",
            "source",
            EndianPolicy::LittleEndian,
            profile,
        );
        assert_eq!(
            header.summary(),
            "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
        assert_eq!(
            header.summary_line(),
            "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
        assert_eq!(
            header.summary_for_body_count(11),
            "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies"
        );
        assert_eq!(
            header.to_string(),
            "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
    }

    #[test]
    fn artifact_profile_coverage_validation_rejects_body_count_drift() {
        let mut coverage = ArtifactProfileCoverageSummary::new(
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
            vec![CelestialBody::Sun, CelestialBody::Moon],
        );
        coverage.body_count += 1;

        let error = coverage
            .validate()
            .expect_err("body-count drift should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error
            .message
            .contains("artifact profile coverage body count does not match bundled body list"));
    }

    #[test]
    fn artifact_profile_coverage_validation_rejects_empty_bodies() {
        let coverage = ArtifactProfileCoverageSummary::new(
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
            Vec::new(),
        );

        let error = coverage
            .validate()
            .expect_err("empty bundled-body lists should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error
            .message
            .contains("artifact profile coverage bundled body list must not be empty"));
    }

    #[test]
    fn artifact_profile_coverage_validation_rejects_duplicate_bodies() {
        let coverage = ArtifactProfileCoverageSummary::new(
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
            vec![CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Sun],
        );

        let error = coverage
            .validate()
            .expect_err("duplicate bundled bodies should be rejected");
        assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
        assert!(error
            .message
            .contains("artifact profile coverage bundled bodies contains duplicate Sun entry"));
    }

    #[test]
    fn artifact_profile_labels_are_stable() {
        assert_eq!(ChannelKind::Longitude.label(), "Longitude");
        assert_eq!(ChannelKind::Latitude.label(), "Latitude");
        assert_eq!(ChannelKind::DistanceAu.label(), "DistanceAu");

        assert_eq!(
            ArtifactOutput::EclipticCoordinates.label(),
            "EclipticCoordinates"
        );
        assert_eq!(
            ArtifactOutput::EquatorialCoordinates.label(),
            "EquatorialCoordinates"
        );
        assert_eq!(
            ArtifactOutput::ApparentCorrections.label(),
            "ApparentCorrections"
        );
        assert_eq!(
            ArtifactOutput::TopocentricCoordinates.label(),
            "TopocentricCoordinates"
        );
        assert_eq!(
            ArtifactOutput::SiderealCoordinates.label(),
            "SiderealCoordinates"
        );
        assert_eq!(ArtifactOutput::Motion.label(), "Motion");

        assert_eq!(SpeedPolicy::Unsupported.label(), "Unsupported");
        assert_eq!(SpeedPolicy::Stored.label(), "Stored");
        assert_eq!(SpeedPolicy::FittedDerivative.label(), "FittedDerivative");
        assert_eq!(
            SpeedPolicy::NumericalDifference.label(),
            "NumericalDifference"
        );

        assert_eq!(ChannelKind::Longitude.to_string(), "Longitude");
        assert_eq!(ArtifactOutput::Motion.to_string(), "Motion");
        assert_eq!(SpeedPolicy::Stored.to_string(), "Stored");
        assert_eq!(
            join_display(&[CelestialBody::Sun, CelestialBody::Moon]),
            "Sun, Moon"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_preserves_artifacts() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("serde demo", "json roundtrip fixture"),
            vec![BodyArtifact::new(
                CelestialBody::Moon,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        15.0,
                        30.0,
                    )],
                )],
            )],
        );

        let decoded: CompressedArtifact = serde_json::from_value(
            serde_json::to_value(&artifact).expect("artifact should serialize"),
        )
        .expect("artifact should deserialize");
        assert_eq!(decoded, artifact);
    }

    #[test]
    fn lookup_interpolates_segment_channels() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("demo", "lookup fixture"),
            vec![BodyArtifact::new(
                CelestialBody::Moon,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 20.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                    ],
                )],
            )],
        );

        let result = artifact
            .lookup_ecliptic(
                &CelestialBody::Moon,
                Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
            )
            .expect("lookup should work");

        assert_eq!(result.longitude.degrees(), 10.0);
        assert_eq!(result.latitude.degrees(), 2.0);
        assert_eq!(result.distance_au, Some(0.15000000000000002));
    }

    #[test]
    fn lookup_rejects_missing_body() {
        let artifact = CompressedArtifact::new(ArtifactHeader::new("demo", "x"), Vec::new());
        let error = artifact
            .lookup_ecliptic(
                &CelestialBody::Sun,
                Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
            )
            .expect_err("missing bodies should error");
        assert_eq!(error.kind, CompressionErrorKind::MissingBody);
    }

    #[test]
    fn random_access_helpers_return_body_and_segment_matches() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("demo", "segment access fixture"),
            vec![BodyArtifact::new(
                CelestialBody::Moon,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        0.0,
                        20.0,
                    )],
                )],
            )],
        );

        let body = artifact
            .body_artifact(&CelestialBody::Moon)
            .expect("body lookup should work");
        assert_eq!(body.body, CelestialBody::Moon);
        assert_eq!(body.segments.len(), 1);

        let segment = artifact
            .segment_for(
                &CelestialBody::Moon,
                Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
            )
            .expect("segment lookup should work");
        assert_eq!(segment.start.julian_day.days(), 0.0);
        assert_eq!(segment.end.julian_day.days(), 2.0);
        assert!(body
            .segment_at(Instant::new(
                pleiades_types::JulianDay::from_days(0.0),
                TimeScale::Tt
            ))
            .is_some());
        assert!(body
            .segment_at(Instant::new(
                pleiades_types::JulianDay::from_days(2.0),
                TimeScale::Tt
            ))
            .is_some());
        assert!(body
            .segment_at(Instant::new(
                pleiades_types::JulianDay::from_days(2.1),
                TimeScale::Tt
            ))
            .is_none());

        let error = artifact
            .segment_for(
                &CelestialBody::Moon,
                Instant::new(pleiades_types::JulianDay::from_days(2.1), TimeScale::Tt),
            )
            .expect_err("out-of-range instant should error");
        assert_eq!(error.kind, CompressionErrorKind::OutOfRangeInstant);
    }

    #[test]
    fn random_access_helpers_prefer_the_later_segment_on_shared_boundaries() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("demo", "boundary fixture"),
            vec![BodyArtifact::new(
                CelestialBody::Moon,
                vec![
                    Segment::new(
                        Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                        Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                        vec![
                            PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 10.0),
                            PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                            PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                        ],
                    ),
                    Segment::new(
                        Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt),
                        Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                        vec![
                            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 30.0),
                            PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
                            PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.3, 0.4),
                        ],
                    ),
                ],
            )],
        );

        let shared_boundary =
            Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt);
        let segment = artifact
            .segment_for(&CelestialBody::Moon, shared_boundary)
            .expect("shared boundary should resolve to the later segment");
        assert_eq!(segment.start.julian_day.days(), 1.0);
        assert_eq!(segment.end.julian_day.days(), 2.0);

        let ecliptic = artifact
            .lookup_ecliptic(&CelestialBody::Moon, shared_boundary)
            .expect("boundary lookup should succeed");
        assert_eq!(ecliptic.longitude.degrees(), 20.0);
        assert_eq!(ecliptic.latitude.degrees(), 3.0);
        assert_eq!(ecliptic.distance_au, Some(0.3));
    }

    #[test]
    fn encode_decode_roundtrip_preserves_lunar_apogee_and_perigee_bodies() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("demo", "lunar point fixture"),
            vec![
                BodyArtifact::new(CelestialBody::MeanApogee, Vec::new()),
                BodyArtifact::new(CelestialBody::TrueApogee, Vec::new()),
                BodyArtifact::new(CelestialBody::MeanPerigee, Vec::new()),
                BodyArtifact::new(CelestialBody::TruePerigee, Vec::new()),
            ],
        );

        let encoded = artifact.encode().expect("artifact should encode");
        let decoded = CompressedArtifact::decode(&encoded).expect("artifact should decode");

        assert_eq!(decoded.bodies.len(), 4);
        assert_eq!(decoded.bodies[0].body, CelestialBody::MeanApogee);
        assert_eq!(decoded.bodies[1].body, CelestialBody::TrueApogee);
        assert_eq!(decoded.bodies[2].body, CelestialBody::MeanPerigee);
        assert_eq!(decoded.bodies[3].body, CelestialBody::TruePerigee);
    }

    #[test]
    fn lookup_ecliptic_requires_the_profile_to_advertise_it() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::with_profile(
                "demo",
                "ecliptic lookup fixture",
                ArtifactProfile::new(
                    vec![
                        ChannelKind::Longitude,
                        ChannelKind::Latitude,
                        ChannelKind::DistanceAu,
                    ],
                    vec![ArtifactOutput::EquatorialCoordinates],
                    vec![
                        ArtifactOutput::ApparentCorrections,
                        ArtifactOutput::TopocentricCoordinates,
                        ArtifactOutput::SiderealCoordinates,
                        ArtifactOutput::Motion,
                    ],
                    SpeedPolicy::Unsupported,
                ),
            ),
            vec![BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.5, 0.75),
                    ],
                )],
            )],
        );

        let instant = Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt);
        let error = artifact
            .lookup_ecliptic(&CelestialBody::Sun, instant)
            .expect_err("ecliptic lookup should respect the advertised profile");

        assert_eq!(
            error.message,
            "artifact profile does not derive EclipticCoordinates"
        );
    }

    #[test]
    fn lookup_equatorial_requires_the_profile_to_advertise_it() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::with_profile(
                "demo",
                "equatorial lookup fixture",
                ArtifactProfile::new(
                    vec![
                        ChannelKind::Longitude,
                        ChannelKind::Latitude,
                        ChannelKind::DistanceAu,
                    ],
                    vec![ArtifactOutput::EclipticCoordinates],
                    vec![
                        ArtifactOutput::ApparentCorrections,
                        ArtifactOutput::TopocentricCoordinates,
                        ArtifactOutput::SiderealCoordinates,
                        ArtifactOutput::Motion,
                    ],
                    SpeedPolicy::Unsupported,
                ),
            ),
            vec![BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.5, 0.75),
                    ],
                )],
            )],
        );

        let instant = Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt);
        let obliquity = Angle::from_degrees(23.439_291_11);
        let error = artifact
            .lookup_equatorial(&CelestialBody::Sun, instant, obliquity)
            .expect_err("equatorial lookup should respect the advertised profile");

        assert_eq!(
            error.message,
            "artifact profile does not derive EquatorialCoordinates"
        );
    }

    #[test]
    fn lookup_equatorial_reconstructs_derived_coordinates() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("demo", "equatorial lookup fixture"),
            vec![BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(pleiades_types::JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(pleiades_types::JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.5, 0.75),
                    ],
                )],
            )],
        );

        let instant = Instant::new(pleiades_types::JulianDay::from_days(1.0), TimeScale::Tt);
        let obliquity = Angle::from_degrees(23.439_291_11);
        let ecliptic = artifact
            .lookup_ecliptic(&CelestialBody::Sun, instant)
            .expect("ecliptic lookup should succeed");
        let equatorial = artifact
            .lookup_equatorial(&CelestialBody::Sun, instant, obliquity)
            .expect("equatorial lookup should succeed");

        assert_eq!(equatorial, ecliptic.to_equatorial(obliquity));
        assert_eq!(equatorial.distance_au, Some(0.625));
    }
}
