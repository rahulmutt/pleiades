use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use pleiades_backend::EphemerisBackend;
use pleiades_backend::{
    Angle, Apparentness, CelestialBody, CoordinateFrame, EphemerisRequest, Instant, JulianDay,
    TimeRange, TimeScale, ZodiacMode,
};
use pleiades_compression::{
    join_display, ArtifactOutput, ArtifactOutputSupport, ArtifactProfile,
    ArtifactProfileCoverageSummary, ArtifactResidualBodyCoverageSummary, ChannelKind,
    CompressedArtifact, EndianPolicy, PolynomialChannel, Segment, SpeedPolicy,
};
use pleiades_jpl::{
    comparison_snapshot_body_class_coverage_summary, format_reference_snapshot_summary,
    independent_holdout_snapshot_body_class_coverage_summary, production_generation_source_summary,
    production_generation_source_summary_for_report, reference_snapshot_summary,
    selected_asteroid_source_request_corpus_summary, JplSnapshotBackend,
    ProductionGenerationSourceSummary, ReferenceSnapshotSummary,
};

use crate::data::packaged_artifact;
use crate::lookup::{
    packaged_artifact_storage_summary_details, packaged_backend,
    packaged_frame_treatment_summary_details, packaged_lookup_epoch_policy_summary_details,
    packaged_request_policy_summary_details, PackagedArtifactStorageSummary,
    PackagedFrameTreatmentSummary, PackagedLookupEpochPolicy, PackagedRequestPolicySummary,
};
use crate::regenerate::{
    artifact_time_range, packaged_artifact_segment_validation_fractions_for_body,
    polynomial_channel_from_samples, PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS,
};
use crate::{
    packaged_artifact_generation_policy_note_text, packaged_artifact_source_text, packaged_bodies,
    ARTIFACT_LABEL, ARTIFACT_PROFILE_ID,
};

/// Structured coverage summary for the bodies bundled into the packaged artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedBodyCoverageSummary {
    /// Number of bodies bundled into the packaged artifact.
    pub body_count: usize,
    /// Bodies bundled into the packaged artifact.
    pub bodies: Vec<CelestialBody>,
}

/// Validation error for a packaged-body coverage summary that drifted from the bundled body set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedBodyCoverageSummaryValidationError {
    /// A rendered summary field no longer matches the current packaged body set.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedBodyCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged body coverage summary field `{field}` is out of sync with the current bundled body set"
            ),
        }
    }
}

impl std::error::Error for PackagedBodyCoverageSummaryValidationError {}

impl PackagedBodyCoverageSummary {
    /// Returns the bundled body set as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged body set: {} bundled bodies ({})",
            self.body_count,
            join_display(&self.bodies)
        )
    }

    /// Returns `Ok(())` when the summary still matches the bundled body set.
    pub fn validate(&self) -> Result<(), PackagedBodyCoverageSummaryValidationError> {
        let expected_bodies = packaged_bodies();

        if self.body_count != expected_bodies.len() {
            return Err(PackagedBodyCoverageSummaryValidationError::FieldOutOfSync {
                field: "body_count",
            });
        }
        if self.bodies.as_slice() != expected_bodies {
            return Err(PackagedBodyCoverageSummaryValidationError::FieldOutOfSync {
                field: "bodies",
            });
        }

        Ok(())
    }

    /// Returns the bundled body set as a validated compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedBodyCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedBodyCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured packaged body coverage summary.
pub fn packaged_body_coverage_summary_details() -> PackagedBodyCoverageSummary {
    let bodies = packaged_bodies().to_vec();
    PackagedBodyCoverageSummary {
        body_count: bodies.len(),
        bodies,
    }
}

pub(crate) fn format_validated_packaged_body_coverage_summary_for_report(
    summary: &PackagedBodyCoverageSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Packaged body set: unavailable ({error})"),
    }
}

/// Returns the packaged body set as a human-readable provenance summary.
pub fn packaged_body_coverage_summary() -> String {
    format_validated_packaged_body_coverage_summary_for_report(
        &packaged_body_coverage_summary_details(),
    )
}

/// Structured generation policy for the packaged artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedArtifactGenerationPolicy {
    /// Same-body source epochs are fit with adjacent quadratic windows.
    AdjacentSameBodyQuadraticWindows,
}

/// Validation error for the packaged-artifact generation policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactGenerationPolicyValidationError {
    /// A policy field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactGenerationPolicyValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact generation policy field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactGenerationPolicyValidationError {}

impl PackagedArtifactGenerationPolicy {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::AdjacentSameBodyQuadraticWindows => "adjacent same-body quadratic windows",
        }
    }

    /// Returns the explanatory note used in release-facing summaries.
    pub fn note(self) -> &'static str {
        match self {
            Self::AdjacentSameBodyQuadraticWindows => {
                packaged_artifact_generation_policy_note_text()
            }
        }
    }

    /// Returns the segment-strategy text used in release-facing summaries.
    pub fn segment_strategy(self) -> &'static str {
        self.note()
    }

    /// Returns the compact release-facing summary for the generation policy.
    pub fn summary_line(self) -> String {
        format!("{}; {}", self.label(), self.note())
    }

    /// Returns `Ok(())` when the generation policy still matches the current packaged-artifact posture.
    pub fn validate(self) -> Result<(), PackagedArtifactGenerationPolicyValidationError> {
        if self != Self::AdjacentSameBodyQuadraticWindows {
            return Err(
                PackagedArtifactGenerationPolicyValidationError::FieldOutOfSync { field: "policy" },
            );
        }

        Ok(())
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Structured summary for the packaged-artifact generation policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedArtifactGenerationPolicySummary {
    /// Policy describing how the packaged artifact is generated.
    pub policy: PackagedArtifactGenerationPolicy,
}

/// Validation error for the packaged-artifact generation policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactGenerationPolicySummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactGenerationPolicySummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact generation policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactGenerationPolicySummaryValidationError {}

pub(crate) fn validate_packaged_artifact_generation_policy_residual_bodies(
    policy: PackagedArtifactGenerationPolicy,
    residual_bodies: &[CelestialBody],
) -> Result<(), PackagedArtifactGenerationPolicySummaryValidationError> {
    match policy {
        PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows => {
            let expected_residual_bodies = packaged_artifact().residual_bodies();
            if residual_bodies != expected_residual_bodies.as_slice() {
                return Err(
                    PackagedArtifactGenerationPolicySummaryValidationError::FieldOutOfSync {
                        field: "residual_bodies",
                    },
                );
            }
        }
    }

    Ok(())
}

impl PackagedArtifactGenerationPolicySummary {
    /// Returns the packaged-artifact generation policy as a compact human-readable line.
    pub fn summary_line(self) -> String {
        self.policy.summary_line()
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactGenerationPolicySummaryValidationError> {
        self.policy.validate().map_err(|error| match error {
            PackagedArtifactGenerationPolicyValidationError::FieldOutOfSync { field } => {
                PackagedArtifactGenerationPolicySummaryValidationError::FieldOutOfSync { field }
            }
        })?;

        validate_packaged_artifact_generation_policy_residual_bodies(
            self.policy,
            &packaged_artifact().residual_bodies(),
        )
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const PACKAGED_ARTIFACT_GENERATION_POLICY_SUMMARY: PackagedArtifactGenerationPolicySummary =
    PackagedArtifactGenerationPolicySummary {
        policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
    };

/// Returns the current packaged-artifact generation policy summary record.
///
/// # Examples
///
/// ```
/// use pleiades_data::{
///     packaged_artifact_generation_policy_summary_details,
///     packaged_artifact_generation_policy_summary_for_report,
/// };
///
/// let summary = packaged_artifact_generation_policy_summary_details();
/// assert_eq!(summary.to_string(), packaged_artifact_generation_policy_summary_for_report());
/// assert!(summary.validate().is_ok());
/// ```
pub fn packaged_artifact_generation_policy_summary_details(
) -> PackagedArtifactGenerationPolicySummary {
    let summary = PACKAGED_ARTIFACT_GENERATION_POLICY_SUMMARY;
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact generation policy summary after validating the structured posture.
pub fn packaged_artifact_generation_policy_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_generation_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.to_string(),
                Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact residual-bearing body coverage summary record.
pub fn packaged_artifact_generation_residual_bodies_summary_details(
) -> ArtifactResidualBodyCoverageSummary {
    let artifact = packaged_artifact();
    let summary = artifact.residual_body_coverage_summary();
    debug_assert!(summary.validate(artifact).is_ok());
    summary
}

/// Returns the current packaged-artifact residual-bearing body set after validating the structured posture.
pub fn packaged_artifact_generation_residual_bodies_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let artifact = packaged_artifact();
            let summary = packaged_artifact_generation_residual_bodies_summary_details();

            match summary.validated_summary_line_with_body_count(artifact) {
                Ok(line) => line,
                Err(error) => format!("residual bodies: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact generation policy summary.
pub fn packaged_artifact_generation_policy_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_generation_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.to_string(),
                Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
            }
        })
        .as_str()
}

/// Structured fit envelope for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitEnvelopeSummary {
    /// Number of successfully measured segment samples.
    pub sample_count: usize,
    /// Number of planned segment samples for the current artifact layout.
    pub expected_sample_count: usize,
    /// Number of bundled bodies covered by the measured sample set.
    pub body_count: usize,
    /// Mean absolute longitude delta in degrees.
    pub mean_longitude_delta_degrees: f64,
    /// Mean absolute latitude delta in degrees.
    pub mean_latitude_delta_degrees: f64,
    /// Mean absolute distance delta in AU.
    pub mean_distance_delta_au: f64,
    /// Maximum absolute longitude delta in degrees.
    pub max_longitude_delta_degrees: f64,
    /// Maximum absolute latitude delta in degrees.
    pub max_latitude_delta_degrees: f64,
    /// Maximum absolute distance delta in AU.
    pub max_distance_delta_au: f64,
}

/// A packaged-artifact fit threshold violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedArtifactFitThresholdViolation {
    /// The field that exceeds the calibrated threshold.
    pub field: &'static str,
    /// The measured value, encoded as raw bits so the summary stays lossless.
    pub measured_bits: u64,
    /// The calibrated threshold, encoded as raw bits so the summary stays lossless.
    pub threshold_bits: u64,
    /// The amount by which the measured value exceeds the calibrated threshold.
    pub overage_bits: u64,
}

impl PackagedArtifactFitThresholdViolation {
    fn summary_line(&self) -> String {
        format!(
            "`{}` measured={:.12}, threshold={:.12}, overage={:+.12}",
            self.field,
            f64::from_bits(self.measured_bits),
            f64::from_bits(self.threshold_bits),
            f64::from_bits(self.overage_bits),
        )
    }
}

fn packaged_artifact_fit_threshold_violations_from_envelope_and_thresholds(
    envelope: &PackagedArtifactFitEnvelopeSummary,
    thresholds: &PackagedArtifactFitThresholdSummary,
) -> Vec<PackagedArtifactFitThresholdViolation> {
    let mut violations = Vec::new();

    macro_rules! check_threshold {
        ($field:literal, $measured:expr, $threshold:expr) => {
            if $measured > $threshold {
                violations.push(PackagedArtifactFitThresholdViolation {
                    field: $field,
                    measured_bits: $measured.to_bits(),
                    threshold_bits: $threshold.to_bits(),
                    overage_bits: ($measured - $threshold).to_bits(),
                });
            }
        };
    }

    check_threshold!(
        "mean_longitude_delta_degrees",
        envelope.mean_longitude_delta_degrees,
        thresholds.max_mean_longitude_delta_degrees
    );
    check_threshold!(
        "mean_latitude_delta_degrees",
        envelope.mean_latitude_delta_degrees,
        thresholds.max_mean_latitude_delta_degrees
    );
    check_threshold!(
        "mean_distance_delta_au",
        envelope.mean_distance_delta_au,
        thresholds.max_mean_distance_delta_au
    );
    check_threshold!(
        "max_longitude_delta_degrees",
        envelope.max_longitude_delta_degrees,
        thresholds.max_longitude_delta_degrees
    );
    check_threshold!(
        "max_latitude_delta_degrees",
        envelope.max_latitude_delta_degrees,
        thresholds.max_latitude_delta_degrees
    );
    check_threshold!(
        "max_distance_delta_au",
        envelope.max_distance_delta_au,
        thresholds.max_distance_delta_au
    );

    violations
}

/// Validation error for a packaged-artifact fit envelope that drifted from the current posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedArtifactFitEnvelopeSummaryValidationError {
    /// A rendered summary field no longer matches the current packaged-artifact fit envelope.
    FieldOutOfSync { field: &'static str },
    /// One or more measured fit fields exceed the calibrated packaged-artifact fit thresholds.
    ThresholdExceeded {
        violations: Vec<PackagedArtifactFitThresholdViolation>,
    },
}

impl PackagedArtifactFitEnvelopeSummaryValidationError {
    /// Returns the number of threshold violations captured by the validation error.
    pub fn violation_count(&self) -> usize {
        match self {
            Self::FieldOutOfSync { .. } => 0,
            Self::ThresholdExceeded { violations } => violations.len(),
        }
    }

    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit envelope summary field `{field}` is out of sync with the current posture"
            ),
            Self::ThresholdExceeded { violations } => {
                let rendered = violations
                    .iter()
                    .map(PackagedArtifactFitThresholdViolation::summary_line)
                    .collect::<Vec<_>>()
                    .join("; ");
                let violation_count = violations.len();
                let violation_label = if violation_count == 1 {
                    "violation"
                } else {
                    "violations"
                };
                format!(
                    "the packaged artifact fit envelope summary exceeds the calibrated fit thresholds ({violation_count} {violation_label}): {rendered}"
                )
            }
        }
    }
}

impl fmt::Display for PackagedArtifactFitEnvelopeSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitEnvelopeSummaryValidationError {}

/// Calibrated fit thresholds for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitThresholdSummary {
    /// Maximum allowed mean absolute longitude delta in degrees.
    pub max_mean_longitude_delta_degrees: f64,
    /// Maximum allowed mean absolute latitude delta in degrees.
    pub max_mean_latitude_delta_degrees: f64,
    /// Maximum allowed mean absolute distance delta in AU.
    pub max_mean_distance_delta_au: f64,
    /// Maximum allowed absolute longitude delta in degrees.
    pub max_longitude_delta_degrees: f64,
    /// Maximum allowed absolute latitude delta in degrees.
    pub max_latitude_delta_degrees: f64,
    /// Maximum allowed absolute distance delta in AU.
    pub max_distance_delta_au: f64,
}

/// Validation error for a packaged-artifact fit threshold summary that drifted from the current posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedArtifactFitThresholdSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact fit threshold posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactFitThresholdSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit threshold summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactFitThresholdSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitThresholdSummaryValidationError {}

