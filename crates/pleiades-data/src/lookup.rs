use std::fmt;
use std::sync::OnceLock;

#[cfg(feature = "packaged-artifact-path")]
use std::path::Path;

use pleiades_backend::{
    Apparentness, CelestialBody, CoordinateFrame, EclipticCoordinates, EphemerisBackend,
    EphemerisRequest, Instant, QualityAnnotation, TimeScale, ZodiacMode,
};
use pleiades_compression::{
    join_display, ArtifactOutput, ArtifactProfile, ChannelKind, CompressedArtifact,
};
use pleiades_jpl::{reference_snapshot, SnapshotEntry};

use crate::backend::PackagedDataBackend;
use crate::coverage::packaged_artifact_profile_summary_details;
use crate::data::{packaged_artifact, PackagedArtifactLoadError};
use crate::regenerate::normalize_lookup_instant;
use crate::{packaged_bodies, packaged_reference_entry_for_body};

/// Structured policy for how packaged-data lookup epochs are handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedLookupEpochPolicy {
    /// TDB-tagged lookups are re-tagged onto the TT grid without relativistic correction.
    RetagToTtGridWithoutRelativisticCorrection,
}

impl PackagedLookupEpochPolicy {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::RetagToTtGridWithoutRelativisticCorrection => {
                "TT-grid retag without relativistic correction"
            }
        }
    }

    /// Returns the explanatory note used in release-facing summaries.
    pub const fn note(self) -> &'static str {
        match self {
            Self::RetagToTtGridWithoutRelativisticCorrection => {
                "TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
            }
        }
    }

    /// Returns `Ok(())` when the policy still matches the current packaged-data posture.
    pub fn validate(self) -> Result<(), PackagedLookupEpochPolicyValidationError> {
        if self != Self::RetagToTtGridWithoutRelativisticCorrection {
            return Err(PackagedLookupEpochPolicyValidationError::FieldOutOfSync {
                field: "policy",
            });
        }

        Ok(())
    }

    /// Returns the compact release-facing summary for the lookup-epoch policy.
    pub fn summary_line(self) -> String {
        format!("{}; {}", self.label(), self.note())
    }
}

impl fmt::Display for PackagedLookupEpochPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Validation error for the packaged-data lookup-epoch policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedLookupEpochPolicyValidationError {
    /// A policy field is out of sync with the current packaged-data posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedLookupEpochPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged lookup-epoch policy field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedLookupEpochPolicyValidationError {}

/// Structured summary for the packaged-data lookup-epoch policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedLookupEpochPolicySummary {
    /// Policy describing how TDB-tagged lookups are handled.
    pub policy: PackagedLookupEpochPolicy,
}

/// Validation error for the packaged-data lookup-epoch policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedLookupEpochPolicySummaryValidationError {
    /// A summary field is out of sync with the current packaged-data posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedLookupEpochPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged lookup-epoch policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedLookupEpochPolicySummaryValidationError {}

impl PackagedLookupEpochPolicySummary {
    /// Returns the packaged lookup-epoch policy as a compact human-readable line.
    pub fn summary_line(self) -> String {
        self.policy.summary_line()
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-data posture.
    pub fn validate(&self) -> Result<(), PackagedLookupEpochPolicySummaryValidationError> {
        self.policy.validate().map_err(|error| match error {
            PackagedLookupEpochPolicyValidationError::FieldOutOfSync { field } => {
                PackagedLookupEpochPolicySummaryValidationError::FieldOutOfSync { field }
            }
        })
    }
}

impl fmt::Display for PackagedLookupEpochPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const PACKAGED_LOOKUP_EPOCH_POLICY_SUMMARY: PackagedLookupEpochPolicySummary =
    PackagedLookupEpochPolicySummary {
        policy: PackagedLookupEpochPolicy::RetagToTtGridWithoutRelativisticCorrection,
    };

/// Returns the current packaged-data lookup-epoch policy summary record.
///
/// # Examples
///
/// ```
/// use pleiades_data::packaged_lookup_epoch_policy_summary_details;
///
/// let summary = packaged_lookup_epoch_policy_summary_details();
/// assert_eq!(
///     summary.summary_line(),
///     "TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
/// );
/// assert!(summary.validate().is_ok());
/// ```
pub fn packaged_lookup_epoch_policy_summary_details() -> PackagedLookupEpochPolicySummary {
    let summary = PACKAGED_LOOKUP_EPOCH_POLICY_SUMMARY;
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-data lookup-epoch policy summary after validating the structured posture.
pub fn packaged_lookup_epoch_policy_summary_for_report() -> String {
    let summary = packaged_lookup_epoch_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged lookup epoch policy: unavailable ({error})"),
    }
}

/// Returns the current packaged-data lookup-epoch policy summary.
pub fn packaged_lookup_epoch_policy_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_lookup_epoch_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.to_string(),
                Err(error) => format!("Packaged lookup epoch policy: unavailable ({error})"),
            }
        })
        .as_str()
}

