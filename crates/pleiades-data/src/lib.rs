//! Packaged compressed ephemeris backend for the common 1500-2500 range.
//!
//! This crate now ships a small stage-5 draft artifact backed by the
//! `pleiades-compression` codec. The bundled data is loaded from a checked-in
//! deterministic binary fixture that covers the comparison-body planetary set
//! plus the source-backed custom asteroid `asteroid:433-Eros`, and the backend
//! falls back to other providers when callers request bodies outside that
//! packaged slice. The packaged artifact stores ecliptic coordinates directly
//! and reconstructs equatorial coordinates from the stored channels and
//! mean-obliquity transform when requested. The fixture is still regenerated
//! from the checked-in JPL reference snapshot in tests so the packaged data
//! stays reproducible. A maintainer-facing regeneration helper can rebuild the
//! checked-in fixture from the bundled JPL reference snapshot without
//! introducing any native tooling. When the `packaged-artifact-path` feature is
//! enabled, callers can also load an explicit artifact file for larger or
//! externally distributed packaged datasets. See `docs/time-observer-policy.md`
//! for the explicit packaged request/lookup-epoch policy, and
//! `spec/data-compression.md` for the stored-vs-derived artifact contract.
//!
//! # Examples
//!
//! ```
//! use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
//! use pleiades_data::{packaged_backend, packaged_body_coverage_summary, packaged_lookup};
//!
//! let _backend = packaged_backend();
//! let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
//! let sun = packaged_lookup(&CelestialBody::Sun, instant)
//!     .expect("Sun should be in the packaged artifact");
//!
//! assert!(sun.distance_au.is_some());
//! assert!(packaged_body_coverage_summary().contains("433-Eros"));
//! ```

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::{cmp::Ordering, fmt};

#[cfg(feature = "packaged-artifact-path")]
use std::path::Path;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    Angle, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, CelestialBody, CoordinateFrame, CustomBodyId, EclipticCoordinates,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    Instant, JulianDay, QualityAnnotation, TimeRange, TimeScale, ZodiacMode,
};
use pleiades_compression::CompressedArtifact;
use pleiades_compression::{
    join_display, ArtifactHeader, ArtifactOutput, ArtifactOutputSupport, ArtifactProfile,
    ArtifactProfileCoverageSummary, ArtifactResidualBodyCoverageSummary, BodyArtifact, ChannelKind,
    EndianPolicy, PolynomialChannel, Segment, SpeedPolicy,
};
use pleiades_jpl::{
    format_reference_snapshot_summary, production_generation_source_summary_for_report,
    reference_snapshot, reference_snapshot_summary, JplSnapshotBackend, ReferenceSnapshotSummary,
    SnapshotEntry,
};

const PACKAGE_NAME: &str = "pleiades-data";
const ARTIFACT_LABEL: &str = "stage-5 packaged-data draft";
const ARTIFACT_PROFILE_ID: &str = "pleiades-packaged-artifact-profile/stage-5-draft";
const ARTIFACT_SOURCE: &str = "Quantized linear segments with residual-corrected Moon spans fitted to JPL Horizons reference epochs (1800, 2000, 2500 CE) for the comparison-body planetary set plus asteroid:433-Eros, with J2000 point segments for the outer planets, Pluto, and the asteroid coverage.";
const PACKAGED_BASE_BODIES: [CelestialBody; 10] = [
    CelestialBody::Sun,
    CelestialBody::Moon,
    CelestialBody::Mercury,
    CelestialBody::Venus,
    CelestialBody::Mars,
    CelestialBody::Jupiter,
    CelestialBody::Saturn,
    CelestialBody::Uranus,
    CelestialBody::Neptune,
    CelestialBody::Pluto,
];

const PACKAGED_REFERENCE_EPOCH_JD: f64 = 2_451_545.0;

fn packaged_bodies() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES.get_or_init(|| {
        let mut bodies = PACKAGED_BASE_BODIES.to_vec();
        bodies.push(CelestialBody::Custom(CustomBodyId::new(
            "asteroid", "433-Eros",
        )));
        bodies
    })
}

fn packaged_reference_entry_for_body(
    snapshot: &[SnapshotEntry],
    body: &CelestialBody,
) -> Option<SnapshotEntry> {
    snapshot
        .iter()
        .find(|entry| {
            entry.body == *body
                && (entry.epoch.julian_day.days() - PACKAGED_REFERENCE_EPOCH_JD).abs()
                    < f64::EPSILON
        })
        .cloned()
        .or_else(|| snapshot.iter().find(|entry| entry.body == *body).cloned())
}

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

