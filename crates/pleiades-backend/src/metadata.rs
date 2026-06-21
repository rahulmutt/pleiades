use crate::capabilities::{BackendCapabilities, BackendCapabilitiesValidationError};
use crate::claims::{BodyClaim, BodyClaimTier};
use crate::errors::{format_display_list, EphemerisError, EphemerisErrorKind};
use crate::identity::{AccuracyClass, BackendFamily, BackendId};
use crate::request::EphemerisRequest;
use crate::validation::{validate_non_blank, validate_non_empty_unique, validate_unique_entries};
use core::fmt;
use pleiades_types::{
    CelestialBody, CoordinateFrame, TimeRange, TimeRangeValidationError, TimeScale, ZodiacMode,
};

/// Provenance summary for a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BackendProvenance {
    /// Short human-readable summary of the backend's source material.
    pub summary: String,
    /// External data or reference sources used by the backend.
    pub data_sources: Vec<String>,
}

impl BackendProvenance {
    /// Creates a new provenance summary.
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            data_sources: Vec::new(),
        }
    }

    /// Returns a compact one-line rendering of the provenance summary.
    pub fn summary_line(&self) -> String {
        self.summary.clone()
    }

    /// Returns `Ok(())` when the provenance summary is internally consistent.
    ///
    /// The shared check keeps backend provenance metadata from silently
    /// carrying blank summary text or duplicate/whitespace-padded source
    /// labels. Empty source lists are allowed for synthesized or routing
    /// backends that do not have external data provenance to list.
    pub fn validate(&self) -> Result<(), BackendProvenanceValidationError> {
        validate_non_blank("provenance summary", &self.summary)
            .map_err(|_| BackendProvenanceValidationError::BlankSummary)?;

        for (index, source) in self.data_sources.iter().enumerate() {
            if source.trim().is_empty() || source.trim() != source {
                return Err(BackendProvenanceValidationError::BlankDataSource { index });
            }
        }

        validate_unique_entries("provenance data sources", &self.data_sources).map_err(|error| {
            match error {
                BackendMetadataValidationError::DuplicateEntry { value, .. } => {
                    BackendProvenanceValidationError::DuplicateDataSource { value }
                }
                _ => {
                    unreachable!("duplicate provenance sources should only fail via DuplicateEntry")
                }
            }
        })
    }

    /// Returns the compact provenance summary after validating it.
    pub fn validated_summary_line(&self) -> Result<String, BackendProvenanceValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Errors returned when backend provenance metadata fails the shared consistency checks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendProvenanceValidationError {
    /// The summary text was blank or whitespace-padded.
    BlankSummary,
    /// A provenance source entry was blank or whitespace-padded.
    BlankDataSource {
        /// Zero-based position of the invalid source entry.
        index: usize,
    },
    /// A provenance source entry appeared more than once.
    DuplicateDataSource {
        /// The duplicated source label.
        value: String,
    },
}

impl BackendProvenanceValidationError {
    /// Returns a compact validation summary string.
    pub fn summary_line(&self) -> String {
        match self {
            Self::BlankSummary => {
                "backend provenance summary must not be blank or whitespace-padded".to_owned()
            }
            Self::BlankDataSource { index } => format!(
                "backend provenance data source at index {index} must not be blank or whitespace-padded"
            ),
            Self::DuplicateDataSource { value } => {
                format!("backend provenance data sources contain duplicate entry `{value}`")
            }
        }
    }
}

impl fmt::Display for BackendProvenanceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for BackendProvenanceValidationError {}

impl fmt::Display for BackendProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary)
    }
}

/// Nominal backend metadata.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BackendMetadata {
    /// Stable backend identifier.
    pub id: BackendId,
    /// Human-readable backend version.
    pub version: String,
    /// Backend family.
    pub family: BackendFamily,
    /// Provenance summary.
    pub provenance: BackendProvenance,
    /// Nominal supported time range.
    pub nominal_range: TimeRange,
    /// Time scales the backend can accept.
    pub supported_time_scales: Vec<TimeScale>,
    /// Supported body coverage and per-body release claims.
    pub body_claims: Vec<BodyClaim>,
    /// Supported coordinate frames.
    pub supported_frames: Vec<CoordinateFrame>,
    /// Declared capabilities.
    pub capabilities: BackendCapabilities,
    /// Published accuracy class.
    pub accuracy: AccuracyClass,
    /// Whether repeated queries are deterministic.
    pub deterministic: bool,
    /// Whether the backend runs fully offline.
    pub offline: bool,
}

