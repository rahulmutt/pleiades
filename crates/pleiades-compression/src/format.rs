//! Artifact format and capability profile types.

use core::fmt;

use pleiades_types::CelestialBody;

use crate::channels::ChannelKind;
use crate::codec::{
    validate_artifact_profile, validate_canonical_body_order, validate_unique_values,
};
use crate::error::{CompressionError, CompressionErrorKind};

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

    pub(crate) const fn ordinal(self) -> u8 {
        match self {
            Self::EclipticCoordinates => 0,
            Self::EquatorialCoordinates => 1,
            Self::ApparentCorrections => 2,
            Self::TopocentricCoordinates => 3,
            Self::SiderealCoordinates => 4,
            Self::Motion => 5,
        }
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
    /// The output is stored directly in the artifact payload.
    Stored,
    /// The output is reconstructed deterministically from stored data.
    Derived,
    /// The output is approximated numerically from neighboring decoded data.
    Approximated,
    /// The output is explicitly unsupported by the profile.
    Unsupported,
    /// The output is neither stored nor explicitly declared by the profile.
    Unlisted,
}

impl ArtifactOutputSupport {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stored => "stored",
            Self::Derived => "derived",
            Self::Approximated => "approximated",
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
            Self::Stored => ArtifactOutputSupport::Stored,
            Self::FittedDerivative => ArtifactOutputSupport::Derived,
            Self::NumericalDifference => ArtifactOutputSupport::Approximated,
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

    /// Validates that the profile does not contain duplicate, conflicting, or
    /// non-canonical entries.
    ///
    /// The codec performs the same checks when encoding or decoding artifacts,
    /// but exposing the validation step directly lets artifact generators fail
    /// before serialization if they assemble an invalid capability profile.
    pub fn validate(&self) -> Result<(), CompressionError> {
        validate_artifact_profile(self)
    }

    /// Returns a compact one-line summary of the stored, derived, approximated,
    /// unsupported, and speed-policy capabilities encoded by this profile.
    pub fn summary(&self) -> String {
        self.summary_line()
    }

    /// Returns how motion output is represented by this profile.
    pub fn motion_output_support(&self) -> ArtifactOutputSupport {
        self.speed_policy.motion_output_support()
    }

    /// Returns a compact one-line summary of the stored, derived, approximated,
    /// unsupported, and speed-policy capabilities encoded by this profile.
    pub fn summary_line(&self) -> String {
        format!(
            "stored channels: {}; derived outputs: {}; unsupported outputs: {}; speed policy: {}",
            format_bracketed_labels(&self.stored_channels),
            format_bracketed_labels(&self.derived_outputs),
            format_bracketed_labels(&self.unsupported_outputs),
            self.speed_policy,
        )
    }

    /// Returns the validated capability summary line.
    pub fn validated_summary_line(&self) -> Result<String, CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the validated compact support entries used by the output-support summary.
    pub fn validated_output_support_entries_summary_line(
        &self,
    ) -> Result<String, CompressionError> {
        self.validate()?;
        Ok(self.output_support_entries_summary_line())
    }

