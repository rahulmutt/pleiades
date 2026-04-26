//! Compression codecs and artifact packing helpers for ephemeris data.
//!
//! The current implementation defines a small, deterministic artifact format
//! with explicit versioning, checksums, artifact capability profiles, and
//! quantized polynomial segments. It is intentionally simple enough to audit
//! while still exercising the same segmented lookup flow and random-access
//! body/segment helpers that later, denser artifacts will use.
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

use pleiades_types::{
    CelestialBody, CustomBodyId, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    TimeScale,
};

/// Current artifact format version.
pub const ARTIFACT_VERSION: u16 = 3;

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

    /// Returns a compact one-line summary of the stored, derived, unsupported,
    /// and speed-policy capabilities encoded by this profile.
    pub fn summary(&self) -> String {
        format!(
            "stored channels: {:?}; derived outputs: {:?}; unsupported outputs: {:?}; speed policy: {:?}",
            self.stored_channels,
            self.derived_outputs,
            self.unsupported_outputs,
            self.speed_policy,
        )
    }

    /// Returns the capability summary annotated with how many bodies share it.
    pub fn summary_for_body_count(&self, body_count: usize) -> String {
        format!(
            "{}; applies to {} bundled bodies",
            self.summary(),
            body_count
        )
    }

    /// Returns the current conservative profile: ecliptic longitude, latitude,
    /// and distance are stored directly; motion and richer coordinate modes are unsupported.
    pub fn ecliptic_longitude_latitude_distance() -> Self {
        Self::new(
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu,
            ],
            vec![ArtifactOutput::EclipticCoordinates],
            vec![
                ArtifactOutput::EquatorialCoordinates,
                ArtifactOutput::ApparentCorrections,
                ArtifactOutput::TopocentricCoordinates,
                ArtifactOutput::SiderealCoordinates,
                ArtifactOutput::Motion,
            ],
            SpeedPolicy::Unsupported,
        )
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
}

impl Segment {
    /// Creates a new segment.
    pub fn new(start: Instant, end: Instant, channels: Vec<PolynomialChannel>) -> Self {
        Self {
            start,
            end,
            channels,
        }
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

    fn evaluate_channel(&self, kind: ChannelKind, x: f64) -> Result<f64, CompressionError> {
        self.channel(kind)
            .map(|channel| channel.evaluate(x))
            .ok_or_else(|| {
                CompressionError::new(
                    CompressionErrorKind::MissingChannel,
                    format!("missing {kind:?} channel"),
                )
            })
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

    /// Returns the segment covering the requested instant, if any.
    pub fn segment_at(&self, instant: Instant) -> Option<&Segment> {
        self.segments
            .iter()
            .find(|segment| segment.contains(instant))
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

        Ok(Self {
            header,
            checksum,
            bodies,
        })
    }

    /// Returns the ecliptic coordinates for a body at a given instant.
    pub fn lookup_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, CompressionError> {
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

    fn encode_payload(&self) -> Result<Vec<u8>, CompressionError> {
        let mut bytes = Vec::new();
        write_string(&mut bytes, &self.header.generation_label);
        write_string(&mut bytes, &self.header.source);
        write_u8(&mut bytes, encode_endian_policy(self.header.endian_policy));
        encode_artifact_profile(&mut bytes, &self.header.profile);
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

fn encode_artifact_profile(bytes: &mut Vec<u8>, profile: &ArtifactProfile) {
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
    Ok(ArtifactProfile::new(
        stored_channels,
        derived_outputs,
        unsupported_outputs,
        speed_policy,
    ))
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
    encode_instant(bytes, segment.start);
    encode_instant(bytes, segment.end);
    write_u8(bytes, segment.channels.len() as u8);
    for channel in &segment.channels {
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
    }
    Ok(())
}

fn decode_segment(cursor: &mut Cursor<'_>) -> Result<Segment, CompressionError> {
    let start = decode_instant(cursor)?;
    let end = decode_instant(cursor)?;
    let channel_count = cursor.read_u8()? as usize;
    let mut channels = Vec::with_capacity(channel_count);
    for _ in 0..channel_count {
        let kind = decode_channel_kind(cursor.read_u8()?)?;
        let scale_exponent = cursor.read_u8()?;
        let coefficient_count = cursor.read_u8()? as usize;
        let scale = 10f64.powi(scale_exponent as i32);
        let mut coefficients = Vec::with_capacity(coefficient_count);
        for _ in 0..coefficient_count {
            coefficients.push(cursor.read_i64()? as f64 / scale);
        }
        channels.push(PolynomialChannel::new(kind, scale_exponent, coefficients));
    }
    Ok(Segment::new(start, end, channels))
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
            vec![ChannelKind::Longitude, ChannelKind::Latitude],
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
            vec![ArtifactOutput::EclipticCoordinates],
            vec![
                ArtifactOutput::EquatorialCoordinates,
                ArtifactOutput::ApparentCorrections,
                ArtifactOutput::TopocentricCoordinates,
                ArtifactOutput::SiderealCoordinates,
                ArtifactOutput::Motion,
            ],
            SpeedPolicy::Unsupported,
        );

        assert_eq!(
            profile.summary(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates]; unsupported outputs: [EquatorialCoordinates, ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
        assert_eq!(
            profile.summary_for_body_count(11),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates]; unsupported outputs: [EquatorialCoordinates, ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies"
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
}