/// Structured request-policy summary for the packaged-data backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackagedRequestPolicySummary {
    /// Whether the backend is geocentric only.
    pub geocentric_only: bool,
    /// Coordinate frames supported by the packaged backend.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales supported by the packaged backend.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes supported by the packaged backend.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes supported by the packaged backend.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the packaged backend accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
    /// Policy describing how TDB-tagged lookups are handled.
    pub lookup_epoch_policy: PackagedLookupEpochPolicy,
}

/// Validation error for the packaged-data request-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedRequestPolicySummaryValidationError {
    /// A summary field is out of sync with the current packaged-data posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedRequestPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged request-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedRequestPolicySummaryValidationError {}

impl PackagedRequestPolicySummary {
    /// Renders the packaged request policy into a release-facing summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged request policy: {}frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}; lookup epoch policy={}",
            if self.geocentric_only {
                "geocentric-only; "
            } else {
                ""
            },
            join_display(self.supported_frames),
            join_display(self.supported_time_scales),
            join_display(self.supported_zodiac_modes),
            join_display(self.supported_apparentness),
            self.supports_topocentric_observer,
            self.lookup_epoch_policy.summary_line(),
        )
    }

    /// Returns `Ok(())` when the packaged request-policy posture still matches the current backend metadata.
    pub fn validate(&self) -> Result<(), PackagedRequestPolicySummaryValidationError> {
        if !self.geocentric_only {
            return Err(
                PackagedRequestPolicySummaryValidationError::FieldOutOfSync {
                    field: "geocentric_only",
                },
            );
        }
        if self.supported_frames != [CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial] {
            return Err(
                PackagedRequestPolicySummaryValidationError::FieldOutOfSync {
                    field: "supported_frames",
                },
            );
        }
        if self.supported_time_scales != [TimeScale::Tt, TimeScale::Tdb] {
            return Err(
                PackagedRequestPolicySummaryValidationError::FieldOutOfSync {
                    field: "supported_time_scales",
                },
            );
        }
        if self.supported_zodiac_modes != [ZodiacMode::Tropical] {
            return Err(
                PackagedRequestPolicySummaryValidationError::FieldOutOfSync {
                    field: "supported_zodiac_modes",
                },
            );
        }
        if self.supported_apparentness != [Apparentness::Mean] {
            return Err(
                PackagedRequestPolicySummaryValidationError::FieldOutOfSync {
                    field: "supported_apparentness",
                },
            );
        }
        if self.supports_topocentric_observer {
            return Err(
                PackagedRequestPolicySummaryValidationError::FieldOutOfSync {
                    field: "supports_topocentric_observer",
                },
            );
        }
        if self.lookup_epoch_policy
            != PackagedLookupEpochPolicy::RetagToTtGridWithoutRelativisticCorrection
        {
            return Err(
                PackagedRequestPolicySummaryValidationError::FieldOutOfSync {
                    field: "lookup_epoch_policy",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for PackagedRequestPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const PACKAGED_REQUEST_POLICY_SUMMARY: PackagedRequestPolicySummary =
    PackagedRequestPolicySummary {
        geocentric_only: true,
        supported_frames: &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
        supported_time_scales: &[TimeScale::Tt, TimeScale::Tdb],
        supported_zodiac_modes: &[ZodiacMode::Tropical],
        supported_apparentness: &[Apparentness::Mean],
        supports_topocentric_observer: false,
        lookup_epoch_policy: PackagedLookupEpochPolicy::RetagToTtGridWithoutRelativisticCorrection,
    };

/// Returns the current packaged-data request-policy summary record.
///
/// # Examples
///
/// ```
/// use pleiades_data::packaged_request_policy_summary_details;
///
/// let summary = packaged_request_policy_summary_details();
/// assert_eq!(
///     summary.summary_line(),
///     "Packaged request policy: geocentric-only; frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false; lookup epoch policy=TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
/// );
/// assert!(summary.validate().is_ok());
/// ```
pub fn packaged_request_policy_summary_details() -> PackagedRequestPolicySummary {
    let summary = PACKAGED_REQUEST_POLICY_SUMMARY;
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-data request policy summary after validating the structured posture.
pub fn packaged_request_policy_summary_for_report() -> String {
    let summary = packaged_request_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged request policy: unavailable ({error})"),
    }
}

/// Returns the current packaged-data request policy summary.
pub fn packaged_request_policy_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_request_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.to_string(),
                Err(error) => format!("Packaged request policy: unavailable ({error})"),
            }
        })
        .as_str()
}