impl PackagedArtifactFitThresholdSummary {
    /// Returns the calibrated fit thresholds as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "fit thresholds: mean Δlon≤{:.12}°, mean Δlat≤{:.12}°, mean Δdist≤{:.12} AU; max Δlon≤{:.12}°, max Δlat≤{:.12}°, max Δdist≤{:.12} AU",
            self.max_mean_longitude_delta_degrees,
            self.max_mean_latitude_delta_degrees,
            self.max_mean_distance_delta_au,
            self.max_longitude_delta_degrees,
            self.max_latitude_delta_degrees,
            self.max_distance_delta_au,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact fit thresholds.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitThresholdSummaryValidationError> {
        if self != &PACKAGED_ARTIFACT_FIT_THRESHOLD_SUMMARY {
            return Err(
                PackagedArtifactFitThresholdSummaryValidationError::FieldOutOfSync {
                    field: "thresholds",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated calibrated fit thresholds as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitThresholdSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitThresholdSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured fit margins for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitMarginSummary {
    /// Measured fit envelope for the current packaged artifact.
    pub envelope: PackagedArtifactFitEnvelopeSummary,
    /// Calibrated thresholds used to compute the current margins.
    pub thresholds: PackagedArtifactFitThresholdSummary,
}

impl PackagedArtifactFitMarginSummary {
    /// Returns the fit margins relative to the calibrated thresholds as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "fit margins: mean Δlon={:+.12}°, mean Δlat={:+.12}°, mean Δdist={:+.12} AU; max Δlon={:+.12}°, max Δlat={:+.12}°, max Δdist={:+.12} AU",
            self.thresholds.max_mean_longitude_delta_degrees - self.envelope.mean_longitude_delta_degrees,
            self.thresholds.max_mean_latitude_delta_degrees - self.envelope.mean_latitude_delta_degrees,
            self.thresholds.max_mean_distance_delta_au - self.envelope.mean_distance_delta_au,
            self.thresholds.max_longitude_delta_degrees - self.envelope.max_longitude_delta_degrees,
            self.thresholds.max_latitude_delta_degrees - self.envelope.max_latitude_delta_degrees,
            self.thresholds.max_distance_delta_au - self.envelope.max_distance_delta_au,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact fit posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let current_envelope = packaged_artifact_fit_envelope_summary_details();
        if self.envelope != current_envelope {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "envelope",
                },
            );
        }

        let current_thresholds = packaged_artifact_fit_threshold_summary_details();
        if self.thresholds != current_thresholds {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "thresholds",
                },
            );
        }

        self.envelope.validate_against_thresholds(&self.thresholds)
    }

    /// Returns the validated fit margins relative to the calibrated thresholds as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitEnvelopeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitMarginSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Packaged-artifact fit threshold violations captured for the current posture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactFitThresholdViolationsSummary {
    /// Threshold violations ordered by field as they appear in the envelope.
    pub violations: Vec<PackagedArtifactFitThresholdViolation>,
}

/// Validation error for a packaged-artifact fit threshold violation summary that drifted from the current posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedArtifactFitThresholdViolationsSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact fit threshold violation posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactFitThresholdViolationsSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit threshold violation summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactFitThresholdViolationsSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitThresholdViolationsSummaryValidationError {}

impl PackagedArtifactFitThresholdViolationsSummary {
    /// Returns the packaged-artifact fit threshold violations as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        let violation_count = self.violations.len();
        if violation_count == 0 {
            return "fit threshold violations: 0; details: none".to_string();
        }