fn format_validated_packaged_body_coverage_summary_for_report(
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
    /// Adjacent same-body source epochs are fit with linear segments.
    AdjacentSameBodyLinearSegments,
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
            Self::AdjacentSameBodyLinearSegments => "adjacent same-body linear segments",
        }
    }

    /// Returns the explanatory note used in release-facing summaries.
    pub const fn note(self) -> &'static str {
        match self {
            Self::AdjacentSameBodyLinearSegments => {
                "bodies with a single sampled epoch use point segments; multi-epoch non-lunar bodies are fit with linear segments between adjacent same-body source epochs; the Moon uses overlapping three-point spans with quadratic residual corrections to keep the high-curvature fit compact"
            }
        }
    }

    /// Returns the segment-strategy text used in release-facing summaries.
    pub const fn segment_strategy(self) -> &'static str {
        self.note()
    }

    /// Returns the compact release-facing summary for the generation policy.
    pub fn summary_line(self) -> String {
        format!("{}; {}", self.label(), self.note())
    }

    /// Returns `Ok(())` when the generation policy still matches the current packaged-artifact posture.
    pub fn validate(self) -> Result<(), PackagedArtifactGenerationPolicyValidationError> {
        if self != Self::AdjacentSameBodyLinearSegments {
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

fn validate_packaged_artifact_generation_policy_residual_bodies(
    policy: PackagedArtifactGenerationPolicy,
    residual_bodies: &[CelestialBody],
) -> Result<(), PackagedArtifactGenerationPolicySummaryValidationError> {
    match policy {
        PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments => {
            if residual_bodies != [CelestialBody::Moon] {
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
        policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments,
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
    let summary = packaged_artifact_generation_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
    }
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
    let artifact = packaged_artifact();
    let summary = packaged_artifact_generation_residual_bodies_summary_details();

    match summary.validated_summary_line_with_body_count(artifact) {
        Ok(line) => line,
        Err(error) => format!("residual bodies: unavailable ({error})"),
    }
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

/// Validation error for a packaged-artifact fit envelope that drifted from the current posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedArtifactFitEnvelopeSummaryValidationError {
    /// A rendered summary field no longer matches the current packaged-artifact fit envelope.
    FieldOutOfSync { field: &'static str },
    /// A measured fit field exceeds the calibrated packaged-artifact fit threshold.
    ThresholdExceeded {
        field: &'static str,
        measured_bits: u64,
        threshold_bits: u64,
    },
}

impl PackagedArtifactFitEnvelopeSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit envelope summary field `{field}` is out of sync with the current posture"
            ),
            Self::ThresholdExceeded {
                field,
                measured_bits,
                threshold_bits,
            } => format!(
                "the packaged artifact fit envelope summary field `{field}` exceeds the calibrated fit threshold (measured={:.12}, threshold={:.12})",
                f64::from_bits(*measured_bits),
                f64::from_bits(*threshold_bits),
            ),
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

impl PackagedArtifactFitEnvelopeSummary {
    /// Returns `Ok(())` when the measured fit envelope stays within the calibrated thresholds.
    pub fn validate_against_thresholds(
        &self,
        thresholds: &PackagedArtifactFitThresholdSummary,
    ) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        if self.mean_longitude_delta_degrees > thresholds.max_mean_longitude_delta_degrees {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded {
                    field: "mean_longitude_delta_degrees",
                    measured_bits: self.mean_longitude_delta_degrees.to_bits(),
                    threshold_bits: thresholds.max_mean_longitude_delta_degrees.to_bits(),
                },
            );
        }
        if self.mean_latitude_delta_degrees > thresholds.max_mean_latitude_delta_degrees {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded {
                    field: "mean_latitude_delta_degrees",
                    measured_bits: self.mean_latitude_delta_degrees.to_bits(),
                    threshold_bits: thresholds.max_mean_latitude_delta_degrees.to_bits(),
                },
            );
        }
        if self.mean_distance_delta_au > thresholds.max_mean_distance_delta_au {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded {
                    field: "mean_distance_delta_au",
                    measured_bits: self.mean_distance_delta_au.to_bits(),
                    threshold_bits: thresholds.max_mean_distance_delta_au.to_bits(),
                },
            );
        }
        if self.max_longitude_delta_degrees > thresholds.max_longitude_delta_degrees {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded {
                    field: "max_longitude_delta_degrees",
                    measured_bits: self.max_longitude_delta_degrees.to_bits(),
                    threshold_bits: thresholds.max_longitude_delta_degrees.to_bits(),
                },
            );
        }
        if self.max_latitude_delta_degrees > thresholds.max_latitude_delta_degrees {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded {
                    field: "max_latitude_delta_degrees",
                    measured_bits: self.max_latitude_delta_degrees.to_bits(),
                    threshold_bits: thresholds.max_latitude_delta_degrees.to_bits(),
                },
            );
        }
        if self.max_distance_delta_au > thresholds.max_distance_delta_au {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded {
                    field: "max_distance_delta_au",
                    measured_bits: self.max_distance_delta_au.to_bits(),
                    threshold_bits: thresholds.max_distance_delta_au.to_bits(),
                },
            );
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PackagedArtifactFitSample {
    body: CelestialBody,
    longitude_delta_degrees: f64,
    latitude_delta_degrees: f64,
    distance_delta_au: f64,
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

fn packaged_artifact_fit_sample_fractions(segment: &Segment) -> &'static [f64] {
    if segment.start.julian_day.days() == segment.end.julian_day.days() {
        &[0.0]
    } else {
        &[0.25, 0.5, 0.75]
    }
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
                .map(|segment| packaged_artifact_fit_sample_fractions(segment).len())
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
            for fraction in packaged_artifact_fit_sample_fractions(segment) {
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

fn packaged_artifact_fit_samples(artifact: &CompressedArtifact) -> Vec<PackagedArtifactFitSample> {
    packaged_artifact_fit_samples_with_filter(artifact, |_| true)
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
    let artifact = packaged_artifact();
    let samples = packaged_artifact_fit_samples(artifact);
    packaged_artifact_fit_envelope_summary_from_samples(
        &samples,
        packaged_artifact_fit_expected_sample_count(artifact),
    )
}

/// Returns the current packaged-artifact fit envelope after validating the structured posture.
pub fn packaged_artifact_fit_envelope_summary_for_report() -> String {
    let summary = packaged_artifact_fit_envelope_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("fit envelope: unavailable ({error})"),
    }
}

const PACKAGED_ARTIFACT_FIT_MAX_MEAN_LONGITUDE_DELTA_DEGREES: f64 = 29.750992955013;
const PACKAGED_ARTIFACT_FIT_MAX_MEAN_LATITUDE_DELTA_DEGREES: f64 = 22.784650147073;
const PACKAGED_ARTIFACT_FIT_MAX_MEAN_DISTANCE_DELTA_AU: f64 = 70_908.319_854_514_6;
const PACKAGED_ARTIFACT_FIT_MAX_LONGITUDE_DELTA_DEGREES: f64 = 179.935747101401;
const PACKAGED_ARTIFACT_FIT_MAX_LATITUDE_DELTA_DEGREES: f64 = 5436.377507814662;
const PACKAGED_ARTIFACT_FIT_MAX_DISTANCE_DELTA_AU: f64 = 19_941_928.384_904_474;

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
pub fn packaged_artifact_fit_margin_summary_for_report() -> String {
    let envelope = packaged_artifact_fit_envelope_summary_details();
    let thresholds = packaged_artifact_fit_threshold_summary_details();

    match envelope.validate_against_thresholds(&thresholds) {
        Ok(()) => format!(
            "fit margins: mean Δlon={:+.12}°, mean Δlat={:+.12}°, mean Δdist={:+.12} AU; max Δlon={:+.12}°, max Δlat={:+.12}°, max Δdist={:+.12} AU",
            thresholds.max_mean_longitude_delta_degrees - envelope.mean_longitude_delta_degrees,
            thresholds.max_mean_latitude_delta_degrees - envelope.mean_latitude_delta_degrees,
            thresholds.max_mean_distance_delta_au - envelope.mean_distance_delta_au,
            thresholds.max_longitude_delta_degrees - envelope.max_longitude_delta_degrees,
            thresholds.max_latitude_delta_degrees - envelope.max_latitude_delta_degrees,
            thresholds.max_distance_delta_au - envelope.max_distance_delta_au,
        ),
        Err(error) => format!("fit margins: unavailable ({error})"),
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
    let samples = packaged_artifact_fit_samples_with_filter(artifact, |body| {
        packaged_artifact_body_scope(body) == scope
    });
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

fn packaged_artifact_target_threshold_scope_envelopes_summary_details(
) -> Vec<PackagedArtifactTargetThresholdScopeSummary> {
    PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES
        .iter()
        .copied()
        .map(packaged_artifact_target_threshold_scope_envelope_summary_details)
        .collect()
}

fn format_scope_bodies(bodies: &[CelestialBody]) -> String {
    match bodies {
        [] => "0 (none)".to_string(),
        [single] => format!("1 ({single})"),
        _ => format!("{} ({})", bodies.len(), join_display(bodies)),
    }
}

fn packaged_artifact_quantization_scales_line() -> String {
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
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Per-channel quantization scales captured from the checked-in artifact.
    pub quantization_scales: String,
    /// Bodies that carry residual correction channels in the packaged artifact.
    pub residual_bodies: Vec<CelestialBody>,
    /// Bodies bundled into the packaged artifact.
    pub bodies: Vec<CelestialBody>,
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
    fn validate_residual_body_subset(&self) -> Result<(), pleiades_compression::CompressionError> {
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
        if self.source != ARTIFACT_SOURCE {
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
            "Packaged artifact regeneration source: label={}; profile id={}; source={}; source revision={}; checksum=0x{:016x}; {}; segment strategy={}; {}; {}; bundled bodies: {}; {}; fit envelope: {}; artifact version={}",
            self.label,
            self.profile_id,
            self.source,
            self.source_revision,
            self.checksum,
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
    let artifact = packaged_artifact();
    let summary = PackagedArtifactRegenerationSummary {
        label: ARTIFACT_LABEL,
        artifact_version: artifact.header.version,
        source: ARTIFACT_SOURCE,
        source_revision: production_generation_source_summary_for_report(),
        profile_id: ARTIFACT_PROFILE_ID,
        checksum: artifact.checksum,
        generation_policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments,
        quantization_scales: packaged_artifact_quantization_scales_line(),
        residual_bodies: artifact.residual_bodies(),
        bodies: packaged_bodies().to_vec(),
        fit_envelope: packaged_artifact_fit_envelope_summary_details(),
        reference_snapshot: reference_snapshot_summary(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the packaged-artifact regeneration provenance summary.
pub fn packaged_artifact_regeneration_summary() -> String {
    packaged_artifact_regeneration_summary_for_report()
}

/// Returns the packaged-artifact regeneration provenance summary after validating
/// the structured source and coverage metadata.
pub fn packaged_artifact_regeneration_summary_for_report() -> String {
    let summary = packaged_artifact_regeneration_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact regeneration source: unavailable ({error})"),
    }
}

const PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATUS: &str = "draft fit envelope recorded";
const PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES: &[&str] = &[
    "luminaries",
    "major planets",
    "pluto",
    "lunar points",
    "selected asteroids",
    "custom bodies",
];

/// Structured target-threshold posture for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactTargetThresholdSummary {
    /// Stable identifier for the release profile that the thresholds apply to.
    pub profile_id: &'static str,
    /// Current release posture for the production thresholds.
    pub status: &'static str,
    /// Body-class scopes that still require finalized thresholds.
    pub scopes: &'static [&'static str],
    /// Measured fit envelope captured for the current packaged artifact posture.
    pub fit_envelope: PackagedArtifactFitEnvelopeSummary,
    /// Body-class-specific fit envelopes captured for the current packaged artifact posture.
    pub scope_envelopes: Vec<PackagedArtifactTargetThresholdScopeSummary>,
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
            "profile id={}; target thresholds: {}; scopes={}; {}; scope envelopes={}",
            self.profile_id,
            self.status,
            self.scopes.join(", "),
            self.fit_envelope.summary_line(),
            join_display(&self.scope_envelopes),
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
        if self.status != PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATUS {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "status",
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
        let expected_scope_envelopes =
            packaged_artifact_target_threshold_scope_envelopes_summary_details();
        if self.scope_envelopes != expected_scope_envelopes {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "scope_envelopes",
                },
            );
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
        status: PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATUS,
        scopes: PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES,
        fit_envelope: packaged_artifact_fit_envelope_summary_details(),
        scope_envelopes: packaged_artifact_target_threshold_scope_envelopes_summary_details(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact target-threshold summary after validating the structured posture.
pub fn packaged_artifact_target_threshold_summary_for_report() -> String {
    let summary = packaged_artifact_target_threshold_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("target thresholds: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact body-class target-threshold envelopes after validating the structured posture.
pub fn packaged_artifact_target_threshold_scope_envelopes_for_report() -> String {
    let summary = packaged_artifact_target_threshold_summary_details();
    match summary.validate() {
        Ok(()) => match summary
            .scope_envelopes
            .iter()
            .map(|scope| scope.validated_summary_line())
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(lines) => format!("scope envelopes: {}", lines.join(", ")),
            Err(error) => format!("scope envelopes: unavailable ({error})"),
        },
        Err(error) => format!("scope envelopes: unavailable ({error})"),
    }
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
    /// Release-facing statement about the still-open production target thresholds.
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
        generation_policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments,
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
    let summary = packaged_artifact_production_profile_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => {
            format!("Packaged artifact production profile draft: unavailable ({error})")
        }
    }
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
    /// Release-facing statement about the still-open production target thresholds.
    pub target_thresholds: PackagedArtifactTargetThresholdSummary,
}

impl PackagedArtifactGeneratorParameters {
    /// Returns the generator parameters as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact generator parameters: profile id={}; label={}; version={}; time range={}; source provenance={}; body coverage={}; artifact profile={}; output support={}; speed policy={}; generation policy={}; segment strategy={}; request policy={}; lookup epoch policy={}; frame treatment={}; storage/reconstruction={}; {}",
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

    /// Returns `Ok(())` when the parameters still match the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let current = packaged_artifact_production_profile_summary_details();

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
        if self.body_coverage != current.body_coverage {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters body coverage does not match the current production profile",
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

/// Structured deterministic manifest for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactGenerationManifest {
    /// Generator parameters used to produce the packaged artifact.
    pub parameters: PackagedArtifactGeneratorParameters,
    /// Regeneration provenance anchored to the checked-in artifact and source snapshot.
    pub regeneration: PackagedArtifactRegenerationSummary,
}

impl PackagedArtifactGenerationManifest {
    /// Returns the deterministic manifest as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact generation manifest: {}; regeneration={}",
            self.parameters, self.regeneration,
        )
    }

    /// Returns `Ok(())` when the manifest still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        self.parameters.validate()?;
        self.regeneration.validate()?;
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
    let parameters = PackagedArtifactGeneratorParameters {
        profile_id: summary.profile_id,
        label: summary.label,
        artifact_version: summary.artifact_version,
        time_range: summary.time_range,
        source_provenance: summary.source_provenance,
        body_coverage: summary.body_coverage,
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
    let manifest = PackagedArtifactGenerationManifest {
        parameters: packaged_artifact_generator_parameters_details(),
        regeneration: packaged_artifact_regeneration_summary_details(),
    };
    debug_assert!(manifest.validate().is_ok());
    manifest
}

/// Returns the current deterministic packaged-artifact generation manifest after validation.
pub fn packaged_artifact_generation_manifest_for_report() -> String {
    let manifest = packaged_artifact_generation_manifest_details();
    match manifest.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact generation manifest: unavailable ({error})"),
    }
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
        Ok(self.summary_line_with_output_support())
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
        self.profile.output_support_entries_summary_line()
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
        self.profile.output_support_entries_summary_line()
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
    let summary = packaged_artifact_output_support_summary_details();
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("unavailable ({error})"),
    }
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
    let summary = packaged_artifact_speed_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("unavailable ({error})"),
    }
}

fn render_packaged_artifact_profile_summary(
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

fn validate_packaged_frame_treatment_summary_line(
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

fn validate_packaged_artifact_storage_summary_line(
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

fn validate_packaged_artifact_storage_profile(
    profile: &ArtifactProfile,
) -> Result<(), PackagedArtifactStorageSummaryValidationError> {
    const EXPECTED_STORED_CHANNELS: [ChannelKind; 3] = [
        ChannelKind::Longitude,
        ChannelKind::Latitude,
        ChannelKind::DistanceAu,
    ];
    const EXPECTED_DERIVED_OUTPUTS: [ArtifactOutput; 2] = [
        ArtifactOutput::EclipticCoordinates,
        ArtifactOutput::EquatorialCoordinates,
    ];
    const EXPECTED_UNSUPPORTED_OUTPUTS: [ArtifactOutput; 4] = [
        ArtifactOutput::ApparentCorrections,
        ArtifactOutput::TopocentricCoordinates,
        ArtifactOutput::SiderealCoordinates,
        ArtifactOutput::Motion,
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
        "Quantized linear segments stored in pleiades-compression artifact format; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, sidereal, and motion outputs remain unsupported"
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

fn validate_packaged_artifact_access_summary_line(
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

const AU_IN_KM: f64 = 149_597_870.7;

/// Returns the canonical package name for this crate.
pub const fn package_name() -> &'static str {
    PACKAGE_NAME
}

const PACKAGED_ARTIFACT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/packaged-artifact.bin");

/// Returns the checked-in packaged artifact bytes.
pub fn packaged_artifact_bytes() -> &'static [u8] {
    PACKAGED_ARTIFACT_FIXTURE
}

/// Returns the bundled packed artifact.
pub fn packaged_artifact() -> &'static CompressedArtifact {
    static ARTIFACT: OnceLock<CompressedArtifact> = OnceLock::new();
    ARTIFACT.get_or_init(build_packaged_artifact)
}

/// Decodes a packaged artifact from raw bytes.
pub fn packaged_artifact_from_bytes(
    bytes: &[u8],
) -> Result<CompressedArtifact, pleiades_compression::CompressionError> {
    let artifact = CompressedArtifact::decode(bytes)?;
    artifact.validate()?;
    Ok(artifact)
}

/// Errors that can occur while loading an external packaged artifact.
#[derive(Debug)]
pub enum PackagedArtifactLoadError {
    /// The artifact could not be read from disk.
    Io {
        /// Path that was attempted.
        path: PathBuf,
        /// The underlying I/O error.
        error: std::io::Error,
    },
    /// The artifact decoded but failed validation.
    Decode(pleiades_compression::CompressionError),
}

impl fmt::Display for PackagedArtifactLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, error } => write!(
                f,
                "failed to read packaged artifact at {}: {}",
                path.display(),
                error
            ),
            Self::Decode(error) => write!(f, "failed to decode packaged artifact: {}", error),
        }
    }
}

impl std::error::Error for PackagedArtifactLoadError {}

#[cfg(feature = "packaged-artifact-path")]
/// Loads a packaged artifact from a file path.
pub fn packaged_artifact_from_path(
    path: impl AsRef<Path>,
) -> Result<CompressedArtifact, PackagedArtifactLoadError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|error| PackagedArtifactLoadError::Io {
        path: path.to_path_buf(),
        error,
    })?;
    packaged_artifact_from_bytes(&bytes).map_err(PackagedArtifactLoadError::Decode)
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

/// A packaged compressed-data backend.
#[derive(Debug, Clone)]
pub struct PackagedDataBackend {
    artifact: Arc<CompressedArtifact>,
}

impl Default for PackagedDataBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PackagedDataBackend {
    /// Creates a new packaged-data backend backed by the checked-in fixture.
    pub fn new() -> Self {
        Self::from_artifact(packaged_artifact().clone())
    }

    /// Creates a packaged-data backend from an explicit artifact.
    pub fn from_artifact(artifact: CompressedArtifact) -> Self {
        Self {
            artifact: Arc::new(artifact),
        }
    }

    /// Creates a packaged-data backend from decoded artifact bytes.
    ///
    /// See [`packaged_backend_from_bytes`] for an end-to-end example that
    /// encodes the checked-in artifact and reloads it through this constructor.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackagedArtifactLoadError> {
        Ok(Self::from_artifact(
            packaged_artifact_from_bytes(bytes).map_err(PackagedArtifactLoadError::Decode)?,
        ))
    }

    #[cfg(feature = "packaged-artifact-path")]
    /// Creates a packaged-data backend from an artifact file.
    ///
    /// See [`packaged_backend_from_path`] for an end-to-end example that writes
    /// the checked-in artifact to a temporary file and reloads it through this
    /// constructor.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, PackagedArtifactLoadError> {
        Ok(Self::from_artifact(packaged_artifact_from_path(path)?))
    }

    fn artifact(&self) -> &CompressedArtifact {
        &self.artifact
    }
}

impl EphemerisBackend for PackagedDataBackend {
    fn metadata(&self) -> BackendMetadata {
        let artifact = self.artifact();
        let bodies = artifact
            .bodies
            .iter()
            .map(|series| series.body.clone())
            .collect::<Vec<_>>();
        let range = artifact_time_range(artifact);

        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: format!(
                "{} checksum:{:016x}",
                artifact.header.version, artifact.checksum
            ),
            family: BackendFamily::CompressedData,
            provenance: BackendProvenance {
                summary: artifact.header.source.clone(),
                data_sources: vec![
                    packaged_body_coverage_summary(),
                    packaged_request_policy_summary_details().to_string(),
                    packaged_frame_treatment_summary_details().to_string(),
                    packaged_artifact_storage_summary_for_report(),
                    packaged_artifact_access_summary_for_report(),
                ],
            },
            nominal_range: range,
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: bodies,
            supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.artifact
            .bodies
            .iter()
            .any(|series| series.body == body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !matches!(req.instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "packaged data only supports TT or TDB requests",
            ));
        }

        validate_request_policy(
            req,
            "packaged data",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            true,
            false,
        )?;