impl BackendMetadata {
    /// Returns the bodies the backend serves (every tier except `Unsupported`).
    pub fn supported_bodies(&self) -> Vec<CelestialBody> {
        self.body_claims
            .iter()
            .filter(|c| c.tier != BodyClaimTier::Unsupported)
            .map(|c| c.body.clone())
            .collect()
    }

    /// Returns the claim for a body, if declared.
    pub fn claim_for(&self, body: &CelestialBody) -> Option<&BodyClaim> {
        self.body_claims.iter().find(|c| &c.body == body)
    }

    /// Returns the bodies claimed `ReleaseGrade`.
    pub fn release_grade_bodies(&self) -> Vec<CelestialBody> {
        self.body_claims
            .iter()
            .filter(|c| c.tier == BodyClaimTier::ReleaseGrade)
            .map(|c| c.body.clone())
            .collect()
    }

    /// Returns claims at a given tier.
    pub fn claims_by_tier(&self, tier: BodyClaimTier) -> Vec<&BodyClaim> {
        self.body_claims.iter().filter(|c| c.tier == tier).collect()
    }

    /// Returns a compact one-line rendering of the backend metadata posture.
    pub fn summary_line(&self) -> String {
        format!(
            "id={}; version={}; family={}; family posture={}; accuracy={}; deterministic={}; offline={}; nominal range={}; time scales=[{}]; bodies=[{}]; frames=[{}]; capabilities=[{}]; provenance={}",
            self.id,
            self.version,
            self.family,
            self.family.posture_label(),
            self.accuracy,
            self.deterministic,
            self.offline,
            self.nominal_range,
            format_display_list(&self.supported_time_scales),
            self.body_claims
                .iter()
                .map(BodyClaim::summary_line)
                .collect::<Vec<_>>()
                .join(", "),
            format_display_list(&self.supported_frames),
            self.capabilities.summary_line(),
            self.provenance.summary_line(),
        )
    }