        let rendered = self
            .violations
            .iter()
            .map(PackagedArtifactFitThresholdViolation::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let violation_label = if violation_count == 1 {
            "violation"
        } else {
            "violations"
        };
        format!(
            "fit threshold violations: {violation_count} {violation_label}; details: {rendered}"
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact fit threshold violation posture.
    pub fn validate(
        &self,
    ) -> Result<(), PackagedArtifactFitThresholdViolationsSummaryValidationError> {
        let current = packaged_artifact_fit_threshold_violation_summary_details();
        if self == &current {
            Ok(())
        } else {
            Err(
                PackagedArtifactFitThresholdViolationsSummaryValidationError::FieldOutOfSync {
                    field: "violations",
                },
            )
        }
    }

    /// Returns the validated packaged-artifact fit threshold violations as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitThresholdViolationsSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitThresholdViolationsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl PackagedArtifactFitEnvelopeSummary {
    /// Returns `Ok(())` when the measured fit envelope stays within the calibrated thresholds.
    pub fn validate_against_thresholds(
        &self,
        thresholds: &PackagedArtifactFitThresholdSummary,
    ) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let violations = packaged_artifact_fit_threshold_violations_from_envelope_and_thresholds(
            self, thresholds,
        );

        if violations.is_empty() {
            Ok(())
        } else {
            Err(PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded { violations })
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PackagedArtifactFitSample {
    pub(crate) body: CelestialBody,
    pub(crate) segment_start: Instant,
    pub(crate) segment_end: Instant,
    pub(crate) sample_instant: Instant,
    pub(crate) sample_fraction: f64,
    pub(crate) longitude_delta_degrees: f64,
    pub(crate) latitude_delta_degrees: f64,
    pub(crate) distance_delta_au: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct PackagedArtifactFitSegmentFamilyKey {
    segment_start_days_bits: u64,
    segment_start_scale: TimeScale,
    segment_end_days_bits: u64,
    segment_end_scale: TimeScale,
}

impl PackagedArtifactFitSegmentFamilyKey {
    fn from_sample(sample: &PackagedArtifactFitSample) -> Self {
        Self {
            segment_start_days_bits: sample.segment_start.julian_day.days().to_bits(),
            segment_start_scale: sample.segment_start.scale,
            segment_end_days_bits: sample.segment_end.julian_day.days().to_bits(),
            segment_end_scale: sample.segment_end.scale,
        }
    }
}

#[derive(Clone, Debug)]
struct PackagedArtifactFitChannelFamilyAccumulator {
    sample_count: usize,
    worst_sample: Option<PackagedArtifactFitSample>,
}

impl PackagedArtifactFitChannelFamilyAccumulator {
    fn new() -> Self {
        Self {
            sample_count: 0,
            worst_sample: None,
        }
    }

    fn push(&mut self, sample: &PackagedArtifactFitSample, channel: ChannelKind) {
        self.sample_count += 1;
        let should_replace = self
            .worst_sample
            .as_ref()
            .map(|existing| {
                let existing_delta = packaged_artifact_fit_channel_delta(existing, channel);
                let candidate_delta = packaged_artifact_fit_channel_delta(sample, channel);
                candidate_delta > existing_delta
                    || (candidate_delta == existing_delta
                        && (sample.segment_end.julian_day.days()
                            - sample.segment_start.julian_day.days())
                            < (existing.segment_end.julian_day.days()
                                - existing.segment_start.julian_day.days()))
            })
            .unwrap_or(true);

        if should_replace {
            self.worst_sample = Some(sample.clone());
        }
    }

    fn finish(self, channel: ChannelKind) -> Option<PackagedArtifactFitChannelOutlier> {
        self.worst_sample.as_ref().map(|sample| {
            PackagedArtifactFitChannelOutlier::from_sample(sample, channel, self.sample_count)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitChannelOutlier {
    /// Channel whose fit error is being tracked.
    pub channel: ChannelKind,
    /// Absolute delta for the channel.
    pub delta: f64,
    /// Inclusive segment start for the source interval.
    pub segment_start: Instant,
    /// Inclusive segment end for the source interval.
    pub segment_end: Instant,
    /// Segment span in days.
    pub segment_span_days: f64,
    /// Sample instant that produced the tracked delta.
    pub sample_instant: Instant,
    /// Sample position inside the segment, expressed as a normalized fraction.
    pub sample_fraction: f64,
    /// Number of fit samples that shared the same body/channel/segment family.
    pub sample_count: usize,
}

impl PackagedArtifactFitChannelOutlier {
    fn from_sample(
        sample: &PackagedArtifactFitSample,
        channel: ChannelKind,
        sample_count: usize,
    ) -> Self {
        Self {
            channel,
            delta: packaged_artifact_fit_channel_delta(sample, channel),
            segment_start: sample.segment_start,
            segment_end: sample.segment_end,
            segment_span_days: sample.segment_end.julian_day.days()
                - sample.segment_start.julian_day.days(),
            sample_instant: sample.sample_instant,
            sample_fraction: sample.sample_fraction,
            sample_count,
        }
    }

    fn delta_unit(&self) -> &'static str {
        match self.channel {
            ChannelKind::Longitude | ChannelKind::Latitude => "°",
            ChannelKind::DistanceAu => " AU",
            _ => unreachable!("unsupported packaged-artifact channel kind"),
        }
    }

    fn summary_line(&self) -> String {
        format!(
            "{}={:.12}{} @ {} (segment {} → {}, span={:.12} d, x={:.3}, samples={})",
            self.channel,
            self.delta,
            self.delta_unit(),
            self.sample_instant,
            self.segment_start,
            self.segment_end,
            self.segment_span_days,
            self.sample_fraction,
            self.sample_count,
        )
    }
}

impl fmt::Display for PackagedArtifactFitChannelOutlier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitBodyOutlierSummary {
    /// Bundled body whose fit outliers are summarized.
    pub body: CelestialBody,
    /// Worst sampled delta for each stored channel.
    pub channel_outliers: Vec<PackagedArtifactFitChannelOutlier>,
}

impl PackagedArtifactFitBodyOutlierSummary {
    pub(crate) fn summary_line(&self) -> String {
        format!("{}{{{}}}", self.body, join_display(&self.channel_outliers))
    }
}

impl fmt::Display for PackagedArtifactFitBodyOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitOutlierSummary {
    /// Number of bundled bodies represented in the outlier report.
    pub body_count: usize,
    /// Body-level summaries for the worst sampled fit deltas.
    pub body_summaries: Vec<PackagedArtifactFitBodyOutlierSummary>,
}

impl PackagedArtifactFitOutlierSummary {
    /// Returns the body/channel fit outliers as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "fit outliers: {} bundled bodies; {}",
            self.body_count,
            join_display(&self.body_summaries)
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged artifact.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let expected = packaged_artifact_fit_outlier_summary_details();
        if self != &expected {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "outlier summary",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body/channel fit outliers as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitEnvelopeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitChannelOutlierSummary {
    /// Channel-level summaries for the worst sampled fit deltas.
    pub channel_summaries: Vec<String>,
}

/// Validation error for a packaged-artifact fit outlier-by-channel summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactFitChannelOutlierSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactFitChannelOutlierSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit outlier-by-channel summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactFitChannelOutlierSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitChannelOutlierSummaryValidationError {}

impl PackagedArtifactFitChannelOutlierSummary {
    /// Returns the channel-level fit outliers as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        if self.channel_summaries.is_empty() {
            "fit outliers by channel: none".to_string()
        } else {
            format!(
                "fit outliers by channel: {}",
                self.channel_summaries.join("; ")
            )
        }
    }

    /// Returns `Ok(())` when the summary still matches the current packaged artifact.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitChannelOutlierSummaryValidationError> {
        let expected = packaged_artifact_fit_channel_outlier_summary_details();
        if self != &expected {
            return Err(
                PackagedArtifactFitChannelOutlierSummaryValidationError::FieldOutOfSync {
                    field: "channel_summaries",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated channel-level fit outliers as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitChannelOutlierSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitChannelOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn packaged_artifact_fit_channel_outlier_summary_for_channel(
    samples: &[PackagedArtifactFitSample],
    channel: ChannelKind,
) -> Option<String> {
    let mut families: HashMap<
        (CelestialBody, PackagedArtifactFitSegmentFamilyKey),
        PackagedArtifactFitChannelFamilyAccumulator,
    > = HashMap::new();

    for sample in samples {
        let family_key = PackagedArtifactFitSegmentFamilyKey::from_sample(sample);
        let entry = families
            .entry((sample.body.clone(), family_key))
            .or_insert_with(PackagedArtifactFitChannelFamilyAccumulator::new);
        entry.push(sample, channel);
    }

    let mut body_outliers: HashMap<CelestialBody, PackagedArtifactFitChannelOutlier> =
        HashMap::new();

    for ((body, _family_key), family) in families {
        let Some(candidate) = family.finish(channel) else {
            continue;
        };
        match body_outliers.get_mut(&body) {
            Some(existing) if existing.delta > candidate.delta => {}
            Some(existing)
                if existing.delta == candidate.delta
                    && existing.segment_span_days <= candidate.segment_span_days => {}
            Some(existing) => *existing = candidate,
            None => {
                body_outliers.insert(body, candidate);
            }
        }
    }

    if body_outliers.is_empty() {
        return None;
    }

    let mut body_entries = body_outliers
        .into_iter()
        .map(|(body, outlier)| format!("{body}{{{outlier}}}"))
        .collect::<Vec<_>>();
    body_entries.sort();

    Some(format!("{channel}{{{}}}", body_entries.join(", ")))
}

/// Returns the current packaged-artifact fit outliers by channel as a structured summary record.
pub fn packaged_artifact_fit_channel_outlier_summary_details(
) -> PackagedArtifactFitChannelOutlierSummary {
    let samples = packaged_artifact_fit_outlier_samples_for_current_artifact();
    let mut channel_summaries = Vec::new();

    for channel in [
        ChannelKind::DistanceAu,
        ChannelKind::Longitude,
        ChannelKind::Latitude,
    ] {
        if let Some(entry) =
            packaged_artifact_fit_channel_outlier_summary_for_channel(samples, channel)
        {
            channel_summaries.push(entry);
        }
    }

    PackagedArtifactFitChannelOutlierSummary { channel_summaries }
}

/// Returns the current packaged-artifact fit outliers by channel after validating the structured posture.
pub fn packaged_artifact_fit_channel_outlier_summary_for_report() -> String {
    let summary = packaged_artifact_fit_channel_outlier_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit outliers by channel: unavailable ({error})"),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactTargetThresholdScopeSummary {
    /// Release-scoped body class that the fit envelope applies to.
    pub scope: &'static str,
    /// Bundled bodies that contribute to the scope envelope.
    pub bodies: Vec<CelestialBody>,
    /// Bundled bodies that contribute to the scope envelope.
    pub body_count: usize,
    /// Measured fit envelope for the scoped body set.
    pub fit_envelope: PackagedArtifactFitEnvelopeSummary,
}

impl PackagedArtifactTargetThresholdScopeSummary {
    /// Returns the scope-specific fit posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "scope={}; bodies={}; {}",
            self.scope,
            format_scope_bodies(&self.bodies),
            self.fit_envelope.summary_line(),
        )
    }

    /// Returns the validated scope-specific fit posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitEnvelopeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the scope summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let expected =
            packaged_artifact_target_threshold_scope_envelope_summary_details(self.scope);
        if self != &expected {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "scope fit envelope",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for PackagedArtifactTargetThresholdScopeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactTargetThresholdScopeEnvelopesSummary {
    /// Scope-specific fit envelopes that make up the current packaged-artifact posture.
    pub scope_envelopes: Vec<PackagedArtifactTargetThresholdScopeSummary>,
}

/// Validation error for a packaged-artifact target-threshold scope envelopes summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged-artifact target-threshold scope envelopes summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError {}

impl PackagedArtifactTargetThresholdScopeEnvelopesSummary {
    /// Returns the scope-envelope posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!("scope envelopes: {}", join_display(&self.scope_envelopes))
    }

    /// Returns `Ok(())` when the scope-envelope posture still matches the current packaged-artifact posture.
    pub fn validate(
        &self,
    ) -> Result<(), PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError> {
        let expected = packaged_artifact_target_threshold_scope_envelopes_summary_details();
        if self != &expected {
            return Err(
                PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError::FieldOutOfSync {
                    field: "scope_envelopes",
                },
            );
        }

        for scope_envelope in &self.scope_envelopes {
            scope_envelope.validate().map_err(|_| {
                PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError::FieldOutOfSync {
                    field: "scope_envelopes",
                }
            })?;
        }

        Ok(())
    }

    /// Returns the validated scope-envelope posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactTargetThresholdScopeEnvelopesSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl PackagedArtifactFitEnvelopeSummary {
    /// Returns the packaged-artifact fit evidence as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "fit envelope: {}/{} segment samples across {} bundled bodies; mean Δlon={:.12}°, mean Δlat={:.12}°, mean Δdist={:.12} AU; max Δlon={:.12}°, max Δlat={:.12}°, max Δdist={:.12} AU",
            self.sample_count,
            self.expected_sample_count,
            self.body_count,
            self.mean_longitude_delta_degrees,
            self.mean_latitude_delta_degrees,
            self.mean_distance_delta_au,
            self.max_longitude_delta_degrees,
            self.max_latitude_delta_degrees,
            self.max_distance_delta_au,
        )
    }

    /// Returns `Ok(())` when the fit envelope still matches the current packaged artifact.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let artifact = packaged_artifact();
        let expected = packaged_artifact_fit_envelope_summary_details();
        let expected_sample_count = packaged_artifact_fit_expected_sample_count(artifact);
        let expected_body_count = artifact.bodies.len();

        if self.expected_sample_count != expected_sample_count {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "expected_sample_count",
                },
            );
        }
        if self.sample_count != expected_sample_count {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != expected_body_count {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }

        if self != &expected {
            for (field, matches) in [
                (
                    "mean_longitude_delta_degrees",
                    self.mean_longitude_delta_degrees == expected.mean_longitude_delta_degrees,
                ),
                (
                    "mean_latitude_delta_degrees",
                    self.mean_latitude_delta_degrees == expected.mean_latitude_delta_degrees,
                ),
                (
                    "mean_distance_delta_au",
                    self.mean_distance_delta_au == expected.mean_distance_delta_au,
                ),
                (
                    "max_longitude_delta_degrees",
                    self.max_longitude_delta_degrees == expected.max_longitude_delta_degrees,
                ),
                (
                    "max_latitude_delta_degrees",
                    self.max_latitude_delta_degrees == expected.max_latitude_delta_degrees,
                ),
                (
                    "max_distance_delta_au",
                    self.max_distance_delta_au == expected.max_distance_delta_au,
                ),
            ] {
                if !matches {
                    return Err(
                        PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync { field },
                    );
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for PackagedArtifactFitEnvelopeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
pub(crate) fn packaged_artifact_fit_sample_fractions(segment: &Segment) -> &'static [f64] {
    if segment.start.julian_day.days() == segment.end.julian_day.days() {
        &[0.0]
    } else {
        &[0.25, 0.5, 0.75]
    }
}

pub(crate) fn packaged_artifact_fit_sample_fractions_for_body(
    body: &CelestialBody,
    segment: &Segment,
) -> &'static [f64] {
    if segment.start.julian_day.days() == segment.end.julian_day.days() {
        &[0.0]
    } else {
        match packaged_artifact_body_cadence(body) {
            PackagedArtifactBodyCadence::Luminaries
            | PackagedArtifactBodyCadence::LunarPoints
            | PackagedArtifactBodyCadence::SelectedAsteroids
            | PackagedArtifactBodyCadence::Pluto
            | PackagedArtifactBodyCadence::CustomBodies => {
                packaged_artifact_segment_validation_fractions_for_body(body)
            }
            PackagedArtifactBodyCadence::InnerPlanets
            | PackagedArtifactBodyCadence::OuterPlanets => {
                PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS
            }
        }
    }
}

pub(crate) fn distance_channel_from_samples(
    start: f64,
    midpoint: Option<f64>,
    end: f64,
) -> PolynomialChannel {
    midpoint
        .map(|midpoint| {
            PolynomialChannel::quadratic(ChannelKind::DistanceAu, 10, start, midpoint, end, 0.5)
        })
        .unwrap_or_else(|| PolynomialChannel::linear(ChannelKind::DistanceAu, 10, start, end))
}

pub(crate) fn distance_channel_from_four_point_control_points(
    start: f64,
    first_third: f64,
    second_third: f64,
    end: f64,
) -> Option<PolynomialChannel> {
    polynomial_channel_from_samples(
        ChannelKind::DistanceAu,
        10,
        &[
            (0.0, start),
            (1.0 / 3.0, first_third),
            (2.0 / 3.0, second_third),
            (1.0, end),
        ],
    )
}

fn channel_from_fit_control_points(
    kind: ChannelKind,
    scale_exponent: u8,
    samples: &[(f64, f64)],
) -> Option<PolynomialChannel> {
    const TARGET_FRACTIONS: [f64; 4] = [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];

    if samples.len() < TARGET_FRACTIONS.len() {
        return None;
    }

    let mut selected_samples = Vec::with_capacity(TARGET_FRACTIONS.len());
    let mut used_indices = vec![false; samples.len()];

    for target_fraction in TARGET_FRACTIONS {
        let mut best_index = None;
        let mut best_distance = f64::INFINITY;

        for (index, (fraction, _)) in samples.iter().enumerate() {
            if used_indices[index] {
                continue;
            }

            let distance = (*fraction - target_fraction).abs();
            if distance < best_distance {
                best_distance = distance;
                best_index = Some(index);
            }
        }

        let index = best_index?;

        used_indices[index] = true;
        selected_samples.push(samples[index]);
    }

    polynomial_channel_from_samples(kind, scale_exponent, &selected_samples)
}

pub(crate) fn channel_from_fit_samples_with_control_points(
    kind: ChannelKind,
    scale_exponent: u8,
    samples: &[(f64, f64)],
) -> Option<PolynomialChannel> {
    polynomial_channel_from_samples(kind, scale_exponent, samples)
        .or_else(|| channel_from_fit_control_points(kind, scale_exponent, samples))
}

pub(crate) fn channel_from_dense_fit_samples_with_control_points(
    kind: ChannelKind,
    scale_exponent: u8,
    samples: &[(f64, f64)],
) -> Option<PolynomialChannel> {
    channel_from_fit_control_points(kind, scale_exponent, samples)
        .or_else(|| polynomial_channel_from_samples(kind, scale_exponent, samples))
}

pub(crate) fn distance_channel_from_dense_fit_samples(
    samples: &[(f64, f64)],
    start: f64,
    midpoint: Option<f64>,
    end: f64,
) -> PolynomialChannel {
    channel_from_fit_control_points(ChannelKind::DistanceAu, 10, samples)
        .or_else(|| {
            channel_from_fit_samples_with_control_points(ChannelKind::DistanceAu, 10, samples)
        })
        .unwrap_or_else(|| distance_channel_from_samples(start, midpoint, end))
}

pub(crate) fn distance_channel_from_fit_samples(
    samples: &[(f64, f64)],
    start: f64,
    midpoint: Option<f64>,
    end: f64,
) -> PolynomialChannel {
    channel_from_fit_samples_with_control_points(ChannelKind::DistanceAu, 10, samples)
        .unwrap_or_else(|| distance_channel_from_samples(start, midpoint, end))
}

fn packaged_artifact_fit_expected_sample_count_with_filter<F>(
    artifact: &CompressedArtifact,
    mut include_body: F,
) -> usize
where
    F: FnMut(&CelestialBody) -> bool,
{
    artifact
        .bodies
        .iter()
        .filter(|body| include_body(&body.body))
        .map(|body| {
            body.segments
                .iter()
                .map(|segment| {
                    packaged_artifact_fit_sample_fractions_for_body(&body.body, segment).len()
                })
                .sum::<usize>()
        })
        .sum()
}

fn packaged_artifact_fit_expected_sample_count(artifact: &CompressedArtifact) -> usize {
    packaged_artifact_fit_expected_sample_count_with_filter(artifact, |_| true)
}

fn packaged_artifact_fit_samples_with_filter<F>(
    artifact: &CompressedArtifact,
    mut include_body: F,
) -> Vec<PackagedArtifactFitSample>
where
    F: FnMut(&CelestialBody) -> bool,
{
    let reference_backend = JplSnapshotBackend;
    let packaged_backend = packaged_backend();
    let mut samples = Vec::new();

    for body_artifact in &artifact.bodies {
        if !include_body(&body_artifact.body) {
            continue;
        }

        for segment in &body_artifact.segments {
            let start = segment.start.julian_day.days();
            let span = segment.end.julian_day.days() - start;
            for fraction in
                packaged_artifact_fit_sample_fractions_for_body(&body_artifact.body, segment)
            {
                let instant = Instant::new(
                    JulianDay::from_days(start + span * fraction),
                    segment.start.scale,
                );
                let request = EphemerisRequest {
                    body: body_artifact.body.clone(),
                    instant,
                    observer: None,
                    frame: CoordinateFrame::Ecliptic,
                    zodiac_mode: ZodiacMode::Tropical,
                    apparent: Apparentness::Mean,
                };
                let expected = match reference_backend.position(&request) {
                    Ok(result) => result,
                    Err(_) => continue,
                };
                let actual = match packaged_backend.position(&request) {
                    Ok(result) => result,
                    Err(_) => continue,
                };

                let (Some(expected_ecliptic), Some(actual_ecliptic)) =
                    (expected.ecliptic, actual.ecliptic)
                else {
                    continue;
                };
                let (Some(expected_distance), Some(actual_distance)) =
                    (expected_ecliptic.distance_au, actual_ecliptic.distance_au)
                else {
                    continue;
                };

                samples.push(PackagedArtifactFitSample {
                    body: body_artifact.body.clone(),
                    segment_start: segment.start,
                    segment_end: segment.end,
                    sample_instant: instant,
                    sample_fraction: *fraction,
                    longitude_delta_degrees: Angle::from_degrees(
                        actual_ecliptic.longitude.degrees() - expected_ecliptic.longitude.degrees(),
                    )
                    .normalized_signed()
                    .degrees()
                    .abs(),
                    latitude_delta_degrees: (actual_ecliptic.latitude.degrees()
                        - expected_ecliptic.latitude.degrees())
                    .abs(),
                    distance_delta_au: (actual_distance - expected_distance).abs(),
                });
            }
        }
    }

    samples
}

fn packaged_artifact_fit_samples_for_current_artifact() -> &'static [PackagedArtifactFitSample] {
    static SAMPLES: OnceLock<Vec<PackagedArtifactFitSample>> = OnceLock::new();
    SAMPLES
        .get_or_init(|| {
            let artifact = packaged_artifact();
            packaged_artifact_fit_samples_with_filter(artifact, |_| true)
        })
        .as_slice()
}

pub(crate) fn packaged_artifact_fit_outlier_sample_fractions(
    body: &CelestialBody,
    segment: &Segment,
) -> &'static [f64] {
    if segment.start.julian_day.days() == segment.end.julian_day.days() {
        &[0.0]
    } else {
        packaged_artifact_segment_validation_fractions_for_body(body)
    }
}

fn packaged_artifact_fit_outlier_samples_with_filter<F>(
    artifact: &CompressedArtifact,
    mut include_body: F,
) -> Vec<PackagedArtifactFitSample>
where
    F: FnMut(&CelestialBody) -> bool,
{
    let reference_backend = JplSnapshotBackend;
    let packaged_backend = packaged_backend();
    let mut samples = Vec::new();

    for body_artifact in &artifact.bodies {
        if !include_body(&body_artifact.body) {
            continue;
        }

        for segment in &body_artifact.segments {
            let start = segment.start.julian_day.days();
            let span = segment.end.julian_day.days() - start;
            for fraction in
                packaged_artifact_fit_outlier_sample_fractions(&body_artifact.body, segment)
            {
                let instant = Instant::new(
                    JulianDay::from_days(start + span * fraction),
                    segment.start.scale,
                );
                let request = EphemerisRequest {
                    body: body_artifact.body.clone(),
                    instant,
                    observer: None,
                    frame: CoordinateFrame::Ecliptic,
                    zodiac_mode: ZodiacMode::Tropical,
                    apparent: Apparentness::Mean,
                };
                let expected = match reference_backend.position(&request) {
                    Ok(result) => result,
                    Err(_) => continue,
                };
                let actual = match packaged_backend.position(&request) {
                    Ok(result) => result,
                    Err(_) => continue,
                };

                let (Some(expected_ecliptic), Some(actual_ecliptic)) =
                    (expected.ecliptic, actual.ecliptic)
                else {
                    continue;
                };
                let (Some(expected_distance), Some(actual_distance)) =
                    (expected_ecliptic.distance_au, actual_ecliptic.distance_au)
                else {
                    continue;
                };

                samples.push(PackagedArtifactFitSample {
                    body: body_artifact.body.clone(),
                    segment_start: segment.start,
                    segment_end: segment.end,
                    sample_instant: instant,
                    sample_fraction: *fraction,
                    longitude_delta_degrees: Angle::from_degrees(
                        actual_ecliptic.longitude.degrees() - expected_ecliptic.longitude.degrees(),
                    )
                    .normalized_signed()
                    .degrees()
                    .abs(),
                    latitude_delta_degrees: (actual_ecliptic.latitude.degrees()
                        - expected_ecliptic.latitude.degrees())
                    .abs(),
                    distance_delta_au: (actual_distance - expected_distance).abs(),
                });
            }
        }
    }

    samples
}

fn packaged_artifact_fit_outlier_samples_for_current_artifact(
) -> &'static [PackagedArtifactFitSample] {
    static SAMPLES: OnceLock<Vec<PackagedArtifactFitSample>> = OnceLock::new();
    SAMPLES
        .get_or_init(|| {
            let artifact = packaged_artifact();
            packaged_artifact_fit_outlier_samples_with_filter(artifact, |_| true)
        })
        .as_slice()
}

fn packaged_artifact_fit_envelope_summary_from_samples(
    samples: &[PackagedArtifactFitSample],
    expected_sample_count: usize,
) -> PackagedArtifactFitEnvelopeSummary {
    let sample_count = samples.len();
    let mut observed_bodies = Vec::new();
    let mut mean_longitude_delta_degrees: f64 = 0.0;
    let mut mean_latitude_delta_degrees: f64 = 0.0;
    let mut mean_distance_delta_au: f64 = 0.0;
    let mut max_longitude_delta_degrees: f64 = 0.0;
    let mut max_latitude_delta_degrees: f64 = 0.0;
    let mut max_distance_delta_au: f64 = 0.0;

    for sample in samples {
        if !observed_bodies.contains(&sample.body) {
            observed_bodies.push(sample.body.clone());
        }
        mean_longitude_delta_degrees += sample.longitude_delta_degrees;
        mean_latitude_delta_degrees += sample.latitude_delta_degrees;
        mean_distance_delta_au += sample.distance_delta_au;
        max_longitude_delta_degrees =
            max_longitude_delta_degrees.max(sample.longitude_delta_degrees);
        max_latitude_delta_degrees = max_latitude_delta_degrees.max(sample.latitude_delta_degrees);
        max_distance_delta_au = max_distance_delta_au.max(sample.distance_delta_au);
    }

    if sample_count > 0 {
        let sample_count = sample_count as f64;
        mean_longitude_delta_degrees /= sample_count;
        mean_latitude_delta_degrees /= sample_count;
        mean_distance_delta_au /= sample_count;
    }

    PackagedArtifactFitEnvelopeSummary {
        sample_count,
        expected_sample_count,
        body_count: observed_bodies.len(),
        mean_longitude_delta_degrees,
        mean_latitude_delta_degrees,
        mean_distance_delta_au,
        max_longitude_delta_degrees,
        max_latitude_delta_degrees,
        max_distance_delta_au,
    }
}

/// Returns the current packaged-artifact fit envelope summary record.
pub fn packaged_artifact_fit_envelope_summary_details() -> PackagedArtifactFitEnvelopeSummary {
    static SUMMARY: OnceLock<PackagedArtifactFitEnvelopeSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let artifact = packaged_artifact();
            let samples = packaged_artifact_fit_samples_for_current_artifact();
            packaged_artifact_fit_envelope_summary_from_samples(
                samples,
                packaged_artifact_fit_expected_sample_count(artifact),
            )
        })
        .clone()
}

/// Returns the current packaged-artifact fit envelope after validating the structured posture.
pub fn packaged_artifact_fit_envelope_summary_for_report() -> String {
    let summary = packaged_artifact_fit_envelope_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("fit envelope: unavailable ({error})"),
    }
}

fn packaged_artifact_fit_channel_rank(channel: ChannelKind) -> usize {
    match channel {
        ChannelKind::DistanceAu => 0,
        ChannelKind::Longitude => 1,
        ChannelKind::Latitude => 2,
        _ => unreachable!("unsupported packaged-artifact channel kind"),
    }
}

fn packaged_artifact_fit_channel_delta(
    sample: &PackagedArtifactFitSample,
    channel: ChannelKind,
) -> f64 {
    match channel {
        ChannelKind::Longitude => sample.longitude_delta_degrees,
        ChannelKind::Latitude => sample.latitude_delta_degrees,
        ChannelKind::DistanceAu => sample.distance_delta_au,
        _ => unreachable!("unsupported packaged-artifact channel kind"),
    }
}

fn packaged_artifact_fit_outlier_summary_from_samples(
    samples: &[PackagedArtifactFitSample],
) -> PackagedArtifactFitOutlierSummary {
    let mut families: HashMap<
        (
            CelestialBody,
            ChannelKind,
            PackagedArtifactFitSegmentFamilyKey,
        ),
        PackagedArtifactFitChannelFamilyAccumulator,
    > = HashMap::new();

    for sample in samples {
        let family_key = PackagedArtifactFitSegmentFamilyKey::from_sample(sample);

        for channel in [
            ChannelKind::DistanceAu,
            ChannelKind::Longitude,
            ChannelKind::Latitude,
        ] {
            let entry = families
                .entry((sample.body.clone(), channel, family_key))
                .or_insert_with(PackagedArtifactFitChannelFamilyAccumulator::new);
            entry.push(sample, channel);
        }
    }

    let mut body_channel_outliers: HashMap<
        CelestialBody,
        [Option<PackagedArtifactFitChannelOutlier>; 3],
    > = HashMap::new();

    for ((body, channel, _family_key), family) in families {
        let Some(outlier) = family.finish(channel) else {
            continue;
        };
        let entry = body_channel_outliers
            .entry(body)
            .or_insert_with(|| [None, None, None]);
        let channel_index = packaged_artifact_fit_channel_rank(channel);
        let should_replace = entry[channel_index]
            .as_ref()
            .map(|existing| {
                outlier.delta > existing.delta
                    || (outlier.delta == existing.delta
                        && outlier.segment_span_days < existing.segment_span_days)
            })
            .unwrap_or(true);

        if should_replace {
            entry[channel_index] = Some(outlier);
        }
    }

    let mut body_summaries = body_channel_outliers
        .into_iter()
        .map(|(body, outliers)| {
            let mut channel_outliers = Vec::new();
            for channel in [
                ChannelKind::DistanceAu,
                ChannelKind::Longitude,
                ChannelKind::Latitude,
            ] {
                if let Some(outlier) = outliers[packaged_artifact_fit_channel_rank(channel)].clone()
                {
                    channel_outliers.push(outlier);
                }
            }
            PackagedArtifactFitBodyOutlierSummary {
                body,
                channel_outliers,
            }
        })
        .collect::<Vec<_>>();

    body_summaries.sort_by_key(|summary| summary.body.to_string());

    PackagedArtifactFitOutlierSummary {
        body_count: body_summaries.len(),
        body_summaries,
    }
}

/// Returns the current packaged-artifact body/channel fit outlier summary record.
pub fn packaged_artifact_fit_outlier_summary_details() -> PackagedArtifactFitOutlierSummary {
    static SUMMARY: OnceLock<PackagedArtifactFitOutlierSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let samples = packaged_artifact_fit_outlier_samples_for_current_artifact();
            packaged_artifact_fit_outlier_summary_from_samples(samples)
        })
        .clone()
}

/// Returns the current packaged-artifact body/channel fit outlier summary after validating the structured posture.
pub fn packaged_artifact_fit_outlier_summary_for_report() -> String {
    let summary = packaged_artifact_fit_outlier_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit outliers: unavailable ({error})"),
    }
}

const PACKAGED_ARTIFACT_FIT_MAX_MEAN_LONGITUDE_DELTA_DEGREES: f64 = 39.066737306976;
const PACKAGED_ARTIFACT_FIT_MAX_MEAN_LATITUDE_DELTA_DEGREES: f64 = 54.258413456361;
const PACKAGED_ARTIFACT_FIT_MAX_MEAN_DISTANCE_DELTA_AU: f64 = 167_525.454_245_761_94;
const PACKAGED_ARTIFACT_FIT_MAX_LONGITUDE_DELTA_DEGREES: f64 = 179.935747101401;
const PACKAGED_ARTIFACT_FIT_MAX_LATITUDE_DELTA_DEGREES: f64 = 5436.377507814662;
const PACKAGED_ARTIFACT_FIT_MAX_DISTANCE_DELTA_AU: f64 = 67_056_450.790_259_87;

const PACKAGED_ARTIFACT_FIT_THRESHOLD_SUMMARY: PackagedArtifactFitThresholdSummary =
    PackagedArtifactFitThresholdSummary {
        max_mean_longitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_MEAN_LONGITUDE_DELTA_DEGREES,
        max_mean_latitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_MEAN_LATITUDE_DELTA_DEGREES,
        max_mean_distance_delta_au: PACKAGED_ARTIFACT_FIT_MAX_MEAN_DISTANCE_DELTA_AU,
        max_longitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_LONGITUDE_DELTA_DEGREES,
        max_latitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_LATITUDE_DELTA_DEGREES,
        max_distance_delta_au: PACKAGED_ARTIFACT_FIT_MAX_DISTANCE_DELTA_AU,
    };

/// Returns the calibrated packaged-artifact fit threshold summary record.
pub fn packaged_artifact_fit_threshold_summary_details() -> PackagedArtifactFitThresholdSummary {
    PACKAGED_ARTIFACT_FIT_THRESHOLD_SUMMARY
}

/// Returns the current packaged-artifact fit thresholds after validating the structured posture.
pub fn packaged_artifact_fit_threshold_summary_for_report() -> String {
    let summary = packaged_artifact_fit_threshold_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit thresholds: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact fit margins relative to the calibrated thresholds.
pub fn packaged_artifact_fit_margin_summary_details() -> PackagedArtifactFitMarginSummary {
    let summary = PackagedArtifactFitMarginSummary {
        envelope: packaged_artifact_fit_envelope_summary_details(),
        thresholds: packaged_artifact_fit_threshold_summary_details(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact fit margins relative to the calibrated thresholds after validating the structured posture.
pub fn packaged_artifact_fit_margin_summary_for_report() -> String {
    let summary = packaged_artifact_fit_margin_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit margins: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact fit threshold violations summary record.
pub fn packaged_artifact_fit_threshold_violation_summary_details(
) -> PackagedArtifactFitThresholdViolationsSummary {
    let envelope = packaged_artifact_fit_envelope_summary_details();
    let thresholds = packaged_artifact_fit_threshold_summary_details();
    let violations = packaged_artifact_fit_threshold_violations_from_envelope_and_thresholds(
        &envelope,
        &thresholds,
    );

    PackagedArtifactFitThresholdViolationsSummary { violations }
}

/// Returns the number of packaged-artifact fit threshold violations relative to the calibrated thresholds.
pub fn packaged_artifact_fit_threshold_violation_count_for_report() -> String {
    let summary = packaged_artifact_fit_threshold_violation_summary_details();

    match summary.validate() {
        Ok(()) => format!("fit threshold violations: {}", summary.violations.len()),
        Err(error) => format!("fit threshold violations: unavailable ({error})"),
    }
}

/// Returns the packaged-artifact fit threshold violations with field-level context.
pub fn packaged_artifact_fit_threshold_violation_summary_for_report() -> String {
    let summary = packaged_artifact_fit_threshold_violation_summary_details();

    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit threshold violations: unavailable ({error})"),
    }
}

fn packaged_artifact_body_scope(body: &CelestialBody) -> &'static str {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => "luminaries",
        CelestialBody::Mercury
        | CelestialBody::Venus
        | CelestialBody::Mars
        | CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune => "major planets",
        CelestialBody::Pluto => "pluto",
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => "lunar points",
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => "selected asteroids",
        CelestialBody::Custom(custom) if custom.catalog.eq_ignore_ascii_case("asteroid") => {
            "selected asteroids"
        }
        CelestialBody::Custom(_) => "custom bodies",
        _ => "custom bodies",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PackagedArtifactBodyCadence {
    Luminaries,
    InnerPlanets,
    OuterPlanets,
    Pluto,
    LunarPoints,
    SelectedAsteroids,
    CustomBodies,
}

impl PackagedArtifactBodyCadence {
    pub(crate) fn uses_dense_sampling(self) -> bool {
        matches!(
            self,
            Self::Luminaries
                | Self::Pluto
                | Self::LunarPoints
                | Self::SelectedAsteroids
                | Self::CustomBodies
        )
    }

    pub(crate) fn uses_dense_validation_sampling(self) -> bool {
        matches!(
            self,
            Self::Luminaries
                | Self::InnerPlanets
                | Self::OuterPlanets
                | Self::Pluto
                | Self::LunarPoints
                | Self::SelectedAsteroids
                | Self::CustomBodies
        )
    }

    pub(crate) fn uses_dense_residual_sample_lattice(self, kind: ChannelKind) -> bool {
        match kind {
            ChannelKind::Longitude | ChannelKind::Latitude => self.uses_dense_sampling(),
            ChannelKind::DistanceAu => {
                matches!(
                    self,
                    Self::InnerPlanets
                        | Self::OuterPlanets
                        | Self::Pluto
                        | Self::LunarPoints
                        | Self::SelectedAsteroids
                        | Self::CustomBodies
                )
            }
            _ => false,
        }
    }
}

pub(crate) fn packaged_artifact_body_cadence(body: &CelestialBody) -> PackagedArtifactBodyCadence {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => PackagedArtifactBodyCadence::Luminaries,
        CelestialBody::Mercury | CelestialBody::Venus | CelestialBody::Mars => {
            PackagedArtifactBodyCadence::InnerPlanets
        }
        CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune => PackagedArtifactBodyCadence::OuterPlanets,
        CelestialBody::Pluto => PackagedArtifactBodyCadence::Pluto,
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => PackagedArtifactBodyCadence::LunarPoints,
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => PackagedArtifactBodyCadence::SelectedAsteroids,
        CelestialBody::Custom(custom) if custom.catalog.eq_ignore_ascii_case("asteroid") => {
            PackagedArtifactBodyCadence::SelectedAsteroids
        }
        CelestialBody::Custom(_) => PackagedArtifactBodyCadence::CustomBodies,
        _ => PackagedArtifactBodyCadence::CustomBodies,
    }
}

fn packaged_artifact_target_threshold_scope_envelope_summary_details(
    scope: &'static str,
) -> PackagedArtifactTargetThresholdScopeSummary {
    let artifact = packaged_artifact();
    let bodies: Vec<CelestialBody> = artifact
        .bodies
        .iter()
        .filter(|body| packaged_artifact_body_scope(&body.body) == scope)
        .map(|body| body.body.clone())
        .collect();
    let body_count = bodies.len();
    let samples = packaged_artifact_fit_samples_for_current_artifact()
        .iter()
        .filter(|sample| packaged_artifact_body_scope(&sample.body) == scope)
        .cloned()
        .collect::<Vec<_>>();
    let expected_sample_count =
        packaged_artifact_fit_expected_sample_count_with_filter(artifact, |body| {
            packaged_artifact_body_scope(body) == scope
        });
    let fit_envelope =
        packaged_artifact_fit_envelope_summary_from_samples(&samples, expected_sample_count);
    PackagedArtifactTargetThresholdScopeSummary {
        scope,
        bodies,
        body_count,
        fit_envelope,
    }
}

fn packaged_artifact_target_threshold_scope_envelope_summaries_details(
) -> Vec<PackagedArtifactTargetThresholdScopeSummary> {
    PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES
        .iter()
        .copied()
        .map(packaged_artifact_target_threshold_scope_envelope_summary_details)
        .collect()
}

/// Returns the current packaged-artifact body-class target-threshold scope envelopes after validating the structured posture.
pub fn packaged_artifact_target_threshold_scope_envelopes_summary_details(
) -> PackagedArtifactTargetThresholdScopeEnvelopesSummary {
    PackagedArtifactTargetThresholdScopeEnvelopesSummary {
        scope_envelopes: packaged_artifact_target_threshold_scope_envelope_summaries_details(),
    }
}

fn format_scope_bodies(bodies: &[CelestialBody]) -> String {
    match bodies {
        [] => "0 (none)".to_string(),
        [single] => format!("1 ({single})"),
        _ => format!("{} ({})", bodies.len(), join_display(bodies)),
    }
}

pub(crate) fn packaged_artifact_quantization_scales_line() -> String {
    let artifact = packaged_artifact();
    let stored = packaged_artifact_channel_quantization_scales(artifact, false);
    let residual = packaged_artifact_channel_quantization_scales(artifact, true);

    if residual.is_empty() {
        format!("quantization scales: stored={stored}")
    } else {
        format!("quantization scales: stored={stored}; residual={residual}")
    }
}

fn packaged_artifact_channel_quantization_scales(
    artifact: &CompressedArtifact,
    residual_channels: bool,
) -> String {
    let entries = [
        ChannelKind::Longitude,
        ChannelKind::Latitude,
        ChannelKind::DistanceAu,
    ]
    .into_iter()
    .filter_map(|kind| {
        let mut exponents = artifact
            .bodies
            .iter()
            .flat_map(|body| body.segments.iter())
            .flat_map(|segment| {
                let channels = if residual_channels {
                    &segment.residual_channels
                } else {
                    &segment.channels
                };

                channels
                    .iter()
                    .filter(move |channel| channel.kind == kind)
                    .map(|channel| channel.scale_exponent)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        if exponents.is_empty() {
            None
        } else {
            exponents.sort_unstable();
            exponents.dedup();
            Some(format!("{kind}={}", join_display(&exponents)))
        }
    })
    .collect::<Vec<_>>();

    if entries.is_empty() {
        "none".to_string()
    } else {
        entries.join(", ")
    }
}

pub(crate) fn packaged_artifact_segment_span_bounds(artifact: &CompressedArtifact) -> (f64, f64) {
    let mut min_span_days: f64 = f64::INFINITY;
    let mut max_span_days: f64 = 0.0;

    for body in &artifact.bodies {
        for segment in &body.segments {
            let span_days = segment.end.julian_day.days() - segment.start.julian_day.days();
            min_span_days = min_span_days.min(span_days);
            max_span_days = max_span_days.max(span_days);
        }
    }

    if min_span_days.is_infinite() {
        (0.0, 0.0)
    } else {
        (min_span_days, max_span_days)
    }
}

pub(crate) fn packaged_artifact_channel_count(
    artifact: &CompressedArtifact,
    residual_channels: bool,
) -> usize {
    artifact
        .bodies
        .iter()
        .flat_map(|body| body.segments.iter())
        .map(|segment| {
            if residual_channels {
                segment.residual_channels.len()
            } else {
                segment.channels.len()
            }
        })
        .sum()
}

fn packaged_artifact_encoded_bytes(artifact: &CompressedArtifact) -> usize {
    artifact
        .encode()
        .expect("packaged artifact should be encodable")
        .len()
}

/// Structured normalized-intermediate provenance for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactNormalizedIntermediateSummary {
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Human-readable provenance/source summary.
    pub source: &'static str,
    /// Canonical source-revision summary for the checked-in production-generation corpus.
    pub source_revision: String,
    /// Stable identifier for the packaged-artifact profile.
    pub profile_id: &'static str,
    /// Covered time range for the normalized intermediate layout.
    pub time_range: TimeRange,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Per-channel quantization scales captured from the checked-in artifact.
    pub quantization_scales: String,
    /// Deterministic checksum of the rendered normalized-intermediate payload.
    pub checksum: u64,
    /// Bodies bundled into the packaged artifact.
    pub body_count: usize,
    /// Total segment count across all bundled bodies.
    pub segment_count: usize,
    /// Total count of segments carrying residual correction channels.
    pub residual_segment_count: usize,
    /// Total count of stored channels across all segments.
    pub stored_channel_count: usize,
    /// Total count of residual channels across all segments.
    pub residual_channel_count: usize,
    /// Smallest observed segment span in days.
    pub min_segment_span_days: f64,
    /// Largest observed segment span in days.
    pub max_segment_span_days: f64,
}

impl PackagedArtifactNormalizedIntermediateSummary {
    /// Returns the normalized-intermediate payload used for checksuming and rendering.
    pub(crate) fn summary_payload_line(&self) -> String {
        format!(
            "label={}; profile id={}; version={}; time range={}; source={}; source revision={}; body count={}; segments={}; residual-bearing segments={}; stored channels={}; residual channels={}; segment span days={:.12}..{:.12}; segment strategy={}; {}",
            self.label,
            self.profile_id,
            self.artifact_version,
            self.time_range,
            self.source,
            self.source_revision,
            self.body_count,
            self.segment_count,
            self.residual_segment_count,
            self.stored_channel_count,
            self.residual_channel_count,
            self.min_segment_span_days,
            self.max_segment_span_days,
            self.generation_policy.segment_strategy(),
            self.quantization_scales,
        )
    }

    /// Returns the normalized intermediates as a compact human-readable line.
    pub fn summary_fields_line(&self) -> String {
        format!(
            "{}; checksum=0x{:016x}",
            self.summary_payload_line(),
            self.checksum,
        )
    }

    /// Returns the normalized intermediates as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact normalized intermediates: {}",
            self.summary_fields_line()
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let artifact = packaged_artifact();
        if self.label != ARTIFACT_LABEL {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary label does not match the checked-in artifact label",
            ));
        }
        if self.artifact_version != artifact.header.version {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary artifact version does not match the checked-in packaged artifact version",
            ));
        }
        if self.source != packaged_artifact_source_text() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary source does not match the checked-in artifact source",
            ));
        }
        if self.source_revision != production_generation_source_summary_for_report() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary source revision does not match the checked-in production-generation source summary",
            ));
        }
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary profile id does not match the checked-in artifact profile id",
            ));
        }
        if self.time_range != artifact_time_range(artifact) {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary time range does not match the checked-in packaged artifact",
            ));
        }
        if self.generation_policy
            != PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows
        {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary generation policy does not match the checked-in packaged artifact",
            ));
        }
        if self.quantization_scales != packaged_artifact_quantization_scales_line() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary quantization scales do not match the checked-in packaged artifact",
            ));
        }
        if self.body_count != artifact.bodies.len() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary body count does not match the checked-in packaged artifact",
            ));
        }
        if self.segment_count != artifact.segment_count() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary segment count does not match the checked-in packaged artifact",
            ));
        }
        if self.residual_segment_count != artifact.residual_segment_count() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary residual segment count does not match the checked-in packaged artifact",
            ));
        }
        if self.stored_channel_count != packaged_artifact_channel_count(artifact, false) {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary stored channel count does not match the checked-in packaged artifact",
            ));
        }
        if self.residual_channel_count != packaged_artifact_channel_count(artifact, true) {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary residual channel count does not match the checked-in packaged artifact",
            ));
        }
        let (expected_min_segment_span_days, expected_max_segment_span_days) =
            packaged_artifact_segment_span_bounds(artifact);
        if self.min_segment_span_days != expected_min_segment_span_days {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary minimum segment span does not match the checked-in packaged artifact",
            ));
        }
        if self.max_segment_span_days != expected_max_segment_span_days {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary maximum segment span does not match the checked-in packaged artifact",
            ));
        }

        let expected_checksum = fnv1a64(self.summary_payload_line().as_bytes());
        if self.checksum != expected_checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact normalized intermediate summary checksum 0x{:016x} does not match the current normalized-intermediate checksum 0x{:016x}",
                    self.checksum,
                    expected_checksum
                ),
            ));
        }

        Ok(())
    }

    /// Returns the validated normalized intermediates as a compact human-readable line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactNormalizedIntermediateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured regeneration provenance for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactRegenerationSummary {
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Human-readable provenance/source summary.
    pub source: &'static str,
    /// Canonical source-revision summary for the checked-in production-generation corpus.
    pub source_revision: String,
    /// Stable identifier for the packaged-artifact profile.
    pub profile_id: &'static str,
    /// Checksum of the checked-in packaged artifact.
    pub checksum: u64,
    /// Encoded size of the checked-in packaged artifact in bytes.
    pub artifact_size_bytes: usize,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Per-channel quantization scales captured from the checked-in artifact.
    pub quantization_scales: String,
    /// Bodies that carry residual correction channels in the packaged artifact.
    pub residual_bodies: Vec<CelestialBody>,
    /// Bodies bundled into the packaged artifact.
    pub bodies: Vec<CelestialBody>,
    /// Normalized intermediate layout captured from the checked-in artifact.
    pub normalized_intermediates: PackagedArtifactNormalizedIntermediateSummary,
    /// Fit envelope measured against the generation source samples.
    pub fit_envelope: PackagedArtifactFitEnvelopeSummary,
    /// Coverage summary for the checked-in JPL reference snapshot used for regeneration.
    pub reference_snapshot: Option<ReferenceSnapshotSummary>,
}