        validate_zodiac_policy(req, "packaged data", &[ZodiacMode::Tropical])?;

        validate_observer_policy(req, "packaged data", false)?;

        let lookup_instant = normalize_lookup_instant(req.instant);
        let ecliptic = self
            .artifact
            .lookup_ecliptic(&req.body, lookup_instant)
            .map_err(map_artifact_error)?;
        let equatorial = ecliptic.to_equatorial(req.instant.mean_obliquity());

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(ecliptic);
        result.equatorial = Some(equatorial);
        result.quality = QualityAnnotation::Interpolated;
        Ok(result)
    }
}

fn build_packaged_artifact() -> CompressedArtifact {
    packaged_artifact_from_bytes(PACKAGED_ARTIFACT_FIXTURE)
        .expect("packaged artifact fixture should decode and validate")
}

/// Rebuilds the packaged artifact from the checked-in JPL reference snapshot.
///
/// This helper is deterministic and pure Rust so maintainers can regenerate the
/// checked-in fixture without relying on platform-specific tooling.
pub fn regenerate_packaged_artifact() -> CompressedArtifact {
    let mut artifact = CompressedArtifact::new(
        ArtifactHeader::new(ARTIFACT_LABEL, ARTIFACT_SOURCE),
        packaged_body_artifacts(),
    );
    artifact.checksum = artifact
        .checksum()
        .expect("packaged artifact checksum should be reproducible");
    artifact
        .validate()
        .expect("packaged artifact should validate before encoding");
    artifact
}

fn packaged_body_artifacts() -> Vec<BodyArtifact> {
    let mut artifacts = Vec::new();
    let snapshot = reference_snapshot();

    for body in packaged_bodies().iter().cloned() {
        let mut entries: Vec<&SnapshotEntry> =
            snapshot.iter().filter(|entry| entry.body == body).collect();
        if entries.is_empty() {
            continue;
        }

        entries.sort_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .partial_cmp(&right.epoch.julian_day.days())
                .unwrap_or(Ordering::Equal)
        });

        let segments = if body == CelestialBody::Moon {
            moon_segments_from_entries(&entries)
        } else if entries.len() == 1 {
            vec![segment_from_entries(entries[0], entries[0])]
        } else {
            entries
                .windows(2)
                .map(|pair| segment_from_entries(pair[0], pair[1]))
                .collect()
        };

        artifacts.push(BodyArtifact::new(body, segments));
    }

    artifacts
}

fn moon_segments_from_entries(entries: &[&SnapshotEntry]) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut index = 0;

    while index + 2 < entries.len() {
        segments.push(segment_from_entries_with_midpoint(
            entries[index],
            entries[index + 1],
            entries[index + 2],
        ));
        index += 2;
    }

    if index + 1 < entries.len() {
        segments.push(segment_from_entries(entries[index], entries[index + 1]));
    }

    segments
}

fn segment_from_entries(start: &SnapshotEntry, end: &SnapshotEntry) -> Segment {
    let start_coordinates = coordinates(start);
    let end_coordinates = coordinates(end);
    Segment::new(
        Instant::new(start.epoch.julian_day, TimeScale::Tt),
        Instant::new(end.epoch.julian_day, TimeScale::Tt),
        vec![
            PolynomialChannel::linear(
                ChannelKind::Longitude,
                9,
                start_coordinates.longitude.degrees(),
                end_coordinates.longitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::Latitude,
                9,
                start_coordinates.latitude.degrees(),
                end_coordinates.latitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::DistanceAu,
                12,
                start_coordinates.distance_au.unwrap_or_default(),
                end_coordinates.distance_au.unwrap_or_default(),
            ),
        ],
    )
}

fn segment_from_entries_with_midpoint(
    start: &SnapshotEntry,
    midpoint: &SnapshotEntry,
    end: &SnapshotEntry,
) -> Segment {
    let start_coordinates = coordinates(start);
    let midpoint_coordinates = coordinates(midpoint);
    let end_coordinates = coordinates(end);
    let start_instant = Instant::new(start.epoch.julian_day, TimeScale::Tt);
    let midpoint_instant = Instant::new(midpoint.epoch.julian_day, TimeScale::Tt);
    let end_instant = Instant::new(end.epoch.julian_day, TimeScale::Tt);
    let span = end_instant.julian_day.days() - start_instant.julian_day.days();
    let midpoint_x = (midpoint_instant.julian_day.days() - start_instant.julian_day.days()) / span;

    Segment::with_residual_channels(
        start_instant,
        end_instant,
        vec![
            PolynomialChannel::linear(
                ChannelKind::Longitude,
                9,
                start_coordinates.longitude.degrees(),
                end_coordinates.longitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::Latitude,
                9,
                start_coordinates.latitude.degrees(),
                end_coordinates.latitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::DistanceAu,
                12,
                start_coordinates.distance_au.unwrap_or_default(),
                end_coordinates.distance_au.unwrap_or_default(),
            ),
        ],
        vec![
            residual_channel(
                ChannelKind::Longitude,
                9,
                start_coordinates.longitude.degrees(),
                midpoint_coordinates.longitude.degrees(),
                end_coordinates.longitude.degrees(),
                midpoint_x,
            ),
            residual_channel(
                ChannelKind::Latitude,
                9,
                start_coordinates.latitude.degrees(),
                midpoint_coordinates.latitude.degrees(),
                end_coordinates.latitude.degrees(),
                midpoint_x,
            ),
            residual_channel(
                ChannelKind::DistanceAu,
                12,
                start_coordinates.distance_au.unwrap_or_default(),
                midpoint_coordinates.distance_au.unwrap_or_default(),
                end_coordinates.distance_au.unwrap_or_default(),
                midpoint_x,
            ),
        ],
    )
}

fn residual_channel(
    kind: ChannelKind,
    scale_exponent: u8,
    start: f64,
    midpoint: f64,
    end: f64,
    midpoint_x: f64,
) -> PolynomialChannel {
    let base_midpoint = start + (end - start) * midpoint_x;
    let delta = midpoint - base_midpoint;
    let scale = midpoint_x * (1.0 - midpoint_x);
    let amplitude = if scale == 0.0 { 0.0 } else { delta / scale };

    PolynomialChannel::new(kind, scale_exponent, vec![0.0, amplitude, -amplitude])
}

fn coordinates(entry: &SnapshotEntry) -> EclipticCoordinates {
    let radius_km =
        (entry.x_km * entry.x_km + entry.y_km * entry.y_km + entry.z_km * entry.z_km).sqrt();
    let longitude = entry.y_km.atan2(entry.x_km).to_degrees();
    let latitude = (entry.z_km / radius_km)
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();
    EclipticCoordinates::new(
        pleiades_backend::Longitude::from_degrees(longitude),
        pleiades_backend::Latitude::from_degrees(latitude),
        Some(radius_km / AU_IN_KM),
    )
}

fn artifact_time_range(artifact: &CompressedArtifact) -> TimeRange {
    let mut start: Option<Instant> = None;
    let mut end: Option<Instant> = None;
    for body in &artifact.bodies {
        for segment in &body.segments {
            start = Some(match start {
                Some(current) => {
                    if segment.start.julian_day.days() < current.julian_day.days() {
                        segment.start
                    } else {
                        current
                    }
                }
                None => segment.start,
            });
            end = Some(match end {
                Some(current) => {
                    if segment.end.julian_day.days() > current.julian_day.days() {
                        segment.end
                    } else {
                        current
                    }
                }
                None => segment.end,
            });
        }
    }
    TimeRange::new(start, end)
}

fn normalize_lookup_instant(instant: Instant) -> Instant {
    match instant.scale {
        TimeScale::Tt => instant,
        TimeScale::Tdb => Instant::new(instant.julian_day, TimeScale::Tt),
        _ => instant,
    }
}