    /// Returns the validated output-support summary line.
    pub fn validated_output_support_summary_line(&self) -> Result<String, CompressionError> {
        self.validate()?;
        Ok(self.output_support_summary_line())
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
    ///
    /// The rendered line also makes the explicit `unlisted` bucket visible so
    /// release-facing summaries can fail closed if a built-in output stops being
    /// classified. It also reports how many outputs are stored, derived,
    /// approximated, unsupported, or unlisted so the profile makes the support
    /// buckets explicit without requiring the reader to count them manually.
    pub fn output_support_summary_line(&self) -> String {
        let mut stored_count = 0usize;
        let mut derived_count = 0usize;
        let mut approximated_count = 0usize;
        let mut unsupported_count = 0usize;
        let mut unlisted_count = 0usize;
        let mut unlisted_outputs = Vec::new();

        for output in ArtifactOutput::all() {
            match self.output_support(output) {
                ArtifactOutputSupport::Stored => stored_count += 1,
                ArtifactOutputSupport::Derived => derived_count += 1,
                ArtifactOutputSupport::Approximated => approximated_count += 1,
                ArtifactOutputSupport::Unsupported => unsupported_count += 1,
                ArtifactOutputSupport::Unlisted => {
                    unlisted_count += 1;
                    unlisted_outputs.push(output);
                }
            }
        }

        format!(
            "{}; unlisted outputs: {}; support counts: stored={}, derived={}, approximated={}, unsupported={}, unlisted={}",
            self.output_support_entries_summary_line(),
            format_bracketed_labels(&unlisted_outputs),
            stored_count,
            derived_count,
            approximated_count,
            unsupported_count,
            unlisted_count,
        )
    }

    /// Returns how a high-level output is represented by this profile.
    pub fn output_support(&self, output: ArtifactOutput) -> ArtifactOutputSupport {
        if output == ArtifactOutput::Motion {
            self.motion_output_support()
        } else if self.derived_outputs.contains(&output) {
            ArtifactOutputSupport::Derived
        } else if self.unsupported_outputs.contains(&output) {
            ArtifactOutputSupport::Unsupported
        } else {
            ArtifactOutputSupport::Unlisted
        }
    }

    /// Returns whether the profile can provide the requested output.
    pub fn supports_output(&self, output: ArtifactOutput) -> bool {
        matches!(
            self.output_support(output),
            ArtifactOutputSupport::Stored
                | ArtifactOutputSupport::Derived
                | ArtifactOutputSupport::Approximated
        )
    }

    /// Returns whether the profile explicitly marks the output unsupported.
    pub fn is_unsupported_output(&self, output: ArtifactOutput) -> bool {
        matches!(
            self.output_support(output),
            ArtifactOutputSupport::Unsupported
        )
    }

    /// Returns the current packaged-artifact profile shorthand: ecliptic longitude,
    /// latitude, and distance are stored directly; ecliptic coordinates are
    /// reconstructed from those channels; equatorial coordinates are derived from
    /// the stored ecliptic coordinates and mean-obliquity policy; and motion/speed
    /// is `Motion = Derived` (`SpeedPolicy::FittedDerivative`), not unsupported.
    pub fn ecliptic_longitude_latitude_distance() -> Self {
        Self::ecliptic_longitude_latitude_distance_with_derived_equatorial()
    }

    /// Returns the current packaged-artifact profile with stored ecliptic
    /// longitude, latitude, and distance channels plus derived equatorial
    /// coordinates.
    pub fn packaged_ecliptic_longitude_latitude_distance_with_derived_equatorial() -> Self {
        Self::new(
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu,
            ],
            vec![
                ArtifactOutput::EclipticCoordinates,
                ArtifactOutput::EquatorialCoordinates,
                ArtifactOutput::Motion,
            ],
            vec![
                ArtifactOutput::ApparentCorrections,
                ArtifactOutput::TopocentricCoordinates,
                ArtifactOutput::SiderealCoordinates,
            ],
            SpeedPolicy::FittedDerivative,
        )
    }

    /// Returns the current packaged-artifact profile with stored ecliptic
    /// longitude, latitude, and distance channels plus derived equatorial
    /// coordinates.
    pub fn ecliptic_longitude_latitude_distance_with_derived_equatorial() -> Self {
        Self::packaged_ecliptic_longitude_latitude_distance_with_derived_equatorial()
    }
}

impl fmt::Display for ArtifactProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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
        validate_canonical_body_order("artifact profile coverage bundled bodies", &self.bodies)?;
        if self.body_count != self.bodies.len() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "artifact profile coverage body count does not match bundled body list",
            ));
        }

        Ok(())
    }

    /// Returns the capability summary annotated with how many bundled bodies
    /// currently appear in the summary.
    pub fn summary_line(&self) -> String {
        self.profile.summary_for_body_count(self.bodies.len())
    }

    /// Returns the validated capability summary annotated with how many bundled
    /// bodies currently appear in the summary.
    pub fn validated_summary_line(&self) -> Result<String, CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the capability summary annotated with how many bodies share it
    /// and lists the bundled bodies explicitly.
    pub fn summary_line_with_bodies(&self) -> String {
        format!(
            "{}; bundled bodies: {}",
            self.summary_line(),
            crate::join_display(&self.bodies)
        )
    }

    /// Returns the bundled-body summary line after validating the coverage record.
    pub fn validated_summary_line_with_bodies(&self) -> Result<String, CompressionError> {
        self.validate()?;
        Ok(self.summary_line_with_bodies())
    }
}