impl PackagedArtifactRegenerationSummary {
    /// Returns the bundled bodies as a compact human-readable line.
    pub fn body_coverage_line(&self) -> String {
        format!(
            "{} bundled bodies ({})",
            self.bodies.len(),
            join_display(&self.bodies)
        )
    }

    /// Returns the checked-in JPL snapshot coverage as a compact human-readable line.
    pub fn reference_snapshot_line(&self) -> String {
        self.reference_snapshot
            .map(|summary| format_reference_snapshot_summary(&summary))
            .unwrap_or_else(|| "Reference snapshot coverage: unavailable".to_string())
    }

    /// Returns the normalized intermediate layout as a compact human-readable line.
    pub fn normalized_intermediates_line(&self) -> String {
        self.normalized_intermediates.summary_fields_line()
    }

    /// Returns the residual-correction body coverage as a compact structured summary.
    pub fn residual_body_coverage_summary(&self) -> ArtifactResidualBodyCoverageSummary {
        ArtifactResidualBodyCoverageSummary::new(self.residual_bodies.clone())
    }

    /// Returns the residual-correction body list as a compact human-readable line.
    pub fn residual_body_line(&self) -> String {
        self.residual_body_coverage_summary()
            .summary_line_with_body_count()
    }

    /// Returns the generation policy as a compact human-readable line.
    pub fn generation_policy_line(&self) -> String {
        format!(
            "generation policy: {}",
            self.generation_policy.summary_line()
        )
    }