/// Structured frame-treatment summary for the packaged artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedFrameTreatmentSummary;

/// Validation error for a packaged frame-treatment summary that drifted away from the compact posture line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedFrameTreatmentSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
}

impl fmt::Display for PackagedFrameTreatmentSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("packaged frame-treatment summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("packaged frame-treatment summary has surrounding whitespace")
            }
        }
    }
}

impl std::error::Error for PackagedFrameTreatmentSummaryValidationError {}

pub(crate) fn validate_packaged_frame_treatment_summary_line(
    summary: &str,
) -> Result<(), PackagedFrameTreatmentSummaryValidationError> {
    if summary.trim().is_empty() {
        Err(PackagedFrameTreatmentSummaryValidationError::BlankSummary)
    } else if summary.trim() != summary {
        Err(PackagedFrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
    } else {
        Ok(())
    }
}

impl PackagedFrameTreatmentSummary {
    /// Returns the frame-treatment posture as a compact human-readable line.
    pub const fn summary_line(self) -> &'static str {
        "checked-in compressed artifact stores ecliptic coordinates directly; equatorial coordinates are reconstructed from the stored channels and mean-obliquity transform"
    }

    /// Returns `Ok(())` when the summary still contains a compact canonical line.
    pub fn validate(&self) -> Result<(), PackagedFrameTreatmentSummaryValidationError> {
        validate_packaged_frame_treatment_summary_line(self.summary_line())
    }

    /// Returns the compact one-line rendering after validation.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, PackagedFrameTreatmentSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedFrameTreatmentSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Returns the structured packaged-artifact frame-treatment summary.
pub const fn packaged_frame_treatment_summary_details() -> PackagedFrameTreatmentSummary {
    PackagedFrameTreatmentSummary
}

/// Returns the packaged-artifact frame-treatment summary for report rendering.
pub fn packaged_frame_treatment_summary_for_report() -> String {
    let summary = packaged_frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("Packaged frame treatment unavailable ({error})"),
    }
}

/// Returns the packaged-artifact frame-treatment summary.
pub fn packaged_frame_treatment_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(packaged_frame_treatment_summary_for_report)
        .as_str()
}

/// Structured storage/reconstruction summary for the packaged artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedArtifactStorageSummary;

/// Validation error for a packaged storage/reconstruction summary that drifted away from the compact posture line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedArtifactStorageSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text no longer matches the current packaged-artifact profile posture.
    ProfileOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactStorageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("packaged artifact storage summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("packaged artifact storage summary has surrounding whitespace")
            }
            Self::ProfileOutOfSync { field } => write!(
                f,
                "packaged artifact storage summary is out of sync with the bundled artifact profile field `{field}`"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactStorageSummaryValidationError {}

pub(crate) fn validate_packaged_artifact_storage_summary_line(
    summary: &str,
) -> Result<(), PackagedArtifactStorageSummaryValidationError> {
    if summary.trim().is_empty() {
        Err(PackagedArtifactStorageSummaryValidationError::BlankSummary)
    } else if summary.trim() != summary {
        Err(PackagedArtifactStorageSummaryValidationError::WhitespacePaddedSummary)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_packaged_artifact_storage_profile(
    profile: &ArtifactProfile,
) -> Result<(), PackagedArtifactStorageSummaryValidationError> {
    const EXPECTED_STORED_CHANNELS: [ChannelKind; 3] = [
        ChannelKind::Longitude,
        ChannelKind::Latitude,
        ChannelKind::DistanceAu,
    ];
    const EXPECTED_DERIVED_OUTPUTS: [ArtifactOutput; 3] = [
        ArtifactOutput::EclipticCoordinates,
        ArtifactOutput::EquatorialCoordinates,
        ArtifactOutput::Motion,
    ];
    const EXPECTED_UNSUPPORTED_OUTPUTS: [ArtifactOutput; 3] = [
        ArtifactOutput::ApparentCorrections,
        ArtifactOutput::TopocentricCoordinates,
        ArtifactOutput::SiderealCoordinates,
    ];

    if profile.stored_channels != EXPECTED_STORED_CHANNELS {
        return Err(
            PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
                field: "stored_channels",
            },
        );
    }

    if profile.derived_outputs.as_slice() != EXPECTED_DERIVED_OUTPUTS {
        return Err(
            PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
                field: "derived_outputs",
            },
        );
    }

    if profile.unsupported_outputs.as_slice() != EXPECTED_UNSUPPORTED_OUTPUTS {
        return Err(
            PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
                field: "unsupported_outputs",
            },
        );
    }

    Ok(())
}