impl fmt::Display for ArtifactProfileCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured body coverage for residual-correction-bearing segments in a compressed artifact.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactResidualBodyCoverageSummary {
    /// Number of bundled bodies that include at least one residual-correction segment.
    pub body_count: usize,
    /// Bodies that include at least one residual-correction segment.
    pub bodies: Vec<CelestialBody>,
}

impl ArtifactResidualBodyCoverageSummary {
    /// Creates a residual-body coverage summary from an explicit body list.
    pub fn new(bodies: Vec<CelestialBody>) -> Self {
        let body_count = bodies.len();
        Self { body_count, bodies }
    }

    /// Validates that the summary still matches the current artifact residual-body set.
    pub fn validate(
        &self,
        artifact: &crate::artifact::CompressedArtifact,
    ) -> Result<(), CompressionError> {
        let expected_bodies = artifact.residual_bodies();

        if self.body_count != self.bodies.len() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "artifact residual-body coverage body count does not match the body list",
            ));
        }
        validate_unique_values("artifact residual-body coverage bodies", &self.bodies)?;

        if self.body_count != expected_bodies.len() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "artifact residual-body coverage body count does not match residual body list",
            ));
        }

        if self.bodies != expected_bodies {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "artifact residual-body coverage body list does not match the current artifact",
            ));
        }

        Ok(())
    }

    /// Returns the residual-body coverage as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        match self.bodies.as_slice() {
            [] => "residual bodies: none".to_string(),
            bodies => format!("residual bodies: {}", crate::join_display(bodies)),
        }
    }

    /// Returns the residual-body coverage after validating the artifact.
    pub fn validated_summary_line(
        &self,
        artifact: &crate::artifact::CompressedArtifact,
    ) -> Result<String, CompressionError> {
        self.validate(artifact)?;
        Ok(self.summary_line())
    }

    /// Returns the residual-body coverage annotated with how many bodies share it.
    pub fn summary_line_with_body_count(&self) -> String {
        format!(
            "{}; applies to {}",
            self.summary_line(),
            self.body_count_suffix()
        )
    }

    /// Returns the residual-body coverage line after validating the artifact.
    pub fn validated_summary_line_with_body_count(
        &self,
        artifact: &crate::artifact::CompressedArtifact,
    ) -> Result<String, CompressionError> {
        let summary = self.validated_summary_line(artifact)?;
        Ok(format!(
            "{}; applies to {}",
            summary,
            self.body_count_suffix()
        ))
    }

    fn body_count_suffix(&self) -> String {
        match self.body_count {
            1 => "1 bundled body".to_string(),
            count => format!("{count} bundled bodies"),
        }
    }
}

impl fmt::Display for ArtifactResidualBodyCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

// ── ArtifactHeader ────────────────────────────────────────────────────────────

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
    /// little-endian byte-order policy, and a conservative profile that stores
    /// only ecliptic longitude/latitude/distance channels (with equatorial
    /// coordinates and motion derived, and apparent/topocentric/sidereal outputs
    /// unsupported).
    pub fn new(generation_label: impl Into<String>, source: impl Into<String>) -> Self {
        Self::with_profile(
            generation_label,
            source,
            ArtifactProfile::packaged_ecliptic_longitude_latitude_distance_with_derived_equatorial(
            ),
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
            version: crate::ARTIFACT_VERSION,
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
        if self.version != crate::ARTIFACT_VERSION {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!(
                    "artifact header version {} does not match the current format version {}",
                    self.version,
                    crate::ARTIFACT_VERSION
                ),
            ));
        }

        crate::codec::validate_canonical_header_text(
            "artifact header generation label",
            &self.generation_label,
        )?;
        crate::codec::validate_canonical_header_text("artifact header source", &self.source)?;

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

impl fmt::Display for ArtifactHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

// ── Formatting helpers ────────────────────────────────────────────────────────

pub(crate) fn format_bracketed_labels<T: fmt::Display>(values: &[T]) -> String {
    format!("[{}]", crate::join_display(values))
}