    /// Validates that every residual body is actually part of the bundled body list.
    pub(crate) fn validate_residual_body_subset(
        &self,
    ) -> Result<(), pleiades_compression::CompressionError> {
        for body in &self.residual_bodies {
            if !self.bodies.contains(body) {
                return Err(pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!(
                        "packaged artifact regeneration summary residual body {body} is not covered by the bundled body list"
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Validates that the regeneration summary stays aligned with the bundled
    /// body list, the current checked-in artifact metadata, and the checked-in
    /// reference snapshot coverage.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let artifact = packaged_artifact();

        if self.label != ARTIFACT_LABEL {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary label does not match the checked-in artifact label",
            ));
        }
        if self.source != packaged_artifact_source_text() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary source does not match the checked-in artifact source",
            ));
        }
        if self.source_revision != production_generation_source_summary_for_report() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary source revision does not match the checked-in production-generation source summary",
            ));
        }
        self.normalized_intermediates.validate().map_err(|error| {
            pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary normalized intermediates are invalid: {error}"
                ),
            )
        })?;
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary profile id does not match the checked-in artifact profile id",
            ));
        }
        if self.artifact_version != artifact.header.version {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary artifact version {} does not match the checked-in packaged artifact version {}",
                    self.artifact_version,
                    artifact.header.version
                ),
            ));
        }
        if self.checksum != artifact.checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary checksum 0x{:016x} does not match the checked-in packaged artifact checksum 0x{:016x}",
                    self.checksum,
                    artifact.checksum
                ),
            ));
        }
        let expected_artifact_size_bytes = packaged_artifact_encoded_bytes(artifact);
        if self.artifact_size_bytes != expected_artifact_size_bytes {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary artifact size {} bytes does not match the checked-in packaged artifact size {} bytes",
                    self.artifact_size_bytes,
                    expected_artifact_size_bytes
                ),
            ));
        }
        self.generation_policy.validate().map_err(|error| {
            pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary generation policy is invalid: {error}"
                ),
            )
        })?;

        if self.quantization_scales != packaged_artifact_quantization_scales_line() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary quantization scales do not match the checked-in packaged artifact",
            ));
        }

        self.residual_body_coverage_summary()
            .validate(artifact)
            .map_err(|error| {
                pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!(
                        "packaged artifact regeneration summary residual body coverage is invalid: {error}"
                    ),
                )
            })?;

        if self.reference_snapshot.is_none() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary is missing reference snapshot coverage",
            ));
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!("packaged artifact regeneration summary contains duplicate body entry {body}"),
                ));
            }
        }

        let expected_bodies = packaged_bodies();
        if self.bodies.as_slice() != expected_bodies {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary body list does not match the checked-in packaged body set: expected [{}]; got [{}]",
                    join_display(expected_bodies),
                    join_display(&self.bodies)
                ),
            ));
        }

        self.validate_residual_body_subset()?;

        self.fit_envelope.validate().map_err(|error| {
            pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!("packaged artifact regeneration fit envelope is invalid: {error}"),
            )
        })?;

        if let Some(reference_snapshot) = self.reference_snapshot {
            reference_snapshot.validate().map_err(|error| {
                pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!(
                        "packaged artifact regeneration reference snapshot is invalid: {error}"
                    ),
                )
            })?;
            for body in &self.bodies {
                if !reference_snapshot.bodies.contains(body) {
                    return Err(pleiades_compression::CompressionError::new(
                        pleiades_compression::CompressionErrorKind::InvalidFormat,
                        format!("packaged artifact regeneration body {body} is not covered by the reference snapshot"),
                    ));
                }
            }
            if self.reference_snapshot != reference_snapshot_summary() {
                return Err(pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    "packaged artifact regeneration summary reference snapshot does not match the checked-in reference snapshot summary",
                ));
            }
        }

        Ok(())
    }

    /// Returns the full packaged-artifact regeneration provenance summary.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact regeneration source: label={}; profile id={}; source={}; source revision={}; normalized intermediates: {}; checksum=0x{:016x}; artifact size={} bytes; {}; segment strategy={}; {}; {}; bundled bodies: {}; {}; fit envelope: {}; artifact version={}",
            self.label,
            self.profile_id,
            self.source,
            self.source_revision,
            self.normalized_intermediates_line(),
            self.checksum,
            self.artifact_size_bytes,
            self.generation_policy_line(),
            self.generation_policy.segment_strategy(),
            self.quantization_scales,
            self.residual_body_line(),
            self.body_coverage_line(),
            self.reference_snapshot_line(),
            self.fit_envelope.summary_line(),
            self.artifact_version,
        )
    }

    /// Returns the full packaged-artifact regeneration provenance summary after validating the structured posture.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactRegenerationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured packaged-artifact regeneration provenance.
pub fn packaged_artifact_regeneration_summary_details() -> PackagedArtifactRegenerationSummary {
    static SUMMARY: OnceLock<PackagedArtifactRegenerationSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let artifact = packaged_artifact();
            let summary = PackagedArtifactRegenerationSummary {
                label: ARTIFACT_LABEL,
                artifact_version: artifact.header.version,
                source: packaged_artifact_source_text(),
                source_revision: production_generation_source_summary_for_report(),
                profile_id: ARTIFACT_PROFILE_ID,
                checksum: artifact.checksum,
                artifact_size_bytes: packaged_artifact_encoded_bytes(artifact),
                generation_policy:
                    PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
                quantization_scales: packaged_artifact_quantization_scales_line(),
                residual_bodies: artifact.residual_bodies(),
                bodies: packaged_bodies().to_vec(),
                normalized_intermediates: packaged_artifact_normalized_intermediate_summary_details(
                ),
                fit_envelope: packaged_artifact_fit_envelope_summary_details(),
                reference_snapshot: reference_snapshot_summary(),
            };
            debug_assert!(summary.validate().is_ok());
            summary
        })
        .clone()
}

/// Returns the packaged-artifact regeneration provenance summary.
pub fn packaged_artifact_regeneration_summary() -> String {
    packaged_artifact_regeneration_summary_for_report()
}