impl PackagedArtifactStorageSummary {
    /// Returns the storage and reconstruction posture as a compact human-readable line.
    pub const fn summary_line(self) -> &'static str {
        "Quantized linear segments stored in pleiades-compression artifact format; body-indexed segment tables support random access by body and lookup time across the advertised range; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, and sidereal outputs remain unsupported; motion/speed is derived from fitted segment derivatives"
    }

    /// Returns `Ok(())` when the summary still contains a compact canonical line.
    pub fn validate(&self) -> Result<(), PackagedArtifactStorageSummaryValidationError> {
        validate_packaged_artifact_storage_summary_line(self.summary_line())?;
        validate_packaged_artifact_storage_profile(
            &packaged_artifact_profile_summary_details().profile,
        )
    }
}

impl fmt::Display for PackagedArtifactStorageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Returns the structured packaged-artifact storage/reconstruction summary.
pub const fn packaged_artifact_storage_summary_details() -> PackagedArtifactStorageSummary {
    PackagedArtifactStorageSummary
}

/// Returns the packaged-artifact storage/reconstruction summary.
pub fn packaged_artifact_storage_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_storage_summary_details();
            match summary.validate() {
                Ok(()) => summary.to_string(),
                Err(error) => format!("Packaged artifact storage unavailable ({error})"),
            }
        })
        .as_str()
}

/// Returns the packaged-artifact storage/reconstruction summary for reporting.
pub fn packaged_artifact_storage_summary_for_report() -> String {
    let summary = packaged_artifact_storage_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged artifact storage/reconstruction: unavailable ({error})"),
    }
}

/// Structured packaged-artifact access summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedArtifactAccessSummary {
    /// Whether explicit artifact-file loading is enabled.
    pub explicit_path_loading: bool,
}

/// Validation error for a packaged artifact access summary that drifted away from the current build posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedArtifactAccessSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The feature-flag state no longer matches the current build posture.
    FeatureStateOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactAccessSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("packaged artifact access summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("packaged artifact access summary has surrounding whitespace")
            }
            Self::FeatureStateOutOfSync { field } => write!(
                f,
                "packaged artifact access summary is out of sync with the build feature field `{field}`"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactAccessSummaryValidationError {}

pub(crate) fn validate_packaged_artifact_access_summary_line(
    summary: &str,
) -> Result<(), PackagedArtifactAccessSummaryValidationError> {
    if summary.trim().is_empty() {
        Err(PackagedArtifactAccessSummaryValidationError::BlankSummary)
    } else if summary.trim() != summary {
        Err(PackagedArtifactAccessSummaryValidationError::WhitespacePaddedSummary)
    } else {
        Ok(())
    }
}

impl PackagedArtifactAccessSummary {
    /// Returns the packaged artifact access posture as a compact human-readable line.
    pub const fn summary_line(self) -> &'static str {
        if self.explicit_path_loading {
            "packaged artifact access: checked-in fixture plus explicit artifact-path loading via `packaged-artifact-path` feature"
        } else {
            "packaged artifact access: checked-in fixture only; explicit artifact-path loading disabled"
        }
    }

    /// Returns the validated packaged artifact access posture as a compact line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, PackagedArtifactAccessSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current build posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactAccessSummaryValidationError> {
        validate_packaged_artifact_access_summary_line(self.summary_line())?;
        if self.explicit_path_loading != packaged_artifact_path_loading_enabled() {
            return Err(
                PackagedArtifactAccessSummaryValidationError::FeatureStateOutOfSync {
                    field: "explicit_path_loading",
                },
            );
        }
        Ok(())
    }
}