fn map_artifact_error(error: pleiades_compression::CompressionError) -> EphemerisError {
    let kind = match error.kind {
        pleiades_compression::CompressionErrorKind::MissingBody => {
            EphemerisErrorKind::UnsupportedBody
        }
        pleiades_compression::CompressionErrorKind::OutOfRangeInstant => {
            EphemerisErrorKind::OutOfRangeInstant
        }
        pleiades_compression::CompressionErrorKind::UnsupportedTimeScale => {
            EphemerisErrorKind::UnsupportedTimeScale
        }
        pleiades_compression::CompressionErrorKind::MissingChannel => {
            EphemerisErrorKind::MissingDataset
        }
        pleiades_compression::CompressionErrorKind::QuantizationOverflow
        | pleiades_compression::CompressionErrorKind::InvalidFormat
        | pleiades_compression::CompressionErrorKind::UnsupportedEndianPolicy
        | pleiades_compression::CompressionErrorKind::InvalidMagic
        | pleiades_compression::CompressionErrorKind::UnsupportedVersion
        | pleiades_compression::CompressionErrorKind::ChecksumMismatch
        | pleiades_compression::CompressionErrorKind::Truncated
        | _ => EphemerisErrorKind::NumericalFailure,
    };

    EphemerisError::new(kind, error.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_artifact_roundtrips_through_codec() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        assert_eq!(encoded, PACKAGED_ARTIFACT_FIXTURE);
        let decoded =
            CompressedArtifact::decode(&encoded).expect("packaged artifact should decode");
        assert_eq!(decoded.header.generation_label, ARTIFACT_LABEL);
        assert_eq!(decoded.bodies.len(), packaged_bodies().len());
        assert_eq!(decoded.checksum, artifact.checksum);
    }

    #[test]
    fn packaged_backend_from_artifact_uses_supplied_metadata() {
        let mut artifact = regenerate_packaged_artifact();
        artifact.header.source = "external packaged artifact".to_string();

        let backend = PackagedDataBackend::from_artifact(artifact);
        let metadata = backend.metadata();

        assert_eq!(metadata.provenance.summary, "external packaged artifact");
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
        assert!(metadata
            .supported_frames
            .contains(&CoordinateFrame::Equatorial));
    }

    #[cfg(feature = "packaged-artifact-path")]
    #[test]
    fn packaged_backend_from_path_loads_a_file_artifact() {
        let path = std::env::temp_dir().join(format!(
            "pleiades-data-packaged-artifact-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos()
        ));
        std::fs::write(&path, PACKAGED_ARTIFACT_FIXTURE).expect("test artifact should be writable");

        let backend = PackagedDataBackend::from_path(&path)
            .expect("packaged artifact path should load successfully");
        let metadata = backend.metadata();

        assert_eq!(metadata.id.as_str(), PACKAGE_NAME);
        assert!(metadata.offline);
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));

        let _ = std::fs::remove_file(&path);
    }

    #[cfg(feature = "packaged-artifact-path")]
    #[test]
    fn packaged_artifact_from_path_rejects_corrupted_artifact() {
        let path = std::env::temp_dir().join(format!(
            "pleiades-data-packaged-artifact-corrupt-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos()
        ));
        std::fs::write(&path, b"not a valid packaged artifact")
            .expect("corrupt artifact should be writable");

        let error = packaged_artifact_from_path(&path)
            .expect_err("corrupted packaged artifact should fail to decode");
        let error_text = error.to_string();

        match error {
            PackagedArtifactLoadError::Decode(_) => {}
            other => panic!("expected decode failure, got {other}"),
        }
        assert!(error_text.contains("failed to decode packaged artifact"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn packaged_artifact_decode_rejects_checksum_corruption() {
        let mut encoded = PACKAGED_ARTIFACT_FIXTURE.to_vec();
        let last_index = encoded.len() - 1;
        encoded[last_index] ^= 0x01;

        let error = CompressedArtifact::decode(&encoded)
            .expect_err("tampered packaged artifact should fail to decode");

        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::ChecksumMismatch
        );
    }

    #[test]
    fn packaged_artifact_fixture_matches_reference_snapshot_generation() {
        let generated = regenerate_packaged_artifact();
        generated
            .validate()
            .expect("generated packaged artifact should validate");
        let encoded = generated
            .encode()
            .expect("generated packaged artifact should encode");
        assert_eq!(encoded, PACKAGED_ARTIFACT_FIXTURE);
        assert_eq!(generated.residual_segment_count(), 13);
        assert_eq!(generated.residual_bodies(), vec![CelestialBody::Moon]);
    }

    #[test]
    fn lookup_uses_packaged_segments() {
        let reference = reference_snapshot()
            .iter()
            .find(|entry| {
                entry.body == CelestialBody::Sun
                    && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
            })
            .expect("reference snapshot should include the Sun at J2000");
        let ecliptic = packaged_lookup(&CelestialBody::Sun, reference.epoch)
            .expect("packaged lookup should succeed");
        let expected = coordinates(reference);

        assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-12);
    }

    #[test]
    fn equatorial_frame_requests_return_derived_coordinates() {
        let backend = packaged_backend();
        let reference = reference_snapshot()
            .iter()
            .find(|entry| {
                entry.body == CelestialBody::Sun
                    && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
            })
            .expect("reference snapshot should include the Sun at J2000");
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: reference.epoch,
            observer: None,
            frame: CoordinateFrame::Equatorial,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("packaged equatorial request should succeed");
        let expected = coordinates(reference).to_equatorial(reference.epoch.mean_obliquity());

        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        let actual_ecliptic = result
            .ecliptic
            .expect("packaged equatorial request should still expose ecliptic coordinates");
        let expected_ecliptic = coordinates(reference);
        assert!(
            (actual_ecliptic.longitude.degrees() - expected_ecliptic.longitude.degrees()).abs()
                < 1e-8
        );
        assert!(
            (actual_ecliptic.latitude.degrees() - expected_ecliptic.latitude.degrees()).abs()
                < 1e-8
        );
        assert!(
            (actual_ecliptic.distance_au.unwrap() - expected_ecliptic.distance_au.unwrap()).abs()
                < 1e-12
        );
        let actual_equatorial = result
            .equatorial
            .expect("packaged equatorial request should return derived equatorial coordinates");
        assert!(
            (actual_equatorial.right_ascension.degrees() - expected.right_ascension.degrees())
                .abs()
                < 1e-8
        );
        assert!(
            (actual_equatorial.declination.degrees() - expected.declination.degrees()).abs() < 1e-8
        );
        assert!(
            (actual_equatorial.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-12
        );
        assert_eq!(result.quality, QualityAnnotation::Interpolated);
    }

    #[test]
    fn lookup_uses_packaged_custom_asteroid_segments() {
        let reference = reference_snapshot()
            .iter()
            .find(|entry| {
                entry.body == CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
                    && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
            })
            .expect("reference snapshot should include asteroid:433-Eros at J2000");
        let body = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
        let ecliptic = packaged_lookup(&body, reference.epoch)
            .expect("packaged lookup should succeed for the custom asteroid");
        let expected = coordinates(reference);

        assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-12);
    }

    #[test]
    fn lookup_uses_packaged_moon_residual_segments() {
        let body = CelestialBody::Moon;
        for epoch in [2_400_000.0, 2_500_000.0] {
            let reference = reference_snapshot()
                .iter()
                .find(|entry| {
                    entry.body == body
                        && (entry.epoch.julian_day.days() - epoch).abs() < f64::EPSILON
                })
                .expect("reference snapshot should include the Moon at the sampled epoch");
            let ecliptic = packaged_lookup(&body, reference.epoch)
                .expect("packaged lookup should succeed for the Moon");
            let expected = coordinates(reference);

            assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
            assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-8);
            assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-12);
        }

        assert_eq!(packaged_artifact().residual_segment_count(), 13);
        assert_eq!(
            packaged_artifact().residual_bodies(),
            vec![CelestialBody::Moon]
        );
    }

    #[test]
    fn lookup_uses_packaged_boundary_epochs_for_every_reference_body() {
        use std::collections::HashMap;

        let mut body_bounds: HashMap<CelestialBody, (Instant, Instant)> = HashMap::new();
        for body in packaged_bodies() {
            let mut body_entries = reference_snapshot()
                .iter()
                .filter(|entry| entry.body == *body);
            let Some(first_entry) = body_entries.next() else {
                panic!("reference snapshot should include packaged body {body}");
            };
            let mut earliest = first_entry.epoch;
            let mut latest = first_entry.epoch;

            for entry in body_entries {
                if entry.epoch.julian_day.days() < earliest.julian_day.days() {
                    earliest = entry.epoch;
                }
                if entry.epoch.julian_day.days() > latest.julian_day.days() {
                    latest = entry.epoch;
                }
            }

            body_bounds.insert(body.clone(), (earliest, latest));
        }

        for (body, (earliest, latest)) in body_bounds {
            for epoch in [earliest, latest] {
                let reference = reference_snapshot()
                    .iter()
                    .find(|entry| entry.body == body && entry.epoch == epoch)
                    .expect("reference snapshot should include the body's boundary epoch");
                let ecliptic = packaged_lookup(&body, epoch)
                    .expect("packaged lookup should succeed for reference boundary epochs");
                let expected = coordinates(reference);

                assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
                assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-8);
                assert!(
                    (ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-12
                );
            }
        }
    }

    #[test]
    fn packaged_backend_rejects_requests_outside_its_time_range() {
        let backend = packaged_backend();
        let time_range = packaged_artifact_production_profile_summary_details().time_range;
        let start = time_range
            .start
            .expect("packaged artifact should have a lower bound");
        let end = time_range
            .end
            .expect("packaged artifact should have an upper bound");

        for instant in [
            Instant::new(
                pleiades_backend::JulianDay::from_days(start.julian_day.days() - 1.0),
                start.scale,
            ),
            Instant::new(
                pleiades_backend::JulianDay::from_days(end.julian_day.days() + 1.0),
                end.scale,
            ),
        ] {
            let request = EphemerisRequest {
                body: CelestialBody::Sun,
                instant,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: pleiades_backend::Apparentness::Mean,
            };

            let error = backend
                .position(&request)
                .expect_err("packaged backend should reject out-of-range requests");

            assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
        }
    }

    #[test]
    fn observer_requests_are_rejected_explicitly() {
        let backend = packaged_backend();
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tdb,
            ),
            observer: Some(pleiades_backend::ObserverLocation::new(
                pleiades_backend::Latitude::from_degrees(51.5),
                pleiades_backend::Longitude::from_degrees(0.0),
                None,
            )),
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };

        let error = backend
            .position(&request)
            .expect_err("packaged data should reject topocentric requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    }

    #[test]
    fn batch_query_rejects_topocentric_requests_explicitly() {
        let backend = packaged_backend();
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tdb,
            ),
            observer: Some(pleiades_backend::ObserverLocation::new(
                pleiades_backend::Latitude::from_degrees(51.5),
                pleiades_backend::Longitude::from_degrees(0.0),
                None,
            )),
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };

        let error = backend
            .positions(&[request])
            .expect_err("packaged data should reject topocentric batch requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    }

    #[test]
    fn apparent_requests_are_rejected_explicitly() {
        let backend = packaged_backend();
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tdb,
            ),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Apparent,
        };

        let error = backend
            .position(&request)
            .expect_err("packaged data should reject apparent-place requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    }

    #[test]
    fn batch_query_rejects_apparent_requests_explicitly() {
        let backend = packaged_backend();
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tdb,
            ),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Apparent,
        };

        let error = backend
            .positions(&[request])
            .expect_err("packaged data should reject apparent batch requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    }

    #[test]
    fn backend_metadata_exposes_packaged_scope() {
        let metadata = packaged_backend().metadata();
        assert_eq!(metadata.id.as_str(), PACKAGE_NAME);
        assert_eq!(metadata.family, BackendFamily::CompressedData);
        assert_eq!(
            packaged_artifact().header.profile.stored_channels,
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu
            ]
        );
        assert_eq!(
            packaged_artifact().header.profile.speed_policy,
            pleiades_compression::SpeedPolicy::Unsupported
        );
        assert!(packaged_artifact()
            .header
            .profile
            .unsupported_outputs
            .contains(&pleiades_compression::ArtifactOutput::Motion));
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
        assert!(metadata.body_coverage.contains(&CelestialBody::Moon));
        assert!(metadata.body_coverage.contains(&CelestialBody::Jupiter));
        assert!(metadata.body_coverage.contains(&CelestialBody::Pluto));
        assert!(metadata
            .body_coverage
            .contains(&CelestialBody::Custom(CustomBodyId::new(
                "asteroid", "433-Eros",
            ))));
        assert!(metadata.provenance.data_sources[0].contains("11 bundled bodies"));
        assert!(metadata.provenance.data_sources[0].contains("asteroid:433-Eros"));
        assert_eq!(
            packaged_body_coverage_summary(),
            metadata.provenance.data_sources[0]
        );
        let request_policy = packaged_request_policy_summary_details();
        assert!(request_policy.validate().is_ok());
        assert!(request_policy.geocentric_only);
        assert_eq!(
            request_policy.supported_frames,
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial]
        );
        assert_eq!(
            request_policy.supported_time_scales,
            &[TimeScale::Tt, TimeScale::Tdb]
        );
        assert_eq!(
            request_policy.supported_zodiac_modes,
            &[ZodiacMode::Tropical]
        );
        assert_eq!(request_policy.supported_apparentness, &[Apparentness::Mean]);
        assert!(!request_policy.supports_topocentric_observer);
        assert_eq!(
            request_policy.lookup_epoch_policy,
            PackagedLookupEpochPolicy::RetagToTtGridWithoutRelativisticCorrection
        );
        assert_eq!(request_policy.lookup_epoch_policy.validate(), Ok(()));
        assert_eq!(
            request_policy.summary_line(),
            "Packaged request policy: geocentric-only; frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false; lookup epoch policy=TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
        );
        assert_eq!(
            request_policy.summary_line(),
            packaged_request_policy_summary_for_report()
        );
        assert_eq!(
            request_policy.summary_line(),
            packaged_request_policy_summary()
        );
        assert_eq!(request_policy.to_string(), request_policy.summary_line());
        let lookup_epoch_policy = packaged_lookup_epoch_policy_summary_details();
        assert_eq!(
            lookup_epoch_policy.policy,
            request_policy.lookup_epoch_policy
        );
        assert_eq!(lookup_epoch_policy.policy.validate(), Ok(()));
        assert_eq!(lookup_epoch_policy.validate(), Ok(()));
        assert_eq!(
            lookup_epoch_policy.summary_line(),
            "TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
        );
        assert_eq!(
            lookup_epoch_policy.summary_line(),
            packaged_lookup_epoch_policy_summary_for_report()
        );
        assert_eq!(
            lookup_epoch_policy.summary_line(),
            packaged_lookup_epoch_policy_summary()
        );
        assert_eq!(
            lookup_epoch_policy.to_string(),
            lookup_epoch_policy.summary_line()
        );
        assert_eq!(
            metadata.provenance.data_sources[1],
            request_policy.to_string()
        );
        assert_eq!(
            metadata.provenance.data_sources[2],
            packaged_frame_treatment_summary_details().summary_line()
        );
        assert_eq!(
            packaged_frame_treatment_summary_details().to_string(),
            packaged_frame_treatment_summary()
        );
        assert!(metadata.provenance.data_sources[2].contains("ecliptic coordinates directly"));
        assert_eq!(
            packaged_frame_treatment_summary_details().validate(),
            Ok(())
        );
        assert_eq!(
            packaged_frame_treatment_summary_details().validated_summary_line(),
            Ok(packaged_frame_treatment_summary_details().summary_line())
        );
        assert!(metadata.provenance.data_sources[2]
            .contains("equatorial coordinates are reconstructed"));
        assert_eq!(
            metadata.provenance.data_sources[3],
            packaged_artifact_storage_summary()
        );
        assert_eq!(
            packaged_artifact_storage_summary_details().to_string(),
            packaged_artifact_storage_summary()
        );
        assert!(metadata.provenance.data_sources[3].contains("Quantized linear segments"));
        assert!(metadata.provenance.data_sources[3]
            .contains("ecliptic and equatorial coordinates are reconstructed at runtime"));
        assert!(metadata.provenance.data_sources[3]
            .contains("apparent, topocentric, sidereal, and motion outputs remain unsupported"));
        assert_eq!(
            packaged_artifact_storage_summary_details().validate(),
            Ok(())
        );
        assert_eq!(
            metadata.provenance.data_sources[4],
            packaged_artifact_access_summary()
        );
        assert_eq!(
            packaged_artifact_access_summary_details().to_string(),
            packaged_artifact_access_summary()
        );
        assert!(metadata.provenance.data_sources[4].contains("checked-in fixture"));
        assert_eq!(
            packaged_artifact_access_summary_details().validate(),
            Ok(())
        );
    }

    #[test]
    fn packaged_request_policy_summary_validation_rejects_drift() {
        let mut summary = packaged_request_policy_summary_details();
        summary.supported_frames = &[CoordinateFrame::Ecliptic];

        let error = summary
            .validate()
            .expect_err("drifted packaged request-policy summary should be rejected");
        assert!(format!("{error}").contains("supported_frames"));
    }

    #[test]
    fn packaged_artifact_profile_summary_details_match_the_bundled_header() {
        let artifact = packaged_artifact();
        let summary = packaged_artifact_profile_summary_details();

        assert_eq!(summary.body_count, artifact.bodies.len());
        assert_eq!(
            summary.bodies,
            artifact
                .bodies
                .iter()
                .map(|series| series.body.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(summary.endian_policy, artifact.header.endian_policy);
        assert_eq!(summary.profile, artifact.header.profile);
        assert_eq!(
            summary.summary_line(),
            artifact
                .header
                .summary_for_body_count(artifact.bodies.len())
        );
        assert_eq!(
            summary.profile.summary_line(),
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
        );
        assert_eq!(summary.validate(), Ok(()));
        let coverage = summary.profile_coverage_summary();
        assert_eq!(coverage.body_count, artifact.bodies.len());
        assert_eq!(coverage.bodies, summary.bodies);
        assert_eq!(coverage.profile, summary.profile);
        assert_eq!(
            coverage.summary_line(),
            summary.profile.summary_for_body_count(summary.body_count)
        );
        assert_eq!(
            coverage.summary_line_with_bodies(),
            format!(
                "{}; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros",
                summary.profile.summary_for_body_count(summary.body_count)
            )
        );
        assert_eq!(coverage.to_string(), coverage.summary_line());
        coverage
            .validate()
            .expect("packaged profile coverage summary should validate");
        assert_eq!(summary.to_string(), summary.summary_line());
        summary
            .validate()
            .expect("packaged artifact profile summary should validate");
        assert_eq!(
            summary.summary_line_with_bodies(),
            format!(
                "{}; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros",
                artifact
                    .header
                    .summary_for_body_count(artifact.bodies.len())
            )
        );
        assert_eq!(
            packaged_artifact_profile_summary(),
            artifact
                .header
                .summary_for_body_count(artifact.bodies.len())
        );
        let output_support_summary = packaged_artifact_output_support_summary_details();
        assert_eq!(output_support_summary.profile, summary.profile);
        assert_eq!(
            output_support_summary.summary_line(),
            summary.profile.output_support_entries_summary_line()
        );
        output_support_summary
            .validate()
            .expect("packaged artifact output-support summary should validate");
        assert_eq!(
            output_support_summary.to_string(),
            output_support_summary.summary_line()
        );
        assert_eq!(
            packaged_artifact_output_support_summary_for_report(),
            summary.profile.output_support_entries_summary_line()
        );
        assert_eq!(
            summary.output_support_summary_line(),
            summary.profile.output_support_entries_summary_line()
        );
        assert_eq!(
            summary.summary_line_with_output_support(),
            format!(
                "{}; output support: {}",
                summary.summary_line_with_bodies(),
                summary.profile.output_support_entries_summary_line()
            )
        );
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.validated_summary_line_with_bodies(),
            Ok(summary.summary_line_with_bodies())
        );
        assert_eq!(
            summary.validated_summary_line_with_output_support(),
            Ok(summary.summary_line_with_output_support())
        );
        assert_eq!(
            packaged_artifact_profile_summary_with_body_coverage(),
            format!(
                "{}; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros",
                artifact
                    .header
                    .summary_for_body_count(artifact.bodies.len())
            )
        );
        assert_eq!(
            packaged_artifact_profile_coverage_summary_details(),
            summary.profile_coverage_summary()
        );
        assert_eq!(
            packaged_artifact_profile_coverage_summary_for_report(),
            summary
                .profile_coverage_summary()
                .summary_line_with_bodies()
        );
        assert_eq!(
            packaged_artifact_profile_summary_with_output_support(),
            summary.summary_line_with_output_support()
        );
        assert_eq!(
            packaged_artifact_profile_summary_with_output_support_for_report(),
            summary.summary_line_with_output_support()
        );
    }

    #[test]
    fn packaged_artifact_profile_summary_validation_rejects_body_count_drift() {
        let mut summary = packaged_artifact_profile_summary_details();
        summary.body_count += 1;

        let error = summary
            .validate()
            .expect_err("body-count drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact profile body count does not match bundled body list"));
    }

    #[test]
    fn packaged_artifact_profile_summary_validation_rejects_profile_drift() {
        let mut summary = packaged_artifact_profile_summary_details();
        summary.profile.derived_outputs.retain(|output| {
            *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates
        });

        let error = summary
            .validate()
            .expect_err("profile drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact profile metadata does not match the checked-in packaged artifact profile"));
    }

    #[test]
    fn packaged_artifact_profile_summary_validation_rejects_bundled_body_set_drift() {
        let mut bodies = packaged_bodies().to_vec();
        bodies[0] = CelestialBody::Ceres;

        let summary = PackagedArtifactProfileSummary {
            body_count: bodies.len(),
            bodies,
            endian_policy: EndianPolicy::LittleEndian,
            profile: ArtifactProfile::ecliptic_longitude_latitude_distance(),
        };

        let error = summary
            .validate()
            .expect_err("packaged body set drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact profile bundled body list does not match the checked-in packaged body set"));
    }

    #[test]
    fn packaged_artifact_profile_summary_validation_rejects_empty_bodies() {
        let summary = PackagedArtifactProfileSummary {
            body_count: 0,
            bodies: Vec::new(),
            endian_policy: EndianPolicy::LittleEndian,
            profile: ArtifactProfile::ecliptic_longitude_latitude_distance(),
        };

        let error = summary
            .validate()
            .expect_err("empty packaged body lists should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("artifact profile coverage bundled body list must not be empty"));
    }

    #[test]
    fn packaged_artifact_profile_summary_validation_rejects_duplicate_bodies() {
        let summary = PackagedArtifactProfileSummary {
            body_count: 3,
            bodies: vec![CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Sun],
            endian_policy: EndianPolicy::LittleEndian,
            profile: ArtifactProfile::ecliptic_longitude_latitude_distance(),
        };

        let error = summary
            .validate()
            .expect_err("duplicate packaged body lists should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("artifact profile coverage bundled bodies contains duplicate Sun entry"));
    }

    #[test]
    fn packaged_artifact_output_support_summary_validation_rejects_profile_drift() {
        let summary = PackagedArtifactOutputSupportSummary {
            profile: ArtifactProfile::new(
                vec![
                    ChannelKind::Longitude,
                    ChannelKind::Latitude,
                    ChannelKind::DistanceAu,
                ],
                vec![
                    pleiades_compression::ArtifactOutput::EclipticCoordinates,
                    pleiades_compression::ArtifactOutput::EquatorialCoordinates,
                ],
                vec![
                    pleiades_compression::ArtifactOutput::ApparentCorrections,
                    pleiades_compression::ArtifactOutput::TopocentricCoordinates,
                    pleiades_compression::ArtifactOutput::SiderealCoordinates,
                ],
                pleiades_compression::SpeedPolicy::Unsupported,
            ),
        };

        let error = summary
            .validate()
            .expect_err("profile drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains("Motion"));
    }

    #[test]
    fn packaged_artifact_output_support_summary_validation_rejects_equatorial_support_drift() {
        let mut profile = packaged_artifact_profile_summary_details().profile.clone();
        profile.derived_outputs.retain(|output| {
            *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates
        });

        let summary = PackagedArtifactOutputSupportSummary { profile };
        let error = summary
            .validate()
            .expect_err("equatorial output support drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains("EquatorialCoordinates"));
    }

    #[test]
    fn packaged_artifact_storage_summary_validation_rejects_profile_drift() {
        let mut profile = packaged_artifact_profile_summary_details().profile.clone();
        profile
            .stored_channels
            .retain(|channel| *channel != ChannelKind::DistanceAu);

        let error = validate_packaged_artifact_storage_profile(&profile)
            .expect_err("drifted packaged storage profile should be rejected");
        assert_eq!(
            error,
            PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
                field: "stored_channels"
            }
        );

        let mut profile = packaged_artifact_profile_summary_details().profile.clone();
        profile.derived_outputs.retain(|output| {
            *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates
        });

        let error = validate_packaged_artifact_storage_profile(&profile)
            .expect_err("drifted packaged storage profile should be rejected");
        assert_eq!(
            error,
            PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
                field: "derived_outputs"
            }
        );

        let mut profile = packaged_artifact_profile_summary_details().profile.clone();
        profile
            .unsupported_outputs
            .retain(|output| *output != pleiades_compression::ArtifactOutput::Motion);

        let error = validate_packaged_artifact_storage_profile(&profile)
            .expect_err("drifted packaged storage profile should be rejected");
        assert_eq!(
            error,
            PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
                field: "unsupported_outputs"
            }
        );
    }

    #[test]
    fn packaged_artifact_access_summary_matches_current_build_posture() {
        let summary = packaged_artifact_access_summary_details();
        assert_eq!(
            summary.explicit_path_loading,
            packaged_artifact_path_loading_enabled()
        );
        assert_eq!(summary.summary_line(), packaged_artifact_access_summary());
        assert_eq!(summary.to_string(), packaged_artifact_access_summary());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            packaged_artifact_access_summary_for_report(),
            summary.to_string()
        );
        summary
            .validate()
            .expect("packaged artifact access summary should validate");
    }

    #[test]
    fn packaged_artifact_output_support_summary_matches_current_build_posture() {
        let summary = packaged_artifact_output_support_summary_details();
        assert_eq!(
            summary.summary_line(),
            summary.profile.output_support_entries_summary_line()
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            packaged_artifact_output_support_summary_for_report(),
            summary.summary_line()
        );
        summary
            .validate()
            .expect("packaged artifact output-support summary should validate");
    }

    #[test]
    fn packaged_artifact_output_support_summary_validation_rejects_drift() {
        let mut summary = packaged_artifact_output_support_summary_details();
        summary
            .profile
            .derived_outputs
            .retain(|output| *output != ArtifactOutput::EquatorialCoordinates);

        assert!(summary.validated_summary_line().is_err());
        assert!(summary.validate().is_err());
    }

    #[test]
    fn packaged_artifact_access_summary_validation_rejects_drift() {
        let mut summary = packaged_artifact_access_summary_details();
        summary.explicit_path_loading = !summary.explicit_path_loading;

        let error = summary
            .validate()
            .expect_err("drifted packaged artifact access summary should be rejected");
        assert_eq!(
            error,
            PackagedArtifactAccessSummaryValidationError::FeatureStateOutOfSync {
                field: "explicit_path_loading"
            }
        );
    }

    #[test]
    fn packaged_artifact_profile_summary_report_marks_drift_as_unavailable() {
        let mut summary = packaged_artifact_profile_summary_details();
        summary.body_count += 1;

        assert_eq!(
            render_packaged_artifact_profile_summary(&summary, false),
            "Packaged artifact profile: unavailable (InvalidFormat: packaged artifact profile body count does not match bundled body list)"
        );
        assert_eq!(
            render_packaged_artifact_profile_summary(&summary, true),
            "Packaged artifact profile with bundled bodies: unavailable (InvalidFormat: packaged artifact profile body count does not match bundled body list)"
        );
    }

    #[test]
    fn packaged_artifact_generation_policy_summary_matches_current_posture() {
        let summary = packaged_artifact_generation_policy_summary_details();
        let artifact = packaged_artifact();
        assert_eq!(
            summary.policy,
            PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments
        );
        assert_eq!(summary.summary_line(), "adjacent same-body linear segments; bodies with a single sampled epoch use point segments; multi-epoch non-lunar bodies are fit with linear segments between adjacent same-body source epochs; the Moon uses overlapping three-point spans with quadratic residual corrections to keep the high-curvature fit compact");
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(artifact.residual_bodies(), vec![CelestialBody::Moon]);
        summary
            .validate()
            .expect("generation policy summary should validate");
        assert_eq!(
            packaged_artifact_generation_policy_summary(),
            summary.to_string()
        );
        let residual_bodies = packaged_artifact_generation_residual_bodies_summary_details();
        assert_eq!(residual_bodies.body_count, 1);
        assert_eq!(residual_bodies.bodies, vec![CelestialBody::Moon]);
        assert_eq!(residual_bodies.summary_line(), "residual bodies: Moon");
        assert_eq!(residual_bodies.to_string(), residual_bodies.summary_line());
        residual_bodies
            .validate(artifact)
            .expect("residual body coverage summary should validate");
        assert_eq!(
            packaged_artifact_generation_residual_bodies_summary_for_report(),
            "residual bodies: Moon; applies to 1 bundled body"
        );
    }

    #[test]
    fn packaged_artifact_generation_policy_summary_rejects_residual_body_drift() {
        let error = validate_packaged_artifact_generation_policy_residual_bodies(
            PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments,
            &[CelestialBody::Sun],
        )
        .expect_err("residual body drift should fail validation");
        assert_eq!(
            error,
            PackagedArtifactGenerationPolicySummaryValidationError::FieldOutOfSync {
                field: "residual_bodies",
            }
        );
        assert_eq!(
            error.summary_line(),
            "the packaged artifact generation policy summary field `residual_bodies` is out of sync with the current posture"
        );
        assert_eq!(error.to_string(), error.summary_line());
    }

    #[test]
    fn packaged_artifact_generation_policy_validation_error_has_summary_line() {
        let error =
            PackagedArtifactGenerationPolicyValidationError::FieldOutOfSync { field: "policy" };
        assert_eq!(
            error.summary_line(),
            "the packaged artifact generation policy field `policy` is out of sync with the current posture"
        );
        assert_eq!(error.to_string(), error.summary_line());
    }

    #[test]
    fn packaged_artifact_regeneration_summary_includes_reference_snapshot_coverage() {
        let summary = packaged_artifact_regeneration_summary_details();
        let artifact = packaged_artifact();
        assert_eq!(summary.label, ARTIFACT_LABEL);
        assert_eq!(summary.artifact_version, artifact.header.version);
        assert_eq!(summary.source, ARTIFACT_SOURCE);
        assert_eq!(
            summary.source_revision,
            production_generation_source_summary_for_report()
        );
        assert_eq!(summary.profile_id, ARTIFACT_PROFILE_ID);
        assert_eq!(summary.checksum, artifact.checksum);
        assert_eq!(
            summary.generation_policy,
            PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments
        );
        assert_eq!(summary.bodies.len(), packaged_bodies().len());
        assert_eq!(
            summary.quantization_scales,
            packaged_artifact_quantization_scales_line()
        );
        assert_eq!(
            summary.body_coverage_line(),
            "11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"
        );
        assert_eq!(
            summary.generation_policy_line(),
            "generation policy: adjacent same-body linear segments; bodies with a single sampled epoch use point segments; multi-epoch non-lunar bodies are fit with linear segments between adjacent same-body source epochs; the Moon uses overlapping three-point spans with quadratic residual corrections to keep the high-curvature fit compact"
        );
        assert_eq!(
            summary.residual_body_line(),
            "residual bodies: Moon; applies to 1 bundled body"
        );
        assert_eq!(summary.fit_envelope.body_count, packaged_bodies().len());
        assert_eq!(
            summary.fit_envelope.expected_sample_count,
            summary.fit_envelope.sample_count
        );
        summary
            .fit_envelope
            .validate()
            .expect("packaged fit envelope should validate");
        let residual_coverage = summary.residual_body_coverage_summary();
        assert_eq!(residual_coverage.body_count, 1);
        assert_eq!(residual_coverage.summary_line(), "residual bodies: Moon");
        assert_eq!(
            residual_coverage.to_string(),
            residual_coverage.summary_line()
        );
        residual_coverage
            .validate(artifact)
            .expect("residual body coverage should validate");
        assert_eq!(
            packaged_body_coverage_summary_details().summary_line(),
            format!("Packaged body set: {}", summary.body_coverage_line())
        );
        assert_eq!(
            packaged_body_coverage_summary_details().validated_summary_line(),
            Ok(packaged_body_coverage_summary_details().summary_line())
        );
        assert_eq!(
            packaged_body_coverage_summary(),
            packaged_body_coverage_summary_details().to_string()
        );

        let provenance = summary.summary_line();
        assert_eq!(summary.to_string(), provenance);
        assert_eq!(summary.validated_summary_line(), Ok(provenance.clone()));
        summary
            .validate()
            .expect("packaged regeneration summary should validate");
        assert!(provenance
            .contains("Packaged artifact regeneration source: label=stage-5 packaged-data draft"));
        assert!(provenance.contains("profile id=pleiades-packaged-artifact-profile/stage-5-draft"));
        assert!(provenance.contains("source revision=Production generation source:"));
        assert!(provenance.contains("checksum=0x"));
        assert!(provenance.contains("generation policy: adjacent same-body linear segments"));
        assert!(provenance
            .contains("quantization scales: stored=Longitude=9, Latitude=9, DistanceAu=12"));
        assert!(provenance.contains("residual bodies: Moon; applies to 1 bundled body"));
        assert!(provenance.contains(&format!("artifact version={}", artifact.header.version)));
        assert!(provenance.contains("11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"));
        assert!(provenance.contains("Reference snapshot coverage:"));
        assert!(provenance.contains("fit envelope:"));
        assert!(provenance.contains("segment samples across"));
        assert!(provenance.contains("rows across"));
        assert!(provenance.contains("asteroid rows"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_profile_id_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.profile_id = "pleiades-packaged-artifact-profile/test-drift";

        let error = summary
            .validate()
            .expect_err("profile id drift should be rejected");

        assert!(error
            .to_string()
            .contains("packaged artifact regeneration summary profile id does not match the checked-in artifact profile id"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_source_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.source = "drifted source";

        let error = summary
            .validate()
            .expect_err("source drift should be rejected");

        assert!(error
            .to_string()
            .contains("packaged artifact regeneration summary source does not match the checked-in artifact source"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_source_revision_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.source_revision = "drifted source revision".to_string();

        let error = summary
            .validate()
            .expect_err("source revision drift should be rejected");

        assert!(error
            .to_string()
            .contains("packaged artifact regeneration summary source revision does not match the checked-in production-generation source summary"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_checksum_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.checksum ^= 1;

        let error = summary
            .validate()
            .expect_err("checksum drift should be rejected");

        assert!(error
            .to_string()
            .contains("packaged artifact regeneration summary checksum"));

        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.quantization_scales = "quantization scales: stored=Longitude=10".to_string();
        let error = summary
            .validate()
            .expect_err("quantization scale drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary quantization scales do not match the checked-in packaged artifact"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_fit_envelope_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.fit_envelope.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("fit envelope drift should be rejected");

        assert!(error
            .to_string()
            .contains("packaged artifact regeneration fit envelope is invalid"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validated_summary_line_rejects_metadata_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.artifact_version += 1;

        let error = summary
            .validated_summary_line()
            .expect_err("metadata drift should be rejected");

        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary artifact version"));
    }

    #[test]
    fn packaged_frame_treatment_summary_reuses_the_structured_report_helper() {
        let summary = PackagedFrameTreatmentSummary;

        assert_eq!(summary.summary_line(), packaged_frame_treatment_summary());
        assert_eq!(summary.to_string(), packaged_frame_treatment_summary());
        assert_eq!(
            packaged_frame_treatment_summary_for_report(),
            summary.to_string()
        );
        assert_eq!(summary.validate(), Ok(()));
    }

    #[test]
    fn packaged_frame_treatment_summary_rejects_whitespace_padded_summary_text() {
        let summary = format!(" {} ", PackagedFrameTreatmentSummary.summary_line());

        assert_eq!(
            validate_packaged_frame_treatment_summary_line(&summary),
            Err(PackagedFrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
        );
    }

    #[test]
    fn packaged_artifact_storage_summary_rejects_whitespace_padded_summary_text() {
        let summary = format!(" {} ", PackagedArtifactStorageSummary.summary_line());

        assert_eq!(
            validate_packaged_artifact_storage_summary_line(&summary),
            Err(PackagedArtifactStorageSummaryValidationError::WhitespacePaddedSummary)
        );
    }

    #[test]
    fn packaged_artifact_storage_summary_rejects_blank_summary_text() {
        assert_eq!(
            validate_packaged_artifact_storage_summary_line(""),
            Err(PackagedArtifactStorageSummaryValidationError::BlankSummary)
        );
    }

    #[test]
    fn packaged_artifact_access_summary_rejects_whitespace_padded_summary_text() {
        let summary = format!(
            " {} ",
            PackagedArtifactAccessSummary {
                explicit_path_loading: cfg!(feature = "packaged-artifact-path"),
            }
            .summary_line()
        );

        assert_eq!(
            validate_packaged_artifact_access_summary_line(&summary),
            Err(PackagedArtifactAccessSummaryValidationError::WhitespacePaddedSummary)
        );
    }

    #[test]
    fn packaged_artifact_access_summary_rejects_blank_summary_text() {
        assert_eq!(
            validate_packaged_artifact_access_summary_line(""),
            Err(PackagedArtifactAccessSummaryValidationError::BlankSummary)
        );
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_duplicate_bodies() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.bodies[1] = summary.bodies[0].clone();

        let error = summary
            .validate()
            .expect_err("duplicate regeneration bodies should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary contains duplicate body entry"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_body_list_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.bodies.swap(0, 1);

        let error = summary
            .validate()
            .expect_err("body order drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary body list does not match the checked-in packaged body set"));
        assert!(error.message.contains("expected [Sun, Moon"));
        assert!(error.message.contains("got [Moon, Sun"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_residual_body_subset_drift() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary
            .validate_residual_body_subset()
            .expect("current residual body coverage should stay within the bundled body list");

        summary
            .residual_bodies
            .push(CelestialBody::Custom(CustomBodyId::new(
                "catalog",
                "designation",
            )));

        let error = summary
            .validate_residual_body_subset()
            .expect_err("residual bodies outside the bundled body list should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary residual body catalog:designation is not covered by the bundled body list"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_metadata_drift() {
        let expected_artifact = packaged_artifact();
        let mut summary = packaged_artifact_regeneration_summary_details();

        summary.label = "drifted label";
        let error = summary
            .validate()
            .expect_err("label drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary label does not match the checked-in artifact label"));

        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.source = "drifted source";
        let error = summary
            .validate()
            .expect_err("source drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary source does not match the checked-in artifact source"));

        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.artifact_version = expected_artifact.header.version + 1;
        let error = summary
            .validate()
            .expect_err("artifact version drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary artifact version"));
        assert!(error
            .message
            .contains("does not match the checked-in packaged artifact version"));

        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.checksum ^= 0x1;
        let error = summary
            .validate()
            .expect_err("checksum drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary checksum 0x"));
        assert!(error
            .message
            .contains("does not match the checked-in packaged artifact checksum 0x"));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_missing_reference_snapshot() {
        let mut summary = packaged_artifact_regeneration_summary_details();
        summary.reference_snapshot = None;

        let error = summary
            .validate()
            .expect_err("missing reference snapshot should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact regeneration summary is missing reference snapshot coverage"
        ));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_reflects_the_current_posture() {
        let summary = packaged_artifact_production_profile_summary_details();
        let artifact = packaged_artifact();

        assert_eq!(summary.profile_id, ARTIFACT_PROFILE_ID);
        assert_eq!(summary.label, ARTIFACT_LABEL);
        assert_eq!(summary.artifact_version, artifact.header.version);
        assert_eq!(summary.time_range, artifact_time_range(artifact));
        assert_eq!(
            summary.body_coverage,
            packaged_body_coverage_summary_details()
        );
        assert_eq!(
            summary.artifact_profile,
            packaged_artifact_profile_summary_details().profile
        );
        assert_eq!(summary.speed_policy, summary.artifact_profile.speed_policy);
        assert_eq!(
            summary.generation_policy,
            PackagedArtifactGenerationPolicy::AdjacentSameBodyLinearSegments
        );
        assert_eq!(
            summary.request_policy,
            packaged_request_policy_summary_details()
        );
        assert_eq!(
            summary.lookup_epoch_policy,
            packaged_lookup_epoch_policy_summary_details().policy
        );
        assert_eq!(
            summary.frame_treatment,
            packaged_frame_treatment_summary_details()
        );
        assert!(summary.summary_line().contains(
            "lookup epoch policy=TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
        ));
        assert_eq!(
            summary.storage_summary,
            packaged_artifact_storage_summary_details()
        );
        assert_eq!(
            summary.target_thresholds,
            packaged_artifact_target_threshold_summary_details()
        );
        assert_eq!(
            summary.target_thresholds.fit_envelope,
            packaged_artifact_fit_envelope_summary_details()
        );
        assert_eq!(
            summary.target_thresholds.scope_envelopes,
            packaged_artifact_target_threshold_scope_envelopes_summary_details()
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        summary
            .validate()
            .expect("production-profile skeleton should validate");
        assert!(summary
            .summary_line()
            .contains("Packaged artifact production profile draft:"));
        assert!(summary
            .summary_line()
            .contains("source provenance=Production generation source:"));
        assert!(summary.summary_line().contains("output support="));
        assert!(summary.summary_line().contains("speed policy=Unsupported"));
        assert!(summary
            .summary_line()
            .contains("segment strategy=bodies with a single sampled epoch use point segments"));
        assert!(summary
            .summary_line()
            .contains("target thresholds: draft fit envelope recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
        assert!(summary
            .summary_line()
            .contains("scope envelopes=scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
        assert!(
            packaged_artifact_target_threshold_scope_envelopes_for_report()
                .contains("scope=luminaries; bodies=2 (Sun, Moon); fit envelope:")
        );
        assert!(packaged_artifact_production_profile_summary_for_report()
            .contains("Packaged artifact production profile draft:"));
        assert_eq!(
            packaged_artifact_production_profile_summary(),
            summary.summary_line()
        );
    }

    #[test]
    fn packaged_artifact_speed_policy_summary_reflects_the_current_posture() {
        let summary = packaged_artifact_speed_policy_summary_details();
        let artifact = packaged_artifact();

        assert_eq!(summary.policy, artifact.header.profile.speed_policy);
        assert_eq!(summary.policy, SpeedPolicy::Unsupported);
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Unsupported; motion output support=unsupported"
        );
        assert_eq!(
            packaged_artifact_speed_policy_summary_for_report(),
            summary.summary_line()
        );
        summary
            .validate()
            .expect("packaged-artifact speed policy should validate");

        let mut drifted = summary;
        drifted.policy = SpeedPolicy::Stored;
        let error = drifted
            .validate()
            .expect_err("speed-policy drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactSpeedPolicySummaryValidationError::FieldOutOfSync { field: "policy" }
        );
        assert!(error
            .to_string()
            .contains("speed-policy summary field `policy`"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_time_range_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.time_range = TimeRange::new(None, None);

        let error = summary
            .validate()
            .expect_err("time-range drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "time_range"
            }
        );
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_source_provenance_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.source_provenance = "drifted source provenance".to_string();

        let error = summary
            .validate()
            .expect_err("source-provenance drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "source_provenance"
            }
        );
        assert!(error.to_string().contains("source_provenance"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_request_policy_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.request_policy.supports_topocentric_observer = true;

        let error = summary
            .validate()
            .expect_err("request-policy drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "request_policy"
            }
        );
        assert!(error.to_string().contains("request_policy"));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_profile_id_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.profile_id = "pleiades-packaged-artifact-profile/test-drift";

        let error = parameters
            .validate()
            .expect_err("profile id drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters profile id does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_label_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.label = "drifted label";

        let error = parameters
            .validate()
            .expect_err("label drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters label does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_body_coverage_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.body_coverage.bodies[0] = CelestialBody::Ceres;

        let error = parameters
            .validate()
            .expect_err("body coverage drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact generator parameters body coverage does not match the current production profile"));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_time_range_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.time_range = TimeRange::new(None, None);

        let error = parameters
            .validate()
            .expect_err("time range drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact generator parameters time range does not match the current production profile"));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_artifact_version_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.artifact_version += 1;

        let error = parameters
            .validate()
            .expect_err("artifact version drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters version does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_source_provenance_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.source_provenance = "drifted source provenance".to_string();

        let error = parameters
            .validate()
            .expect_err("source-provenance drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters source provenance does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_artifact_profile_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.artifact_profile.speed_policy = pleiades_compression::SpeedPolicy::Stored;

        let error = parameters
            .validate()
            .expect_err("artifact profile drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters artifact profile does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_speed_policy_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.speed_policy = pleiades_compression::SpeedPolicy::Stored;

        let error = parameters
            .validate()
            .expect_err("speed policy drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters speed policy does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_request_policy_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.request_policy.supports_topocentric_observer = true;

        let error = parameters
            .validate()
            .expect_err("request policy drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters request policy does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generator_parameters_validation_rejects_target_threshold_drift() {
        let mut parameters = packaged_artifact_generator_parameters_details();
        parameters.target_thresholds.status = "drifted";

        let error = parameters
            .validate()
            .expect_err("target threshold drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters target thresholds do not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_request_policy_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest
            .parameters
            .request_policy
            .supports_topocentric_observer = true;

        let error = manifest
            .validate()
            .expect_err("request policy drift should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters request policy does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generation_manifest_reflects_the_current_posture() {
        let manifest = packaged_artifact_generation_manifest_details();
        let parameters = packaged_artifact_generator_parameters_details();
        let regeneration = packaged_artifact_regeneration_summary_details();

        assert_eq!(manifest.parameters, parameters);
        assert_eq!(manifest.regeneration, regeneration);
        assert_eq!(
            parameters.speed_policy,
            parameters.artifact_profile.speed_policy
        );
        assert_eq!(manifest.to_string(), manifest.summary_line());
        assert_eq!(
            manifest.validated_summary_line(),
            Ok(manifest.summary_line())
        );
        manifest
            .validate()
            .expect("generation manifest should validate");
        assert!(manifest
            .summary_line()
            .contains("Packaged artifact generation manifest:"));
        assert!(manifest.summary_line().contains("output support="));
        assert!(manifest.summary_line().contains("speed policy=Unsupported"));
        assert!(manifest.summary_line().contains("segment strategy="));
        assert!(manifest
            .summary_line()
            .contains("source revision=Production generation source:"));
        assert!(manifest.summary_line().contains("regeneration="));
        assert!(packaged_artifact_generation_manifest_for_report()
            .contains("Packaged artifact generation manifest:"));
        assert_eq!(
            packaged_artifact_generation_manifest(),
            manifest.summary_line()
        );
    }

    #[test]
    fn packaged_artifact_generation_artifacts_keep_lookup_epoch_and_segment_strategy_aligned() {
        let production_profile = packaged_artifact_production_profile_summary_details();
        let generator_parameters = packaged_artifact_generator_parameters_details();
        let manifest = packaged_artifact_generation_manifest_details();

        assert_eq!(
            production_profile.lookup_epoch_policy,
            generator_parameters.lookup_epoch_policy
        );
        assert_eq!(
            generator_parameters.lookup_epoch_policy,
            manifest.parameters.lookup_epoch_policy
        );
        assert_eq!(
            production_profile.lookup_epoch_policy.summary_line(),
            generator_parameters.lookup_epoch_policy.summary_line()
        );
        assert_eq!(
            generator_parameters.generation_policy.segment_strategy(),
            manifest.parameters.generation_policy.segment_strategy()
        );
        assert_eq!(
            production_profile.generation_policy.segment_strategy(),
            generator_parameters.generation_policy.segment_strategy()
        );
        assert!(production_profile
            .summary_line()
            .contains("source provenance=Production generation source:"));
        assert!(production_profile
            .summary_line()
            .contains("lookup epoch policy=TT-grid retag without relativistic correction"));
        assert!(manifest
            .summary_line()
            .contains("source provenance=Production generation source:"));
        assert!(manifest
            .summary_line()
            .contains("segment strategy=bodies with a single sampled epoch use point segments"));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_profile_id_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.parameters.profile_id = "pleiades-packaged-artifact-profile/test-drift";

        let error = manifest
            .validate()
            .expect_err("drifted generation parameters should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters profile id does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_label_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.parameters.label = "drifted label";

        let error = manifest
            .validate()
            .expect_err("drifted generation label should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters label does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_artifact_profile_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.parameters.artifact_profile.speed_policy =
            pleiades_compression::SpeedPolicy::Stored;

        let error = manifest
            .validate()
            .expect_err("drifted generation artifact profile should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters artifact profile does not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_source_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.regeneration.source = "drifted source";

        let error = manifest
            .validate()
            .expect_err("drifted regeneration source should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact regeneration summary source does not match the checked-in artifact source"
        ));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_artifact_version_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.regeneration.artifact_version += 1;

        let error = manifest
            .validate()
            .expect_err("drifted regeneration artifact version should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration summary artifact version"));
        assert!(error
            .message
            .contains("does not match the checked-in packaged artifact version"));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_parameter_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.parameters.target_thresholds = PackagedArtifactTargetThresholdSummary {
            profile_id: ARTIFACT_PROFILE_ID,
            status: "drifted",
            scopes: &["luminaries"],
            fit_envelope: packaged_artifact_fit_envelope_summary_details(),
            scope_envelopes: packaged_artifact_target_threshold_scope_envelopes_summary_details(),
        };

        let error = manifest
            .validate()
            .expect_err("drifted generation parameters should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error.message.contains(
            "packaged artifact generator parameters target thresholds do not match the current production profile"
        ));
    }

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_regeneration_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.regeneration.fit_envelope.sample_count += 1;

        let error = manifest
            .validate()
            .expect_err("drifted regeneration metadata should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration fit envelope is invalid"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_label_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.label = "drifted label";

        let error = summary
            .validate()
            .expect_err("label drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "label",
            }
        );
        assert!(error.to_string().contains("label"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_artifact_version_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.artifact_version += 1;

        let error = summary
            .validate()
            .expect_err("artifact version drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "artifact_version",
            }
        );
        assert!(error.to_string().contains("artifact_version"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_artifact_profile_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.artifact_profile.speed_policy = pleiades_compression::SpeedPolicy::Stored;

        let error = summary
            .validate()
            .expect_err("artifact profile drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "artifact_profile",
            }
        );
        assert!(error.to_string().contains("artifact_profile"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_speed_policy_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.speed_policy = pleiades_compression::SpeedPolicy::Stored;

        let error = summary
            .validate()
            .expect_err("speed policy drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "speed_policy",
            }
        );
        assert!(error.to_string().contains("speed_policy"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_stored_channel_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary
            .artifact_profile
            .stored_channels
            .retain(|channel| *channel != ChannelKind::DistanceAu);

        let error = summary
            .validate()
            .expect_err("stored channel drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "artifact_profile",
            }
        );
        assert!(error.to_string().contains("artifact_profile"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_body_coverage_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.body_coverage.body_count += 1;

        let error = summary
            .validate()
            .expect_err("body coverage drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "body_coverage",
            }
        );
        assert!(error.to_string().contains("body_coverage"));
    }

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_target_threshold_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.target_thresholds = PackagedArtifactTargetThresholdSummary {
            profile_id: ARTIFACT_PROFILE_ID,
            status: "drifted",
            scopes: &["luminaries"],
            fit_envelope: packaged_artifact_fit_envelope_summary_details(),
            scope_envelopes: packaged_artifact_target_threshold_scope_envelopes_summary_details(),
        };

        let error = summary
            .validate()
            .expect_err("target threshold drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "target_thresholds",
            }
        );
        assert!(error.to_string().contains("target_thresholds"));
    }

    #[test]
    fn packaged_artifact_fit_threshold_summary_reflects_the_current_posture() {
        let summary = packaged_artifact_fit_threshold_summary_details();

        assert_eq!(
            summary.summary_line(),
            "fit thresholds: mean Δlon≤29.750992955013°, mean Δlat≤22.784650147073°, mean Δdist≤70908.319854514601 AU; max Δlon≤179.935747101401°, max Δlat≤5436.377507814662°, max Δdist≤19941928.384904474020 AU"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.validate().is_ok());
        assert!(packaged_artifact_fit_threshold_summary_for_report()
            .contains("fit thresholds: mean Δlon≤29.750992955013°"));
    }

    #[test]
    fn packaged_artifact_fit_margin_summary_reflects_the_current_posture() {
        let envelope = packaged_artifact_fit_envelope_summary_details();
        let thresholds = packaged_artifact_fit_threshold_summary_details();

        assert_eq!(
            packaged_artifact_fit_margin_summary_for_report(),
            format!(
                "fit margins: mean Δlon={:+.12}°, mean Δlat={:+.12}°, mean Δdist={:+.12} AU; max Δlon={:+.12}°, max Δlat={:+.12}°, max Δdist={:+.12} AU",
                thresholds.max_mean_longitude_delta_degrees - envelope.mean_longitude_delta_degrees,
                thresholds.max_mean_latitude_delta_degrees - envelope.mean_latitude_delta_degrees,
                thresholds.max_mean_distance_delta_au - envelope.mean_distance_delta_au,
                thresholds.max_longitude_delta_degrees - envelope.max_longitude_delta_degrees,
                thresholds.max_latitude_delta_degrees - envelope.max_latitude_delta_degrees,
                thresholds.max_distance_delta_au - envelope.max_distance_delta_au,
            )
        );
    }

    #[test]
    fn packaged_artifact_target_threshold_summary_reflects_the_current_posture() {
        let summary = packaged_artifact_target_threshold_summary_details();

        assert_eq!(summary.profile_id, ARTIFACT_PROFILE_ID);
        assert_eq!(summary.status, PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATUS);
        assert_eq!(summary.scopes, PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES);
        assert_eq!(
            summary.scope_envelopes,
            packaged_artifact_target_threshold_scope_envelopes_summary_details()
        );
        assert_eq!(summary.summary_line(), format!("profile id={}; target thresholds: draft fit envelope recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; {}; scope envelopes={}", ARTIFACT_PROFILE_ID, summary.fit_envelope.summary_line(), join_display(&summary.scope_envelopes)));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.validate().is_ok());
        assert!(
            packaged_artifact_target_threshold_summary_for_report().contains(&format!(
                "profile id={}; target thresholds: draft fit envelope recorded",
                ARTIFACT_PROFILE_ID
            ))
        );
    }

    #[test]
    fn packaged_artifact_fit_envelope_validation_rejects_threshold_drift() {
        let summary = packaged_artifact_fit_envelope_summary_details();
        let thresholds = PackagedArtifactFitThresholdSummary {
            max_mean_longitude_delta_degrees: summary.mean_longitude_delta_degrees - 1.0,
            max_mean_latitude_delta_degrees: summary.mean_latitude_delta_degrees - 1.0,
            max_mean_distance_delta_au: summary.mean_distance_delta_au - 1.0,
            max_longitude_delta_degrees: summary.max_longitude_delta_degrees - 1.0,
            max_latitude_delta_degrees: summary.max_latitude_delta_degrees - 1.0,
            max_distance_delta_au: summary.max_distance_delta_au - 1.0,
        };

        let error = summary
            .validate_against_thresholds(&thresholds)
            .expect_err("threshold drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded {
                field: "mean_longitude_delta_degrees",
                measured_bits: summary.mean_longitude_delta_degrees.to_bits(),
                threshold_bits: thresholds.max_mean_longitude_delta_degrees.to_bits(),
            }
        );
        assert!(error.summary_line().contains("measured="));
        assert!(error.summary_line().contains("threshold="));
    }

    #[test]
    fn packaged_artifact_target_threshold_scope_summary_validation_rejects_drift() {
        let mut scope_summary =
            packaged_artifact_target_threshold_scope_envelopes_summary_details()[0].clone();
        scope_summary.body_count += 1;

        let error = scope_summary
            .validate()
            .expect_err("scope envelope drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                field: "scope fit envelope"
            }
        );
    }

    #[test]
    fn packaged_artifact_target_threshold_scope_envelopes_reflect_the_current_posture() {
        let summary = packaged_artifact_target_threshold_summary_details();
        let expected = format!(
            "scope envelopes: {}",
            join_display(&summary.scope_envelopes)
        );

        assert_eq!(
            packaged_artifact_target_threshold_scope_envelopes_for_report(),
            expected
        );
        assert_eq!(
            summary.scope_envelopes[0].validated_summary_line(),
            Ok(summary.scope_envelopes[0].summary_line())
        );
        assert!(expected.contains("scope=luminaries; bodies="));
        assert!(expected.contains("scope=major planets; bodies="));
        assert!(expected.contains("scope=pluto; bodies=1 (Pluto); fit envelope:"));
        assert!(expected.contains("scope=lunar points; bodies="));
        assert!(expected.contains("scope=selected asteroids; bodies="));
        assert!(expected.contains("scope=custom bodies; bodies="));
    }

    #[test]
    fn packaged_artifact_regeneration_summary_validation_rejects_invalid_reference_snapshot_coverage(
    ) {
        let mut summary = packaged_artifact_regeneration_summary_details();
        let mut reference_snapshot = summary
            .reference_snapshot
            .expect("packaged regeneration summary should include reference snapshot coverage");
        reference_snapshot.body_count += 1;
        summary.reference_snapshot = Some(reference_snapshot);

        let error = summary
            .validate()
            .expect_err("invalid reference snapshot coverage should be rejected");
        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::InvalidFormat
        );
        assert!(error
            .message
            .contains("packaged artifact regeneration reference snapshot is invalid"));
        assert!(error
            .message
            .contains("body count 17 does not match body list length 16"));
    }

    #[test]
    fn packaged_tdb_batch_requests_match_tt_grid_lookups() {
        let backend = packaged_backend();
        let tt_request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };
        let tdb_request = EphemerisRequest {
            instant: Instant::new(tt_request.instant.julian_day, TimeScale::Tdb),
            ..tt_request.clone()
        };

        let tt_results = backend
            .positions(std::slice::from_ref(&tt_request))
            .expect("TT requests should succeed through the batch path");
        let tdb_results = backend
            .positions(std::slice::from_ref(&tdb_request))
            .expect("TDB requests should succeed through the batch path");

        assert_eq!(tt_results.len(), 1);
        assert_eq!(tdb_results.len(), 1);

        let tt_result = &tt_results[0];
        let tdb_result = &tdb_results[0];
        let tt_single_result = backend
            .position(&tt_request)
            .expect("TT requests should succeed through the single-request path");
        let tdb_single_result = backend
            .position(&tdb_request)
            .expect("TDB requests should succeed through the single-request path");

        assert_eq!(tt_result.instant.scale, TimeScale::Tt);
        assert_eq!(tdb_result.instant.scale, TimeScale::Tdb);
        assert_eq!(tt_result.quality, QualityAnnotation::Interpolated);
        assert_eq!(tdb_result.quality, QualityAnnotation::Interpolated);
        assert_eq!(tt_result.ecliptic, tt_single_result.ecliptic);
        assert_eq!(tdb_result.ecliptic, tdb_single_result.ecliptic);
        assert_eq!(tt_result.backend_id, tt_single_result.backend_id);
        assert_eq!(tdb_result.backend_id, tdb_single_result.backend_id);
        assert_eq!(tt_result.body, tt_single_result.body);
        assert_eq!(tdb_result.body, tdb_single_result.body);
        assert_eq!(tt_result.apparent, tt_single_result.apparent);
        assert_eq!(tdb_result.apparent, tdb_single_result.apparent);
    }

    #[test]
    fn packaged_mixed_frame_batch_requests_preserve_request_frames() {
        let backend = packaged_backend();
        let requests = packaged_mixed_frame_batch_parity_requests()
            .expect("packaged mixed frame batch parity requests should be available");
        let alias_requests = packaged_mixed_frame_batch_parity_request_corpus()
            .expect("packaged mixed frame batch parity request corpus should be available");

        assert_eq!(requests, alias_requests);
        assert_eq!(requests.len(), packaged_bodies().len());
        assert!(requests.iter().enumerate().all(|(index, request)| {
            matches!(
                (index % 2, request.frame),
                (0, CoordinateFrame::Ecliptic) | (1, CoordinateFrame::Equatorial)
            )
        }));

        let batch_results = backend
            .positions(&requests)
            .expect("mixed frame requests should succeed through the batch path");
        assert_eq!(batch_results.len(), requests.len());

        for (request, result) in requests.iter().zip(batch_results.iter()) {
            let single = backend
                .position(request)
                .expect("mixed frame requests should succeed through the single-request path");

            assert_eq!(result.frame, request.frame);
            assert_eq!(result.quality, QualityAnnotation::Interpolated);
            assert_eq!(result, &single);
        }
    }

    #[test]
    fn packaged_mixed_frame_batch_parity_summary_is_release_facing() {
        let summary = packaged_mixed_frame_batch_parity_summary()
            .expect("packaged mixed frame batch parity should be available");

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.validate().is_ok());
        assert_eq!(summary.request_count, packaged_bodies().len());
        assert_eq!(summary.body_count, packaged_bodies().len());
        assert_eq!(
            summary.ecliptic_request_count + summary.equatorial_request_count,
            summary.request_count
        );
        assert!(summary.order_preserved);
        assert!(summary.single_query_parity_preserved);
        assert!(summary
            .summary_line()
            .contains("Packaged mixed frame batch parity:"));
        assert!(packaged_mixed_frame_batch_parity_summary_for_report()
            .contains("Packaged mixed frame batch parity:"));
    }

    #[test]
    fn packaged_mixed_frame_batch_parity_summary_report_marks_drift_as_unavailable() {
        let mut summary = packaged_mixed_frame_batch_parity_summary()
            .expect("packaged mixed frame batch parity should be available");
        summary.request_count += 1;

        assert_eq!(
            summary.validated_summary_line(),
            Err(PackagedBatchParitySummaryValidationError::FieldOutOfSync {
                field: "request_count/body_count",
            })
        );
        assert_eq!(
            format_validated_packaged_mixed_frame_batch_parity_summary_for_report(&summary),
            "Packaged mixed frame batch parity: unavailable (the packaged mixed-frame batch-parity summary field `request_count/body_count` is out of sync with the current packaged posture)"
        );
    }

    #[test]
    fn packaged_mixed_frame_batch_parity_summary_report_marks_frame_mix_drift_as_unavailable() {
        let mut summary = packaged_mixed_frame_batch_parity_summary()
            .expect("packaged mixed frame batch parity should be available");
        summary.ecliptic_request_count = summary.request_count;
        summary.equatorial_request_count = 0;

        assert_eq!(
            format_validated_packaged_mixed_frame_batch_parity_summary_for_report(&summary),
            "Packaged mixed frame batch parity: unavailable (the packaged mixed-frame batch-parity summary field `frame_mix` is out of sync with the current packaged posture)"
        );
    }

    #[test]
    fn packaged_mixed_frame_batch_parity_summary_report_marks_order_drift_as_unavailable() {
        let mut summary = packaged_mixed_frame_batch_parity_summary()
            .expect("packaged mixed frame batch parity should be available");
        summary.order_preserved = false;

        assert_eq!(
            format_validated_packaged_mixed_frame_batch_parity_summary_for_report(&summary),
            "Packaged mixed frame batch parity: unavailable (the packaged mixed-frame batch-parity summary field `order_preserved` is out of sync with the current packaged posture)"
        );
    }

    #[test]
    fn packaged_mixed_tt_tdb_batch_requests_preserve_request_scales() {
        let backend = packaged_backend();
        let requests = packaged_mixed_tt_tdb_batch_parity_requests()
            .expect("packaged mixed TT/TDB batch parity requests should be available");
        let alias_requests = packaged_mixed_tt_tdb_batch_parity_request_corpus()
            .expect("packaged mixed TT/TDB batch parity request corpus should be available");

        assert_eq!(requests, alias_requests);
        assert_eq!(requests.len(), packaged_bodies().len());
        assert!(requests.iter().enumerate().all(|(index, request)| {
            matches!(
                (index % 2, request.instant.scale),
                (0, TimeScale::Tt) | (1, TimeScale::Tdb)
            )
        }));

        let batch_results = backend
            .positions(&requests)
            .expect("mixed TT/TDB requests should succeed through the batch path");
        assert_eq!(batch_results.len(), requests.len());

        for (request, result) in requests.iter().zip(batch_results.iter()) {
            let single = backend
                .position(request)
                .expect("mixed TT/TDB requests should succeed through the single-request path");

            assert_eq!(result.instant.scale, request.instant.scale);
            assert_eq!(result.quality, QualityAnnotation::Interpolated);
            assert_eq!(result, &single);
        }
    }

    #[test]
    fn packaged_mixed_tt_tdb_batch_parity_summary_is_release_facing() {
        let summary = packaged_mixed_tt_tdb_batch_parity_summary()
            .expect("packaged mixed TT/TDB batch parity should be available");

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.validate().is_ok());
        assert_eq!(summary.request_count, packaged_bodies().len());
        assert_eq!(summary.body_count, packaged_bodies().len());
        assert_eq!(
            summary.tt_request_count + summary.tdb_request_count,
            summary.request_count
        );
        assert!(summary.order_preserved);
        assert!(summary.single_query_parity_preserved);
        assert!(summary
            .summary_line()
            .contains("Packaged mixed TT/TDB batch parity:"));
        assert!(packaged_mixed_tt_tdb_batch_parity_summary_for_report()
            .contains("Packaged mixed TT/TDB batch parity:"));
    }

    #[test]
    fn packaged_mixed_tt_tdb_batch_parity_summary_report_marks_drift_as_unavailable() {
        let mut summary = packaged_mixed_tt_tdb_batch_parity_summary()
            .expect("packaged mixed TT/TDB batch parity should be available");
        summary.request_count += 1;

        assert_eq!(
            summary.validated_summary_line(),
            Err(
                PackagedTimeScaleBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "request_count/body_count",
                }
            )
        );
        assert_eq!(
            format_validated_packaged_mixed_tt_tdb_batch_parity_summary_for_report(&summary),
            "Packaged mixed TT/TDB batch parity: unavailable (the packaged mixed TT/TDB batch-parity summary field `request_count/body_count` is out of sync with the current packaged posture)"
        );
    }

    #[test]
    fn packaged_mixed_tt_tdb_batch_parity_summary_report_marks_time_scale_mix_drift_as_unavailable()
    {
        let mut summary = packaged_mixed_tt_tdb_batch_parity_summary()
            .expect("packaged mixed TT/TDB batch parity should be available");
        summary.tt_request_count = summary.request_count;
        summary.tdb_request_count = 0;

        assert_eq!(
            format_validated_packaged_mixed_tt_tdb_batch_parity_summary_for_report(&summary),
            "Packaged mixed TT/TDB batch parity: unavailable (the packaged mixed TT/TDB batch-parity summary field `time_scale_mix` is out of sync with the current packaged posture)"
        );
    }

    #[test]
    fn packaged_mixed_tt_tdb_batch_parity_summary_report_marks_parity_drift_as_unavailable() {
        let mut summary = packaged_mixed_tt_tdb_batch_parity_summary()
            .expect("packaged mixed TT/TDB batch parity should be available");
        summary.single_query_parity_preserved = false;

        assert_eq!(
            format_validated_packaged_mixed_tt_tdb_batch_parity_summary_for_report(&summary),
            "Packaged mixed TT/TDB batch parity: unavailable (the packaged mixed TT/TDB batch-parity summary field `single_query_parity_preserved` is out of sync with the current packaged posture)"
        );
    }

    #[test]
    fn packaged_tdb_single_requests_match_tt_grid_lookups() {
        let backend = packaged_backend();
        let tt_request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };
        let tdb_request = EphemerisRequest {
            instant: Instant::new(tt_request.instant.julian_day, TimeScale::Tdb),
            ..tt_request.clone()
        };

        let tt_result = backend
            .position(&tt_request)
            .expect("TT requests should succeed through the single-request path");
        let tdb_result = backend
            .position(&tdb_request)
            .expect("TDB requests should succeed through the single-request path");

        assert_eq!(tt_result.instant.scale, TimeScale::Tt);
        assert_eq!(tdb_result.instant.scale, TimeScale::Tdb);
        assert_eq!(tt_result.quality, QualityAnnotation::Interpolated);
        assert_eq!(tdb_result.quality, QualityAnnotation::Interpolated);
        assert_eq!(tt_result.ecliptic, tdb_result.ecliptic);
        assert_eq!(tt_result.backend_id, tdb_result.backend_id);
        assert_eq!(tt_result.body, tdb_result.body);
        assert_eq!(tt_result.apparent, tdb_result.apparent);
    }

    #[test]
    fn packaged_body_coverage_summary_matches_the_packaged_body_set() {
        let summary = packaged_body_coverage_summary_details();
        assert_eq!(summary.body_count, packaged_bodies().len());
        assert_eq!(summary.bodies, packaged_bodies().to_vec());
        assert_eq!(
            summary.summary_line(),
            "Packaged body set: 11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"
        );
        assert_eq!(packaged_body_coverage_summary(), summary.to_string());
    }

    #[test]
    fn packaged_body_coverage_summary_validation_rejects_body_count_drift() {
        let mut summary = packaged_body_coverage_summary_details();
        summary.body_count += 1;

        let error = summary
            .validate()
            .expect_err("body-count drift should be rejected");
        assert_eq!(
            error,
            PackagedBodyCoverageSummaryValidationError::FieldOutOfSync {
                field: "body_count",
            }
        );
        assert_eq!(
            error.to_string(),
            "the packaged body coverage summary field `body_count` is out of sync with the current bundled body set"
        );
    }

    #[test]
    fn packaged_body_coverage_summary_validated_summary_line_rejects_body_drift() {
        let mut summary = packaged_body_coverage_summary_details();
        summary.bodies.swap(0, 1);

        let error = summary
            .validated_summary_line()
            .expect_err("body-order drift should be rejected");
        assert_eq!(
            error,
            PackagedBodyCoverageSummaryValidationError::FieldOutOfSync { field: "bodies" }
        );
        assert_eq!(
            error.to_string(),
            "the packaged body coverage summary field `bodies` is out of sync with the current bundled body set"
        );
    }

    #[test]
    fn packaged_body_coverage_summary_report_marks_drift_as_unavailable() {
        let mut summary = packaged_body_coverage_summary_details();
        summary.bodies.swap(0, 1);

        assert_eq!(
            format_validated_packaged_body_coverage_summary_for_report(&summary),
            "Packaged body set: unavailable (the packaged body coverage summary field `bodies` is out of sync with the current bundled body set)"
        );
    }
}