/// Returns the packaged-artifact regeneration provenance summary after validating
/// the structured source and coverage metadata.
pub fn packaged_artifact_regeneration_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_regeneration_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => {
                    format!("Packaged artifact regeneration source: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the structured normalized-intermediate provenance.
pub fn packaged_artifact_normalized_intermediate_summary_details(
) -> PackagedArtifactNormalizedIntermediateSummary {
    let artifact = packaged_artifact();
    let payload_checksum = fnv1a64(
        format!(
            "label={}; profile id={}; version={}; time range={}; source={}; source revision={}; body count={}; segments={}; residual-bearing segments={}; stored channels={}; residual channels={}; segment span days={:.12}..{:.12}; segment strategy={}; {}",
            ARTIFACT_LABEL,
            ARTIFACT_PROFILE_ID,
            artifact.header.version,
            artifact_time_range(artifact),
            packaged_artifact_source_text(),
            production_generation_source_summary_for_report(),
            artifact.bodies.len(),
            artifact.segment_count(),
            artifact.residual_segment_count(),
            packaged_artifact_channel_count(artifact, false),
            packaged_artifact_channel_count(artifact, true),
            packaged_artifact_segment_span_bounds(artifact).0,
            packaged_artifact_segment_span_bounds(artifact).1,
            PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows.segment_strategy(),
            packaged_artifact_quantization_scales_line(),
        )
        .as_bytes(),
    );
    let summary = PackagedArtifactNormalizedIntermediateSummary {
        label: ARTIFACT_LABEL,
        artifact_version: artifact.header.version,
        source: packaged_artifact_source_text(),
        source_revision: production_generation_source_summary_for_report(),
        profile_id: ARTIFACT_PROFILE_ID,
        time_range: artifact_time_range(artifact),
        generation_policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
        quantization_scales: packaged_artifact_quantization_scales_line(),
        checksum: payload_checksum,
        body_count: artifact.bodies.len(),
        segment_count: artifact.segment_count(),
        residual_segment_count: artifact.residual_segment_count(),
        stored_channel_count: packaged_artifact_channel_count(artifact, false),
        residual_channel_count: packaged_artifact_channel_count(artifact, true),
        min_segment_span_days: packaged_artifact_segment_span_bounds(artifact).0,
        max_segment_span_days: packaged_artifact_segment_span_bounds(artifact).1,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the normalized-intermediate provenance summary.
pub fn packaged_artifact_normalized_intermediate_summary_for_report() -> String {
    let summary = packaged_artifact_normalized_intermediate_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact normalized intermediates: unavailable ({error})"),
    }
}

/// Release-state for the packaged-artifact target thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedArtifactTargetThresholdState {
    /// Calibrated fit envelope is recorded, but production thresholds are not yet release-ready.
    Draft,
    /// Production thresholds have been finalized for the current profile.
    ProductionReady,
}

/// Validation error for a packaged-artifact target-threshold state that is not release-ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactTargetThresholdStateValidationError {
    /// The target-threshold state is still draft.
    Draft,
}

impl fmt::Display for PackagedArtifactTargetThresholdStateValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(
                f,
                "the packaged-artifact target-threshold state is draft; production thresholds are not yet release-ready"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactTargetThresholdStateValidationError {}

impl PackagedArtifactTargetThresholdState {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draft => {
                "calibrated fit envelope recorded; production thresholds not yet release-ready"
            }
            Self::ProductionReady => "production thresholds recorded",
        }
    }

    /// Returns a compact human-readable line for the target-threshold state.
    pub fn summary_line(self) -> String {
        format!("target-threshold state: {}", self)
    }

    /// Returns whether the target thresholds are finalized for production release.
    pub const fn is_production_ready(self) -> bool {
        matches!(self, Self::ProductionReady)
    }

    /// Returns `Ok(())` when the state is release-ready.
    pub fn validate_production_ready(
        self,
    ) -> Result<(), PackagedArtifactTargetThresholdStateValidationError> {
        if self.is_production_ready() {
            Ok(())
        } else {
            Err(PackagedArtifactTargetThresholdStateValidationError::Draft)
        }
    }

    /// Returns the validated target-threshold state as a compact human-readable line.
    pub fn validated_summary_line(
        self,
    ) -> Result<String, PackagedArtifactTargetThresholdStateValidationError> {
        self.validate_production_ready()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactTargetThresholdState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Returns the current packaged-artifact target-threshold state after validating the release posture.
pub fn packaged_artifact_target_threshold_state_for_report() -> String {
    match PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("target-threshold state: unavailable ({error})"),
    }
}

const PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE: PackagedArtifactTargetThresholdState =
    PackagedArtifactTargetThresholdState::ProductionReady;
pub(crate) const PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES: &[&str] = &[
    "luminaries",
    "major planets",
    "pluto",
    "lunar points",
    "selected asteroids",
    "custom bodies",
];

/// Phase-2 corpus evidence used to keep the packaged-artifact threshold policy aligned
/// with the current reference, fixture-exactness, comparison, hold-out, selected-asteroid, boundary-overlay, and production-generation corpora.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactPhase2CorpusAlignmentSummary {
    /// Source-material evidence from the checked-in reference snapshot.
    pub reference_snapshot_source: pleiades_jpl::ReferenceSnapshotSourceSummary,
    /// Body-class coverage evidence from the checked-in reference snapshot.
    pub reference_snapshot: pleiades_jpl::ReferenceSnapshotBodyClassCoverageSummary,
    /// Exact J2000 fixture-exactness evidence from the checked-in reference snapshot.
    pub reference_snapshot_exact_j2000: pleiades_jpl::ReferenceSnapshotExactJ2000EvidenceSummary,
    /// Source-material evidence from the checked-in comparison snapshot.
    pub comparison_snapshot_source: pleiades_jpl::ComparisonSnapshotSourceSummary,
    /// Body-class coverage evidence from the checked-in comparison snapshot.
    pub comparison_snapshot: pleiades_jpl::ComparisonSnapshotBodyClassCoverageSummary,
    /// Source-material evidence from the checked-in independent hold-out snapshot.
    pub independent_holdout_source: pleiades_jpl::IndependentHoldoutSourceSummary,
    /// Body-class coverage evidence from the checked-in independent hold-out snapshot.
    pub independent_holdout: pleiades_jpl::IndependentHoldoutSnapshotBodyClassCoverageSummary,
    /// Source-backed evidence for the selected-asteroid validation corpus.
    pub selected_asteroid_source: pleiades_jpl::SelectedAsteroidSourceSummary,
    /// Source-backed window evidence for the selected-asteroid validation corpus.
    pub selected_asteroid_source_windows: pleiades_jpl::SelectedAsteroidSourceWindowSummary,
    /// Checked-in request-corpus evidence for the selected-asteroid validation corpus in the ecliptic frame.
    pub selected_asteroid_source_request_corpus:
        pleiades_jpl::SelectedAsteroidSourceRequestCorpusSummary,
    /// Checked-in request-corpus evidence for the selected-asteroid validation corpus in the equatorial frame.
    pub selected_asteroid_source_request_corpus_equatorial:
        pleiades_jpl::SelectedAsteroidSourceRequestCorpusSummary,
    /// Source-material evidence for the checked-in production-generation boundary overlay.
    pub production_generation_boundary_source: pleiades_jpl::IndependentHoldoutSourceSummary,
    /// Body-class coverage evidence for the checked-in production-generation corpus.
    pub production_generation_body_class_coverage:
        pleiades_jpl::ProductionGenerationSnapshotBodyClassCoverageSummary,
    /// Combined provenance for the checked-in production-generation corpus.
    pub production_generation_source: ProductionGenerationSourceSummary,
}

/// Validation error for a phase-2 corpus alignment summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactPhase2CorpusAlignmentSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactPhase2CorpusAlignmentSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact phase-2 corpus alignment summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactPhase2CorpusAlignmentSummaryValidationError {}

fn phase2_corpus_alignment_validation_field_path(field: &'static str) -> &'static str {
    match field {
        "reference_snapshot_source" => "phase2_corpus_alignment.reference_snapshot_source",
        "reference_snapshot" => "phase2_corpus_alignment.reference_snapshot",
        "reference_snapshot_exact_j2000" => {
            "phase2_corpus_alignment.reference_snapshot_exact_j2000"
        }
        "comparison_snapshot_source" => "phase2_corpus_alignment.comparison_snapshot_source",
        "comparison_snapshot" => "phase2_corpus_alignment.comparison_snapshot",
        "independent_holdout_source" => "phase2_corpus_alignment.independent_holdout_source",
        "independent_holdout" => "phase2_corpus_alignment.independent_holdout",
        "selected_asteroid_source" => "phase2_corpus_alignment.selected_asteroid_source",
        "selected_asteroid_source_windows" => {
            "phase2_corpus_alignment.selected_asteroid_source_windows"
        }
        "selected_asteroid_source_request_corpus" => {
            "phase2_corpus_alignment.selected_asteroid_source_request_corpus"
        }
        "selected_asteroid_source_request_corpus_equatorial" => {
            "phase2_corpus_alignment.selected_asteroid_source_request_corpus_equatorial"
        }
        "production_generation_boundary_source" => {
            "phase2_corpus_alignment.production_generation_boundary_source"
        }
        "production_generation_body_class_coverage" => {
            "phase2_corpus_alignment.production_generation_body_class_coverage"
        }
        "production_generation_source" => "phase2_corpus_alignment.production_generation_source",
        _ => "phase2_corpus_alignment",
    }
}

impl PackagedArtifactPhase2CorpusAlignmentSummary {
    /// Returns the phase-2 corpus alignment posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "reference source={}; reference snapshot={}; reference exact J2000 evidence={}; comparison source={}; comparison snapshot={}; independent hold-out source={}; independent hold-out={}; selected asteroid source evidence={}; selected asteroid source windows={}; selected asteroid source request corpus={}; selected asteroid source request corpus equatorial={}; production generation boundary source={}; production generation body-class coverage={}; production generation source={}",
            self.reference_snapshot_source.summary_line(),
            self.reference_snapshot.summary_line(),
            self.reference_snapshot_exact_j2000.summary_line(),
            self.comparison_snapshot_source.summary_line(),
            self.comparison_snapshot.summary_line(),
            self.independent_holdout_source.summary_line(),
            self.independent_holdout.summary_line(),
            self.selected_asteroid_source.summary_line(),
            self.selected_asteroid_source_windows.summary_line(),
            self.selected_asteroid_source_request_corpus.summary_line(),
            self.selected_asteroid_source_request_corpus_equatorial.summary_line(),
            pleiades_jpl::format_production_generation_boundary_source_summary(
                &self.production_generation_boundary_source,
            ),
            self.production_generation_body_class_coverage.summary_line(),
            self.production_generation_source.summary_line(),
        )
    }

    /// Returns `Ok(())` when the phase-2 corpus evidence still matches the current packaged-artifact posture.
    pub fn validate(
        &self,
    ) -> Result<(), PackagedArtifactPhase2CorpusAlignmentSummaryValidationError> {
        let Some(expected) = packaged_artifact_phase2_corpus_alignment_summary_details() else {
            return Err(
                PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            );
        };

        let field_out_of_sync = |field| {
            PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync { field }
        };

        if self.reference_snapshot_source != expected.reference_snapshot_source {
            return Err(field_out_of_sync("reference_snapshot_source"));
        }
        self.reference_snapshot_source
            .validate()
            .map_err(|_| field_out_of_sync("reference_snapshot_source"))?;

        if self.reference_snapshot != expected.reference_snapshot {
            return Err(field_out_of_sync("reference_snapshot"));
        }
        self.reference_snapshot
            .validate()
            .map_err(|_| field_out_of_sync("reference_snapshot"))?;

        if self.reference_snapshot_exact_j2000 != expected.reference_snapshot_exact_j2000 {
            return Err(field_out_of_sync("reference_snapshot_exact_j2000"));
        }
        self.reference_snapshot_exact_j2000
            .validate()
            .map_err(|_| field_out_of_sync("reference_snapshot_exact_j2000"))?;

        if self.comparison_snapshot_source != expected.comparison_snapshot_source {
            return Err(field_out_of_sync("comparison_snapshot_source"));
        }
        self.comparison_snapshot_source
            .validate()
            .map_err(|_| field_out_of_sync("comparison_snapshot_source"))?;

        if self.comparison_snapshot != expected.comparison_snapshot {
            return Err(field_out_of_sync("comparison_snapshot"));
        }
        self.comparison_snapshot
            .validate()
            .map_err(|_| field_out_of_sync("comparison_snapshot"))?;

        if self.independent_holdout_source != expected.independent_holdout_source {
            return Err(field_out_of_sync("independent_holdout_source"));
        }
        self.independent_holdout_source
            .validate()
            .map_err(|_| field_out_of_sync("independent_holdout_source"))?;

        if self.independent_holdout != expected.independent_holdout {
            return Err(field_out_of_sync("independent_holdout"));
        }
        self.independent_holdout
            .validate()
            .map_err(|_| field_out_of_sync("independent_holdout"))?;

        if self.selected_asteroid_source != expected.selected_asteroid_source {
            return Err(field_out_of_sync("selected_asteroid_source"));
        }
        self.selected_asteroid_source
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source"))?;

        if self.selected_asteroid_source_windows != expected.selected_asteroid_source_windows {
            return Err(field_out_of_sync("selected_asteroid_source_windows"));
        }
        self.selected_asteroid_source_windows
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source_windows"))?;

        if self.selected_asteroid_source_request_corpus
            != expected.selected_asteroid_source_request_corpus
        {
            return Err(field_out_of_sync("selected_asteroid_source_request_corpus"));
        }
        self.selected_asteroid_source_request_corpus
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source_request_corpus"))?;

        if self.selected_asteroid_source_request_corpus_equatorial
            != expected.selected_asteroid_source_request_corpus_equatorial
        {
            return Err(field_out_of_sync(
                "selected_asteroid_source_request_corpus_equatorial",
            ));
        }
        self.selected_asteroid_source_request_corpus_equatorial
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source_request_corpus_equatorial"))?;

        if self.production_generation_boundary_source
            != expected.production_generation_boundary_source
        {
            return Err(field_out_of_sync("production_generation_boundary_source"));
        }
        self.production_generation_boundary_source
            .validate()
            .map_err(|_| field_out_of_sync("production_generation_boundary_source"))?;

        if self.production_generation_body_class_coverage
            != expected.production_generation_body_class_coverage
        {
            return Err(field_out_of_sync(
                "production_generation_body_class_coverage",
            ));
        }
        self.production_generation_body_class_coverage
            .validate()
            .map_err(|_| field_out_of_sync("production_generation_body_class_coverage"))?;

        if self.production_generation_source != expected.production_generation_source {
            return Err(field_out_of_sync("production_generation_source"));
        }
        self.production_generation_source
            .validate()
            .map_err(|_| field_out_of_sync("production_generation_source"))?;

        Ok(())
    }

    /// Returns the validated phase-2 corpus alignment posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactPhase2CorpusAlignmentSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactPhase2CorpusAlignmentSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured phase-2 corpus alignment posture used to keep the packaged-artifact threshold policy aligned with the current corpus.
pub fn packaged_artifact_phase2_corpus_alignment_summary_details(
) -> Option<PackagedArtifactPhase2CorpusAlignmentSummary> {
    Some(PackagedArtifactPhase2CorpusAlignmentSummary {
        reference_snapshot_source: pleiades_jpl::reference_snapshot_source_summary(),
        reference_snapshot: pleiades_jpl::reference_snapshot_body_class_coverage_summary()?,
        reference_snapshot_exact_j2000:
            pleiades_jpl::reference_snapshot_exact_j2000_evidence_summary()?,
        comparison_snapshot_source: pleiades_jpl::comparison_snapshot_source_summary(),
        comparison_snapshot: comparison_snapshot_body_class_coverage_summary()?,
        independent_holdout_source: pleiades_jpl::independent_holdout_source_summary(),
        independent_holdout: independent_holdout_snapshot_body_class_coverage_summary()?,
        selected_asteroid_source: pleiades_jpl::selected_asteroid_source_evidence_summary()?,
        selected_asteroid_source_windows: pleiades_jpl::selected_asteroid_source_window_summary()?,
        selected_asteroid_source_request_corpus: selected_asteroid_source_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )?,
        selected_asteroid_source_request_corpus_equatorial:
            selected_asteroid_source_request_corpus_summary(CoordinateFrame::Equatorial)?,
        production_generation_boundary_source:
            pleiades_jpl::production_generation_boundary_source_summary(),
        production_generation_body_class_coverage:
            pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()?,
        production_generation_source: production_generation_source_summary(),
    })
}

/// Returns the current packaged-artifact phase-2 corpus alignment posture after validating the structured evidence.
pub fn packaged_artifact_phase2_corpus_alignment_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_phase2_corpus_alignment_summary_details();
            match summary.as_ref().map(PackagedArtifactPhase2CorpusAlignmentSummary::validated_summary_line) {
                Some(Ok(line)) => line,
                Some(Err(error)) => format!("phase 2 corpus alignment: unavailable ({error})"),
                None => "phase 2 corpus alignment: unavailable (phase-2 corpus evidence should be available)".to_string(),
            }
        })
        .clone()
}

/// Structured target-threshold posture for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactTargetThresholdSummary {
    /// Stable identifier for the release profile that the thresholds apply to.
    pub profile_id: &'static str,
    /// Current release posture for the production thresholds.
    pub state: PackagedArtifactTargetThresholdState,
    /// Body-class scopes covered by the current threshold policy.
    pub scopes: &'static [&'static str],
    /// Measured fit envelope captured for the current packaged artifact posture.
    pub fit_envelope: PackagedArtifactFitEnvelopeSummary,
    /// Body-class-specific fit envelopes captured for the current packaged artifact posture.
    pub scope_envelopes: PackagedArtifactTargetThresholdScopeEnvelopesSummary,
    /// Phase-2 corpus evidence that keeps the threshold posture aligned with the current corpus.
    pub phase2_corpus_alignment: PackagedArtifactPhase2CorpusAlignmentSummary,
}

/// Validation error for a packaged-artifact target-threshold summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactTargetThresholdSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactTargetThresholdSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact target-threshold summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactTargetThresholdSummaryValidationError {}

impl PackagedArtifactTargetThresholdSummary {
    /// Returns the target-threshold posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "profile id={}; target thresholds: {}; scopes={}; {}; scope envelopes={}; phase 2 corpus alignment={}",
            self.profile_id,
            self.state,
            self.scopes.join(", "),
            self.fit_envelope.summary_line(),
            join_display(&self.scope_envelopes.scope_envelopes),
            self.phase2_corpus_alignment.summary_line(),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactTargetThresholdSummaryValidationError> {
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "profile_id",
                },
            );
        }
        self.state.validate_production_ready().map_err(|_| {
            PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync { field: "state" }
        })?;
        if self.state != PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "state",
                },
            );
        }
        if self.scopes != PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "scopes",
                },
            );
        }

        let expected_fit_envelope = packaged_artifact_fit_envelope_summary_details();
        if self.fit_envelope != expected_fit_envelope {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "fit_envelope",
                },
            );
        }
        if self.state.is_production_ready() {
            let thresholds = packaged_artifact_fit_threshold_summary_details();
            self.fit_envelope
                .validate_against_thresholds(&thresholds)
                .map_err(|_| {
                    PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                        field: "fit_envelope",
                    }
                })?;
        }
        let expected_scope_envelopes =
            packaged_artifact_target_threshold_scope_envelopes_summary_details();
        if self.scope_envelopes != expected_scope_envelopes {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "scope_envelopes",
                },
            );
        }
        self.scope_envelopes.validate().map_err(|_| {
            PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                field: "scope_envelopes",
            }
        })?;

        let expected_phase2_corpus_alignment =
            packaged_artifact_phase2_corpus_alignment_summary_details().ok_or(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            )?;
        self.phase2_corpus_alignment
            .validate()
            .map_err(|error| match error {
                PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
                    field,
                } => PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: phase2_corpus_alignment_validation_field_path(field),
                },
            })?;
        if self.phase2_corpus_alignment != expected_phase2_corpus_alignment {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            );
        }

        let thresholds = packaged_artifact_fit_threshold_summary_details();
        for scope_envelope in &self.scope_envelopes.scope_envelopes {
            scope_envelope
                .fit_envelope
                .validate_against_thresholds(&thresholds)
                .map_err(|_| {
                    PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                        field: "scope_envelopes",
                    }
                })?;
        }

        Ok(())
    }

    /// Returns the validated target-threshold posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactTargetThresholdSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactTargetThresholdSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact target-threshold summary record.
pub fn packaged_artifact_target_threshold_summary_details() -> PackagedArtifactTargetThresholdSummary
{
    let summary = PackagedArtifactTargetThresholdSummary {
        profile_id: ARTIFACT_PROFILE_ID,
        state: PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE,
        scopes: PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES,
        fit_envelope: packaged_artifact_fit_envelope_summary_details(),
        scope_envelopes: packaged_artifact_target_threshold_scope_envelopes_summary_details(),
        phase2_corpus_alignment: packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available"),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact target-threshold summary after validating the structured posture.
pub fn packaged_artifact_target_threshold_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_target_threshold_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => format!("target thresholds: unavailable ({error})"),
            }
        })
        .clone()
}

/// Structured sync summary for the packaged-artifact source-fit and hold-out checks.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactSourceFitHoldoutSyncSummary {
    /// Calibrated fit thresholds used by the current packaged-artifact posture.
    pub fit_thresholds: PackagedArtifactFitThresholdSummary,
    /// Release-threshold posture that keeps the phase-2 corpus alignment synchronized.
    pub target_thresholds: PackagedArtifactTargetThresholdSummary,
    /// Phase-2 corpus evidence that anchors the threshold posture to current source coverage.
    pub phase2_corpus_alignment: PackagedArtifactPhase2CorpusAlignmentSummary,
}

/// Validation error for a packaged-artifact source-fit and hold-out sync summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactSourceFitHoldoutSyncSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactSourceFitHoldoutSyncSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact source-fit and hold-out sync summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactSourceFitHoldoutSyncSummaryValidationError {}

impl PackagedArtifactSourceFitHoldoutSyncSummary {
    /// Returns the source-fit and hold-out sync posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "source-fit and hold-out sync: fit thresholds={}; target thresholds={}; phase 2 corpus alignment={}",
            self.fit_thresholds.summary_line(),
            self.target_thresholds.summary_line(),
            self.phase2_corpus_alignment.summary_line(),
        )
    }

    /// Returns `Ok(())` when the sync summary still matches the current packaged-artifact posture.
    pub fn validate(
        &self,
    ) -> Result<(), PackagedArtifactSourceFitHoldoutSyncSummaryValidationError> {
        let expected_fit_thresholds = packaged_artifact_fit_threshold_summary_details();
        if self.fit_thresholds != expected_fit_thresholds {
            return Err(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "fit_thresholds",
                },
            );
        }
        self.fit_thresholds.validate().map_err(|_| {
            PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                field: "fit_thresholds",
            }
        })?;

        let expected_target_thresholds = packaged_artifact_target_threshold_summary_details();
        if self.target_thresholds != expected_target_thresholds {
            return Err(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "target_thresholds",
                },
            );
        }
        self.target_thresholds.validate().map_err(|_| {
            PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                field: "target_thresholds",
            }
        })?;

        let expected_phase2_corpus_alignment =
            packaged_artifact_phase2_corpus_alignment_summary_details().ok_or(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            )?;
        self.phase2_corpus_alignment
            .validate()
            .map_err(|error| match error {
                PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
                    field,
                } => PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: phase2_corpus_alignment_validation_field_path(field),
                },
            })?;
        if self.phase2_corpus_alignment != expected_phase2_corpus_alignment {
            return Err(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated source-fit and hold-out sync posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactSourceFitHoldoutSyncSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactSourceFitHoldoutSyncSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact source-fit and hold-out sync summary record.
pub fn packaged_artifact_source_fit_holdout_sync_summary_details(
) -> PackagedArtifactSourceFitHoldoutSyncSummary {
    let summary = PackagedArtifactSourceFitHoldoutSyncSummary {
        fit_thresholds: packaged_artifact_fit_threshold_summary_details(),
        target_thresholds: packaged_artifact_target_threshold_summary_details(),
        phase2_corpus_alignment: packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available"),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact source-fit and hold-out sync posture after validating the structured evidence.
pub fn packaged_artifact_source_fit_holdout_sync_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_source_fit_holdout_sync_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => format!("source-fit and hold-out sync: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact body-class target-threshold envelopes after validating the structured posture.
pub fn packaged_artifact_target_threshold_scope_envelopes_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_target_threshold_scope_envelopes_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => format!("scope envelopes: unavailable ({error})"),
            }
        })
        .clone()
}

/// Structured production-profile skeleton for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactProductionProfileSummary {
    /// Stable identifier for the production-profile skeleton.
    pub profile_id: &'static str,
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Covered time range for the packaged artifact.
    pub time_range: TimeRange,
    /// Provenance summary for the checked-in production-generation corpus.
    pub source_provenance: String,
    /// Bodies bundled into the packaged artifact.
    pub body_coverage: PackagedBodyCoverageSummary,
    /// Capability profile encoded by the packaged artifact.
    pub artifact_profile: ArtifactProfile,
    /// Output speed policy encoded by the packaged artifact.
    pub speed_policy: pleiades_compression::SpeedPolicy,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Request policy encoded by the packaged artifact.
    pub request_policy: PackagedRequestPolicySummary,
    /// Lookup-epoch policy encoded by the packaged artifact.
    pub lookup_epoch_policy: PackagedLookupEpochPolicy,
    /// Frame-treatment policy encoded by the packaged artifact.
    pub frame_treatment: PackagedFrameTreatmentSummary,
    /// Storage/reconstruction policy encoded by the packaged artifact.
    pub storage_summary: PackagedArtifactStorageSummary,
    /// Release-facing statement about the packaged-artifact target thresholds.
    pub target_thresholds: PackagedArtifactTargetThresholdSummary,
}

/// Validation error for a packaged artifact production-profile skeleton that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactProductionProfileSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactProductionProfileSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact production profile summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactProductionProfileSummaryValidationError {}

impl PackagedArtifactProductionProfileSummary {
    /// Returns the production-profile skeleton as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact production profile draft: profile id={}; label={}; version={}; time range={}; source provenance={}; body coverage={}; artifact profile={}; output support={}; speed policy={}; generation policy={}; segment strategy={}; request policy={}; lookup epoch policy={}; frame treatment={}; storage/reconstruction={}; {}",
            self.profile_id,
            self.label,
            self.artifact_version,
            self.time_range,
            self.source_provenance,
            self.body_coverage,
            self.artifact_profile,
            self.artifact_profile.output_support_entries_summary_line(),
            self.speed_policy,
            self.generation_policy,
            self.generation_policy.segment_strategy(),
            self.request_policy,
            self.lookup_epoch_policy.summary_line(),
            self.frame_treatment,
            self.storage_summary,
            self.target_thresholds,
        )
    }

    /// Returns `Ok(())` when the production-profile skeleton still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactProductionProfileSummaryValidationError> {
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "profile_id",
                },
            );
        }
        if self.label != ARTIFACT_LABEL {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "label",
                },
            );
        }
        if self.artifact_version != packaged_artifact().header.version {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "artifact_version",
                },
            );
        }
        if self.time_range != artifact_time_range(packaged_artifact()) {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "time_range",
                },
            );
        }
        if self.source_provenance != production_generation_source_summary_for_report() {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "source_provenance",
                },
            );
        }
        self.body_coverage.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "body_coverage",
            }
        })?;
        if self.artifact_profile != packaged_artifact_profile_summary_details().profile {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "artifact_profile",
                },
            );
        }
        if self.speed_policy != self.artifact_profile.speed_policy {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "speed_policy",
                },
            );
        }
        self.generation_policy.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "generation_policy",
            }
        })?;
        self.request_policy.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "request_policy",
            }
        })?;
        if self.lookup_epoch_policy != packaged_lookup_epoch_policy_summary_details().policy {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "lookup_epoch_policy",
                },
            );
        }
        self.frame_treatment.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "frame_treatment",
            }
        })?;
        self.storage_summary.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "storage_summary",
            }
        })?;
        self.target_thresholds.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "target_thresholds",
            }
        })?;

        Ok(())
    }

    /// Returns the validated production-profile skeleton summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactProductionProfileSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactProductionProfileSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact production-profile skeleton.
pub fn packaged_artifact_production_profile_summary_details(
) -> PackagedArtifactProductionProfileSummary {
    let artifact = packaged_artifact();
    let profile_summary = packaged_artifact_profile_summary_details();
    let speed_policy = profile_summary.profile.speed_policy;
    let summary = PackagedArtifactProductionProfileSummary {
        profile_id: ARTIFACT_PROFILE_ID,
        label: ARTIFACT_LABEL,
        artifact_version: artifact.header.version,
        time_range: artifact_time_range(artifact),
        source_provenance: production_generation_source_summary_for_report(),
        body_coverage: packaged_body_coverage_summary_details(),
        artifact_profile: profile_summary.profile,
        speed_policy,
        generation_policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
        request_policy: packaged_request_policy_summary_details(),
        lookup_epoch_policy: packaged_lookup_epoch_policy_summary_details().policy,
        frame_treatment: packaged_frame_treatment_summary_details(),
        storage_summary: packaged_artifact_storage_summary_details(),
        target_thresholds: packaged_artifact_target_threshold_summary_details(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact production-profile draft after validating the structured posture.
pub fn packaged_artifact_production_profile_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_production_profile_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => {
                    format!("Packaged artifact production profile draft: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the current packaged-artifact production-profile draft summary.
pub fn packaged_artifact_production_profile_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_production_profile_summary_details();
            match summary.validated_summary_line() {
                Ok(rendered) => rendered,
                Err(error) => {
                    format!("Packaged artifact production profile draft: unavailable ({error})")
                }
            }
        })
        .as_str()
}

/// Structured generation parameters for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactGeneratorParameters {
    /// Stable identifier for the generation profile.
    pub profile_id: &'static str,
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Covered time range for the packaged artifact.
    pub time_range: TimeRange,
    /// Provenance summary for the checked-in production-generation corpus.
    pub source_provenance: String,
    /// Deterministic checksum of the checked-in packaged artifact.
    pub checksum: u64,
    /// Encoded size of the checked-in packaged artifact in bytes.
    pub artifact_size_bytes: usize,
    /// Bodies bundled into the packaged artifact.
    pub body_coverage: PackagedBodyCoverageSummary,
    /// Residual-bearing body coverage encoded by the packaged artifact.
    pub residual_body_coverage: ArtifactResidualBodyCoverageSummary,
    /// Capability profile encoded by the packaged artifact.
    pub artifact_profile: ArtifactProfile,
    /// Output speed policy encoded by the packaged artifact.
    pub speed_policy: pleiades_compression::SpeedPolicy,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Request policy encoded by the packaged artifact.
    pub request_policy: PackagedRequestPolicySummary,
    /// Lookup-epoch policy encoded by the packaged artifact.
    pub lookup_epoch_policy: PackagedLookupEpochPolicy,
    /// Frame-treatment policy encoded by the packaged artifact.
    pub frame_treatment: PackagedFrameTreatmentSummary,
    /// Storage/reconstruction policy encoded by the packaged artifact.
    pub storage_summary: PackagedArtifactStorageSummary,
    /// Release-facing statement about the packaged-artifact target thresholds.
    pub target_thresholds: PackagedArtifactTargetThresholdSummary,
}

impl PackagedArtifactGeneratorParameters {
    /// Returns the generator parameters as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact generator parameters: profile id={}; label={}; version={}; time range={}; source provenance={}; checksum=0x{:016x}; artifact size={} bytes; body coverage={}; residual bodies={}; artifact profile={}; output support={}; speed policy={}; generation policy={}; segment strategy={}; request policy={}; lookup epoch policy={}; frame treatment={}; storage/reconstruction={}; {}",
            self.profile_id,
            self.label,
            self.artifact_version,
            self.time_range,
            self.source_provenance,
            self.checksum,
            self.artifact_size_bytes,
            self.body_coverage,
            self.residual_body_coverage.summary_line_with_body_count(),
            self.artifact_profile,
            self.artifact_profile.output_support_entries_summary_line(),
            self.speed_policy,
            self.generation_policy,
            self.generation_policy.segment_strategy(),
            self.request_policy,
            self.lookup_epoch_policy.summary_line(),
            self.frame_treatment,
            self.storage_summary,
            self.target_thresholds,
        )
    }

    /// Returns `Ok(())` when the parameters still match the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let current = packaged_artifact_production_profile_summary_details();
        let artifact = packaged_artifact();

        if self.profile_id != current.profile_id {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters profile id does not match the current production profile",
            ));
        }
        if self.label != current.label {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters label does not match the current production profile",
            ));
        }
        if self.artifact_version != current.artifact_version {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters version does not match the current production profile",
            ));
        }
        if self.time_range != current.time_range {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters time range does not match the current production profile",
            ));
        }
        if self.source_provenance != current.source_provenance {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters source provenance does not match the current production profile",
            ));
        }
        if self.checksum != artifact.checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters checksum does not match the current packaged artifact",
            ));
        }
        let expected_artifact_size_bytes = packaged_artifact_encoded_bytes(artifact);
        if self.artifact_size_bytes != expected_artifact_size_bytes {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact generator parameters artifact size {} bytes does not match the current packaged artifact size {} bytes",
                    self.artifact_size_bytes,
                    expected_artifact_size_bytes
                ),
            ));
        }
        if self.body_coverage != current.body_coverage {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters body coverage does not match the current production profile",
            ));
        }
        if self.residual_body_coverage != artifact.residual_body_coverage_summary() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters residual body coverage does not match the current packaged artifact",
            ));
        }
        if self.artifact_profile != current.artifact_profile {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters artifact profile does not match the current production profile",
            ));
        }
        if self.speed_policy != current.speed_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters speed policy does not match the current production profile",
            ));
        }
        if self.generation_policy != current.generation_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters generation policy does not match the current production profile",
            ));
        }
        if self.request_policy != current.request_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters request policy does not match the current production profile",
            ));
        }
        if self.lookup_epoch_policy != current.lookup_epoch_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters lookup epoch policy does not match the current production profile",
            ));
        }
        if self.frame_treatment != current.frame_treatment {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters frame-treatment policy does not match the current production profile",
            ));
        }
        if self.storage_summary != current.storage_summary {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters storage summary does not match the current production profile",
            ));
        }
        self.target_thresholds
            .state
            .validate_production_ready()
            .map_err(|_| {
                pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    "packaged artifact generator parameters target thresholds do not match the current production profile",
                )
            })?;
        if self.target_thresholds != current.target_thresholds {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters target thresholds do not match the current production profile",
            ));
        }

        Ok(())
    }

    /// Returns the validated generator parameters summary line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactGeneratorParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn packaged_artifact_generation_manifest_checksum(
    parameters: &PackagedArtifactGeneratorParameters,
    regeneration: &PackagedArtifactRegenerationSummary,
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(parameters.summary_line().as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(regeneration.summary_line().as_bytes());
    fnv1a64(&bytes)
}