impl fmt::Display for PackagedArtifactAccessSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Returns whether this build can load packaged artifacts from an explicit path.
pub const fn packaged_artifact_path_loading_enabled() -> bool {
    cfg!(feature = "packaged-artifact-path")
}

/// Returns the structured packaged-artifact access summary.
///
/// # Examples
///
/// ```
/// use pleiades_data::{
///     packaged_artifact_access_summary_details,
///     packaged_artifact_access_summary_for_report,
///     packaged_artifact_path_loading_enabled,
/// };
///
/// let summary = packaged_artifact_access_summary_details();
/// assert_eq!(summary.explicit_path_loading, packaged_artifact_path_loading_enabled());
/// assert_eq!(summary.to_string(), packaged_artifact_access_summary_for_report());
/// assert!(summary.validate().is_ok());
/// ```
pub const fn packaged_artifact_access_summary_details() -> PackagedArtifactAccessSummary {
    PackagedArtifactAccessSummary {
        explicit_path_loading: packaged_artifact_path_loading_enabled(),
    }
}

/// Returns the packaged-artifact access summary.
pub fn packaged_artifact_access_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_access_summary_details();
            match summary.validated_summary_line() {
                Ok(rendered) => rendered.to_string(),
                Err(error) => format!("Packaged artifact access unavailable ({error})"),
            }
        })
        .as_str()
}

/// Returns the packaged-artifact access summary for reporting.
pub fn packaged_artifact_access_summary_for_report() -> String {
    let summary = packaged_artifact_access_summary_details();
    match summary.validated_summary_line() {
        Ok(rendered) => rendered.to_string(),
        Err(error) => format!("Packaged artifact access: unavailable ({error})"),
    }
}

/// Structured mixed batch-parity summary for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedBatchParitySummary {
    /// Number of requests in the batch regression.
    pub request_count: usize,
    /// Number of bodies covered by the batch regression.
    pub body_count: usize,
    /// Number of requests using the ecliptic frame.
    pub ecliptic_request_count: usize,
    /// Number of requests using the equatorial frame.
    pub equatorial_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
    /// Whether the batch regression preserved request order.
    pub order_preserved: bool,
    /// Whether the batch regression preserved batch/single parity.
    pub single_query_parity_preserved: bool,
}

/// Validation error for a packaged mixed-frame batch-parity summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedBatchParitySummaryValidationError {
    /// A summary field is out of sync with the current packaged batch-parity posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged mixed-frame batch-parity summary field `{field}` is out of sync with the current packaged posture"
            ),
        }
    }
}

impl std::error::Error for PackagedBatchParitySummaryValidationError {}

impl PackagedBatchParitySummary {
    /// Returns `Ok(())` when the summary still matches the current packaged batch-parity posture.
    pub fn validate(&self) -> Result<(), PackagedBatchParitySummaryValidationError> {
        if self.request_count != self.body_count {
            return Err(PackagedBatchParitySummaryValidationError::FieldOutOfSync {
                field: "request_count/body_count",
            });
        }

        if self.ecliptic_request_count + self.equatorial_request_count != self.request_count {
            return Err(PackagedBatchParitySummaryValidationError::FieldOutOfSync {
                field: "frame_counts",
            });
        }

        if self.ecliptic_request_count == 0 || self.equatorial_request_count == 0 {
            return Err(PackagedBatchParitySummaryValidationError::FieldOutOfSync {
                field: "frame_mix",
            });
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.request_count
        {
            return Err(PackagedBatchParitySummaryValidationError::FieldOutOfSync {
                field: "quality_counts",
            });
        }

        if !self.order_preserved {
            return Err(PackagedBatchParitySummaryValidationError::FieldOutOfSync {
                field: "order_preserved",
            });
        }

        if !self.single_query_parity_preserved {
            return Err(PackagedBatchParitySummaryValidationError::FieldOutOfSync {
                field: "single_query_parity_preserved",
            });
        }

        Ok(())
    }

    /// Returns the validated batch-parity summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

fn packaged_mixed_frame_batch_parity_request_entries(
) -> Option<(Vec<EphemerisRequest>, Vec<SnapshotEntry>)> {
    let snapshot = reference_snapshot();
    let mut requests = Vec::with_capacity(packaged_bodies().len());
    let mut entries = Vec::with_capacity(packaged_bodies().len());

    for (index, body) in packaged_bodies().iter().cloned().enumerate() {
        let entry = packaged_reference_entry_for_body(snapshot, &body)?;
        entries.push(entry.clone());
        requests.push(EphemerisRequest {
            body,
            instant: Instant::new(entry.epoch.julian_day, TimeScale::Tt),
            observer: None,
            frame: if index % 2 == 0 {
                CoordinateFrame::Ecliptic
            } else {
                CoordinateFrame::Equatorial
            },
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        });
    }

    Some((requests, entries))
}

/// Returns the packaged mixed-frame batch-parity request corpus used by downstream tooling.
pub fn packaged_mixed_frame_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    packaged_mixed_frame_batch_parity_request_entries().map(|(requests, _)| requests)
}

/// This is a compatibility alias for [`packaged_mixed_frame_batch_parity_requests`].
#[doc(alias = "packaged_mixed_frame_batch_parity_requests")]
pub fn packaged_mixed_frame_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    packaged_mixed_frame_batch_parity_requests()
}

