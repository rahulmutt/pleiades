//! The top-level `CompressedArtifact` type with encode/decode/lookup.

use core::fmt;

use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, EquatorialCoordinates, Instant, TimeScale,
};

use crate::channels::{BodyArtifact, ChannelKind};
use crate::codec::validate_body_artifacts;
use crate::codec::{
    decode_artifact_profile, decode_body, decode_endian_policy, encode_artifact_profile,
    encode_body, encode_endian_policy, fnv1a64, write_u16, write_u64, Cursor,
};
use crate::error::{CompressionError, CompressionErrorKind};
use crate::format::{
    ArtifactHeader, ArtifactOutput, ArtifactProfileCoverageSummary,
    ArtifactResidualBodyCoverageSummary,
};

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
    /// This checks the header metadata, the bundled body/profile coverage,
    /// canonical body ordering, duplicate body entries, and every body
    /// segment's metadata. It is useful for generators that want to fail fast
    /// before writing a deterministic binary payload.
    pub fn validate(&self) -> Result<(), CompressionError> {
        self.header.validate()?;
        validate_body_artifacts(&self.bodies)?;
        self.profile_coverage_summary().validate()?;
        for body in &self.bodies {
            body.validate()?;
        }

        Ok(())
    }

    /// Returns the artifact profile paired with the bundled body list.
    pub fn profile_coverage_summary(&self) -> ArtifactProfileCoverageSummary {
        ArtifactProfileCoverageSummary::new(
            self.header.profile.clone(),
            self.bodies.iter().map(|body| body.body.clone()).collect(),
        )
    }

    /// Returns the bodies that include at least one residual-correction segment.
    pub fn residual_body_coverage_summary(&self) -> ArtifactResidualBodyCoverageSummary {
        ArtifactResidualBodyCoverageSummary::new(self.residual_bodies())
    }

    /// Returns the body artifact for the requested body, if present.
    pub fn body_artifact(&self, body: &CelestialBody) -> Option<&BodyArtifact> {
        self.bodies.iter().find(|series| &series.body == body)
    }

    /// Returns the total number of segments stored in the artifact.
    pub fn segment_count(&self) -> usize {
        self.bodies.iter().map(|body| body.segments.len()).sum()
    }

    /// Returns the number of segments that carry residual-correction channels.
    pub fn residual_segment_count(&self) -> usize {
        self.bodies
            .iter()
            .flat_map(|body| body.segments.iter())
            .filter(|segment| !segment.residual_channels.is_empty())
            .count()
    }

    /// Returns the bundled bodies that include at least one residual-correction segment.
    pub fn residual_bodies(&self) -> Vec<CelestialBody> {
        self.bodies
            .iter()
            .filter(|body| {
                body.segments
                    .iter()
                    .any(|segment| !segment.residual_channels.is_empty())
            })
            .map(|body| body.body.clone())
            .collect()
    }

    /// Returns a compact one-line summary of the artifact body, segment, and
    /// residual-correction coverage.
    pub fn summary_line(&self) -> String {
        let residual_bodies = self.residual_bodies();
        let residual_bodies = if residual_bodies.is_empty() {
            "none".to_string()
        } else {
            crate::join_display(&residual_bodies)
        };

        format!(
            "bodies: {}; segments: {}; residual-bearing segments: {}; residual-bearing bodies: {}",
            self.bodies.len(),
            self.segment_count(),
            self.residual_segment_count(),
            residual_bodies,
        )
    }

    /// Returns the body segment covering the requested instant.
    pub fn segment_for(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<&crate::channels::Segment, CompressionError> {
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
        bytes.extend_from_slice(&crate::ARTIFACT_MAGIC);
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
        if magic != crate::ARTIFACT_MAGIC {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidMagic,
                "compressed artifact magic header did not match",
            ));
        }

        let version = cursor.read_u16()?;
        if version != crate::ARTIFACT_VERSION {
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

        use pleiades_types::{Latitude, Longitude};
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
                format!("artifact profile does not support {output}"),
            ))
        }
    }

    fn encode_payload(&self) -> Result<Vec<u8>, CompressionError> {
        self.validate()?;

        let mut bytes = Vec::new();
        crate::codec::write_string(&mut bytes, &self.header.generation_label);
        crate::codec::write_string(&mut bytes, &self.header.source);
        crate::codec::write_u8(&mut bytes, encode_endian_policy(self.header.endian_policy));
        encode_artifact_profile(&mut bytes, &self.header.profile)?;
        write_u16(&mut bytes, self.bodies.len() as u16);
        for body in &self.bodies {
            encode_body(&mut bytes, body)?;
        }
        Ok(bytes)
    }
}

impl fmt::Display for CompressedArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