/// Structured deterministic manifest for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactGenerationManifest {
    /// Generator parameters used to produce the packaged artifact.
    pub parameters: PackagedArtifactGeneratorParameters,
    /// Regeneration provenance anchored to the checked-in artifact and source snapshot.
    pub regeneration: PackagedArtifactRegenerationSummary,
    /// Deterministic checksum of the rendered manifest content.
    pub manifest_checksum: u64,
}

impl PackagedArtifactGenerationManifest {
    /// Returns the deterministic manifest as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact generation manifest: manifest checksum=0x{:016x}; {}; regeneration={}",
            self.manifest_checksum, self.parameters, self.regeneration,
        )
    }

    /// Returns `Ok(())` when the manifest still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        self.parameters.validate()?;
        self.regeneration.validate()?;

        let expected_checksum =
            packaged_artifact_generation_manifest_checksum(&self.parameters, &self.regeneration);
        if self.manifest_checksum != expected_checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact generation manifest checksum 0x{:016x} does not match the current packaged-artifact manifest checksum 0x{:016x}",
                    self.manifest_checksum,
                    expected_checksum
                ),
            ));
        }

        Ok(())
    }

    /// Returns the validated manifest summary line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactGenerationManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact generator parameters.
pub fn packaged_artifact_generator_parameters_details() -> PackagedArtifactGeneratorParameters {
    let summary = packaged_artifact_production_profile_summary_details();
    let regeneration = packaged_artifact_regeneration_summary_details();
    let parameters = PackagedArtifactGeneratorParameters {
        profile_id: summary.profile_id,
        label: summary.label,
        artifact_version: summary.artifact_version,
        time_range: summary.time_range,
        source_provenance: summary.source_provenance,
        checksum: regeneration.checksum,
        artifact_size_bytes: regeneration.artifact_size_bytes,
        body_coverage: summary.body_coverage,
        residual_body_coverage: regeneration.residual_body_coverage_summary(),
        artifact_profile: summary.artifact_profile,
        speed_policy: summary.speed_policy,
        generation_policy: summary.generation_policy,
        request_policy: summary.request_policy,
        lookup_epoch_policy: summary.lookup_epoch_policy,
        frame_treatment: summary.frame_treatment,
        storage_summary: summary.storage_summary,
        target_thresholds: summary.target_thresholds,
    };
    debug_assert!(parameters.validate().is_ok());
    parameters
}

/// Returns the current deterministic packaged-artifact generation manifest.
pub fn packaged_artifact_generation_manifest_details() -> PackagedArtifactGenerationManifest {
    let parameters = packaged_artifact_generator_parameters_details();
    let regeneration = packaged_artifact_regeneration_summary_details();
    let manifest = PackagedArtifactGenerationManifest {
        manifest_checksum: packaged_artifact_generation_manifest_checksum(
            &parameters,
            &regeneration,
        ),
        parameters,
        regeneration,
    };
    debug_assert!(manifest.validate().is_ok());
    manifest
}

/// Returns the current deterministic packaged-artifact generation manifest after validation.
pub fn packaged_artifact_generation_manifest_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = packaged_artifact_generation_manifest_details();
            match manifest.validated_summary_line() {
                Ok(line) => line,
                Err(error) => {
                    format!("Packaged artifact generation manifest: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the current deterministic packaged-artifact generation manifest checksum after validation.
pub fn packaged_artifact_generation_manifest_checksum_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = packaged_artifact_generation_manifest_details();
            match manifest.validate() {
                Ok(()) => format!(
                    "Packaged artifact generation manifest checksum: 0x{:016x}",
                    manifest.manifest_checksum
                ),
                Err(error) => {
                    format!("Packaged artifact generation manifest checksum: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the current deterministic packaged-artifact generation manifest.
pub fn packaged_artifact_generation_manifest() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = packaged_artifact_generation_manifest_details();
            match manifest.validated_summary_line() {
                Ok(rendered) => rendered,
                Err(error) => {
                    format!("Packaged artifact generation manifest: unavailable ({error})")
                }
            }
        })
        .as_str()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactProfileSummary {
    /// Number of bundled bodies that share the packaged artifact profile.
    pub body_count: usize,
    /// Bodies bundled under the packaged artifact profile.
    pub bodies: Vec<CelestialBody>,
    /// Byte-order policy encoded by the packaged artifact.
    pub endian_policy: EndianPolicy,
    /// Capability profile encoded by the packaged artifact.
    pub profile: ArtifactProfile,
}

impl PackagedArtifactProfileSummary {
    /// Returns the packaged artifact profile coverage as a typed summary.
    pub fn profile_coverage_summary(&self) -> ArtifactProfileCoverageSummary {
        ArtifactProfileCoverageSummary::new(self.profile.clone(), self.bodies.clone())
    }

    /// Validates that the packaged artifact profile summary is internally
    /// consistent with its bundled body list, byte-order policy, and embedded capability profile.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let artifact = packaged_artifact();
        if self.endian_policy != artifact.header.endian_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact profile byte-order policy does not match the checked-in packaged artifact header",
            ));
        }
        if self.profile != artifact.profile_coverage_summary().profile {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact profile metadata does not match the checked-in packaged artifact profile",
            ));
        }

        if self.body_count != self.bodies.len() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact profile body count does not match bundled body list",
            ));
        }

        if self.bodies.is_empty() {
            let coverage = self.profile_coverage_summary();
            coverage.validate()?;
            return Ok(());
        }

        if self
            .bodies
            .iter()
            .enumerate()
            .any(|(index, body)| self.bodies[..index].contains(body))
        {
            let coverage = self.profile_coverage_summary();
            coverage.validate()?;
            return Ok(());
        }

        if self.bodies.as_slice() != packaged_bodies() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact profile bundled body list does not match the checked-in packaged body set: expected [{}]; got [{}]",
                    join_display(packaged_bodies()),
                    join_display(&self.bodies)
                ),
            ));
        }

        let coverage = self.profile_coverage_summary();
        coverage.validate()?;

        Ok(())
    }

    /// Returns the validated packaged artifact profile summary line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the validated packaged artifact profile summary line with bundled bodies.
    pub fn validated_summary_line_with_bodies(
        &self,
    ) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line_with_bodies())
    }

    /// Returns the validated packaged artifact profile summary line with output support.
    pub fn validated_summary_line_with_output_support(
        &self,
    ) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(format!(
            "{}; output support: {}",
            self.summary_line_with_bodies(),
            self.profile.validated_output_support_summary_line()?
        ))
    }

    /// Renders the packaged artifact profile into a release-facing summary line.
    pub fn summary_line(&self) -> String {
        let coverage = self.profile_coverage_summary();
        format!(
            "byte order: {}; {}",
            self.endian_policy,
            coverage.summary_line()
        )
    }

    /// Returns the packaged artifact profile's output-support summary line.
    pub fn output_support_summary_line(&self) -> String {
        self.profile.output_support_summary_line()
    }

    /// Renders the packaged artifact profile with its bundled body list.
    pub fn summary_line_with_bodies(&self) -> String {
        let coverage = self.profile_coverage_summary();
        format!(
            "byte order: {}; {}",
            self.endian_policy,
            coverage.summary_line_with_bodies(),
        )
    }

    /// Renders the packaged artifact profile together with the built-in output
    /// support posture used by the current packaged artifact.
    pub fn summary_line_with_output_support(&self) -> String {
        format!(
            "{}; output support: {}",
            self.summary_line_with_bodies(),
            self.output_support_summary_line()
        )
    }
}

impl fmt::Display for PackagedArtifactProfileSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact profile summary record.
pub fn packaged_artifact_profile_summary_details() -> PackagedArtifactProfileSummary {
    let artifact = packaged_artifact();
    let coverage = artifact.profile_coverage_summary();
    let summary = PackagedArtifactProfileSummary {
        body_count: coverage.body_count,
        bodies: coverage.bodies,
        endian_policy: artifact.header.endian_policy,
        profile: coverage.profile,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact profile coverage summary record.
pub fn packaged_artifact_profile_coverage_summary_details() -> ArtifactProfileCoverageSummary {
    packaged_artifact_profile_summary_details().profile_coverage_summary()
}

/// Returns the current packaged-artifact profile coverage summary for reporting.
pub fn packaged_artifact_profile_coverage_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_profile_summary_details();
            match summary.validate() {
                Ok(()) => match summary
                    .profile_coverage_summary()
                    .validated_summary_line_with_bodies()
                {
                    Ok(line) => line,
                    Err(error) => format!("Artifact profile coverage: unavailable ({error})"),
                },
                Err(error) => format!("Artifact profile coverage: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact profile summary.
///
/// The summary is validated before it is rendered so release-facing callers
/// see an explicit unavailable marker if the bundled profile metadata drifts.
pub fn packaged_artifact_profile_summary() -> String {
    render_packaged_artifact_profile_summary(&packaged_artifact_profile_summary_details(), false)
}

/// Returns the current packaged-artifact profile summary with bundled body coverage.
///
/// The summary is validated before it is rendered so release-facing callers
/// see an explicit unavailable marker if the bundled profile metadata drifts.
pub fn packaged_artifact_profile_summary_with_body_coverage() -> String {
    render_packaged_artifact_profile_summary(&packaged_artifact_profile_summary_details(), true)
}

/// Returns the current packaged-artifact profile summary with the output-support posture.
///
/// The summary is validated before it is rendered so release-facing callers
/// see an explicit unavailable marker if the bundled profile metadata drifts.
pub fn packaged_artifact_profile_summary_with_output_support() -> String {
    let summary = packaged_artifact_profile_summary_details();
    match summary.validated_summary_line_with_output_support() {
        Ok(line) => line,
        Err(error) => {
            format!("Packaged artifact profile with output support: unavailable ({error})")
        }
    }
}

/// Returns the current packaged-artifact profile summary with output support for reporting.
pub fn packaged_artifact_profile_summary_with_output_support_for_report() -> String {
    packaged_artifact_profile_summary_with_output_support()
}

/// Structured output-support semantics for the packaged artifact profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactOutputSupportSummary {
    /// Capability profile encoded by the packaged artifact.
    pub profile: ArtifactProfile,
}

fn validate_packaged_artifact_output_support_profile(
    profile: &ArtifactProfile,
) -> Result<(), pleiades_compression::CompressionError> {
    let expected_states = [
        (
            ArtifactOutput::EclipticCoordinates,
            ArtifactOutputSupport::Derived,
        ),
        (
            ArtifactOutput::EquatorialCoordinates,
            ArtifactOutputSupport::Derived,
        ),
        (
            ArtifactOutput::ApparentCorrections,
            ArtifactOutputSupport::Unsupported,
        ),
        (
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutputSupport::Unsupported,
        ),
        (
            ArtifactOutput::SiderealCoordinates,
            ArtifactOutputSupport::Unsupported,
        ),
        (ArtifactOutput::Motion, ArtifactOutputSupport::Unsupported),
    ];

    for (output, expected_support) in expected_states {
        let actual_support = profile.output_support(output);
        if actual_support != expected_support {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact output support summary is out of sync with the bundled artifact profile field `output_support[{output}]`: expected {expected_support}, found {actual_support}"
                ),
            ));
        }
    }

    Ok(())
}

impl PackagedArtifactOutputSupportSummary {
    /// Validates that the embedded artifact profile is internally consistent
    /// and still advertises the packaged artifact's current built-in output
    /// support posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        self.profile.validate()?;
        validate_packaged_artifact_output_support_profile(&self.profile)
    }

    /// Returns the validated output-support posture for the packaged artifact profile.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Renders the packaged artifact profile's output-support semantics.
    pub fn summary_line(&self) -> String {
        self.profile.output_support_summary_line()
    }
}

impl fmt::Display for PackagedArtifactOutputSupportSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact output-support summary record.
pub fn packaged_artifact_output_support_summary_details() -> PackagedArtifactOutputSupportSummary {
    let summary = PackagedArtifactOutputSupportSummary {
        profile: packaged_artifact_profile_summary_details().profile,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the output-support semantics of the packaged artifact profile for reporting.
pub fn packaged_artifact_output_support_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_output_support_summary_details();
            match summary.validated_summary_line() {
                Ok(rendered) => rendered,
                Err(error) => format!("unavailable ({error})"),
            }
        })
        .clone()
}

/// Structured speed-policy semantics for the packaged artifact profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedArtifactSpeedPolicySummary {
    /// Speed policy encoded by the packaged artifact.
    pub policy: SpeedPolicy,
}

/// Validation error for the packaged-data speed-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactSpeedPolicySummaryValidationError {
    /// A summary field is out of sync with the current packaged-data posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactSpeedPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact speed-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactSpeedPolicySummaryValidationError {}

impl PackagedArtifactSpeedPolicySummary {
    /// Returns the packaged artifact speed policy as a compact human-readable line.
    pub fn summary_line(self) -> String {
        format!(
            "{}; motion output support={}",
            self.policy,
            self.policy.motion_output_support()
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-data posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactSpeedPolicySummaryValidationError> {
        let current_policy = packaged_artifact_profile_summary_details()
            .profile
            .speed_policy;
        if self.policy != current_policy {
            return Err(
                PackagedArtifactSpeedPolicySummaryValidationError::FieldOutOfSync {
                    field: "policy",
                },
            );
        }
        Ok(())
    }

    /// Returns the validated packaged artifact speed-policy summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactSpeedPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactSpeedPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const PACKAGED_ARTIFACT_SPEED_POLICY_SUMMARY: PackagedArtifactSpeedPolicySummary =
    PackagedArtifactSpeedPolicySummary {
        policy: SpeedPolicy::Unsupported,
    };

/// Returns the current packaged-artifact speed-policy summary record.
pub fn packaged_artifact_speed_policy_summary_details() -> PackagedArtifactSpeedPolicySummary {
    let summary = PACKAGED_ARTIFACT_SPEED_POLICY_SUMMARY;
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the packaged-artifact speed-policy semantics for reporting.
pub fn packaged_artifact_speed_policy_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_speed_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.summary_line(),
                Err(error) => format!("unavailable ({error})"),
            }
        })
        .clone()
}

pub(crate) fn render_packaged_artifact_profile_summary(
    summary: &PackagedArtifactProfileSummary,
    with_bodies: bool,
) -> String {
    if with_bodies {
        match summary.validated_summary_line_with_bodies() {
            Ok(line) => line,
            Err(error) => {
                format!("Packaged artifact profile with bundled bodies: unavailable ({error})")
            }
        }
    } else {
        match summary.validated_summary_line() {
            Ok(line) => line,
            Err(error) => format!("Packaged artifact profile: unavailable ({error})"),
        }
    }
}