    /// Returns the compact backend metadata summary after validating the stored fields.
    pub fn validated_summary_line(&self) -> Result<String, BackendMetadataValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates a request shape against this metadata before backend computation.
    ///
    /// Routing backends still defer frame, time-scale, value-mode, and zodiac
    /// checks to the selected provider, but they continue to validate the
    /// request's custom definitions, observer syntax, and body coverage here so
    /// unsupported shapes fail closed before execution.
    pub fn validate_request(&self, req: &EphemerisRequest) -> Result<(), EphemerisError> {
        req.validate_custom_definitions()?;

        if !self.family.is_routing() {
            crate::policy::current::validate_request_policy(
                req,
                self.id.as_str(),
                &self.supported_time_scales,
                &self.supported_frames,
                self.capabilities.mean,
                self.capabilities.apparent,
            )?;

            if !self.capabilities.native_sidereal {
                crate::policy::current::validate_zodiac_policy(
                    req,
                    self.id.as_str(),
                    &[ZodiacMode::Tropical],
                )?;
            }

            crate::policy::current::validate_request_observer_location(req)?;
            crate::policy::current::validate_observer_policy(
                req,
                self.id.as_str(),
                self.capabilities.topocentric,
            )?;
        } else {
            crate::policy::current::validate_request_observer_location(req)?;
        }

        if !self.supported_bodies().contains(&req.body) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                format!("{} does not support {}", self.id, req.body),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for BackendMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Errors returned when backend metadata fails the shared consistency checks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendMetadataValidationError {
    /// A required metadata field is blank or whitespace-padded.
    BlankField { field: &'static str },
    /// A required list field is empty.
    EmptyField { field: &'static str },
    /// A catalog-style list field contains a duplicate entry.
    DuplicateEntry { field: &'static str, value: String },
    /// The nominal range contains a non-finite Julian-day bound.
    NominalRangeNotFinite,
    /// The nominal range bounds use different time scales.
    NominalRangeScaleMismatch,
    /// The nominal range end precedes the start.
    NominalRangeOutOfOrder,
    /// The declared capability flags are internally inconsistent.
    InvalidCapabilities {
        /// The invalid field name.
        field: &'static str,
        /// A short description of the capability mismatch.
        message: &'static str,
    },
}

impl BackendMetadataValidationError {
    /// Returns a compact validation summary string.
    pub fn summary_line(&self) -> String {
        match self {
            Self::BlankField { field } => {
                format!("backend metadata field `{field}` is blank or whitespace-padded")
            }
            Self::EmptyField { field } => {
                format!("backend metadata field `{field}` must not be empty")
            }
            Self::DuplicateEntry { field, value } => {
                format!("backend metadata field `{field}` contains duplicate entry `{value}`")
            }
            Self::NominalRangeNotFinite => {
                "backend metadata nominal range must use finite Julian-day bounds".to_owned()
            }
            Self::NominalRangeScaleMismatch => {
                "backend metadata nominal range bounds must use the same time scale".to_owned()
            }
            Self::NominalRangeOutOfOrder => {
                "backend metadata nominal range end must not precede the start".to_owned()
            }
            Self::InvalidCapabilities { field, message } => {
                format!("backend metadata field `{field}` is invalid: {message}")
            }
        }
    }
}

impl fmt::Display for BackendMetadataValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for BackendMetadataValidationError {}

impl BackendMetadata {
    /// Returns `Ok(())` when the metadata is internally consistent.
    ///
    /// The shared check keeps the release-facing backend inventory from
    /// silently advertising blank identifiers, duplicate coverage entries, or
    /// an invalid nominal range. It does not attempt to validate source-specific
    /// accuracy claims; those still belong to the backend crate that owns the
    /// data.
    pub fn validate(&self) -> Result<(), BackendMetadataValidationError> {
        validate_non_blank("id", self.id.as_str())?;
        validate_non_blank("version", &self.version)?;
        self.provenance.validate().map_err(|error| match error {
            BackendProvenanceValidationError::BlankSummary => {
                BackendMetadataValidationError::BlankField {
                    field: "provenance summary",
                }
            }
            BackendProvenanceValidationError::BlankDataSource { .. } => {
                BackendMetadataValidationError::BlankField {
                    field: "provenance data sources",
                }
            }
            BackendProvenanceValidationError::DuplicateDataSource { value } => {
                BackendMetadataValidationError::DuplicateEntry {
                    field: "provenance data sources",
                    value,
                }
            }
        })?;
        validate_non_empty_unique("supported time scales", &self.supported_time_scales)?;
        if self.body_claims.is_empty() {
            return Err(BackendMetadataValidationError::EmptyField {
                field: "body claims",
            });
        }
        let mut seen: Vec<CelestialBody> = Vec::new();
        for claim in &self.body_claims {
            if seen.contains(&claim.body) {
                return Err(BackendMetadataValidationError::DuplicateEntry {
                    field: "body claims",
                    value: claim.body.to_string(),
                });
            }
            seen.push(claim.body.clone());
        }
        validate_non_empty_unique("supported frames", &self.supported_frames)?;
        self.capabilities.validate().map_err(|error| match error {
            BackendCapabilitiesValidationError::MissingPositionMode => {
                BackendMetadataValidationError::InvalidCapabilities {
                    field: "capabilities",
                    message: error.summary_line(),
                }
            }
            BackendCapabilitiesValidationError::MissingValueMode => {
                BackendMetadataValidationError::InvalidCapabilities {
                    field: "capabilities",
                    message: error.summary_line(),
                }
            }
        })?;
        self.validate_nominal_range()?;
        Ok(())
    }

    fn validate_nominal_range(&self) -> Result<(), BackendMetadataValidationError> {
        match self.nominal_range.validate() {
            Ok(()) => Ok(()),
            Err(TimeRangeValidationError::NonFiniteBound { .. }) => {
                Err(BackendMetadataValidationError::NominalRangeNotFinite)
            }
            Err(TimeRangeValidationError::ScaleMismatch { .. }) => {
                Err(BackendMetadataValidationError::NominalRangeScaleMismatch)
            }
            Err(TimeRangeValidationError::OutOfOrder { .. }) => {
                Err(BackendMetadataValidationError::NominalRangeOutOfOrder)
            }
        }
    }
}

/// Merges two claim lists, keeping the stronger-ranked tier on body collisions.
pub fn merge_body_claims(a: &[BodyClaim], b: &[BodyClaim]) -> Vec<BodyClaim> {
    let mut out: Vec<BodyClaim> = a.to_vec();
    for claim in b {
        match out.iter_mut().find(|c| c.body == claim.body) {
            Some(existing) => {
                if claim.tier.rank() > existing.tier.rank() {
                    *existing = claim.clone();
                }
            }
            None => out.push(claim.clone()),
        }
    }
    out
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