/// Returns a compact mixed-frame batch-parity summary for the packaged artifact.
pub fn packaged_mixed_frame_batch_parity_summary() -> Option<PackagedBatchParitySummary> {
    let backend = packaged_backend();
    let (requests, entries) = packaged_mixed_frame_batch_parity_request_entries()?;

    let results = backend.positions(&requests).ok()?;
    if results.len() != requests.len() {
        return None;
    }

    let mut ecliptic_request_count = 0usize;
    let mut equatorial_request_count = 0usize;
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for ((request, result), entry) in requests.iter().zip(results.iter()).zip(entries.iter()) {
        let single = backend.position(request).ok();
        single_query_parity &= single.as_ref().is_some_and(|single| single == result);

        order_preserved &= result.body == entry.body
            && result.instant == request.instant
            && result.frame == request.frame
            && result.zodiac_mode == request.zodiac_mode
            && result.apparent == request.apparent;

        match request.frame {
            CoordinateFrame::Ecliptic => ecliptic_request_count += 1,
            CoordinateFrame::Equatorial => equatorial_request_count += 1,
            _ => return None,
        }

        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some(PackagedBatchParitySummary {
        request_count: requests.len(),
        body_count: entries.len(),
        ecliptic_request_count,
        equatorial_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
        order_preserved,
        single_query_parity_preserved: single_query_parity,
    })
}

impl PackagedBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let order = if self.order_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        let parity = if self.single_query_parity_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        format!(
            "Packaged mixed frame batch parity: {} requests across {} bodies, ecliptic requests={}, equatorial requests={}; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; order={}, single-query parity={}",
            self.request_count,
            self.body_count,
            self.ecliptic_request_count,
            self.equatorial_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
            order,
            parity,
        )
    }
}

impl fmt::Display for PackagedBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn format_validated_packaged_mixed_frame_batch_parity_summary_for_report(
    summary: &PackagedBatchParitySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged mixed frame batch parity: unavailable ({error})"),
    }
}

/// Returns the packaged mixed-frame batch-parity summary.
pub fn packaged_mixed_frame_batch_parity_summary_for_report() -> String {
    packaged_mixed_frame_batch_parity_summary()
        .as_ref()
        .map(format_validated_packaged_mixed_frame_batch_parity_summary_for_report)
        .unwrap_or_else(|| "Packaged mixed frame batch parity: unavailable".to_string())
}

/// Returns the packaged frame-parity summary.
pub fn packaged_frame_parity_summary_for_report() -> String {
    packaged_mixed_frame_batch_parity_summary_for_report()
}

/// Structured mixed TT/TDB batch-parity summary for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedTimeScaleBatchParitySummary {
    /// Number of requests in the mixed-scale batch regression.
    pub request_count: usize,
    /// Number of bodies covered by the batch regression.
    pub body_count: usize,
    /// Number of TT requests in the batch regression.
    pub tt_request_count: usize,
    /// Number of TDB requests in the batch regression.
    pub tdb_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
    /// Whether the batch regression preserved request order.
    pub order_preserved: bool,
    /// Whether the batch regression preserved batch/single parity.
    pub single_query_parity_preserved: bool,
}

