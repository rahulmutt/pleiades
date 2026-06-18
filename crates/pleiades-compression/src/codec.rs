//! Low-level binary encoding/decoding primitives and shared validation helpers.

use core::fmt;
use std::collections::HashMap;

use pleiades_types::{CelestialBody, CustomBodyId, Instant, JulianDay, TimeScale};

use crate::channels::{ChannelKind, PolynomialChannel, Segment};
use crate::error::{CompressionError, CompressionErrorKind};
use crate::format::{
    ArtifactOutput, ArtifactOutputSupport, ArtifactProfile, EndianPolicy, SpeedPolicy,
};

// ── Write primitives ─────────────────────────────────────────────────────────

pub(crate) fn write_string(bytes: &mut Vec<u8>, value: &str) {
    write_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

pub(crate) fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

pub(crate) fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn write_i64(bytes: &mut Vec<u8>, value: i64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn write_f64(bytes: &mut Vec<u8>, value: f64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

// ── Cursor (read helper) ─────────────────────────────────────────────────────

pub(crate) struct Cursor<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) offset: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(crate) fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.offset..]
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.offset >= self.bytes.len()
    }

    pub(crate) fn read_array<const N: usize>(&mut self) -> Result<[u8; N], CompressionError> {
        let bytes = self.read_exact(N)?;
        let mut array = [0u8; N];
        array.copy_from_slice(bytes);
        Ok(array)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, CompressionError> {
        Ok(self.read_exact(1)?[0])
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16, CompressionError> {
        Ok(u16::from_le_bytes(self.read_array()?))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, CompressionError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64, CompressionError> {
        Ok(u64::from_le_bytes(self.read_array()?))
    }

    pub(crate) fn read_i64(&mut self) -> Result<i64, CompressionError> {
        Ok(i64::from_le_bytes(self.read_array()?))
    }

    pub(crate) fn read_f64(&mut self) -> Result<f64, CompressionError> {
        Ok(f64::from_le_bytes(self.read_array()?))
    }

    pub(crate) fn read_string(&mut self) -> Result<String, CompressionError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|error| {
            CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("compressed artifact string was not valid UTF-8: {error}"),
            )
        })
    }

    pub(crate) fn read_exact(&mut self, len: usize) -> Result<&'a [u8], CompressionError> {
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

// ── FNV-1a checksum ──────────────────────────────────────────────────────────

pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ── Tag encode/decode for discriminated types ─────────────────────────────────

pub(crate) fn encode_endian_policy(policy: EndianPolicy) -> u8 {
    match policy {
        EndianPolicy::LittleEndian => 0,
    }
}

pub(crate) fn decode_endian_policy(value: u8) -> Result<EndianPolicy, CompressionError> {
    match value {
        0 => Ok(EndianPolicy::LittleEndian),
        other => Err(CompressionError::new(
            CompressionErrorKind::UnsupportedEndianPolicy,
            format!("artifact byte-order policy {other} is not supported"),
        )),
    }
}

pub(crate) fn encode_channel_kind(kind: ChannelKind) -> u8 {
    match kind {
        ChannelKind::Longitude => 0,
        ChannelKind::Latitude => 1,
        ChannelKind::DistanceAu => 2,
    }
}

