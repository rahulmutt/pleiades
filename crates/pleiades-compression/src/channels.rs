//! Channel, segment, and body artifact types.

use core::fmt;

use pleiades_types::{CelestialBody, Instant};

use crate::codec::{validate_body_segments, validate_segment};
use crate::error::{CompressionError, CompressionErrorKind};

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

    /// Creates a quadratic channel that interpolates start, midpoint, and end values
    /// over the normalized interval `[0, 1]`.
    pub fn quadratic(
        kind: ChannelKind,
        scale_exponent: u8,
        start: f64,
        midpoint: f64,
        end: f64,
        midpoint_x: f64,
    ) -> Self {
        let linear_delta = end - start;
        let midpoint_residual = midpoint - (start + linear_delta * midpoint_x);
        let curvature_scale = midpoint_x * (1.0 - midpoint_x);

        if curvature_scale == 0.0 {
            return Self::linear(kind, scale_exponent, start, end);
        }

        let curvature = midpoint_residual / curvature_scale;
        Self::new(
            kind,
            scale_exponent,
            vec![start, linear_delta + curvature, -curvature],
        )
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

    pub(crate) fn evaluate(&self, x: f64) -> f64 {
        let mut result = 0.0;
        let mut power = 1.0;
        for coefficient in &self.coefficients {
            result += coefficient * power;
            power *= x;
        }
        result
    }

    /// Derivative dP/dx of the monomial polynomial at normalized time `x`.
    /// Coefficients are ascending power order, so d/dx(Σ c_i x^i) = Σ i·c_i·x^(i-1).
    pub(crate) fn evaluate_derivative(&self, x: f64) -> f64 {
        let mut result = 0.0;
        let mut power = 1.0; // x^(i-1), starting at i=1
        for (i, coefficient) in self.coefficients.iter().enumerate().skip(1) {
            result += (i as f64) * coefficient * power;
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
            crate::format::format_bracketed_labels(&stored_channels),
            crate::format::format_bracketed_labels(&residual_channels),
        )
    }

    pub(crate) fn contains(&self, instant: Instant) -> bool {
        self.start.scale == instant.scale
            && self.end.scale == instant.scale
            && self.start.julian_day.days() <= instant.julian_day.days()
            && instant.julian_day.days() <= self.end.julian_day.days()
    }

    pub(crate) fn span_days(&self) -> f64 {
        self.end.julian_day.days() - self.start.julian_day.days()
    }

    pub(crate) fn channel(&self, kind: ChannelKind) -> Option<&PolynomialChannel> {
        self.channels.iter().find(|channel| channel.kind == kind)
    }

    fn residual_channel(&self, kind: ChannelKind) -> Option<&PolynomialChannel> {
        self.residual_channels
            .iter()
            .find(|channel| channel.kind == kind)
    }

    pub(crate) fn evaluate_channel(
        &self,
        kind: ChannelKind,
        x: f64,
    ) -> Result<f64, CompressionError> {
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

    /// (base + residual) derivative dP/dx at normalized time `x` for `kind`.
    pub(crate) fn evaluate_channel_derivative(
        &self,
        kind: ChannelKind,
        x: f64,
    ) -> Result<f64, CompressionError> {
        let base = self
            .channel(kind)
            .map(|channel| channel.evaluate_derivative(x))
            .ok_or_else(|| {
                CompressionError::new(
                    CompressionErrorKind::MissingChannel,
                    format!("missing {kind:?} channel"),
                )
            })?;
        let residual = self
            .residual_channel(kind)
            .map(|channel| channel.evaluate_derivative(x))
            .unwrap_or(0.0);
        Ok(base + residual)
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// The coordinate frame a body's stored channels are expressed in.
///
/// `Geocentric` channels are returned directly at lookup. `Heliocentric` channels
/// are recombined with the geocentric Sun (`P_geo = P_helio + S_geo`) before being
/// returned, so the public lookup output is always geocentric ecliptic.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum StoredFrame {
    /// Stored channels are geocentric ecliptic; returned as-is.
    Geocentric,
    /// Stored channels are heliocentric ecliptic; recombined with the Sun at lookup.
    Heliocentric,
}

/// All segments for a single body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BodyArtifact {
    /// Body identifier.
    pub body: CelestialBody,
    /// Time segments for the body.
    pub segments: Vec<Segment>,
    /// Frame the stored channels are expressed in.
    pub frame: StoredFrame,
}

impl BodyArtifact {
    /// Creates a new geocentric body artifact (the default frame).
    pub fn new(body: CelestialBody, segments: Vec<Segment>) -> Self {
        Self { body, segments, frame: StoredFrame::Geocentric }
    }

    /// Creates a body artifact with an explicit stored frame.
    pub fn with_frame(body: CelestialBody, segments: Vec<Segment>, frame: StoredFrame) -> Self {
        Self { body, segments, frame }
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

#[cfg(test)]
mod frame_field_tests {
    use super::*;
    use pleiades_types::CelestialBody;

    #[test]
    fn new_defaults_to_geocentric() {
        let b = BodyArtifact::new(CelestialBody::Sun, vec![]);
        assert_eq!(b.frame, StoredFrame::Geocentric);
    }

    #[test]
    fn with_frame_sets_heliocentric() {
        let b = BodyArtifact::with_frame(CelestialBody::Jupiter, vec![], StoredFrame::Heliocentric);
        assert_eq!(b.frame, StoredFrame::Heliocentric);
    }
}

#[cfg(test)]
mod derivative_tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    #[test]
    fn polynomial_derivative_matches_power_rule() {
        // P(x) = 2 + 3x + 4x^2  ->  P'(x) = 3 + 8x
        let ch = PolynomialChannel::new(ChannelKind::Longitude, 9, vec![2.0, 3.0, 4.0]);
        assert!((ch.evaluate_derivative(0.0) - 3.0).abs() < 1e-12);
        assert!((ch.evaluate_derivative(1.0) - 11.0).abs() < 1e-12);
        assert!((ch.evaluate_derivative(0.5) - 7.0).abs() < 1e-12);
    }

    #[test]
    fn segment_channel_derivative_includes_residual() {
        let start = Instant::new(JulianDay::from_days(0.0), TimeScale::Tt);
        let end = Instant::new(JulianDay::from_days(1.0), TimeScale::Tt);
        let seg = Segment::with_residual_channels(
            start,
            end,
            vec![PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.0, 2.0])], // base' = 2
            vec![PolynomialChannel::new(
                ChannelKind::Longitude,
                9,
                vec![0.0, 0.0, 5.0], // residual' = 10x
            )],
        );
        // total derivative at x=1: 2 + 10 = 12
        let d = seg.evaluate_channel_derivative(ChannelKind::Longitude, 1.0).unwrap();
        assert!((d - 12.0).abs() < 1e-12);
    }
}