/// Validation error for a packaged mixed TT/TDB batch-parity summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedTimeScaleBatchParitySummaryValidationError {
    /// A summary field is out of sync with the current packaged batch-parity posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedTimeScaleBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged mixed TT/TDB batch-parity summary field `{field}` is out of sync with the current packaged posture"
            ),
        }
    }
}

impl std::error::Error for PackagedTimeScaleBatchParitySummaryValidationError {}

fn packaged_mixed_tt_tdb_batch_parity_request_entries(
) -> Option<(Vec<EphemerisRequest>, Vec<SnapshotEntry>)> {
    let snapshot = reference_snapshot();
    let mut requests = Vec::with_capacity(packaged_bodies().len());
    let mut entries = Vec::with_capacity(packaged_bodies().len());

    for (index, body) in packaged_bodies().iter().cloned().enumerate() {
        let entry = packaged_reference_entry_for_body(snapshot, &body)?;
        entries.push(entry.clone());
        requests.push(EphemerisRequest {
            body,
            instant: Instant::new(
                entry.epoch.julian_day,
                if index % 2 == 0 {
                    TimeScale::Tt
                } else {
                    TimeScale::Tdb
                },
            ),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        });
    }

    Some((requests, entries))
}

/// Returns the packaged mixed TT/TDB batch-parity request corpus used by downstream tooling.
pub fn packaged_mixed_tt_tdb_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    packaged_mixed_tt_tdb_batch_parity_request_entries().map(|(requests, _)| requests)
}

/// This is a compatibility alias for [`packaged_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "packaged_mixed_tt_tdb_batch_parity_requests")]
pub fn packaged_mixed_tt_tdb_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    packaged_mixed_tt_tdb_batch_parity_requests()
}

/// Returns a compact mixed TT/TDB batch-parity summary for the packaged artifact.
pub fn packaged_mixed_tt_tdb_batch_parity_summary() -> Option<PackagedTimeScaleBatchParitySummary> {
    let backend = packaged_backend();
    let (requests, entries) = packaged_mixed_tt_tdb_batch_parity_request_entries()?;

    let results = backend.positions(&requests).ok()?;
    if results.len() != requests.len() {
        return None;
    }

    let mut tt_request_count = 0usize;
    let mut tdb_request_count = 0usize;
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for ((request, result), entry) in requests.iter().zip(results.iter()).zip(entries.iter()) {
        let single = backend.position(request).ok();
        single_query_parity &= single.as_ref().is_some_and(|single| single == result);

        order_preserved &= result.body == entry.body
            && result.instant == request.instant
            && result.frame == request.frame
            && result.zodiac_mode == request.zodiac_mode
            && result.apparent == request.apparent;

        match request.instant.scale {
            TimeScale::Tt => tt_request_count += 1,
            TimeScale::Tdb => tdb_request_count += 1,
            _ => return None,
        }

        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some(PackagedTimeScaleBatchParitySummary {
        request_count: requests.len(),
        body_count: entries.len(),
        tt_request_count,
        tdb_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
        order_preserved,
        single_query_parity_preserved: single_query_parity,
    })
}

impl PackagedTimeScaleBatchParitySummary {
    /// Returns `Ok(())` when the summary still matches the current packaged batch-parity posture.
    pub fn validate(&self) -> Result<(), PackagedTimeScaleBatchParitySummaryValidationError> {
        if self.request_count != self.body_count {
            return Err(
                PackagedTimeScaleBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "request_count/body_count",
                },
            );
        }

        if self.tt_request_count + self.tdb_request_count != self.request_count {
            return Err(
                PackagedTimeScaleBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "time_scale_counts",
                },
            );
        }

        if self.tt_request_count == 0 || self.tdb_request_count == 0 {
            return Err(
                PackagedTimeScaleBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "time_scale_mix",
                },
            );
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.request_count
        {
            return Err(
                PackagedTimeScaleBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }

        if !self.order_preserved {
            return Err(
                PackagedTimeScaleBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "order_preserved",
                },
            );
        }

        if !self.single_query_parity_preserved {
            return Err(
                PackagedTimeScaleBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "single_query_parity_preserved",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated batch-parity summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedTimeScaleBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let order = if self.order_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        let parity = if self.single_query_parity_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        format!(
            "Packaged mixed TT/TDB batch parity: {} requests across {} bodies, TT requests={}, TDB requests={}; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; order={}, single-query parity={}",
            self.request_count,
            self.body_count,
            self.tt_request_count,
            self.tdb_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
            order,
            parity,
        )
    }
}

impl fmt::Display for PackagedTimeScaleBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn format_validated_packaged_mixed_tt_tdb_batch_parity_summary_for_report(
    summary: &PackagedTimeScaleBatchParitySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged mixed TT/TDB batch parity: unavailable ({error})"),
    }
}