pub(crate) fn decode_channel_kind(tag: u8) -> Result<ChannelKind, CompressionError> {
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

pub(crate) fn encode_artifact_output(output: ArtifactOutput) -> u8 {
    match output {
        ArtifactOutput::EclipticCoordinates => 0,
        ArtifactOutput::EquatorialCoordinates => 1,
        ArtifactOutput::ApparentCorrections => 2,
        ArtifactOutput::TopocentricCoordinates => 3,
        ArtifactOutput::SiderealCoordinates => 4,
        ArtifactOutput::Motion => 5,
    }
}

pub(crate) fn decode_artifact_output(tag: u8) -> Result<ArtifactOutput, CompressionError> {
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

pub(crate) fn encode_speed_policy(policy: SpeedPolicy) -> u8 {
    match policy {
        SpeedPolicy::Unsupported => 0,
        SpeedPolicy::Stored => 1,
        SpeedPolicy::FittedDerivative => 2,
        SpeedPolicy::NumericalDifference => 3,
    }
}

pub(crate) fn decode_speed_policy(tag: u8) -> Result<SpeedPolicy, CompressionError> {
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

pub(crate) fn encode_time_scale(scale: TimeScale) -> u8 {
    match scale {
        TimeScale::Utc => 0,
        TimeScale::Ut1 => 1,
        TimeScale::Tt => 2,
        TimeScale::Tdb => 3,
        _ => 255,
    }
}

pub(crate) fn decode_time_scale(tag: u8) -> Result<TimeScale, CompressionError> {
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

// ── Struct encode/decode ──────────────────────────────────────────────────────

pub(crate) fn encode_artifact_profile(
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

pub(crate) fn decode_artifact_profile(
    cursor: &mut Cursor<'_>,
) -> Result<ArtifactProfile, CompressionError> {
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

pub(crate) fn encode_instant(bytes: &mut Vec<u8>, instant: Instant) {
    write_f64(bytes, instant.julian_day.days());
    write_u8(bytes, encode_time_scale(instant.scale));
}

pub(crate) fn decode_instant(cursor: &mut Cursor<'_>) -> Result<Instant, CompressionError> {
    let julian_day = cursor.read_f64()?;
    let scale = decode_time_scale(cursor.read_u8()?)?;
    Ok(Instant::new(JulianDay::from_days(julian_day), scale))
}

pub(crate) fn encode_polynomial_channel(
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

pub(crate) fn decode_polynomial_channel(
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

pub(crate) fn encode_celestial_body(
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

pub(crate) fn decode_celestial_body(
    cursor: &mut Cursor<'_>,
) -> Result<CelestialBody, CompressionError> {
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

pub(crate) fn encode_segment(
    bytes: &mut Vec<u8>,
    segment: &Segment,
) -> Result<(), CompressionError> {
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

pub(crate) fn decode_segment(cursor: &mut Cursor<'_>) -> Result<Segment, CompressionError> {
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

pub(crate) fn encode_body(
    bytes: &mut Vec<u8>,
    body: &crate::channels::BodyArtifact,
) -> Result<(), CompressionError> {
    encode_celestial_body(bytes, &body.body)?;
    write_u32(bytes, body.segments.len() as u32);
    for segment in &body.segments {
        encode_segment(bytes, segment)?;
    }
    Ok(())
}

pub(crate) fn decode_body(
    cursor: &mut Cursor<'_>,
) -> Result<crate::channels::BodyArtifact, CompressionError> {
    let body = decode_celestial_body(cursor)?;
    let segment_count = cursor.read_u32()? as usize;
    let mut segments = Vec::with_capacity(segment_count);
    for _ in 0..segment_count {
        segments.push(decode_segment(cursor)?);
    }
    Ok(crate::channels::BodyArtifact { body, segments })
}

// ── Validation helpers ────────────────────────────────────────────────────────

pub(crate) fn validate_canonical_header_text(
    field: &str,
    value: &str,
) -> Result<(), CompressionError> {
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

pub(crate) fn validate_artifact_profile(profile: &ArtifactProfile) -> Result<(), CompressionError> {
    validate_unique_values("artifact profile stored channels", &profile.stored_channels)?;
    validate_unique_values("artifact profile derived outputs", &profile.derived_outputs)?;
    validate_unique_values(
        "artifact profile unsupported outputs",
        &profile.unsupported_outputs,
    )?;
    validate_channel_kind_order("artifact profile stored channels", &profile.stored_channels)?;
    validate_artifact_output_order("artifact profile derived outputs", &profile.derived_outputs)?;
    validate_artifact_output_order(
        "artifact profile unsupported outputs",
        &profile.unsupported_outputs,
    )?;
    validate_disjoint_values(
        "artifact profile derived outputs",
        &profile.derived_outputs,
        "artifact profile unsupported outputs",
        &profile.unsupported_outputs,
    )?;
    validate_explicit_output_classification(profile)?;
    validate_coordinate_output_policy(profile)?;
    validate_motion_policy(profile)?;
    Ok(())
}

fn validate_artifact_output_order(
    field: &str,
    outputs: &[ArtifactOutput],
) -> Result<(), CompressionError> {
    for pair in outputs.windows(2) {
        if pair[0].ordinal() > pair[1].ordinal() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!(
                    "{field} must be ordered by artifact output kind; found {:?} before {:?}",
                    pair[0], pair[1]
                ),
            ));
        }
    }

    Ok(())
}

fn validate_explicit_output_classification(
    profile: &ArtifactProfile,
) -> Result<(), CompressionError> {
    for output in ArtifactOutput::all() {
        if profile.output_support(output) == ArtifactOutputSupport::Unlisted {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!(
                    "artifact profile output {output} must be explicitly listed as stored, derived, approximated, or unsupported"
                ),
            ));
        }
    }

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
    match profile.motion_output_support() {
        ArtifactOutputSupport::Stored => {
            if profile.derived_outputs.contains(&ArtifactOutput::Motion) {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    "artifact profile speed policy Stored must not list Motion in derived outputs",
                ));
            }
            if profile
                .unsupported_outputs
                .contains(&ArtifactOutput::Motion)
            {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    "artifact profile speed policy Stored must not list Motion in unsupported outputs",
                ));
            }
        }
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
            if profile
                .unsupported_outputs
                .contains(&ArtifactOutput::Motion)
            {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    "artifact profile speed policy FittedDerivative must not list Motion in unsupported outputs",
                ));
            }
        }
        ArtifactOutputSupport::Approximated => {
            if profile.derived_outputs.contains(&ArtifactOutput::Motion) {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    "artifact profile speed policy NumericalDifference must not list Motion in derived outputs",
                ));
            }
            if profile
                .unsupported_outputs
                .contains(&ArtifactOutput::Motion)
            {
                return Err(CompressionError::new(
                    CompressionErrorKind::InvalidFormat,
                    "artifact profile speed policy NumericalDifference must not list Motion in unsupported outputs",
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
            unreachable!("motion support is always stored, derived, approximated, or unsupported")
        }
    }

    Ok(())
}

pub(crate) fn validate_channel_kind_order(
    field: &str,
    kinds: &[ChannelKind],
) -> Result<(), CompressionError> {
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

pub(crate) fn validate_segment(segment: &Segment) -> Result<(), CompressionError> {
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

pub(crate) fn validate_unique_values<T: fmt::Display + Eq>(
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

pub(crate) fn validate_disjoint_values<T: fmt::Display + Eq>(
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

pub(crate) fn validate_body_artifacts(
    bodies: &[crate::channels::BodyArtifact],
) -> Result<(), CompressionError> {
    for (index, body) in bodies.iter().enumerate() {
        if bodies[..index].iter().any(|other| other.body == body.body) {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("compressed artifact contains duplicate body entry {body:?}"),
            ));
        }
    }

    let body_order = bodies
        .iter()
        .map(|body| body.body.clone())
        .collect::<Vec<_>>();
    validate_canonical_body_order("compressed artifact body entries", &body_order)?;

    Ok(())
}

pub(crate) fn validate_canonical_body_order(
    field: &str,
    bodies: &[CelestialBody],
) -> Result<(), CompressionError> {
    for pair in bodies.windows(2) {
        if canonical_body_order_key(&pair[0]) > canonical_body_order_key(&pair[1]) {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!(
                    "{field} must be ordered canonically; found {:?} before {:?}",
                    pair[0], pair[1]
                ),
            ));
        }
    }

    Ok(())
}

fn canonical_body_order_key(body: &CelestialBody) -> (u8, &str, &str) {
    match body {
        CelestialBody::Sun => (0, "", ""),
        CelestialBody::Moon => (1, "", ""),
        CelestialBody::Mercury => (2, "", ""),
        CelestialBody::Venus => (3, "", ""),
        CelestialBody::Mars => (4, "", ""),
        CelestialBody::Jupiter => (5, "", ""),
        CelestialBody::Saturn => (6, "", ""),
        CelestialBody::Uranus => (7, "", ""),
        CelestialBody::Neptune => (8, "", ""),
        CelestialBody::Pluto => (9, "", ""),
        CelestialBody::MeanNode => (10, "", ""),
        CelestialBody::TrueNode => (11, "", ""),
        CelestialBody::MeanApogee => (12, "", ""),
        CelestialBody::TrueApogee => (13, "", ""),
        CelestialBody::MeanPerigee => (14, "", ""),
        CelestialBody::TruePerigee => (15, "", ""),
        CelestialBody::Ceres => (16, "", ""),
        CelestialBody::Pallas => (17, "", ""),
        CelestialBody::Juno => (18, "", ""),
        CelestialBody::Vesta => (19, "", ""),
        CelestialBody::Custom(custom) => (20, custom.catalog.as_str(), custom.designation.as_str()),
        _ => (u8::MAX, "", ""),
    }
}

pub(crate) fn validate_body_segments(segments: &[Segment]) -> Result<(), CompressionError> {
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