/// Returns the packaged mixed TT/TDB batch-parity summary.
pub fn packaged_mixed_tt_tdb_batch_parity_summary_for_report() -> String {
    packaged_mixed_tt_tdb_batch_parity_summary()
        .as_ref()
        .map(format_validated_packaged_mixed_tt_tdb_batch_parity_summary_for_report)
        .unwrap_or_else(|| "Packaged mixed TT/TDB batch parity: unavailable".to_string())
}

/// Returns a packaged lookup for a body and instant.
///
/// # Examples
///
/// ```
/// use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
/// use pleiades_data::packaged_lookup;
///
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let sun = packaged_lookup(&CelestialBody::Sun, instant)
///     .expect("Sun should be present in the packaged artifact");
///
/// assert!(sun.distance_au.is_some());
/// ```
pub fn packaged_lookup(
    body: &CelestialBody,
    instant: Instant,
) -> Result<EclipticCoordinates, pleiades_compression::CompressionError> {
    packaged_artifact().lookup_ecliptic(body, normalize_lookup_instant(instant))
}

/// Returns a packaged-data backend instance.
///
/// # Examples
///
/// ```
/// use pleiades_backend::{CoordinateFrame, EphemerisBackend};
/// use pleiades_data::packaged_backend;
///
/// let backend = packaged_backend();
/// let metadata = backend.metadata();
///
/// assert_eq!(metadata.id.as_str(), "pleiades-data");
/// assert!(metadata.offline);
/// assert!(metadata.deterministic);
/// assert!(metadata.supported_frames.contains(&CoordinateFrame::Equatorial));
/// ```
pub fn packaged_backend() -> PackagedDataBackend {
    PackagedDataBackend::new()
}

/// Returns a packaged-data backend built from an explicit artifact.
pub fn packaged_backend_from_artifact(artifact: CompressedArtifact) -> PackagedDataBackend {
    PackagedDataBackend::from_artifact(artifact)
}

/// Returns a packaged-data backend built from decoded artifact bytes.
///
/// # Examples
///
/// ```
/// use pleiades_backend::EphemerisBackend;
/// use pleiades_data::{packaged_artifact, packaged_backend_from_bytes};
///
/// let bytes = packaged_artifact()
///     .encode()
///     .expect("packaged artifact should encode");
/// let backend = packaged_backend_from_bytes(&bytes)
///     .expect("packaged artifact bytes should decode");
///
/// assert_eq!(backend.metadata().id.as_str(), "pleiades-data");
/// assert!(backend.metadata().offline);
/// ```
pub fn packaged_backend_from_bytes(
    bytes: &[u8],
) -> Result<PackagedDataBackend, PackagedArtifactLoadError> {
    PackagedDataBackend::from_bytes(bytes)
}

#[cfg(feature = "packaged-artifact-path")]
/// Returns a packaged-data backend built from a decoded artifact file.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use pleiades_backend::EphemerisBackend;
/// use pleiades_data::{packaged_artifact, packaged_backend_from_path};
///
/// let bytes = packaged_artifact()
///     .encode()
///     .expect("packaged artifact should encode");
/// let path = std::env::temp_dir().join(format!(
///     "pleiades-packaged-artifact-{}.bin",
///     std::process::id()
/// ));
/// fs::write(&path, &bytes).expect("artifact fixture should be writable");
/// let backend = packaged_backend_from_path(&path)
///     .expect("packaged artifact file should decode");
/// assert_eq!(backend.metadata().id.as_str(), "pleiades-data");
/// let _ = fs::remove_file(&path);
/// ```
pub fn packaged_backend_from_path(
    path: impl AsRef<Path>,
) -> Result<PackagedDataBackend, PackagedArtifactLoadError> {
    PackagedDataBackend::from_path(path)
}
