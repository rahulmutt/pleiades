//! Validation, comparison, and benchmarking helpers for the workspace.
//!
//! The validation crate compares the algorithmic chart backends against the
//! checked-in JPL Horizons snapshot corpus and renders reproducible reports for
//! stage-4 work.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant as StdInstant;

mod artifact;
mod chart_benchmark;
mod house_validation;

pub use artifact::{
    artifact_boundary_envelope_summary_for_report, artifact_inspection_summary_for_report,
    render_artifact_report, render_artifact_summary, ArtifactBodyInspection,
    ArtifactDecodeBenchmarkReport, ArtifactDecodeBenchmarkReportValidationError,
    ArtifactInspectionReport, ArtifactLookupBenchmarkReport,
    ArtifactLookupBenchmarkReportValidationError,
};
pub use chart_benchmark::{
    benchmark_chart_backend, chart_benchmark_corpus_summary, ChartBenchmarkReport,
};
pub use house_validation::{
    house_validation_report, house_validation_summary_line_for_report, HouseValidationReport,
    HouseValidationReportValidationError, HouseValidationSample, HouseValidationScenario,
};

use pleiades_ayanamsa::{
    ayanamsa_catalog_validation_summary, baseline_ayanamsas, built_in_ayanamsas, descriptor,
    metadata_coverage, release_ayanamsas, resolve_ayanamsa, validate_ayanamsa_catalog,
};
use pleiades_backend::{
    delta_t_policy_summary_for_report, frame_policy_summary_details,
    frame_policy_summary_for_report, request_policy_summary_for_report,
    time_scale_policy_summary_for_report, zodiac_policy_summary_for_report,
};
use pleiades_core::{
    current_api_stability_profile, current_compatibility_profile,
    current_release_profile_identifiers, default_chart_bodies, validate_custom_definition_labels,
    AccuracyClass, Angle, Apparentness, BackendCapabilities, BackendFamily, BackendMetadata,
    CelestialBody, CompatibilityProfile, CompositeBackend, CoordinateFrame, EclipticCoordinates,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    Instant, JulianDay, Longitude, ReleaseProfileIdentifiers, TimeRange, TimeScale, ZodiacMode,
};
use pleiades_data::{
    packaged_artifact_access_summary_for_report, packaged_artifact_bytes,
    packaged_artifact_generation_manifest_for_report,
    packaged_artifact_generation_policy_summary_for_report,
    packaged_artifact_generation_residual_bodies_summary_for_report,
    packaged_artifact_output_support_summary_for_report,
    packaged_artifact_production_profile_summary_for_report,
    packaged_artifact_profile_coverage_summary_for_report,
    packaged_artifact_profile_summary_with_body_coverage,
    packaged_artifact_regeneration_summary_for_report,
    packaged_artifact_storage_summary_for_report,
    packaged_artifact_target_threshold_scope_envelopes_for_report,
    packaged_artifact_target_threshold_summary_for_report,
    packaged_frame_parity_summary_for_report, packaged_frame_treatment_summary_for_report,
    packaged_lookup_epoch_policy_summary_for_report,
    packaged_mixed_tt_tdb_batch_parity_summary_for_report,
    packaged_request_policy_summary_for_report, regenerate_packaged_artifact, PackagedDataBackend,
};
use pleiades_elp::{
    lunar_apparent_comparison_evidence, lunar_apparent_comparison_summary,
    lunar_apparent_comparison_summary_for_report,
    lunar_equatorial_reference_batch_parity_summary_for_report,
    lunar_equatorial_reference_evidence, lunar_equatorial_reference_evidence_envelope_for_report,
    lunar_equatorial_reference_evidence_summary,
    lunar_equatorial_reference_evidence_summary_for_report,
    lunar_high_curvature_continuity_evidence_for_report,
    lunar_high_curvature_equatorial_continuity_evidence_for_report,
    lunar_reference_batch_parity_summary_for_report, lunar_reference_evidence,
    lunar_reference_evidence_envelope_for_report, lunar_reference_evidence_summary,
    lunar_reference_evidence_summary_for_report, lunar_source_window_summary_for_report,
    lunar_theory_capability_summary_for_report, lunar_theory_catalog_summary_for_report,
    lunar_theory_catalog_validation_summary_for_report,
    lunar_theory_frame_treatment_summary_for_report, lunar_theory_request_policy_summary,
    lunar_theory_source_summary_for_report, lunar_theory_specification,
    lunar_theory_summary_for_report, ElpBackend,
};
use pleiades_houses::{
    baseline_house_systems, built_in_house_systems, house_system_code_aliases_summary_line,
    release_house_systems, resolve_house_system, validate_house_catalog,
    validate_house_system_code_aliases,
};
use pleiades_jpl::{
    comparison_snapshot_batch_parity_summary_for_report,
    comparison_snapshot_body_class_coverage_summary_for_report,
    comparison_snapshot_manifest_summary_for_report, comparison_snapshot_requests,
    comparison_snapshot_source_summary_for_report,
    comparison_snapshot_source_window_summary_for_report, comparison_snapshot_summary_for_report,
    format_jpl_interpolation_quality_summary_for_report,
    frame_treatment_summary_for_report as jpl_frame_treatment_summary_for_report,
    independent_holdout_manifest_summary_for_report,
    independent_holdout_snapshot_batch_parity_summary_for_report as jpl_independent_holdout_snapshot_batch_parity_summary_for_report,
    independent_holdout_snapshot_body_class_coverage_summary_for_report,
    independent_holdout_snapshot_equatorial_parity_summary_for_report as jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report,
    independent_holdout_snapshot_source_window_summary_for_report,
    independent_holdout_source_summary_for_report, interpolation_quality_samples,
    jpl_independent_holdout_summary_for_report, jpl_interpolation_posture_summary,
    jpl_interpolation_posture_summary_for_report,
    jpl_interpolation_quality_kind_coverage_for_report,
    jpl_snapshot_batch_error_taxonomy_summary_for_report, jpl_snapshot_evidence_summary_for_report,
    jpl_snapshot_request_policy_summary_for_report,
    production_generation_boundary_body_class_coverage_summary_for_report,
    production_generation_boundary_request_corpus_summary_for_report,
    production_generation_boundary_source_summary_for_report,
    production_generation_boundary_summary_for_report,
    production_generation_boundary_window_summary_for_report,
    production_generation_snapshot_body_class_coverage_summary_for_report,
    production_generation_snapshot_summary_for_report,
    production_generation_snapshot_window_summary_for_report,
    production_generation_source_summary_for_report,
    reference_asteroid_equatorial_evidence_summary_for_report, reference_asteroid_evidence,
    reference_asteroid_evidence_summary_for_report,
    reference_asteroid_source_window_summary_for_report, reference_asteroids,
    reference_holdout_overlap_summary_for_report,
    reference_snapshot_1749_major_body_boundary_summary_for_report,
    reference_snapshot_1800_major_body_boundary_summary_for_report,
    reference_snapshot_2500_major_body_boundary_summary_for_report,
    reference_snapshot_batch_parity_summary_for_report,
    reference_snapshot_body_class_coverage_summary_for_report,
    reference_snapshot_boundary_epoch_coverage_summary_for_report,
    reference_snapshot_early_major_body_boundary_summary_for_report,
    reference_snapshot_equatorial_parity_summary_for_report,
    reference_snapshot_high_curvature_summary_for_report,
    reference_snapshot_high_curvature_window_summary_for_report,
    reference_snapshot_lunar_boundary_summary_for_report,
    reference_snapshot_major_body_boundary_summary_for_report,
    reference_snapshot_major_body_boundary_window_summary_for_report,
    reference_snapshot_manifest_summary_for_report,
    reference_snapshot_mars_jupiter_boundary_summary_for_report,
    reference_snapshot_mars_outer_boundary_summary_for_report,
    reference_snapshot_source_summary_for_report,
    reference_snapshot_source_window_summary_for_report, reference_snapshot_summary_for_report,
    selected_asteroid_batch_parity_summary_for_report,
    selected_asteroid_boundary_summary_for_report,
    selected_asteroid_source_evidence_summary_for_report,
    selected_asteroid_source_window_summary_for_report,
    selected_asteroid_terminal_boundary_summary_for_report, JplSnapshotBackend,
};
use pleiades_vsop87::{
    body_source_profiles, canonical_epoch_equatorial_body_class_evidence_summary_for_report,
    canonical_epoch_equatorial_evidence_summary_for_report,
    canonical_epoch_evidence_summary_for_report, canonical_epoch_outlier_note_for_report,
    canonical_j1900_batch_parity_summary_for_report,
    canonical_j2000_batch_parity_summary_for_report,
    canonical_mixed_time_scale_batch_parity_summary_for_report, frame_treatment_summary_for_report,
    generated_binary_audit_summary_for_report, source_audit_summary_for_report, source_audits,
    source_body_class_evidence_summary_for_report, source_body_evidence_summary_for_report,
    source_documentation_health_summary_for_report, source_documentation_summary_for_report,
    source_specifications, supported_body_j1900_ecliptic_batch_parity_summary_for_report,
    supported_body_j1900_equatorial_batch_parity_summary_for_report,
    supported_body_j2000_ecliptic_batch_parity_summary_for_report,
    supported_body_j2000_equatorial_batch_parity_summary_for_report,
    vsop87_request_policy_summary_for_report, Vsop87Backend,
};

const DEFAULT_BENCHMARK_ROUNDS: usize = 10_000;
const SUMMARY_BENCHMARK_ROUNDS: usize = 1;
const BANNER: &str = "pleiades-validate stage 4 tool";
const LUMINARY_LONGITUDE_THRESHOLD_DEG: f64 = 7.5;
const LUMINARY_LATITUDE_THRESHOLD_DEG: f64 = 0.75;
const LUMINARY_DISTANCE_THRESHOLD_AU: f64 = 0.001;
const MAJOR_PLANET_LONGITUDE_THRESHOLD_DEG: f64 = 0.01;
const MAJOR_PLANET_LATITUDE_THRESHOLD_DEG: f64 = 0.01;
const MAJOR_PLANET_DISTANCE_THRESHOLD_AU: f64 = 0.001;
const LUNAR_POINT_LONGITUDE_THRESHOLD_DEG: f64 = 0.1;
const LUNAR_POINT_LATITUDE_THRESHOLD_DEG: f64 = 0.01;
const LUNAR_POINT_DISTANCE_THRESHOLD_AU: f64 = 0.001;
const ASTEROID_LONGITUDE_THRESHOLD_DEG: f64 = 0.25;
const ASTEROID_LATITUDE_THRESHOLD_DEG: f64 = 0.05;
const ASTEROID_DISTANCE_THRESHOLD_AU: f64 = 0.01;
const CUSTOM_LONGITUDE_THRESHOLD_DEG: f64 = 0.25;
const CUSTOM_LATITUDE_THRESHOLD_DEG: f64 = 0.05;
const CUSTOM_DISTANCE_THRESHOLD_AU: f64 = 0.01;
const PLUTO_LONGITUDE_THRESHOLD_DEG: f64 = 45.0;
const PLUTO_LATITUDE_THRESHOLD_DEG: f64 = 1.0;
const PLUTO_DISTANCE_THRESHOLD_AU: f64 = 0.25;

/// A validation corpus made up of request samples.
#[derive(Clone, Debug)]
pub struct ValidationCorpus {
    /// Human-readable corpus name.
    pub name: String,
    /// Short description of what the corpus covers.
    pub description: &'static str,
    /// Apparentness mode used for the requests.
    pub apparentness: Apparentness,
    /// Requests sent to both backends.
    pub requests: Vec<EphemerisRequest>,
}

/// A compact summary of a validation corpus.
#[derive(Clone, Debug)]
pub struct CorpusSummary {
    /// Human-readable corpus name.
    pub name: String,
    /// Short description of what the corpus covers.
    pub description: &'static str,
    /// Apparentness mode used for the corpus requests.
    pub apparentness: Apparentness,
    /// Total number of requests in the corpus.
    pub request_count: usize,
    /// Number of unique instants covered by the corpus.
    pub epoch_count: usize,
    /// Unique instants covered by the corpus, preserved in chronological order.
    pub epochs: Vec<Instant>,
    /// Number of unique bodies covered by the corpus.
    pub body_count: usize,
    /// Earliest Julian day in the corpus.
    pub earliest_julian_day: f64,
    /// Latest Julian day in the corpus.
    pub latest_julian_day: f64,
}

impl CorpusSummary {
    /// Returns a compact one-line summary used by release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "corpus name={} apparentness={} requests={} epochs={} bodies={} julian day span={:.1} → {:.1}",
            self.name,
            self.apparentness,
            self.request_count,
            self.epoch_count,
            self.body_count,
            self.earliest_julian_day,
            self.latest_julian_day,
        )
    }

    /// Returns `Ok(())` when the corpus summary is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary name must not be blank",
            ));
        }

        if self.description.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary description must not be blank",
            ));
        }

        if self.request_count == 0 {
            if self.epoch_count != 0 || self.body_count != 0 || !self.epochs.is_empty() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "corpus summary with no requests must also have no epochs or bodies",
                ));
            }
        } else if self.epoch_count == 0 || self.body_count == 0 || self.epochs.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary with requests must also have epochs and bodies",
            ));
        }

        if self.epoch_count != self.epochs.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "corpus summary epoch-count mismatch: expected {}, found {}",
                    self.epoch_count,
                    self.epochs.len()
                ),
            ));
        }

        for (index, epoch) in self.epochs.iter().enumerate() {
            if !epoch.julian_day.days().is_finite() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "corpus summary epoch {} has a non-finite Julian day",
                        index + 1
                    ),
                ));
            }

            if self.epochs[..index]
                .iter()
                .any(|prior| prior.julian_day.days() >= epoch.julian_day.days())
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "corpus summary epochs must be strictly increasing: epoch {} is out of order",
                        index + 1
                    ),
                ));
            }
        }

        match self.epochs.first() {
            Some(first) => {
                if self.earliest_julian_day != first.julian_day.days() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        "corpus summary earliest Julian day does not match the first epoch",
                    ));
                }
            }
            None => {
                if self.earliest_julian_day != 0.0 || self.latest_julian_day != 0.0 {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        "corpus summary with no epochs must have a zero Julian day span",
                    ));
                }
            }
        }

        if let Some(last) = self.epochs.last() {
            if self.latest_julian_day != last.julian_day.days() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "corpus summary latest Julian day does not match the final epoch",
                ));
            }
        }

        if self.request_count > 0 && self.body_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary with requests must cover at least one body",
            ));
        }

        Ok(())
    }
}

impl fmt::Display for CorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl ValidationCorpus {
    /// Creates the default JPL snapshot corpus.
    pub fn jpl_snapshot() -> Self {
        let requests = comparison_snapshot_requests(CoordinateFrame::Ecliptic)
            .expect("comparison snapshot requests should exist");

        Self {
            name: "JPL Horizons comparison window".to_string(),
            description: "Source-backed comparison corpus built from the checked-in JPL Horizons snapshot across a small set of reference epochs, restricted to the bodies shared by the algorithmic comparison backend.",
            apparentness: Apparentness::Mean,
            requests,
        }
    }

    /// Creates a representative benchmark corpus spanning the target 1500-2500 window.
    pub fn representative_window() -> Self {
        let bodies = default_chart_bodies();
        let instants = [
            Instant::new(JulianDay::from_days(2_268_559.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_268_924.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_305_448.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_329_555.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_390_550.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_512_176.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_573_171.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_597_642.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_634_532.0), TimeScale::Tt),
        ];

        Self::from_epochs(
            "Representative 1500-2500 window",
            "Eleven-epoch benchmark corpus that broadens the representative sweep with explicit guard epochs just outside the target span and mid-window coverage.",
            Apparentness::Mean,
            &instants,
            bodies,
        )
    }

    /// Returns a compact metadata summary for display purposes.
    pub fn summary(&self) -> CorpusSummary {
        let mut epochs = self
            .requests
            .iter()
            .map(|request| request.instant)
            .collect::<Vec<_>>();
        epochs.sort_by(|left, right| {
            left.julian_day
                .days()
                .partial_cmp(&right.julian_day.days())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        epochs.dedup();

        let mut bodies = Vec::new();
        for request in &self.requests {
            if !bodies.contains(&request.body) {
                bodies.push(request.body.clone());
            }
        }

        let earliest_julian_day = epochs
            .first()
            .map(|instant| instant.julian_day.days())
            .unwrap_or_default();
        let latest_julian_day = epochs
            .last()
            .map(|instant| instant.julian_day.days())
            .unwrap_or_default();

        CorpusSummary {
            name: self.name.clone(),
            description: self.description,
            apparentness: self.apparentness,
            request_count: self.requests.len(),
            epoch_count: epochs.len(),
            epochs,
            body_count: bodies.len(),
            earliest_julian_day,
            latest_julian_day,
        }
    }

    /// Returns an estimated heap footprint for the corpus data in bytes.
    pub fn estimated_heap_bytes(&self) -> usize {
        self.requests
            .capacity()
            .saturating_mul(std::mem::size_of::<EphemerisRequest>())
            .saturating_add(self.name.capacity())
    }

    fn from_epochs(
        name: impl Into<String>,
        description: &'static str,
        apparentness: Apparentness,
        instants: &[Instant],
        bodies: &[CelestialBody],
    ) -> Self {
        let requests = instants
            .iter()
            .copied()
            .flat_map(|instant| {
                bodies.iter().cloned().map(move |body| EphemerisRequest {
                    body,
                    instant,
                    observer: None,
                    frame: CoordinateFrame::Ecliptic,
                    zodiac_mode: ZodiacMode::Tropical,
                    apparent: apparentness,
                })
            })
            .collect();

        Self {
            name: name.into(),
            description,
            apparentness,
            requests,
        }
    }
}

/// A single comparison sample.
#[derive(Clone, Debug)]
pub struct ComparisonSample {
    /// Body queried for this sample.
    pub body: CelestialBody,
    /// Reference result.
    pub reference: EclipticCoordinates,
    /// Candidate result.
    pub candidate: EclipticCoordinates,
    /// Absolute longitude delta in degrees.
    pub longitude_delta_deg: f64,
    /// Absolute latitude delta in degrees.
    pub latitude_delta_deg: f64,
    /// Absolute distance delta in astronomical units.
    pub distance_delta_au: Option<f64>,
}

/// Summary statistics for a comparison run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComparisonSummary {
    /// Number of samples compared.
    pub sample_count: usize,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute longitude delta.
    pub max_longitude_delta_deg: f64,
    /// Mean absolute longitude delta.
    pub mean_longitude_delta_deg: f64,
    /// Root-mean-square absolute longitude delta.
    pub rms_longitude_delta_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute latitude delta.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta.
    pub mean_latitude_delta_deg: f64,
    /// Root-mean-square absolute latitude delta.
    pub rms_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Maximum absolute distance delta.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta.
    pub mean_distance_delta_au: Option<f64>,
    /// Root-mean-square absolute distance delta.
    pub rms_distance_delta_au: Option<f64>,
}

impl ComparisonSummary {
    /// Returns the compact release-facing summary line for the aggregate comparison envelope.
    pub fn summary_line(&self) -> String {
        format!(
            "samples: {}, max longitude delta: {:.12}°{}, mean longitude delta: {:.12}°, rms longitude delta: {:.12}°, max latitude delta: {:.12}°{}, mean latitude delta: {:.12}°, rms latitude delta: {:.12}°, max distance delta: {}{}, mean distance delta: {}, rms distance delta: {}",
            self.sample_count,
            self.max_longitude_delta_deg,
            self.max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_longitude_delta_deg,
            self.rms_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_latitude_delta_deg,
            self.rms_latitude_delta_deg,
            self.max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.rms_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
        )
    }

    /// Returns the compact comparison summary line after validating the aggregate envelope.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the aggregate envelope is finite and self-consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.sample_count == 0 {
            if self.max_longitude_delta_body.is_some()
                || self.max_latitude_delta_body.is_some()
                || self.max_distance_delta_body.is_some()
                || self.max_distance_delta_au.is_some()
                || self.mean_distance_delta_au.is_some()
                || self.rms_distance_delta_au.is_some()
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison summary with zero samples must not carry per-body or distance extrema",
                ));
            }

            for (label, value) in [
                ("max_longitude_delta_deg", self.max_longitude_delta_deg),
                ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
                ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
                ("max_latitude_delta_deg", self.max_latitude_delta_deg),
                ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
                ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
            ] {
                if value != 0.0 {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!("comparison summary field `{label}` must be zero when there are no samples"),
                    ));
                }
            }
        } else if self.max_longitude_delta_body.is_none() || self.max_latitude_delta_body.is_none()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison summary with samples must name the bodies that drive the longitude and latitude maxima",
            ));
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison summary field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        match (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_au,
            self.mean_distance_delta_au,
            self.rms_distance_delta_au,
        ) {
            (None, None, None, None) => {}
            (Some(_), Some(max), Some(mean), Some(rms)) => {
                for (label, value) in [
                    ("max_distance_delta_au", max),
                    ("mean_distance_delta_au", mean),
                    ("rms_distance_delta_au", rms),
                ] {
                    if !value.is_finite() || value.is_sign_negative() {
                        return Err(EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            format!("comparison summary field `{label}` must be a finite non-negative value"),
                        ));
                    }
                }
            }
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison summary distance metrics must either all be present or all be absent",
                ));
            }
        }

        Ok(())
    }
}

impl fmt::Display for ComparisonSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Summary statistics for the comparison-audit gate over a comparison run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComparisonAuditSummary {
    /// Number of compared bodies.
    pub body_count: usize,
    /// Number of bodies whose measured deltas stayed within tolerance.
    pub within_tolerance_body_count: usize,
    /// Number of bodies whose measured deltas exceeded tolerance.
    pub outside_tolerance_body_count: usize,
    /// Number of notable regressions in the comparison corpus.
    pub regression_count: usize,
}

impl ComparisonAuditSummary {
    fn status_label(&self) -> &'static str {
        if self.regression_count == 0 {
            "clean"
        } else {
            "regressions found"
        }
    }

    /// Returns the compact release-facing comparison-audit summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "status={}, bodies checked={}, within tolerance bodies={}, outside tolerance bodies={}, notable regressions={}",
            self.status_label(),
            self.body_count,
            self.within_tolerance_body_count,
            self.outside_tolerance_body_count,
            self.regression_count,
        )
    }

    /// Returns the compact comparison-audit summary line after validating the
    /// counted bodies.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the comparison-audit counts before the summary is rendered or reused.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison audit summary must include at least one compared body",
            ));
        }

        let within_and_outside = self
            .within_tolerance_body_count
            .saturating_add(self.outside_tolerance_body_count);

        if within_and_outside != self.body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison audit summary body-count mismatch: expected {}, found within tolerance {} + outside tolerance {} = {}",
                    self.body_count,
                    self.within_tolerance_body_count,
                    self.outside_tolerance_body_count,
                    within_and_outside
                ),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for ComparisonAuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Summary statistics for a single body within a comparison run.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyComparisonSummary {
    /// Body queried for this summary.
    pub body: CelestialBody,
    /// Number of samples compared for this body.
    pub sample_count: usize,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute longitude delta.
    pub max_longitude_delta_deg: f64,
    /// Mean absolute longitude delta.
    pub mean_longitude_delta_deg: f64,
    /// Root-mean-square absolute longitude delta.
    pub rms_longitude_delta_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute latitude delta.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta.
    pub mean_latitude_delta_deg: f64,
    /// Root-mean-square absolute latitude delta.
    pub rms_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Maximum absolute distance delta.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta.
    pub mean_distance_delta_au: Option<f64>,
    /// Root-mean-square absolute distance delta.
    pub rms_distance_delta_au: Option<f64>,
}

impl BodyComparisonSummary {
    /// Returns the compact release-facing summary line for one body.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, rms Δdist={}",
            self.body,
            self.sample_count,
            self.max_longitude_delta_deg,
            self.max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_longitude_delta_deg,
            self.rms_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_latitude_delta_deg,
            self.rms_latitude_delta_deg,
            self.max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.rms_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }

    /// Returns the compact body-specific summary line after validating the envelope.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the body-specific envelope is finite and self-consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body comparison summary for {} field `{label}` must be a finite non-negative value",
                        self.body
                    ),
                ));
            }
        }

        match self.sample_count {
            0 => {
                if self.max_longitude_delta_body.is_some()
                    || self.max_latitude_delta_body.is_some()
                    || self.max_distance_delta_body.is_some()
                    || self.max_distance_delta_au.is_some()
                    || self.mean_distance_delta_au.is_some()
                    || self.rms_distance_delta_au.is_some()
                {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "body comparison summary for {} with zero samples must not carry per-body or distance extrema",
                            self.body
                        ),
                    ));
                }
            }
            _ => {
                if self.max_longitude_delta_body.as_ref() != Some(&self.body)
                    || self.max_latitude_delta_body.as_ref() != Some(&self.body)
                {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "body comparison summary for {} must name that body as the longitude and latitude extrema driver",
                            self.body
                        ),
                    ));
                }

                match (
                    self.max_distance_delta_body.as_ref(),
                    self.max_distance_delta_au,
                    self.mean_distance_delta_au,
                    self.rms_distance_delta_au,
                ) {
                    (None, None, None, None) => {}
                    (Some(body), Some(max), Some(mean), Some(rms)) => {
                        if body != &self.body {
                            return Err(EphemerisError::new(
                                EphemerisErrorKind::InvalidRequest,
                                format!(
                                    "body comparison summary for {} must name that body as the distance extrema driver",
                                    self.body
                                ),
                            ));
                        }

                        for (label, value) in [
                            ("max_distance_delta_au", max),
                            ("mean_distance_delta_au", mean),
                            ("rms_distance_delta_au", rms),
                        ] {
                            if !value.is_finite() || value.is_sign_negative() {
                                return Err(EphemerisError::new(
                                    EphemerisErrorKind::InvalidRequest,
                                    format!(
                                        "body comparison summary for {} field `{label}` must be a finite non-negative value",
                                        self.body
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            format!(
                                "body comparison summary for {} distance metrics must either all be present or all be absent",
                                self.body
                            ),
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for BodyComparisonSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Expected comparison tolerance for a body or body class.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonTolerance {
    /// Backend family this tolerance profile is currently scoped to.
    pub backend_family: BackendFamily,
    /// Human-readable tolerance profile label.
    pub profile: &'static str,
    /// Maximum accepted absolute longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Maximum accepted absolute latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Maximum accepted absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
}

fn validate_comparison_tolerance_profile(profile: &str) -> Result<(), EphemerisError> {
    if profile.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison tolerance profile must not be blank",
        ));
    }

    if has_surrounding_whitespace(profile) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "comparison tolerance profile '{}' contains surrounding whitespace",
                profile
            ),
        ));
    }

    Ok(())
}

fn validate_comparison_tolerance(tolerance: &ComparisonTolerance) -> Result<(), EphemerisError> {
    validate_comparison_tolerance_profile(tolerance.profile)?;

    for (label, value) in [
        ("longitude", tolerance.max_longitude_delta_deg),
        ("latitude", tolerance.max_latitude_delta_deg),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance '{}' has invalid {} limit {}",
                    tolerance.profile, label, value
                ),
            ));
        }
    }

    if let Some(value) = tolerance.max_distance_delta_au {
        if !value.is_finite() || value < 0.0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance '{}' has invalid distance limit {}",
                    tolerance.profile, value
                ),
            ));
        }
    }

    Ok(())
}

/// Expected tolerance scope used by the validation catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ComparisonToleranceScope {
    /// Luminary body-class scope.
    Luminary,
    /// Major-planet body-class scope.
    MajorPlanet,
    /// Lunar-point body-class scope.
    LunarPoint,
    /// Asteroid body-class scope.
    Asteroid,
    /// Custom-body body-class scope.
    Custom,
    /// Pluto-specific approximate fallback scope.
    Pluto,
}

impl ComparisonToleranceScope {
    const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminaries",
            Self::MajorPlanet => "Major planets",
            Self::LunarPoint => "Lunar points",
            Self::Asteroid => "Asteroids",
            Self::Custom => "Custom bodies",
            Self::Pluto => "Pluto fallback (approximate)",
        }
    }
}

/// One expected tolerance entry in the validation catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonToleranceEntry {
    /// Scope covered by this tolerance entry.
    pub scope: ComparisonToleranceScope,
    /// Expected tolerance for the scope.
    pub tolerance: ComparisonTolerance,
}

impl ComparisonToleranceEntry {
    /// Validates the tolerance entry before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        validate_comparison_tolerance(&self.tolerance)
    }

    /// Renders the compact report wording for this tolerance entry.
    pub fn summary_line(&self) -> String {
        let tolerance = &self.tolerance;
        format!(
            "{}: Δlon≤{:.3}°, Δlat≤{:.3}°, Δdist={}",
            self.scope.label(),
            tolerance.max_longitude_delta_deg,
            tolerance.max_latitude_delta_deg,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.3} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }

    /// Returns the compact summary line after validating the entry.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonToleranceEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured comparison tolerance policy details for a validation report.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonTolerancePolicySummary {
    /// Backend family used to select the comparison tolerance catalog.
    pub backend_family: BackendFamily,
    /// Tolerance entries in catalog order.
    pub entries: Vec<ComparisonToleranceEntry>,
    /// Scope coverage rows in catalog order.
    pub coverage: Vec<ComparisonToleranceScopeCoverageSummary>,
    /// Number of unique bodies covered by the comparison corpus.
    pub comparison_body_count: usize,
    /// Number of comparison samples in the corpus.
    pub comparison_sample_count: usize,
    /// Comparison corpus time window.
    pub comparison_window: TimeRange,
    /// Candidate backend coordinate frames covered by the comparison corpus.
    pub coordinate_frames: Vec<CoordinateFrame>,
}

impl ComparisonTolerancePolicySummary {
    /// Validates the policy summary before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.entries.len() != self.coverage.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary entry/coverage mismatch: expected {} coverage rows for {} entries, found {}",
                    self.entries.len(),
                    self.entries.len(),
                    self.coverage.len()
                ),
            ));
        }

        let mut seen_bodies: Vec<(CelestialBody, ComparisonToleranceScope)> = Vec::new();

        for (index, entry) in self.entries.iter().enumerate() {
            if self.entries[..index]
                .iter()
                .any(|prior| prior.scope == entry.scope)
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary has duplicate scope '{}'",
                        entry.scope.label()
                    ),
                ));
            }

            entry.validate()?;

            let coverage = &self.coverage[index];
            if coverage.entry.scope != entry.scope {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary scope mismatch at index {}: expected {}, found {}",
                        index + 1,
                        entry.scope.label(),
                        coverage.entry.scope.label()
                    ),
                ));
            }

            if coverage.entry.tolerance.backend_family != self.backend_family {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary backend family mismatch at index {}: expected {}, found {}",
                        index + 1,
                        backend_family_label(&self.backend_family),
                        backend_family_label(&coverage.entry.tolerance.backend_family)
                    ),
                ));
            }

            coverage.validate()?;

            for body in &coverage.bodies {
                if let Some((prior_body, prior_scope)) =
                    seen_bodies.iter().find(|(seen_body, _)| seen_body == body)
                {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison tolerance policy summary body '{}' appears in multiple scope rows: first seen in {}, repeated in {}",
                            prior_body,
                            prior_scope.label(),
                            entry.scope.label()
                        ),
                    ));
                }
            }
            seen_bodies.extend(
                coverage
                    .bodies
                    .iter()
                    .cloned()
                    .map(|body| (body, entry.scope)),
            );
        }

        let comparison_body_count = self
            .coverage
            .iter()
            .map(|coverage| coverage.body_count)
            .sum::<usize>();
        if comparison_body_count != self.comparison_body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary body-count mismatch: expected {}, found {}",
                    self.comparison_body_count, comparison_body_count
                ),
            ));
        }

        let comparison_sample_count = self
            .coverage
            .iter()
            .map(|coverage| coverage.sample_count)
            .sum::<usize>();
        if comparison_sample_count != self.comparison_sample_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary sample-count mismatch: expected {}, found {}",
                    self.comparison_sample_count, comparison_sample_count
                ),
            ));
        }

        if self.coordinate_frames.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison tolerance policy summary has no coordinate frames",
            ));
        }

        self.comparison_window.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary has invalid comparison window: {error}"
                ),
            )
        })?;

        for (index, frame) in self.coordinate_frames.iter().enumerate() {
            if self.coordinate_frames[..index].contains(frame) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary has duplicate coordinate frame '{frame}'"
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Renders the compact report wording for this tolerance policy.
    pub fn summary_line(&self) -> String {
        let scopes = self
            .entries
            .iter()
            .map(|entry| entry.scope.label())
            .collect::<Vec<_>>()
            .join(", ");
        let limits = format_comparison_tolerance_limits_for_report(&self.entries);
        let coverage = self
            .coverage
            .iter()
            .map(ComparisonToleranceScopeCoverageSummary::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let coordinate_frames = format_frames(&self.coordinate_frames);
        let window = self.comparison_window.summary_line();

        format!(
            "backend family={}; scopes={} ({scopes}); limits={limits}; coverage={coverage}; window={window}; frames={coordinate_frames}; evidence={} bodies, {} samples",
            backend_family_label(&self.backend_family),
            self.entries.len(),
            self.comparison_body_count,
            self.comparison_sample_count,
        )
    }

    /// Validates the policy summary and returns its compact report line.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonTolerancePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Scope coverage row within a comparison tolerance policy summary.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonToleranceScopeCoverageSummary {
    /// Tolerance entry covered by this scope summary.
    pub entry: ComparisonToleranceEntry,
    /// Bodies assigned to this tolerance scope in first-seen order.
    pub bodies: Vec<CelestialBody>,
    /// Number of distinct bodies assigned to this tolerance scope.
    pub body_count: usize,
    /// Total number of samples covered by this tolerance scope.
    pub sample_count: usize,
}

impl ComparisonToleranceScopeCoverageSummary {
    /// Validates the coverage row before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body_count != self.bodies.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance coverage row '{}' body-count mismatch: expected {}, found {}",
                    self.entry.scope.label(),
                    self.body_count,
                    self.bodies.len()
                ),
            ));
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].contains(body) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance coverage row '{}' has duplicate body '{}': first-seen order must be unique",
                        self.entry.scope.label(),
                        body
                    ),
                ));
            }
        }

        self.entry.validate()?;
        Ok(())
    }

    /// Renders the compact report wording for this tolerance coverage row.
    pub fn summary_line(&self) -> String {
        let bodies = if self.bodies.is_empty() {
            "none".to_string()
        } else {
            format_bodies(&self.bodies)
        };
        let tolerance = &self.entry.tolerance;
        format!(
            "{}: backend family={}, profile={}, bodies={} ({}), samples={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}",
            self.entry.scope.label(),
            tolerance_backend_family_label(&tolerance.backend_family),
            tolerance.profile,
            self.body_count,
            bodies,
            self.sample_count,
            tolerance.max_longitude_delta_deg,
            tolerance.max_latitude_delta_deg,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }

    /// Returns the compact summary line after validating the coverage row.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonToleranceScopeCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn comparison_tolerance_policy_coverage(
    comparison: &ComparisonReport,
) -> Vec<ComparisonToleranceScopeCoverageSummary> {
    let entries = comparison_tolerance_policy_entries(&comparison.candidate_backend.family);
    let tolerance_summaries = comparison.tolerance_summaries();

    entries
        .into_iter()
        .map(|entry| {
            let mut bodies = Vec::new();
            let mut sample_count = 0;

            for summary in &tolerance_summaries {
                if comparison_tolerance_scope_for_body(&summary.body) == entry.scope {
                    bodies.push(summary.body.clone());
                    sample_count += summary.sample_count;
                }
            }

            ComparisonToleranceScopeCoverageSummary {
                entry,
                body_count: bodies.len(),
                bodies,
                sample_count,
            }
        })
        .collect()
}

fn write_tolerance_policy(
    f: &mut fmt::Formatter<'_>,
    comparison: &ComparisonReport,
) -> fmt::Result {
    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let coverage = comparison_tolerance_policy_coverage(comparison);
    let coordinate_frames = format_frames(comparison_coordinate_frames(comparison));
    writeln!(f, "Tolerance policy catalog")?;
    writeln!(f, "  candidate backend family: {}", family_label)?;
    writeln!(
        f,
        "  comparison evidence: {} bodies, {} samples",
        comparison.body_summaries().len(),
        comparison.summary.sample_count
    )?;
    writeln!(
        f,
        "  comparison window: JD {:.1} → {:.1}",
        comparison.corpus_summary.earliest_julian_day, comparison.corpus_summary.latest_julian_day
    )?;
    writeln!(f, "  coordinate frames: {}", coordinate_frames)?;
    for scope_coverage in coverage {
        writeln!(
            f,
            "  {}",
            match scope_coverage.validated_summary_line() {
                Ok(line) => line,
                Err(error) => format!(
                    "{} unavailable ({error})",
                    scope_coverage.entry.scope.label()
                ),
            }
        )?;
    }
    Ok(())
}

fn write_tolerance_policy_text(text: &mut String, comparison: &ComparisonReport) {
    use std::fmt::Write as _;

    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let coverage = comparison_tolerance_policy_coverage(comparison);
    let coordinate_frames = format_frames(comparison_coordinate_frames(comparison));
    let _ = writeln!(text, "Tolerance policy catalog");
    let _ = writeln!(text, "  candidate backend family: {}", family_label);
    let _ = writeln!(
        text,
        "  comparison evidence: {} bodies, {} samples",
        comparison.body_summaries().len(),
        comparison.summary.sample_count
    );
    let _ = writeln!(
        text,
        "  comparison window: JD {:.1} → {:.1}",
        comparison.corpus_summary.earliest_julian_day, comparison.corpus_summary.latest_julian_day
    );
    let _ = writeln!(text, "  coordinate frames: {}", coordinate_frames);
    for scope_coverage in coverage {
        let _ = writeln!(
            text,
            "  {}",
            match scope_coverage.validated_summary_line() {
                Ok(line) => line,
                Err(error) => format!(
                    "{} unavailable ({error})",
                    scope_coverage.entry.scope.label()
                ),
            }
        );
    }
}

/// Per-body comparison status against the expected tolerance table.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyToleranceSummary {
    /// Body queried for this tolerance summary.
    pub body: CelestialBody,
    /// Expected tolerance for the body.
    pub tolerance: ComparisonTolerance,
    /// Number of samples compared for this body.
    pub sample_count: usize,
    /// Whether all measured deltas are within the expected tolerance.
    pub within_tolerance: bool,
    /// Maximum absolute longitude delta measured for this body.
    pub max_longitude_delta_deg: f64,
    /// Signed margin between the longitude limit and measured maximum.
    pub longitude_margin_deg: f64,
    /// Maximum absolute latitude delta measured for this body.
    pub max_latitude_delta_deg: f64,
    /// Signed margin between the latitude limit and measured maximum.
    pub latitude_margin_deg: f64,
    /// Maximum absolute distance delta measured for this body.
    pub max_distance_delta_au: Option<f64>,
    /// Signed margin between the distance limit and measured maximum.
    pub distance_margin_au: Option<f64>,
}

impl BodyToleranceSummary {
    /// Returns `Ok(())` when the tolerance status is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        validate_comparison_tolerance(&self.tolerance)?;

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} has no samples to compare",
                    self.body
                ),
            ));
        }

        for (label, value) in [
            ("longitude", self.max_longitude_delta_deg),
            ("latitude", self.max_latitude_delta_deg),
            ("longitude margin", self.longitude_margin_deg),
            ("latitude margin", self.latitude_margin_deg),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid {} {}",
                        self.body, label, value
                    ),
                ));
            }
        }

        if let Some(value) = self.max_distance_delta_au {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance delta {}",
                        self.body, value
                    ),
                ));
            }
        }

        if let Some(value) = self.distance_margin_au {
            if !value.is_finite() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance margin {}",
                        self.body, value
                    ),
                ));
            }
        }

        let tolerance = &self.tolerance;
        let distance_margin = self.distance_margin_au;
        let has_distance_limit = tolerance.max_distance_delta_au.is_some();
        let has_distance_measurement = self.max_distance_delta_au.is_some();
        if distance_margin.is_some() != (has_distance_limit && has_distance_measurement) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} distance-margin presence does not match the measured values and tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_longitude_margin =
            tolerance.max_longitude_delta_deg - self.max_longitude_delta_deg;
        if self.longitude_margin_deg != expected_longitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} longitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_latitude_margin =
            tolerance.max_latitude_delta_deg - self.max_latitude_delta_deg;
        if self.latitude_margin_deg != expected_latitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} latitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        if let (Some(measured), Some(limit), Some(margin)) = (
            self.max_distance_delta_au,
            tolerance.max_distance_delta_au,
            self.distance_margin_au,
        ) {
            if margin != limit - measured {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} distance margin drifted from the declared tolerance limit",
                        self.body
                    ),
                ));
            }
        }

        let within_tolerance = self.longitude_margin_deg >= 0.0
            && self.latitude_margin_deg >= 0.0
            && self
                .distance_margin_au
                .map(|value| value >= 0.0)
                .unwrap_or(true);
        if self.within_tolerance != within_tolerance {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} status disagrees with the measured margins",
                    self.body
                ),
            ));
        }

        Ok(())
    }

    /// Renders the compact report wording after validating the summary fields.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Renders the compact report wording for this tolerance status.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: backend family={}, profile={}, samples={}, status={}, limit Δlon≤{:.6}°, margin Δlon={:+.12}°, limit Δlat≤{:.6}°, margin Δlat={:+.12}°, limit Δdist={}, margin Δdist={}",
            self.body,
            tolerance_backend_family_label(&self.tolerance.backend_family),
            self.tolerance.profile,
            self.sample_count,
            if self.within_tolerance { "within" } else { "exceeded" },
            self.tolerance.max_longitude_delta_deg,
            self.longitude_margin_deg,
            self.tolerance.max_latitude_delta_deg,
            self.latitude_margin_deg,
            self.tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.distance_margin_au
                .map(|value| format!("{value:+.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }
}

impl fmt::Display for BodyToleranceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A comparison report generated by the validation tooling.
#[derive(Clone, Debug)]
pub struct ComparisonReport {
    /// Corpus name.
    pub corpus_name: String,
    /// Corpus summary, including its epoch range and body coverage.
    pub corpus_summary: CorpusSummary,
    /// Apparentness mode used by the corpus.
    pub apparentness: Apparentness,
    /// Metadata for the reference backend.
    pub reference_backend: BackendMetadata,
    /// Metadata for the candidate backend.
    pub candidate_backend: BackendMetadata,
    /// Per-body comparison samples.
    pub samples: Vec<ComparisonSample>,
    /// Aggregate statistics.
    pub summary: ComparisonSummary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyClass {
    Luminary,
    MajorPlanet,
    LunarPoint,
    Asteroid,
    Custom,
}

impl BodyClass {
    const ALL: [Self; 5] = [
        Self::Luminary,
        Self::MajorPlanet,
        Self::LunarPoint,
        Self::Asteroid,
        Self::Custom,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminaries",
            Self::MajorPlanet => "Major planets",
            Self::LunarPoint => "Lunar points",
            Self::Asteroid => "Asteroids",
            Self::Custom => "Custom bodies",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Luminary => 0,
            Self::MajorPlanet => 1,
            Self::LunarPoint => 2,
            Self::Asteroid => 3,
            Self::Custom => 4,
        }
    }
}

fn body_class(body: &CelestialBody) -> BodyClass {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => BodyClass::Luminary,
        CelestialBody::Mercury
        | CelestialBody::Venus
        | CelestialBody::Mars
        | CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune
        | CelestialBody::Pluto => BodyClass::MajorPlanet,
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => BodyClass::LunarPoint,
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => BodyClass::Asteroid,
        CelestialBody::Custom(_) => BodyClass::Custom,
        _ => BodyClass::Custom,
    }
}

#[derive(Clone, Debug)]
struct BodyClassSummary {
    class: BodyClass,
    sample_count: usize,
    max_longitude_delta_body: Option<CelestialBody>,
    max_longitude_delta_deg: f64,
    sum_longitude_delta_deg: f64,
    sum_longitude_delta_sq_deg: f64,
    median_longitude_delta_deg: f64,
    percentile_longitude_delta_deg: f64,
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: f64,
    sum_latitude_delta_deg: f64,
    sum_latitude_delta_sq_deg: f64,
    median_latitude_delta_deg: f64,
    percentile_latitude_delta_deg: f64,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    sum_distance_delta_au: f64,
    sum_distance_delta_sq_au: f64,
    distance_count: usize,
    median_distance_delta_au: Option<f64>,
    percentile_distance_delta_au: Option<f64>,
}

#[derive(Clone, Debug)]
struct BodyClassToleranceSummary {
    class: BodyClass,
    tolerance: ComparisonTolerance,
    body_count: usize,
    sample_count: usize,
    within_tolerance_body_count: usize,
    outside_tolerance_body_count: usize,
    outside_tolerance_sample_count: usize,
    max_longitude_delta_body: Option<CelestialBody>,
    max_longitude_delta_deg: Option<f64>,
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: Option<f64>,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    sum_longitude_delta_deg: f64,
    sum_longitude_delta_sq_deg: f64,
    sum_latitude_delta_deg: f64,
    sum_latitude_delta_sq_deg: f64,
    sum_distance_delta_au: f64,
    sum_distance_delta_sq_au: f64,
    distance_count: usize,
    median_longitude_delta_deg: f64,
    percentile_longitude_delta_deg: f64,
    median_latitude_delta_deg: f64,
    percentile_latitude_delta_deg: f64,
    median_distance_delta_au: Option<f64>,
    percentile_distance_delta_au: Option<f64>,
    outside_bodies: Vec<CelestialBody>,
}

impl BodyClassToleranceSummary {
    fn validate(&self) -> Result<(), EphemerisError> {
        if self.body_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "body-class tolerance summary must include at least one body",
            ));
        }

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "body-class tolerance summary must include at least one sample",
            ));
        }

        let within_and_outside = self
            .within_tolerance_body_count
            .saturating_add(self.outside_tolerance_body_count);
        if within_and_outside != self.body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary body-count mismatch: expected {}, found within tolerance {} + outside tolerance {} = {}",
                    self.body_count,
                    self.within_tolerance_body_count,
                    self.outside_tolerance_body_count,
                    within_and_outside
                ),
            ));
        }

        if self.outside_tolerance_sample_count > self.sample_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary outside-sample count {} exceeds sample count {}",
                    self.outside_tolerance_sample_count, self.sample_count
                ),
            ));
        }

        if self.outside_bodies.len() != self.outside_tolerance_body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary outside-body list mismatch: expected {}, found {}",
                    self.outside_tolerance_body_count,
                    self.outside_bodies.len()
                ),
            ));
        }

        let mut seen_outside_bodies = BTreeSet::new();
        for body in &self.outside_bodies {
            if !seen_outside_bodies.insert(body.to_string()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body-class tolerance summary contains a duplicate outside body: {body}"
                    ),
                ));
            }
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            (
                "median_longitude_delta_deg",
                Some(self.median_longitude_delta_deg),
            ),
            (
                "percentile_longitude_delta_deg",
                Some(self.percentile_longitude_delta_deg),
            ),
            (
                "median_latitude_delta_deg",
                Some(self.median_latitude_delta_deg),
            ),
            (
                "percentile_latitude_delta_deg",
                Some(self.percentile_latitude_delta_deg),
            ),
            (
                "sum_longitude_delta_deg",
                Some(self.sum_longitude_delta_deg),
            ),
            (
                "sum_longitude_delta_sq_deg",
                Some(self.sum_longitude_delta_sq_deg),
            ),
            ("sum_latitude_delta_deg", Some(self.sum_latitude_delta_deg)),
            (
                "sum_latitude_delta_sq_deg",
                Some(self.sum_latitude_delta_sq_deg),
            ),
            ("sum_distance_delta_au", Some(self.sum_distance_delta_au)),
            (
                "sum_distance_delta_sq_au",
                Some(self.sum_distance_delta_sq_au),
            ),
        ] {
            if let Some(value) = value {
                if !value.is_finite() || value.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "body-class tolerance summary for {} field `{label}` must be a finite non-negative value",
                            self.class.label()
                        ),
                    ));
                }
            }
        }

        if let Some(value) = self.max_distance_delta_au {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body-class tolerance summary for {} field `max_distance_delta_au` must be a finite non-negative value",
                        self.class.label()
                    ),
                ));
            }
        }

        if self.distance_count > self.sample_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary distance count {} exceeds sample count {}",
                    self.distance_count, self.sample_count
                ),
            ));
        }

        let distance_fields_present = self.max_distance_delta_body.is_some()
            || self.max_distance_delta_au.is_some()
            || self.median_distance_delta_au.is_some()
            || self.percentile_distance_delta_au.is_some();
        if self.distance_count == 0 {
            if distance_fields_present {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body-class tolerance summary for {} must not carry distance metrics when no distance samples are present",
                        self.class.label()
                    ),
                ));
            }
        } else if self.max_distance_delta_body.is_none()
            || self.max_distance_delta_au.is_none()
            || self.median_distance_delta_au.is_none()
            || self.percentile_distance_delta_au.is_none()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary for {} must carry distance extrema and spread metrics when distance samples are present",
                    self.class.label()
                ),
            ));
        }

        if self.max_longitude_delta_body.is_none() || self.max_latitude_delta_body.is_none() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary for {} must name the longitude and latitude extrema drivers",
                    self.class.label()
                ),
            ));
        }

        Ok(())
    }

    fn mean_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_longitude_delta_deg / self.sample_count as f64
        }
    }

    fn rms_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_longitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    fn rms_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_latitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    fn rms_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some((self.sum_distance_delta_sq_au / self.distance_count as f64).sqrt())
        }
    }

    fn summary_line(&self) -> String {
        let tolerance = &self.tolerance;
        let max_longitude_body = self
            .max_longitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_latitude_body = self
            .max_latitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_distance_body = self
            .max_distance_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let longitude_margin = self
            .max_longitude_delta_deg
            .map(|value| format!("{:+.12}°", tolerance.max_longitude_delta_deg - value))
            .unwrap_or_else(|| "n/a".to_string());
        let latitude_margin = self
            .max_latitude_delta_deg
            .map(|value| format!("{:+.12}°", tolerance.max_latitude_delta_deg - value))
            .unwrap_or_else(|| "n/a".to_string());
        let distance_margin = self
            .max_distance_delta_au
            .zip(tolerance.max_distance_delta_au)
            .map(|(value, limit)| format!("{:+.12} AU", limit - value))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "{}: backend family={}, profile={}, bodies={}, samples={}, within tolerance bodies={}, outside tolerance bodies={}, limit Δlon≤{:.6}°, margin Δlon={}, limit Δlat≤{:.6}°, margin Δlat={}, limit Δdist={}, margin Δdist={}, max Δlon={}{}, max Δlat={}{}, max Δdist={}{}",
            self.class.label(),
            tolerance_backend_family_label(&tolerance.backend_family),
            tolerance.profile,
            self.body_count,
            self.sample_count,
            self.within_tolerance_body_count,
            self.outside_tolerance_body_count,
            tolerance.max_longitude_delta_deg,
            longitude_margin,
            tolerance.max_latitude_delta_deg,
            latitude_margin,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            distance_margin,
            self.max_longitude_delta_deg
                .map(|value| format!("{value:.12}°"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_longitude_body,
            self.max_latitude_delta_deg
                .map(|value| format!("{value:.12}°"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_latitude_body,
            self.max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_distance_body
        )
    }

    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  {}", self.class.label())?;
        writeln!(
            f,
            "    backend family: {}",
            tolerance_backend_family_label(&self.tolerance.backend_family)
        )?;
        writeln!(f, "    profile: {}", self.tolerance.profile)?;
        writeln!(f, "    bodies: {}", self.body_count)?;
        writeln!(f, "    samples: {}", self.sample_count)?;
        writeln!(
            f,
            "    within tolerance bodies: {}",
            self.within_tolerance_body_count
        )?;
        writeln!(
            f,
            "    outside tolerance bodies: {}",
            self.outside_tolerance_body_count
        )?;
        writeln!(
            f,
            "    outside tolerance samples: {}",
            self.outside_tolerance_sample_count
        )?;
        if !self.outside_bodies.is_empty() {
            writeln!(
                f,
                "    outside bodies: {}",
                format_bodies(&self.outside_bodies)
            )?;
        }
        if let (Some(body), Some(value)) = (
            self.max_longitude_delta_body.as_ref(),
            self.max_longitude_delta_deg,
        ) {
            writeln!(
                f,
                "    max longitude delta: {:.12}° ({}) | limit Δlon≤{:.6}°, margin Δlon={:+.12}°",
                value,
                body,
                self.tolerance.max_longitude_delta_deg,
                self.tolerance.max_longitude_delta_deg - value
            )?;
        }
        writeln!(
            f,
            "    mean longitude delta: {:.12}°",
            self.mean_longitude_delta_deg()
        )?;
        writeln!(
            f,
            "    median longitude delta: {:.12}°",
            self.median_longitude_delta_deg
        )?;
        writeln!(
            f,
            "    95th percentile longitude delta: {:.12}°",
            self.percentile_longitude_delta_deg
        )?;
        writeln!(
            f,
            "    rms longitude delta: {:.12}°",
            self.rms_longitude_delta_deg()
        )?;
        if let (Some(body), Some(value)) = (
            self.max_latitude_delta_body.as_ref(),
            self.max_latitude_delta_deg,
        ) {
            writeln!(
                f,
                "    max latitude delta: {:.12}° ({}) | limit Δlat≤{:.6}°, margin Δlat={:+.12}°",
                value,
                body,
                self.tolerance.max_latitude_delta_deg,
                self.tolerance.max_latitude_delta_deg - value
            )?;
        }
        writeln!(
            f,
            "    mean latitude delta: {:.12}°",
            self.mean_latitude_delta_deg()
        )?;
        writeln!(
            f,
            "    median latitude delta: {:.12}°",
            self.median_latitude_delta_deg
        )?;
        writeln!(
            f,
            "    95th percentile latitude delta: {:.12}°",
            self.percentile_latitude_delta_deg
        )?;
        writeln!(
            f,
            "    rms latitude delta: {:.12}°",
            self.rms_latitude_delta_deg()
        )?;
        if let (Some(body), Some(value)) = (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_au,
        ) {
            let limit = self
                .tolerance
                .max_distance_delta_au
                .expect("distance tolerance should exist for class summaries");
            writeln!(
                f,
                "    max distance delta: {:.12} AU ({}) | limit Δdist={:.6} AU, margin Δdist={:+.12} AU",
                value,
                body,
                limit,
                limit - value
            )?;
        }
        if let Some(value) = self.mean_distance_delta_au() {
            writeln!(f, "    mean distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.median_distance_delta_au {
            writeln!(f, "    median distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.percentile_distance_delta_au {
            writeln!(f, "    95th percentile distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.rms_distance_delta_au() {
            writeln!(f, "    rms distance delta: {:.12} AU", value)?;
        }

        Ok(())
    }
}

impl fmt::Display for BodyClassToleranceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl BodyClassSummary {
    fn mean_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_longitude_delta_deg / self.sample_count as f64
        }
    }

    fn rms_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_longitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    fn rms_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_latitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    fn rms_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some((self.sum_distance_delta_sq_au / self.distance_count as f64).sqrt())
        }
    }

    fn summary_line(&self) -> String {
        if self.sample_count == 0 {
            return String::new();
        }

        let max_distance = self
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let mean_distance = self
            .mean_distance_delta_au()
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let median_distance = self
            .median_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let percentile_distance = self
            .percentile_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let rms_distance = self
            .rms_distance_delta_au()
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, median Δlon={:.12}°, 95th percentile longitude delta: {:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, median Δlat={:.12}°, 95th percentile latitude delta: {:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, median Δdist={}, 95th percentile distance delta: {}, rms Δdist={}",
            self.sample_count,
            self.max_longitude_delta_deg,
            format_summary_body(&self.max_longitude_delta_body),
            self.mean_longitude_delta_deg(),
            self.median_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.rms_longitude_delta_deg(),
            self.max_latitude_delta_deg,
            format_summary_body(&self.max_latitude_delta_body),
            self.mean_latitude_delta_deg(),
            self.median_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            self.rms_latitude_delta_deg(),
            max_distance,
            format_summary_body(&self.max_distance_delta_body),
            mean_distance,
            median_distance,
            percentile_distance,
            rms_distance,
        )
    }

    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sample_count == 0 {
            return Ok(());
        }

        writeln!(f, "  {}", self.class.label())?;
        writeln!(f, "    {}", self.summary_line())?;

        Ok(())
    }
}

impl fmt::Display for BodyClassSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A notable regression observed in a comparison report.
#[derive(Clone, Debug, PartialEq)]
pub struct RegressionFinding {
    /// Body that triggered the regression note.
    pub body: CelestialBody,
    /// Absolute longitude delta in degrees.
    pub longitude_delta_deg: f64,
    /// Absolute latitude delta in degrees.
    pub latitude_delta_deg: f64,
    /// Absolute distance delta in astronomical units.
    pub distance_delta_au: Option<f64>,
    /// Human-readable note describing why the sample is notable.
    pub note: String,
}

impl RegressionFinding {
    /// Returns the compact release-facing summary line for the regression case.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
            self.body,
            self.longitude_delta_deg,
            self.latitude_delta_deg,
            self.distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.note,
        )
    }
}

impl fmt::Display for RegressionFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl RegressionFinding {
    /// Returns `Ok(())` when the regression note is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.note.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "regression finding note must not be blank",
            ));
        }

        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "regression finding field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance) = self.distance_delta_au {
            if !distance.is_finite() || distance.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "regression finding distance delta must be a finite non-negative value",
                ));
            }
        }

        Ok(())
    }
}

/// Benchmark summary for a backend.
#[derive(Clone, Debug)]
pub struct BenchmarkReport {
    /// Backend metadata.
    pub backend: BackendMetadata,
    /// Corpus name used for the benchmark.
    pub corpus_name: String,
    /// Apparentness mode used by the benchmark corpus.
    pub apparentness: Apparentness,
    /// Number of benchmark rounds.
    pub rounds: usize,
    /// Number of requests per round.
    pub sample_count: usize,
    /// Total elapsed time for the single-request path.
    pub elapsed: std::time::Duration,
    /// Total elapsed time for the batch-request path.
    pub batch_elapsed: std::time::Duration,
    /// Estimated heap footprint of the benchmark corpus in bytes.
    pub estimated_corpus_heap_bytes: usize,
}

impl BenchmarkReport {
    /// Returns a compact release-facing summary line for the benchmark.
    pub fn summary_line(&self) -> String {
        format!(
            "backend={}; corpus={}; apparentness={}; rounds={}; samples per round={}; single ns/request={}; batch ns/request={}; batch throughput={:.2} req/s; estimated corpus heap footprint={} bytes",
            self.backend.id,
            self.corpus_name,
            self.apparentness,
            self.rounds,
            self.sample_count,
            format_ns(self.nanoseconds_per_request()),
            format_ns(self.batch_nanoseconds_per_request()),
            self.batch_requests_per_second(),
            self.estimated_corpus_heap_bytes,
        )
    }

    /// Returns the validated compact summary line for the benchmark.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the average number of nanoseconds per request for the single-request path.
    pub fn nanoseconds_per_request(&self) -> f64 {
        let total_requests = (self.rounds * self.sample_count) as f64;
        if total_requests == 0.0 {
            return 0.0;
        }

        self.elapsed.as_secs_f64() * 1_000_000_000.0 / total_requests
    }

    /// Returns the average number of nanoseconds per request for the batch path.
    pub fn batch_nanoseconds_per_request(&self) -> f64 {
        let total_requests = (self.rounds * self.sample_count) as f64;
        if total_requests == 0.0 {
            return 0.0;
        }

        self.batch_elapsed.as_secs_f64() * 1_000_000_000.0 / total_requests
    }

    /// Returns the average throughput in requests per second for the batch path.
    pub fn batch_requests_per_second(&self) -> f64 {
        let total_requests = (self.rounds * self.sample_count) as f64;
        if self.batch_elapsed.is_zero() || total_requests == 0.0 {
            return 0.0;
        }

        total_requests / self.batch_elapsed.as_secs_f64()
    }

    /// Returns the benchmark methodology summary.
    pub fn methodology_summary(&self) -> String {
        format!(
            "{} rounds x {} requests per round on the {} corpus; apparentness {}; single-request and batch paths are measured separately",
            self.rounds, self.sample_count, self.corpus_name, self.apparentness
        )
    }

    /// Validates the benchmark metadata before the report is returned or formatted.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        self.backend.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("benchmark backend metadata is invalid: {error}"),
            )
        })?;

        if self.corpus_name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "benchmark corpus name must not be blank",
            ));
        }

        if self.rounds == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "benchmark rounds must be greater than zero",
            ));
        }

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "benchmark sample count must be greater than zero",
            ));
        }

        if self.estimated_corpus_heap_bytes == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "benchmark estimated corpus heap footprint must be greater than zero",
            ));
        }

        Ok(())
    }
}

/// A preserved archive of regression cases from a comparison run.
#[derive(Clone, Debug)]
pub struct RegressionArchive {
    /// Corpus that produced the archived cases.
    pub corpus_name: String,
    /// Regression findings that should stay visible in reports and tests.
    pub cases: Vec<RegressionFinding>,
}

/// A full validation report containing comparison, house, and benchmark data.
#[derive(Clone, Debug)]
pub struct ValidationReport {
    /// Comparison corpus summary.
    pub comparison_corpus: CorpusSummary,
    /// Benchmark corpus summary.
    pub benchmark_corpus: CorpusSummary,
    /// Packaged-data benchmark corpus summary.
    pub packaged_benchmark_corpus: CorpusSummary,
    /// Chart-benchmark corpus summary.
    pub chart_benchmark_corpus: CorpusSummary,
    /// Packaged-artifact decode benchmark.
    pub artifact_decode_benchmark: ArtifactDecodeBenchmarkReport,
    /// House-validation corpus summary.
    pub house_validation: HouseValidationReport,
    /// Comparison output.
    pub comparison: ComparisonReport,
    /// Archived regression cases preserved from the comparison corpus.
    pub archived_regressions: RegressionArchive,
    /// Benchmark output for the reference backend.
    pub reference_benchmark: BenchmarkReport,
    /// Benchmark output for the candidate backend.
    pub candidate_benchmark: BenchmarkReport,
    /// Benchmark output for the packaged-data backend.
    pub packaged_benchmark: BenchmarkReport,
    /// Benchmark output for full chart assembly.
    pub chart_benchmark: ChartBenchmarkReport,
}

impl RegressionArchive {
    /// Returns `Ok(())` when the archived regression cases are internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.corpus_name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "regression archive corpus name must not be blank",
            ));
        }

        for (index, case) in self.cases.iter().enumerate() {
            case.validate().map_err(|error| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!("regression archive case #{} is invalid: {error}", index + 1),
                )
            })?;
        }

        Ok(())
    }
}

impl ValidationReport {
    /// Returns `Ok(())` when the report is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        self.comparison_corpus.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report comparison corpus is invalid: {error}"),
            )
        })?;
        self.benchmark_corpus.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report benchmark corpus is invalid: {error}"),
            )
        })?;
        self.packaged_benchmark_corpus.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report packaged benchmark corpus is invalid: {error}"),
            )
        })?;
        self.chart_benchmark_corpus.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report chart benchmark corpus is invalid: {error}"),
            )
        })?;
        self.artifact_decode_benchmark.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report artifact decode benchmark is invalid: {error}"),
            )
        })?;
        self.house_validation.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report house validation corpus is invalid: {error}"),
            )
        })?;
        self.comparison.summary.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report comparison summary is invalid: {error}"),
            )
        })?;
        self.comparison.corpus_summary.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report comparison corpus summary is invalid: {error}"),
            )
        })?;
        if self.comparison.summary.sample_count != self.comparison.samples.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "validation report comparison summary sample-count mismatch: summary has {}, samples have {}",
                    self.comparison.summary.sample_count,
                    self.comparison.samples.len()
                ),
            ));
        }
        if self.comparison.corpus_summary.request_count != self.comparison.samples.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "validation report comparison corpus request-count mismatch: summary has {}, samples have {}",
                    self.comparison.corpus_summary.request_count,
                    self.comparison.samples.len()
                ),
            ));
        }
        if self.comparison.corpus_summary.name != self.comparison.corpus_name {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report comparison corpus name does not match the comparison summary",
            ));
        }
        if self.comparison.corpus_summary.apparentness != self.comparison.apparentness {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report comparison corpus apparentness does not match the comparison summary",
            ));
        }
        if self.reference_benchmark.corpus_name != self.comparison_corpus.name {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report reference benchmark corpus does not match the comparison corpus",
            ));
        }
        if self.reference_benchmark.apparentness != self.comparison_corpus.apparentness {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report reference benchmark apparentness does not match the comparison corpus",
            ));
        }
        if self.reference_benchmark.sample_count != self.comparison_corpus.request_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report reference benchmark sample count does not match the comparison corpus",
            ));
        }
        if self.candidate_benchmark.corpus_name != self.benchmark_corpus.name {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report candidate benchmark corpus does not match the benchmark corpus",
            ));
        }
        if self.candidate_benchmark.apparentness != self.benchmark_corpus.apparentness {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report candidate benchmark apparentness does not match the benchmark corpus",
            ));
        }
        if self.candidate_benchmark.sample_count != self.benchmark_corpus.request_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report candidate benchmark sample count does not match the benchmark corpus",
            ));
        }
        if self.packaged_benchmark.corpus_name != self.packaged_benchmark_corpus.name {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report packaged benchmark corpus does not match the packaged benchmark corpus summary",
            ));
        }
        if self.packaged_benchmark.apparentness != self.packaged_benchmark_corpus.apparentness {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report packaged benchmark apparentness does not match the packaged benchmark corpus",
            ));
        }
        if self.packaged_benchmark.sample_count != self.packaged_benchmark_corpus.request_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report packaged benchmark sample count does not match the packaged benchmark corpus",
            ));
        }
        if self.chart_benchmark.corpus_name != self.chart_benchmark_corpus.name {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report chart benchmark corpus does not match the chart benchmark corpus summary",
            ));
        }
        if self.chart_benchmark.apparentness != self.chart_benchmark_corpus.apparentness {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report chart benchmark apparentness does not match the chart benchmark corpus",
            ));
        }
        if self.chart_benchmark.sample_count != self.chart_benchmark_corpus.request_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "validation report chart benchmark sample count does not match the chart benchmark corpus",
            ));
        }
        self.reference_benchmark.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report reference benchmark is invalid: {error}"),
            )
        })?;
        self.candidate_benchmark.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report candidate benchmark is invalid: {error}"),
            )
        })?;
        self.packaged_benchmark.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report packaged benchmark is invalid: {error}"),
            )
        })?;
        self.chart_benchmark.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report chart benchmark is invalid: {error}"),
            )
        })?;
        self.archived_regressions.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("validation report regression archive is invalid: {error}"),
            )
        })?;

        Ok(())
    }
}

/// A generated release bundle containing the compatibility profile, release-profile
/// identifiers, release notes, release checklist, backend matrix, API posture,
/// API stability summary, comparison-envelope summary, comparison-corpus release-guard summary,
/// validation report summary, artifact summary, packaged-artifact generation manifest, benchmark report,
/// validation report, and manifest.
#[derive(Clone, Debug)]
pub struct ReleaseBundle {
    /// Source revision recorded when the bundle was generated.
    pub source_revision: String,
    /// Workspace status recorded when the bundle was generated.
    pub workspace_status: String,
    /// Rust compiler version recorded when the bundle was generated.
    pub rustc_version: String,
    /// Output directory chosen by the caller.
    pub output_dir: PathBuf,
    /// Path to the generated compatibility profile file.
    pub compatibility_profile_path: PathBuf,
    /// Path to the generated compatibility-profile summary file.
    pub compatibility_profile_summary_path: PathBuf,
    /// Path to the generated release notes file.
    pub release_notes_path: PathBuf,
    /// Path to the generated release notes summary file.
    pub release_notes_summary_path: PathBuf,
    /// Path to the generated release summary file.
    pub release_summary_path: PathBuf,
    /// Path to the generated release-profile identifiers file.
    pub release_profile_identifiers_path: PathBuf,
    /// Path to the generated release checklist file.
    pub release_checklist_path: PathBuf,
    /// Path to the generated release checklist summary file.
    pub release_checklist_summary_path: PathBuf,
    /// Path to the generated backend capability matrix file.
    pub backend_matrix_path: PathBuf,
    /// Path to the generated backend capability matrix summary file.
    pub backend_matrix_summary_path: PathBuf,
    /// Path to the generated API stability posture file.
    pub api_stability_path: PathBuf,
    /// Path to the generated API stability summary file.
    pub api_stability_summary_path: PathBuf,
    /// Path to the generated comparison-envelope summary file.
    pub comparison_envelope_summary_path: PathBuf,
    /// Path to the generated comparison-corpus release-guard summary file.
    pub comparison_corpus_release_guard_summary_path: PathBuf,
    /// Path to the generated validation report summary file.
    pub validation_report_summary_path: PathBuf,
    /// Path to the generated workspace-audit summary file.
    pub workspace_audit_summary_path: PathBuf,
    /// Path to the generated artifact summary file.
    pub artifact_summary_path: PathBuf,
    /// Path to the generated packaged-artifact generation manifest file.
    pub packaged_artifact_generation_manifest_path: PathBuf,
    /// Path to the generated benchmark report file.
    pub benchmark_report_path: PathBuf,
    /// Path to the generated validation report file.
    pub validation_report_path: PathBuf,
    /// Path to the generated bundle manifest.
    pub manifest_path: PathBuf,
    /// Path to the generated manifest checksum sidecar.
    pub manifest_checksum_path: PathBuf,
    /// Number of bytes written for the compatibility profile.
    pub compatibility_profile_bytes: usize,
    /// Number of bytes written for the compatibility-profile summary.
    pub compatibility_profile_summary_bytes: usize,
    /// Number of bytes written for the release notes.
    pub release_notes_bytes: usize,
    /// Number of bytes written for the release notes summary.
    pub release_notes_summary_bytes: usize,
    /// Number of bytes written for the release summary.
    pub release_summary_bytes: usize,
    /// Number of bytes written for the release-profile identifiers file.
    pub release_profile_identifiers_bytes: usize,
    /// Number of bytes written for the release checklist.
    pub release_checklist_bytes: usize,
    /// Number of bytes written for the release checklist summary.
    pub release_checklist_summary_bytes: usize,
    /// Number of bytes written for the backend capability matrix.
    pub backend_matrix_bytes: usize,
    /// Number of bytes written for the backend capability matrix summary.
    pub backend_matrix_summary_bytes: usize,
    /// Number of bytes written for the API stability posture.
    pub api_stability_bytes: usize,
    /// Number of bytes written for the API stability summary.
    pub api_stability_summary_bytes: usize,
    /// Number of bytes written for the comparison-envelope summary.
    pub comparison_envelope_summary_bytes: usize,
    /// Number of bytes written for the comparison-corpus release-guard summary.
    pub comparison_corpus_release_guard_summary_bytes: usize,
    /// Number of bytes written for the validation report summary.
    pub validation_report_summary_bytes: usize,
    /// Number of bytes written for the workspace-audit summary.
    pub workspace_audit_summary_bytes: usize,
    /// Number of bytes written for the artifact summary.
    pub artifact_summary_bytes: usize,
    /// Number of bytes written for the packaged-artifact generation manifest.
    pub packaged_artifact_generation_manifest_bytes: usize,
    /// Number of bytes written for the benchmark report.
    pub benchmark_report_bytes: usize,
    /// Number of bytes written for the validation report.
    pub validation_report_bytes: usize,
    /// Number of bytes written for the manifest checksum sidecar.
    pub manifest_checksum_bytes: usize,
    /// Deterministic checksum for the compatibility profile contents.
    pub compatibility_profile_checksum: u64,
    /// Deterministic checksum for the compatibility-profile summary contents.
    pub compatibility_profile_summary_checksum: u64,
    /// Deterministic checksum for the release notes contents.
    pub release_notes_checksum: u64,
    /// Deterministic checksum for the release notes summary contents.
    pub release_notes_summary_checksum: u64,
    /// Deterministic checksum for the release summary contents.
    pub release_summary_checksum: u64,
    /// Deterministic checksum for the release-profile identifiers contents.
    pub release_profile_identifiers_checksum: u64,
    /// Deterministic checksum for the release checklist contents.
    pub release_checklist_checksum: u64,
    /// Deterministic checksum for the release checklist summary contents.
    pub release_checklist_summary_checksum: u64,
    /// Deterministic checksum for the backend capability matrix contents.
    pub backend_matrix_checksum: u64,
    /// Deterministic checksum for the backend capability matrix summary contents.
    pub backend_matrix_summary_checksum: u64,
    /// Deterministic checksum for the API stability posture contents.
    pub api_stability_checksum: u64,
    /// Deterministic checksum for the API stability summary contents.
    pub api_stability_summary_checksum: u64,
    /// Deterministic checksum for the comparison-envelope summary contents.
    pub comparison_envelope_summary_checksum: u64,
    /// Deterministic checksum for the comparison-corpus release-guard summary contents.
    pub comparison_corpus_release_guard_summary_checksum: u64,
    /// Deterministic checksum for the validation report summary contents.
    pub validation_report_summary_checksum: u64,
    /// Deterministic checksum for the workspace-audit summary contents.
    pub workspace_audit_summary_checksum: u64,
    /// Deterministic checksum for the artifact summary contents.
    pub artifact_summary_checksum: u64,
    /// Deterministic checksum for the packaged-artifact generation manifest contents.
    pub packaged_artifact_generation_manifest_checksum: u64,
    /// Deterministic checksum for the benchmark report contents.
    pub benchmark_report_checksum: u64,
    /// Deterministic checksum for the validation report contents.
    pub validation_report_checksum: u64,
    /// Deterministic checksum recorded in the manifest checksum sidecar.
    pub manifest_checksum: u64,
    /// Number of validation rounds recorded in the bundle manifest.
    pub validation_rounds: usize,
}

/// Errors produced while assembling a release bundle.
#[derive(Debug)]
pub enum ReleaseBundleError {
    /// File-system failure while creating or writing the bundle.
    Io(std::io::Error),
    /// Validation failure while rendering the compatibility profile, API posture, or report.
    Validation(EphemerisError),
    /// Release-bundle verification failed after writing or reading the staged artifacts.
    Verification(String),
}

impl fmt::Display for ReleaseBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Validation(error) => write!(f, "{error}"),
            Self::Verification(message) => {
                write!(f, "release bundle verification failed: {message}")
            }
        }
    }
}

impl std::error::Error for ReleaseBundleError {}

impl From<std::io::Error> for ReleaseBundleError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<EphemerisError> for ReleaseBundleError {
    fn from(error: EphemerisError) -> Self {
        Self::Validation(error)
    }
}

/// A deterministic workspace audit that checks for mandatory native build hooks
/// in the first-party crates and lockfile.
#[derive(Clone, Debug)]
pub struct WorkspaceAuditReport {
    /// Workspace root used for the scan.
    pub workspace_root: PathBuf,
    /// Workspace manifest files that were checked.
    pub manifest_paths: Vec<PathBuf>,
    /// Workspace lockfile path that was checked.
    pub lockfile_path: PathBuf,
    /// Detected policy violations.
    pub violations: Vec<WorkspaceAuditViolation>,
}

/// A single workspace-audit finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceAuditViolation {
    /// File that triggered the finding.
    pub path: PathBuf,
    /// Stable rule identifier for the finding.
    pub rule: &'static str,
    /// Human-readable explanation of the finding.
    pub detail: String,
}

impl WorkspaceAuditReport {
    /// Returns whether the workspace passed the audit cleanly.
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// A compact workspace-audit summary derived from the detailed report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceAuditSummary {
    /// Workspace root used for the scan.
    pub workspace_root: PathBuf,
    /// Number of manifests that were checked.
    pub manifest_count: usize,
    /// Workspace lockfile path that was checked.
    pub lockfile_path: PathBuf,
    /// Number of detected policy violations.
    pub violation_count: usize,
    /// Policy-violation counts grouped by rule in stable order.
    pub rule_counts: Vec<(&'static str, usize)>,
    /// Whether the workspace passed the audit cleanly.
    pub clean: bool,
}

impl WorkspaceAuditSummary {
    /// Validates the compact summary before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let mut counted_violations = 0usize;
        let mut seen_rules = BTreeSet::new();

        for (rule, count) in &self.rule_counts {
            if rule.trim().is_empty() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "workspace audit summary contains a blank rule label".to_string(),
                ));
            }
            if *count == 0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!("workspace audit summary rule '{}' has a zero count", rule),
                ));
            }
            if !seen_rules.insert(*rule) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "workspace audit summary contains a duplicate rule count for '{}'",
                        rule
                    ),
                ));
            }
            counted_violations = counted_violations.checked_add(*count).ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "workspace audit summary rule counts overflowed the aggregate violation count"
                        .to_string(),
                )
            })?;
        }

        if counted_violations != self.violation_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "workspace audit summary violation count mismatch: expected {}, found {}",
                    counted_violations, self.violation_count
                ),
            ));
        }

        if self.clean != (self.violation_count == 0) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "workspace audit summary clean flag mismatch: clean={}, violations={}",
                    self.clean, self.violation_count
                ),
            ));
        }

        if self.clean && !self.rule_counts.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "workspace audit summary marked clean but still carries rule counts".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let mut text = format!(
            "workspace root: {}; manifests checked: {}; lockfile: {}; violations: {}; result: {}",
            self.workspace_root.display(),
            self.manifest_count,
            self.lockfile_path.display(),
            self.violation_count,
            if self.clean {
                "no mandatory native build hooks detected"
            } else {
                "violations found"
            }
        );
        if !self.rule_counts.is_empty() {
            text.push_str("; rule counts: ");
            text.push_str(
                &self
                    .rule_counts
                    .iter()
                    .map(|(rule, count)| format!("{}: {}", rule, count))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        text
    }

    /// Validates and returns the compact summary line.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for WorkspaceAuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const RELEASE_CHECKLIST_REPOSITORY_MANAGED_RELEASE_GATES: [&str; 7] = [
    "[x] mise run fmt",
    "[x] mise run lint",
    "[x] mise run test",
    "[x] mise run audit",
    "[x] mise run release-smoke",
    "[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile",
    "[x] cargo run -q -p pleiades-validate -- validate-artifact",
];

const RELEASE_CHECKLIST_MANUAL_BUNDLE_WORKFLOW: [&str; 3] = [
    "[x] cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release",
    "[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release",
    "[x] docs/release-reproducibility.md",
];

const RELEASE_CHECKLIST_BUNDLE_CONTENTS: [&str; 17] = [
    "[x] compatibility-profile.txt",
    "[x] compatibility-profile-summary.txt",
    "[x] release-notes.txt",
    "[x] release-notes-summary.txt",
    "[x] release-summary.txt",
    "[x] release-checklist.txt",
    "[x] release-checklist-summary.txt",
    "[x] backend-matrix.txt",
    "[x] backend-matrix-summary.txt",
    "[x] api-stability.txt",
    "[x] api-stability-summary.txt",
    "[x] validation-report-summary.txt",
    "[x] workspace-audit-summary.txt",
    "[x] validation-report.txt",
    "[x] bundle-manifest.txt",
    "[x] bundle-manifest.checksum.txt",
    "[x] verify-release-bundle",
];

const RELEASE_CHECKLIST_EXTERNAL_PUBLISHING_REMINDERS: [&str; 3] = [
    "[ ] tag and archive the release commit",
    "[ ] publish or attach the release bundle outside the workspace",
    "[ ] review any documented compatibility gaps before announcing the release",
];

fn release_checklist_repository_managed_release_gates() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_REPOSITORY_MANAGED_RELEASE_GATES
}

fn release_checklist_manual_bundle_workflow() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_MANUAL_BUNDLE_WORKFLOW
}

fn release_checklist_bundle_contents() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_BUNDLE_CONTENTS
}

fn release_checklist_external_publishing_reminders() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_EXTERNAL_PUBLISHING_REMINDERS
}

/// A compact summary of the release checklist posture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseChecklistSummary {
    /// Release-profile identifiers used by the checklist.
    pub release_profile_identifiers: ReleaseProfileIdentifiers,
    /// Number of repository-managed release gates.
    pub repository_managed_release_gates: usize,
    /// Number of manual bundle workflow items.
    pub manual_bundle_workflow_items: usize,
    /// Number of bundle content items.
    pub bundle_contents_items: usize,
    /// Number of external publishing reminders.
    pub external_publishing_reminders: usize,
}

impl ReleaseChecklistSummary {
    /// Returns `Ok(())` when the compact summary still matches the release posture.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        self.release_profile_identifiers
            .validate()
            .map_err(|error| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                    "release checklist summary release-profile identifiers are invalid: {error}"
                ),
                )
            })?;

        let repository_managed_release_gates =
            release_checklist_repository_managed_release_gates().len();
        if self.repository_managed_release_gates != repository_managed_release_gates {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary repository-managed release gate count mismatch: expected {}, found {}",
                    repository_managed_release_gates, self.repository_managed_release_gates
                ),
            ));
        }

        let manual_bundle_workflow = release_checklist_manual_bundle_workflow().len();
        if self.manual_bundle_workflow_items != manual_bundle_workflow {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary manual bundle workflow count mismatch: expected {}, found {}",
                    manual_bundle_workflow, self.manual_bundle_workflow_items
                ),
            ));
        }

        let bundle_contents = release_checklist_bundle_contents().len();
        if self.bundle_contents_items != bundle_contents {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary bundle contents count mismatch: expected {}, found {}",
                    bundle_contents, self.bundle_contents_items
                ),
            ));
        }

        let external_publishing_reminders = release_checklist_external_publishing_reminders().len();
        if self.external_publishing_reminders != external_publishing_reminders {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary external publishing reminder count mismatch: expected {}, found {}",
                    external_publishing_reminders, self.external_publishing_reminders
                ),
            ));
        }

        Ok(())
    }

    /// Returns the compact summary used in release-facing reports.
    pub fn summary_line(&self) -> String {
        format!(
            "v{} compatibility={}, api-stability={}; repository-managed release gates={}; manual bundle workflow={}; bundle contents={}; external publishing reminders={}",
            ReleaseProfileIdentifiers::schema_version(),
            self.release_profile_identifiers.compatibility_profile_id,
            self.release_profile_identifiers.api_stability_profile_id,
            self.repository_managed_release_gates,
            self.manual_bundle_workflow_items,
            self.bundle_contents_items,
            self.external_publishing_reminders,
        )
    }

    /// Returns a compact summary line after validating the release posture.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReleaseChecklistSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the compact release-checklist summary derived from the release posture.
pub fn release_checklist_summary() -> ReleaseChecklistSummary {
    ReleaseChecklistSummary {
        release_profile_identifiers: current_release_profile_identifiers(),
        repository_managed_release_gates: release_checklist_repository_managed_release_gates()
            .len(),
        manual_bundle_workflow_items: release_checklist_manual_bundle_workflow().len(),
        bundle_contents_items: release_checklist_bundle_contents().len(),
        external_publishing_reminders: release_checklist_external_publishing_reminders().len(),
    }
}

/// Returns the compact workspace-audit summary derived from the detailed report.
pub fn workspace_audit_summary(report: &WorkspaceAuditReport) -> WorkspaceAuditSummary {
    WorkspaceAuditSummary {
        workspace_root: report.workspace_root.clone(),
        manifest_count: report.manifest_paths.len(),
        lockfile_path: report.lockfile_path.clone(),
        violation_count: report.violations.len(),
        rule_counts: workspace_audit_rule_counts(&report.violations)
            .into_iter()
            .collect(),
        clean: report.is_clean(),
    }
}

fn workspace_audit_rule_counts(
    violations: &[WorkspaceAuditViolation],
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for violation in violations {
        *counts.entry(violation.rule).or_insert(0) += 1;
    }
    counts
}

impl fmt::Display for WorkspaceAuditReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Workspace audit")?;
        writeln!(f, "Workspace root: {}", self.workspace_root.display())?;
        writeln!(f, "Checked manifests: {}", self.manifest_paths.len())?;
        writeln!(f, "Checked lockfile: {}", self.lockfile_path.display())?;
        if self.violations.is_empty() {
            writeln!(f, "Result: no mandatory native build hooks detected")?;
            return Ok(());
        }

        writeln!(f, "Result: violations found")?;
        writeln!(f, "Rule counts:")?;
        for (rule, count) in workspace_audit_rule_counts(&self.violations) {
            writeln!(f, "  {}: {}", rule, count)?;
        }
        for violation in &self.violations {
            writeln!(
                f,
                "- {} [{}]: {}",
                violation.path.display(),
                violation.rule,
                violation.detail
            )?;
        }
        Ok(())
    }
}

fn render_workspace_audit_summary_text(report: &WorkspaceAuditReport) -> String {
    let summary = workspace_audit_summary(report);
    match summary.validated_summary_line() {
        Ok(summary_line) => {
            let mut text = String::new();
            text.push_str("Workspace audit summary\n");
            text.push_str("Workspace root: ");
            text.push_str(&summary.workspace_root.display().to_string());
            text.push('\n');
            text.push_str("Checked manifests: ");
            text.push_str(&summary.manifest_count.to_string());
            text.push('\n');
            text.push_str("Checked lockfile: ");
            text.push_str(&summary.lockfile_path.display().to_string());
            text.push('\n');
            text.push_str("Summary: ");
            text.push_str(&summary_line);
            text.push('\n');
            text.push_str("Violations: ");
            text.push_str(&summary.violation_count.to_string());
            text.push('\n');
            if !summary.clean {
                text.push_str("Rule counts:\n");
                for (rule, count) in &summary.rule_counts {
                    text.push_str("  ");
                    text.push_str(rule);
                    text.push_str(": ");
                    text.push_str(&count.to_string());
                    text.push('\n');
                }
            }
            text.push_str("Result: ");
            text.push_str(if summary.clean {
                "no mandatory native build hooks detected"
            } else {
                "violations found"
            });
            text.push('\n');
            text
        }
        Err(error) => format!("Workspace audit summary unavailable ({error})"),
    }
}

/// Renders a compact workspace audit summary used by the CLI and release bundle.
pub fn render_workspace_audit_summary() -> Result<String, std::io::Error> {
    let report = workspace_audit_report()?;
    Ok(render_workspace_audit_summary_text(&report))
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Err(error) = self.validate() {
            return write!(f, "Validation report unavailable ({error})");
        }

        writeln!(f, "Validation report")?;
        writeln!(f)?;
        let release_profiles = current_release_profile_identifiers();
        writeln!(f, "Compatibility profile")?;
        writeln!(f, "  id: {}", release_profiles.compatibility_profile_id)?;
        writeln!(f, "{}", current_compatibility_profile())?;
        writeln!(f)?;
        writeln!(f, "API stability posture")?;
        writeln!(f, "  id: {}", release_profiles.api_stability_profile_id)?;
        writeln!(
            f,
            "Release profile identifiers: {}",
            format_release_profile_identifiers_summary(&release_profiles)
        )?;
        writeln!(f, "{}", current_api_stability_profile())?;
        writeln!(f)?;
        write_backend_catalog(
            f,
            "Implemented backend matrices",
            &implemented_backend_catalog(),
        )?;
        writeln!(f)?;
        write_reference_asteroid_section(f)?;
        writeln!(f)?;
        writeln!(f, "Comparison corpus")?;
        write_corpus_summary(f, &self.comparison_corpus)?;
        writeln!(
            f,
            "  release-grade guard: {}",
            comparison_corpus_release_guard_summary()
        )?;
        writeln!(f, "  {}", comparison_snapshot_summary_for_report())?;
        writeln!(
            f,
            "  {}",
            comparison_snapshot_body_class_coverage_summary_for_report()
        )?;
        writeln!(
            f,
            "  {}",
            comparison_snapshot_batch_parity_summary_for_report()
        )?;
        writeln!(f)?;
        writeln!(f, "Benchmark corpus")?;
        write_corpus_summary(f, &self.benchmark_corpus)?;
        writeln!(f)?;
        writeln!(f, "Packaged-data benchmark corpus")?;
        write_corpus_summary(f, &self.packaged_benchmark_corpus)?;
        writeln!(f)?;
        writeln!(f, "Chart benchmark corpus")?;
        write_corpus_summary(f, &self.chart_benchmark_corpus)?;
        writeln!(f)?;
        writeln!(f, "{}", self.house_validation)?;
        writeln!(f)?;
        writeln!(f, "Reference backend")?;
        write_backend_matrix(f, &self.comparison.reference_backend)?;
        writeln!(f)?;
        writeln!(f, "Candidate backend")?;
        write_backend_matrix(f, &self.comparison.candidate_backend)?;
        writeln!(f)?;
        writeln!(f, "Comparison summary")?;
        write_comparison_summary(f, &self.comparison)?;
        writeln!(f)?;
        write_body_comparison_summaries(f, &self.comparison.body_summaries())?;
        writeln!(f)?;
        write_body_class_envelopes(f, &self.comparison.samples)?;
        writeln!(f)?;
        write_body_class_tolerance_posture(
            f,
            &self.comparison.samples,
            &self.comparison.candidate_backend.family,
        )?;
        writeln!(f)?;
        write_tolerance_policy(f, &self.comparison)?;
        writeln!(f)?;
        write_tolerance_summaries(f, &self.comparison.tolerance_summaries())?;
        writeln!(f)?;
        write_regression_section(
            f,
            "Notable regressions",
            &self.comparison.notable_regressions(),
        )?;
        writeln!(f)?;
        write_regression_archive_section(f, &self.archived_regressions)?;
        writeln!(f)?;
        writeln!(f, "{}", benchmark_provenance_text())?;
        writeln!(f)?;
        writeln!(f, "Benchmark summaries")?;
        writeln!(f, "Reference benchmark")?;
        writeln!(f, "  corpus: {}", self.reference_benchmark.corpus_name)?;
        writeln!(
            f,
            "  ns/request (single): {}",
            format_ns(self.reference_benchmark.nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "  ns/request (batch): {}",
            format_ns(self.reference_benchmark.batch_nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "  estimated corpus heap footprint: {} bytes",
            self.reference_benchmark.estimated_corpus_heap_bytes
        )?;
        writeln!(
            f,
            "  batch throughput: {:.2} req/s",
            self.reference_benchmark.batch_requests_per_second()
        )?;
        writeln!(f)?;
        writeln!(f, "Candidate benchmark")?;
        writeln!(f, "  corpus: {}", self.candidate_benchmark.corpus_name)?;
        writeln!(
            f,
            "  ns/request (single): {}",
            format_ns(self.candidate_benchmark.nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "  ns/request (batch): {}",
            format_ns(self.candidate_benchmark.batch_nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "  estimated corpus heap footprint: {} bytes",
            self.candidate_benchmark.estimated_corpus_heap_bytes
        )?;
        writeln!(
            f,
            "  batch throughput: {:.2} req/s",
            self.candidate_benchmark.batch_requests_per_second()
        )?;
        writeln!(f)?;
        writeln!(f, "Packaged-data benchmark")?;
        writeln!(f, "  corpus: {}", self.packaged_benchmark.corpus_name)?;
        writeln!(
            f,
            "  ns/request (single): {}",
            format_ns(self.packaged_benchmark.nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "  ns/request (batch): {}",
            format_ns(self.packaged_benchmark.batch_nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "  estimated corpus heap footprint: {} bytes",
            self.packaged_benchmark.estimated_corpus_heap_bytes
        )?;
        writeln!(
            f,
            "  batch throughput: {:.2} req/s",
            self.packaged_benchmark.batch_requests_per_second()
        )?;
        writeln!(f)?;
        writeln!(f, "Packaged artifact decode benchmark")?;
        writeln!(
            f,
            "  artifact: {}",
            self.artifact_decode_benchmark.artifact_label
        )?;
        writeln!(f, "  source: {}", self.artifact_decode_benchmark.source)?;
        writeln!(f, "  rounds: {}", self.artifact_decode_benchmark.rounds)?;
        writeln!(
            f,
            "  decodes per round: {}",
            self.artifact_decode_benchmark.sample_count
        )?;
        writeln!(
            f,
            "  encoded bytes: {}",
            self.artifact_decode_benchmark.encoded_bytes
        )?;
        writeln!(
            f,
            "  decode elapsed: {}",
            format_duration(self.artifact_decode_benchmark.elapsed)
        )?;
        writeln!(
            f,
            "  ns/decode: {}",
            format_ns(self.artifact_decode_benchmark.nanoseconds_per_decode())
        )?;
        writeln!(
            f,
            "  decodes per second: {:.2} decodes/s",
            self.artifact_decode_benchmark.decodes_per_second()
        )?;
        writeln!(f)?;
        writeln!(f, "Chart benchmark")?;
        writeln!(f, "  corpus: {}", self.chart_benchmark.corpus_name)?;
        writeln!(
            f,
            "  ns/chart: {}",
            format_ns(self.chart_benchmark.nanoseconds_per_chart())
        )?;
        writeln!(
            f,
            "  estimated corpus heap footprint: {} bytes",
            self.chart_benchmark.estimated_corpus_heap_bytes
        )?;
        writeln!(
            f,
            "  charts per second: {:.2} charts/s",
            self.chart_benchmark.charts_per_second()
        )?;
        writeln!(f)?;
        writeln!(f, "Samples")?;
        for sample in &self.comparison.samples {
            writeln!(
                f,
                "  {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}",
                sample.body,
                sample.longitude_delta_deg,
                sample.latitude_delta_deg,
                sample
                    .distance_delta_au
                    .map(|value| format!("{value:.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string())
            )?;
        }
        Ok(())
    }
}

impl fmt::Display for ReleaseBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Release bundle")?;
        writeln!(f, "  output directory: {}", self.output_dir.display())?;
        writeln!(
            f,
            "  compatibility profile: {}",
            self.compatibility_profile_path.display()
        )?;
        writeln!(
            f,
            "  compatibility profile summary: {}",
            self.compatibility_profile_summary_path.display()
        )?;
        writeln!(f, "  release notes: {}", self.release_notes_path.display())?;
        writeln!(
            f,
            "  release notes summary: {}",
            self.release_notes_summary_path.display()
        )?;
        writeln!(
            f,
            "  release summary: {}",
            self.release_summary_path.display()
        )?;
        writeln!(
            f,
            "  release-profile identifiers: {}",
            self.release_profile_identifiers_path.display()
        )?;
        writeln!(
            f,
            "  release checklist: {}",
            self.release_checklist_path.display()
        )?;
        writeln!(
            f,
            "  release checklist summary: {}",
            self.release_checklist_summary_path.display()
        )?;
        writeln!(
            f,
            "  backend matrix: {}",
            self.backend_matrix_path.display()
        )?;
        writeln!(
            f,
            "  backend matrix summary: {}",
            self.backend_matrix_summary_path.display()
        )?;
        writeln!(
            f,
            "  API stability posture: {}",
            self.api_stability_path.display()
        )?;
        writeln!(
            f,
            "  API stability summary: {}",
            self.api_stability_summary_path.display()
        )?;
        writeln!(
            f,
            "  comparison-envelope summary: {}",
            self.comparison_envelope_summary_path.display()
        )?;
        writeln!(
            f,
            "  comparison-corpus release-guard summary: {}",
            self.comparison_corpus_release_guard_summary_path.display()
        )?;
        writeln!(
            f,
            "  validation report summary: {}",
            self.validation_report_summary_path.display()
        )?;
        writeln!(
            f,
            "  workspace audit summary: {}",
            self.workspace_audit_summary_path.display()
        )?;
        writeln!(
            f,
            "  artifact summary: {}",
            self.artifact_summary_path.display()
        )?;
        writeln!(
            f,
            "  benchmark report: {}",
            self.benchmark_report_path.display()
        )?;
        writeln!(
            f,
            "  validation report: {}",
            self.validation_report_path.display()
        )?;
        writeln!(f, "  manifest: {}", self.manifest_path.display())?;
        writeln!(
            f,
            "  manifest checksum sidecar: {}",
            self.manifest_checksum_path.display()
        )?;
        writeln!(f, "  source revision: {}", self.source_revision)?;
        writeln!(f, "  workspace status: {}", self.workspace_status)?;
        writeln!(f, "  rustc version: {}", self.rustc_version)?;
        writeln!(f, "  validation rounds: {}", self.validation_rounds)?;
        writeln!(
            f,
            "  compatibility profile bytes: {}",
            self.compatibility_profile_bytes
        )?;
        writeln!(
            f,
            "  compatibility profile summary bytes: {}",
            self.compatibility_profile_summary_bytes
        )?;
        writeln!(f, "  release notes bytes: {}", self.release_notes_bytes)?;
        writeln!(
            f,
            "  release notes summary bytes: {}",
            self.release_notes_summary_bytes
        )?;
        writeln!(f, "  release summary bytes: {}", self.release_summary_bytes)?;
        writeln!(
            f,
            "  release-profile identifiers bytes: {}",
            self.release_profile_identifiers_bytes
        )?;
        writeln!(
            f,
            "  release checklist bytes: {}",
            self.release_checklist_bytes
        )?;
        writeln!(
            f,
            "  release checklist summary bytes: {}",
            self.release_checklist_summary_bytes
        )?;
        writeln!(
            f,
            "  compatibility profile checksum: 0x{:016x}",
            self.compatibility_profile_checksum
        )?;
        writeln!(
            f,
            "  compatibility profile summary checksum: 0x{:016x}",
            self.compatibility_profile_summary_checksum
        )?;
        writeln!(
            f,
            "  release notes checksum: 0x{:016x}",
            self.release_notes_checksum
        )?;
        writeln!(
            f,
            "  release notes summary checksum: 0x{:016x}",
            self.release_notes_summary_checksum
        )?;
        writeln!(
            f,
            "  release summary checksum: 0x{:016x}",
            self.release_summary_checksum
        )?;
        writeln!(
            f,
            "  release-profile identifiers checksum: 0x{:016x}",
            self.release_profile_identifiers_checksum
        )?;
        writeln!(
            f,
            "  release checklist checksum: 0x{:016x}",
            self.release_checklist_checksum
        )?;
        writeln!(
            f,
            "  release checklist summary checksum: 0x{:016x}",
            self.release_checklist_summary_checksum
        )?;
        writeln!(f, "  backend matrix bytes: {}", self.backend_matrix_bytes)?;
        writeln!(
            f,
            "  backend matrix summary bytes: {}",
            self.backend_matrix_summary_bytes
        )?;
        writeln!(
            f,
            "  backend matrix checksum: 0x{:016x}",
            self.backend_matrix_checksum
        )?;
        writeln!(
            f,
            "  backend matrix summary checksum: 0x{:016x}",
            self.backend_matrix_summary_checksum
        )?;
        writeln!(
            f,
            "  API stability posture bytes: {}",
            self.api_stability_bytes
        )?;
        writeln!(
            f,
            "  API stability summary bytes: {}",
            self.api_stability_summary_bytes
        )?;
        writeln!(
            f,
            "  API stability posture checksum: 0x{:016x}",
            self.api_stability_checksum
        )?;
        writeln!(
            f,
            "  API stability summary checksum: 0x{:016x}",
            self.api_stability_summary_checksum
        )?;
        writeln!(
            f,
            "  comparison-envelope summary bytes: {}",
            self.comparison_envelope_summary_bytes
        )?;
        writeln!(
            f,
            "  comparison-corpus release-guard summary bytes: {}",
            self.comparison_corpus_release_guard_summary_bytes
        )?;
        writeln!(
            f,
            "  validation report summary bytes: {}",
            self.validation_report_summary_bytes
        )?;
        writeln!(
            f,
            "  workspace audit summary bytes: {}",
            self.workspace_audit_summary_bytes
        )?;
        writeln!(
            f,
            "  artifact summary bytes: {}",
            self.artifact_summary_bytes
        )?;
        writeln!(
            f,
            "  benchmark report bytes: {}",
            self.benchmark_report_bytes
        )?;
        writeln!(
            f,
            "  validation report bytes: {}",
            self.validation_report_bytes
        )?;
        writeln!(
            f,
            "  manifest checksum bytes: {}",
            self.manifest_checksum_bytes
        )?;
        writeln!(
            f,
            "  comparison-envelope summary checksum: 0x{:016x}",
            self.comparison_envelope_summary_checksum
        )?;
        writeln!(
            f,
            "  comparison-corpus release-guard summary checksum: 0x{:016x}",
            self.comparison_corpus_release_guard_summary_checksum
        )?;
        writeln!(
            f,
            "  validation report summary checksum: 0x{:016x}",
            self.validation_report_summary_checksum
        )?;
        writeln!(
            f,
            "  workspace audit summary checksum: 0x{:016x}",
            self.workspace_audit_summary_checksum
        )?;
        writeln!(
            f,
            "  artifact summary checksum: 0x{:016x}",
            self.artifact_summary_checksum
        )?;
        writeln!(
            f,
            "  benchmark report checksum: 0x{:016x}",
            self.benchmark_report_checksum
        )?;
        writeln!(
            f,
            "  validation report checksum: 0x{:016x}",
            self.validation_report_checksum
        )?;
        writeln!(f, "  manifest checksum: 0x{:016x}", self.manifest_checksum)
    }
}

impl ReleaseBundle {
    /// Validates the release bundle metadata before it is surfaced to callers.
    ///
    /// The provenance fields are required to stay canonical one-line values so the
    /// release-bundle summary and manifest text remain stable.
    pub fn validate(&self) -> Result<(), ReleaseBundleError> {
        for (path, expected_name, label) in [
            (
                &self.compatibility_profile_path,
                "compatibility-profile.txt",
                "compatibility profile",
            ),
            (
                &self.compatibility_profile_summary_path,
                "compatibility-profile-summary.txt",
                "compatibility profile summary",
            ),
            (
                &self.release_notes_path,
                "release-notes.txt",
                "release notes",
            ),
            (
                &self.release_notes_summary_path,
                "release-notes-summary.txt",
                "release notes summary",
            ),
            (
                &self.release_summary_path,
                "release-summary.txt",
                "release summary",
            ),
            (
                &self.release_profile_identifiers_path,
                "release-profile-identifiers.txt",
                "release-profile identifiers",
            ),
            (
                &self.release_checklist_path,
                "release-checklist.txt",
                "release checklist",
            ),
            (
                &self.release_checklist_summary_path,
                "release-checklist-summary.txt",
                "release checklist summary",
            ),
            (
                &self.backend_matrix_path,
                "backend-matrix.txt",
                "backend matrix",
            ),
            (
                &self.backend_matrix_summary_path,
                "backend-matrix-summary.txt",
                "backend matrix summary",
            ),
            (
                &self.api_stability_path,
                "api-stability.txt",
                "API stability",
            ),
            (
                &self.api_stability_summary_path,
                "api-stability-summary.txt",
                "API stability summary",
            ),
            (
                &self.validation_report_summary_path,
                "validation-report-summary.txt",
                "validation report summary",
            ),
            (
                &self.workspace_audit_summary_path,
                "workspace-audit-summary.txt",
                "workspace audit summary",
            ),
            (
                &self.artifact_summary_path,
                "artifact-summary.txt",
                "artifact summary",
            ),
            (
                &self.packaged_artifact_generation_manifest_path,
                "packaged-artifact-generation-manifest.txt",
                "packaged-artifact generation manifest",
            ),
            (
                &self.benchmark_report_path,
                "benchmark-report.txt",
                "benchmark report",
            ),
            (
                &self.validation_report_path,
                "validation-report.txt",
                "validation report",
            ),
            (
                &self.manifest_path,
                "bundle-manifest.txt",
                "bundle manifest",
            ),
            (
                &self.manifest_checksum_path,
                "bundle-manifest.checksum.txt",
                "bundle manifest checksum sidecar",
            ),
        ] {
            let expected_path = self.output_dir.join(expected_name);
            if path != &expected_path {
                return Err(ReleaseBundleError::Verification(format!(
                    "unexpected {label} file path: expected {}, found {}",
                    expected_path.display(),
                    path.display()
                )));
            }
        }

        ensure_canonical_manifest_value(&self.source_revision, "source revision")?;
        ensure_canonical_manifest_value(&self.workspace_status, "workspace status")?;
        ensure_canonical_manifest_value(&self.rustc_version, "rustc version")?;
        if self.validation_rounds == 0 {
            return Err(ReleaseBundleError::Verification(
                "release bundle validation rounds must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Builds the default validation corpus.
pub fn default_corpus() -> ValidationCorpus {
    ValidationCorpus::jpl_snapshot()
}

/// Builds the release-grade comparison corpus with Pluto excluded from tolerance evidence.
pub fn release_grade_corpus() -> ValidationCorpus {
    let mut corpus = default_corpus();
    corpus.name = "JPL Horizons release-grade comparison window".to_string();
    corpus.description = "Release-grade comparison corpus built from the checked-in JPL Horizons snapshot, with Pluto excluded from tolerance evidence because Pluto remains an approximate fallback; the 2451913.5 boundary day stays out of the audit slice to preserve the current release-grade comparison window.";
    corpus
        .requests
        .retain(|request| request.body != CelestialBody::Pluto);
    corpus
}

/// Returns the CLI banner.
pub fn banner() -> &'static str {
    BANNER
}

/// Creates the default benchmark corpus.
pub fn benchmark_corpus() -> ValidationCorpus {
    ValidationCorpus::representative_window()
}

/// Renders the command-line interface output.
pub fn render_cli(args: &[&str]) -> Result<String, String> {
    match args.first().copied() {
        Some("compare-backends") => {
            ensure_no_extra_args(&args[1..], "compare-backends")?;
            render_comparison_report().map_err(render_error)
        }
        Some("compare-backends-audit") => {
            ensure_no_extra_args(&args[1..], "compare-backends-audit")?;
            render_comparison_audit_report()
        }
        Some("backend-matrix") | Some("capability-matrix") => {
            ensure_no_extra_args(&args[1..], "backend-matrix")?;
            render_backend_matrix_report().map_err(render_error)
        }
        Some("backend-matrix-summary") | Some("matrix-summary") => {
            ensure_no_extra_args(&args[1..], "backend-matrix-summary")?;
            Ok(render_backend_matrix_summary())
        }
        Some("compatibility-profile") | Some("profile") => {
            ensure_no_extra_args(&args[1..], "compatibility-profile")?;
            validated_compatibility_profile_for_report().map(|profile| profile.to_string())
        }
        Some("benchmark") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_benchmark_report(rounds).map_err(render_error)
        }
        Some("comparison-corpus-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-summary")?;
            Ok(render_comparison_corpus_summary_text())
        }
        Some("comparison-corpus-release-guard-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-release-guard-summary")?;
            Ok(render_comparison_corpus_release_guard_summary_text())
        }
        Some("comparison-corpus-guard-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-corpus-release-guard-summary")?;
            Ok(render_comparison_corpus_release_guard_summary_text())
        }
        Some("benchmark-corpus-summary") => {
            ensure_no_extra_args(&args[1..], "benchmark-corpus-summary")?;
            Ok(render_benchmark_corpus_summary_text())
        }
        Some("report") | Some("generate-report") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_validation_report(rounds).map_err(render_error)
        }
        Some("validation-report-summary") | Some("report-summary") | Some("validation-summary") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_validation_report_summary(rounds).map_err(render_error)
        }
        Some("validate-artifact") => {
            ensure_no_extra_args(&args[1..], "validate-artifact")?;
            render_artifact_report().map_err(render_artifact_error)
        }
        Some("regenerate-packaged-artifact") => {
            match parse_packaged_artifact_command(&args[1..])? {
                PackagedArtifactCommand::Write { output_path } => {
                    render_packaged_artifact_regeneration(output_path)
                }
                PackagedArtifactCommand::Check => render_packaged_artifact_regeneration_check(),
            }
        }
        Some("artifact-summary") | Some("artifact-posture-summary") => {
            ensure_no_extra_args(&args[1..], "artifact-summary")?;
            render_artifact_summary().map_err(render_artifact_error)
        }
        Some("artifact-boundary-envelope-summary") => {
            ensure_no_extra_args(&args[1..], "artifact-boundary-envelope-summary")?;
            artifact_boundary_envelope_summary_for_report()
                .map(|summary| summary.summary_line())
                .map_err(render_artifact_error)
        }
        Some("artifact-profile-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "artifact-profile-coverage-summary")?;
            Ok(format!(
                "Artifact profile coverage: {}",
                packaged_artifact_profile_coverage_summary_for_report()
            ))
        }
        Some("packaged-artifact-output-support-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-output-support-summary")?;
            Ok(format!(
                "Packaged-artifact output support: {}",
                packaged_artifact_output_support_summary_for_report()
            ))
        }
        Some("packaged-artifact-storage-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-storage-summary")?;
            Ok(format!(
                "Packaged-artifact storage/reconstruction: {}",
                packaged_artifact_storage_summary_for_report()
            ))
        }
        Some("packaged-artifact-production-profile-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-production-profile-summary")?;
            Ok(packaged_artifact_production_profile_summary_for_report())
        }
        Some("packaged-artifact-target-threshold-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-target-threshold-summary")?;
            Ok(format!(
                "Packaged-artifact target thresholds: {}",
                packaged_artifact_target_threshold_summary_for_report()
            ))
        }
        Some("packaged-artifact-target-threshold-scope-envelopes-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-target-threshold-scope-envelopes-summary",
            )?;
            Ok(format!(
                "Packaged-artifact target-threshold scope envelopes: {}",
                packaged_artifact_target_threshold_scope_envelopes_for_report()
            ))
        }
        Some("packaged-artifact-generation-manifest-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-generation-manifest-summary")?;
            Ok(packaged_artifact_generation_manifest_for_report())
        }
        Some("packaged-artifact-generation-policy-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-generation-policy-summary")?;
            Ok(packaged_artifact_generation_policy_summary_for_report())
        }
        Some("packaged-artifact-generation-residual-bodies-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "packaged-artifact-generation-residual-bodies-summary",
            )?;
            Ok(format!(
                "Packaged-artifact generation residual bodies: {}",
                packaged_artifact_generation_residual_bodies_summary_for_report()
            ))
        }
        Some("packaged-artifact-regeneration-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-artifact-regeneration-summary")?;
            Ok(format!(
                "Packaged-artifact regeneration: {}",
                packaged_artifact_regeneration_summary_for_report()
            ))
        }
        Some("packaged-lookup-epoch-policy-summary") => {
            ensure_no_extra_args(&args[1..], "packaged-lookup-epoch-policy-summary")?;
            Ok(packaged_lookup_epoch_policy_summary_for_report())
        }
        Some("workspace-audit") | Some("audit") | Some("native-dependency-audit") => {
            ensure_no_extra_args(&args[1..], "workspace-audit")?;
            let report = workspace_audit_report().map_err(|error| error.to_string())?;
            if report.is_clean() {
                Ok(report.to_string())
            } else {
                Err(format!("workspace audit failed:\n{report}"))
            }
        }
        Some("workspace-audit-summary") | Some("native-dependency-audit-summary") => {
            ensure_no_extra_args(&args[1..], "workspace-audit-summary")?;
            render_workspace_audit_summary().map_err(|error| error.to_string())
        }
        Some("api-stability-summary") | Some("api-posture-summary") => {
            ensure_no_extra_args(&args[1..], "api-stability-summary")?;
            Ok(render_api_stability_summary())
        }
        Some("api-stability") | Some("api-posture") => {
            ensure_no_extra_args(&args[1..], "api-stability")?;
            Ok(current_api_stability_profile().to_string())
        }
        Some("compatibility-profile-summary") | Some("profile-summary") => {
            ensure_no_extra_args(&args[1..], "compatibility-profile-summary")?;
            Ok(render_compatibility_profile_summary())
        }
        Some("catalog-inventory-summary") => {
            ensure_no_extra_args(&args[1..], "catalog-inventory-summary")?;
            Ok(render_catalog_inventory_summary())
        }
        Some("verify-compatibility-profile") => {
            ensure_no_extra_args(&args[1..], "verify-compatibility-profile")?;
            verify_compatibility_profile().map_err(render_error)
        }
        Some("release-notes") => {
            ensure_no_extra_args(&args[1..], "release-notes")?;
            Ok(render_release_notes_text())
        }
        Some("release-notes-summary") => {
            ensure_no_extra_args(&args[1..], "release-notes-summary")?;
            Ok(render_release_notes_summary_text())
        }
        Some("release-checklist") => {
            ensure_no_extra_args(&args[1..], "release-checklist")?;
            Ok(render_release_checklist_text())
        }
        Some("release-checklist-summary") | Some("checklist-summary") => {
            ensure_no_extra_args(&args[1..], "release-checklist-summary")?;
            Ok(render_release_checklist_summary_text())
        }
        Some("release-summary") => {
            ensure_no_extra_args(&args[1..], "release-summary")?;
            Ok(render_release_summary_text())
        }
        Some("jpl-batch-error-taxonomy-summary") => {
            ensure_no_extra_args(&args[1..], "jpl-batch-error-taxonomy-summary")?;
            Ok(jpl_snapshot_batch_error_taxonomy_summary_for_report())
        }
        Some("jpl-snapshot-evidence-summary") => {
            ensure_no_extra_args(&args[1..], "jpl-snapshot-evidence-summary")?;
            Ok(jpl_snapshot_evidence_summary_for_report())
        }
        Some("production-generation-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-summary")?;
            Ok(production_generation_boundary_summary_for_report())
        }
        Some("production-generation-boundary-request-corpus-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "production-generation-boundary-request-corpus-summary",
            )?;
            Ok(production_generation_boundary_request_corpus_summary_for_report())
        }
        Some("production-generation-body-class-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "production-generation-body-class-coverage-summary",
            )?;
            Ok(production_generation_snapshot_body_class_coverage_summary_for_report())
        }
        Some("production-generation-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-source-window-summary")?;
            Ok(production_generation_snapshot_window_summary_for_report())
        }
        Some("production-generation-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-summary")?;
            Ok(production_generation_snapshot_summary_for_report())
        }
        Some("production-generation-boundary-source-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-source-summary")?;
            Ok(production_generation_boundary_source_summary_for_report())
        }
        Some("production-generation-boundary-window-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-boundary-window-summary")?;
            Ok(production_generation_boundary_window_summary_for_report())
        }
        Some("production-generation-source-summary") => {
            ensure_no_extra_args(&args[1..], "production-generation-source-summary")?;
            Ok(production_generation_source_summary_for_report())
        }
        Some("comparison-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source-summary")?;
            Ok(comparison_snapshot_source_summary_for_report())
        }
        Some("comparison-snapshot-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-source-window-summary")?;
            Ok(comparison_snapshot_source_window_summary_for_report())
        }
        Some("comparison-snapshot-body-class-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "comparison-snapshot-body-class-coverage-summary",
            )?;
            Ok(comparison_snapshot_body_class_coverage_summary_for_report())
        }
        Some("comparison-snapshot-manifest-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-manifest-summary")?;
            Ok(comparison_snapshot_manifest_summary_for_report())
        }
        Some("comparison-snapshot-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-summary")?;
            Ok(render_comparison_snapshot_summary_text())
        }
        Some("comparison-snapshot-batch-parity-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-snapshot-batch-parity-summary")?;
            Ok(comparison_snapshot_batch_parity_summary_for_report())
        }
        Some("reference-snapshot-source-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source-summary")?;
            Ok(reference_snapshot_source_summary_for_report())
        }
        Some("reference-snapshot-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-source-window-summary")?;
            Ok(reference_snapshot_source_window_summary_for_report())
        }
        Some("reference-snapshot-lunar-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-lunar-boundary-summary")?;
            Ok(reference_snapshot_lunar_boundary_summary_for_report())
        }
        Some("reference-snapshot-1749-major-body-boundary-summary")
        | Some("1749-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1749-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1749_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-early-major-body-boundary-summary")
        | Some("early-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-early-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_early_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-1800-major-body-boundary-summary")
        | Some("1800-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-1800-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_1800_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-2500-major-body-boundary-summary")
        | Some("2500-major-body-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-2500-major-body-boundary-summary",
            )?;
            Ok(reference_snapshot_2500_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-body-class-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-body-class-coverage-summary")?;
            Ok(reference_snapshot_body_class_coverage_summary_for_report())
        }
        Some("reference-snapshot-manifest-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-manifest-summary")?;
            Ok(reference_snapshot_manifest_summary_for_report())
        }
        Some("reference-snapshot-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-summary")?;
            Ok(render_reference_snapshot_summary_text())
        }
        Some("reference-snapshot-batch-parity-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-batch-parity-summary")?;
            Ok(reference_snapshot_batch_parity_summary_for_report())
        }
        Some("reference-snapshot-equatorial-parity-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-equatorial-parity-summary")?;
            Ok(reference_snapshot_equatorial_parity_summary_for_report())
        }
        Some("reference-high-curvature-summary") | Some("high-curvature-summary") => {
            ensure_no_extra_args(&args[1..], "reference-high-curvature-summary")?;
            Ok(reference_snapshot_high_curvature_summary_for_report())
        }
        Some("reference-snapshot-major-body-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-major-body-boundary-summary")?;
            Ok(reference_snapshot_major_body_boundary_summary_for_report())
        }
        Some("reference-snapshot-mars-jupiter-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-mars-jupiter-boundary-summary",
            )?;
            Ok(reference_snapshot_mars_jupiter_boundary_summary_for_report())
        }
        Some("reference-snapshot-mars-outer-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "reference-snapshot-mars-outer-boundary-summary")?;
            Ok(reference_snapshot_mars_outer_boundary_summary_for_report())
        }
        Some("reference-snapshot-major-body-boundary-window-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-major-body-boundary-window-summary",
            )?;
            Ok(reference_snapshot_major_body_boundary_window_summary_for_report())
        }
        Some("reference-high-curvature-window-summary") | Some("high-curvature-window-summary") => {
            ensure_no_extra_args(&args[1..], "reference-high-curvature-window-summary")?;
            Ok(reference_snapshot_high_curvature_window_summary_for_report())
        }
        Some("reference-snapshot-boundary-epoch-coverage-summary")
        | Some("boundary-epoch-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-boundary-epoch-coverage-summary",
            )?;
            Ok(reference_snapshot_boundary_epoch_coverage_summary_for_report())
        }
        Some("source-documentation-summary") => {
            ensure_no_extra_args(&args[1..], "source-documentation-summary")?;
            Ok(source_documentation_summary_for_report())
        }
        Some("source-documentation-health-summary") => {
            ensure_no_extra_args(&args[1..], "source-documentation-health-summary")?;
            Ok(source_documentation_health_summary_for_report())
        }
        Some("source-audit-summary") => {
            ensure_no_extra_args(&args[1..], "source-audit-summary")?;
            Ok(source_audit_summary_for_report())
        }
        Some("generated-binary-audit-summary") => {
            ensure_no_extra_args(&args[1..], "generated-binary-audit-summary")?;
            Ok(generated_binary_audit_summary_for_report())
        }
        Some("time-scale-policy-summary") => {
            ensure_no_extra_args(&args[1..], "time-scale-policy-summary")?;
            Ok(render_time_scale_policy_summary_text())
        }
        Some("delta-t-policy-summary") => {
            ensure_no_extra_args(&args[1..], "delta-t-policy-summary")?;
            Ok(render_delta_t_policy_summary_text())
        }
        Some("observer-policy-summary") => {
            ensure_no_extra_args(&args[1..], "observer-policy-summary")?;
            Ok(render_observer_policy_summary_text())
        }
        Some("apparentness-policy-summary") => {
            ensure_no_extra_args(&args[1..], "apparentness-policy-summary")?;
            Ok(render_apparentness_policy_summary_text())
        }
        Some("interpolation-posture-summary") => {
            ensure_no_extra_args(&args[1..], "interpolation-posture-summary")?;
            Ok(render_interpolation_posture_summary_text())
        }
        Some("interpolation-quality-summary") => {
            ensure_no_extra_args(&args[1..], "interpolation-quality-summary")?;
            Ok(render_interpolation_quality_summary_text())
        }
        Some("interpolation-quality-kind-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "interpolation-quality-kind-coverage-summary")?;
            Ok(jpl_interpolation_quality_kind_coverage_for_report())
        }
        Some("lunar-reference-error-envelope-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-reference-error-envelope-summary")?;
            Ok(render_lunar_reference_error_envelope_summary_text())
        }
        Some("lunar-equatorial-reference-error-envelope-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "lunar-equatorial-reference-error-envelope-summary",
            )?;
            Ok(render_lunar_equatorial_reference_error_envelope_summary_text())
        }
        Some("lunar-apparent-comparison-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-apparent-comparison-summary")?;
            Ok(render_lunar_apparent_comparison_summary_text())
        }
        Some("lunar-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-source-window-summary")?;
            Ok(lunar_source_window_summary_for_report())
        }
        Some("lunar-theory-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-summary")?;
            Ok(lunar_theory_summary_for_report())
        }
        Some("lunar-theory-capability-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-capability-summary")?;
            Ok(lunar_theory_capability_summary_for_report())
        }
        Some("lunar-theory-source-summary") => {
            ensure_no_extra_args(&args[1..], "lunar-theory-source-summary")?;
            Ok(lunar_theory_source_summary_for_report())
        }
        Some("selected-asteroid-boundary-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-boundary-summary")?;
            Ok(selected_asteroid_boundary_summary_for_report())
        }
        Some("reference-snapshot-selected-asteroid-terminal-boundary-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "reference-snapshot-selected-asteroid-terminal-boundary-summary",
            )?;
            Ok(selected_asteroid_terminal_boundary_summary_for_report())
        }
        Some("selected-asteroid-source-evidence-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-evidence-summary")?;
            Ok(selected_asteroid_source_evidence_summary_for_report())
        }
        Some("selected-asteroid-source-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-summary")?;
            Ok(selected_asteroid_source_evidence_summary_for_report())
        }
        Some("selected-asteroid-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-source-window-summary")?;
            Ok(selected_asteroid_source_window_summary_for_report())
        }
        Some("selected-asteroid-batch-parity-summary") => {
            ensure_no_extra_args(&args[1..], "selected-asteroid-batch-parity-summary")?;
            Ok(selected_asteroid_batch_parity_summary_for_report())
        }
        Some("reference-asteroid-evidence-summary") => {
            ensure_no_extra_args(&args[1..], "reference-asteroid-evidence-summary")?;
            Ok(reference_asteroid_evidence_summary_for_report())
        }
        Some("reference-asteroid-equatorial-evidence-summary") => {
            ensure_no_extra_args(&args[1..], "reference-asteroid-equatorial-evidence-summary")?;
            Ok(reference_asteroid_equatorial_evidence_summary_for_report())
        }
        Some("reference-asteroid-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "reference-asteroid-source-window-summary")?;
            Ok(reference_asteroid_source_window_summary_for_report())
        }
        Some("independent-holdout-source-window-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-source-window-summary")?;
            Ok(independent_holdout_snapshot_source_window_summary_for_report())
        }
        Some("independent-holdout-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-summary")?;
            Ok(jpl_independent_holdout_summary_for_report())
        }
        Some("independent-holdout-source-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-source-summary")?;
            Ok(independent_holdout_source_summary_for_report())
        }
        Some("independent-holdout-body-class-coverage-summary") => {
            ensure_no_extra_args(
                &args[1..],
                "independent-holdout-body-class-coverage-summary",
            )?;
            Ok(independent_holdout_snapshot_body_class_coverage_summary_for_report())
        }
        Some("independent-holdout-batch-parity-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-batch-parity-summary")?;
            Ok(jpl_independent_holdout_snapshot_batch_parity_summary_for_report())
        }
        Some("independent-holdout-equatorial-parity-summary") => {
            ensure_no_extra_args(&args[1..], "independent-holdout-equatorial-parity-summary")?;
            Ok(jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report())
        }
        Some("reference-holdout-overlap-summary") => {
            ensure_no_extra_args(&args[1..], "reference-holdout-overlap-summary")?;
            Ok(render_reference_holdout_overlap_summary_text())
        }
        Some("house-validation-summary") => {
            ensure_no_extra_args(&args[1..], "house-validation-summary")?;
            Ok(house_validation_summary_line_for_report(
                &house_validation_report(),
            ))
        }
        Some("house-formula-families-summary") => {
            ensure_no_extra_args(&args[1..], "house-formula-families-summary")?;
            Ok(format_house_formula_families_for_report())
        }
        Some("house-code-aliases-summary") => {
            ensure_no_extra_args(&args[1..], "house-code-aliases-summary")?;
            Ok(format_house_code_aliases_for_report())
        }
        Some("house-code-alias-summary") => {
            ensure_no_extra_args(&args[1..], "house-code-alias-summary")?;
            Ok(format_house_code_aliases_for_report())
        }
        Some("ayanamsa-catalog-validation-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-catalog-validation-summary")?;
            Ok(ayanamsa_catalog_validation_summary().summary_line())
        }
        Some("ayanamsa-metadata-coverage-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-metadata-coverage-summary")?;
            Ok(format_ayanamsa_metadata_coverage_for_report())
        }
        Some("ayanamsa-reference-offsets-summary") => {
            ensure_no_extra_args(&args[1..], "ayanamsa-reference-offsets-summary")?;
            Ok(format_ayanamsa_reference_offsets_for_report())
        }
        Some("frame-policy-summary") => {
            ensure_no_extra_args(&args[1..], "frame-policy-summary")?;
            Ok(render_frame_policy_summary_text())
        }
        Some("mean-obliquity-frame-round-trip-summary") => {
            ensure_no_extra_args(&args[1..], "mean-obliquity-frame-round-trip-summary")?;
            Ok(mean_obliquity_frame_round_trip_summary_for_report())
        }
        Some("release-profile-identifiers-summary") => {
            ensure_no_extra_args(&args[1..], "release-profile-identifiers-summary")?;
            Ok(render_release_profile_identifiers_summary())
        }
        Some("request-surface-summary") => {
            ensure_no_extra_args(&args[1..], "request-surface-summary")?;
            Ok(render_request_surface_summary_text())
        }
        Some("request-policy-summary") => {
            ensure_no_extra_args(&args[1..], "request-policy-summary")?;
            Ok(render_request_policy_summary_text())
        }
        Some("request-semantics-summary") => {
            ensure_no_extra_args(&args[1..], "request-semantics-summary")?;
            Ok(render_request_policy_summary_text())
        }
        Some("comparison-tolerance-policy-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-tolerance-policy-summary")?;
            Ok(render_comparison_tolerance_policy_summary_text())
        }
        Some("comparison-envelope-summary") => {
            ensure_no_extra_args(&args[1..], "comparison-envelope-summary")?;
            Ok(render_comparison_envelope_summary_text())
        }
        Some("pluto-fallback-summary") => {
            ensure_no_extra_args(&args[1..], "pluto-fallback-summary")?;
            Ok(render_pluto_fallback_summary_text())
        }
        Some("bundle-release") => {
            let (output_dir, rounds) =
                parse_release_bundle_args(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_release_bundle(rounds, output_dir)
                .map(|bundle| bundle.to_string())
                .map_err(render_release_bundle_error)
        }
        Some("verify-release-bundle") => {
            let (output_dir, _) = parse_release_bundle_args(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            verify_release_bundle(output_dir)
                .map(|bundle| bundle.to_string())
                .map_err(render_release_bundle_error)
        }
        Some("help") | Some("--help") | Some("-h") | None => Ok(help_text()),
        Some(other) => Err(format!("unknown command: {other}\n\n{}", help_text())),
    }
}

/// Creates the default reference backend.
pub fn default_reference_backend() -> JplSnapshotBackend {
    JplSnapshotBackend::new()
}

/// Creates the default candidate backend.
pub fn default_candidate_backend() -> CompositeBackend<Vsop87Backend, ElpBackend> {
    CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())
}

/// Compares two backends across the supplied corpus.
pub fn compare_backends(
    reference: &dyn EphemerisBackend,
    candidate: &dyn EphemerisBackend,
    corpus: &ValidationCorpus,
) -> Result<ComparisonReport, EphemerisError> {
    let mut samples = Vec::with_capacity(corpus.requests.len());
    let mut summary = ComparisonSummary::default();
    let mut distance_sum = 0.0;
    let mut distance_sum_sq = 0.0;
    let mut distance_count = 0usize;

    for request in &corpus.requests {
        let reference_result = extract_ecliptic(reference.position(request)?)?;
        let candidate_result = extract_ecliptic(candidate.position(request)?)?;
        let longitude_delta_deg =
            angular_delta(reference_result.longitude, candidate_result.longitude);
        let latitude_delta_deg =
            (reference_result.latitude.degrees() - candidate_result.latitude.degrees()).abs();
        let distance_delta_au = match (reference_result.distance_au, candidate_result.distance_au) {
            (Some(reference), Some(candidate)) => Some((reference - candidate).abs()),
            _ => None,
        };

        if let Some(delta) = distance_delta_au {
            distance_sum += delta;
            distance_sum_sq += delta * delta;
            distance_count += 1;
            match summary.max_distance_delta_au {
                Some(current) if delta < current => {}
                _ => {
                    summary.max_distance_delta_au = Some(delta);
                    summary.max_distance_delta_body = Some(request.body.clone());
                }
            }
        }

        summary.sample_count += 1;
        if longitude_delta_deg >= summary.max_longitude_delta_deg {
            summary.max_longitude_delta_deg = longitude_delta_deg;
            summary.max_longitude_delta_body = Some(request.body.clone());
        }
        summary.mean_longitude_delta_deg += longitude_delta_deg;
        if latitude_delta_deg >= summary.max_latitude_delta_deg {
            summary.max_latitude_delta_deg = latitude_delta_deg;
            summary.max_latitude_delta_body = Some(request.body.clone());
        }
        summary.mean_latitude_delta_deg += latitude_delta_deg;

        samples.push(ComparisonSample {
            body: request.body.clone(),
            reference: reference_result,
            candidate: candidate_result,
            longitude_delta_deg,
            latitude_delta_deg,
            distance_delta_au,
        });
    }

    if summary.sample_count > 0 {
        let sample_count = summary.sample_count as f64;
        summary.mean_longitude_delta_deg /= sample_count;
        summary.rms_longitude_delta_deg = (samples
            .iter()
            .map(|sample| sample.longitude_delta_deg * sample.longitude_delta_deg)
            .sum::<f64>()
            / sample_count)
            .sqrt();
        summary.mean_latitude_delta_deg /= sample_count;
        summary.rms_latitude_delta_deg = (samples
            .iter()
            .map(|sample| sample.latitude_delta_deg * sample.latitude_delta_deg)
            .sum::<f64>()
            / sample_count)
            .sqrt();
    }
    if distance_count > 0 {
        summary.mean_distance_delta_au = Some(distance_sum / distance_count as f64);
        summary.rms_distance_delta_au = Some((distance_sum_sq / distance_count as f64).sqrt());
    }

    Ok(ComparisonReport {
        corpus_name: corpus.name.clone(),
        corpus_summary: corpus.summary(),
        apparentness: corpus.apparentness,
        reference_backend: reference.metadata(),
        candidate_backend: candidate.metadata(),
        samples,
        summary,
    })
}

/// Benchmarks a backend against a validation corpus.
pub fn benchmark_backend(
    backend: &dyn EphemerisBackend,
    corpus: &ValidationCorpus,
    rounds: usize,
) -> Result<BenchmarkReport, EphemerisError> {
    let single_start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(backend.position(request)?);
        }
    }
    let elapsed = single_start.elapsed();

    let batch_start = StdInstant::now();
    for _ in 0..rounds {
        std::hint::black_box(backend.positions(&corpus.requests)?);
    }
    let batch_elapsed = batch_start.elapsed();

    let report = BenchmarkReport {
        backend: backend.metadata(),
        corpus_name: corpus.name.clone(),
        apparentness: corpus.apparentness,
        rounds,
        sample_count: corpus.requests.len(),
        elapsed,
        batch_elapsed,
        estimated_corpus_heap_bytes: corpus.estimated_heap_bytes(),
    };
    report.validate()?;
    Ok(report)
}

/// Computes a deterministic 64-bit checksum for bundle text.
fn checksum64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Renders the compact compatibility-profile summary used by release tooling.
pub fn render_compatibility_profile_summary() -> String {
    render_compatibility_profile_summary_text()
}

/// Renders the compact compatibility catalog inventory summary used by release tooling.
pub fn render_catalog_inventory_summary() -> String {
    match validated_compatibility_profile_for_report() {
        Ok(profile) => match profile.validated_catalog_inventory_summary_line() {
            Ok(summary) => summary,
            Err(error) => format!("Compatibility catalog inventory unavailable ({error})"),
        },
        Err(error) => format!("Compatibility catalog inventory unavailable ({error})"),
    }
}

/// Renders the release notes used by release tooling.
pub fn render_release_notes() -> String {
    render_release_notes_text()
}

/// Renders the compact release notes summary used by release tooling.
pub fn render_release_notes_summary() -> String {
    render_release_notes_summary_text()
}

/// Renders the release checklist used by release tooling.
pub fn render_release_checklist() -> String {
    render_release_checklist_text()
}

/// Renders the compact release checklist summary used by release tooling.
pub fn render_release_checklist_summary() -> String {
    render_release_checklist_summary_text()
}

/// Renders the compact release summary used by release tooling.
pub fn render_release_summary() -> String {
    render_release_summary_text()
}

/// Renders the compact Delta T policy summary used by validation and release tooling.
pub fn render_delta_t_policy_summary() -> String {
    render_delta_t_policy_summary_text()
}

/// Renders the compact request-policy summary used by validation and release tooling.
pub fn render_request_policy_summary() -> String {
    render_request_policy_summary_text()
}

/// Renders the compact request-surface inventory used by validation and release tooling.
pub fn render_request_surface_summary() -> String {
    render_request_surface_summary_text()
}

/// A compact summary of the compatibility-profile verification posture.
const RELEASE_POSTURE_SUMMARY: &str = "Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented";

#[derive(Clone, Debug)]
pub struct CompatibilityProfileVerificationSummary {
    /// Release profile identifier that was verified.
    pub profile_id: String,
    /// Number of house-system descriptors checked.
    pub house_system_descriptor_count: usize,
    /// Number of house-system labels checked, including aliases.
    pub house_system_label_count: usize,
    /// Number of Swiss Ephemeris house-table code aliases checked.
    pub house_code_alias_count: usize,
    /// Compact mapping of the Swiss Ephemeris house-table code aliases.
    pub house_code_aliases_summary: String,
    /// Latitude-sensitive house systems exposed by the profile.
    pub latitude_sensitive_house_systems: Vec<String>,
    /// Number of ayanamsa descriptors checked.
    pub ayanamsa_descriptor_count: usize,
    /// Number of ayanamsa labels checked, including aliases.
    pub ayanamsa_label_count: usize,
    /// Number of baseline house-system descriptors.
    pub baseline_house_system_count: usize,
    /// Number of release-specific house-system descriptors.
    pub release_house_system_count: usize,
    /// Number of baseline ayanamsa descriptors.
    pub baseline_ayanamsa_count: usize,
    /// Number of release-specific ayanamsa descriptors.
    pub release_ayanamsa_count: usize,
    /// Release-specific house-system canonical names.
    pub release_house_canonical_names: String,
    /// Release-specific ayanamsa canonical names.
    pub release_ayanamsa_canonical_names: String,
    /// Built-in house formula families surfaced by the profile.
    pub house_formula_family_names: String,
    /// Human-readable release posture summary line.
    pub release_posture: String,
    /// Number of built-in ayanamsa descriptors that carry reference epoch/offset metadata.
    pub ayanamsa_metadata_count: usize,
    /// Number of built-in ayanamsa descriptors that still lack reference epoch/offset metadata.
    pub ayanamsa_metadata_gap_count: usize,
    /// Release-specific custom-definition label names.
    pub custom_definition_label_names: String,
    /// Number of release notes documented in the profile.
    pub release_note_count: usize,
    /// Number of validation reference points documented in the profile.
    pub validation_reference_point_count: usize,
    /// Number of custom-definition labels checked.
    pub custom_definition_label_count: usize,
    /// Number of documented compatibility caveats.
    pub compatibility_caveat_count: usize,
}

impl CompatibilityProfileVerificationSummary {
    /// Validates that the summary still matches the current release profile.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let profile = current_compatibility_profile();
        profile.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("compatibility profile validation failed: {error}"),
            )
        })?;
        let release_profiles = validated_release_profile_identifiers_for_report()
            .map_err(|error| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile verification summary release profile identifiers invalid: {error}"
                    ),
                )
            })?;

        if self.profile_id != release_profiles.compatibility_profile_id {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary profile id mismatch: expected {}, found {}",
                    release_profiles.compatibility_profile_id, self.profile_id
                ),
            ));
        }

        let house_labels_checked = verify_house_system_aliases(profile.house_systems)?;
        if self.house_system_descriptor_count != profile.house_systems.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-system descriptor count mismatch: expected {}, found {}",
                    profile.house_systems.len(), self.house_system_descriptor_count
                ),
            ));
        }
        if self.house_system_label_count != house_labels_checked {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-system label count mismatch: expected {}, found {}",
                    house_labels_checked, self.house_system_label_count
                ),
            ));
        }
        if self.house_code_alias_count != profile.house_code_alias_count() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-code alias count mismatch: expected {}, found {}",
                    profile.house_code_alias_count(), self.house_code_alias_count
                ),
            ));
        }
        let expected_house_code_aliases = profile.house_code_aliases_summary_line();
        if self.house_code_aliases_summary != expected_house_code_aliases {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-code aliases mismatch: expected [{}], found [{}]",
                    expected_house_code_aliases, self.house_code_aliases_summary
                ),
            ));
        }

        let expected_latitude_sensitive = profile.latitude_sensitive_house_systems();
        validate_name_sequence(
            "compatibility profile latitude-sensitive house systems",
            expected_latitude_sensitive.iter().copied(),
        )?;
        if self.latitude_sensitive_house_systems != expected_latitude_sensitive {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary latitude-sensitive house systems mismatch: expected [{}], found [{}]",
                    expected_latitude_sensitive.join(", "),
                    self.latitude_sensitive_house_systems.join(", ")
                ),
            ));
        }

        let ayanamsa_labels_checked = verify_ayanamsa_aliases(profile.ayanamsas)?;
        if self.ayanamsa_descriptor_count != profile.ayanamsas.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa descriptor count mismatch: expected {}, found {}",
                    profile.ayanamsas.len(), self.ayanamsa_descriptor_count
                ),
            ));
        }
        if self.ayanamsa_label_count != ayanamsa_labels_checked {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa label count mismatch: expected {}, found {}",
                    ayanamsa_labels_checked, self.ayanamsa_label_count
                ),
            ));
        }

        if self.baseline_house_system_count != profile.baseline_house_systems.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary baseline house-system count mismatch: expected {}, found {}",
                    profile.baseline_house_systems.len(), self.baseline_house_system_count
                ),
            ));
        }
        if self.release_house_system_count != profile.release_house_systems.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release house-system count mismatch: expected {}, found {}",
                    profile.release_house_systems.len(), self.release_house_system_count
                ),
            ));
        }
        if self.baseline_ayanamsa_count != profile.baseline_ayanamsas.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary baseline ayanamsa count mismatch: expected {}, found {}",
                    profile.baseline_ayanamsas.len(), self.baseline_ayanamsa_count
                ),
            ));
        }
        if self.release_ayanamsa_count != profile.release_ayanamsas.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release ayanamsa count mismatch: expected {}, found {}",
                    profile.release_ayanamsas.len(), self.release_ayanamsa_count
                ),
            ));
        }

        let expected_release_house_names =
            profile.validated_release_house_system_canonical_names_summary_line().map_err(
                |error| {
                    EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "compatibility profile verification summary release house-system canonical names invalid: {error}"
                        ),
                    )
                },
            )?;
        if self.release_house_canonical_names != expected_release_house_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release house-system canonical names mismatch: expected {}, found {}",
                    expected_release_house_names, self.release_house_canonical_names
                ),
            ));
        }

        let expected_release_ayanamsa_names =
            profile.validated_release_ayanamsa_canonical_names_summary_line().map_err(
                |error| {
                    EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "compatibility profile verification summary release ayanamsa canonical names invalid: {error}"
                        ),
                    )
                },
            )?;
        if self.release_ayanamsa_canonical_names != expected_release_ayanamsa_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release ayanamsa canonical names mismatch: expected {}, found {}",
                    expected_release_ayanamsa_names, self.release_ayanamsa_canonical_names
                ),
            ));
        }

        let expected_house_formula_family_names = profile.house_formula_family_names();
        validate_name_sequence(
            "compatibility profile house formula families",
            expected_house_formula_family_names
                .iter()
                .map(String::as_str),
        )?;
        let expected_house_formula_family_names = expected_house_formula_family_names.join(", ");
        if self.house_formula_family_names != expected_house_formula_family_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house formula families mismatch: expected {}, found {}",
                    expected_house_formula_family_names, self.house_formula_family_names
                ),
            ));
        }
        if self.release_posture != RELEASE_POSTURE_SUMMARY {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release posture mismatch: expected {}, found {}",
                    RELEASE_POSTURE_SUMMARY, self.release_posture
                ),
            ));
        }

        let expected_ayanamsa_metadata_count = profile
            .ayanamsas
            .iter()
            .filter(|entry| entry.has_sidereal_metadata())
            .count();
        let expected_ayanamsa_metadata_gap_count =
            profile.ayanamsas.len() - expected_ayanamsa_metadata_count;
        if self.ayanamsa_metadata_count != expected_ayanamsa_metadata_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa metadata count mismatch: expected {}, found {}",
                    expected_ayanamsa_metadata_count, self.ayanamsa_metadata_count
                ),
            ));
        }
        if self.ayanamsa_metadata_gap_count != expected_ayanamsa_metadata_gap_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa metadata gap count mismatch: expected {}, found {}",
                    expected_ayanamsa_metadata_gap_count, self.ayanamsa_metadata_gap_count
                ),
            ));
        }

        let expected_custom_definition_label_names = profile.custom_definition_labels.join(", ");
        if self.custom_definition_label_names != expected_custom_definition_label_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary custom-definition label names mismatch: expected {}, found {}",
                    expected_custom_definition_label_names, self.custom_definition_label_names
                ),
            ));
        }

        if self.release_note_count != profile.release_notes.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release note count mismatch: expected {}, found {}",
                    profile.release_notes.len(), self.release_note_count
                ),
            ));
        }
        if self.validation_reference_point_count != profile.validation_reference_points.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary validation reference point count mismatch: expected {}, found {}",
                    profile.validation_reference_points.len(), self.validation_reference_point_count
                ),
            ));
        }
        let custom_definition_labels_checked =
            verify_custom_definition_labels(profile.custom_definition_labels)?;
        if self.custom_definition_label_count != custom_definition_labels_checked {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary custom-definition label count mismatch: expected {}, found {}",
                    custom_definition_labels_checked, self.custom_definition_label_count
                ),
            ));
        }
        if self.compatibility_caveat_count != profile.known_gaps.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary compatibility caveat count mismatch: expected {}, found {}",
                    profile.known_gaps.len(), self.compatibility_caveat_count
                ),
            ));
        }

        verify_profile_text_section("target-house-scope", profile.target_house_scope)?;
        verify_profile_text_section("target-ayanamsa-scope", profile.target_ayanamsa_scope)?;
        verify_profile_text_section("release-note", profile.release_notes)?;
        verify_profile_text_section(
            "validation-reference-point",
            profile.validation_reference_points,
        )?;
        verify_profile_text_section("compatibility-caveat", profile.known_gaps)?;
        verify_profile_text_sections_are_disjoint(&[
            ("target-house-scope", profile.target_house_scope),
            ("target-ayanamsa-scope", profile.target_ayanamsa_scope),
            ("release-note", profile.release_notes),
            (
                "validation-reference-point",
                profile.validation_reference_points,
            ),
            ("compatibility-caveat", profile.known_gaps),
        ])?;

        Ok(())
    }

    /// Renders the verification summary as compact release-facing text.
    pub fn summary_line(&self) -> String {
        let mut text = String::new();
        text.push_str("Compatibility profile verification\n");
        text.push_str("Profile: ");
        text.push_str(&self.profile_id);
        text.push('\n');
        text.push_str("House systems verified: ");
        text.push_str(&self.house_system_descriptor_count.to_string());
        text.push_str(" descriptors, ");
        text.push_str(&self.house_system_label_count.to_string());
        text.push_str(" labels\n");
        text.push_str("House code aliases verified: ");
        text.push_str(&self.house_code_alias_count.to_string());
        text.push_str(" short-form labels\n");
        text.push_str("House code aliases: ");
        text.push_str(&self.house_code_aliases_summary);
        text.push('\n');
        text.push_str("Alias uniqueness checks: exact and case-insensitive labels verified\n");
        text.push_str("Latitude-sensitive house systems verified: ");
        text.push_str(&self.latitude_sensitive_house_systems.len().to_string());
        text.push_str(" descriptors, ");
        text.push_str(&self.latitude_sensitive_house_systems.len().to_string());
        text.push_str(" labels");
        if !self.latitude_sensitive_house_systems.is_empty() {
            text.push_str(" (");
            text.push_str(&self.latitude_sensitive_house_systems.join(", "));
            text.push(')');
        } else {
            text.push_str(" (none)");
        }
        text.push('\n');
        text.push_str("Ayanamsas verified: ");
        text.push_str(&self.ayanamsa_descriptor_count.to_string());
        text.push_str(" descriptors, ");
        text.push_str(&self.ayanamsa_label_count.to_string());
        text.push_str(" labels\n");
        text.push_str("Baseline/release slices: ");
        text.push_str(&self.baseline_house_system_count.to_string());
        text.push_str(" house baseline + ");
        text.push_str(&self.release_house_system_count.to_string());
        text.push_str(" house release, ");
        text.push_str(&self.baseline_ayanamsa_count.to_string());
        text.push_str(" ayanamsa baseline + ");
        text.push_str(&self.release_ayanamsa_count.to_string());
        text.push_str(" ayanamsa release\n");
        text.push_str("Release-specific house-system canonical names verified: ");
        text.push_str(&self.release_house_canonical_names);
        text.push('\n');
        text.push_str("Release-specific ayanamsa canonical names verified: ");
        text.push_str(&self.release_ayanamsa_canonical_names);
        text.push('\n');
        text.push_str("House formula families verified: ");
        text.push_str(&self.house_formula_family_names);
        text.push('\n');
        text.push_str(&self.release_posture);
        text.push('\n');
        text.push_str("Ayanamsa reference metadata verified: ");
        text.push_str(&self.ayanamsa_metadata_count.to_string());
        text.push_str(" descriptors with epoch/offset metadata, ");
        text.push_str(&self.ayanamsa_metadata_gap_count.to_string());
        text.push_str(" metadata gaps\n");
        text.push_str("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented\n");
        text.push_str("Release notes documented: ");
        text.push_str(&self.release_note_count.to_string());
        text.push_str(" entries\n");
        text.push_str("Validation reference points documented: ");
        text.push_str(&self.validation_reference_point_count.to_string());
        text.push_str(" entries\n");
        text.push_str("Custom-definition labels verified: ");
        text.push_str(&self.custom_definition_label_count.to_string());
        text.push_str(" labels, all remain custom-definition territory\n");
        text.push_str("Custom-definition label names verified: ");
        text.push_str(&self.custom_definition_label_names);
        text.push('\n');
        text.push_str("Compatibility caveats documented: ");
        text.push_str(&self.compatibility_caveat_count.to_string());
        text.push('\n');
        text
    }
}

impl fmt::Display for CompatibilityProfileVerificationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured compatibility-profile verification posture.
pub fn compatibility_profile_verification_summary(
) -> Result<CompatibilityProfileVerificationSummary, EphemerisError> {
    let profile = current_compatibility_profile();
    let release_profiles = validated_release_profile_identifiers_for_report().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile verification summary release profile identifiers invalid: {error}"
            ),
        )
    })?;

    if profile.profile_id != release_profiles.compatibility_profile_id {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile identifier mismatch: profile {} does not match release profile {}",
                profile.profile_id, release_profiles.compatibility_profile_id
            ),
        ));
    }

    ensure_profile_slice_matches(
        "house-system catalog",
        profile.house_systems,
        built_in_house_systems(),
    )?;
    ensure_profile_slice_matches(
        "baseline house-system slice",
        profile.baseline_house_systems,
        baseline_house_systems(),
    )?;
    ensure_profile_slice_matches(
        "release house-system slice",
        profile.release_house_systems,
        release_house_systems(),
    )?;
    ensure_profile_slice_matches("ayanamsa catalog", profile.ayanamsas, built_in_ayanamsas())?;
    ensure_profile_slice_matches(
        "baseline ayanamsa slice",
        profile.baseline_ayanamsas,
        baseline_ayanamsas(),
    )?;
    ensure_profile_slice_matches(
        "release ayanamsa slice",
        profile.release_ayanamsas,
        release_ayanamsas(),
    )?;

    verify_profile_catalog_partitions_are_disjoint(
        "house-system",
        profile.baseline_house_systems,
        profile.release_house_systems,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )?;
    verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        profile.baseline_ayanamsas,
        profile.release_ayanamsas,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )?;

    let house_labels_checked = verify_house_system_aliases(profile.house_systems)?;
    let ayanamsa_labels_checked = verify_ayanamsa_aliases(profile.ayanamsas)?;
    let custom_definition_labels_checked =
        verify_custom_definition_labels(profile.custom_definition_labels)?;
    verify_profile_text_section("release-note", profile.release_notes)?;
    verify_profile_text_section(
        "validation-reference-point",
        profile.validation_reference_points,
    )?;
    verify_profile_text_section("compatibility-caveat", profile.known_gaps)?;
    verify_profile_text_sections_are_disjoint(&[
        ("release-note", profile.release_notes),
        (
            "validation-reference-point",
            profile.validation_reference_points,
        ),
        ("compatibility-caveat", profile.known_gaps),
    ])?;

    let release_house_names =
        profile.validated_release_house_system_canonical_names_summary_line().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release house-system canonical names invalid: {error}"
                ),
            )
        })?;
    let release_ayanamsa_names =
        profile.validated_release_ayanamsa_canonical_names_summary_line().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release ayanamsa canonical names invalid: {error}"
                ),
            )
        })?;
    let house_formula_family_names = profile.house_formula_family_names().join(", ");
    let ayanamsa_metadata_count = profile
        .ayanamsas
        .iter()
        .filter(|entry| entry.has_sidereal_metadata())
        .count();

    Ok(CompatibilityProfileVerificationSummary {
        profile_id: release_profiles.compatibility_profile_id.to_string(),
        house_system_descriptor_count: profile.house_systems.len(),
        house_system_label_count: house_labels_checked,
        house_code_alias_count: profile.house_code_alias_count(),
        house_code_aliases_summary: profile.house_code_aliases_summary_line(),
        latitude_sensitive_house_systems: profile
            .latitude_sensitive_house_systems()
            .into_iter()
            .map(|label| label.to_string())
            .collect(),
        ayanamsa_descriptor_count: profile.ayanamsas.len(),
        ayanamsa_label_count: ayanamsa_labels_checked,
        baseline_house_system_count: profile.baseline_house_systems.len(),
        release_house_system_count: profile.release_house_systems.len(),
        baseline_ayanamsa_count: profile.baseline_ayanamsas.len(),
        release_ayanamsa_count: profile.release_ayanamsas.len(),
        release_house_canonical_names: release_house_names,
        release_ayanamsa_canonical_names: release_ayanamsa_names,
        house_formula_family_names,
        release_posture: RELEASE_POSTURE_SUMMARY.to_string(),
        ayanamsa_metadata_count,
        ayanamsa_metadata_gap_count: profile.ayanamsas.len() - ayanamsa_metadata_count,
        release_note_count: profile.release_notes.len(),
        validation_reference_point_count: profile.validation_reference_points.len(),
        custom_definition_label_count: custom_definition_labels_checked,
        custom_definition_label_names: profile.custom_definition_labels.join(", "),
        compatibility_caveat_count: profile.known_gaps.len(),
    })
}

/// Verifies that the release compatibility profile stays synchronized with the
/// canonical house-system and ayanamsa catalogs.
pub fn verify_compatibility_profile() -> Result<String, EphemerisError> {
    let summary = compatibility_profile_verification_summary()?;
    summary.validate()?;
    Ok(summary.summary_line())
}

fn ensure_profile_slice_matches<T>(
    label: &str,
    actual: &[T],
    expected: &[T],
) -> Result<(), EphemerisError>
where
    T: PartialEq + fmt::Debug,
{
    if actual != expected {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {label} mismatch: expected {} entries, found {}",
                expected.len(),
                actual.len()
            ),
        ));
    }

    Ok(())
}

fn verify_profile_catalog_partitions_are_disjoint<T>(
    catalog_label: &str,
    baseline_entries: &[T],
    release_entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
    aliases: impl Fn(&T) -> &'static [&'static str],
) -> Result<(), EphemerisError> {
    let mut baseline_labels = BTreeSet::new();

    for entry in baseline_entries {
        baseline_labels.insert(canonical_name(entry).trim().to_ascii_lowercase());
        for alias in aliases(entry) {
            baseline_labels.insert(alias.trim().to_ascii_lowercase());
        }
    }

    for entry in release_entries {
        for label in std::iter::once(canonical_name(entry)).chain(aliases(entry).iter().copied()) {
            let normalized_label = label.trim().to_ascii_lowercase();
            if baseline_labels.contains(&normalized_label) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile {catalog_label} baseline and release slices overlap on label '{label}'",
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn verify_house_system_aliases(
    entries: &[pleiades_houses::HouseSystemDescriptor],
) -> Result<usize, EphemerisError> {
    if let Err(error) = validate_house_catalog() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("house catalog validation failed: {error}"),
        ));
    }
    if let Err(error) = pleiades_houses::validate_house_system_code_aliases() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("house-code alias validation failed: {error}"),
        ));
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();
    let mut seen_labels_case_insensitive = BTreeMap::new();

    for entry in entries {
        ensure_profile_descriptor_metadata("house-system", entry.canonical_name, entry.notes)?;

        labels_checked += 1;
        ensure_unique_profile_label(
            "house-system",
            entry.canonical_name,
            entry.canonical_name,
            &mut seen_labels,
            &mut seen_labels_case_insensitive,
        )?;
        if resolve_house_system(entry.canonical_name) != Some(entry.system.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile house-system alias mismatch: canonical label '{}' should resolve to {}",
                    entry.canonical_name, entry.system
                ),
            ));
        }

        for alias in entry.aliases {
            labels_checked += 1;
            if has_surrounding_whitespace(alias) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile house-system descriptor '{}' contains surrounding whitespace in its label",
                        alias
                    ),
                ));
            }
            ensure_unique_profile_label(
                "house-system",
                alias,
                entry.canonical_name,
                &mut seen_labels,
                &mut seen_labels_case_insensitive,
            )?;
            if resolve_house_system(alias) != Some(entry.system.clone()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile house-system alias mismatch: alias '{}' should resolve to {}",
                        alias, entry.system
                    ),
                ));
            }
        }
    }

    Ok(labels_checked)
}

fn format_ayanamsa_label(ayanamsa: &pleiades_core::Ayanamsa) -> String {
    pleiades_ayanamsa::descriptor(ayanamsa)
        .map(|descriptor| descriptor.canonical_name.to_owned())
        .unwrap_or_else(|| ayanamsa.to_string())
}

fn verify_ayanamsa_aliases(
    entries: &[pleiades_ayanamsa::AyanamsaDescriptor],
) -> Result<usize, EphemerisError> {
    if let Err(error) = validate_ayanamsa_catalog() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("ayanamsa catalog validation failed: {error}"),
        ));
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();
    let mut seen_labels_case_insensitive = BTreeMap::new();

    for entry in entries {
        ensure_profile_descriptor_metadata("ayanamsa", entry.canonical_name, entry.notes)?;

        labels_checked += 1;
        ensure_unique_profile_label(
            "ayanamsa",
            entry.canonical_name,
            entry.canonical_name,
            &mut seen_labels,
            &mut seen_labels_case_insensitive,
        )?;
        if resolve_ayanamsa(entry.canonical_name) != Some(entry.ayanamsa.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile ayanamsa alias mismatch: canonical label '{}' should resolve to {}",
                    entry.canonical_name,
                    format_ayanamsa_label(&entry.ayanamsa)
                ),
            ));
        }

        for alias in entry.aliases {
            labels_checked += 1;
            if has_surrounding_whitespace(alias) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile ayanamsa descriptor '{}' contains surrounding whitespace in its label",
                        alias
                    ),
                ));
            }
            ensure_unique_profile_label(
                "ayanamsa",
                alias,
                entry.canonical_name,
                &mut seen_labels,
                &mut seen_labels_case_insensitive,
            )?;
            if resolve_ayanamsa(alias) != Some(entry.ayanamsa.clone()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile ayanamsa alias mismatch: alias '{}' should resolve to {}",
                        alias,
                        format_ayanamsa_label(&entry.ayanamsa)
                    ),
                ));
            }
        }
    }

    Ok(labels_checked)
}

#[cfg(test)]
const INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS: &[&str] = &[
    "Babylonian (House)",
    "Babylonian (Sissy)",
    "Babylonian (True Geoc)",
    "Babylonian (True Topc)",
    "Babylonian (True Obs)",
    "Babylonian (House Obs)",
];

#[cfg(test)]
fn is_intentional_custom_definition_ayanamsa_homograph(label: &str) -> bool {
    INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS.contains(&label)
}

fn verify_custom_definition_labels(labels: &[&'static str]) -> Result<usize, EphemerisError> {
    validate_custom_definition_labels(labels)
        .map_err(|error| EphemerisError::new(EphemerisErrorKind::InvalidRequest, error.to_string()))
}

fn has_surrounding_whitespace(value: &str) -> bool {
    !value.is_empty() && value.trim() != value
}

fn verify_profile_text_section(
    section_label: &str,
    entries: &[&str],
) -> Result<usize, EphemerisError> {
    let mut entries_checked = 0usize;
    let mut seen_entries = BTreeSet::new();
    let mut seen_entries_case_insensitive = BTreeMap::new();

    for entry in entries {
        entries_checked += 1;
        if entry.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("compatibility profile {section_label} entry is blank"),
            ));
        }

        if has_surrounding_whitespace(entry) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile {section_label} entry '{}' contains surrounding whitespace",
                    entry
                ),
            ));
        }

        let normalized_entry = entry.trim().to_string();
        if !seen_entries.insert(normalized_entry.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("compatibility profile {section_label} entries are not unique: duplicate entry '{}'", entry),
            ));
        }

        let normalized_case_insensitive = normalized_entry.to_ascii_lowercase();
        if let Some(existing_entry) =
            seen_entries_case_insensitive.get(&normalized_case_insensitive)
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile {section_label} entries are not unique ignoring case: duplicate entry '{}' conflicts with '{}'",
                    entry, existing_entry
                ),
            ));
        }
        seen_entries_case_insensitive.insert(normalized_case_insensitive, normalized_entry);
    }

    Ok(entries_checked)
}

fn verify_profile_text_sections_are_disjoint(
    sections: &[(&'static str, &'static [&'static str])],
) -> Result<(), EphemerisError> {
    let mut seen_entries = BTreeMap::<String, &'static str>::new();
    let mut seen_entries_case_insensitive = BTreeMap::<String, (&'static str, String)>::new();

    for (section_label, entries) in sections {
        for entry in *entries {
            if entry.trim().is_empty() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!("compatibility profile {section_label} entry is blank"),
                ));
            }

            if has_surrounding_whitespace(entry) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile {section_label} entry '{}' contains surrounding whitespace",
                        entry
                    ),
                ));
            }

            let normalized_entry = entry.trim().to_string();
            if let Some(existing_section) = seen_entries.get(&normalized_entry) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile text sections are not unique: duplicate entry '{}' appears in both {} and {}",
                        entry, existing_section, section_label
                    ),
                ));
            }

            let normalized_case_insensitive = normalized_entry.to_ascii_lowercase();
            if let Some((existing_section, existing_entry)) =
                seen_entries_case_insensitive.get(&normalized_case_insensitive)
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile text sections are not unique ignoring case: duplicate entry '{}' appears in both {} and {} (conflicts with '{}')",
                        entry, existing_section, section_label, existing_entry
                    ),
                ));
            }

            seen_entries.insert(normalized_entry.clone(), section_label);
            seen_entries_case_insensitive.insert(
                normalized_case_insensitive,
                (section_label, normalized_entry),
            );
        }
    }

    Ok(())
}

fn ensure_profile_descriptor_metadata(
    catalog_label: &str,
    canonical_name: &str,
    notes: &str,
) -> Result<(), EphemerisError> {
    if canonical_name.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("compatibility profile {catalog_label} descriptor is missing a canonical name"),
        ));
    }

    if has_surrounding_whitespace(canonical_name) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{canonical_name}' contains surrounding whitespace in its canonical name",
            ),
        ));
    }

    if notes.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{}' is missing notes metadata",
                canonical_name
            ),
        ));
    }

    if has_surrounding_whitespace(notes) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{}' contains surrounding whitespace in its notes metadata",
                canonical_name
            ),
        ));
    }

    Ok(())
}

fn ensure_unique_profile_label(
    catalog_label: &str,
    label: &str,
    item_identity: &str,
    seen_labels: &mut BTreeSet<String>,
    seen_labels_case_insensitive: &mut BTreeMap<String, String>,
) -> Result<(), EphemerisError> {
    if label.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("compatibility profile {catalog_label} descriptor contains a blank label"),
        ));
    }

    if has_surrounding_whitespace(label) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{}' contains surrounding whitespace in its label",
                item_identity
            ),
        ));
    }

    let normalized = label.trim().to_string();
    if !seen_labels.insert(normalized.clone()) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} labels are not unique: duplicate label '{}'",
                label
            ),
        ));
    }

    let normalized_case_insensitive = normalized.to_ascii_lowercase();
    match seen_labels_case_insensitive.get(&normalized_case_insensitive) {
        Some(existing_identity) if existing_identity == item_identity => {}
        Some(_) => {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile {catalog_label} labels are not unique ignoring case: duplicate label '{}'",
                    label
                ),
            ));
        }
        None => {
            seen_labels_case_insensitive
                .insert(normalized_case_insensitive, item_identity.to_string());
        }
    }

    Ok(())
}

fn validation_reference_point_summary(point: &str) -> String {
    if point.contains("stage-4 validation corpus") {
        "stage-4 validation corpus".to_string()
    } else {
        point.to_string()
    }
}

fn summarize_validation_reference_points(points: &[&str]) -> String {
    match points {
        [] => "0".to_string(),
        [point] => format!("1 ({})", validation_reference_point_summary(point)),
        _ => format!(
            "{} ({})",
            points.len(),
            points
                .iter()
                .map(|point| validation_reference_point_summary(point))
                .collect::<Vec<_>>()
                .join("; ")
        ),
    }
}

#[derive(Clone, Debug, PartialEq)]
struct AyanamsaReferenceOffsetExample {
    canonical_name: &'static str,
    epoch: JulianDay,
    offset_degrees: Angle,
}

#[derive(Clone, Debug, PartialEq)]
struct AyanamsaReferenceOffsetsSummary {
    examples: Vec<AyanamsaReferenceOffsetExample>,
}

impl AyanamsaReferenceOffsetsSummary {
    fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "ayanamsa reference offsets",
            self.examples.iter().map(|example| example.canonical_name),
        )?;

        Ok(())
    }

    fn summary_line(&self) -> String {
        match self.examples.as_slice() {
            [] => "representative zero-point examples: 0 (none)".to_string(),
            [single] => format!(
                "representative zero-point examples: 1 ({}: epoch={}; offset={})",
                single.canonical_name, single.epoch, single.offset_degrees
            ),
            _ => format!(
                "representative zero-point examples: {}",
                self.examples
                    .iter()
                    .map(|example| format!(
                        "{}: epoch={}; offset={}",
                        example.canonical_name, example.epoch, example.offset_degrees
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

impl fmt::Display for AyanamsaReferenceOffsetsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn summarize_ayanamsa_reference_offsets() -> Result<AyanamsaReferenceOffsetsSummary, EphemerisError>
{
    let samples = pleiades_ayanamsa::reference_offset_sample_ayanamsas();

    let mut examples = Vec::with_capacity(samples.len());
    for sample in samples {
        let descriptor = descriptor(sample).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("ayanamsa reference offsets sample `{sample}` is unavailable"),
            )
        })?;
        let epoch = descriptor.epoch.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference epoch",
                    descriptor.canonical_name
                ),
            )
        })?;
        let offset_degrees = descriptor.offset_degrees.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference offset",
                    descriptor.canonical_name
                ),
            )
        })?;

        examples.push(AyanamsaReferenceOffsetExample {
            canonical_name: descriptor.canonical_name,
            epoch,
            offset_degrees,
        });
    }

    let summary = AyanamsaReferenceOffsetsSummary { examples };
    summary.validate()?;
    Ok(summary)
}

fn format_ayanamsa_reference_offsets_for_report() -> String {
    match summarize_ayanamsa_reference_offsets() {
        Ok(summary) => format!("Ayanamsa reference offsets: {summary}"),
        Err(error) => format!("Ayanamsa reference offsets: unavailable ({error})"),
    }
}

fn format_ayanamsa_metadata_coverage_for_report() -> String {
    match metadata_coverage().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("ayanamsa sidereal metadata: unavailable ({error})"),
    }
}

fn format_house_code_aliases_for_report() -> String {
    match validate_house_system_code_aliases() {
        Ok(()) => house_system_code_aliases_summary_line(),
        Err(error) => format!("house-code aliases unavailable ({error})"),
    }
}

fn format_house_formula_families_for_report() -> String {
    match validated_compatibility_profile_for_report() {
        Ok(profile) => {
            format_house_formula_families_summary(&summarize_house_formula_families(&profile))
        }
        Err(error) => format!("house formula families unavailable ({error})"),
    }
}

fn summarize_latitude_sensitive_house_systems(profile: &CompatibilityProfile) -> String {
    profile.latitude_sensitive_house_systems_summary_line()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HouseFormulaFamiliesSummary {
    names: Vec<String>,
}

impl HouseFormulaFamiliesSummary {
    fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "house formula families summary",
            self.names.iter().map(String::as_str),
        )
    }

    fn summary_line(&self) -> String {
        match self.names.as_slice() {
            [] => "0 (none)".to_string(),
            [single] => format!("1 ({single})"),
            _ => format!("{} ({})", self.names.len(), self.names.join(", ")),
        }
    }
}

impl fmt::Display for HouseFormulaFamiliesSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn summarize_house_formula_families(profile: &CompatibilityProfile) -> HouseFormulaFamiliesSummary {
    HouseFormulaFamiliesSummary {
        names: profile.house_formula_family_names(),
    }
}

fn format_house_formula_families_summary(summary: &HouseFormulaFamiliesSummary) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("unavailable ({error})"),
    }
}

fn validate_name_sequence<'a, I>(
    section_label: &'static str,
    names: I,
) -> Result<(), EphemerisError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen_names = BTreeSet::new();
    let mut seen_names_case_insensitive = BTreeMap::new();

    for name in names {
        if name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a blank name"),
            ));
        }

        if has_surrounding_whitespace(name) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} entry '{name}' contains surrounding whitespace"),
            ));
        }

        let normalized_name = name.trim().to_string();
        if !seen_names.insert(normalized_name.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a duplicate name '{name}'"),
            ));
        }

        let normalized_name_case_insensitive = normalized_name.to_ascii_lowercase();
        if let Some(existing_name) =
            seen_names_case_insensitive.get(&normalized_name_case_insensitive)
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "{section_label} contains a case-insensitive duplicate name '{name}' that conflicts with '{existing_name}'"
                ),
            ));
        }
        seen_names_case_insensitive.insert(normalized_name_case_insensitive, normalized_name);
    }

    Ok(())
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DescriptorNamesSummary {
    names: Vec<&'static str>,
}

#[cfg(test)]
impl DescriptorNamesSummary {
    fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence("descriptor-name summary", self.names.iter().copied())
    }

    fn summary_line(&self) -> String {
        match self.names.as_slice() {
            [] => "0 (none)".to_string(),
            [single] => format!("1 ({single})"),
            _ => format!("{} ({})", self.names.len(), self.names.join(", ")),
        }
    }
}

#[cfg(test)]
impl fmt::Display for DescriptorNamesSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
fn summarize_descriptor_names<T>(
    entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
) -> DescriptorNamesSummary {
    DescriptorNamesSummary {
        names: entries.iter().map(canonical_name).collect::<Vec<_>>(),
    }
}

fn render_compatibility_profile_summary_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    };
    let coverage = metadata_coverage();
    let mut text = String::new();

    text.push_str("Compatibility profile summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("House systems: ");
    text.push_str(&profile.house_systems.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_house_systems.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_house_systems.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str("Latitude-sensitive house systems: ");
    text.push_str(&summarize_latitude_sensitive_house_systems(&profile));
    text.push('\n');
    text.push_str("House formula families: ");
    text.push_str(&format_house_formula_families_summary(
        &summarize_house_formula_families(&profile),
    ));
    text.push('\n');
    text.push_str("House code aliases: ");
    match profile.validated_house_code_aliases_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Ayanamsas: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str(&profile.target_house_scope.join("; "));
    text.push('\n');
    text.push_str(&profile.target_ayanamsa_scope.join("; "));
    text.push('\n');
    match coverage.validated_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&ayanamsa_catalog_validation_summary().summary_line());
    text.push('\n');
    text.push_str(&format_ayanamsa_metadata_coverage_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_reference_offsets_for_report());
    text.push('\n');
    text.push_str("Release-specific house-system canonical names: ");
    match profile.validated_release_house_system_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Release-specific ayanamsa canonical names: ");
    match profile.validated_release_ayanamsa_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Custom-definition label names: ");
    if profile.custom_definition_labels.is_empty() {
        text.push_str("none");
    } else {
        text.push_str(&profile.custom_definition_labels.join(", "));
    }
    text.push('\n');
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

fn render_release_notes_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Release notes unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Release notes unavailable ({error})"),
    };
    let mut text = String::new();

    text.push_str("Release notes\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Summary:\n");
    text.push_str(profile.summary);
    text.push('\n');
    text.push('\n');
    match profile.validated_catalog_inventory_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("House code aliases: ");
    match profile.validated_house_code_aliases_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&format_release_profile_identifiers_summary(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push('\n');

    match validated_api_stability_profile_for_report() {
        Ok(api_stability) => {
            text.push_str("API stability posture:\n");
            text.push_str("- ");
            text.push_str(api_stability.summary);
            text.push('\n');
            text.push_str("Deprecation policy:\n");
            for item in api_stability.deprecation_policy {
                text.push_str("- ");
                text.push_str(item);
                text.push('\n');
            }
            text.push('\n');
        }
        Err(error) => {
            text.push_str("API stability posture unavailable (");
            text.push_str(&error);
            text.push_str(")\n\n");
        }
    }

    if !profile.release_notes.is_empty() {
        text.push_str("Release-specific coverage:\n");
        for note in profile.release_notes {
            text.push_str("- ");
            text.push_str(note);
            text.push('\n');
        }
        text.push('\n');
    }
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_equatorial_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_early_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1800_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_manifest_summary_for_report());
    text.push('\n');

    if !profile.custom_definition_labels.is_empty() {
        text.push_str("Custom-definition labels:\n");
        for label in profile.custom_definition_labels {
            text.push_str("- ");
            text.push_str(label);
            text.push('\n');
        }
        text.push('\n');
    }

    if !profile.validation_reference_points.is_empty() {
        text.push_str("Validation reference points:\n");
        for point in profile.validation_reference_points {
            text.push_str("- ");
            text.push_str(point);
            text.push('\n');
        }
        text.push('\n');
    }

    if !profile.known_gaps.is_empty() {
        text.push_str("Compatibility caveats:\n");
        for gap in profile.known_gaps {
            text.push_str("- ");
            text.push_str(gap);
            text.push('\n');
        }
        text.push('\n');
    }

    text.push_str("Bundle provenance:\n");
    text.push_str("- source revision, workspace status, and Rust compiler version are recorded in the manifest\n");

    text
}

fn render_release_notes_summary_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Release notes summary unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Release notes summary unavailable ({error})"),
    };
    let mut text = String::new();

    text.push_str("Release notes summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(release_profiles.api_stability_profile_id);
    text.push('\n');
    text.push_str("Release-specific coverage: ");
    text.push_str(&profile.release_notes.len().to_string());
    text.push('\n');
    text.push_str("Latitude-sensitive house systems: ");
    text.push_str(&summarize_latitude_sensitive_house_systems(&profile));
    text.push('\n');
    text.push_str("House formula families: ");
    text.push_str(&format_house_formula_families_summary(
        &summarize_house_formula_families(&profile),
    ));
    text.push('\n');
    text.push_str("House code aliases: ");
    text.push_str(&profile.house_code_aliases_summary_line());
    text.push('\n');
    text.push_str(&request_surface_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_equatorial_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_early_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1800_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_source_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&source_documentation_health_summary_for_report());
    text.push('\n');
    text.push_str("JPL request policy: ");
    text.push_str(&jpl_snapshot_request_policy_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str("Comparison tolerance policy: ");
    text.push_str(&comparison_tolerance_policy_summary_for_release_notes());
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Custom-definition label names: ");
    if profile.custom_definition_labels.is_empty() {
        text.push_str("none");
    } else {
        text.push_str(&profile.custom_definition_labels.join(", "));
    }
    text.push('\n');
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    text.push_str(&profile.target_house_scope.join("; "));
    text.push('\n');
    text.push_str(&profile.target_ayanamsa_scope.join("; "));
    text.push('\n');
    text.push_str("API stability summary line: ");
    text.push_str(&api_stability_summary_line_for_report());
    text.push('\n');
    text.push_str("Release notes: release-notes\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Packaged-artifact storage/reconstruction: ");
    text.push_str(&packaged_artifact_storage_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact access: ");
    text.push_str(&format_packaged_artifact_access_summary());
    text.push('\n');
    text.push_str("Packaged-artifact generation policy: ");
    text.push_str(&packaged_artifact_generation_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation residual bodies: ");
    text.push_str(&packaged_artifact_generation_residual_bodies_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact target thresholds: ");
    text.push_str(&packaged_artifact_target_threshold_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact target-threshold scope envelopes: ");
    text.push_str(&packaged_artifact_target_threshold_scope_envelopes_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation manifest: ");
    text.push_str(&packaged_artifact_generation_manifest_for_report());
    text.push('\n');
    text.push_str("Packaged request policy: ");
    text.push_str(&packaged_request_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged lookup epoch policy: ");
    text.push_str(&packaged_lookup_epoch_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged batch parity: ");
    text.push_str(&packaged_mixed_tt_tdb_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Artifact boundary envelope: ");
    text.push_str(
        &artifact_boundary_envelope_summary_for_report()
            .map(|summary| summary.summary_line())
            .unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Artifact inspection: ");
    text.push_str(
        &artifact_inspection_summary_for_report().unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&format_release_profile_identifiers_summary(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("See release-notes for the full maintainer-facing artifact.\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

fn comparison_tolerance_policy_summary_for_release_notes() -> String {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    match compare_backends(&reference, &candidate, &corpus) {
        Ok(report) => format_comparison_tolerance_policy_for_report(&report),
        Err(error) => format!("comparison tolerance policy unavailable ({error})"),
    }
}

fn release_checklist_summary_for_report() -> Result<ReleaseChecklistSummary, String> {
    let release_profile_identifiers =
        validated_release_profile_identifiers_for_report().map_err(|error| error.to_string())?;
    let summary = ReleaseChecklistSummary {
        release_profile_identifiers,
        repository_managed_release_gates: release_checklist_repository_managed_release_gates()
            .len(),
        manual_bundle_workflow_items: release_checklist_manual_bundle_workflow().len(),
        bundle_contents_items: release_checklist_bundle_contents().len(),
        external_publishing_reminders: release_checklist_external_publishing_reminders().len(),
    };
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

fn render_release_checklist_text() -> String {
    let summary = match release_checklist_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => return format!("Release checklist unavailable ({error})"),
    };
    let mut text = String::new();

    text.push_str("Release checklist\n");
    text.push_str("Profile: ");
    text.push_str(summary.release_profile_identifiers.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(summary.release_profile_identifiers.api_stability_profile_id);
    text.push('\n');
    text.push_str("Summary: ");
    text.push_str(&summary.summary_line());
    text.push('\n');
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary\n");
    text.push('\n');
    text.push_str("Repository-managed release gates:\n");
    for &item in release_checklist_repository_managed_release_gates() {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str("Manual bundle workflow:\n");
    for &item in release_checklist_manual_bundle_workflow() {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str("Bundle contents:\n");
    for &item in release_checklist_bundle_contents() {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str("External publishing reminders:\n");
    for &item in release_checklist_external_publishing_reminders() {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }

    text
}

fn render_release_summary_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Release summary unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Release summary unavailable ({error})"),
    };
    let request_policy = request_policy_summary_for_report();
    let mut text = String::new();

    text.push_str("Release summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(release_profiles.api_stability_profile_id);
    text.push('\n');
    text.push_str("Release profile identifiers: ");
    text.push_str(&format_release_profile_identifiers_summary(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str(&profile.target_house_scope.join("; "));
    text.push('\n');
    text.push_str(&profile.target_ayanamsa_scope.join("; "));
    text.push('\n');
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text.push_str("Frame policy: ");
    text.push_str(request_policy.frame);
    text.push('\n');
    text.push_str("Mean-obliquity frame round-trip: ");
    text.push_str(&mean_obliquity_frame_round_trip_summary_for_report());
    text.push('\n');
    text.push_str(&request_surface_summary_for_report());
    text.push('\n');
    text.push_str("JPL interpolation posture: ");
    text.push_str(&jpl_interpolation_posture_summary_for_report());
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]));
    text.push('\n');
    text.push_str("Release summary line: ");
    text.push_str(profile.summary);
    text.push('\n');
    match profile.validated_catalog_inventory_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("House code aliases: ");
    match profile.validated_house_code_aliases_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Latitude-sensitive house systems: ");
    text.push_str(&summarize_latitude_sensitive_house_systems(&profile));
    text.push('\n');
    text.push_str("House formula families: ");
    text.push_str(&format_house_formula_families_summary(
        &summarize_house_formula_families(&profile),
    ));
    text.push('\n');
    text.push_str(&lunar_theory_catalog_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_catalog_validation_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_source_summary_for_report());
    text.push('\n');
    text.push_str("House systems: ");
    text.push_str(&profile.house_systems.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_house_systems.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_house_systems.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str("House-code aliases: ");
    text.push_str(&profile.house_code_alias_count().to_string());
    text.push('\n');
    text.push_str("Release-specific house-system canonical names: ");
    match profile.validated_release_house_system_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Ayanamsas: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str("Release-specific ayanamsa canonical names: ");
    match profile.validated_release_ayanamsa_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Release notes unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&format_ayanamsa_reference_offsets_for_report());
    text.push('\n');
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Custom-definition label names: ");
    text.push_str(&profile.custom_definition_labels.join(", "));
    text.push('\n');
    text.push_str("Custom-definition ayanamsas: ");
    text.push_str(
        &profile
            .ayanamsas
            .iter()
            .filter(|entry| {
                profile
                    .custom_definition_labels
                    .contains(&entry.canonical_name)
            })
            .count()
            .to_string(),
    );
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    if let Ok(report) = build_validation_report(SUMMARY_BENCHMARK_ROUNDS) {
        let tolerance_summaries = report.comparison.tolerance_summaries();
        let body_class_tolerance_summaries = report.comparison.body_class_tolerance_summaries();
        let tolerance_outside_bodies: usize = body_class_tolerance_summaries
            .iter()
            .map(|summary| summary.outside_tolerance_body_count)
            .sum();
        let outside_tolerance_body_count = tolerance_summaries
            .iter()
            .filter(|summary| !summary.within_tolerance)
            .count();
        let outside_tolerance_body_names = body_class_tolerance_summaries
            .iter()
            .flat_map(|summary| summary.outside_bodies.iter().cloned())
            .collect::<Vec<_>>();
        let outside_class_count = body_class_tolerance_summaries
            .iter()
            .filter(|summary| summary.outside_tolerance_body_count > 0)
            .count();
        text.push_str("Comparison envelope: ");
        text.push_str(&format_comparison_envelope_for_report(
            &report.comparison.summary,
            &report.comparison.samples,
        ));
        text.push('\n');
        text.push_str("Comparison tail envelope: ");
        text.push_str(&format_comparison_percentile_envelope_for_report(
            &report.comparison.samples,
        ));
        text.push('\n');
        text.push_str("Body-class error envelopes:\n");
        for summary in report.comparison.body_class_summaries() {
            text.push_str("  ");
            text.push_str(summary.class.label());
            text.push_str(": ");
            text.push_str(&format_body_class_comparison_envelope_for_report(&summary));
            text.push('\n');
        }
        text.push('\n');
        text.push_str("Body-class tolerance posture: ");
        text.push_str(&body_class_tolerance_summaries.len().to_string());
        text.push_str(" classes checked, ");
        text.push_str(&outside_class_count.to_string());
        text.push_str(" classes with outlier bodies, outlier bodies: ");
        text.push_str(&format_bodies(&outside_tolerance_body_names));
        text.push('\n');
        text.push_str("Expected tolerance status: ");
        text.push_str(&tolerance_summaries.len().to_string());
        text.push_str(" bodies checked, ");
        text.push_str(
            &tolerance_summaries
                .len()
                .saturating_sub(outside_tolerance_body_count)
                .to_string(),
        );
        text.push_str(" within tolerance, ");
        text.push_str(&outside_tolerance_body_count.to_string());
        text.push_str(" outside tolerance");
        text.push('\n');
        text.push_str("Validation evidence: ");
        text.push_str(&report.comparison.summary.sample_count.to_string());
        text.push_str(" comparison samples, ");
        text.push_str(&report.comparison.notable_regressions().len().to_string());
        text.push_str(" notable regressions, ");
        text.push_str(&tolerance_outside_bodies.to_string());
        text.push_str(" outside-tolerance bodies");
        text.push('\n');
        text.push_str("Comparison audit: ");
        text.push_str(&comparison_audit_summary_for_report(&report.comparison));
        text.push('\n');
        text.push_str("House validation corpus: ");
        text.push_str(&house_validation_summary_line_for_report(
            &report.house_validation,
        ));
        text.push('\n');
        text.push_str(&ayanamsa_catalog_validation_summary().summary_line());
        text.push('\n');
        text.push_str("Comparison tolerance policy: ");
        text.push_str(&format_comparison_tolerance_policy_for_report(
            &report.comparison,
        ));
        text.push('\n');
        text.push_str(&comparison_snapshot_summary_for_report());
        text.push('\n');
        text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
        text.push('\n');
        text.push_str(&reference_snapshot_batch_parity_summary_for_report());
        text.push('\n');
    }
    text.push_str("JPL interpolation evidence: ");
    text.push_str(&format_jpl_interpolation_quality_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_independent_holdout_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_holdout_overlap_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_independent_holdout_snapshot_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str("JPL request policy: ");
    text.push_str(&jpl_snapshot_request_policy_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str("JPL frame treatment: ");
    text.push_str(&format_jpl_frame_treatment_summary());
    text.push('\n');
    text.push_str(&reference_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_early_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1800_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_mars_jupiter_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_mars_outer_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_boundary_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation source windows: ");
    text.push_str(&production_generation_snapshot_window_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation body-class coverage: ");
    text.push_str(&production_generation_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary overlay: ");
    text.push_str(&production_generation_boundary_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary body-class coverage: ");
    text.push_str(&production_generation_boundary_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary windows: ");
    text.push_str(&production_generation_boundary_window_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary request corpus: ");
    text.push_str(&production_generation_boundary_request_corpus_summary_for_report());
    text.push('\n');
    text.push_str(&production_generation_boundary_source_summary_for_report());
    text.push('\n');
    text.push_str("Comparison snapshot source windows: ");
    text.push_str(&comparison_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str("Source-backed backend evidence: ");
    text.push_str(&jpl_snapshot_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Selected asteroid evidence: ");
    text.push_str(&selected_asteroid_source_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Selected asteroid source windows: ");
    text.push_str(&selected_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&independent_holdout_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str("VSOP87 evidence: ");
    text.push_str(&format_vsop87_source_documentation_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_source_documentation_health_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_frame_treatment_summary());
    text.push_str(" | ");
    text.push_str("VSOP87 request policy: ");
    text.push_str(&format_vsop87_request_policy_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_source_audit_summary());
    text.push_str(" | ");
    text.push_str(&generated_binary_audit_summary_for_report());
    text.push_str(" | ");
    text.push_str(&format_vsop87_canonical_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_canonical_outlier_note_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_equatorial_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_j2000_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j2000_ecliptic_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j2000_equatorial_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j1900_ecliptic_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_supported_body_j1900_equatorial_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_mixed_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_j1900_batch_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_body_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_source_body_class_evidence_summary());
    text.push_str(" | ");
    text.push_str(&format_vsop87_equatorial_body_class_evidence_summary());
    text.push('\n');
    text.push_str("ELP lunar capability: ");
    text.push_str(&lunar_theory_capability_summary_for_report());
    text.push('\n');
    text.push_str("ELP lunar request policy: ");
    text.push_str(&lunar_theory_request_policy_summary());
    text.push('\n');
    text.push_str("ELP frame treatment: ");
    text.push_str(&format_lunar_frame_treatment_summary());
    text.push('\n');
    text.push_str("Lunar reference: ");
    text.push_str(&lunar_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar reference batch parity: ");
    text.push_str(&lunar_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Lunar reference envelope: ");
    text.push_str(&lunar_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference: ");
    text.push_str(&lunar_equatorial_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference batch parity: ");
    text.push_str(&lunar_equatorial_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference envelope: ");
    text.push_str(&lunar_equatorial_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar apparent comparison: ");
    text.push_str(&lunar_apparent_comparison_summary_for_report());
    text.push('\n');
    text.push_str("Lunar source windows: ");
    text.push_str(&lunar_source_window_summary_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature continuity evidence\n");
    text.push_str(&lunar_high_curvature_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature equatorial continuity evidence\n");
    text.push_str(&lunar_high_curvature_equatorial_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Workspace audit: workspace-audit / audit\n");
    text.push_str("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Packaged-artifact profile: ");
    text.push_str(&format_packaged_artifact_profile_summary());
    text.push('\n');
    text.push_str("Packaged-artifact production profile skeleton: ");
    text.push_str(&packaged_artifact_production_profile_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact target thresholds: ");
    text.push_str(&packaged_artifact_target_threshold_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact target-threshold scope envelopes: ");
    text.push_str(&packaged_artifact_target_threshold_scope_envelopes_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation manifest: ");
    text.push_str(&packaged_artifact_generation_manifest_for_report());
    text.push('\n');
    text.push_str("Artifact profile coverage: ");
    text.push_str(&packaged_artifact_profile_coverage_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact output support: ");
    text.push_str(&format_packaged_artifact_output_support_summary());
    text.push('\n');
    text.push_str("Packaged-artifact storage/reconstruction: ");
    text.push_str(&format_packaged_artifact_storage_summary());
    text.push('\n');
    text.push_str("Packaged-artifact access: ");
    text.push_str(&format_packaged_artifact_access_summary());
    text.push('\n');
    text.push_str("Packaged-artifact generation policy: ");
    text.push_str(&packaged_artifact_generation_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact generation residual bodies: ");
    text.push_str(&packaged_artifact_generation_residual_bodies_summary_for_report());
    text.push('\n');
    text.push_str("Packaged-artifact regeneration: ");
    text.push_str(&packaged_artifact_regeneration_summary_for_report());
    text.push('\n');
    text.push_str("Packaged request policy: ");
    text.push_str(&packaged_request_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged lookup epoch policy: ");
    text.push_str(&packaged_lookup_epoch_policy_summary_for_report());
    text.push('\n');
    text.push_str("Packaged batch parity: ");
    text.push_str(&packaged_mixed_tt_tdb_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str("Packaged frame parity: ");
    text.push_str(&format_packaged_frame_parity_summary());
    text.push('\n');
    text.push_str("Packaged frame treatment: ");
    text.push_str(&format_packaged_frame_treatment_summary());
    text.push('\n');
    text.push_str("Artifact boundary envelope: ");
    text.push_str(
        &artifact_boundary_envelope_summary_for_report()
            .map(|summary| summary.summary_line())
            .unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Artifact inspection: ");
    text.push_str(
        &artifact_inspection_summary_for_report().unwrap_or_else(|_| "unavailable".to_string()),
    );
    text.push('\n');
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push('\n');
    text.push_str("API stability summary line: ");
    text.push_str(&api_stability_summary_line_for_report());
    text.push('\n');
    text.push_str("Release gate reminders:\n");
    for item in [
        "[x] mise run fmt",
        "[x] mise run lint",
        "[x] mise run test",
        "[x] mise run audit",
        "[x] mise run release-smoke",
        "[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile",
        "[x] cargo run -q -p pleiades-validate -- validate-artifact",
        "[x] cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release",
        "[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release",
    ] {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str(
        "See release-notes and release-checklist for the full maintainer-facing artifacts; use release-checklist-summary for a compact checklist audit.\n",
    );

    text
}

fn render_release_checklist_summary_text() -> String {
    let summary = match release_checklist_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => return format!("Release checklist summary unavailable ({error})"),
    };
    let mut text = String::new();

    text.push_str("Release checklist summary\n");
    text.push_str("Profile: ");
    text.push_str(summary.release_profile_identifiers.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(summary.release_profile_identifiers.api_stability_profile_id);
    text.push('\n');
    text.push_str("Summary: ");
    text.push_str(&summary.summary_line());
    text.push('\n');
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Zodiac policy: ");
    text.push_str(&zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]));
    text.push('\n');
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Workspace audit summary: workspace-audit-summary\n");
    text.push_str("Workspace audit: workspace-audit / audit\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary\n");
    text.push_str("Repository-managed release gates: ");
    text.push_str(&summary.repository_managed_release_gates.to_string());
    text.push_str(" items\n");
    text.push_str("Manual bundle workflow: ");
    text.push_str(&summary.manual_bundle_workflow_items.to_string());
    text.push_str(" items\n");
    text.push_str("Bundle contents: ");
    text.push_str(&summary.bundle_contents_items.to_string());
    text.push_str(" items\n");
    text.push_str("External publishing reminders: ");
    text.push_str(&summary.external_publishing_reminders.to_string());
    text.push_str(" items\n");
    text.push('\n');
    text.push_str("See release-checklist for the full maintainer-facing artifact.\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceProvenance {
    /// Short git revision for the current workspace state.
    pub source_revision: String,
    /// Whether the workspace is clean, dirty, or unavailable.
    pub workspace_status: String,
    /// The current rustc version string.
    pub rustc_version: String,
}

/// Validation error for a workspace provenance record that drifted away from the compact report shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceProvenanceValidationError {
    /// A provenance field was blank, padded, or multi-line.
    FieldInvalid { field: &'static str },
}

impl fmt::Display for WorkspaceProvenanceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldInvalid { field } => write!(
                f,
                "workspace provenance field `{field}` must be a single non-empty line"
            ),
        }
    }
}

impl std::error::Error for WorkspaceProvenanceValidationError {}

impl WorkspaceProvenance {
    /// Returns the compact release-facing benchmark provenance block.
    pub fn summary_line(&self) -> String {
        format!(
            "Benchmark provenance\n  source revision: {}\n  workspace status: {}\n  rustc version: {}",
            self.source_revision, self.workspace_status, self.rustc_version
        )
    }

    /// Returns `Ok(())` when the provenance fields are safe for release-facing rendering.
    pub fn validate(&self) -> Result<(), WorkspaceProvenanceValidationError> {
        validate_workspace_provenance_field(&self.source_revision, "source revision")?;
        validate_workspace_provenance_field(&self.workspace_status, "workspace status")?;
        validate_workspace_provenance_field(&self.rustc_version, "rustc version")?;
        Ok(())
    }
}

impl fmt::Display for WorkspaceProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn validate_workspace_provenance_field(
    value: &str,
    field: &'static str,
) -> Result<(), WorkspaceProvenanceValidationError> {
    if value.trim().is_empty() || value.contains('\n') || value.contains('\r') {
        return Err(WorkspaceProvenanceValidationError::FieldInvalid { field });
    }

    Ok(())
}

pub fn workspace_provenance() -> WorkspaceProvenance {
    let source_revision = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let workspace_status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .map(|value| {
            if value.is_empty() {
                "clean".to_string()
            } else {
                "dirty".to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    WorkspaceProvenance {
        source_revision,
        workspace_status,
        rustc_version,
    }
}

pub fn benchmark_provenance_text() -> String {
    let provenance = workspace_provenance();
    match provenance.validate() {
        Ok(()) => provenance.summary_line(),
        Err(error) => format!("Benchmark provenance unavailable ({error})"),
    }
}

/// Writes a release bundle containing the compatibility profile, release-profile
/// identifiers, release notes, release notes summary, release summary, release checklist,
/// release checklist summary, backend matrix, API posture, API stability summary,
/// comparison-envelope summary, comparison-corpus release-guard summary, validation report summary, artifact summary,
/// packaged-artifact generation manifest, benchmark report, validation report, and a manifest.
pub fn render_release_bundle(
    rounds: usize,
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let profile_text = current_compatibility_profile().to_string();
    let profile_summary_text = render_compatibility_profile_summary_text();
    let release_notes_text = render_release_notes_text();
    let release_summary_text = render_release_summary_text();
    let release_profile_identifiers = current_release_profile_identifiers();
    release_profile_identifiers
        .validate()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let release_profile_identifiers_text = format!(
        "Release profile identifiers: {}\n",
        release_profile_identifiers.summary_line()
    );
    let release_checklist_text = render_release_checklist_text();
    let release_checklist_summary_text = render_release_checklist_summary_text();
    let backend_matrix_text = render_backend_matrix_report()?;
    let backend_matrix_summary_text = render_backend_matrix_summary();
    let api_stability_text = current_api_stability_profile().to_string();
    let api_stability_summary_text = render_api_stability_summary();
    let validation_report = build_validation_report(rounds)?;
    let validation_report_text = validation_report.to_string();
    let comparison_envelope_summary_text = render_comparison_envelope_summary_text();
    let comparison_corpus_release_guard_summary_text =
        render_comparison_corpus_release_guard_summary_text();
    let validation_report_summary_text = render_validation_report_summary_text(&validation_report);
    let benchmark_report_text = render_benchmark_report(rounds)?;
    let workspace_audit_summary_text = render_workspace_audit_summary()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let artifact_summary_text = render_artifact_summary()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let packaged_artifact_generation_manifest_text =
        packaged_artifact_generation_manifest_for_report();
    let provenance = workspace_provenance();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let profile_summary_path = output_dir.join("compatibility-profile-summary.txt");
    let release_notes_path = output_dir.join("release-notes.txt");
    let release_notes_summary_path = output_dir.join("release-notes-summary.txt");
    let release_summary_path = output_dir.join("release-summary.txt");
    let release_profile_identifiers_path = output_dir.join("release-profile-identifiers.txt");
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let release_checklist_summary_path = output_dir.join("release-checklist-summary.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let comparison_envelope_summary_path = output_dir.join("comparison-envelope-summary.txt");
    let comparison_corpus_release_guard_summary_path =
        output_dir.join("comparison-corpus-release-guard-summary.txt");
    let validation_report_summary_path = output_dir.join("validation-report-summary.txt");
    let workspace_audit_summary_path = output_dir.join("workspace-audit-summary.txt");
    let artifact_summary_path = output_dir.join("artifact-summary.txt");
    let packaged_artifact_generation_manifest_path =
        output_dir.join("packaged-artifact-generation-manifest.txt");
    let benchmark_report_path = output_dir.join("benchmark-report.txt");
    let report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let manifest_checksum_path = output_dir.join("bundle-manifest.checksum.txt");
    let compatibility_profile_checksum = checksum64(&profile_text);
    let compatibility_profile_summary_checksum = checksum64(&profile_summary_text);
    let release_notes_checksum = checksum64(&release_notes_text);
    let release_notes_summary_text = render_release_notes_summary_text();
    let release_notes_summary_checksum = checksum64(&release_notes_summary_text);
    let release_summary_checksum = checksum64(&release_summary_text);
    let release_profile_identifiers_checksum = checksum64(&release_profile_identifiers_text);
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let release_checklist_summary_checksum = checksum64(&release_checklist_summary_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let comparison_envelope_summary_checksum = checksum64(&comparison_envelope_summary_text);
    let comparison_corpus_release_guard_summary_checksum =
        checksum64(&comparison_corpus_release_guard_summary_text);
    let validation_report_summary_checksum = checksum64(&validation_report_summary_text);
    let workspace_audit_summary_checksum = checksum64(&workspace_audit_summary_text);
    let artifact_summary_checksum = checksum64(&artifact_summary_text);
    let packaged_artifact_generation_manifest_checksum =
        checksum64(&packaged_artifact_generation_manifest_text);
    let benchmark_report_checksum = checksum64(&benchmark_report_text);
    let validation_report_checksum = checksum64(&validation_report_text);
    let manifest_text = format!(
        "Release bundle manifest\nprofile: compatibility-profile.txt\nprofile checksum (fnv1a-64): 0x{compatibility_profile_checksum:016x}\nprofile summary: compatibility-profile-summary.txt\nprofile summary checksum (fnv1a-64): 0x{compatibility_profile_summary_checksum:016x}\nrelease notes: release-notes.txt\nrelease notes checksum (fnv1a-64): 0x{release_notes_checksum:016x}\nrelease notes summary: release-notes-summary.txt\nrelease notes summary checksum (fnv1a-64): 0x{release_notes_summary_checksum:016x}\nrelease summary: release-summary.txt\nrelease summary checksum (fnv1a-64): 0x{release_summary_checksum:016x}\nrelease-profile identifiers: release-profile-identifiers.txt\nrelease-profile identifiers checksum (fnv1a-64): 0x{release_profile_identifiers_checksum:016x}\nrelease checklist: release-checklist.txt\nrelease checklist checksum (fnv1a-64): 0x{release_checklist_checksum:016x}\nrelease checklist summary: release-checklist-summary.txt\nrelease checklist summary checksum (fnv1a-64): 0x{release_checklist_summary_checksum:016x}\nbackend matrix: backend-matrix.txt\nbackend matrix checksum (fnv1a-64): 0x{backend_matrix_checksum:016x}\nbackend matrix summary: backend-matrix-summary.txt\nbackend matrix summary checksum (fnv1a-64): 0x{backend_matrix_summary_checksum:016x}\napi stability posture: api-stability.txt\napi stability checksum (fnv1a-64): 0x{api_stability_checksum:016x}\napi stability summary: api-stability-summary.txt\napi stability summary checksum (fnv1a-64): 0x{api_stability_summary_checksum:016x}\ncomparison-envelope summary: comparison-envelope-summary.txt\ncomparison-envelope summary checksum (fnv1a-64): 0x{comparison_envelope_summary_checksum:016x}\ncomparison-corpus release-guard summary: comparison-corpus-release-guard-summary.txt\ncomparison-corpus release-guard summary checksum (fnv1a-64): 0x{comparison_corpus_release_guard_summary_checksum:016x}\nvalidation report summary: validation-report-summary.txt\nvalidation report summary checksum (fnv1a-64): 0x{validation_report_summary_checksum:016x}\nworkspace audit summary: workspace-audit-summary.txt\nworkspace audit summary checksum (fnv1a-64): 0x{workspace_audit_summary_checksum:016x}\nartifact summary: artifact-summary.txt\nartifact summary checksum (fnv1a-64): 0x{artifact_summary_checksum:016x}\npackaged-artifact generation manifest: packaged-artifact-generation-manifest.txt\npackaged-artifact generation manifest checksum (fnv1a-64): 0x{packaged_artifact_generation_manifest_checksum:016x}\nbenchmark report: benchmark-report.txt\nbenchmark report checksum (fnv1a-64): 0x{benchmark_report_checksum:016x}\nvalidation report: validation-report.txt\nvalidation report checksum (fnv1a-64): 0x{validation_report_checksum:016x}\nsource revision: {}\nworkspace status: {}\nrustc version: {}\nprofile id: {}\napi stability posture id: {}\nvalidation rounds: {}\n",
        provenance.source_revision,
        provenance.workspace_status,
        provenance.rustc_version,
        current_compatibility_profile().profile_id,
        current_api_stability_profile().profile_id,
        rounds,
    );

    fs::write(&profile_path, profile_text.as_bytes())?;
    fs::write(&profile_summary_path, profile_summary_text.as_bytes())?;
    fs::write(&release_notes_path, release_notes_text.as_bytes())?;
    fs::write(
        &release_notes_summary_path,
        release_notes_summary_text.as_bytes(),
    )?;
    fs::write(&release_summary_path, release_summary_text.as_bytes())?;
    fs::write(
        &release_profile_identifiers_path,
        release_profile_identifiers_text.as_bytes(),
    )?;
    fs::write(
        &backend_matrix_summary_path,
        backend_matrix_summary_text.as_bytes(),
    )?;
    fs::write(&release_checklist_path, release_checklist_text.as_bytes())?;
    fs::write(
        &release_checklist_summary_path,
        release_checklist_summary_text.as_bytes(),
    )?;
    fs::write(&backend_matrix_path, backend_matrix_text.as_bytes())?;
    fs::write(&api_stability_path, api_stability_text.as_bytes())?;
    fs::write(
        &api_stability_summary_path,
        api_stability_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_envelope_summary_path,
        comparison_envelope_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_corpus_release_guard_summary_path,
        comparison_corpus_release_guard_summary_text.as_bytes(),
    )?;
    fs::write(
        &validation_report_summary_path,
        validation_report_summary_text.as_bytes(),
    )?;
    fs::write(
        &workspace_audit_summary_path,
        workspace_audit_summary_text.as_bytes(),
    )?;
    let manifest_checksum = checksum64(&manifest_text);
    let manifest_checksum_text = format!("0x{manifest_checksum:016x}\n");
    fs::write(&artifact_summary_path, artifact_summary_text.as_bytes())?;
    fs::write(
        &packaged_artifact_generation_manifest_path,
        packaged_artifact_generation_manifest_text.as_bytes(),
    )?;
    fs::write(&benchmark_report_path, benchmark_report_text.as_bytes())?;
    fs::write(&report_path, validation_report_text.as_bytes())?;
    fs::write(&manifest_path, manifest_text.as_bytes())?;
    fs::write(&manifest_checksum_path, manifest_checksum_text.as_bytes())?;

    verify_release_bundle(output_dir)
}

#[derive(Debug)]
struct ParsedReleaseBundleManifest {
    profile_path: String,
    profile_checksum: u64,
    profile_summary_path: String,
    profile_summary_checksum: u64,
    release_notes_path: String,
    release_notes_checksum: u64,
    release_notes_summary_path: String,
    release_notes_summary_checksum: u64,
    release_summary_path: String,
    release_summary_checksum: u64,
    release_profile_identifiers_path: String,
    release_profile_identifiers_checksum: u64,
    release_checklist_path: String,
    release_checklist_checksum: u64,
    release_checklist_summary_path: String,
    release_checklist_summary_checksum: u64,
    backend_matrix_path: String,
    backend_matrix_checksum: u64,
    backend_matrix_summary_path: String,
    backend_matrix_summary_checksum: u64,
    api_stability_path: String,
    api_stability_checksum: u64,
    api_stability_summary_path: String,
    api_stability_summary_checksum: u64,
    comparison_envelope_summary_path: String,
    comparison_envelope_summary_checksum: u64,
    comparison_corpus_release_guard_summary_path: String,
    comparison_corpus_release_guard_summary_checksum: u64,
    validation_report_summary_path: String,
    validation_report_summary_checksum: u64,
    workspace_audit_summary_path: String,
    workspace_audit_summary_checksum: u64,
    artifact_summary_path: String,
    artifact_summary_checksum: u64,
    packaged_artifact_generation_manifest_path: String,
    packaged_artifact_generation_manifest_checksum: u64,
    benchmark_report_path: String,
    benchmark_report_checksum: u64,
    validation_report_path: String,
    validation_report_checksum: u64,
    source_revision: String,
    workspace_status: String,
    rustc_version: String,
    profile_id: String,
    api_stability_posture_id: String,
    validation_rounds: usize,
}

impl ParsedReleaseBundleManifest {
    fn parse(text: &str) -> Result<Self, ReleaseBundleError> {
        Ok(Self {
            profile_path: parse_manifest_string(text, "profile:")?,
            profile_checksum: parse_manifest_checksum(text, "profile checksum (fnv1a-64):")?,
            profile_summary_path: parse_manifest_string(text, "profile summary:")?,
            profile_summary_checksum: parse_manifest_checksum(
                text,
                "profile summary checksum (fnv1a-64):",
            )?,
            release_notes_path: parse_manifest_string(text, "release notes:")?,
            release_notes_checksum: parse_manifest_checksum(
                text,
                "release notes checksum (fnv1a-64):",
            )?,
            release_notes_summary_path: parse_manifest_string(text, "release notes summary:")?,
            release_notes_summary_checksum: parse_manifest_checksum(
                text,
                "release notes summary checksum (fnv1a-64):",
            )?,
            release_summary_path: parse_manifest_string(text, "release summary:")?,
            release_summary_checksum: parse_manifest_checksum(
                text,
                "release summary checksum (fnv1a-64):",
            )?,
            release_profile_identifiers_path: parse_manifest_string(
                text,
                "release-profile identifiers:",
            )?,
            release_profile_identifiers_checksum: parse_manifest_checksum(
                text,
                "release-profile identifiers checksum (fnv1a-64):",
            )?,
            release_checklist_path: parse_manifest_string(text, "release checklist:")?,
            release_checklist_checksum: parse_manifest_checksum(
                text,
                "release checklist checksum (fnv1a-64):",
            )?,
            release_checklist_summary_path: parse_manifest_string(
                text,
                "release checklist summary:",
            )?,
            release_checklist_summary_checksum: parse_manifest_checksum(
                text,
                "release checklist summary checksum (fnv1a-64):",
            )?,
            backend_matrix_path: parse_manifest_string(text, "backend matrix:")?,
            backend_matrix_checksum: parse_manifest_checksum(
                text,
                "backend matrix checksum (fnv1a-64):",
            )?,
            backend_matrix_summary_path: parse_manifest_string(text, "backend matrix summary:")?,
            backend_matrix_summary_checksum: parse_manifest_checksum(
                text,
                "backend matrix summary checksum (fnv1a-64):",
            )?,
            api_stability_path: parse_manifest_string(text, "api stability posture:")?,
            api_stability_checksum: parse_manifest_checksum(
                text,
                "api stability checksum (fnv1a-64):",
            )?,
            api_stability_summary_path: parse_manifest_string(text, "api stability summary:")?,
            api_stability_summary_checksum: parse_manifest_checksum(
                text,
                "api stability summary checksum (fnv1a-64):",
            )?,
            comparison_envelope_summary_path: parse_manifest_string(
                text,
                "comparison-envelope summary:",
            )?,
            comparison_envelope_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-envelope summary checksum (fnv1a-64):",
            )?,
            comparison_corpus_release_guard_summary_path: parse_manifest_string(
                text,
                "comparison-corpus release-guard summary:",
            )?,
            comparison_corpus_release_guard_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-corpus release-guard summary checksum (fnv1a-64):",
            )?,
            validation_report_summary_path: parse_manifest_string(
                text,
                "validation report summary:",
            )?,
            validation_report_summary_checksum: parse_manifest_checksum(
                text,
                "validation report summary checksum (fnv1a-64):",
            )?,
            workspace_audit_summary_path: parse_manifest_string(text, "workspace audit summary:")?,
            workspace_audit_summary_checksum: parse_manifest_checksum(
                text,
                "workspace audit summary checksum (fnv1a-64):",
            )?,
            artifact_summary_path: parse_manifest_string(text, "artifact summary:")?,
            artifact_summary_checksum: parse_manifest_checksum(
                text,
                "artifact summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_generation_manifest_path: parse_manifest_string(
                text,
                "packaged-artifact generation manifest:",
            )?,
            packaged_artifact_generation_manifest_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact generation manifest checksum (fnv1a-64):",
            )?,
            benchmark_report_path: parse_manifest_string(text, "benchmark report:")?,
            benchmark_report_checksum: parse_manifest_checksum(
                text,
                "benchmark report checksum (fnv1a-64):",
            )?,
            validation_report_path: parse_manifest_string(text, "validation report:")?,
            validation_report_checksum: parse_manifest_checksum(
                text,
                "validation report checksum (fnv1a-64):",
            )?,
            source_revision: parse_manifest_string(text, "source revision:")?,
            workspace_status: parse_manifest_string(text, "workspace status:")?,
            rustc_version: parse_manifest_string(text, "rustc version:")?,
            profile_id: parse_manifest_string(text, "profile id:")?,
            api_stability_posture_id: parse_manifest_string(text, "api stability posture id:")?,
            validation_rounds: parse_manifest_usize(
                text,
                "validation rounds:",
                "validation rounds",
            )?,
        })
    }
}

fn ensure_release_bundle_directory_contents(output_dir: &Path) -> Result<(), ReleaseBundleError> {
    let expected_entries: BTreeSet<String> = [
        "compatibility-profile.txt",
        "compatibility-profile-summary.txt",
        "release-notes.txt",
        "release-notes-summary.txt",
        "release-summary.txt",
        "release-profile-identifiers.txt",
        "release-checklist.txt",
        "release-checklist-summary.txt",
        "backend-matrix.txt",
        "backend-matrix-summary.txt",
        "api-stability.txt",
        "api-stability-summary.txt",
        "comparison-envelope-summary.txt",
        "comparison-corpus-release-guard-summary.txt",
        "validation-report-summary.txt",
        "workspace-audit-summary.txt",
        "artifact-summary.txt",
        "packaged-artifact-generation-manifest.txt",
        "benchmark-report.txt",
        "validation-report.txt",
        "bundle-manifest.txt",
        "bundle-manifest.checksum.txt",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut actual_entries = BTreeSet::new();
    for entry in fs::read_dir(output_dir)? {
        actual_entries.insert(entry?.file_name().to_string_lossy().into_owned());
    }

    if actual_entries != expected_entries {
        let unexpected = actual_entries
            .difference(&expected_entries)
            .cloned()
            .collect::<Vec<_>>();
        let missing = expected_entries
            .difference(&actual_entries)
            .cloned()
            .collect::<Vec<_>>();
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release bundle directory contents: unexpected [{}], missing [{}]",
            unexpected.join(", "),
            missing.join(", ")
        )));
    }

    Ok(())
}

fn ensure_release_bundle_manifest_is_canonical(
    manifest_text: &str,
) -> Result<(), ReleaseBundleError> {
    const EXPECTED_MANIFEST_LINES: [&str; 47] = [
        "Release bundle manifest",
        "profile:",
        "profile checksum (fnv1a-64):",
        "profile summary:",
        "profile summary checksum (fnv1a-64):",
        "release notes:",
        "release notes checksum (fnv1a-64):",
        "release notes summary:",
        "release notes summary checksum (fnv1a-64):",
        "release summary:",
        "release summary checksum (fnv1a-64):",
        "release-profile identifiers:",
        "release-profile identifiers checksum (fnv1a-64):",
        "release checklist:",
        "release checklist checksum (fnv1a-64):",
        "release checklist summary:",
        "release checklist summary checksum (fnv1a-64):",
        "backend matrix:",
        "backend matrix checksum (fnv1a-64):",
        "backend matrix summary:",
        "backend matrix summary checksum (fnv1a-64):",
        "api stability posture:",
        "api stability checksum (fnv1a-64):",
        "api stability summary:",
        "api stability summary checksum (fnv1a-64):",
        "comparison-envelope summary:",
        "comparison-envelope summary checksum (fnv1a-64):",
        "comparison-corpus release-guard summary:",
        "comparison-corpus release-guard summary checksum (fnv1a-64):",
        "validation report summary:",
        "validation report summary checksum (fnv1a-64):",
        "workspace audit summary:",
        "workspace audit summary checksum (fnv1a-64):",
        "artifact summary:",
        "artifact summary checksum (fnv1a-64):",
        "packaged-artifact generation manifest:",
        "packaged-artifact generation manifest checksum (fnv1a-64):",
        "benchmark report:",
        "benchmark report checksum (fnv1a-64):",
        "validation report:",
        "validation report checksum (fnv1a-64):",
        "source revision:",
        "workspace status:",
        "rustc version:",
        "profile id:",
        "api stability posture id:",
        "validation rounds:",
    ];

    let lines = manifest_text.lines().collect::<Vec<_>>();
    if lines.len() != EXPECTED_MANIFEST_LINES.len() {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release bundle manifest line count: expected {}, found {}",
            EXPECTED_MANIFEST_LINES.len(),
            lines.len()
        )));
    }

    for (index, (line, expected_prefix)) in lines.iter().zip(EXPECTED_MANIFEST_LINES).enumerate() {
        if !line.starts_with(expected_prefix) {
            return Err(ReleaseBundleError::Verification(format!(
                "unexpected release bundle manifest line {}: expected prefix `{}`, found `{}`",
                index + 1,
                expected_prefix,
                line
            )));
        }
    }

    Ok(())
}

fn ensure_release_bundle_regular_file(path: &Path, label: &str) -> Result<(), ReleaseBundleError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(ReleaseBundleError::Verification(format!(
            "unexpected non-regular {label} file: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ReleaseBundleError::Io(error)),
    }
}

fn read_required_bundle_text(path: &Path, label: &str) -> Result<String, ReleaseBundleError> {
    fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ReleaseBundleError::Verification(format!("missing {label} file: {}", path.display()))
        } else {
            ReleaseBundleError::Io(error)
        }
    })
}

fn verify_release_bundle(
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let profile_summary_path = output_dir.join("compatibility-profile-summary.txt");
    let release_notes_path = output_dir.join("release-notes.txt");
    let release_notes_summary_path = output_dir.join("release-notes-summary.txt");
    let release_summary_path = output_dir.join("release-summary.txt");
    let release_profile_identifiers_path = output_dir.join("release-profile-identifiers.txt");
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let release_checklist_summary_path = output_dir.join("release-checklist-summary.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let comparison_envelope_summary_path = output_dir.join("comparison-envelope-summary.txt");
    let comparison_corpus_release_guard_summary_path =
        output_dir.join("comparison-corpus-release-guard-summary.txt");
    let validation_report_summary_path = output_dir.join("validation-report-summary.txt");
    let workspace_audit_summary_path = output_dir.join("workspace-audit-summary.txt");
    let artifact_summary_path = output_dir.join("artifact-summary.txt");
    let packaged_artifact_generation_manifest_path =
        output_dir.join("packaged-artifact-generation-manifest.txt");
    let benchmark_report_path = output_dir.join("benchmark-report.txt");
    let validation_report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let manifest_checksum_path = output_dir.join("bundle-manifest.checksum.txt");

    for (path, label) in [
        (&profile_path, "compatibility profile"),
        (&profile_summary_path, "compatibility profile summary"),
        (&release_notes_path, "release notes"),
        (&release_notes_summary_path, "release notes summary"),
        (&release_summary_path, "release summary"),
        (
            &release_profile_identifiers_path,
            "release-profile identifiers",
        ),
        (&release_checklist_path, "release checklist"),
        (&release_checklist_summary_path, "release checklist summary"),
        (&backend_matrix_path, "backend matrix"),
        (&backend_matrix_summary_path, "backend matrix summary"),
        (&api_stability_path, "API stability"),
        (&api_stability_summary_path, "API stability summary"),
        (
            &comparison_envelope_summary_path,
            "comparison envelope summary",
        ),
        (
            &comparison_corpus_release_guard_summary_path,
            "comparison-corpus release-guard summary",
        ),
        (&validation_report_summary_path, "validation report summary"),
        (&workspace_audit_summary_path, "workspace audit summary"),
        (&artifact_summary_path, "artifact summary"),
        (&benchmark_report_path, "benchmark report"),
        (&validation_report_path, "validation report"),
        (&manifest_path, "bundle manifest"),
        (&manifest_checksum_path, "bundle manifest checksum sidecar"),
    ] {
        ensure_release_bundle_regular_file(path, label)?;
    }

    let profile_text = read_required_bundle_text(&profile_path, "compatibility profile")?;
    let profile_summary_text =
        read_required_bundle_text(&profile_summary_path, "compatibility profile summary")?;
    let release_notes_text = read_required_bundle_text(&release_notes_path, "release notes")?;
    let release_notes_summary_text =
        read_required_bundle_text(&release_notes_summary_path, "release notes summary")?;
    let release_summary_text = read_required_bundle_text(&release_summary_path, "release summary")?;
    let release_profile_identifiers_text = read_required_bundle_text(
        &release_profile_identifiers_path,
        "release-profile identifiers",
    )?;
    let release_checklist_text =
        read_required_bundle_text(&release_checklist_path, "release checklist")?;
    let release_checklist_summary_text =
        read_required_bundle_text(&release_checklist_summary_path, "release checklist summary")?;
    let backend_matrix_text = read_required_bundle_text(&backend_matrix_path, "backend matrix")?;
    let backend_matrix_summary_text =
        read_required_bundle_text(&backend_matrix_summary_path, "backend matrix summary")?;
    let api_stability_text = read_required_bundle_text(&api_stability_path, "API stability")?;
    let api_stability_summary_text =
        read_required_bundle_text(&api_stability_summary_path, "API stability summary")?;
    let comparison_envelope_summary_text = read_required_bundle_text(
        &comparison_envelope_summary_path,
        "comparison envelope summary",
    )?;
    let comparison_corpus_release_guard_summary_text = read_required_bundle_text(
        &comparison_corpus_release_guard_summary_path,
        "comparison-corpus release-guard summary",
    )?;
    let validation_report_summary_text =
        read_required_bundle_text(&validation_report_summary_path, "validation report summary")?;
    let workspace_audit_summary_text =
        read_required_bundle_text(&workspace_audit_summary_path, "workspace audit summary")?;
    let artifact_summary_text =
        read_required_bundle_text(&artifact_summary_path, "artifact summary")?;
    let packaged_artifact_generation_manifest_text = read_required_bundle_text(
        &packaged_artifact_generation_manifest_path,
        "packaged-artifact generation manifest",
    )?;
    let benchmark_report_text =
        read_required_bundle_text(&benchmark_report_path, "benchmark report")?;
    let validation_report_text =
        read_required_bundle_text(&validation_report_path, "validation report")?;
    let manifest_text = read_required_bundle_text(&manifest_path, "bundle manifest")?;
    let manifest_checksum_text =
        read_required_bundle_text(&manifest_checksum_path, "bundle manifest checksum sidecar")?;

    let manifest = ParsedReleaseBundleManifest::parse(&manifest_text)?;
    ensure_release_bundle_manifest_is_canonical(&manifest_text)?;
    ensure_release_bundle_directory_contents(output_dir)?;
    ensure_canonical_manifest_value(&manifest.source_revision, "source revision")?;
    ensure_canonical_manifest_value(&manifest.workspace_status, "workspace status")?;
    ensure_canonical_manifest_value(&manifest.rustc_version, "rustc version")?;
    ensure_canonical_manifest_value(&manifest.profile_id, "profile id")?;
    ensure_canonical_manifest_value(
        &manifest.api_stability_posture_id,
        "API stability posture id",
    )?;
    if manifest.profile_path != "compatibility-profile.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected profile file entry: {}",
            manifest.profile_path
        )));
    }
    if manifest.profile_summary_path != "compatibility-profile-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected compatibility profile summary file entry: {}",
            manifest.profile_summary_path
        )));
    }
    if manifest.release_notes_path != "release-notes.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release notes file entry: {}",
            manifest.release_notes_path
        )));
    }
    if manifest.release_notes_summary_path != "release-notes-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release notes summary file entry: {}",
            manifest.release_notes_summary_path
        )));
    }
    if manifest.release_summary_path != "release-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release summary file entry: {}",
            manifest.release_summary_path
        )));
    }
    if manifest.release_profile_identifiers_path != "release-profile-identifiers.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release-profile identifiers file entry: {}",
            manifest.release_profile_identifiers_path
        )));
    }
    if manifest.release_checklist_path != "release-checklist.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release checklist file entry: {}",
            manifest.release_checklist_path
        )));
    }
    if manifest.release_checklist_summary_path != "release-checklist-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release checklist summary file entry: {}",
            manifest.release_checklist_summary_path
        )));
    }
    if manifest.backend_matrix_path != "backend-matrix.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected backend matrix file entry: {}",
            manifest.backend_matrix_path
        )));
    }
    if manifest.backend_matrix_summary_path != "backend-matrix-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected backend matrix summary file entry: {}",
            manifest.backend_matrix_summary_path
        )));
    }
    if manifest.api_stability_path != "api-stability.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected API stability file entry: {}",
            manifest.api_stability_path
        )));
    }
    if manifest.api_stability_summary_path != "api-stability-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected API stability summary file entry: {}",
            manifest.api_stability_summary_path
        )));
    }
    if manifest.comparison_envelope_summary_path != "comparison-envelope-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison envelope summary file entry: {}",
            manifest.comparison_envelope_summary_path
        )));
    }
    if manifest.comparison_corpus_release_guard_summary_path
        != "comparison-corpus-release-guard-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison-corpus release-guard summary file entry: {}",
            manifest.comparison_corpus_release_guard_summary_path
        )));
    }
    if manifest.validation_report_summary_path != "validation-report-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected validation report summary file entry: {}",
            manifest.validation_report_summary_path
        )));
    }
    if manifest.workspace_audit_summary_path != "workspace-audit-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected workspace audit summary file entry: {}",
            manifest.workspace_audit_summary_path
        )));
    }
    if manifest.artifact_summary_path != "artifact-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected artifact summary file entry: {}",
            manifest.artifact_summary_path
        )));
    }
    if manifest.packaged_artifact_generation_manifest_path
        != "packaged-artifact-generation-manifest.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact generation manifest file entry: {}",
            manifest.packaged_artifact_generation_manifest_path
        )));
    }
    if manifest.benchmark_report_path != "benchmark-report.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected benchmark report file entry: {}",
            manifest.benchmark_report_path
        )));
    }
    if manifest.validation_report_path != "validation-report.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected validation report file entry: {}",
            manifest.validation_report_path
        )));
    }

    let compatibility_profile_checksum = checksum64(&profile_text);
    let compatibility_profile_summary_checksum = checksum64(&profile_summary_text);
    let release_notes_summary_checksum = checksum64(&release_notes_summary_text);
    let release_summary_checksum = checksum64(&release_summary_text);
    let release_profile_identifiers_checksum = checksum64(&release_profile_identifiers_text);
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let release_checklist_summary_checksum = checksum64(&release_checklist_summary_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let comparison_envelope_summary_checksum = checksum64(&comparison_envelope_summary_text);
    let comparison_corpus_release_guard_summary_checksum =
        checksum64(&comparison_corpus_release_guard_summary_text);
    let validation_report_summary_checksum = checksum64(&validation_report_summary_text);
    let workspace_audit_summary_checksum = checksum64(&workspace_audit_summary_text);
    let artifact_summary_checksum = checksum64(&artifact_summary_text);
    let packaged_artifact_generation_manifest_checksum =
        checksum64(&packaged_artifact_generation_manifest_text);
    let benchmark_report_checksum = checksum64(&benchmark_report_text);
    let validation_report_checksum = checksum64(&validation_report_text);
    let manifest_checksum = checksum64(&manifest_text);
    let manifest_checksum_value =
        parse_checksum_value(&manifest_checksum_text, "bundle manifest checksum sidecar")?;
    let profile_id = extract_prefixed_value(&profile_text, "Compatibility profile: ")?;
    let api_stability_posture_id =
        extract_prefixed_value(&api_stability_text, "API stability posture: ")?;

    if manifest.release_summary_checksum != release_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_summary_checksum, release_summary_checksum
        )));
    }
    if manifest.release_profile_identifiers_checksum != release_profile_identifiers_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release-profile identifiers checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_profile_identifiers_checksum, release_profile_identifiers_checksum
        )));
    }
    if manifest.release_checklist_checksum != release_checklist_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release checklist checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_checklist_checksum, release_checklist_checksum
        )));
    }
    if manifest.release_checklist_summary_checksum != release_checklist_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release checklist summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_checklist_summary_checksum, release_checklist_summary_checksum
        )));
    }
    if manifest.profile_id != profile_id {
        return Err(ReleaseBundleError::Verification(format!(
            "profile id mismatch: manifest has {}, file has {}",
            manifest.profile_id, profile_id
        )));
    }
    if manifest.profile_checksum != compatibility_profile_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "compatibility profile checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.profile_checksum, compatibility_profile_checksum
        )));
    }
    if manifest.profile_summary_checksum != compatibility_profile_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "compatibility profile summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.profile_summary_checksum, compatibility_profile_summary_checksum
        )));
    }
    if manifest.release_notes_summary_checksum != release_notes_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release notes summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_notes_summary_checksum, release_notes_summary_checksum
        )));
    }
    let release_notes_checksum = checksum64(&release_notes_text);
    if manifest.release_notes_checksum != release_notes_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release notes checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_notes_checksum, release_notes_checksum
        )));
    }
    if manifest.backend_matrix_checksum != backend_matrix_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "backend matrix checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.backend_matrix_checksum, backend_matrix_checksum
        )));
    }
    if manifest.backend_matrix_summary_checksum != backend_matrix_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "backend matrix summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.backend_matrix_summary_checksum, backend_matrix_summary_checksum
        )));
    }
    if manifest.api_stability_posture_id != api_stability_posture_id {
        return Err(ReleaseBundleError::Verification(format!(
            "API stability posture id mismatch: manifest has {}, file has {}",
            manifest.api_stability_posture_id, api_stability_posture_id
        )));
    }
    if manifest.api_stability_checksum != api_stability_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "API stability checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.api_stability_checksum, api_stability_checksum
        )));
    }
    if manifest.api_stability_summary_checksum != api_stability_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "API stability summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.api_stability_summary_checksum, api_stability_summary_checksum
        )));
    }

    ensure_release_profile_line_alignment("release notes", &release_notes_text, profile_id)?;
    ensure_release_profile_summary_alignment(
        "release notes summary",
        &release_notes_summary_text,
        profile_id,
        api_stability_posture_id,
    )?;
    ensure_release_profile_summary_alignment(
        "release summary",
        &release_summary_text,
        profile_id,
        api_stability_posture_id,
    )?;
    ensure_release_profile_summary_alignment(
        "release checklist",
        &release_checklist_text,
        profile_id,
        api_stability_posture_id,
    )?;
    ensure_release_profile_identifiers_alignment(
        &release_profile_identifiers_text,
        profile_id,
        api_stability_posture_id,
    )?;

    if manifest.comparison_envelope_summary_checksum != comparison_envelope_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison envelope summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_envelope_summary_checksum, comparison_envelope_summary_checksum
        )));
    }
    if manifest.comparison_corpus_release_guard_summary_checksum
        != comparison_corpus_release_guard_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison-corpus release-guard summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_corpus_release_guard_summary_checksum,
            comparison_corpus_release_guard_summary_checksum
        )));
    }
    if manifest.validation_report_summary_checksum != validation_report_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "validation report summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.validation_report_summary_checksum, validation_report_summary_checksum
        )));
    }
    if manifest.workspace_audit_summary_checksum != workspace_audit_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "workspace audit summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.workspace_audit_summary_checksum, workspace_audit_summary_checksum
        )));
    }
    if manifest.artifact_summary_checksum != artifact_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "artifact summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.artifact_summary_checksum, artifact_summary_checksum
        )));
    }
    if manifest.packaged_artifact_generation_manifest_checksum
        != packaged_artifact_generation_manifest_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation manifest checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_generation_manifest_checksum,
            packaged_artifact_generation_manifest_checksum
        )));
    }
    if manifest.benchmark_report_checksum != benchmark_report_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "benchmark report checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.benchmark_report_checksum, benchmark_report_checksum
        )));
    }
    if manifest.validation_report_checksum != validation_report_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "validation report checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.validation_report_checksum, validation_report_checksum
        )));
    }
    if manifest_checksum_value != manifest_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "bundle manifest checksum mismatch: manifest has 0x{:016x}, checksum file has 0x{:016x}",
            manifest_checksum, manifest_checksum_value
        )));
    }

    let bundle = ReleaseBundle {
        source_revision: manifest.source_revision,
        workspace_status: manifest.workspace_status,
        rustc_version: manifest.rustc_version,
        output_dir: output_dir.to_path_buf(),
        compatibility_profile_path: profile_path,
        compatibility_profile_summary_path: profile_summary_path,
        release_notes_path,
        release_notes_summary_path,
        release_summary_path,
        release_profile_identifiers_path,
        release_checklist_path,
        release_checklist_summary_path,
        backend_matrix_path,
        backend_matrix_summary_path,
        api_stability_path,
        api_stability_summary_path,
        comparison_envelope_summary_path,
        comparison_corpus_release_guard_summary_path,
        validation_report_summary_path,
        workspace_audit_summary_path,
        artifact_summary_path,
        packaged_artifact_generation_manifest_path,
        benchmark_report_path,
        validation_report_path,
        manifest_path,
        manifest_checksum_path,
        compatibility_profile_bytes: profile_text.len(),
        compatibility_profile_summary_bytes: profile_summary_text.len(),
        release_notes_bytes: release_notes_text.len(),
        release_notes_summary_bytes: release_notes_summary_text.len(),
        release_summary_bytes: release_summary_text.len(),
        release_profile_identifiers_bytes: release_profile_identifiers_text.len(),
        release_checklist_bytes: release_checklist_text.len(),
        release_checklist_summary_bytes: release_checklist_summary_text.len(),
        backend_matrix_bytes: backend_matrix_text.len(),
        backend_matrix_summary_bytes: backend_matrix_summary_text.len(),
        api_stability_bytes: api_stability_text.len(),
        api_stability_summary_bytes: api_stability_summary_text.len(),
        comparison_envelope_summary_bytes: comparison_envelope_summary_text.len(),
        comparison_corpus_release_guard_summary_bytes: comparison_corpus_release_guard_summary_text
            .len(),
        validation_report_summary_bytes: validation_report_summary_text.len(),
        workspace_audit_summary_bytes: workspace_audit_summary_text.len(),
        artifact_summary_bytes: artifact_summary_text.len(),
        packaged_artifact_generation_manifest_bytes: packaged_artifact_generation_manifest_text
            .len(),
        benchmark_report_bytes: benchmark_report_text.len(),
        validation_report_bytes: validation_report_text.len(),
        manifest_checksum_bytes: manifest_checksum_text.len(),
        compatibility_profile_checksum,
        compatibility_profile_summary_checksum,
        release_notes_checksum,
        release_notes_summary_checksum,
        release_summary_checksum,
        release_profile_identifiers_checksum,
        release_checklist_checksum,
        release_checklist_summary_checksum,
        backend_matrix_checksum,
        backend_matrix_summary_checksum,
        api_stability_checksum,
        api_stability_summary_checksum,
        comparison_envelope_summary_checksum,
        comparison_corpus_release_guard_summary_checksum,
        validation_report_summary_checksum,
        workspace_audit_summary_checksum,
        artifact_summary_checksum,
        packaged_artifact_generation_manifest_checksum,
        benchmark_report_checksum,
        validation_report_checksum,
        manifest_checksum: manifest_checksum_value,
        validation_rounds: manifest.validation_rounds,
    };
    bundle.validate()?;
    Ok(bundle)
}

fn parse_manifest_string(text: &str, prefix: &str) -> Result<String, ReleaseBundleError> {
    extract_prefixed_value(text, prefix).map(|value| value.to_string())
}

fn ensure_canonical_manifest_value(
    value: &str,
    field_name: &str,
) -> Result<(), ReleaseBundleError> {
    if value.is_empty() {
        return Err(ReleaseBundleError::Verification(format!(
            "missing {field_name} entry"
        )));
    }

    if value != value.trim() {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {field_name} entry: unexpected leading or trailing whitespace"
        )));
    }

    if value.contains('\n') || value.contains('\r') {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {field_name} entry: unexpected line break"
        )));
    }

    if value.eq_ignore_ascii_case("unknown") {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {field_name} entry: placeholder values are not allowed"
        )));
    }

    Ok(())
}

fn parse_manifest_usize(
    text: &str,
    prefix: &str,
    field_name: &str,
) -> Result<usize, ReleaseBundleError> {
    let value = extract_prefixed_value(text, prefix)?;
    value.parse::<usize>().map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {field_name} entry: {error}"))
    })
}

fn parse_manifest_checksum(text: &str, prefix: &str) -> Result<u64, ReleaseBundleError> {
    let value = extract_prefixed_value(text, prefix)?;
    let value = value.strip_prefix("0x").ok_or_else(|| {
        ReleaseBundleError::Verification(format!("missing 0x prefix for {prefix}"))
    })?;
    if value.len() != 16 || !value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')) {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {prefix} value: expected exactly 16 lowercase hex digits"
        )));
    }
    u64::from_str_radix(value, 16).map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {prefix} value: {error}"))
    })
}

fn parse_checksum_value(text: &str, label: &str) -> Result<u64, ReleaseBundleError> {
    let mut lines = text.lines();
    let Some(line) = lines.next() else {
        return Err(ReleaseBundleError::Verification(format!("missing {label}")));
    };

    if lines.next().is_some() {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {label} value: expected exactly one checksum line"
        )));
    }

    if line != line.trim() {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {label} value: unexpected leading or trailing whitespace"
        )));
    }

    let value = line.strip_prefix("0x").ok_or_else(|| {
        ReleaseBundleError::Verification(format!("missing 0x prefix for {label}"))
    })?;
    if value.len() != 16 || !value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')) {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {label} value: expected exactly 16 lowercase hex digits"
        )));
    }
    u64::from_str_radix(value, 16).map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {label} value: {error}"))
    })
}

fn extract_prefixed_value<'a>(text: &'a str, prefix: &str) -> Result<&'a str, ReleaseBundleError> {
    let mut matches = text.lines().filter_map(|line| line.strip_prefix(prefix));

    let Some(value) = matches.next() else {
        return Err(ReleaseBundleError::Verification(format!(
            "missing manifest entry: {prefix}"
        )));
    };

    if matches.next().is_some() {
        return Err(ReleaseBundleError::Verification(format!(
            "duplicate entry: {prefix}"
        )));
    }

    if value.is_empty() {
        return Ok(value);
    }

    let value = if prefix.ends_with(' ') {
        value
    } else {
        let Some(value) = value.strip_prefix(' ') else {
            return Err(ReleaseBundleError::Verification(format!(
                "unexpected whitespace in manifest entry: {prefix}"
            )));
        };
        value
    };

    if value != value.trim() {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected leading or trailing whitespace in manifest entry: {prefix}"
        )));
    }

    Ok(value)
}

fn ensure_release_profile_line_alignment(
    label: &str,
    text: &str,
    expected_profile_id: &str,
) -> Result<(), ReleaseBundleError> {
    let profile_id = match extract_prefixed_value(text, "Profile: ") {
        Ok(profile_id) => profile_id,
        Err(ReleaseBundleError::Verification(message)) => {
            return Err(ReleaseBundleError::Verification(format!(
                "{label}: {message}"
            )));
        }
        Err(error) => return Err(error),
    };

    if profile_id != expected_profile_id {
        return Err(ReleaseBundleError::Verification(format!(
            "{label} profile id mismatch: expected {expected_profile_id}, found {profile_id}"
        )));
    }

    Ok(())
}

fn ensure_release_profile_summary_alignment(
    label: &str,
    text: &str,
    expected_profile_id: &str,
    expected_api_stability_posture_id: &str,
) -> Result<(), ReleaseBundleError> {
    ensure_release_profile_line_alignment(label, text, expected_profile_id)?;

    let api_stability_posture_id = match extract_prefixed_value(text, "API stability posture: ") {
        Ok(api_stability_posture_id) => api_stability_posture_id,
        Err(ReleaseBundleError::Verification(message)) => {
            return Err(ReleaseBundleError::Verification(format!(
                "{label}: {message}"
            )));
        }
        Err(error) => return Err(error),
    };

    if api_stability_posture_id != expected_api_stability_posture_id {
        return Err(ReleaseBundleError::Verification(format!(
            "{label} API stability posture id mismatch: expected {expected_api_stability_posture_id}, found {api_stability_posture_id}"
        )));
    }

    Ok(())
}

fn ensure_release_profile_identifiers_alignment(
    text: &str,
    expected_profile_id: &str,
    expected_api_stability_posture_id: &str,
) -> Result<(), ReleaseBundleError> {
    let expected = format!(
        "Release profile identifiers: v1 compatibility={expected_profile_id}, api-stability={expected_api_stability_posture_id}"
    );
    let found = text.trim_end();
    if found != expected {
        return Err(ReleaseBundleError::Verification(format!(
            "release-profile identifiers mismatch: expected '{expected}', found '{found}'"
        )));
    }

    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn workspace_manifest_paths(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut manifests = vec![root.join("Cargo.toml")];
    let crates_dir = root.join("crates");
    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        let manifest = entry.path().join("Cargo.toml");
        if manifest.is_file() {
            manifests.push(manifest);
        }
    }
    manifests.sort();
    Ok(manifests)
}

fn manifest_has_assignment(line: &str, key: &str) -> bool {
    let Some(rest) = line.strip_prefix(key) else {
        return false;
    };
    rest.trim_start().starts_with('=')
}

fn manifest_dependency_rule(line: &str, forbidden: &str) -> bool {
    if manifest_has_assignment(line, forbidden) {
        return true;
    }

    line.contains(&format!("package = \"{forbidden}\""))
}

fn manifest_dependency_name(line: &str) -> Option<&str> {
    let (name, _) = line.split_once('=')?;
    let name = name.trim().trim_matches('"');
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn manifest_dependency_package_name(line: &str) -> Option<&str> {
    let needle = "package = \"";
    let start = line.find(needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    let package_name = &rest[..end];
    if package_name.is_empty() {
        None
    } else {
        Some(package_name)
    }
}

fn audit_manifest_text(path: &Path, text: &str) -> Vec<WorkspaceAuditViolation> {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Other,
        Package,
        Dependencies,
    }

    const FORBIDDEN_DEPENDENCIES: [&str; 4] = ["cc", "bindgen", "cmake", "pkg-config"];

    let mut section = Section::Other;
    let mut violations = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = if line == "[package]" {
                Section::Package
            } else if line == "[dependencies]"
                || line == "[dev-dependencies]"
                || line == "[build-dependencies]"
                || line.contains(".dependencies]")
            {
                Section::Dependencies
            } else {
                Section::Other
            };
            continue;
        }

        match section {
            Section::Package => {
                if manifest_has_assignment(line, "build") {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "package.build",
                        detail: "package declares a build script, which violates the pure-Rust workspace policy".to_string(),
                    });
                }
                if manifest_has_assignment(line, "links") {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "package.links",
                        detail: "package declares a native links value, which indicates an external build requirement".to_string(),
                    });
                }
            }
            Section::Dependencies => {
                if let Some(native_package_name) = manifest_dependency_name(line)
                    .filter(|name| name.ends_with("-sys"))
                    .or_else(|| {
                        manifest_dependency_package_name(line).filter(|name| name.ends_with("-sys"))
                    })
                {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "dependency.native-package",
                        detail: format!(
                            "dependency table references `{native_package_name}`, which suggests a native build dependency"
                        ),
                    });
                }

                for forbidden in FORBIDDEN_DEPENDENCIES {
                    if manifest_dependency_rule(line, forbidden) {
                        violations.push(WorkspaceAuditViolation {
                            path: path.to_path_buf(),
                            rule: "dependency.native-tool",
                            detail: format!(
                                "dependency table references `{forbidden}`, which is reserved for native build tooling"
                            ),
                        });
                    }
                }
            }
            Section::Other => {}
        }
    }

    violations
}

fn audit_lockfile_text(path: &Path, text: &str) -> Vec<WorkspaceAuditViolation> {
    const FORBIDDEN_LOCKFILE_PACKAGES: [&str; 4] = ["cc", "bindgen", "cmake", "pkg-config"];
    let mut violations = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        let Some(name) = line.strip_prefix("name = \"") else {
            continue;
        };
        let Some((package_name, _)) = name.split_once('"') else {
            continue;
        };
        if package_name.ends_with("-sys") || FORBIDDEN_LOCKFILE_PACKAGES.contains(&package_name) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "lockfile.native-package",
                detail: format!(
                    "lockfile package `{package_name}` suggests a native build dependency and should be reviewed"
                ),
            });
        }
    }

    violations
}

fn audit_build_script_path(manifest_path: &Path) -> Option<WorkspaceAuditViolation> {
    let build_script = manifest_path.parent()?.join("build.rs");
    if build_script.is_file() {
        Some(WorkspaceAuditViolation {
            path: build_script,
            rule: "package.build-script",
            detail:
                "package includes a build.rs script, which violates the pure-Rust workspace policy"
                    .to_string(),
        })
    } else {
        None
    }
}

/// Renders the workspace audit used by the CLI and release smoke checks.
pub fn workspace_audit_report() -> Result<WorkspaceAuditReport, std::io::Error> {
    let workspace_root = fs::canonicalize(workspace_root())?;
    let manifest_paths = workspace_manifest_paths(&workspace_root)?;
    let lockfile_path = workspace_root.join("Cargo.lock");
    let mut violations = Vec::new();

    for path in &manifest_paths {
        let text = fs::read_to_string(path)?;
        violations.extend(audit_manifest_text(path, &text));
        if let Some(violation) = audit_build_script_path(path) {
            violations.push(violation);
        }
    }

    if lockfile_path.is_file() {
        let text = fs::read_to_string(&lockfile_path)?;
        violations.extend(audit_lockfile_text(&lockfile_path, &text));
    } else {
        violations.push(WorkspaceAuditViolation {
            path: lockfile_path.clone(),
            rule: "lockfile.missing",
            detail: "Cargo.lock is missing from the workspace root".to_string(),
        });
    }

    Ok(WorkspaceAuditReport {
        workspace_root,
        manifest_paths,
        lockfile_path,
        violations,
    })
}

fn build_validation_report(rounds: usize) -> Result<ValidationReport, EphemerisError> {
    let comparison_corpus = release_grade_corpus();
    let benchmark_corpus = benchmark_corpus();
    let packaged_benchmark_corpus = artifact::packaged_artifact_corpus();
    let chart_benchmark_corpus = chart_benchmark_corpus_summary();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let packaged = PackagedDataBackend::new();
    let comparison = compare_backends(&reference, &candidate, &comparison_corpus)?;
    let reference_benchmark = benchmark_backend(&reference, &comparison_corpus, rounds)?;
    let candidate_benchmark = benchmark_backend(&candidate, &benchmark_corpus, rounds)?;
    let packaged_benchmark = benchmark_backend(&packaged, &packaged_benchmark_corpus, rounds)?;
    let artifact_decode_benchmark =
        artifact::benchmark_packaged_artifact_decode(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let chart_benchmark = benchmark_chart_backend(default_candidate_backend(), rounds)?;
    let archived_regressions = comparison.regression_archive();

    let report = ValidationReport {
        comparison_corpus: comparison_corpus.summary(),
        benchmark_corpus: benchmark_corpus.summary(),
        packaged_benchmark_corpus: packaged_benchmark_corpus.summary(),
        chart_benchmark_corpus,
        artifact_decode_benchmark,
        house_validation: house_validation_report(),
        comparison,
        archived_regressions,
        reference_benchmark,
        candidate_benchmark,
        packaged_benchmark,
        chart_benchmark,
    };
    report.validate()?;
    Ok(report)
}

/// Renders the validation report used by the CLI.
pub fn render_validation_report(rounds: usize) -> Result<String, EphemerisError> {
    Ok(build_validation_report(rounds)?.to_string())
}

/// Renders a compact validation-report summary used by the CLI.
pub fn render_validation_report_summary(rounds: usize) -> Result<String, EphemerisError> {
    let report = build_validation_report(rounds)?;
    Ok(render_validation_report_summary_text(&report))
}

/// Renders the comparison report used by the CLI.
pub fn render_comparison_report() -> Result<String, EphemerisError> {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    Ok(compare_backends(&reference, &candidate, &corpus)?.to_string())
}

fn comparison_audit_summary(report: &ComparisonReport) -> ComparisonAuditSummary {
    let tolerance_summaries = report.tolerance_summaries();
    let body_count = tolerance_summaries.len();
    let within_tolerance_body_count = tolerance_summaries
        .iter()
        .filter(|summary| summary.within_tolerance)
        .count();
    let outside_tolerance_body_count = body_count.saturating_sub(within_tolerance_body_count);
    let regression_count = report.notable_regressions().len();

    ComparisonAuditSummary {
        body_count,
        within_tolerance_body_count,
        outside_tolerance_body_count,
        regression_count,
    }
}

fn comparison_audit_summary_for_report(report: &ComparisonReport) -> String {
    let summary = comparison_audit_summary(report);

    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("comparison audit unavailable ({error})"),
    }
}

fn comparison_audit_totals(report: &ComparisonReport) -> (usize, usize, usize, usize) {
    let summary = comparison_audit_summary(report);

    (
        summary.body_count,
        summary.within_tolerance_body_count,
        summary.outside_tolerance_body_count,
        summary.regression_count,
    )
}

fn format_regression_bodies(regressions: &[RegressionFinding]) -> String {
    let mut bodies = regressions
        .iter()
        .map(|finding| finding.body.to_string())
        .collect::<Vec<_>>();
    bodies.sort();
    bodies.dedup();

    if bodies.is_empty() {
        "none".to_string()
    } else {
        bodies.join(", ")
    }
}

fn format_summary_body(body: &Option<CelestialBody>) -> String {
    body.as_ref()
        .map(|body| format!(" ({body})"))
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonMedianEnvelope {
    longitude_delta_deg: f64,
    latitude_delta_deg: f64,
    distance_delta_au: Option<f64>,
}

impl ComparisonMedianEnvelope {
    /// Validates the stored median comparison envelope.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison median envelope field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = self.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison median envelope field `distance_delta_au` must be a finite non-negative value",
                ));
            }
        }

        Ok(())
    }

    /// Returns the compact median comparison envelope line.
    pub fn summary_line(&self) -> String {
        let distance = self
            .distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "median longitude delta: {:.12}°, median latitude delta: {:.12}°, median distance delta: {}",
            self.longitude_delta_deg, self.latitude_delta_deg, distance,
        )
    }
}

impl fmt::Display for ComparisonMedianEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the median comparison envelope used by the compact report.
pub fn comparison_median_envelope(
    samples: &[ComparisonSample],
) -> Result<ComparisonMedianEnvelope, EphemerisError> {
    validate_comparison_samples_for_report(samples)?;

    let envelope = comparison_median_envelope_for_samples(samples);
    envelope.validate()?;
    Ok(envelope)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonPercentileEnvelope {
    longitude_delta_deg: f64,
    latitude_delta_deg: f64,
    distance_delta_au: Option<f64>,
}

impl ComparisonPercentileEnvelope {
    /// Validates the stored 95th-percentile comparison envelope.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison percentile envelope field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = self.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison percentile envelope field `distance_delta_au` must be a finite non-negative value",
                ));
            }
        }

        Ok(())
    }

    /// Returns the compact 95th-percentile comparison envelope line.
    pub fn summary_line(&self) -> String {
        let distance = self
            .distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "95th percentile absolute deltas: longitude {:.12}°, latitude {:.12}°, distance {}",
            self.longitude_delta_deg, self.latitude_delta_deg, distance,
        )
    }
}

impl fmt::Display for ComparisonPercentileEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the 95th-percentile comparison envelope used by the compact tail report.
pub fn comparison_tail_envelope(
    samples: &[ComparisonSample],
) -> Result<ComparisonPercentileEnvelope, EphemerisError> {
    validate_comparison_samples_for_report(samples)?;

    let envelope = comparison_percentile_envelope(samples, 0.95);
    envelope.validate()?;
    Ok(envelope)
}

/// Combined comparison envelope summary used by the compact report.
///
/// The summary keeps the aggregate comparison record, the median deltas, and
/// the 95th-percentile tail together so downstream tooling can reuse the same
/// validated envelope that the report formatter renders.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonEnvelopeSummary {
    summary: ComparisonSummary,
    median: ComparisonMedianEnvelope,
    percentile: ComparisonPercentileEnvelope,
}

impl ComparisonEnvelopeSummary {
    /// Returns the compact comparison summary line with the median envelope.
    pub fn summary_line(&self) -> String {
        let summary = self
            .summary
            .validated_summary_line()
            .unwrap_or_else(|error| format!("comparison summary unavailable ({error})"));
        format!("{}; {}", summary, self.median)
    }

    /// Returns the compact comparison summary line after validating against samples.
    pub fn validated_summary_line(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<String, EphemerisError> {
        self.validate_against_samples(samples)?;
        Ok(self.summary_line())
    }

    /// Returns the compact 95th-percentile tail line.
    pub fn percentile_line(&self) -> String {
        self.percentile.summary_line()
    }

    /// Returns the compact 95th-percentile tail line after validating against samples.
    pub fn validated_percentile_line(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<String, EphemerisError> {
        self.validate_against_samples(samples)?;
        Ok(self.percentile_line())
    }

    /// Validates the stored envelope against the provided comparison samples.
    pub fn validate_against_samples(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<(), EphemerisError> {
        self.summary.validate()?;

        if self.summary.sample_count != samples.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison envelope summary sample-count mismatch: expected {}, found {}",
                    self.summary.sample_count,
                    samples.len()
                ),
            ));
        }

        if samples.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary has no samples",
            ));
        }

        for (index, sample) in samples.iter().enumerate() {
            for (label, value) in [
                ("longitude_delta_deg", sample.longitude_delta_deg),
                ("latitude_delta_deg", sample.latitude_delta_deg),
            ] {
                if !value.is_finite() || value.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison sample {} field `{label}` must be a finite non-negative value",
                            index + 1
                        ),
                    ));
                }
            }

            if let Some(distance_delta_au) = sample.distance_delta_au {
                if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison sample {} field `distance_delta_au` must be a finite non-negative value",
                            index + 1
                        ),
                    ));
                }
            }
        }

        validate_comparison_sample_distance_channels(samples)?;
        self.median.validate()?;
        self.percentile.validate()?;

        let expected_median = comparison_median_envelope_for_samples(samples);
        if self.median != expected_median {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary median drifted from the sampled comparison values",
            ));
        }

        let expected_percentile = comparison_percentile_envelope(samples, 0.95);
        if self.percentile != expected_percentile {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary percentile drifted from the sampled comparison values",
            ));
        }

        Ok(())
    }
}

impl fmt::Display for ComparisonEnvelopeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the combined comparison envelope summary used by the compact report.
pub fn comparison_envelope_summary(
    summary: &ComparisonSummary,
    samples: &[ComparisonSample],
) -> ComparisonEnvelopeSummary {
    ComparisonEnvelopeSummary {
        summary: summary.clone(),
        median: comparison_median_envelope_for_samples(samples),
        percentile: comparison_percentile_envelope(samples, 0.95),
    }
}

fn median_value(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.total_cmp(right));
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[middle - 1] + values[middle]) / 2.0)
    } else {
        Some(values[middle])
    }
}

fn percentile_value(values: &mut [f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.total_cmp(right));
    let percentile = percentile.clamp(0.0, 1.0);
    let position = percentile * (values.len().saturating_sub(1)) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        Some(values[lower_index])
    } else {
        let weight = position - lower_index as f64;
        Some(values[lower_index] + (values[upper_index] - values[lower_index]) * weight)
    }
}

fn comparison_median_envelope_for_samples(
    samples: &[ComparisonSample],
) -> ComparisonMedianEnvelope {
    let mut longitude_values = samples
        .iter()
        .map(|sample| sample.longitude_delta_deg)
        .collect::<Vec<_>>();
    let mut latitude_values = samples
        .iter()
        .map(|sample| sample.latitude_delta_deg)
        .collect::<Vec<_>>();
    let mut distance_values = samples
        .iter()
        .filter_map(|sample| sample.distance_delta_au)
        .collect::<Vec<_>>();

    ComparisonMedianEnvelope {
        longitude_delta_deg: median_value(&mut longitude_values).unwrap_or_default(),
        latitude_delta_deg: median_value(&mut latitude_values).unwrap_or_default(),
        distance_delta_au: median_value(&mut distance_values),
    }
}

fn comparison_percentile_envelope(
    samples: &[ComparisonSample],
    percentile: f64,
) -> ComparisonPercentileEnvelope {
    let mut longitude_values = samples
        .iter()
        .map(|sample| sample.longitude_delta_deg)
        .collect::<Vec<_>>();
    let mut latitude_values = samples
        .iter()
        .map(|sample| sample.latitude_delta_deg)
        .collect::<Vec<_>>();
    let mut distance_values = samples
        .iter()
        .filter_map(|sample| sample.distance_delta_au)
        .collect::<Vec<_>>();

    ComparisonPercentileEnvelope {
        longitude_delta_deg: percentile_value(&mut longitude_values, percentile)
            .unwrap_or_default(),
        latitude_delta_deg: percentile_value(&mut latitude_values, percentile).unwrap_or_default(),
        distance_delta_au: percentile_value(&mut distance_values, percentile),
    }
}

fn format_comparison_percentile_envelope_for_report(samples: &[ComparisonSample]) -> String {
    match comparison_tail_envelope(samples) {
        Ok(envelope) => envelope.summary_line(),
        Err(error) => format!("comparison percentile envelope unavailable ({error})"),
    }
}

fn format_comparison_envelope_for_report(
    summary: &ComparisonSummary,
    samples: &[ComparisonSample],
) -> String {
    let envelope = comparison_envelope_summary(summary, samples);
    match envelope.validated_summary_line(samples) {
        Ok(rendered) => rendered,
        Err(error) => format!("comparison envelope unavailable ({error})"),
    }
}

fn format_body_class_comparison_envelope_for_report(summary: &BodyClassSummary) -> String {
    summary.summary_line()
}

fn format_body_class_tolerance_envelope_for_report(summary: &BodyClassToleranceSummary) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("body-class tolerance envelope unavailable ({error})"),
    }
}

fn validate_comparison_sample_distance_channels(
    samples: &[ComparisonSample],
) -> Result<(), EphemerisError> {
    let has_distance = samples
        .iter()
        .any(|sample| sample.distance_delta_au.is_some());
    let has_missing_distance = samples
        .iter()
        .any(|sample| sample.distance_delta_au.is_none());

    if has_distance && has_missing_distance {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison sample slice must either provide distance deltas for every sample or for none of them",
        ));
    }

    Ok(())
}

fn validate_comparison_samples_for_report(
    samples: &[ComparisonSample],
) -> Result<(), EphemerisError> {
    if samples.is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison sample slice is empty",
        ));
    }

    for (index, sample) in samples.iter().enumerate() {
        for (label, value) in [
            ("longitude_delta_deg", sample.longitude_delta_deg),
            ("latitude_delta_deg", sample.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison sample {} field `{label}` must be a finite non-negative value",
                        index + 1
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = sample.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison sample {} field `distance_delta_au` must be a finite non-negative value",
                        index + 1
                    ),
                ));
            }
        }
    }

    validate_comparison_sample_distance_channels(samples)
}

fn comparison_tolerance_policy_summary_details(
    comparison: &ComparisonReport,
) -> ComparisonTolerancePolicySummary {
    let entries = comparison_tolerance_policy_entries(&comparison.candidate_backend.family);
    let coverage = comparison_tolerance_policy_coverage(comparison);
    let comparison_window = TimeRange::new(
        comparison.corpus_summary.epochs.first().copied(),
        comparison.corpus_summary.epochs.last().copied(),
    );

    ComparisonTolerancePolicySummary {
        backend_family: comparison.candidate_backend.family.clone(),
        entries,
        coverage,
        comparison_body_count: comparison.body_summaries().len(),
        comparison_sample_count: comparison.summary.sample_count,
        comparison_window,
        coordinate_frames: comparison_coordinate_frames(comparison).to_vec(),
    }
}

fn format_comparison_tolerance_policy_for_report(comparison: &ComparisonReport) -> String {
    let summary = comparison_tolerance_policy_summary_details(comparison);
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("comparison tolerance policy unavailable ({error})"),
    }
}

fn format_comparison_tolerance_limits_for_report(entries: &[ComparisonToleranceEntry]) -> String {
    entries
        .iter()
        .map(format_comparison_tolerance_limit_for_report)
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_comparison_tolerance_limit_for_report(entry: &ComparisonToleranceEntry) -> String {
    match entry.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("{} unavailable ({error})", entry.scope.label()),
    }
}

fn comparison_coordinate_frames(comparison: &ComparisonReport) -> &[CoordinateFrame] {
    &comparison.candidate_backend.supported_frames
}

/// Renders a release-grade comparison tolerance audit used by the CLI.
pub fn render_comparison_audit_report() -> Result<String, String> {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let comparison =
        compare_backends(&reference, &candidate, &corpus).map_err(|error| error.to_string())?;
    let (_, _, _, regression_count) = comparison_audit_totals(&comparison);
    let rendered = render_comparison_audit_report_text(&comparison);

    if regression_count == 0 {
        Ok(rendered)
    } else {
        Err(format!("comparison audit failed:\n{rendered}"))
    }
}

fn comparison_audit_result_label(regression_count: usize) -> &'static str {
    if regression_count == 0 {
        "clean"
    } else {
        "regressions found"
    }
}

fn render_comparison_audit_report_text(report: &ComparisonReport) -> String {
    use std::fmt::Write as _;

    let (body_count, within_tolerance_body_count, outside_tolerance_body_count, regression_count) =
        comparison_audit_totals(report);
    let mut text = String::new();

    let _ = writeln!(text, "Comparison tolerance audit");
    let _ = writeln!(text, "  corpus: {}", report.corpus_name);
    let _ = writeln!(
        text,
        "  reference backend: {} ({})",
        report.reference_backend.id,
        report
            .reference_backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    );
    let _ = writeln!(
        text,
        "  candidate backend: {} ({})",
        report.candidate_backend.id,
        report
            .candidate_backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    );
    let _ = writeln!(text, "  comparison corpus");
    write_corpus_summary_text(&mut text, &report.corpus_summary);
    let _ = writeln!(text, "  bodies checked: {}", body_count);
    let _ = writeln!(
        text,
        "  within tolerance bodies: {}",
        within_tolerance_body_count
    );
    let _ = writeln!(
        text,
        "  outside tolerance bodies: {}",
        outside_tolerance_body_count
    );
    let _ = writeln!(text, "  notable regressions: {}", regression_count);
    let _ = writeln!(
        text,
        "  regression bodies: {}",
        format_regression_bodies(&report.notable_regressions())
    );
    let _ = writeln!(
        text,
        "  result: {}",
        comparison_audit_result_label(regression_count)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(text, "  samples: {}", report.summary.sample_count);
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°{}",
        report.summary.max_longitude_delta_deg,
        format_summary_body(&report.summary.max_longitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°{}",
        report.summary.max_latitude_delta_deg,
        format_summary_body(&report.summary.max_latitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}{}",
        report
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string()),
        format_summary_body(&report.summary.max_distance_delta_body)
    );
    let _ = writeln!(
        text,
        "  {}",
        format_comparison_percentile_envelope_for_report(&report.samples)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class error envelopes");
    for summary in report.body_class_summaries() {
        let _ = writeln!(text, "  {}", summary.class.label());
        let _ = writeln!(text, "    samples: {}", summary.sample_count);
        let _ = writeln!(
            text,
            "    max longitude delta: {:.12}°{}",
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default()
        );
        let _ = writeln!(
            text,
            "    mean longitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                summary.sum_longitude_delta_deg / summary.sample_count as f64
            }
        );
        let _ = writeln!(
            text,
            "    median longitude delta: {:.12}°",
            summary.median_longitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms longitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                (summary.sum_longitude_delta_sq_deg / summary.sample_count as f64).sqrt()
            }
        );
        let _ = writeln!(
            text,
            "    max latitude delta: {:.12}°{}",
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default()
        );
        let _ = writeln!(
            text,
            "    mean latitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                summary.sum_latitude_delta_deg / summary.sample_count as f64
            }
        );
        let _ = writeln!(
            text,
            "    median latitude delta: {:.12}°",
            summary.median_latitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms latitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                (summary.sum_latitude_delta_sq_deg / summary.sample_count as f64).sqrt()
            }
        );
        if let Some(value) = summary.max_distance_delta_au {
            let _ = writeln!(text, "    max distance delta: {:.12} AU", value);
        }
        if summary.distance_count > 0 {
            let mean_distance = summary.sum_distance_delta_au / summary.distance_count as f64;
            let median_distance = summary.median_distance_delta_au.unwrap_or(mean_distance);
            let rms_distance =
                (summary.sum_distance_delta_sq_au / summary.distance_count as f64).sqrt();
            let _ = writeln!(text, "    mean distance delta: {:.12} AU", mean_distance);
            let _ = writeln!(
                text,
                "    median distance delta: {:.12} AU",
                median_distance
            );
            let _ = writeln!(text, "    rms distance delta: {:.12} AU", rms_distance);
        }
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class tolerance posture");
    for summary in report.body_class_tolerance_summaries() {
        let _ = writeln!(text, "  {}", summary.class.label());
        let _ = writeln!(text, "    bodies: {}", summary.body_count);
        let _ = writeln!(text, "    samples: {}", summary.sample_count);
        let _ = writeln!(
            text,
            "    within tolerance bodies: {}",
            summary.within_tolerance_body_count
        );
        let _ = writeln!(
            text,
            "    outside tolerance bodies: {}",
            summary.outside_tolerance_body_count
        );
        if !summary.outside_bodies.is_empty() {
            let _ = writeln!(
                text,
                "    outside bodies: {}",
                format_bodies(&summary.outside_bodies)
            );
        }
        let _ = writeln!(
            text,
            "    mean longitude delta: {:.12}°",
            summary.mean_longitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    median longitude delta: {:.12}°",
            summary.median_longitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms longitude delta: {:.12}°",
            summary.rms_longitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    mean latitude delta: {:.12}°",
            summary.mean_latitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    median latitude delta: {:.12}°",
            summary.median_latitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms latitude delta: {:.12}°",
            summary.rms_latitude_delta_deg()
        );
        if let Some(value) = summary.mean_distance_delta_au() {
            let _ = writeln!(text, "    mean distance delta: {:.12} AU", value);
        }
        if let Some(value) = summary.median_distance_delta_au {
            let _ = writeln!(text, "    median distance delta: {:.12} AU", value);
        }
        if let Some(value) = summary.rms_distance_delta_au() {
            let _ = writeln!(text, "    rms distance delta: {:.12} AU", value);
        }
        if let (Some(body), Some(value)) = (
            summary.max_longitude_delta_body.as_ref(),
            summary.max_longitude_delta_deg,
        ) {
            let _ = writeln!(text, "    max longitude delta: {:.12}° ({})", value, body);
        }
        if let (Some(body), Some(value)) = (
            summary.max_latitude_delta_body.as_ref(),
            summary.max_latitude_delta_deg,
        ) {
            let _ = writeln!(text, "    max latitude delta: {:.12}° ({})", value, body);
        }
        if let (Some(body), Some(value)) = (
            summary.max_distance_delta_body.as_ref(),
            summary.max_distance_delta_au,
        ) {
            let _ = writeln!(text, "    max distance delta: {:.12} AU ({})", value, body);
        }
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Tolerance policy");
    write_tolerance_policy_text(&mut text, report);
    let _ = writeln!(text);
    let _ = writeln!(text, "Notable regressions");
    let regressions = report.notable_regressions();
    if regressions.is_empty() {
        let _ = writeln!(text, "  none");
    } else {
        for finding in regressions {
            let _ = writeln!(
                text,
                "  {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
                finding.body,
                finding.longitude_delta_deg,
                finding.latitude_delta_deg,
                finding
                    .distance_delta_au
                    .map(|value| format!("{value:.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string()),
                finding.note
            );
        }
    }

    text
}

/// Renders a benchmark report used by the CLI.
pub fn render_benchmark_report(rounds: usize) -> Result<String, EphemerisError> {
    let corpus = benchmark_corpus();
    let candidate = default_candidate_backend();
    let backend_report = benchmark_backend(&candidate, &corpus, rounds)?;
    let artifact_report =
        artifact::benchmark_packaged_artifact_decode(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let chart_report = benchmark_chart_backend(default_candidate_backend(), rounds)?;
    Ok(format!(
        "{}\n\n{}\n\n{}\n\n{}",
        benchmark_provenance_text(),
        backend_report,
        artifact_report,
        chart_report
    ))
}

fn vsop87_canonical_body_evidence() -> Option<Vec<pleiades_vsop87::Vsop87CanonicalBodyEvidence>> {
    pleiades_vsop87::canonical_epoch_body_evidence()
}

fn format_vsop87_canonical_evidence_summary() -> String {
    canonical_epoch_evidence_summary_for_report()
}

fn format_vsop87_equatorial_evidence_summary() -> String {
    canonical_epoch_equatorial_evidence_summary_for_report()
}

fn format_vsop87_j2000_batch_summary() -> String {
    canonical_j2000_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j2000_ecliptic_batch_summary() -> String {
    supported_body_j2000_ecliptic_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j2000_equatorial_batch_summary() -> String {
    supported_body_j2000_equatorial_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j1900_ecliptic_batch_summary() -> String {
    supported_body_j1900_ecliptic_batch_parity_summary_for_report()
}

fn format_vsop87_supported_body_j1900_equatorial_batch_summary() -> String {
    supported_body_j1900_equatorial_batch_parity_summary_for_report()
}

fn format_vsop87_mixed_batch_summary() -> String {
    canonical_mixed_time_scale_batch_parity_summary_for_report()
}

fn format_vsop87_j1900_batch_summary() -> String {
    canonical_j1900_batch_parity_summary_for_report()
}

fn format_vsop87_body_evidence_summary() -> String {
    source_body_evidence_summary_for_report()
}

fn format_vsop87_source_body_class_evidence_summary() -> String {
    source_body_class_evidence_summary_for_report()
}

fn format_vsop87_equatorial_body_class_evidence_summary() -> String {
    canonical_epoch_equatorial_body_class_evidence_summary_for_report()
}

fn format_vsop87_canonical_outlier_note_summary() -> String {
    canonical_epoch_outlier_note_for_report()
}

fn format_vsop87_source_documentation_summary() -> String {
    source_documentation_summary_for_report()
}

fn format_vsop87_source_documentation_health_summary() -> String {
    source_documentation_health_summary_for_report()
}

fn format_vsop87_frame_treatment_summary() -> String {
    frame_treatment_summary_for_report()
}

fn format_jpl_frame_treatment_summary() -> String {
    jpl_frame_treatment_summary_for_report()
}

/// Compact validation evidence for the shared mean-obliquity frame round-trip samples.
#[derive(Clone, Debug, PartialEq)]
pub struct MeanObliquityFrameRoundTripSummary {
    sample_count: usize,
    max_longitude_delta_deg: f64,
    max_latitude_delta_deg: f64,
    max_distance_delta_au: f64,
    mean_longitude_delta_deg: f64,
    mean_latitude_delta_deg: f64,
    mean_distance_delta_au: f64,
    percentile_longitude_delta_deg: f64,
    percentile_latitude_delta_deg: f64,
    percentile_distance_delta_au: f64,
}

impl MeanObliquityFrameRoundTripSummary {
    /// Validates the stored round-trip envelope.
    pub fn validate(&self) -> Result<(), String> {
        if self.sample_count == 0 {
            return Err("mean-obliquity frame round-trip summary has no samples".to_string());
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "mean-obliquity frame round-trip summary field `{label}` must be a finite non-negative value"
                ));
            }
        }

        let expected = expected_mean_obliquity_frame_round_trip_summary()?;
        if *self != expected {
            return Err(
                "mean-obliquity frame round-trip summary drifted from the canonical sample set"
                    .to_string(),
            );
        }

        Ok(())
    }

    fn summary_line(&self) -> String {
        format!(
            "{} samples, max |Δlon|={:.12}°, mean |Δlon|={:.12}°, p95 |Δlon|={:.12}°, max |Δlat|={:.12}°, mean |Δlat|={:.12}°, p95 |Δlat|={:.12}°, max |Δdist|={:.12} AU, mean |Δdist|={:.12} AU, p95 |Δdist|={:.12} AU",
            self.sample_count,
            self.max_longitude_delta_deg,
            self.mean_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.mean_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            self.max_distance_delta_au,
            self.mean_distance_delta_au,
            self.percentile_distance_delta_au,
        )
    }

    /// Returns the compact round-trip summary line after validating the canonical sample set.
    pub fn validated_summary_line(&self) -> Result<String, String> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for MeanObliquityFrameRoundTripSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the canonical sample corpus used to validate the shared mean-obliquity frame round-trip envelope.
///
/// Downstream tooling can reuse this exact input set instead of reconstructing it from report text.
/// The corpus intentionally covers a near-polar wraparound case so the report evidence exercises the
/// same precision edge that the frame regression tests pin.
pub fn mean_obliquity_frame_round_trip_sample_corpus() -> [(EclipticCoordinates, Instant); 6] {
    [
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(123.45),
                pleiades_core::Latitude::from_degrees(-6.75),
                Some(0.123),
            ),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(90.0),
                pleiades_core::Latitude::from_degrees(0.0),
                Some(1.0),
            ),
            Instant::new(JulianDay::from_days(2_459_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(27.5),
                pleiades_core::Latitude::from_degrees(-33.25),
                Some(2.5),
            ),
            Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(315.0),
                pleiades_core::Latitude::from_degrees(18.0),
                Some(4.25),
            ),
            Instant::new(JulianDay::from_days(2_440_587.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(5.0),
                pleiades_core::Latitude::from_degrees(66.0),
                Some(0.75),
            ),
            Instant::new(JulianDay::from_days(2_500_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(359.875),
                pleiades_core::Latitude::from_degrees(89.25),
                Some(0.5),
            ),
            Instant::new(JulianDay::from_days(2_450_000.5), TimeScale::Tt),
        ),
    ]
}

fn mean_obliquity_frame_round_trip_summary_from_samples(
    samples: &[(EclipticCoordinates, Instant)],
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    if samples.is_empty() {
        return Err("mean-obliquity frame round-trip summary has no samples".to_string());
    }

    let mut sample_count = 0usize;
    let mut max_longitude_delta_deg: f64 = 0.0;
    let mut max_latitude_delta_deg: f64 = 0.0;
    let mut max_distance_delta_au: f64 = 0.0;
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut distance_deltas = Vec::with_capacity(samples.len());

    for (ecliptic, instant) in samples.iter().copied() {
        let obliquity = instant.mean_obliquity();
        let round_trip = ecliptic.to_equatorial(obliquity).to_ecliptic(obliquity);
        let longitude_delta_deg =
            (round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs();
        let latitude_delta_deg =
            (round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs();
        let distance_delta_au = (round_trip.distance_au.unwrap_or_default()
            - ecliptic.distance_au.unwrap_or_default())
        .abs();

        if !longitude_delta_deg.is_finite()
            || !latitude_delta_deg.is_finite()
            || !distance_delta_au.is_finite()
        {
            return Err("non-finite round-trip delta".to_string());
        }

        max_longitude_delta_deg = max_longitude_delta_deg.max(longitude_delta_deg);
        max_latitude_delta_deg = max_latitude_delta_deg.max(latitude_delta_deg);
        max_distance_delta_au = max_distance_delta_au.max(distance_delta_au);
        longitude_deltas.push(longitude_delta_deg);
        latitude_deltas.push(latitude_delta_deg);
        distance_deltas.push(distance_delta_au);
        sample_count += 1;
    }

    Ok(MeanObliquityFrameRoundTripSummary {
        sample_count,
        max_longitude_delta_deg,
        max_latitude_delta_deg,
        max_distance_delta_au,
        mean_longitude_delta_deg: arithmetic_mean(&longitude_deltas),
        mean_latitude_delta_deg: arithmetic_mean(&latitude_deltas),
        mean_distance_delta_au: arithmetic_mean(&distance_deltas),
        percentile_longitude_delta_deg: percentile_linear_interpolation(&longitude_deltas, 0.95),
        percentile_latitude_delta_deg: percentile_linear_interpolation(&latitude_deltas, 0.95),
        percentile_distance_delta_au: percentile_linear_interpolation(&distance_deltas, 0.95),
    })
}

fn expected_mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )
}

fn arithmetic_mean(values: &[f64]) -> f64 {
    values.iter().copied().sum::<f64>() / values.len() as f64
}

fn percentile_linear_interpolation(values: &[f64], percentile: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    if sorted.len() == 1 {
        return sorted[0];
    }

    let clamped = percentile.clamp(0.0, 1.0);
    let position = clamped * (sorted.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;

    if lower_index == upper_index {
        sorted[lower_index]
    } else {
        let lower_value = sorted[lower_index];
        let upper_value = sorted[upper_index];
        let fraction = position - lower_index as f64;
        lower_value + (upper_value - lower_value) * fraction
    }
}

/// Computes the shared mean-obliquity frame round-trip validation summary.
pub fn mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    let summary = mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )?;
    summary.validate()?;
    Ok(summary)
}

fn format_mean_obliquity_frame_round_trip_summary_for_report(
    summary: &MeanObliquityFrameRoundTripSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}

fn mean_obliquity_frame_round_trip_summary_for_report() -> String {
    match mean_obliquity_frame_round_trip_summary() {
        Ok(summary) => format_mean_obliquity_frame_round_trip_summary_for_report(&summary),
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}

fn format_time_scale_policy_summary_for_report(
    summary: &pleiades_backend::TimeScalePolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("time-scale policy unavailable ({error})"),
    }
}

fn format_delta_t_policy_summary_for_report(
    summary: &pleiades_backend::DeltaTPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("delta T policy unavailable ({error})"),
    }
}

fn format_observer_policy_summary_for_report(
    summary: &pleiades_backend::ObserverPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("observer policy unavailable ({error})"),
    }
}

fn format_apparentness_policy_summary_for_report(
    summary: &pleiades_backend::ApparentnessPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("apparentness policy unavailable ({error})"),
    }
}

fn format_request_policy_summary_for_report(
    summary: &pleiades_backend::RequestPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("request policy unavailable ({error})"),
    }
}

fn validated_request_policy_summary_for_report(
) -> Result<pleiades_backend::RequestPolicySummary, String> {
    let summary = request_policy_summary_for_report();
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

fn format_request_semantics_summary_for_report(
    time_scale_policy: &pleiades_backend::TimeScalePolicySummary,
) -> String {
    use std::fmt::Write as _;

    let mut text = String::new();
    let _ = writeln!(
        text,
        "Time-scale policy: {}",
        format_time_scale_policy_summary_for_report(time_scale_policy)
    );

    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );

    let request_policy = match validated_request_policy_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => {
            let _ = writeln!(text, "Observer policy unavailable ({error})");
            let _ = writeln!(text, "Apparentness policy unavailable ({error})");
            let _ = writeln!(text, "Request policy unavailable ({error})");
            return text;
        }
    };

    let observer_policy = pleiades_backend::observer_policy_summary_for_report();
    let apparentness_policy = pleiades_backend::apparentness_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Observer policy: {}",
        format_observer_policy_summary_for_report(&observer_policy)
    );
    let _ = writeln!(
        text,
        "Apparentness policy: {}",
        format_apparentness_policy_summary_for_report(&apparentness_policy)
    );
    let _ = writeln!(
        text,
        "Request policy: {}",
        format_request_policy_summary_for_report(&request_policy)
    );
    text
}

fn render_time_scale_policy_summary_text() -> String {
    match time_scale_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!(
            "Time-scale policy summary\nTime-scale policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Time-scale policy summary\nTime-scale policy unavailable ({error})\n")
        }
    }
}

fn render_delta_t_policy_summary_text() -> String {
    match delta_t_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!("Delta T policy summary\nDelta T policy: {}\n", summary),
        Err(error) => format!("Delta T policy summary\nDelta T policy unavailable ({error})\n"),
    }
}

fn render_observer_policy_summary_text() -> String {
    match pleiades_backend::observer_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!("Observer policy summary\nObserver policy: {}\n", summary),
        Err(error) => format!("Observer policy summary\nObserver policy unavailable ({error})\n"),
    }
}

fn render_apparentness_policy_summary_text() -> String {
    match pleiades_backend::apparentness_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!(
            "Apparentness policy summary\nApparentness policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Apparentness policy summary\nApparentness policy unavailable ({error})\n")
        }
    }
}

fn render_interpolation_posture_summary_text() -> String {
    match jpl_interpolation_posture_summary() {
        Some(summary) => {
            match summary.validated_summary_line() {
                Ok(summary) => format!(
                    "Interpolation posture summary\nInterpolation posture: {}\n",
                    summary
                ),
                Err(error) => {
                    format!("Interpolation posture summary\nInterpolation posture unavailable ({error})\n")
                }
            }
        }
        None => "Interpolation posture summary\nInterpolation posture unavailable\n".to_string(),
    }
}

fn render_interpolation_quality_summary_text() -> String {
    format!(
        "Interpolation quality summary\n{}\n",
        format_jpl_interpolation_quality_summary_for_report()
    )
}

fn render_comparison_snapshot_summary_text() -> String {
    format!(
        "Comparison snapshot summary\n{}\n",
        comparison_snapshot_summary_for_report()
    )
}

fn comparison_corpus_release_guard_summary() -> &'static str {
    "Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"
}

fn render_comparison_corpus_summary_text() -> String {
    use std::fmt::Write as _;

    let corpus = release_grade_corpus();
    let summary = corpus.summary();
    let mut text = String::from("Comparison corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    let _ = writeln!(
        text,
        "  release-grade guard: {}",
        comparison_corpus_release_guard_summary()
    );
    text.push('\n');
    text
}

fn render_comparison_corpus_release_guard_summary_text() -> String {
    format!(
        "Comparison corpus release-grade guard summary\nRelease-grade guard: {}\n",
        comparison_corpus_release_guard_summary()
    )
}

fn render_benchmark_corpus_summary_text() -> String {
    let corpus = benchmark_corpus();
    let summary = corpus.summary();
    let mut text = String::from("Benchmark corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    text.push('\n');
    text
}

fn render_reference_snapshot_summary_text() -> String {
    format!(
        "Reference snapshot summary\n{}\n",
        reference_snapshot_summary_for_report()
    )
}

fn render_lunar_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar reference error envelope summary\n{}\n",
        lunar_reference_evidence_envelope_for_report()
    )
}

fn render_lunar_equatorial_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar equatorial reference error envelope summary\n{}\n",
        lunar_equatorial_reference_evidence_envelope_for_report()
    )
}

fn render_lunar_apparent_comparison_summary_text() -> String {
    format!(
        "Lunar apparent comparison summary\n{}\n",
        lunar_apparent_comparison_summary_for_report()
    )
}

fn render_frame_policy_summary_text() -> String {
    match frame_policy_summary_details().validated_summary_line() {
        Ok(summary) => format!("Frame policy summary\nFrame policy: {}\n", summary),
        Err(error) => format!("Frame policy summary\nFrame policy unavailable ({error})\n"),
    }
}

fn render_reference_holdout_overlap_summary_text() -> String {
    reference_holdout_overlap_summary_for_report()
}

fn render_request_policy_summary_text() -> String {
    let time_scale_policy = time_scale_policy_summary_for_report();
    let mut text = String::from("Request policy summary\n");
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text
}

fn render_request_surface_summary_text() -> String {
    format!(
        "Request surface summary\n{}\n",
        request_surface_summary_for_report()
    )
}

fn render_comparison_tolerance_policy_summary_text() -> String {
    let report = compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &default_corpus(),
    )
    .expect("comparison tolerance policy summary should build");
    format!(
        "Comparison tolerance policy summary\nComparison tolerance policy: {}\n",
        format_comparison_tolerance_policy_for_report(&report)
    )
}

fn render_comparison_envelope_summary_text() -> String {
    let report = compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &default_corpus(),
    )
    .expect("comparison envelope summary should build");
    let envelope = comparison_envelope_summary(&report.summary, &report.samples);
    let summary_line = envelope
        .validated_summary_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison envelope unavailable ({error})"));
    let percentile_line = envelope
        .validated_percentile_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison percentile envelope unavailable ({error})"));

    format!(
        "Comparison envelope summary\nSummary line: {summary_line}\nPercentile line: {percentile_line}\n"
    )
}

fn render_pluto_fallback_summary_text() -> String {
    let report = compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &default_corpus(),
    )
    .expect("Pluto fallback summary should build");
    let summary = comparison_tolerance_policy_summary_details(&report)
        .entries
        .into_iter()
        .find(|entry| entry.scope == ComparisonToleranceScope::Pluto)
        .expect("Pluto fallback summary should include a Pluto scope entry");
    match summary.validated_summary_line() {
        Ok(line) => format!("Pluto fallback summary\nPluto fallback: {}\n", line),
        Err(error) => format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n"),
    }
}

fn validated_api_stability_profile_for_report() -> Result<pleiades_core::ApiStabilityProfile, String>
{
    let profile = current_api_stability_profile();
    profile.validate().map_err(|error| error.to_string())?;
    Ok(profile)
}

fn validated_compatibility_profile_for_report() -> Result<CompatibilityProfile, String> {
    let profile = current_compatibility_profile();
    profile.validate().map_err(|error| error.to_string())?;
    Ok(profile)
}

fn validated_release_profile_identifiers_for_report() -> Result<ReleaseProfileIdentifiers, String> {
    let release_profiles = current_release_profile_identifiers();
    release_profiles
        .validate()
        .map_err(|error| error.to_string())?;
    Ok(release_profiles)
}

fn format_release_profile_identifiers_summary(
    release_profiles: &ReleaseProfileIdentifiers,
) -> String {
    match release_profiles.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("unavailable ({error})"),
    }
}

/// Renders the compact release-profile identifiers summary.
pub fn render_release_profile_identifiers_summary() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Release profile identifiers summary unavailable ({error})"),
    };

    let mut text = String::new();
    text.push_str("Release profile identifiers summary\n");
    text.push_str("Summary line: ");
    text.push_str(&format_release_profile_identifiers_summary(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("Compatibility profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(release_profiles.api_stability_profile_id);
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Release summary: release-summary\n");

    text
}

fn api_stability_summary_line_for_report() -> String {
    match validated_api_stability_profile_for_report() {
        Ok(profile) => profile.summary_line(),
        Err(error) => format!("API stability summary unavailable ({error})"),
    }
}

/// Compact inventory of the public request surfaces that are called out in the
/// time-observer policy and release-facing validation summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestSurfaceSummary {
    instant: &'static str,
    chart_request: &'static str,
    backend_request: &'static str,
    house_request: &'static str,
    cli_chart: &'static str,
}

impl RequestSurfaceSummary {
    /// Returns the current compact request-surface inventory.
    pub const fn current() -> Self {
        Self {
            instant: "pleiades-types::Instant (tagged instant plus caller-supplied retagging)",
            chart_request: "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)",
            backend_request:
                "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)",
            house_request: "pleiades-houses::HouseRequest (house-only observer calculations)",
            cli_chart: "pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)",
        }
    }

    /// Validates that the cached inventory still matches the documented request surfaces.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        const EXPECTED_INSTANT: &str =
            "pleiades-types::Instant (tagged instant plus caller-supplied retagging)";
        const EXPECTED_CHART_REQUEST: &str =
            "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)";
        const EXPECTED_BACKEND_REQUEST: &str =
            "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)";
        const EXPECTED_HOUSE_REQUEST: &str =
            "pleiades-houses::HouseRequest (house-only observer calculations)";
        const EXPECTED_CLI_CHART: &str = "pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)";

        validate_request_surface_label("instant", self.instant, EXPECTED_INSTANT)?;
        validate_request_surface_label(
            "chart request",
            self.chart_request,
            EXPECTED_CHART_REQUEST,
        )?;
        validate_request_surface_label(
            "backend request",
            self.backend_request,
            EXPECTED_BACKEND_REQUEST,
        )?;
        validate_request_surface_label(
            "house request",
            self.house_request,
            EXPECTED_HOUSE_REQUEST,
        )?;
        validate_request_surface_label("CLI chart", self.cli_chart, EXPECTED_CLI_CHART)?;

        Ok(())
    }

    /// Returns the chart-help clause that spells out the explicit UTC/UT1 and
    /// TT/TDB aliases used by the chart CLI.
    pub const fn chart_help_clause(self) -> &'static str {
        self.cli_chart
    }

    /// Returns the compact `Primary request surfaces:` line.
    pub fn summary_line(self) -> String {
        format!(
            "Primary request surfaces: {}; {}; {}; {}; {}",
            self.instant,
            self.chart_request,
            self.backend_request,
            self.house_request,
            self.cli_chart,
        )
    }

    /// Validates the summary and returns its compact report line.
    pub fn validated_summary_line(self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for RequestSurfaceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn validate_request_surface_label(
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), EphemerisError> {
    if actual == expected {
        return Ok(());
    }

    Err(EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("primary request surface {field} mismatch: expected {expected}, found {actual}"),
    ))
}

/// Returns the current compact request-surface inventory.
pub const fn current_request_surface_summary() -> RequestSurfaceSummary {
    RequestSurfaceSummary::current()
}

fn request_surface_summary_for_report() -> String {
    let summary = RequestSurfaceSummary::current();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("primary request surfaces unavailable ({error})"),
    }
}

fn format_vsop87_request_policy_summary() -> String {
    vsop87_request_policy_summary_for_report()
}

fn format_vsop87_source_audit_summary() -> String {
    source_audit_summary_for_report()
}

fn format_packaged_artifact_profile_summary() -> String {
    packaged_artifact_profile_summary_with_body_coverage()
}

fn format_packaged_artifact_output_support_summary() -> String {
    packaged_artifact_output_support_summary_for_report()
}

fn format_packaged_artifact_storage_summary() -> String {
    packaged_artifact_storage_summary_for_report()
}

fn format_packaged_artifact_access_summary() -> String {
    packaged_artifact_access_summary_for_report()
}

fn format_packaged_frame_parity_summary() -> String {
    packaged_frame_parity_summary_for_report()
}

fn format_lunar_frame_treatment_summary() -> String {
    lunar_theory_frame_treatment_summary_for_report()
}

fn format_packaged_frame_treatment_summary() -> String {
    packaged_frame_treatment_summary_for_report()
}

fn render_validation_report_summary_text(report: &ValidationReport) -> String {
    use std::fmt::Write as _;

    if let Err(error) = report.validate() {
        return format!("Validation report summary unavailable ({error})");
    }

    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Validation report summary unavailable ({error})"),
    };
    let request_policy = request_policy_summary_for_report();
    let comparison_regressions = report.comparison.notable_regressions().len();
    let mut text = String::new();
    let _ = writeln!(text, "Validation report summary");
    let _ = writeln!(
        text,
        "Profile: {}",
        release_profiles.compatibility_profile_id
    );
    let _ = writeln!(
        text,
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    );
    let _ = writeln!(
        text,
        "Release profile identifiers: {}",
        format_release_profile_identifiers_summary(&release_profiles)
    );
    let _ = writeln!(text, "Time-scale policy: {}", request_policy.time_scale);
    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );
    let _ = writeln!(text, "Observer policy: {}", request_policy.observer);
    let _ = writeln!(text, "Apparentness policy: {}", request_policy.apparentness);
    let _ = writeln!(text, "Frame policy: {}", request_policy.frame);
    let _ = writeln!(
        text,
        "Mean-obliquity frame round-trip: {}",
        mean_obliquity_frame_round_trip_summary_for_report()
    );
    let _ = writeln!(
        text,
        "Request policy: {}",
        format_request_policy_summary_for_report(&request_policy)
    );
    let _ = writeln!(text, "{}", request_surface_summary_for_report());
    let _ = writeln!(
        text,
        "Zodiac policy: {}",
        zodiac_policy_summary_for_report(&[ZodiacMode::Tropical])
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison corpus");
    let _ = writeln!(text, "  name: {}", report.comparison_corpus.name);
    let _ = writeln!(
        text,
        "  requests: {}",
        report.comparison_corpus.request_count
    );
    let _ = writeln!(text, "  epochs: {}", report.comparison_corpus.epoch_count);
    let _ = writeln!(
        text,
        "  epoch labels: {}",
        format_instant_list(&report.comparison_corpus.epochs)
    );
    let _ = writeln!(text, "  bodies: {}", report.comparison_corpus.body_count);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.comparison_corpus.apparentness
    );
    let _ = writeln!(text, "  {}", comparison_snapshot_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        comparison_snapshot_body_class_coverage_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        comparison_snapshot_source_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        comparison_snapshot_manifest_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  release-grade guard: {}",
        comparison_corpus_release_guard_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Reference snapshot");
    let _ = writeln!(text, "  {}", reference_snapshot_summary_for_report());
    let _ = writeln!(text, "  {}", reference_snapshot_source_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_source_window_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_body_class_coverage_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "House validation corpus");
    let _ = writeln!(
        text,
        "  {}",
        house_validation_summary_line_for_report(&report.house_validation)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(
        text,
        "  samples: {}",
        report.comparison.summary.sample_count
    );
    let median = comparison_median_envelope_for_samples(&report.comparison.samples);
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°{}",
        report.comparison.summary.max_longitude_delta_deg,
        format_summary_body(&report.comparison.summary.max_longitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°{}",
        report.comparison.summary.max_latitude_delta_deg,
        format_summary_body(&report.comparison.summary.max_latitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}{}",
        report
            .comparison
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string()),
        format_summary_body(&report.comparison.summary.max_distance_delta_body)
    );
    let _ = writeln!(
        text,
        "  mean longitude delta: {:.12}°",
        report.comparison.summary.mean_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  median longitude delta: {:.12}°",
        median.longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  rms longitude delta: {:.12}°",
        report.comparison.summary.rms_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  mean latitude delta: {:.12}°",
        report.comparison.summary.mean_latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  median latitude delta: {:.12}°",
        median.latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  rms latitude delta: {:.12}°",
        report.comparison.summary.rms_latitude_delta_deg
    );
    if let Some(value) = report.comparison.summary.mean_distance_delta_au {
        let _ = writeln!(text, "  mean distance delta: {:.12} AU", value);
    }
    if let Some(value) = median.distance_delta_au {
        let _ = writeln!(text, "  median distance delta: {:.12} AU", value);
    }
    if let Some(value) = report.comparison.summary.rms_distance_delta_au {
        let _ = writeln!(text, "  rms distance delta: {:.12} AU", value);
    }
    let _ = writeln!(
        text,
        "  {}",
        format_comparison_percentile_envelope_for_report(&report.comparison.samples)
    );
    let _ = writeln!(text, "  notable regressions: {}", comparison_regressions);
    let _ = writeln!(
        text,
        "  regression bodies: {}",
        format_regression_bodies(&report.comparison.notable_regressions())
    );
    let _ = writeln!(
        text,
        "Comparison tolerance policy: {}",
        format_comparison_tolerance_policy_for_report(&report.comparison)
    );
    let _ = writeln!(
        text,
        "Comparison audit: {}",
        comparison_audit_summary_for_report(&report.comparison)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "JPL interpolation quality");
    let _ = writeln!(
        text,
        "  {}",
        format_jpl_interpolation_quality_summary_for_report()
    );
    let _ = writeln!(text, "  {}", jpl_independent_holdout_summary_for_report());
    let _ = writeln!(text, "  {}", reference_holdout_overlap_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_mars_jupiter_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_mars_outer_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_window_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        jpl_independent_holdout_snapshot_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "JPL request policy: {}",
        jpl_snapshot_request_policy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "{}",
        jpl_snapshot_batch_error_taxonomy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "JPL frame treatment: {}",
        format_jpl_frame_treatment_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark summaries");
    let _ = writeln!(text, "Reference benchmark");
    let _ = writeln!(text, "  corpus: {}", report.reference_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.reference_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.reference_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.reference_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.reference_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.reference_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.reference_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Candidate benchmark");
    let _ = writeln!(text, "  corpus: {}", report.candidate_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.candidate_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.candidate_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.candidate_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.candidate_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.candidate_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.candidate_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-data benchmark");
    let _ = writeln!(text, "  corpus: {}", report.packaged_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.packaged_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.packaged_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.packaged_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.packaged_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.packaged_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.packaged_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged artifact decode benchmark");
    let _ = writeln!(
        text,
        "  artifact: {}",
        report.artifact_decode_benchmark.artifact_label
    );
    let _ = writeln!(
        text,
        "  source: {}",
        report.artifact_decode_benchmark.source
    );
    let _ = writeln!(
        text,
        "  rounds: {}",
        report.artifact_decode_benchmark.rounds
    );
    let _ = writeln!(
        text,
        "  decodes per round: {}",
        report.artifact_decode_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  encoded bytes: {}",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(
        text,
        "  ns/decode: {}",
        format_ns(report.artifact_decode_benchmark.nanoseconds_per_decode())
    );
    let _ = writeln!(
        text,
        "  decodes per second: {:.2} decodes/s",
        report.artifact_decode_benchmark.decodes_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Chart benchmark");
    let _ = writeln!(text, "  corpus: {}", report.chart_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.chart_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.chart_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.chart_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/chart: {}",
        format_ns(report.chart_benchmark.nanoseconds_per_chart())
    );
    let _ = writeln!(
        text,
        "  charts per second: {:.2} charts/s",
        report.chart_benchmark.charts_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "ELP lunar capability: {}",
        lunar_theory_capability_summary_for_report()
    );
    let _ = writeln!(
        text,
        "ELP lunar request policy: {}",
        lunar_theory_request_policy_summary()
    );
    let _ = writeln!(
        text,
        "ELP frame treatment: {}",
        format_lunar_frame_treatment_summary()
    );
    let _ = writeln!(text, "  {}", lunar_theory_source_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar reference");
    let _ = writeln!(text, "  {}", lunar_reference_evidence_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        lunar_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_reference_evidence_envelope_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar equatorial reference");
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_evidence_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_evidence_envelope_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar apparent comparison");
    let _ = writeln!(text, "  {}", lunar_apparent_comparison_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar source windows");
    let _ = writeln!(text, "  {}", lunar_source_window_summary_for_report());
    let _ = writeln!(text, "Lunar high-curvature continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        lunar_high_curvature_continuity_evidence_for_report()
    );
    let _ = writeln!(text, "Lunar high-curvature equatorial continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        lunar_high_curvature_equatorial_continuity_evidence_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Body comparison summaries");
    for summary in report.comparison.body_summaries() {
        let _ = writeln!(
            text,
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, rms Δdist={}",
            summary.body,
            summary.sample_count,
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_longitude_delta_deg,
            summary.rms_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_latitude_delta_deg,
            summary.rms_latitude_delta_deg,
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary
                .mean_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class error envelopes");
    for summary in report.comparison.body_class_summaries() {
        let max_longitude_body = summary
            .max_longitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_latitude_body = summary
            .max_latitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_distance_body = summary
            .max_distance_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let _ = writeln!(
            text,
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, median Δdist={}, p95 Δdist={}, rms Δdist={}",
            summary.class.label(),
            summary.sample_count,
            summary.max_longitude_delta_deg,
            max_longitude_body,
            summary.mean_longitude_delta_deg(),
            summary.median_longitude_delta_deg,
            summary.percentile_longitude_delta_deg,
            summary.rms_longitude_delta_deg(),
            summary.max_latitude_delta_deg,
            max_latitude_body,
            summary.mean_latitude_delta_deg(),
            summary.median_latitude_delta_deg,
            summary.percentile_latitude_delta_deg,
            summary.rms_latitude_delta_deg(),
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_distance_body,
            summary
                .mean_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .median_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .percentile_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class tolerance posture");
    for summary in report.comparison.body_class_tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}",
            format_body_class_tolerance_envelope_for_report(&summary)
        );
        if !summary.outside_bodies.is_empty() {
            let _ = writeln!(
                text,
                "    outside bodies: {}",
                format_bodies(&summary.outside_bodies)
            );
        }
        let _ = writeln!(
            text,
            "    mean Δlon={:.12}°, median Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={}, median Δdist={}, rms Δdist={}",
            summary.mean_longitude_delta_deg(),
            summary.median_longitude_delta_deg,
            summary.rms_longitude_delta_deg(),
            summary.mean_latitude_delta_deg(),
            summary.median_latitude_delta_deg,
            summary.rms_latitude_delta_deg(),
            summary
                .mean_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .median_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Tolerance policy");
    write_tolerance_policy_text(&mut text, &report.comparison);
    let _ = writeln!(text);
    let _ = writeln!(text, "Expected tolerance status");
    for summary in report.comparison.tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}: profile={}, status={}, limit Δlon≤{:.6}°, margin Δlon={:+.12}°, limit Δlat≤{:.6}°, margin Δlat={:+.12}°, limit Δdist={}, margin Δdist={}, measured max Δlon={:.12}°, max Δlat={:.12}°, max Δdist={}",
            summary.body,
            summary.tolerance.profile,
            if summary.within_tolerance { "within" } else { "exceeded" },
            summary.tolerance.max_longitude_delta_deg,
            summary.longitude_margin_deg,
            summary.tolerance.max_latitude_delta_deg,
            summary.latitude_margin_deg,
            summary
                .tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .distance_margin_au
                .map(|value| format!("{value:+.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary.max_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison tolerance audit");
    let (audit_body_count, audit_within_count, audit_outside_count, audit_regression_count) =
        comparison_audit_totals(&report.comparison);
    let _ = writeln!(text, "  command: compare-backends-audit");
    let _ = writeln!(
        text,
        "  status: {}",
        if audit_regression_count == 0 {
            "clean"
        } else {
            "regressions found"
        }
    );
    let _ = writeln!(text, "  bodies checked: {}", audit_body_count);
    let _ = writeln!(text, "  within tolerance bodies: {}", audit_within_count);
    let _ = writeln!(text, "  outside tolerance bodies: {}", audit_outside_count);
    let _ = writeln!(text, "  notable regressions: {}", audit_regression_count);
    let _ = writeln!(text);
    let house_validation_summary =
        house_validation_summary_line_for_report(&report.house_validation);
    let house_validation_summary = house_validation_summary
        .strip_prefix("House validation corpus: ")
        .unwrap_or(&house_validation_summary);
    let _ = writeln!(
        text,
        "House validation corpus: {}",
        house_validation_summary
    );
    let _ = writeln!(
        text,
        "{}",
        ayanamsa_catalog_validation_summary().summary_line()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "VSOP87 source-backed evidence");
    let _ = writeln!(text, "  {}", format_vsop87_source_documentation_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_source_documentation_health_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_frame_treatment_summary());
    let _ = writeln!(
        text,
        "  VSOP87 request policy: {}",
        format_vsop87_request_policy_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_source_audit_summary());
    let _ = writeln!(text, "  {}", generated_binary_audit_summary_for_report());
    let _ = writeln!(text, "  {}", format_vsop87_canonical_evidence_summary());
    let _ = writeln!(text, "  {}", format_vsop87_canonical_outlier_note_summary());
    let _ = writeln!(text, "  {}", format_vsop87_equatorial_evidence_summary());
    let _ = writeln!(text, "  {}", format_vsop87_j2000_batch_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j2000_ecliptic_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j2000_equatorial_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j1900_ecliptic_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j1900_equatorial_batch_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_mixed_batch_summary());
    let _ = writeln!(text, "  {}", format_vsop87_j1900_batch_summary());
    let _ = writeln!(text, "  {}", format_vsop87_body_evidence_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_source_body_class_evidence_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_equatorial_body_class_evidence_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "ELP lunar theory specification");
    let _ = writeln!(text, "  {}", lunar_theory_catalog_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        lunar_theory_catalog_validation_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_theory_source_summary_for_report());
    let _ = writeln!(text, "  {}", lunar_theory_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-artifact profile");
    let _ = writeln!(text, "  {}", format_packaged_artifact_profile_summary());
    let _ = writeln!(
        text,
        "  Packaged-artifact output support: {}",
        format_packaged_artifact_output_support_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact storage/reconstruction: {}",
        format_packaged_artifact_storage_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact access: {}",
        format_packaged_artifact_access_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation policy: {}",
        packaged_artifact_generation_policy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation residual bodies: {}",
        packaged_artifact_generation_residual_bodies_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target thresholds: {}",
        packaged_artifact_target_threshold_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target-threshold scope envelopes: {}",
        packaged_artifact_target_threshold_scope_envelopes_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation manifest: {}",
        packaged_artifact_generation_manifest_for_report()
    );
    let _ = writeln!(text, "  {}", packaged_request_policy_summary_for_report());
    let _ = writeln!(
        text,
        "  Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged batch parity: {}",
        packaged_mixed_tt_tdb_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged frame parity: {}",
        format_packaged_frame_parity_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged frame treatment: {}",
        format_packaged_frame_treatment_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark summaries");
    let _ = writeln!(text, "Reference benchmark");
    let _ = writeln!(text, "  corpus: {}", report.reference_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.reference_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.reference_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.reference_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.reference_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.reference_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.reference_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Candidate benchmark");
    let _ = writeln!(text, "  corpus: {}", report.candidate_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.candidate_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.candidate_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.candidate_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.candidate_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.candidate_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.candidate_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-data benchmark");
    let _ = writeln!(text, "  corpus: {}", report.packaged_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.packaged_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.packaged_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.packaged_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.packaged_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.packaged_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.packaged_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged artifact decode benchmark");
    let _ = writeln!(
        text,
        "  artifact: {}",
        report.artifact_decode_benchmark.artifact_label
    );
    let _ = writeln!(
        text,
        "  source: {}",
        report.artifact_decode_benchmark.source
    );
    let _ = writeln!(
        text,
        "  rounds: {}",
        report.artifact_decode_benchmark.rounds
    );
    let _ = writeln!(
        text,
        "  decodes per round: {}",
        report.artifact_decode_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  encoded bytes: {}",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(
        text,
        "  ns/decode: {}",
        format_ns(report.artifact_decode_benchmark.nanoseconds_per_decode())
    );
    let _ = writeln!(
        text,
        "  decodes per second: {:.2} decodes/s",
        report.artifact_decode_benchmark.decodes_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Chart benchmark");
    let _ = writeln!(text, "  corpus: {}", report.chart_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.chart_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.chart_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.chart_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/chart: {}",
        format_ns(report.chart_benchmark.nanoseconds_per_chart())
    );
    let _ = writeln!(
        text,
        "  charts per second: {:.2} charts/s",
        report.chart_benchmark.charts_per_second()
    );
    let _ = writeln!(text, "Release bundle verification: verify-release-bundle");
    let _ = writeln!(text, "Workspace audit: workspace-audit / audit");
    let _ = writeln!(
        text,
        "Compatibility profile summary: compatibility-profile-summary"
    );
    let _ = writeln!(text, "Release notes summary: release-notes-summary");
    let _ = writeln!(text, "Release checklist summary: release-checklist-summary");
    let _ = writeln!(text, "Release summary: release-summary");

    text
}

/// Renders a compact summary of the implemented backend capability matrix catalog.
pub fn render_backend_matrix_summary() -> String {
    render_backend_matrix_summary_text()
}

fn render_backend_matrix_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    };
    let profile = current_compatibility_profile();
    let catalog = implemented_backend_catalog();
    let mut family_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut bodies: Vec<String> = Vec::new();
    let mut frames: Vec<String> = Vec::new();
    let mut time_scales: Vec<String> = Vec::new();
    let mut deterministic_count = 0usize;
    let mut offline_count = 0usize;
    let mut batch_count = 0usize;
    let mut native_sidereal_count = 0usize;
    let mut bounded_nominal_range_count = 0usize;
    let mut open_ended_nominal_range_count = 0usize;
    let mut exact_accuracy_count = 0usize;
    let mut high_accuracy_count = 0usize;
    let mut moderate_accuracy_count = 0usize;
    let mut approximate_accuracy_count = 0usize;
    let mut unknown_accuracy_count = 0usize;
    let mut selected_asteroid_count = 0usize;
    let mut data_source_count = 0usize;
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();

    for entry in &catalog {
        *status_counts
            .entry(entry.implementation_status.label().to_string())
            .or_insert(0) += 1;

        *family_counts
            .entry(backend_family_label(&entry.metadata.family))
            .or_insert(0) += 1;
        deterministic_count += usize::from(entry.metadata.deterministic);
        offline_count += usize::from(entry.metadata.offline);
        batch_count += usize::from(entry.metadata.capabilities.batch);
        native_sidereal_count += usize::from(entry.metadata.capabilities.native_sidereal);
        if entry.metadata.nominal_range.start.is_some()
            || entry.metadata.nominal_range.end.is_some()
        {
            bounded_nominal_range_count += 1;
        } else {
            open_ended_nominal_range_count += 1;
        }
        match entry.metadata.accuracy {
            AccuracyClass::Exact => exact_accuracy_count += 1,
            AccuracyClass::High => high_accuracy_count += 1,
            AccuracyClass::Moderate => moderate_accuracy_count += 1,
            AccuracyClass::Approximate => approximate_accuracy_count += 1,
            AccuracyClass::Unknown => unknown_accuracy_count += 1,
            _ => unknown_accuracy_count += 1,
        }
        if selected_asteroid_coverage(&entry.metadata.body_coverage).is_some() {
            selected_asteroid_count += 1;
        }
        if !entry.metadata.provenance.data_sources.is_empty() {
            data_source_count += 1;
        }
        for body in &entry.metadata.body_coverage {
            push_unique(&mut bodies, body.to_string());
        }
        for frame in &entry.metadata.supported_frames {
            push_unique(&mut frames, frame.to_string());
        }
        for scale in &entry.metadata.supported_time_scales {
            push_unique(&mut time_scales, scale.to_string());
        }
    }

    let mut family_entries = family_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    family_entries.sort();

    let mut status_entries = status_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    status_entries.sort();

    let mut text = String::new();
    text.push_str("Backend matrix summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Backends: ");
    text.push_str(&catalog.len().to_string());
    text.push('\n');
    text.push_str("Families: ");
    text.push_str(&family_entries.join(", "));
    text.push('\n');
    text.push_str("Implementation statuses: ");
    text.push_str(&status_entries.join(", "));
    text.push('\n');
    text.push_str("Deterministic backends: ");
    text.push_str(&deterministic_count.to_string());
    text.push('\n');
    text.push_str("Offline backends: ");
    text.push_str(&offline_count.to_string());
    text.push('\n');
    text.push_str("Batch-capable backends: ");
    text.push_str(&batch_count.to_string());
    text.push('\n');
    text.push_str("Native sidereal backends: ");
    text.push_str(&native_sidereal_count.to_string());
    text.push('\n');
    text.push_str("Nominal ranges: bounded: ");
    text.push_str(&bounded_nominal_range_count.to_string());
    text.push_str(", open-ended: ");
    text.push_str(&open_ended_nominal_range_count.to_string());
    text.push('\n');
    text.push_str("Accuracy classes: Exact: ");
    text.push_str(&exact_accuracy_count.to_string());
    text.push_str(", High: ");
    text.push_str(&high_accuracy_count.to_string());
    text.push_str(", Moderate: ");
    text.push_str(&moderate_accuracy_count.to_string());
    text.push_str(", Approximate: ");
    text.push_str(&approximate_accuracy_count.to_string());
    text.push_str(", Unknown: ");
    text.push_str(&unknown_accuracy_count.to_string());
    text.push('\n');
    text.push_str("Backends with selected asteroid coverage: ");
    text.push_str(&selected_asteroid_count.to_string());
    text.push('\n');
    text.push_str(&selected_asteroid_source_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_boundary_summary_for_report());
    text.push('\n');
    text.push_str("House code aliases: ");
    text.push_str(&profile.house_code_aliases_summary_line());
    text.push('\n');
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_early_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1800_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_manifest_summary_for_report());
    text.push('\n');
    if let Ok(report) = build_validation_report(SUMMARY_BENCHMARK_ROUNDS) {
        text.push_str("Comparison audit: compare-backends-audit; ");
        text.push_str(&comparison_audit_summary_for_report(&report.comparison));
        text.push('\n');
    }
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text.push_str(&request_surface_summary_for_report());
    text.push('\n');
    text.push_str("Frame policy: ");
    text.push_str(frame_policy_summary_for_report());
    text.push('\n');
    text.push_str("Mean-obliquity frame round-trip: ");
    text.push_str(&mean_obliquity_frame_round_trip_summary_for_report());
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]));
    text.push('\n');
    text.push_str("Backends with external data sources: ");
    text.push_str(&data_source_count.to_string());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_health_summary());
    text.push('\n');
    text.push_str(&format_vsop87_frame_treatment_summary());
    text.push('\n');
    text.push_str("VSOP87 request policy: ");
    text.push_str(&format_vsop87_request_policy_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_audit_summary());
    text.push('\n');
    text.push_str(&generated_binary_audit_summary_for_report());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_outlier_note_summary());
    text.push('\n');
    text.push_str(&format_vsop87_equatorial_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j2000_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_mixed_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j1900_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_body_evidence_summary());
    text.push('\n');
    text.push_str(&lunar_theory_catalog_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_catalog_validation_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_source_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference\n");
    text.push_str(&lunar_equatorial_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference batch parity\n");
    text.push_str(&lunar_equatorial_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_equatorial_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar source windows: ");
    text.push_str(&lunar_source_window_summary_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature continuity evidence\n");
    text.push_str(&lunar_high_curvature_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature equatorial continuity evidence\n");
    text.push_str(&lunar_high_curvature_equatorial_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Distinct bodies covered: ");
    text.push_str(&bodies.len().to_string());
    text.push_str(" (");
    text.push_str(&bodies.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct coordinate frames: ");
    text.push_str(&frames.len().to_string());
    text.push_str(" (");
    text.push_str(&frames.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct time scales: ");
    text.push_str(&time_scales.len().to_string());
    text.push_str(" (");
    text.push_str(&time_scales.join(", "));
    text.push_str(")\n");
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str("Time-scale policy: ");
    text.push_str(&format_time_scale_policy_summary_for_report(
        &time_scale_policy,
    ));
    text.push('\n');
    text.push_str("Delta T policy: ");
    text.push_str(&format_delta_t_policy_summary_for_report(
        &delta_t_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Observer policy: ");
    text.push_str(&format_observer_policy_summary_for_report(
        &pleiades_backend::observer_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Apparentness policy: ");
    text.push_str(&format_apparentness_policy_summary_for_report(
        &pleiades_backend::apparentness_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]));
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&format_release_profile_identifiers_summary(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

/// Renders a compact summary of the API stability posture.
pub fn render_api_stability_summary() -> String {
    render_api_stability_summary_text()
}

fn render_api_stability_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("API stability summary unavailable ({error})"),
    };

    match validated_api_stability_profile_for_report() {
        Ok(profile) => {
            let mut text = String::new();

            text.push_str("API stability summary\n");
            text.push_str("Profile: ");
            text.push_str(profile.profile_id);
            text.push('\n');
            text.push_str("Summary line: ");
            text.push_str(&profile.summary_line());
            text.push('\n');
            text.push_str("Compatibility profile: ");
            text.push_str(release_profiles.compatibility_profile_id);
            text.push('\n');
            text.push_str("Release profile identifiers: ");
            text.push_str(&format_release_profile_identifiers_summary(
                &release_profiles,
            ));
            text.push('\n');
            text.push_str("Stable surfaces: ");
            text.push_str(&profile.stable_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Experimental surfaces: ");
            text.push_str(&profile.experimental_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Deprecation policy items: ");
            text.push_str(&profile.deprecation_policy.len().to_string());
            text.push('\n');
            text.push_str("Intentional limits: ");
            text.push_str(&profile.intentional_limits.len().to_string());
            text.push('\n');
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("Backend matrix summary: backend-matrix-summary\n");
            text.push_str("Release notes summary: release-notes-summary\n");
            text.push_str("Release checklist summary: release-checklist-summary\n");
            text.push_str("Release bundle verification: verify-release-bundle\n");
            text.push_str("See release-summary for the compact one-screen release overview.\n");

            text
        }
        Err(error) => format!("API stability summary unavailable ({error})"),
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn backend_family_label(family: &BackendFamily) -> String {
    family.to_string()
}

/// Renders a backend capability matrix for the implemented backend catalog.
pub fn render_backend_matrix_report() -> Result<String, EphemerisError> {
    let profile = current_compatibility_profile();
    let mut rendered = String::new();
    fmt::write(
        &mut rendered,
        format_args!("Implemented backend matrices\n\n"),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    fmt::write(
        &mut rendered,
        format_args!(
            "House code aliases: {}\n\n",
            profile.house_code_aliases_summary_line()
        ),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    for entry in implemented_backend_catalog() {
        validate_backend_matrix_entry(&entry)?;
        fmt::write(&mut rendered, format_args!("{}\n", entry.label)).map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
        fmt::write(
            &mut rendered,
            format_args!("{}\n\n", BackendMatrixDisplay(&entry)),
        )
        .map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
    }

    Ok(rendered)
}

fn validate_backend_matrix_entry(entry: &BackendMatrixEntry) -> Result<(), EphemerisError> {
    entry.metadata.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "backend matrix entry `{}` has invalid metadata: {error}",
                entry.label
            ),
        )
    })
}

struct BackendMatrixDisplay<'a>(&'a BackendMatrixEntry);

impl fmt::Display for BackendMatrixDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_backend_catalog_entry(f, self.0)
    }
}

impl fmt::Display for ComparisonReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Comparison report")?;
        writeln!(f, "Comparison corpus")?;
        write_corpus_summary(f, &self.corpus_summary)?;
        writeln!(
            f,
            "  release-grade guard: {}",
            comparison_corpus_release_guard_summary()
        )?;
        writeln!(f)?;
        writeln!(f, "Reference backend: {}", self.reference_backend.id)?;
        writeln!(f, "Candidate backend: {}", self.candidate_backend.id)?;
        writeln!(f)?;
        write_comparison_summary(f, self)?;
        writeln!(f)?;
        write_body_comparison_summaries(f, &self.body_summaries())?;
        writeln!(f)?;
        write_body_class_envelopes(f, &self.samples)?;
        writeln!(f)?;
        write_body_class_tolerance_posture(f, &self.samples, &self.candidate_backend.family)?;
        writeln!(f)?;
        write_tolerance_summaries(f, &self.tolerance_summaries())?;
        writeln!(f)?;
        write_regression_section(f, "Notable regressions", &self.notable_regressions())?;
        writeln!(f)?;
        writeln!(f, "Samples")?;
        for sample in &self.samples {
            writeln!(
                f,
                "  {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}",
                sample.body,
                sample.longitude_delta_deg,
                sample.latitude_delta_deg,
                sample
                    .distance_delta_au
                    .map(|value| format!("{value:.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string())
            )?;
        }
        Ok(())
    }
}

impl fmt::Display for BenchmarkReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Benchmark report")?;
        writeln!(f, "Summary: {}", self.summary_line())?;
        writeln!(f, "Backend: {}", self.backend.id)?;
        writeln!(f, "Corpus: {}", self.corpus_name)?;
        writeln!(f, "Apparentness: {}", self.apparentness)?;
        writeln!(f, "Rounds: {}", self.rounds)?;
        writeln!(f, "Samples per round: {}", self.sample_count)?;
        writeln!(f, "Methodology: {}", self.methodology_summary())?;
        writeln!(
            f,
            "Single-request elapsed: {}",
            format_duration(self.elapsed)
        )?;
        writeln!(f, "Batch elapsed: {}", format_duration(self.batch_elapsed))?;
        writeln!(
            f,
            "Estimated corpus heap footprint: {} bytes",
            self.estimated_corpus_heap_bytes
        )?;
        writeln!(
            f,
            "Nanoseconds per request (single): {}",
            format_ns(self.nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "Nanoseconds per request (batch): {}",
            format_ns(self.batch_nanoseconds_per_request())
        )?;
        writeln!(
            f,
            "Batch throughput: {:.2} req/s",
            self.batch_requests_per_second()
        )
    }
}

impl ComparisonReport {
    /// Returns the structured comparison-audit summary used by compact report surfaces.
    pub fn comparison_audit_summary(&self) -> ComparisonAuditSummary {
        comparison_audit_summary(self)
    }

    /// Returns per-body comparison statistics preserving first-seen body order.
    pub fn body_summaries(&self) -> Vec<BodyComparisonSummary> {
        body_comparison_summaries(&self.samples)
    }

    /// Returns per-body-class comparison envelopes preserving first-seen class order.
    pub(crate) fn body_class_summaries(&self) -> Vec<BodyClassSummary> {
        body_class_summaries(&self.samples)
    }

    /// Returns per-body tolerance status preserving first-seen body order.
    pub fn tolerance_summaries(&self) -> Vec<BodyToleranceSummary> {
        let candidate_family = &self.candidate_backend.family;
        self.body_summaries()
            .into_iter()
            .map(|summary| body_tolerance_summary(summary, candidate_family))
            .collect()
    }

    /// Returns the expected tolerance catalog for the candidate backend family.
    pub fn tolerance_policy_entries(&self) -> Vec<ComparisonToleranceEntry> {
        comparison_tolerance_policy_entries(&self.candidate_backend.family)
    }

    /// Returns the structured tolerance policy summary for the candidate backend family.
    pub fn tolerance_policy_summary(&self) -> ComparisonTolerancePolicySummary {
        comparison_tolerance_policy_summary_details(self)
    }

    /// Returns per-body-class tolerance posture preserving first-seen class order.
    pub(crate) fn body_class_tolerance_summaries(&self) -> Vec<BodyClassToleranceSummary> {
        body_class_tolerance_summaries(&self.samples, &self.candidate_backend.family)
    }

    /// Returns the samples that exceed the built-in regression thresholds.
    pub fn notable_regressions(&self) -> Vec<RegressionFinding> {
        self.samples
            .iter()
            .filter_map(|sample| regression_finding(sample, &self.candidate_backend.family))
            .collect()
    }

    /// Returns a preserved archive of the current regression findings.
    pub fn regression_archive(&self) -> RegressionArchive {
        RegressionArchive {
            corpus_name: self.corpus_name.clone(),
            cases: self.notable_regressions(),
        }
    }
}

fn extract_ecliptic(result: EphemerisResult) -> Result<EclipticCoordinates, EphemerisError> {
    result.ecliptic.ok_or_else(|| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "backend did not return ecliptic coordinates for the validation corpus",
        )
    })
}

fn angular_delta(reference: Longitude, candidate: Longitude) -> f64 {
    (pleiades_core::Angle::from_degrees(reference.degrees() - candidate.degrees())
        .normalized_signed()
        .degrees())
    .abs()
}

#[derive(Clone, Debug)]
struct BodyComparisonAccumulator {
    body: CelestialBody,
    sample_count: usize,
    longitude_sum_deg: f64,
    longitude_sum_sq_deg: f64,
    latitude_sum_deg: f64,
    latitude_sum_sq_deg: f64,
    distance_sum_au: f64,
    distance_sum_sq_au: f64,
    distance_count: usize,
    max_longitude_delta_deg: f64,
    max_longitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: f64,
    max_latitude_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    max_distance_delta_body: Option<CelestialBody>,
}

impl BodyComparisonAccumulator {
    fn new(body: CelestialBody) -> Self {
        Self {
            body,
            sample_count: 0,
            longitude_sum_deg: 0.0,
            longitude_sum_sq_deg: 0.0,
            latitude_sum_deg: 0.0,
            latitude_sum_sq_deg: 0.0,
            distance_sum_au: 0.0,
            distance_sum_sq_au: 0.0,
            distance_count: 0,
            max_longitude_delta_deg: 0.0,
            max_longitude_delta_body: None,
            max_latitude_delta_deg: 0.0,
            max_latitude_delta_body: None,
            max_distance_delta_au: None,
            max_distance_delta_body: None,
        }
    }

    fn push(&mut self, sample: &ComparisonSample) {
        self.sample_count += 1;
        self.longitude_sum_deg += sample.longitude_delta_deg;
        self.longitude_sum_sq_deg += sample.longitude_delta_deg * sample.longitude_delta_deg;
        if sample.longitude_delta_deg >= self.max_longitude_delta_deg {
            self.max_longitude_delta_deg = sample.longitude_delta_deg;
            self.max_longitude_delta_body = Some(sample.body.clone());
        }
        self.latitude_sum_deg += sample.latitude_delta_deg;
        self.latitude_sum_sq_deg += sample.latitude_delta_deg * sample.latitude_delta_deg;
        if sample.latitude_delta_deg >= self.max_latitude_delta_deg {
            self.max_latitude_delta_deg = sample.latitude_delta_deg;
            self.max_latitude_delta_body = Some(sample.body.clone());
        }
        if let Some(delta) = sample.distance_delta_au {
            self.distance_sum_au += delta;
            self.distance_sum_sq_au += delta * delta;
            self.distance_count += 1;
            match self.max_distance_delta_au {
                Some(current) if delta < current => {}
                _ => {
                    self.max_distance_delta_au = Some(delta);
                    self.max_distance_delta_body = Some(sample.body.clone());
                }
            }
        }
    }

    fn finish(self) -> BodyComparisonSummary {
        let sample_count = self.sample_count as f64;
        BodyComparisonSummary {
            body: self.body,
            sample_count: self.sample_count,
            max_longitude_delta_body: self.max_longitude_delta_body,
            max_longitude_delta_deg: self.max_longitude_delta_deg,
            mean_longitude_delta_deg: if self.sample_count > 0 {
                self.longitude_sum_deg / sample_count
            } else {
                0.0
            },
            rms_longitude_delta_deg: if self.sample_count > 0 {
                (self.longitude_sum_sq_deg / sample_count).sqrt()
            } else {
                0.0
            },
            max_latitude_delta_body: self.max_latitude_delta_body,
            max_latitude_delta_deg: self.max_latitude_delta_deg,
            mean_latitude_delta_deg: if self.sample_count > 0 {
                self.latitude_sum_deg / sample_count
            } else {
                0.0
            },
            rms_latitude_delta_deg: if self.sample_count > 0 {
                (self.latitude_sum_sq_deg / sample_count).sqrt()
            } else {
                0.0
            },
            max_distance_delta_body: self.max_distance_delta_body,
            max_distance_delta_au: self.max_distance_delta_au,
            mean_distance_delta_au: if self.distance_count > 0 {
                Some(self.distance_sum_au / self.distance_count as f64)
            } else {
                None
            },
            rms_distance_delta_au: if self.distance_count > 0 {
                Some((self.distance_sum_sq_au / self.distance_count as f64).sqrt())
            } else {
                None
            },
        }
    }
}

fn body_comparison_summaries(samples: &[ComparisonSample]) -> Vec<BodyComparisonSummary> {
    let mut accumulators: Vec<BodyComparisonAccumulator> = Vec::new();

    for sample in samples {
        if let Some(accumulator) = accumulators
            .iter_mut()
            .find(|accumulator| accumulator.body == sample.body)
        {
            accumulator.push(sample);
        } else {
            let mut accumulator = BodyComparisonAccumulator::new(sample.body.clone());
            accumulator.push(sample);
            accumulators.push(accumulator);
        }
    }

    accumulators
        .into_iter()
        .map(BodyComparisonAccumulator::finish)
        .collect()
}

#[derive(Clone, Debug)]
struct BodyClassAccumulator {
    class: BodyClass,
    sample_count: usize,
    longitude_sum_deg: f64,
    longitude_sum_sq_deg: f64,
    longitude_values: Vec<f64>,
    latitude_sum_deg: f64,
    latitude_sum_sq_deg: f64,
    latitude_values: Vec<f64>,
    distance_sum_au: f64,
    distance_sum_sq_au: f64,
    distance_values: Vec<f64>,
    distance_count: usize,
    max_longitude_delta_deg: f64,
    max_longitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: f64,
    max_latitude_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    max_distance_delta_body: Option<CelestialBody>,
}

impl BodyClassAccumulator {
    fn new(class: BodyClass) -> Self {
        Self {
            class,
            sample_count: 0,
            longitude_sum_deg: 0.0,
            longitude_sum_sq_deg: 0.0,
            longitude_values: Vec::new(),
            latitude_sum_deg: 0.0,
            latitude_sum_sq_deg: 0.0,
            latitude_values: Vec::new(),
            distance_sum_au: 0.0,
            distance_sum_sq_au: 0.0,
            distance_values: Vec::new(),
            distance_count: 0,
            max_longitude_delta_deg: 0.0,
            max_longitude_delta_body: None,
            max_latitude_delta_deg: 0.0,
            max_latitude_delta_body: None,
            max_distance_delta_au: None,
            max_distance_delta_body: None,
        }
    }

    fn push(&mut self, sample: &ComparisonSample) {
        self.sample_count += 1;
        self.longitude_sum_deg += sample.longitude_delta_deg;
        self.longitude_sum_sq_deg += sample.longitude_delta_deg * sample.longitude_delta_deg;
        self.longitude_values.push(sample.longitude_delta_deg);
        if sample.longitude_delta_deg >= self.max_longitude_delta_deg {
            self.max_longitude_delta_deg = sample.longitude_delta_deg;
            self.max_longitude_delta_body = Some(sample.body.clone());
        }
        self.latitude_sum_deg += sample.latitude_delta_deg;
        self.latitude_sum_sq_deg += sample.latitude_delta_deg * sample.latitude_delta_deg;
        self.latitude_values.push(sample.latitude_delta_deg);
        if sample.latitude_delta_deg >= self.max_latitude_delta_deg {
            self.max_latitude_delta_deg = sample.latitude_delta_deg;
            self.max_latitude_delta_body = Some(sample.body.clone());
        }
        if let Some(delta) = sample.distance_delta_au {
            self.distance_sum_au += delta;
            self.distance_sum_sq_au += delta * delta;
            self.distance_values.push(delta);
            self.distance_count += 1;
            match self.max_distance_delta_au {
                Some(current) if delta < current => {}
                _ => {
                    self.max_distance_delta_au = Some(delta);
                    self.max_distance_delta_body = Some(sample.body.clone());
                }
            }
        }
    }

    fn finish(self) -> BodyClassSummary {
        let mut longitude_values = self.longitude_values;
        let mut latitude_values = self.latitude_values;
        let mut distance_values = self.distance_values;
        BodyClassSummary {
            class: self.class,
            sample_count: self.sample_count,
            max_longitude_delta_body: self.max_longitude_delta_body,
            max_longitude_delta_deg: self.max_longitude_delta_deg,
            sum_longitude_delta_deg: self.longitude_sum_deg,
            sum_longitude_delta_sq_deg: self.longitude_sum_sq_deg,
            median_longitude_delta_deg: median_value(&mut longitude_values).unwrap_or_default(),
            percentile_longitude_delta_deg: percentile_value(&mut longitude_values, 0.95)
                .unwrap_or_default(),
            max_latitude_delta_body: self.max_latitude_delta_body,
            max_latitude_delta_deg: self.max_latitude_delta_deg,
            sum_latitude_delta_deg: self.latitude_sum_deg,
            sum_latitude_delta_sq_deg: self.latitude_sum_sq_deg,
            median_latitude_delta_deg: median_value(&mut latitude_values).unwrap_or_default(),
            percentile_latitude_delta_deg: percentile_value(&mut latitude_values, 0.95)
                .unwrap_or_default(),
            max_distance_delta_body: self.max_distance_delta_body,
            max_distance_delta_au: self.max_distance_delta_au,
            sum_distance_delta_au: self.distance_sum_au,
            sum_distance_delta_sq_au: self.distance_sum_sq_au,
            distance_count: self.distance_count,
            median_distance_delta_au: median_value(&mut distance_values),
            percentile_distance_delta_au: percentile_value(&mut distance_values, 0.95),
        }
    }
}

fn body_class_summaries(samples: &[ComparisonSample]) -> Vec<BodyClassSummary> {
    let mut accumulators = BodyClass::ALL.map(BodyClassAccumulator::new);

    for sample in samples {
        accumulators[body_class(&sample.body).index()].push(sample);
    }

    accumulators
        .into_iter()
        .filter(|summary| summary.sample_count > 0)
        .map(BodyClassAccumulator::finish)
        .collect()
}

#[derive(Clone, Debug)]
struct BodyClassToleranceAccumulator {
    class: BodyClass,
    backend_family: BackendFamily,
    body_count: usize,
    sample_count: usize,
    sum_longitude_delta_deg: f64,
    sum_longitude_delta_sq_deg: f64,
    longitude_values: Vec<f64>,
    sum_latitude_delta_deg: f64,
    sum_latitude_delta_sq_deg: f64,
    latitude_values: Vec<f64>,
    sum_distance_delta_au: f64,
    sum_distance_delta_sq_au: f64,
    distance_values: Vec<f64>,
    distance_count: usize,
    within_tolerance_body_count: usize,
    outside_tolerance_body_count: usize,
    outside_tolerance_sample_count: usize,
    max_longitude_delta_body: Option<CelestialBody>,
    max_longitude_delta_deg: Option<f64>,
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: Option<f64>,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    outside_bodies: Vec<CelestialBody>,
}

impl BodyClassToleranceAccumulator {
    fn new(class: BodyClass, backend_family: BackendFamily) -> Self {
        Self {
            class,
            backend_family,
            body_count: 0,
            sample_count: 0,
            sum_longitude_delta_deg: 0.0,
            sum_longitude_delta_sq_deg: 0.0,
            longitude_values: Vec::new(),
            sum_latitude_delta_deg: 0.0,
            sum_latitude_delta_sq_deg: 0.0,
            latitude_values: Vec::new(),
            sum_distance_delta_au: 0.0,
            sum_distance_delta_sq_au: 0.0,
            distance_values: Vec::new(),
            distance_count: 0,
            within_tolerance_body_count: 0,
            outside_tolerance_body_count: 0,
            outside_tolerance_sample_count: 0,
            max_longitude_delta_body: None,
            max_longitude_delta_deg: None,
            max_latitude_delta_body: None,
            max_latitude_delta_deg: None,
            max_distance_delta_body: None,
            max_distance_delta_au: None,
            outside_bodies: Vec::new(),
        }
    }

    fn push_sample(&mut self, sample: &ComparisonSample) {
        self.sample_count += 1;
        self.sum_longitude_delta_deg += sample.longitude_delta_deg;
        self.sum_longitude_delta_sq_deg += sample.longitude_delta_deg * sample.longitude_delta_deg;
        self.longitude_values.push(sample.longitude_delta_deg);
        match self.max_longitude_delta_deg {
            Some(current) if sample.longitude_delta_deg < current => {}
            _ => {
                self.max_longitude_delta_deg = Some(sample.longitude_delta_deg);
                self.max_longitude_delta_body = Some(sample.body.clone());
            }
        }
        self.sum_latitude_delta_deg += sample.latitude_delta_deg;
        self.sum_latitude_delta_sq_deg += sample.latitude_delta_deg * sample.latitude_delta_deg;
        self.latitude_values.push(sample.latitude_delta_deg);
        match self.max_latitude_delta_deg {
            Some(current) if sample.latitude_delta_deg < current => {}
            _ => {
                self.max_latitude_delta_deg = Some(sample.latitude_delta_deg);
                self.max_latitude_delta_body = Some(sample.body.clone());
            }
        }
        if let Some(delta) = sample.distance_delta_au {
            self.sum_distance_delta_au += delta;
            self.sum_distance_delta_sq_au += delta * delta;
            self.distance_values.push(delta);
            self.distance_count += 1;
            match self.max_distance_delta_au {
                Some(current) if delta < current => {}
                _ => {
                    self.max_distance_delta_au = Some(delta);
                    self.max_distance_delta_body = Some(sample.body.clone());
                }
            }
        }

        let tolerance = comparison_tolerance_for_class(self.class, &self.backend_family);
        let distance_within = match (sample.distance_delta_au, tolerance.max_distance_delta_au) {
            (Some(measured), Some(limit)) => measured <= limit,
            (None, _) | (_, None) => true,
        };
        if sample.longitude_delta_deg > tolerance.max_longitude_delta_deg
            || sample.latitude_delta_deg > tolerance.max_latitude_delta_deg
            || !distance_within
        {
            self.outside_tolerance_sample_count += 1;
        }
    }

    fn push_tolerance(&mut self, summary: &BodyToleranceSummary) {
        self.body_count += 1;
        if summary.within_tolerance {
            self.within_tolerance_body_count += 1;
        } else {
            self.outside_tolerance_body_count += 1;
            self.outside_bodies.push(summary.body.clone());
        }
    }

    fn finish(self) -> BodyClassToleranceSummary {
        let mut longitude_values = self.longitude_values;
        let mut latitude_values = self.latitude_values;
        let mut distance_values = self.distance_values;
        BodyClassToleranceSummary {
            class: self.class,
            tolerance: comparison_tolerance_for_class(self.class, &self.backend_family),
            body_count: self.body_count,
            sample_count: self.sample_count,
            within_tolerance_body_count: self.within_tolerance_body_count,
            outside_tolerance_body_count: self.outside_tolerance_body_count,
            outside_tolerance_sample_count: self.outside_tolerance_sample_count,
            max_longitude_delta_body: self.max_longitude_delta_body,
            max_longitude_delta_deg: self.max_longitude_delta_deg,
            max_latitude_delta_body: self.max_latitude_delta_body,
            max_latitude_delta_deg: self.max_latitude_delta_deg,
            max_distance_delta_body: self.max_distance_delta_body,
            max_distance_delta_au: self.max_distance_delta_au,
            sum_longitude_delta_deg: self.sum_longitude_delta_deg,
            sum_longitude_delta_sq_deg: self.sum_longitude_delta_sq_deg,
            sum_latitude_delta_deg: self.sum_latitude_delta_deg,
            sum_latitude_delta_sq_deg: self.sum_latitude_delta_sq_deg,
            sum_distance_delta_au: self.sum_distance_delta_au,
            sum_distance_delta_sq_au: self.sum_distance_delta_sq_au,
            distance_count: self.distance_count,
            median_longitude_delta_deg: median_value(&mut longitude_values).unwrap_or_default(),
            percentile_longitude_delta_deg: percentile_value(&mut longitude_values, 0.95)
                .unwrap_or_default(),
            median_latitude_delta_deg: median_value(&mut latitude_values).unwrap_or_default(),
            percentile_latitude_delta_deg: percentile_value(&mut latitude_values, 0.95)
                .unwrap_or_default(),
            median_distance_delta_au: median_value(&mut distance_values),
            percentile_distance_delta_au: percentile_value(&mut distance_values, 0.95),
            outside_bodies: self.outside_bodies,
        }
    }
}

fn body_class_tolerance_summaries(
    samples: &[ComparisonSample],
    backend_family: &BackendFamily,
) -> Vec<BodyClassToleranceSummary> {
    let body_summaries = body_comparison_summaries(samples);
    let mut accumulators = BodyClass::ALL
        .map(|class| BodyClassToleranceAccumulator::new(class, backend_family.clone()));

    for sample in samples {
        accumulators[body_class(&sample.body).index()].push_sample(sample);
    }

    for summary in body_summaries {
        let tolerance_summary = body_tolerance_summary(summary, backend_family);
        accumulators[body_class(&tolerance_summary.body).index()]
            .push_tolerance(&tolerance_summary);
    }

    accumulators
        .into_iter()
        .filter(|summary| summary.body_count > 0)
        .map(BodyClassToleranceAccumulator::finish)
        .collect()
}

/// Returns the expected comparison-tolerance catalog for a backend family.
///
/// The catalog is grouped by body-class scope and keeps the backend family
/// explicit in each tolerance row so validation and release tooling can reuse
/// the same stored thresholds without rebuilding them ad hoc.
pub fn comparison_tolerance_catalog_entries(
    backend_family: &BackendFamily,
) -> Vec<ComparisonToleranceEntry> {
    [
        ComparisonToleranceScope::Luminary,
        ComparisonToleranceScope::MajorPlanet,
        ComparisonToleranceScope::LunarPoint,
        ComparisonToleranceScope::Asteroid,
        ComparisonToleranceScope::Custom,
        ComparisonToleranceScope::Pluto,
    ]
    .into_iter()
    .map(|scope| ComparisonToleranceEntry {
        scope,
        tolerance: comparison_tolerance_for_scope(scope, backend_family),
    })
    .collect()
}

fn comparison_tolerance_policy_entries(
    backend_family: &BackendFamily,
) -> Vec<ComparisonToleranceEntry> {
    comparison_tolerance_catalog_entries(backend_family)
}

fn comparison_tolerance_for_scope(
    scope: ComparisonToleranceScope,
    backend_family: &BackendFamily,
) -> ComparisonTolerance {
    match scope {
        ComparisonToleranceScope::Luminary => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 full-file VSOP87B planetary evidence",
            max_longitude_delta_deg: LUMINARY_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: LUMINARY_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(LUMINARY_DISTANCE_THRESHOLD_AU),
        },
        ComparisonToleranceScope::MajorPlanet => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 full-file VSOP87B planetary evidence",
            max_longitude_delta_deg: MAJOR_PLANET_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: MAJOR_PLANET_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(MAJOR_PLANET_DISTANCE_THRESHOLD_AU),
        },
        ComparisonToleranceScope::LunarPoint => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 compact-ELP lunar evidence",
            max_longitude_delta_deg: LUNAR_POINT_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: LUNAR_POINT_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(LUNAR_POINT_DISTANCE_THRESHOLD_AU),
        },
        ComparisonToleranceScope::Asteroid => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 asteroid comparison evidence",
            max_longitude_delta_deg: ASTEROID_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: ASTEROID_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(ASTEROID_DISTANCE_THRESHOLD_AU),
        },
        ComparisonToleranceScope::Custom => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 uncategorized comparison evidence",
            max_longitude_delta_deg: CUSTOM_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: CUSTOM_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(CUSTOM_DISTANCE_THRESHOLD_AU),
        },
        ComparisonToleranceScope::Pluto => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 Pluto approximate fallback evidence",
            max_longitude_delta_deg: PLUTO_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: PLUTO_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(PLUTO_DISTANCE_THRESHOLD_AU),
        },
    }
}

fn comparison_tolerance_scope_for_class(class: BodyClass) -> ComparisonToleranceScope {
    match class {
        BodyClass::Luminary => ComparisonToleranceScope::Luminary,
        BodyClass::MajorPlanet => ComparisonToleranceScope::MajorPlanet,
        BodyClass::LunarPoint => ComparisonToleranceScope::LunarPoint,
        BodyClass::Asteroid => ComparisonToleranceScope::Asteroid,
        BodyClass::Custom => ComparisonToleranceScope::Custom,
    }
}

fn comparison_tolerance_scope_for_body(body: &CelestialBody) -> ComparisonToleranceScope {
    if matches!(body, CelestialBody::Pluto) {
        return ComparisonToleranceScope::Pluto;
    }

    comparison_tolerance_scope_for_class(body_class(body))
}

fn comparison_tolerance_for_class(
    class: BodyClass,
    backend_family: &BackendFamily,
) -> ComparisonTolerance {
    comparison_tolerance_for_scope(comparison_tolerance_scope_for_class(class), backend_family)
}

fn comparison_tolerance_for_body(
    body: &CelestialBody,
    backend_family: &BackendFamily,
) -> ComparisonTolerance {
    comparison_tolerance_for_scope(comparison_tolerance_scope_for_body(body), backend_family)
}

fn body_tolerance_summary(
    summary: BodyComparisonSummary,
    backend_family: &BackendFamily,
) -> BodyToleranceSummary {
    let tolerance = comparison_tolerance_for_body(&summary.body, backend_family);
    let distance_within = match (
        summary.max_distance_delta_au,
        tolerance.max_distance_delta_au,
    ) {
        (Some(measured), Some(limit)) => measured <= limit,
        (None, _) | (_, None) => true,
    };
    let within_tolerance = summary.max_longitude_delta_deg <= tolerance.max_longitude_delta_deg
        && summary.max_latitude_delta_deg <= tolerance.max_latitude_delta_deg
        && distance_within;
    let longitude_margin_deg = tolerance.max_longitude_delta_deg - summary.max_longitude_delta_deg;
    let latitude_margin_deg = tolerance.max_latitude_delta_deg - summary.max_latitude_delta_deg;
    let distance_margin_au = match (
        summary.max_distance_delta_au,
        tolerance.max_distance_delta_au,
    ) {
        (Some(measured), Some(limit)) => Some(limit - measured),
        _ => None,
    };

    BodyToleranceSummary {
        body: summary.body,
        tolerance,
        sample_count: summary.sample_count,
        within_tolerance,
        max_longitude_delta_deg: summary.max_longitude_delta_deg,
        longitude_margin_deg,
        max_latitude_delta_deg: summary.max_latitude_delta_deg,
        latitude_margin_deg,
        max_distance_delta_au: summary.max_distance_delta_au,
        distance_margin_au,
    }
}

fn write_corpus_summary(f: &mut fmt::Formatter<'_>, corpus: &CorpusSummary) -> fmt::Result {
    if let Err(error) = corpus.validate() {
        writeln!(f, "  corpus summary unavailable ({error})")?;
        return Ok(());
    }

    writeln!(f, "  name: {}", corpus.name)?;
    writeln!(f, "  description: {}", corpus.description)?;
    writeln!(f, "  Apparentness: {}", corpus.apparentness)?;
    writeln!(f, "  requests: {}", corpus.request_count)?;
    writeln!(f, "  epochs: {}", corpus.epoch_count)?;
    writeln!(f, "  epoch labels: {}", format_instant_list(&corpus.epochs))?;
    writeln!(f, "  bodies: {}", corpus.body_count)?;
    writeln!(
        f,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    )
}

fn write_corpus_summary_text(text: &mut String, corpus: &CorpusSummary) {
    use std::fmt::Write as _;

    if let Err(error) = corpus.validate() {
        let _ = writeln!(text, "  corpus summary unavailable ({error})");
        return;
    }

    let _ = writeln!(text, "  name: {}", corpus.name);
    let _ = writeln!(text, "  description: {}", corpus.description);
    let _ = writeln!(text, "  Apparentness: {}", corpus.apparentness);
    let _ = writeln!(text, "  requests: {}", corpus.request_count);
    let _ = writeln!(text, "  epochs: {}", corpus.epoch_count);
    let _ = writeln!(
        text,
        "  epoch labels: {}",
        format_instant_list(&corpus.epochs)
    );
    let _ = writeln!(text, "  bodies: {}", corpus.body_count);
    let _ = writeln!(
        text,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    );
}

fn write_backend_matrix(f: &mut fmt::Formatter<'_>, backend: &BackendMetadata) -> fmt::Result {
    writeln!(
        f,
        "  summary: {}",
        backend
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    writeln!(f, "  id: {}", backend.id)?;
    writeln!(f, "  version: {}", backend.version)?;
    writeln!(f, "  family: {}", backend.family)?;
    writeln!(f, "  family posture: {}", backend.family.posture())?;
    writeln!(f, "  accuracy: {}", backend.accuracy)?;
    writeln!(f, "  deterministic: {}", backend.deterministic)?;
    writeln!(f, "  offline: {}", backend.offline)?;
    writeln!(f, "  nominal range: {}", backend.nominal_range)?;
    writeln!(
        f,
        "  time scales: {}",
        format_time_scales(&backend.supported_time_scales)
    )?;
    writeln!(f, "  bodies: {}", format_bodies(&backend.body_coverage))?;
    if let Some(asteroids) = selected_asteroid_coverage(&backend.body_coverage) {
        writeln!(
            f,
            "  selected asteroid coverage: {} bodies ({})",
            asteroids.len(),
            format_bodies(&asteroids)
        )?;
        if backend.id.as_str() == "jpl-snapshot" {
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_evidence_summary_for_report()
            )?;
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_window_summary_for_report()
            )?;
            writeln!(f, "  {}", selected_asteroid_boundary_summary_for_report())?;
            let evidence = reference_asteroid_evidence();
            if let Some(first) = evidence.first() {
                writeln!(
                    f,
                    "  exact J2000 evidence: {} bodies at JD {:.1}",
                    evidence.len(),
                    first.epoch.julian_day.days()
                )?;
                for sample in evidence {
                    writeln!(
                        f,
                        "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                        sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                    )?;
                }
            }
        }
    }
    writeln!(f, "  frames: {}", format_frames(&backend.supported_frames))?;
    writeln!(
        f,
        "  capabilities: {}",
        format_capabilities(&backend.capabilities)
    )?;
    writeln!(
        f,
        "  provenance: {}",
        backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    if !backend.provenance.data_sources.is_empty() {
        writeln!(
            f,
            "  provenance sources: {}",
            backend.provenance.data_sources.join("; ")
        )?;
    }
    Ok(())
}

fn write_backend_catalog_entry(
    f: &mut fmt::Formatter<'_>,
    entry: &BackendMatrixEntry,
) -> fmt::Result {
    write_backend_matrix(f, &entry.metadata)?;
    writeln!(
        f,
        "  implementation status: {}",
        entry.implementation_status.label()
    )?;
    writeln!(f, "  implementation note: {}", entry.status_note)?;
    if entry.metadata.id.as_str() == "pleiades-vsop87" {
        writeln!(f, "  body source profiles:")?;
        for profile in body_source_profiles() {
            writeln!(f, "    {}", profile.summary_line())?;
        }

        writeln!(f, "  source documentation:")?;
        for spec in source_specifications() {
            writeln!(
                f,
                "    {}: {} {} | {} | {} | {} | {} | {} | {} | {}",
                spec.body,
                spec.variant,
                spec.source_file,
                spec.coordinate_family,
                spec.frame,
                spec.units,
                spec.reduction,
                spec.transform_note,
                spec.truncation_policy,
                spec.date_range
            )?;
        }

        writeln!(f, "  source audit:")?;
        for audit in source_audits() {
            writeln!(
                f,
                "    {}: {} bytes, {} lines, {} terms, 0x{:016x}",
                audit.body,
                audit.byte_length,
                audit.line_count,
                audit.term_count,
                audit.fingerprint
            )?;
        }

        writeln!(f, "  generated binary audit:")?;
        writeln!(f, "    {}", generated_binary_audit_summary_for_report())?;

        writeln!(f, "  canonical J2000 VSOP87B evidence:")?;
        match vsop87_canonical_body_evidence() {
            Some(body_evidence) => {
                for evidence in body_evidence {
                    writeln!(
                        f,
                        "    {}: kind={} from {} — {} — Δlon={:.12}° / limit {:.12}° / margin {:+.12}°, Δlat={:.12}° / limit {:.12}° / margin {:+.12}°, Δdist={:.12} AU / limit {:.12} AU / margin {:+.12} AU",
                        evidence.body,
                        evidence.source_kind,
                        evidence.source_file,
                        if evidence.within_interim_limits {
                            evidence.provenance
                        } else {
                            "outside interim limits"
                        },
                        evidence.longitude_delta_deg,
                        evidence.longitude_limit_deg,
                        evidence.longitude_limit_deg - evidence.longitude_delta_deg,
                        evidence.latitude_delta_deg,
                        evidence.latitude_limit_deg,
                        evidence.latitude_limit_deg - evidence.latitude_delta_deg,
                        evidence.distance_delta_au,
                        evidence.distance_limit_au,
                        evidence.distance_limit_au - evidence.distance_delta_au
                    )?;
                }
            }
            None => {
                writeln!(f, "    unavailable")?;
            }
        }
        writeln!(
            f,
            "  body profile evidence summary: {}",
            format_vsop87_body_evidence_summary()
        )?;
    } else if entry.metadata.id.as_str() == "pleiades-elp" {
        let theory = lunar_theory_specification();
        writeln!(f, "  lunar theory specification:")?;
        writeln!(
            f,
            "    catalog summary: {}",
            lunar_theory_catalog_summary_for_report()
        )?;
        writeln!(
            f,
            "    catalog validation: {}",
            lunar_theory_catalog_validation_summary_for_report()
        )?;
        writeln!(f, "    model: {}", theory.model_name)?;
        writeln!(
            f,
            "    source family: {}",
            pleiades_elp::lunar_theory_source_family().label()
        )?;
        writeln!(
            f,
            "    capability summary: {}",
            lunar_theory_capability_summary_for_report()
        )?;
        writeln!(
            f,
            "    specification summary: {}",
            lunar_theory_summary_for_report()
        )?;
        writeln!(f, "    source identifier: {}", theory.source_identifier)?;
        writeln!(f, "    source citation: {}", theory.source_citation)?;
        writeln!(f, "    source material: {}", theory.source_material)?;
        writeln!(f, "    redistribution note: {}", theory.redistribution_note)?;
        writeln!(f, "    license note: {}", theory.license_note)?;
        writeln!(
            f,
            "    supported bodies: {}",
            format_bodies(theory.supported_bodies)
        )?;
        writeln!(
            f,
            "    unsupported bodies: {}",
            format_bodies(theory.unsupported_bodies)
        )?;
        writeln!(
            f,
            "    request policy: {}",
            lunar_theory_request_policy_summary()
        )?;
        writeln!(f, "    validation window: {}", theory.validation_window)?;
        writeln!(f, "    date-range note: {}", theory.date_range_note)?;
        writeln!(f, "    frame note: {}", theory.frame_note)?;
        write_lunar_reference_evidence(f)?;
        write_lunar_equatorial_reference_evidence(f)?;
        write_lunar_apparent_comparison_evidence(f)?;
        write_lunar_source_window_evidence(f)?;
        writeln!(f, "  Lunar high-curvature continuity evidence:")?;
        writeln!(
            f,
            "    {}",
            lunar_high_curvature_continuity_evidence_for_report()
        )?;
        write_lunar_high_curvature_equatorial_continuity_evidence(f)?;
    }
    if entry.metadata.id.as_str() == "jpl-snapshot" {
        write_jpl_interpolation_quality(f)?;
        writeln!(
            f,
            "    {}",
            jpl_snapshot_batch_error_taxonomy_summary_for_report()
        )?;
    }
    writeln!(
        f,
        "  expected error classes: {}",
        format_error_kinds(entry.expected_error_kinds)
    )?;
    if entry.required_data_files.is_empty() {
        writeln!(f, "  required external data files: none")?;
    } else {
        writeln!(
            f,
            "  required external data files: {}",
            format_data_files(entry.required_data_files)
        )?;
    }
    Ok(())
}

fn write_jpl_interpolation_quality(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  interpolation quality checks:")?;
    let Some(summary) = jpl_interpolation_quality_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        format_jpl_interpolation_quality_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        jpl_interpolation_quality_kind_coverage_for_report()
    )?;
    writeln!(f, "    {}", jpl_interpolation_posture_summary_for_report())?;
    writeln!(f, "    {}", jpl_independent_holdout_summary_for_report())?;
    writeln!(f, "    {}", reference_holdout_overlap_summary_for_report())?;
    writeln!(
        f,
        "    {}",
        independent_holdout_snapshot_body_class_coverage_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        jpl_independent_holdout_snapshot_batch_parity_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    )?;
    for sample in interpolation_quality_samples() {
        writeln!(f, "    {}", sample.summary_line())?;
    }
    Ok(())
}

fn jpl_interpolation_quality_summary() -> Option<pleiades_jpl::JplInterpolationQualitySummary> {
    pleiades_jpl::jpl_interpolation_quality_summary()
}

fn format_jpl_interpolation_quality_summary(
    summary: &pleiades_jpl::JplInterpolationQualitySummary,
) -> String {
    pleiades_jpl::format_jpl_interpolation_quality_summary(summary)
}

fn write_lunar_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar reference:")?;
    let Some(summary) = lunar_reference_evidence_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        pleiades_elp::format_lunar_reference_evidence_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        pleiades_elp::lunar_reference_batch_parity_summary_for_report()
    )?;
    writeln!(f, "    {}", lunar_reference_evidence_envelope_for_report())?;
    for sample in lunar_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

fn write_lunar_equatorial_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar equatorial reference:")?;
    if lunar_equatorial_reference_evidence_summary().is_none() {
        writeln!(f, "    none")?;
        return Ok(());
    }

    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_evidence_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_batch_parity_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_evidence_envelope_for_report()
    )?;
    for sample in lunar_equatorial_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

fn write_lunar_apparent_comparison_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar apparent comparison:")?;
    let Some(summary) = lunar_apparent_comparison_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(f, "    {}", summary.summary_line())?;
    for sample in lunar_apparent_comparison_evidence() {
        writeln!(
            f,
            "    {} at JD {:.1}: apparent lon={:.12}°, apparent lat={:.12}°, apparent dist={:.12} AU, apparent RA={:.12}°, apparent Dec={:.12}°, note={}",
            sample.body,
            sample.epoch.julian_day.days(),
            sample.apparent_longitude_deg,
            sample.apparent_latitude_deg,
            sample.apparent_distance_au,
            sample.apparent_right_ascension_deg,
            sample.apparent_declination_deg,
            sample.note
        )?;
    }
    Ok(())
}

fn write_lunar_source_window_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar source windows:")?;
    writeln!(f, "    {}", lunar_source_window_summary_for_report())?;
    Ok(())
}

fn write_lunar_high_curvature_equatorial_continuity_evidence(
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(f, "  Lunar high-curvature equatorial continuity evidence:")?;
    writeln!(
        f,
        "    {}",
        lunar_high_curvature_equatorial_continuity_evidence_for_report()
    )?;
    Ok(())
}

fn write_comparison_summary(f: &mut fmt::Formatter<'_>, report: &ComparisonReport) -> fmt::Result {
    let summary = &report.summary;
    let comparison_envelope = comparison_envelope_summary(summary, &report.samples);
    let median = comparison_envelope.median;

    writeln!(f, "  samples: {}", summary.sample_count)?;
    writeln!(
        f,
        "  max longitude delta: {:.12}°",
        summary.max_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean longitude delta: {:.12}°",
        summary.mean_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  median longitude delta: {:.12}°",
        median.longitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms longitude delta: {:.12}°",
        summary.rms_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  max latitude delta: {:.12}°",
        summary.max_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean latitude delta: {:.12}°",
        summary.mean_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  median latitude delta: {:.12}°",
        median.latitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms latitude delta: {:.12}°",
        summary.rms_latitude_delta_deg
    )?;
    if let Some(value) = summary.max_distance_delta_au {
        writeln!(f, "  max distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.mean_distance_delta_au {
        writeln!(f, "  mean distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = median.distance_delta_au {
        writeln!(f, "  median distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.rms_distance_delta_au {
        writeln!(f, "  rms distance delta: {:.12} AU", value)?;
    }
    match comparison_envelope.validated_percentile_line(&report.samples) {
        Ok(line) => writeln!(f, "  {line}")?,
        Err(error) => writeln!(f, "  comparison percentile envelope unavailable ({error})")?,
    }
    Ok(())
}

fn format_body_comparison_summary_for_report(summary: &BodyComparisonSummary) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!(
            "body comparison summary for {} unavailable ({error})",
            summary.body
        ),
    }
}

fn write_body_comparison_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyComparisonSummary],
) -> fmt::Result {
    writeln!(f, "Body comparison summaries")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        writeln!(
            f,
            "  {}",
            format_body_comparison_summary_for_report(summary)
        )?;
    }
    Ok(())
}

fn write_body_class_envelopes(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
) -> fmt::Result {
    writeln!(f, "Body-class error envelopes")?;
    let summaries = body_class_summaries(samples);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

fn write_body_class_tolerance_posture(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
    backend_family: &BackendFamily,
) -> fmt::Result {
    writeln!(f, "Body-class tolerance posture")?;
    let summaries = body_class_tolerance_summaries(samples, backend_family);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

fn tolerance_backend_family_label(family: &BackendFamily) -> String {
    match family {
        BackendFamily::Algorithmic => "algorithmic".to_string(),
        BackendFamily::ReferenceData => "reference data".to_string(),
        BackendFamily::CompressedData => "compressed data".to_string(),
        BackendFamily::Composite => "composite".to_string(),
        BackendFamily::Other(value) => format!("other ({value})"),
        _ => "other (unknown)".to_string(),
    }
}

fn write_tolerance_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyToleranceSummary],
) -> fmt::Result {
    writeln!(f, "Expected tolerance status")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        match summary.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(
                f,
                "  body tolerance summary for {} unavailable ({error})",
                summary.body
            ),
        }?;
    }
    Ok(())
}

fn write_regression_section(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    findings: &[RegressionFinding],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    if findings.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in findings {
        writeln!(f, "  {}", finding.summary_line())?;
    }
    Ok(())
}

fn write_regression_archive_section(
    f: &mut fmt::Formatter<'_>,
    archive: &RegressionArchive,
) -> fmt::Result {
    writeln!(f, "Archived regression cases")?;
    writeln!(f, "  corpus: {}", archive.corpus_name)?;
    if archive.cases.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in &archive.cases {
        writeln!(f, "  {}", finding.summary_line())?;
    }
    Ok(())
}

fn write_reference_asteroid_section(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Selected asteroid coverage")?;
    let asteroids = reference_asteroids();
    if asteroids.is_empty() {
        writeln!(f, "  none")?;
    } else {
        writeln!(f, "  bodies: {}", format_bodies(asteroids))?;
        let evidence = reference_asteroid_evidence();
        if evidence.is_empty() {
            writeln!(f, "  exact J2000 evidence: unavailable")?;
        } else {
            writeln!(
                f,
                "  exact J2000 evidence: {} bodies at JD {:.1}",
                evidence.len(),
                evidence[0].epoch.julian_day.days()
            )?;
            for sample in evidence {
                writeln!(
                    f,
                    "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                    sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                )?;
            }
        }
        writeln!(
            f,
            "  note: comparison reports stay on the planetary subset while the JPL snapshot preserves selected asteroid coverage."
        )?;
    }
    Ok(())
}

fn regression_finding(
    sample: &ComparisonSample,
    backend_family: &BackendFamily,
) -> Option<RegressionFinding> {
    let tolerance = comparison_tolerance_for_body(&sample.body, backend_family);
    let mut notes = Vec::new();
    if sample.longitude_delta_deg >= tolerance.max_longitude_delta_deg {
        notes.push(format!(
            "longitude delta exceeds {:.1}°",
            tolerance.max_longitude_delta_deg
        ));
    }
    if sample.latitude_delta_deg >= tolerance.max_latitude_delta_deg {
        notes.push(format!(
            "latitude delta exceeds {:.2}°",
            tolerance.max_latitude_delta_deg
        ));
    }
    if sample
        .distance_delta_au
        .is_some_and(|value| value >= tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY))
    {
        notes.push(format!(
            "distance delta exceeds {:.3} AU",
            tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY)
        ));
    }

    if notes.is_empty() {
        return None;
    }

    Some(RegressionFinding {
        body: sample.body.clone(),
        longitude_delta_deg: sample.longitude_delta_deg,
        latitude_delta_deg: sample.latitude_delta_deg,
        distance_delta_au: sample.distance_delta_au,
        note: notes.join(", "),
    })
}

const JPL_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
];
const JPL_REQUIRED_DATA_FILES: &[&str] = &[
    "crates/pleiades-jpl/data/reference_snapshot.csv",
    "crates/pleiades-jpl/data/j2000_snapshot.csv",
];
const VSOP87_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
const ELP_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
const PACKAGED_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];
const COMPOSITE_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];

fn implemented_backend_catalog() -> Vec<BackendMatrixEntry> {
    vec![
        BackendMatrixEntry {
            label: "JPL snapshot reference backend",
            metadata: default_reference_backend().metadata(),
            implementation_status: BackendImplementationStatus::FixtureReference,
            status_note: "checked-in public-input derivative fixture with exact lookup and cubic interpolation on four-sample windows when available, with quadratic and linear fallbacks for sparser bodies; expanded hold-out coverage now includes additional Jupiter rows and a Moon sample, while a larger reference corpus remains planned",
            expected_error_kinds: JPL_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
        BackendMatrixEntry {
            label: "VSOP87 planetary backend",
            metadata: Vsop87Backend::new().metadata(),
            implementation_status: BackendImplementationStatus::PartialSourceBacked,
            status_note: "Sun through Neptune now use generated binary VSOP87B source tables derived from the vendored full-file inputs, and Pluto remains the current approximate mean-element fallback special case until a Pluto-specific source path is selected",
            expected_error_kinds: VSOP87_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "ELP lunar backend (Moon and lunar nodes)",
            metadata: ElpBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
            status_note: "compact lunar and lunar-point formulas provide the current deterministic baseline while documented production lunar-theory ingestion remains open",
            expected_error_kinds: ELP_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Packaged data backend",
            metadata: PackagedDataBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::PrototypeArtifact,
            status_note: "sample packaged artifact exercises lookup and profile plumbing; generated 1500-2500 production artifacts are Phase 2 work",
            expected_error_kinds: PACKAGED_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Composite routed backend",
            metadata: default_candidate_backend().metadata(),
            implementation_status: BackendImplementationStatus::RoutingFacade,
            status_note: "routes current planetary and lunar implementations for chart-facing validation without increasing underlying backend accuracy claims",
            expected_error_kinds: COMPOSITE_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
    ]
}

struct BackendMatrixEntry {
    label: &'static str,
    metadata: BackendMetadata,
    implementation_status: BackendImplementationStatus,
    status_note: &'static str,
    expected_error_kinds: &'static [EphemerisErrorKind],
    required_data_files: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BackendImplementationStatus {
    FixtureReference,
    PartialSourceBacked,
    PreliminaryAlgorithm,
    PrototypeArtifact,
    RoutingFacade,
}

impl BackendImplementationStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::FixtureReference => "fixture-reference",
            Self::PartialSourceBacked => "partial-source-backed",
            Self::PreliminaryAlgorithm => "preliminary-algorithm",
            Self::PrototypeArtifact => "prototype-artifact",
            Self::RoutingFacade => "routing-facade",
        }
    }
}

fn write_backend_catalog(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    catalog: &[BackendMatrixEntry],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for entry in catalog {
        writeln!(f, "{}", entry.label)?;
        write_backend_catalog_entry(f, entry)?;
        writeln!(f)?;
    }
    Ok(())
}

fn format_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(|body| body.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn selected_asteroid_coverage(bodies: &[CelestialBody]) -> Option<Vec<CelestialBody>> {
    let asteroids = bodies
        .iter()
        .filter(|body| is_selected_asteroid(body))
        .cloned()
        .collect::<Vec<_>>();

    if asteroids.is_empty() {
        None
    } else {
        Some(asteroids)
    }
}

fn is_selected_asteroid(body: &CelestialBody) -> bool {
    match body {
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => true,
        CelestialBody::Custom(custom) => custom.catalog == "asteroid",
        _ => false,
    }
}

fn format_frames(frames: &[CoordinateFrame]) -> String {
    frames
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_time_scales(scales: &[TimeScale]) -> String {
    scales
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_capabilities(capabilities: &BackendCapabilities) -> String {
    match capabilities.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("unavailable ({error})"),
    }
}

fn format_error_kinds(kinds: &[EphemerisErrorKind]) -> String {
    kinds
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_data_files(files: &[&str]) -> String {
    files.join("; ")
}

fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

fn format_instant_list(instants: &[Instant]) -> String {
    if instants.is_empty() {
        return "none".to_string();
    }

    instants
        .iter()
        .copied()
        .map(format_instant)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_ns(value: f64) -> String {
    format!("{value:.2}")
}

fn format_duration(duration: std::time::Duration) -> String {
    format!("{:.6}s", duration.as_secs_f64())
}

fn ensure_no_extra_args(args: &[&str], command: &str) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("{command} does not accept extra arguments"))
    }
}

fn parse_rounds(args: &[&str], default: usize) -> Result<usize, String> {
    let mut rounds = default;
    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--rounds" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value for --rounds".to_string())?;
                rounds = value
                    .parse::<usize>()
                    .map_err(|error| format!("invalid value for --rounds: {error}"))?;
                if rounds == 0 {
                    return Err(
                        "invalid value for --rounds: expected a positive integer".to_string()
                    );
                }
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(rounds)
}

fn help_text() -> String {
    let corpus_size = default_corpus().requests.len();
    format!(
        "{banner}\n\nCommands:\n  compare-backends          Compare the JPL snapshot against the algorithmic composite backend\n  compare-backends-audit    Compare the JPL snapshot against the algorithmic composite backend and fail if the tolerance audit reports regressions\n  backend-matrix            Print the implemented backend capability matrices\n  capability-matrix         Alias for backend-matrix\n  backend-matrix-summary    Print the compact backend capability matrix summary\n  matrix-summary            Alias for backend-matrix-summary\n  compatibility-profile     Print the release compatibility profile\n  profile                   Alias for compatibility-profile\n  benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus and full chart assembly on representative house scenarios\n  comparison-corpus-summary  Print the compact release-grade comparison corpus summary\n  comparison-corpus-release-guard-summary  Print the compact release-grade comparison corpus guard summary\n  comparison-corpus-guard-summary  Alias for comparison-corpus-release-guard-summary\n  comparison-envelope-summary  Print the compact comparison envelope summary\n  benchmark-corpus-summary  Print the compact representative benchmark corpus summary\n  report [--rounds N]       Render the full validation report\n  generate-report           Alias for report\n  validation-report-summary [--rounds N]  Render a compact validation report summary\n  report-summary [--rounds N]  Alias for validation-report-summary\n  validation-summary        Alias for validation-report-summary\n  validate-artifact         Inspect and validate the bundled compressed artifact\n  regenerate-packaged-artifact  Rebuild or verify the packaged artifact fixture from the checked-in reference snapshot; pass a file path, --out FILE, or --check\n  artifact-summary          Print the compact packaged-artifact summary\n  artifact-posture-summary  Alias for artifact-summary\n  artifact-boundary-envelope-summary  Print the compact packaged-artifact boundary envelope summary\n  artifact-profile-coverage-summary  Print the packaged-artifact profile coverage summary\n  packaged-artifact-output-support-summary  Print the packaged-artifact output support summary\n  packaged-artifact-storage-summary  Print the packaged-artifact storage/reconstruction summary\n  packaged-artifact-production-profile-summary  Print the packaged-artifact production profile skeleton summary\n  packaged-artifact-target-threshold-summary  Print the packaged-artifact target thresholds summary\n  packaged-artifact-target-threshold-scope-envelopes-summary  Print the packaged-artifact target-threshold scope envelopes summary\n  packaged-artifact-generation-manifest-summary  Print the packaged-artifact generation manifest summary\n  packaged-artifact-generation-policy-summary  Print the packaged-artifact generation policy summary\n  packaged-artifact-generation-residual-bodies-summary  Print the packaged-artifact generation residual bodies summary\n  packaged-artifact-regeneration-summary  Print the packaged-artifact regeneration summary\n  packaged-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary\n  workspace-audit           Check the workspace for mandatory native build hooks\n  audit                     Alias for workspace-audit\n  native-dependency-audit   Alias for workspace-audit\n  workspace-audit-summary   Print the compact workspace audit summary\n  native-dependency-audit-summary  Alias for workspace-audit-summary\n  api-stability             Print the release API stability posture\n  api-posture               Alias for api-stability\n  api-stability-summary     Print the compact API stability summary\n  api-posture-summary       Alias for api-stability-summary\n  compatibility-profile-summary  Print the compact compatibility profile summary\n  catalog-inventory-summary  Print the compact compatibility catalog inventory summary\n  profile-summary           Alias for compatibility-profile-summary\n  verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs\n  release-notes             Print the release compatibility notes\n  release-notes-summary     Print the compact release notes summary\n  release-checklist         Print the release maintainer checklist\n  release-checklist-summary Print the compact release checklist summary\n  checklist-summary        Alias for release-checklist-summary\n  release-summary           Print the compact release summary\n  jpl-batch-error-taxonomy-summary  Print the compact JPL batch error taxonomy summary\n  jpl-snapshot-evidence-summary  Print the compact combined JPL evidence summary\n  production-generation-boundary-summary  Print the compact production-generation boundary overlay summary\n  production-generation-boundary-request-corpus-summary  Print the compact production-generation boundary request corpus summary\n  production-generation-body-class-coverage-summary  Print the compact production-generation body-class coverage summary\n  production-generation-source-window-summary  Print the compact production-generation source windows summary\n  production-generation-summary  Print the compact production-generation coverage summary
  production-generation-boundary-source-summary  Print the compact production-generation boundary source summary
  production-generation-boundary-window-summary  Print the compact production-generation boundary windows summary\n  production-generation-source-summary  Print the compact production-generation source summary\n  comparison-snapshot-source-window-summary  Print the compact comparison snapshot source windows summary\n  comparison-snapshot-source-summary  Print the compact comparison snapshot source summary\n  comparison-snapshot-body-class-coverage-summary  Print the compact comparison snapshot body-class coverage summary\n  comparison-snapshot-manifest-summary  Print the compact comparison snapshot manifest summary\n  comparison-snapshot-summary  Print the compact comparison snapshot summary\n  comparison-snapshot-batch-parity-summary  Print the compact comparison snapshot batch parity summary\n  reference-snapshot-source-window-summary  Print the compact reference snapshot source windows summary\n  reference-snapshot-source-summary  Print the compact reference snapshot source summary\n  reference-snapshot-lunar-boundary-summary  Print the compact reference lunar boundary evidence summary\n  reference-snapshot-1749-major-body-boundary-summary  Print the compact reference 1749 major-body boundary evidence summary\n  1749-major-body-boundary-summary  Alias for reference-snapshot-1749-major-body-boundary-summary\n  reference-snapshot-early-major-body-boundary-summary  Print the compact reference early major-body boundary evidence summary\n  early-major-body-boundary-summary  Alias for reference-snapshot-early-major-body-boundary-summary\n  reference-snapshot-1800-major-body-boundary-summary  Print the compact reference 1800 major-body boundary evidence summary\n  1800-major-body-boundary-summary  Alias for reference-snapshot-1800-major-body-boundary-summary\n  reference-snapshot-2500-major-body-boundary-summary  Print the compact reference 2500 major-body boundary evidence summary\n  2500-major-body-boundary-summary  Alias for reference-snapshot-2500-major-body-boundary-summary\n  reference-snapshot-major-body-boundary-summary  Print the compact reference major-body boundary evidence summary\n  reference-snapshot-mars-jupiter-boundary-summary  Print the compact reference Mars/Jupiter boundary evidence summary\n  reference-snapshot-mars-outer-boundary-summary  Print the compact reference Mars outer-boundary evidence summary\n  reference-snapshot-major-body-boundary-window-summary  Print the compact reference major-body boundary windows summary\n  reference-snapshot-body-class-coverage-summary  Print the compact reference snapshot body-class coverage summary\n  reference-snapshot-manifest-summary  Print the compact reference snapshot manifest summary\n  reference-snapshot-summary  Print the compact reference snapshot summary\n  reference-snapshot-batch-parity-summary  Print the compact reference snapshot batch parity summary\n  reference-snapshot-equatorial-parity-summary  Print the compact reference snapshot equatorial parity summary\n  reference-high-curvature-summary  Print the compact reference major-body high-curvature evidence summary\n  high-curvature-summary  Alias for reference-high-curvature-summary\n  reference-high-curvature-window-summary  Print the compact reference major-body high-curvature windows summary\n  high-curvature-window-summary  Alias for reference-high-curvature-window-summary\n  reference-snapshot-boundary-epoch-coverage-summary  Print the compact reference snapshot boundary epoch coverage summary\n  boundary-epoch-coverage-summary  Alias for reference-snapshot-boundary-epoch-coverage-summary\n  source-documentation-summary  Print the compact VSOP87 source-documentation summary\n  source-documentation-health-summary  Print the compact VSOP87 source-documentation health summary\n  source-audit-summary      Print the compact VSOP87 source audit summary\n  generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary\n  time-scale-policy-summary  Print the compact time-scale policy summary\n  delta-t-policy-summary   Print the compact Delta T policy summary\n  observer-policy-summary  Print the compact observer policy summary\n  apparentness-policy-summary  Print the compact apparentness policy summary\n  interpolation-posture-summary  Print the compact JPL interpolation posture summary\n  interpolation-quality-summary  Print the compact JPL interpolation quality summary\n  interpolation-quality-kind-coverage-summary  Print the compact JPL interpolation quality kind coverage summary\n  lunar-reference-error-envelope-summary  Print the compact lunar reference error envelope summary\n  lunar-equatorial-reference-error-envelope-summary  Print the compact lunar equatorial reference error envelope summary\n  lunar-apparent-comparison-summary  Print the compact lunar apparent comparison summary\n  lunar-source-window-summary  Print the compact lunar source windows summary\n  lunar-theory-summary      Print the compact ELP lunar theory specification\n  lunar-theory-capability-summary  Print the compact ELP lunar capability summary\n  lunar-theory-source-summary  Print the compact ELP lunar source summary\n  selected-asteroid-boundary-summary  Print the compact selected-asteroid boundary evidence summary\n  reference-snapshot-selected-asteroid-terminal-boundary-summary  Print the compact selected-asteroid terminal boundary evidence summary\n  selected-asteroid-source-evidence-summary  Print the compact selected-asteroid source evidence summary\n  selected-asteroid-source-summary  Alias for selected-asteroid-source-evidence-summary\n  selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary\n  selected-asteroid-batch-parity-summary  Print the compact selected-asteroid batch-parity summary\n  reference-asteroid-evidence-summary  Print the compact reference asteroid evidence summary\n  reference-asteroid-equatorial-evidence-summary  Print the compact reference asteroid equatorial evidence summary\n  reference-asteroid-source-window-summary  Print the compact reference asteroid source windows summary\n  reference-holdout-overlap-summary  Print the compact reference/hold-out overlap summary\n  independent-holdout-source-window-summary  Print the compact independent hold-out source windows summary\n  independent-holdout-summary  Print the compact independent hold-out summary\n  independent-holdout-source-summary  Print the compact independent hold-out source summary\n  independent-holdout-body-class-coverage-summary  Print the compact independent hold-out body-class coverage summary\n  independent-holdout-batch-parity-summary  Print the compact independent hold-out batch parity summary\n  independent-holdout-equatorial-parity-summary  Print the compact independent hold-out equatorial parity summary\n  house-validation-summary   Print the compact house-validation corpus summary\n  house-formula-families-summary  Print the compact house formula families summary\n  house-code-aliases-summary  Print the compact house-code alias summary\n  house-code-alias-summary  Alias for house-code-aliases-summary\n  ayanamsa-catalog-validation-summary  Print the compact ayanamsa catalog validation summary\n  ayanamsa-metadata-coverage-summary  Print the compact ayanamsa sidereal metadata coverage summary\n  ayanamsa-reference-offsets-summary  Print the compact ayanamsa reference offsets summary\n  frame-policy-summary      Print the compact frame-policy summary\n  mean-obliquity-frame-round-trip-summary  Print the compact mean-obliquity frame round-trip summary\n  release-profile-identifiers-summary  Print the compact release-profile identifiers summary\n  request-surface-summary  Print the compact request-surface inventory summary\n  request-policy-summary    Print the compact request-policy summary\n  request-semantics-summary Alias for request-policy-summary\n  comparison-tolerance-policy-summary  Print the compact comparison tolerance policy summary\n  pluto-fallback-summary   Print the compact Pluto fallback summary\n  bundle-release --out DIR  Write the release compatibility profile, profile summary, release notes, release notes summary, release summary, release-profile identifiers, release checklist, release checklist summary, backend matrix, backend matrix summary, API posture, API stability summary, validation report summary, workspace audit summary, artifact summary, packaged-artifact generation manifest, benchmark report, validation report, manifest, and manifest checksum sidecar\n  verify-release-bundle     Read a staged release bundle back and verify its manifest checksums\n  help                      Show this help text\n\nDefault benchmark rounds: {DEFAULT_BENCHMARK_ROUNDS}\nDefault comparison corpus size: {corpus_size}",
        banner = banner(),
        corpus_size = corpus_size,
    )
}

fn parse_release_bundle_args(
    args: &[&str],
    default_rounds: usize,
) -> Result<(PathBuf, usize), String> {
    let mut output_dir: Option<PathBuf> = None;
    let mut rounds = default_rounds;
    let mut iter = args.iter().copied();

    while let Some(arg) = iter.next() {
        match arg {
            "--out" | "--output" => {
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;
                output_dir = Some(PathBuf::from(value));
            }
            "--rounds" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value for --rounds".to_string())?;
                rounds = value
                    .parse::<usize>()
                    .map_err(|error| format!("invalid value for --rounds: {error}"))?;
                if rounds == 0 {
                    return Err(
                        "invalid value for --rounds: expected a positive integer".to_string()
                    );
                }
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let output_dir =
        output_dir.ok_or_else(|| "missing required --out <dir> argument".to_string())?;
    Ok((output_dir, rounds))
}

enum PackagedArtifactCommand {
    Write { output_path: String },
    Check,
}

fn parse_packaged_artifact_command(args: &[&str]) -> Result<PackagedArtifactCommand, String> {
    match args {
        [] => Err(
            "missing required output path argument; pass a file path, --out <file>, or --check"
                .to_string(),
        ),
        ["--check"] => Ok(PackagedArtifactCommand::Check),
        ["--out"] => Err("missing value for --out".to_string()),
        ["--out", path] => Ok(PackagedArtifactCommand::Write {
            output_path: (*path).to_string(),
        }),
        ["--out", _, extra, ..] => Err(format!("unknown argument: {extra}")),
        [path] if !path.starts_with('-') => Ok(PackagedArtifactCommand::Write {
            output_path: (*path).to_string(),
        }),
        [other, ..] => Err(format!("unknown argument: {other}")),
    }
}

fn render_packaged_artifact_regeneration(output_path: String) -> Result<String, String> {
    let artifact = regenerate_packaged_artifact();
    let encoded = artifact.encode().map_err(|error| error.to_string())?;
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }
    fs::write(&output_path, &encoded)
        .map_err(|error| format!("failed to write {}: {error}", output_path))?;

    Ok(format!(
        "Packaged artifact regenerated\n  path: {}\n  label: {}\n  source: {}\n  checksum: 0x{:016x}\n  bytes: {}\n  {}",
        output_path,
        artifact.header.generation_label,
        artifact.header.source,
        artifact.checksum,
        encoded.len(),
        packaged_artifact_regeneration_summary_for_report(),
    ))
}

fn render_packaged_artifact_regeneration_check() -> Result<String, String> {
    let artifact = regenerate_packaged_artifact();
    let regenerated = artifact.encode().map_err(|error| error.to_string())?;
    let committed = packaged_artifact_bytes();

    if regenerated.as_slice() != committed {
        return Err(format!(
            "packaged artifact regeneration check failed: regenerated {} bytes did not match the checked-in fixture {} bytes",
            regenerated.len(),
            committed.len()
        ));
    }

    Ok(format!(
        "Packaged artifact regeneration check passed\n  label: {}\n  source: {}\n  checksum: 0x{:016x}\n  bytes: {}\n  {}",
        artifact.header.generation_label,
        artifact.header.source,
        artifact.checksum,
        regenerated.len(),
        packaged_artifact_regeneration_summary_for_report(),
    ))
}

fn render_error(error: EphemerisError) -> String {
    error.summary_line()
}

fn render_artifact_error(error: crate::artifact::ArtifactInspectionError) -> String {
    error.to_string()
}

fn render_release_bundle_error(error: ReleaseBundleError) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use pleiades_core::{
        current_release_profile_identifiers, Apparentness, Ayanamsa, CoordinateFrame, HouseSystem,
        JulianDay, Latitude, TimeScale,
    };

    use super::*;
    use pleiades_jpl::comparison_bodies;
    use std::path::Path;

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = format!(
            "{}-{}-{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after UNIX_EPOCH")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("temporary directory should be creatable");
        path
    }

    fn assert_report_contains_exact_line(report: &str, expected: &str) {
        let expected = expected.trim_start();
        assert!(
            report.lines().any(|line| line.trim_start() == expected),
            "expected report to contain line `{expected}`\nreport:\n{report}"
        );
    }

    fn assert_release_bundle_rejects_tampered_text_file(
        bundle_dir_prefix: &str,
        file_name: &str,
        expected_fragment: &str,
    ) {
        let bundle_dir = unique_temp_dir(bundle_dir_prefix);
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let file_path = bundle_dir.join(file_name);
        let mut text = std::fs::read_to_string(&file_path)
            .unwrap_or_else(|error| panic!("{file_name} should exist: {error}"));
        text.push_str("\nTampered for regression coverage.\n");
        std::fs::write(&file_path, text)
            .unwrap_or_else(|error| panic!("{file_name} should be writable: {error}"));

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a tampered release bundle file");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains(expected_fragment),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[cfg(unix)]
    fn assert_release_bundle_rejects_symlinked_text_file(
        bundle_dir_prefix: &str,
        file_name: &str,
        link_target: &str,
        expected_fragment: &str,
    ) {
        use std::os::unix::fs::symlink;

        let bundle_dir = unique_temp_dir(bundle_dir_prefix);
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let file_path = bundle_dir.join(file_name);
        std::fs::remove_file(&file_path).expect("bundled text file should be removable");
        symlink(link_target, &file_path).expect("symlink should be creatable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a symlinked release bundle file");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains(expected_fragment),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    fn assert_release_bundle_rejects_missing_manifest_entry(
        bundle_dir_prefix: &str,
        manifest_line_prefix: &str,
        expected_fragments: &[&str],
    ) {
        let bundle_dir = unique_temp_dir(bundle_dir_prefix);
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let filtered = manifest
            .lines()
            .filter(|line| !line.starts_with(manifest_line_prefix))
            .map(str::to_owned)
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&manifest_path, format!("{filtered}\n"))
            .expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a manifest missing the requested entry");
        assert!(
            expected_fragments
                .iter()
                .any(|fragment| error.contains(fragment)),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    fn assert_release_bundle_rejects_blank_manifest_value(
        bundle_dir_prefix: &str,
        manifest_line_prefix: &str,
        expected_fragments: &[&str],
    ) {
        let bundle_dir = unique_temp_dir(bundle_dir_prefix);
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let rewritten = manifest
            .lines()
            .map(|line| {
                if line.starts_with(manifest_line_prefix) {
                    manifest_line_prefix.to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&manifest_path, format!("{rewritten}\n"))
            .expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a manifest with a blank requested entry");
        assert!(
            expected_fragments
                .iter()
                .any(|fragment| error.contains(fragment)),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    fn assert_release_bundle_rejects_duplicate_manifest_entry(
        bundle_dir_prefix: &str,
        manifest_line_prefix: &str,
        expected_fragments: &[&str],
    ) {
        let bundle_dir = unique_temp_dir(bundle_dir_prefix);
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let duplicate_line = manifest
            .lines()
            .find(|line| line.starts_with(manifest_line_prefix))
            .unwrap_or_else(|| panic!("{manifest_line_prefix} should exist"));
        let mut lines = manifest.lines().map(str::to_owned).collect::<Vec<_>>();
        lines.push(duplicate_line.to_string());
        std::fs::write(&manifest_path, format!("{}\n", lines.join("\n")))
            .expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a manifest with a duplicate requested entry");
        assert!(
            expected_fragments
                .iter()
                .any(|fragment| error.contains(fragment)),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    fn assert_release_bundle_rejects_whitespace_manifest_entry(
        bundle_dir_prefix: &str,
        manifest_line_prefix: &str,
        expected_fragments: &[&str],
    ) {
        let bundle_dir = unique_temp_dir(bundle_dir_prefix);
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let rewritten = manifest
            .lines()
            .map(|line| {
                if line.starts_with(manifest_line_prefix) {
                    format!("{line} ")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&manifest_path, format!("{rewritten}\n"))
            .expect("manifest should be writable");

        let checksum = checksum64(
            &std::fs::read_to_string(&manifest_path).expect("manifest should exist after rewrite"),
        );
        std::fs::write(
            bundle_dir.join("bundle-manifest.checksum.txt"),
            format!("0x{checksum:016x}\n"),
        )
        .expect("manifest checksum sidecar should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a manifest with noncanonical whitespace");
        assert!(
            expected_fragments
                .iter()
                .any(|fragment| error.contains(fragment)),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn default_corpus_covers_the_comparison_snapshot() {
        let corpus = default_corpus();
        let summary = corpus.summary();
        assert_eq!(corpus.requests.len(), 112);
        assert_eq!(summary.epoch_count, 14);
        assert_eq!(summary.epochs.len(), 14);
        assert!(summary
            .epochs
            .iter()
            .all(|epoch| epoch.scale == TimeScale::Tt));
        assert_eq!(summary.epochs[0].julian_day.days(), 2_360_233.5);
        assert_eq!(summary.body_count, comparison_bodies().len());
        assert!(corpus
            .requests
            .iter()
            .all(|request| request.instant.scale == TimeScale::Tt));
        assert!(corpus.requests.iter().all(|request| matches!(
            request.body,
            CelestialBody::Sun
                | CelestialBody::Moon
                | CelestialBody::Mercury
                | CelestialBody::Venus
                | CelestialBody::Mars
                | CelestialBody::Jupiter
                | CelestialBody::Saturn
                | CelestialBody::Uranus
                | CelestialBody::Neptune
                | CelestialBody::Pluto
        )));
        assert!(corpus
            .requests
            .iter()
            .any(|request| request.instant.julian_day.days() == 2_360_233.5));
        assert!(corpus
            .requests
            .iter()
            .any(|request| request.instant.julian_day.days() == 2_451_545.0));
        assert!(corpus
            .requests
            .iter()
            .any(|request| request.instant.julian_day.days() == 2_634_167.0));
        assert_eq!(corpus.requests[0].frame, CoordinateFrame::Ecliptic);
        assert_eq!(corpus.apparentness, Apparentness::Mean);
        assert_eq!(corpus.requests[0].apparent, Apparentness::Mean);
    }

    #[test]
    fn release_grade_corpus_excludes_pluto_from_tolerance_evidence() {
        let corpus = release_grade_corpus();
        let summary = corpus.summary();
        assert!(corpus
            .requests
            .iter()
            .all(|request| request.body != CelestialBody::Pluto));
        assert!(summary.body_count < default_corpus().summary().body_count);
        assert!(summary.request_count < default_corpus().summary().request_count);
        assert!(corpus.name.contains("release-grade comparison window"));
        assert!(corpus
            .description
            .contains("Pluto excluded from tolerance evidence"));
        assert!(corpus
            .description
            .contains("2451913.5 boundary day stays out of the audit slice"));
    }

    #[test]
    fn comparison_report_uses_the_snapshot_backend() {
        let report = render_comparison_report().expect("comparison should render");
        assert_eq!(report.lines().next(), Some("Comparison report"));
        assert!(report.lines().any(|line| line == "Comparison corpus"));
        assert!(report
            .lines()
            .any(|line| line == "  name: JPL Horizons comparison window"));
        assert!(report.lines().any(|line| {
            line == "  description: Source-backed comparison corpus built from the checked-in JPL Horizons snapshot across a small set of reference epochs, restricted to the bodies shared by the algorithmic comparison backend."
        }));
        assert!(report.lines().any(|line| line == "  Apparentness: Mean"));
        assert!(report.lines().any(|line| {
            line == "  epoch labels: JD 2360233.5 (TT), JD 2378499.0 (TT), JD 2400000.0 (TT), JD 2451545.0 (TT), JD 2451910.5 (TT), JD 2451911.5 (TT), JD 2451912.5 (TT), JD 2451914.5 (TT), JD 2451916.5 (TT), JD 2451918.5 (TT), JD 2453000.5 (TT), JD 2500000.0 (TT), JD 2600000.0 (TT), JD 2634167.0 (TT)"
        }));
        assert!(report
            .lines()
            .any(|line| line == "  julian day span: 2360233.5 → 2634167.0"));
        assert!(report
            .lines()
            .any(|line| line == "Reference backend: jpl-snapshot"));
        assert!(report
            .lines()
            .any(|line| line == "Candidate backend: composite:pleiades-vsop87+pleiades-elp"));
    }

    #[test]
    fn request_surface_summary_validation_matches_the_report_line() {
        let summary = RequestSurfaceSummary::current();
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), request_surface_summary_for_report());
        assert_eq!(
            summary.summary_line(),
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        );
        assert_eq!(summary.summary_line().lines().count(), 1);
    }

    #[test]
    fn request_surface_summary_validated_summary_line_rejects_drift() {
        let summary = RequestSurfaceSummary {
            instant: "",
            chart_request: "",
            backend_request: "",
            house_request: "",
            cli_chart: "",
        };

        let error = summary
            .validated_summary_line()
            .expect_err("summary should reject blank request-surface labels");
        assert!(error
            .to_string()
            .contains("primary request surface instant mismatch"));
    }

    #[test]
    fn comparison_tolerance_policy_summary_matches_the_rendered_line() {
        let corpus = release_grade_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let summary = report.tolerance_policy_summary();

        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            summary.summary_line(),
            format_comparison_tolerance_policy_for_report(&report)
        );
        assert_eq!(
            summary
                .validated_summary_line()
                .expect("summary should validate"),
            summary.summary_line()
        );
        assert!(summary.summary_line().contains("frames=Ecliptic"));
        assert_eq!(summary.coverage.len(), summary.entries.len());
        assert_eq!(summary.comparison_body_count, report.body_summaries().len());
        assert!(summary.coverage.iter().any(|coverage| coverage.entry.scope
            == ComparisonToleranceScope::Pluto
            && coverage.body_count == 0
            && coverage.sample_count == 0));
        assert!(summary.coverage.iter().all(|coverage| coverage.entry.scope
            != ComparisonToleranceScope::Pluto
            || coverage.bodies.is_empty()));
        assert_eq!(summary.comparison_sample_count, report.summary.sample_count);
        assert_eq!(
            summary.comparison_window.start,
            corpus.summary().epochs.first().copied()
        );
        assert_eq!(
            summary.comparison_window.end,
            corpus.summary().epochs.last().copied()
        );
    }

    #[test]
    fn comparison_tolerance_policy_summary_validated_summary_line_rejects_drift() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let mut summary = report.tolerance_policy_summary();
        summary.comparison_sample_count += 1;

        let error = summary
            .validated_summary_line()
            .expect_err("summary should reject drifted counts");
        assert!(error.to_string().contains("sample-count mismatch"));
    }

    #[test]
    fn comparison_tolerance_catalog_entries_track_the_backend_family_and_scopes() {
        let entries = comparison_tolerance_catalog_entries(&BackendFamily::Algorithmic);

        assert_eq!(entries.len(), 6);
        assert_eq!(entries[0].scope, ComparisonToleranceScope::Luminary);
        assert_eq!(entries[1].scope, ComparisonToleranceScope::MajorPlanet);
        assert_eq!(entries[2].scope, ComparisonToleranceScope::LunarPoint);
        assert_eq!(entries[3].scope, ComparisonToleranceScope::Asteroid);
        assert_eq!(entries[4].scope, ComparisonToleranceScope::Custom);
        assert_eq!(entries[5].scope, ComparisonToleranceScope::Pluto);
        assert!(entries
            .iter()
            .all(|entry| entry.tolerance.backend_family == BackendFamily::Algorithmic));
    }

    #[test]
    fn comparison_tolerance_catalog_entries_use_body_class_specific_limits() {
        let entries = comparison_tolerance_catalog_entries(&BackendFamily::Composite);

        assert_eq!(
            entries[0].summary_line(),
            "Luminaries: Δlon≤7.500°, Δlat≤0.750°, Δdist=0.001 AU"
        );
        assert_eq!(
            entries[1].summary_line(),
            "Major planets: Δlon≤0.010°, Δlat≤0.010°, Δdist=0.001 AU"
        );
        assert_eq!(
            entries[2].summary_line(),
            "Lunar points: Δlon≤0.100°, Δlat≤0.010°, Δdist=0.001 AU"
        );
        assert_eq!(
            entries[5].summary_line(),
            "Pluto fallback (approximate): Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU"
        );
    }

    #[test]
    fn comparison_tolerance_scope_coverage_summary_validated_summary_line_rejects_body_count_drift()
    {
        let summary = ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 2,
            sample_count: 1,
        };

        let error = summary
            .validated_summary_line()
            .expect_err("summary should reject body-count drift");
        assert!(error.to_string().contains("body-count mismatch"));
    }

    #[test]
    fn comparison_tolerance_entry_has_a_validated_summary_line() {
        let entry = ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        };

        assert_eq!(entry.summary_line(), entry.to_string());
        assert_eq!(entry.validated_summary_line(), Ok(entry.summary_line()));
    }

    #[test]
    fn comparison_tolerance_validation_rejects_blank_profile() {
        let error = validate_comparison_tolerance(&ComparisonTolerance {
            backend_family: BackendFamily::Algorithmic,
            profile: "",
            max_longitude_delta_deg: 0.1,
            max_latitude_delta_deg: 0.2,
            max_distance_delta_au: Some(0.3),
        })
        .expect_err("tolerance should reject a blank profile label");
        assert!(error.to_string().contains("must not be blank"));
    }

    #[test]
    fn comparison_tolerance_scope_coverage_summary_has_a_validated_summary_line() {
        let summary = ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 1,
            sample_count: 1,
        };

        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    }

    #[test]
    fn comparison_tolerance_validation_rejects_padded_profile() {
        let error = validate_comparison_tolerance(&ComparisonTolerance {
            backend_family: BackendFamily::Algorithmic,
            profile: " test tolerance ",
            max_longitude_delta_deg: 0.1,
            max_latitude_delta_deg: 0.2,
            max_distance_delta_au: Some(0.3),
        })
        .expect_err("tolerance should reject a padded profile label");
        assert!(error
            .to_string()
            .contains("contains surrounding whitespace"));
    }

    #[test]
    fn comparison_tolerance_policy_summary_validation_rejects_drift() {
        let summary = ComparisonTolerancePolicySummary {
            backend_family: BackendFamily::Algorithmic,
            entries: vec![ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            }],
            coverage: vec![ComparisonToleranceScopeCoverageSummary {
                entry: ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::Luminary,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.1,
                        max_latitude_delta_deg: 0.2,
                        max_distance_delta_au: Some(0.3),
                    },
                },
                bodies: vec![CelestialBody::Sun],
                body_count: 1,
                sample_count: 1,
            }],
            comparison_body_count: 2,
            comparison_sample_count: 1,
            comparison_window: TimeRange::new(None, None),
            coordinate_frames: vec![CoordinateFrame::Ecliptic],
        };

        let error = summary
            .validate()
            .expect_err("summary should reject body-count drift");
        assert!(error.to_string().contains("body-count mismatch"));
    }

    #[test]
    fn comparison_tolerance_policy_summary_validation_rejects_invalid_comparison_window() {
        let summary = ComparisonTolerancePolicySummary {
            backend_family: BackendFamily::Algorithmic,
            entries: vec![ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            }],
            coverage: vec![ComparisonToleranceScopeCoverageSummary {
                entry: ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::Luminary,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.1,
                        max_latitude_delta_deg: 0.2,
                        max_distance_delta_au: Some(0.3),
                    },
                },
                bodies: vec![CelestialBody::Sun],
                body_count: 1,
                sample_count: 1,
            }],
            comparison_body_count: 1,
            comparison_sample_count: 1,
            comparison_window: TimeRange::new(
                Some(Instant::new(
                    JulianDay::from_days(2_451_546.0),
                    TimeScale::Tt,
                )),
                Some(Instant::new(
                    JulianDay::from_days(2_451_545.0),
                    TimeScale::Tt,
                )),
            ),
            coordinate_frames: vec![CoordinateFrame::Ecliptic],
        };

        let error = summary
            .validate()
            .expect_err("summary should reject an invalid comparison window");
        assert!(error.to_string().contains("invalid comparison window"));
        assert!(error.to_string().contains("must not precede the start"));
    }

    #[test]
    fn comparison_tolerance_policy_summary_validation_rejects_duplicate_coordinate_frames() {
        let summary = ComparisonTolerancePolicySummary {
            backend_family: BackendFamily::Algorithmic,
            entries: vec![ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            }],
            coverage: vec![ComparisonToleranceScopeCoverageSummary {
                entry: ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::Luminary,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.1,
                        max_latitude_delta_deg: 0.2,
                        max_distance_delta_au: Some(0.3),
                    },
                },
                bodies: vec![CelestialBody::Sun],
                body_count: 1,
                sample_count: 1,
            }],
            comparison_body_count: 1,
            comparison_sample_count: 1,
            comparison_window: TimeRange::new(None, None),
            coordinate_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Ecliptic],
        };

        let error = summary
            .validate()
            .expect_err("summary should reject duplicate coordinate frames");
        assert!(error.to_string().contains("duplicate coordinate frame"));
    }

    #[test]
    fn comparison_tolerance_policy_summary_validation_rejects_duplicate_bodies_across_scopes() {
        let summary = ComparisonTolerancePolicySummary {
            backend_family: BackendFamily::Algorithmic,
            entries: vec![
                ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::Luminary,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.1,
                        max_latitude_delta_deg: 0.2,
                        max_distance_delta_au: Some(0.3),
                    },
                },
                ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::MajorPlanet,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.4,
                        max_latitude_delta_deg: 0.5,
                        max_distance_delta_au: Some(0.6),
                    },
                },
            ],
            coverage: vec![
                ComparisonToleranceScopeCoverageSummary {
                    entry: ComparisonToleranceEntry {
                        scope: ComparisonToleranceScope::Luminary,
                        tolerance: ComparisonTolerance {
                            backend_family: BackendFamily::Algorithmic,
                            profile: "test tolerance",
                            max_longitude_delta_deg: 0.1,
                            max_latitude_delta_deg: 0.2,
                            max_distance_delta_au: Some(0.3),
                        },
                    },
                    bodies: vec![CelestialBody::Sun],
                    body_count: 1,
                    sample_count: 1,
                },
                ComparisonToleranceScopeCoverageSummary {
                    entry: ComparisonToleranceEntry {
                        scope: ComparisonToleranceScope::MajorPlanet,
                        tolerance: ComparisonTolerance {
                            backend_family: BackendFamily::Algorithmic,
                            profile: "test tolerance",
                            max_longitude_delta_deg: 0.4,
                            max_latitude_delta_deg: 0.5,
                            max_distance_delta_au: Some(0.6),
                        },
                    },
                    bodies: vec![CelestialBody::Sun],
                    body_count: 1,
                    sample_count: 1,
                },
            ],
            comparison_body_count: 2,
            comparison_sample_count: 2,
            comparison_window: TimeRange::new(None, None),
            coordinate_frames: vec![CoordinateFrame::Ecliptic],
        };

        let error = summary
            .validate()
            .expect_err("summary should reject duplicate bodies across scopes");
        assert!(error.to_string().contains("appears in multiple scope rows"));
        assert!(error.to_string().contains("Luminaries"));
        assert!(error.to_string().contains("Major planets"));
    }

    #[test]
    fn comparison_tolerance_entry_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let summary = report.tolerance_policy_summary();
        let entry = summary
            .entries
            .first()
            .expect("comparison should include at least one tolerance entry");

        assert_eq!(entry.summary_line(), entry.to_string());
        entry
            .validate()
            .expect("reported tolerance entry should validate");
        assert!(entry.summary_line().contains(entry.scope.label()));
        assert!(entry.summary_line().contains("Δlon≤"));
        assert!(entry.summary_line().contains("Δlat≤"));
    }

    #[test]
    fn comparison_tolerance_scope_coverage_summary_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let summary = report.tolerance_policy_summary();
        let coverage = summary
            .coverage
            .first()
            .expect("comparison should include at least one tolerance coverage row");

        assert_eq!(coverage.summary_line(), coverage.to_string());
        assert!(coverage.summary_line().contains("backend family="));
        assert!(coverage
            .summary_line()
            .contains(coverage.entry.scope.label()));
        assert_eq!(coverage.body_count, coverage.bodies.len());
    }

    #[test]
    fn body_tolerance_summary_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let tolerance_summaries = report.tolerance_summaries();
        let summary = tolerance_summaries
            .first()
            .expect("comparison should include at least one tolerance summary");

        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            summary.validated_summary_line().unwrap(),
            summary.summary_line()
        );
        assert!(summary.summary_line().contains("backend family="));
        assert!(summary.summary_line().contains("status="));
    }

    #[test]
    fn body_tolerance_summary_validate_accepts_the_reported_summary() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let tolerance_summaries = report.tolerance_summaries();
        let summary = tolerance_summaries
            .first()
            .expect("comparison should include at least one tolerance summary");

        summary
            .validate()
            .expect("reported body tolerance summary should validate");
    }

    #[test]
    fn body_tolerance_summary_validate_rejects_margin_drift() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let tolerance_summaries = report.tolerance_summaries();
        let mut summary = tolerance_summaries
            .first()
            .expect("comparison should include at least one tolerance summary")
            .clone();

        summary.longitude_margin_deg += 1.0;
        let error = summary
            .validate()
            .expect_err("mutated body tolerance summary should fail validation");
        assert!(error.to_string().contains("longitude margin"));
    }

    #[test]
    fn body_tolerance_summary_validate_rejects_zero_sample_counts() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let tolerance_summaries = report.tolerance_summaries();
        let mut summary = tolerance_summaries
            .first()
            .expect("comparison should include at least one tolerance summary")
            .clone();

        summary.sample_count = 0;
        let error = summary
            .validate()
            .expect_err("zero-sample body tolerance summary should fail validation");
        assert!(error.to_string().contains("has no samples to compare"));
    }

    #[test]
    fn body_tolerance_summary_validated_summary_line_rejects_drift() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let tolerance_summaries = report.tolerance_summaries();
        let mut summary = tolerance_summaries
            .first()
            .expect("comparison should include at least one tolerance summary")
            .clone();

        summary.sample_count = 0;
        let error = summary
            .validated_summary_line()
            .expect_err("zero-sample body tolerance summary should fail validation");
        assert!(error.to_string().contains("has no samples to compare"));
    }

    #[test]
    fn body_tolerance_summary_validate_rejects_non_finite_metrics() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let tolerance_summaries = report.tolerance_summaries();
        let mut summary = tolerance_summaries
            .first()
            .expect("comparison should include at least one tolerance summary")
            .clone();

        summary.max_latitude_delta_deg = f64::NAN;
        let error = summary
            .validate()
            .expect_err("non-finite body tolerance summary should fail validation");
        assert!(error.to_string().contains("has invalid latitude"));
    }

    #[test]
    fn body_comparison_summary_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let body_summaries = report.body_summaries();
        let summary = body_summaries
            .first()
            .expect("comparison should include at least one body summary");

        assert_eq!(summary.summary_line(), summary.to_string());
        assert!(summary.summary_line().contains("samples="));
        assert!(summary.summary_line().contains("max Δlon="));
    }

    #[test]
    fn body_comparison_summary_validate_accepts_the_reported_body_summary() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let body_summaries = report.body_summaries();
        let summary = body_summaries
            .first()
            .expect("comparison should include at least one body summary");

        summary
            .validate()
            .expect("reported body summary should validate");
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    }

    #[test]
    fn body_comparison_summary_validate_rejects_inconsistent_distance_fields() {
        let summary = BodyComparisonSummary {
            body: CelestialBody::Sun,
            sample_count: 1,
            max_longitude_delta_body: Some(CelestialBody::Sun),
            max_longitude_delta_deg: 0.1,
            mean_longitude_delta_deg: 0.1,
            rms_longitude_delta_deg: 0.1,
            max_latitude_delta_body: Some(CelestialBody::Sun),
            max_latitude_delta_deg: 0.1,
            mean_latitude_delta_deg: 0.1,
            rms_latitude_delta_deg: 0.1,
            max_distance_delta_body: Some(CelestialBody::Sun),
            max_distance_delta_au: None,
            mean_distance_delta_au: None,
            rms_distance_delta_au: None,
        };

        let error = summary
            .validate()
            .expect_err("mismatched distance fields should fail");
        assert!(error
            .to_string()
            .contains("distance metrics must either all be present or all be absent"));
    }

    #[test]
    fn comparison_percentile_envelope_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let envelope = comparison_percentile_envelope(&report.samples, 0.95);

        assert_eq!(envelope.summary_line(), envelope.to_string());
        assert!(envelope
            .summary_line()
            .contains("95th percentile absolute deltas:"));
        assert!(envelope.summary_line().contains("longitude"));
        assert!(envelope.summary_line().contains("latitude"));
    }

    #[test]
    fn comparison_median_envelope_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let envelope =
            comparison_median_envelope(&report.samples).expect("median envelope should exist");

        assert_eq!(envelope.summary_line(), envelope.to_string());
        assert!(envelope.summary_line().contains("median longitude delta:"));
        assert!(envelope.summary_line().contains("median latitude delta:"));
    }

    #[test]
    fn comparison_envelope_summary_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let envelope = comparison_envelope_summary(&report.summary, &report.samples);

        assert_eq!(envelope.summary_line(), envelope.to_string());
        assert_eq!(
            envelope.validated_summary_line(&report.samples),
            Ok(envelope.summary_line())
        );
        assert_eq!(
            envelope.percentile_line(),
            comparison_percentile_envelope(&report.samples, 0.95).summary_line()
        );
        assert_eq!(
            envelope.validated_percentile_line(&report.samples),
            Ok(envelope.percentile_line())
        );
        assert!(envelope.summary_line().contains("median longitude delta:"));
        assert!(envelope
            .percentile_line()
            .contains("95th percentile absolute deltas:"));
    }

    #[test]
    fn comparison_envelope_summary_rejects_median_drift() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let mut envelope = comparison_envelope_summary(&report.summary, &report.samples);
        envelope.median.longitude_delta_deg += 0.0001;

        let error = envelope
            .validated_summary_line(&report.samples)
            .expect_err("drifted median should fail validation");
        assert!(error
            .to_string()
            .contains("median drifted from the sampled comparison values"));
    }

    #[test]
    fn comparison_envelope_summary_rejects_percentile_drift() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let mut envelope = comparison_envelope_summary(&report.summary, &report.samples);
        envelope.percentile.longitude_delta_deg += 0.0001;

        let error = envelope
            .validated_percentile_line(&report.samples)
            .expect_err("drifted percentile should fail validation");
        assert!(error
            .to_string()
            .contains("percentile drifted from the sampled comparison values"));
    }

    #[test]
    fn comparison_median_and_percentile_envelopes_validate_their_fields() {
        let median = ComparisonMedianEnvelope {
            longitude_delta_deg: 1.25,
            latitude_delta_deg: 0.75,
            distance_delta_au: Some(0.03125),
        };
        let percentile = ComparisonPercentileEnvelope {
            longitude_delta_deg: 2.5,
            latitude_delta_deg: 1.5,
            distance_delta_au: Some(0.0625),
        };

        assert_eq!(median.validate(), Ok(()));
        assert_eq!(percentile.validate(), Ok(()));
        assert_eq!(median.summary_line(), median.to_string());
        assert_eq!(percentile.summary_line(), percentile.to_string());

        let invalid_median = ComparisonMedianEnvelope {
            longitude_delta_deg: -1.0,
            ..median
        };
        let median_error = invalid_median
            .validate()
            .expect_err("negative median deltas should fail validation");
        assert!(median_error
            .to_string()
            .contains("comparison median envelope field `longitude_delta_deg`"));

        let invalid_percentile = ComparisonPercentileEnvelope {
            distance_delta_au: Some(-0.5),
            ..percentile
        };
        let percentile_error = invalid_percentile
            .validate()
            .expect_err("negative percentile deltas should fail validation");
        assert!(percentile_error
            .to_string()
            .contains("comparison percentile envelope field `distance_delta_au`"));
    }

    #[test]
    fn comparison_tail_envelope_is_publicly_reusable() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let envelope =
            comparison_tail_envelope(&report.samples).expect("tail envelope should exist");

        assert_eq!(envelope.summary_line(), envelope.to_string());
        assert_eq!(
            envelope.summary_line(),
            format_comparison_percentile_envelope_for_report(&report.samples)
        );
    }

    #[test]
    fn regression_finding_has_a_displayable_summary_line() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report =
            compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
        let notable_regressions = report.notable_regressions();
        let finding = notable_regressions
            .first()
            .expect("comparison should include at least one notable regression");

        assert_eq!(finding.summary_line(), finding.to_string());
        assert!(finding.summary_line().contains("Δlon="));
        assert!(finding.summary_line().contains("Δlat="));
        assert!(finding.summary_line().contains("Δdist="));
    }

    #[test]
    fn comparison_audit_summary_has_a_displayable_summary_line() {
        let summary = ComparisonAuditSummary {
            body_count: 10,
            within_tolerance_body_count: 4,
            outside_tolerance_body_count: 6,
            regression_count: 12,
        };

        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            summary.summary_line(),
            "status=regressions found, bodies checked=10, within tolerance bodies=4, outside tolerance bodies=6, notable regressions=12"
        );
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.validate(), Ok(()));
    }

    #[test]
    fn comparison_audit_summary_validate_rejects_body_count_mismatch() {
        let summary = ComparisonAuditSummary {
            body_count: 10,
            within_tolerance_body_count: 4,
            outside_tolerance_body_count: 5,
            regression_count: 0,
        };

        let error = summary
            .validated_summary_line()
            .expect_err("mismatched audit counts should fail validation");
        assert!(error
            .to_string()
            .contains("comparison audit summary body-count mismatch"));
    }

    #[test]
    fn comparison_audit_summary_validate_rejects_empty_body_counts() {
        let summary = ComparisonAuditSummary {
            body_count: 0,
            within_tolerance_body_count: 0,
            outside_tolerance_body_count: 0,
            regression_count: 0,
        };

        let error = summary
            .validate()
            .expect_err("an empty audit summary should fail validation");
        assert!(error
            .to_string()
            .contains("must include at least one compared body"));
    }

    #[test]
    fn comparison_report_exposes_a_public_audit_summary() {
        let corpus = default_corpus();
        let reference = default_reference_backend();
        let candidate = default_candidate_backend();
        let report = compare_backends(&reference, &candidate, &corpus)
            .expect("comparison report should build");
        let summary = report.comparison_audit_summary();

        assert_eq!(summary.summary_line(), summary.to_string());
        assert!(summary.validate().is_ok());
        assert_eq!(summary.body_count, report.tolerance_summaries().len());
        assert_eq!(summary.regression_count, report.notable_regressions().len());
    }

    #[test]
    fn comparison_audit_command_reports_clean_release_grade_corpus() {
        let report =
            render_cli(&["compare-backends-audit"]).expect("comparison audit should render");
        assert!(report.contains("Comparison tolerance audit"));
        assert!(report.contains("comparison corpus"));
        assert!(report.contains("julian day span:"));
        assert!(report.contains("Body-class error envelopes"));
        assert!(report.contains("rms longitude delta:"));
        assert!(report.contains("rms latitude delta:"));
        assert!(report.contains("rms distance delta:"));
        assert!(report.contains("Body-class tolerance posture"));
        assert!(report.contains("Tolerance policy"));
        assert!(report.contains("Notable regressions\n  none"));
        assert!(report.contains("regression bodies: none"));
        assert!(report.contains("Pluto fallback (approximate): backend family=composite, profile=phase-1 Pluto approximate fallback evidence, bodies=0 (none), samples=0"));
    }

    #[test]
    fn benchmark_report_renders_a_time_summary() {
        let report = render_benchmark_report(10).expect("benchmark should render");
        let provenance = workspace_provenance();
        assert_eq!(provenance.validate(), Ok(()));
        assert!(report.contains(&provenance.summary_line()));
        assert!(report.contains("Benchmark report"));
        assert!(report.contains("Summary: backend="));
        assert!(report.contains("Representative 1500-2500 window"));
        assert!(report.contains("Apparentness: Mean"));
        assert!(report.contains("Methodology:"));
        assert!(report.contains("single-request and batch paths are measured separately"));
        assert!(report.contains("chart assembly is measured end to end"));
        assert!(report.contains("Single-request elapsed:"));
        assert!(report.contains("Batch elapsed:"));
        assert!(report.contains("Estimated corpus heap footprint:"));
        assert!(report.contains("Nanoseconds per request (single):"));
        assert!(report.contains("Nanoseconds per request (batch):"));
        assert!(report.contains("Batch throughput:"));
        assert!(report.contains("Artifact decode benchmark report"));
        assert!(report.contains("Encoded bytes:"));
        assert!(report.contains("Nanoseconds per decode:"));
        assert!(report.contains("Decodes per second:"));
        assert!(report.contains("Chart benchmark report"));
        assert!(report.contains("Summary: backend="));
        assert!(report.contains("Representative chart validation scenarios"));
        assert!(report.contains("Chart elapsed:"));
        assert!(report.contains("Nanoseconds per chart:"));
        assert!(report.contains("Charts per second:"));
    }

    #[test]
    fn benchmark_report_summary_line_mentions_the_backend_and_throughput() {
        let corpus = benchmark_corpus();
        let backend = default_candidate_backend();
        let report =
            benchmark_backend(&backend, &corpus, 1).expect("benchmark should produce a report");
        let summary = report.summary_line();
        assert_eq!(report.validated_summary_line(), Ok(summary.clone()));
        assert!(summary.contains("backend="));
        assert!(summary.contains("corpus="));
        assert!(summary.contains("apparentness="));
        assert!(summary.contains("samples per round="));
        assert!(summary.contains("single ns/request="));
        assert!(summary.contains("batch ns/request="));
        assert!(summary.contains("batch throughput="));
        assert!(summary.contains("estimated corpus heap footprint="));
    }

    #[test]
    fn benchmark_backend_rejects_zero_rounds() {
        let corpus = benchmark_corpus();
        let backend = default_candidate_backend();

        let error = benchmark_backend(&backend, &corpus, 0)
            .expect_err("zero-round benchmarks should be rejected");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("benchmark rounds must be greater than zero"));
    }

    #[test]
    fn benchmark_chart_backend_rejects_zero_rounds() {
        let error = benchmark_chart_backend(default_candidate_backend(), 0)
            .expect_err("zero-round chart benchmarks should be rejected");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("chart benchmark rounds must be greater than zero"));
    }

    #[test]
    fn benchmark_backend_rejects_zero_heap_footprint() {
        let corpus = benchmark_corpus();
        let backend = default_candidate_backend();
        let mut report =
            benchmark_backend(&backend, &corpus, 1).expect("benchmark should produce a report");
        report.estimated_corpus_heap_bytes = 0;

        let error = report
            .validate()
            .expect_err("zero-heap benchmarks should be rejected");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("benchmark estimated corpus heap footprint must be greater than zero"));
    }

    #[test]
    fn benchmark_workspace_provenance_display_matches_the_summary_helper() {
        let provenance = workspace_provenance();
        let expected = format!(
            "Benchmark provenance\n  source revision: {}\n  workspace status: {}\n  rustc version: {}",
            provenance.source_revision, provenance.workspace_status, provenance.rustc_version
        );

        assert_eq!(provenance.validate(), Ok(()));
        assert_eq!(provenance.summary_line(), expected);
        assert_eq!(provenance.to_string(), expected);
    }

    #[test]
    fn benchmark_workspace_provenance_validation_rejects_blank_or_multiline_fields() {
        let blank_source_revision = WorkspaceProvenance {
            source_revision: String::new(),
            workspace_status: "clean".to_string(),
            rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        };
        assert_eq!(
            blank_source_revision.validate().unwrap_err(),
            WorkspaceProvenanceValidationError::FieldInvalid {
                field: "source revision"
            }
        );

        let multiline_rustc_version = WorkspaceProvenance {
            source_revision: "abc123def456".to_string(),
            workspace_status: "dirty".to_string(),
            rustc_version: "rustc 1.0.0\nextra".to_string(),
        };
        assert_eq!(
            multiline_rustc_version.validate().unwrap_err(),
            WorkspaceProvenanceValidationError::FieldInvalid {
                field: "rustc version"
            }
        );
    }

    #[test]
    fn validation_report_validate_rejects_drifted_chart_benchmark_corpus() {
        let mut report = build_validation_report(10).expect("validation report should build");
        report.chart_benchmark_corpus.name.clear();

        let error = report
            .validate()
            .expect_err("chart benchmark corpus drift should be rejected");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("validation report chart benchmark corpus is invalid"));
        assert!(error
            .message
            .contains("corpus summary name must not be blank"));
    }

    #[test]
    fn validation_report_display_rejects_drifted_regression_archive() {
        let mut report = build_validation_report(10).expect("validation report should build");
        report.archived_regressions.corpus_name.clear();

        let rendered = report.to_string();
        assert!(rendered.contains("Validation report unavailable"));
        assert!(rendered.contains("regression archive corpus name must not be blank"));
    }

    #[test]
    fn validation_report_includes_corpus_metadata() {
        let report = render_validation_report(10).expect("validation report should render");
        let validation_report =
            build_validation_report(10).expect("validation report should build");
        assert!(report.contains("Validation report"));
        let release_profiles = current_release_profile_identifiers();
        assert!(report.contains("Compatibility profile"));
        assert!(report.contains(&format!(
            "  id: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(report.contains("API stability posture"));
        assert!(report.contains(&format!(
            "  id: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(report.contains(&format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )));
        assert!(report.contains("Implemented backend matrices"));
        assert!(report.contains("Selected asteroid coverage"));
        assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
        assert!(report.contains("exact J2000 evidence: 5 bodies at JD 2451545.0"));
        assert!(report.contains("Ceres"));
        assert!(report.contains("Pallas"));
        assert!(report.contains("Juno"));
        assert!(report.contains("Vesta"));
        assert!(report.contains("asteroid:433-Eros"));
        assert!(report.contains("JPL snapshot reference backend"));
        assert!(report.contains("VSOP87 planetary backend"));
        assert!(report.contains("ELP lunar backend (Moon and lunar nodes)"));
        assert!(report.contains("Packaged data backend"));
        assert!(report.contains("Composite routed backend"));
        assert!(report.contains("Target compatibility catalog:"));
        assert!(report.contains("Comparison corpus"));
        assert!(report.contains("JPL Horizons release-grade comparison window"));
        assert_report_contains_exact_line(
            &report,
            "  Comparison snapshot coverage: 112 rows across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto",
        );
        assert!(report.contains("Apparentness: Mean"));
        assert!(report.contains("Benchmark corpus"));
        assert!(report.contains("Representative 1500-2500 window"));
        assert!(report.contains("estimated corpus heap footprint:"));
        assert!(report.contains("Chart benchmark corpus"));
        assert!(report.contains("Representative chart validation scenarios"));
        assert!(report.contains("Packaged artifact decode benchmark"));
        assert!(report.contains("Chart benchmark"));
        assert!(report.contains("House validation corpus"));
        assert!(report.contains("request: instant="));
        assert!(report.contains("Mid-latitude reference chart"));
        assert!(report.contains("Polar stress chart"));
        assert!(report.contains("Equatorial reference chart"));
        assert!(report.contains("Southern hemisphere reference chart"));
        assert!(report.contains("Reference backend"));
        assert!(report.contains("Candidate backend"));
        assert!(report.contains("Comparison summary"));
        assert!(report.contains("95th percentile absolute deltas:"));
        assert!(report.contains(&format!(
            "max longitude delta: {:.12}° ({})",
            validation_report.comparison.summary.max_longitude_delta_deg,
            validation_report
                .comparison
                .summary
                .max_longitude_delta_body
                .as_ref()
                .expect("comparison summary should include a max longitude body")
        )));
        assert!(report.contains(&format!(
            "max latitude delta: {:.12}° ({})",
            validation_report.comparison.summary.max_latitude_delta_deg,
            validation_report
                .comparison
                .summary
                .max_latitude_delta_body
                .as_ref()
                .expect("comparison summary should include a max latitude body")
        )));
        assert!(report.contains(&format!(
            "max distance delta: {:.12} AU ({})",
            validation_report
                .comparison
                .summary
                .max_distance_delta_au
                .expect("comparison summary should include a max distance delta"),
            validation_report
                .comparison
                .summary
                .max_distance_delta_body
                .as_ref()
                .expect("comparison summary should include a max distance body")
        )));
        assert!(report.contains("Body-class error envelopes"));
        let body_class_envelopes = report
            .split("Body-class error envelopes")
            .nth(1)
            .expect("report should include body-class error envelopes");
        assert!(body_class_envelopes.contains("max longitude delta:"));
        assert!(body_class_envelopes.contains("median longitude delta:"));
        assert!(body_class_envelopes.contains("95th percentile longitude delta:"));
        assert!(body_class_envelopes.contains("median latitude delta:"));
        assert!(body_class_envelopes.contains("95th percentile latitude delta:"));
        assert!(body_class_envelopes.contains("rms longitude delta:"));
        assert!(body_class_envelopes.contains("rms latitude delta:"));
        assert!(body_class_envelopes.contains("median distance delta:"));
        assert!(body_class_envelopes.contains("95th percentile distance delta:"));
        assert!(body_class_envelopes.contains("rms distance delta:"));
        assert!(body_class_envelopes.contains(" ("));
        assert!(report.contains("Body-class tolerance posture"));
        let body_class_tolerance_posture = report
            .split("Body-class tolerance posture")
            .nth(1)
            .expect("report should include body-class tolerance posture");
        assert!(body_class_tolerance_posture.contains("backend family: composite"));
        assert!(body_class_tolerance_posture
            .contains("profile: phase-1 full-file VSOP87B planetary evidence"));
        assert!(body_class_tolerance_posture.contains("mean longitude delta:"));
        assert!(body_class_tolerance_posture.contains("median longitude delta:"));
        assert!(body_class_tolerance_posture.contains("95th percentile longitude delta:"));
        assert!(body_class_tolerance_posture.contains("rms longitude delta:"));
        assert!(body_class_tolerance_posture.contains("mean latitude delta:"));
        assert!(body_class_tolerance_posture.contains("median latitude delta:"));
        assert!(body_class_tolerance_posture.contains("95th percentile latitude delta:"));
        assert!(body_class_tolerance_posture.contains("rms latitude delta:"));
        assert!(body_class_tolerance_posture.contains("mean distance delta:"));
        assert!(body_class_tolerance_posture.contains("median distance delta:"));
        assert!(body_class_tolerance_posture.contains("95th percentile distance delta:"));
        assert!(body_class_tolerance_posture.contains("rms distance delta:"));
        assert!(body_class_tolerance_posture.contains("outside tolerance samples:"));
        assert!(report.contains("Expected tolerance status"));
        assert!(report.contains("margin Δlon="));
        assert!(report.contains("margin Δdist="));
        assert!(report.contains("Tolerance policy"));
        assert!(report.contains("candidate backend family: composite"));
        assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
        assert!(report.contains(
            "Pluto fallback (approximate): backend family=composite, profile=phase-1 Pluto approximate fallback evidence, bodies=0 (none), samples=0"
        ));
        assert!(report.contains("Luminaries"));
        assert!(report.contains("Major planets"));
        assert!(report.contains("interpolation quality checks:"));
        assert!(report.contains("JPL interpolation quality: 102 samples across 10 bodies"));
        assert!(report.contains("JPL interpolation quality kind coverage:"));
        assert!(report.contains("JPL interpolation posture: source="));
        assert!(report.contains("Reference/hold-out overlap:"));
        assert!(report.contains("JPL independent hold-out:"));
        assert!(report.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
        assert!(report.contains("JPL independent hold-out equatorial parity:"));
        assert!(report.contains("JPL independent hold-out batch parity:"));
        assert!(report.contains("Lunar reference"));
        assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5 (TT) → JD 2459278.5 (TT)"
        ));
        assert!(report.contains(
            "lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies, TT requests=5, TDB requests=4, order=preserved, single-query parity=preserved"
        ));
        assert!(report.contains(
            "lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"
        ));
        assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
        assert!(report.contains("exact J2000 evidence: 5 bodies at JD 2451545.0"));
        assert!(report.contains("Body comparison summaries"));
        assert!(report.contains("Sun: samples="));
        assert!(report.contains("Notable regressions"));
        assert!(report.contains("Archived regression cases"));
        assert!(report.contains("Reference benchmark"));
        assert!(report.contains("Candidate benchmark"));
        assert!(report.contains("Packaged-data benchmark corpus"));
        assert!(report.contains("Packaged-data benchmark"));
        assert!(report.contains("ns/request (single):"));
        assert!(report.contains("ns/request (batch):"));
        assert!(report.contains("batch throughput:"));
    }

    #[test]
    fn validation_report_summary_renders_a_compact_overview() {
        let report =
            render_validation_report_summary(10).expect("validation summary should render");
        let validation_report =
            build_validation_report(10).expect("validation report should build");
        let release_profiles = current_release_profile_identifiers();
        assert!(report.contains("Validation report summary"));
        assert!(report.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(report.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(report.contains("Comparison corpus"));
        assert!(report.contains(&request_surface_summary_for_report()));
        assert!(report.contains("Reference snapshot"));
        assert!(report.contains(&reference_snapshot_summary_for_report()));
        assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(report.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
        assert!(report.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"));
        assert!(report.contains("epoch labels: JD 2360233.5 (TT)"));
        assert!(report.contains("House validation corpus"));
        assert!(report.contains("House validation corpus: 5 scenarios"));
        assert!(report.contains("Comparison summary"));
        assert!(report.contains("95th percentile absolute deltas:"));
        assert!(report.contains("median longitude delta:"));
        assert!(report.contains("median latitude delta:"));
        assert!(report.contains("rms longitude delta:"));
        assert!(report.contains("rms latitude delta:"));
        assert!(report.contains("rms distance delta:"));
        assert!(report.contains("Tolerance policy"));
        assert!(report.contains("candidate backend family: composite"));
        assert!(report.contains("comparison window: JD"));
        assert!(report.contains("coordinate frames: Ecliptic"));
        assert!(report.contains(&format!(
            "max longitude delta: {:.12}° ({})",
            validation_report.comparison.summary.max_longitude_delta_deg,
            validation_report
                .comparison
                .summary
                .max_longitude_delta_body
                .as_ref()
                .expect("comparison summary should include a max longitude body")
        )));
        assert!(report.contains(&format!(
            "max latitude delta: {:.12}° ({})",
            validation_report.comparison.summary.max_latitude_delta_deg,
            validation_report
                .comparison
                .summary
                .max_latitude_delta_body
                .as_ref()
                .expect("comparison summary should include a max latitude body")
        )));
        assert!(report.contains(&format!(
            "max distance delta: {:.12} AU ({})",
            validation_report
                .comparison
                .summary
                .max_distance_delta_au
                .expect("comparison summary should include a max distance delta"),
            validation_report
                .comparison
                .summary
                .max_distance_delta_body
                .as_ref()
                .expect("comparison summary should include a max distance body")
        )));
        assert!(report.contains("Tolerance policy"));
        assert!(report.contains("candidate backend family: composite"));
        assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
        assert!(report.contains("Comparison tolerance audit"));
        assert!(report.contains("command: compare-backends-audit"));
        assert!(report.contains("status: clean"));
        assert!(!report.contains("regression bodies: Pluto"));
        let body_class_tolerance_posture = report
            .split("Body-class tolerance posture")
            .nth(1)
            .expect("report should include body-class tolerance posture");
        assert!(body_class_tolerance_posture.contains("mean Δlon="));
        assert!(body_class_tolerance_posture.contains("rms Δlon="));
        assert!(body_class_tolerance_posture.contains("mean Δlat="));
        assert!(body_class_tolerance_posture.contains("rms Δlat="));
        assert!(body_class_tolerance_posture.contains("mean Δdist="));
        assert!(body_class_tolerance_posture.contains("rms Δdist="));
        assert!(report.contains("JPL interpolation quality"));
        assert!(report.contains("JPL interpolation quality: 102 samples across 10 bodies"));
        assert!(report.contains("Reference/hold-out overlap:"));
        assert!(report.contains("JPL independent hold-out:"));
        assert!(report.contains("JPL independent hold-out equatorial parity:"));
        assert!(report.contains("JPL independent hold-out batch parity:"));
        assert!(report.contains("leave-one-out runtime interpolation evidence"));
        assert!(report.contains("@ JD"));
        assert!(report.contains("ELP lunar capability: lunar capability summary:"));
        assert!(report.contains(
            "ELP lunar request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        ));
        assert!(report.contains("Lunar reference"));
        assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5 (TT) → JD 2459278.5 (TT)"
        ));
        assert!(report.contains("Lunar source windows\n  lunar source windows: 7 exact Moon samples across 1 bodies in 2 exact windows; 4 reference-only apparent Moon samples across 1 bodies in 4 apparent windows"));
        assert!(report.contains("Lunar high-curvature continuity evidence"));
        assert!(report.contains("Lunar high-curvature equatorial continuity evidence"));
        assert!(report.contains("within regression limits=true"));
        assert!(report.contains("Body comparison summaries"));
        assert!(report.contains("Body-class error envelopes"));
        assert!(report.contains("max longitude delta:"));
        assert!(report.contains("rms longitude delta:"));
        assert!(report.contains("rms latitude delta:"));
        assert!(report.contains("rms distance delta:"));
        let body_class_tolerance_posture = report
            .split("Body-class tolerance posture")
            .nth(1)
            .expect("report should include body-class tolerance posture");
        assert!(body_class_tolerance_posture.contains("mean Δlon="));
        assert!(body_class_tolerance_posture.contains("rms Δlon="));
        assert!(body_class_tolerance_posture.contains("mean Δlat="));
        assert!(body_class_tolerance_posture.contains("rms Δlat="));
        assert!(body_class_tolerance_posture.contains("mean Δdist="));
        assert!(body_class_tolerance_posture.contains("rms Δdist="));
        assert!(report.contains(" ("));
        assert!(report.contains("VSOP87 source-backed evidence"));
        assert!(report
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
        assert!(report
            .contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files"));
        assert!(report.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(report.contains("VSOP87 canonical J2000 interim outliers: none"));
        assert!(report.contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
        assert!(report.contains("generated binary VSOP87B"));
        assert!(report.contains("generated binary VSOP87B; VSOP87B."));
        assert!(report.contains("max Δlon="));
        assert!(report.contains("max Δlat="));
        assert!(report.contains("max Δdist="));
        assert!(report.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
        assert!(report.contains("House validation corpus"));
        assert!(report.contains("Benchmark provenance"));
        assert!(report.contains("source revision:"));
        assert!(report.contains("workspace status:"));
        assert!(report.contains("rustc version:"));
        assert!(report.contains("Benchmark summaries"));
        assert!(report.contains("Release bundle verification: verify-release-bundle"));
        assert!(report.contains("Workspace audit: workspace-audit / audit"));
        assert!(report.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(report.contains("Release notes summary: release-notes-summary"));
        assert!(report.contains("Release checklist summary: release-checklist-summary"));
        assert!(report.contains("Release summary: release-summary"));
        assert!(report.contains("Reference benchmark"));
        assert!(report.contains("Candidate benchmark"));
        assert!(report.contains("Packaged-data benchmark"));
        assert!(report.contains("Packaged artifact decode benchmark"));
        assert!(report.contains("Chart benchmark"));
    }

    #[test]
    fn jpl_interpolation_quality_summary_includes_worst_case_body_labels() {
        let summary = jpl_interpolation_quality_summary().expect("summary should exist");
        assert!(!summary.max_bracket_span_body.is_empty());
        assert!(!summary.max_longitude_error_body.is_empty());
        assert!(!summary.max_latitude_error_body.is_empty());
        assert!(!summary.max_distance_error_body.is_empty());

        let rendered = format_jpl_interpolation_quality_summary(&summary);
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_bracket_span_body,
            format_instant(summary.max_bracket_span_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_longitude_error_body,
            format_instant(summary.max_longitude_error_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_latitude_error_body,
            format_instant(summary.max_latitude_error_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_distance_error_body,
            format_instant(summary.max_distance_error_epoch)
        )));
    }

    #[test]
    fn cli_report_summary_lists_the_summary_command() {
        let rendered = render_cli(&["report-summary", "--rounds", "10"])
            .expect("report summary should render");
        assert!(rendered.contains("Validation report summary"));
        assert!(rendered.contains("Comparison corpus"));
        assert!(rendered.contains("Reference snapshot"));
        assert!(rendered.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
        assert_report_contains_exact_line(
            &rendered,
            "  Comparison snapshot coverage: 112 rows across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto",
        );
        assert!(rendered.contains("Body comparison summaries"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(rendered.contains("Packaged-artifact profile"));
        assert!(rendered.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported"
        ));
        assert!(rendered.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-prototype; target thresholds: prototype fit envelope recorded; scopes=luminaries, major planets, lunar points, selected asteroids, custom bodies; fit envelope:"));
        assert!(rendered.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2; fit envelope:"));
        assert!(rendered.contains(
            "Packaged-artifact generation manifest: Packaged artifact generation manifest:"
        ));
        assert!(rendered.contains("Packaged batch parity:"));
        assert!(rendered.contains("Packaged frame parity:"));
        assert!(rendered.contains("Benchmark provenance"));
        assert!(rendered.contains("source revision:"));
        assert!(rendered.contains("workspace status:"));
        assert!(rendered.contains("rustc version:"));
        assert!(rendered.contains("Benchmark summaries"));
        assert!(rendered.contains("Packaged artifact decode benchmark"));

        let artifact_profile_coverage = render_cli(&["artifact-profile-coverage-summary"])
            .expect("artifact-profile-coverage-summary should render");
        assert_eq!(
            artifact_profile_coverage,
            format!(
                "Artifact profile coverage: {}",
                packaged_artifact_profile_coverage_summary_for_report()
            )
        );

        let validation_report_summary =
            render_cli(&["validation-report-summary", "--rounds", "10"])
                .expect("validation-report-summary should render");
        assert!(validation_report_summary.contains("Validation report summary"));
        assert!(validation_report_summary.contains("Comparison corpus"));
        assert!(validation_report_summary.contains("Reference snapshot"));
        assert!(validation_report_summary.contains(&reference_snapshot_summary_for_report()));
        assert!(validation_report_summary
            .contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(validation_report_summary
            .contains(&reference_snapshot_body_class_coverage_summary_for_report()));
        assert!(validation_report_summary.contains("Body comparison summaries"));
        assert!(validation_report_summary.contains("Body-class error envelopes"));
        assert!(validation_report_summary.contains("Body-class tolerance posture"));
        assert!(validation_report_summary.contains("Expected tolerance status"));
        assert!(validation_report_summary.contains("margin Δlon="));
        assert!(validation_report_summary.contains("margin Δdist="));
        assert!(validation_report_summary.contains("Chart benchmark"));
        assert!(validation_report_summary.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
        assert!(validation_report_summary.contains("Comparison tolerance audit"));
        assert!(validation_report_summary.contains("command: compare-backends-audit"));
        assert!(validation_report_summary.contains("status: clean"));
        assert!(validation_report_summary.contains("regression bodies:"));
        assert!(!validation_report_summary.contains("regression bodies: Pluto"));
        assert!(validation_report_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(validation_report_summary.contains("ayanamsa catalog validation: ok"));
        assert!(validation_report_summary.contains("Release notes summary: release-notes-summary"));
        assert!(validation_report_summary.contains("Packaged-artifact profile"));
        assert!(validation_report_summary.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported"
        ));
        assert!(validation_report_summary.contains(
            "Packaged-artifact storage/reconstruction: Quantized linear segments stored in pleiades-compression artifact format; equatorial coordinates are reconstructed at runtime from stored channels"
        ));
        assert!(validation_report_summary.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-prototype; target thresholds: prototype fit envelope recorded; scopes=luminaries, major planets, lunar points, selected asteroids, custom bodies; fit envelope:"));
        assert!(validation_report_summary.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2; fit envelope:"));
        assert!(validation_report_summary.contains(
            "Packaged-artifact generation manifest: Packaged artifact generation manifest:"
        ));
        assert_report_contains_exact_line(
            &validation_report_summary,
            &format!(
                "Packaged-artifact generation policy: {}",
                packaged_artifact_generation_policy_summary_for_report()
            ),
        );
        assert_report_contains_exact_line(
            &validation_report_summary,
            &format!(
                "Packaged-artifact generation residual bodies: {}",
                packaged_artifact_generation_residual_bodies_summary_for_report()
            ),
        );
        assert!(validation_report_summary.contains("Packaged request policy"));
        assert!(validation_report_summary
            .contains(&reference_snapshot_major_body_boundary_summary_for_report()));
        assert!(validation_report_summary
            .contains(&reference_snapshot_mars_jupiter_boundary_summary_for_report()));
        assert!(validation_report_summary
            .contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
        assert_report_contains_exact_line(
            &validation_report_summary,
            "Packaged lookup epoch policy: TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
        );
        assert!(validation_report_summary.contains("Packaged frame parity"));
        assert!(validation_report_summary.lines().any(|line| {
            line == format!(
                "  Packaged frame treatment: {}",
                packaged_frame_treatment_summary_for_report()
            )
        }));
        assert!(validation_report_summary.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        }));
        assert!(validation_report_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
        assert!(validation_report_summary.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        }));
        assert!(validation_report_summary.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
        assert!(validation_report_summary.contains("Frame policy:"));
        let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");
        assert!(validation_report_summary.lines().any(|line| {
            line == format!(
                "Mean-obliquity frame round-trip: {}",
                mean_obliquity_frame_round_trip
            )
        }));
        assert!(validation_report_summary.contains("Zodiac policy:"));
        assert!(validation_report_summary.contains(
            "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.6.123, api-stability=pleiades-api-stability/0.1.0"
        ));
        assert!(validation_report_summary
            .contains("lookup epoch policy=TT-grid retag without relativistic correction"));
        assert!(validation_report_summary.contains("Benchmark summaries"));
        assert!(validation_report_summary.contains("Packaged artifact decode benchmark"));
    }

    #[test]
    fn cli_rejects_zero_rounds_for_benchmark_and_bundle_release() {
        let benchmark_error = render_cli(&["benchmark", "--rounds", "0"])
            .expect_err("benchmark should reject zero rounds");
        assert!(benchmark_error.contains("invalid value for --rounds: expected a positive integer"));

        let bundle_dir = unique_temp_dir("pleiades-release-bundle-zero-rounds-command");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        let bundle_error = render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "0",
        ])
        .expect_err("bundle-release should reject zero rounds");
        assert!(bundle_error.contains("invalid value for --rounds: expected a positive integer"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn benchmark_corpus_spans_the_target_window() {
        let corpus = benchmark_corpus();
        let summary = corpus.summary();
        assert_eq!(summary.epoch_count, 11);
        assert_eq!(summary.body_count, default_chart_bodies().len());
        assert_eq!(summary.request_count, 110);
        assert_eq!(summary.apparentness, Apparentness::Mean);
        assert!(summary.earliest_julian_day < summary.latest_julian_day);
    }

    #[test]
    fn corpus_summary_has_a_displayable_summary_line() {
        let summary = benchmark_corpus().summary();
        assert_eq!(summary.summary_line(), summary.to_string());
        assert!(summary
            .summary_line()
            .contains("corpus name=Representative 1500-2500 window"));
        assert!(summary.summary_line().contains("requests=110"));
        assert!(summary.summary_line().contains("epochs=11"));
        assert!(summary.summary_line().contains("bodies="));
    }

    #[test]
    fn corpus_summary_validate_accepts_the_reported_summary() {
        let summary = benchmark_corpus().summary();
        summary
            .validate()
            .expect("reported corpus summary should validate");
    }

    #[test]
    fn corpus_summary_validate_rejects_epoch_order_drift() {
        let mut summary = benchmark_corpus().summary();
        summary.epochs.reverse();
        let error = summary
            .validate()
            .expect_err("mutated corpus summary should fail validation");
        assert!(
            error.to_string().contains("out of order") || error.to_string().contains("first epoch")
        );
    }

    #[test]
    fn packaged_benchmark_corpus_uses_packaged_artifact_coverage() {
        let corpus = artifact::packaged_artifact_corpus();
        let summary = corpus.summary();
        assert!(summary.name.contains("Packaged artifact"));
        assert_eq!(summary.apparentness, Apparentness::Mean);
        assert_eq!(
            summary.body_count,
            pleiades_data::packaged_artifact().bodies.len()
        );
        assert!(summary.request_count > 0);
        assert!(summary.earliest_julian_day <= summary.latest_julian_day);
    }

    #[test]
    fn house_validation_report_includes_representative_scenarios() {
        let report = house_validation_report();
        assert_eq!(report.scenarios.len(), 5);
        assert!(report
            .scenarios
            .iter()
            .any(|scenario| scenario.label == "Southern polar stress chart"));
        assert!(report
            .scenarios
            .iter()
            .any(|scenario| scenario.label == "Southern hemisphere reference chart"));
    }

    #[test]
    fn comparison_report_uses_release_grade_corpus_without_pluto() {
        let corpus = release_grade_corpus();
        let report = compare_backends(
            &default_reference_backend(),
            &default_candidate_backend(),
            &corpus,
        )
        .expect("comparison should succeed");

        let regressions = report.notable_regressions();
        assert!(regressions.is_empty());

        let body_summaries = report.body_summaries();
        assert_eq!(body_summaries.len(), report.tolerance_summaries().len());
        assert!(body_summaries
            .iter()
            .all(|summary| summary.sample_count > 0 && summary.body != CelestialBody::Pluto));
        assert!(body_summaries
            .iter()
            .any(|summary| summary.body == CelestialBody::Jupiter
                && summary.max_longitude_delta_deg > 0.0
                && summary.max_longitude_delta_deg < 0.01));

        let archive = report.regression_archive();
        assert_eq!(archive.corpus_name, corpus.name);
        assert!(archive.cases.is_empty());
        let tolerance_summaries = report.tolerance_summaries();
        assert!(tolerance_summaries.iter().any(|summary| {
            summary.body == CelestialBody::Jupiter
                && summary.tolerance.profile.contains("full-file VSOP87B")
        }));
        assert!(tolerance_summaries
            .iter()
            .all(|summary| summary.body != CelestialBody::Pluto));
        let tolerance_policy_entries = report.tolerance_policy_entries();
        assert_eq!(tolerance_policy_entries.len(), 6);
        assert!(tolerance_policy_entries
            .iter()
            .any(|entry| entry.scope == ComparisonToleranceScope::Luminary));
        assert!(tolerance_policy_entries
            .iter()
            .any(|entry| entry.scope == ComparisonToleranceScope::Pluto));
        let body_class_tolerance_summaries = report.body_class_tolerance_summaries();
        assert!(body_class_tolerance_summaries.iter().any(|summary| {
            summary.class == BodyClass::MajorPlanet
                && summary.body_count >= 1
                && summary.sample_count >= summary.body_count
                && summary.outside_tolerance_body_count == 0
                && !summary.outside_bodies.contains(&CelestialBody::Pluto)
                && summary.max_longitude_delta_body.is_some()
                && summary.max_latitude_delta_body.is_some()
                && summary.max_distance_delta_body.is_some()
        }));

        let rendered = report.to_string();
        assert!(rendered.contains("Body comparison summaries"));
        assert!(rendered.contains("Body-class tolerance posture"));
        assert!(rendered.contains("Expected tolerance status"));
        assert!(rendered.contains("phase-1 full-file VSOP87B planetary evidence"));
        assert!(rendered.contains("Notable regressions"));
    }

    #[test]
    fn body_class_summary_line_reuses_the_typed_formatter() {
        let summary = BodyClassSummary {
            class: BodyClass::MajorPlanet,
            sample_count: 1,
            max_longitude_delta_body: Some(CelestialBody::Mars),
            max_longitude_delta_deg: 1.234,
            sum_longitude_delta_deg: 1.0,
            sum_longitude_delta_sq_deg: 1.0,
            median_longitude_delta_deg: 1.0,
            percentile_longitude_delta_deg: 1.0,
            max_latitude_delta_body: Some(CelestialBody::Mars),
            max_latitude_delta_deg: 0.5,
            sum_latitude_delta_deg: 0.5,
            sum_latitude_delta_sq_deg: 0.25,
            median_latitude_delta_deg: 0.5,
            percentile_latitude_delta_deg: 0.5,
            max_distance_delta_body: Some(CelestialBody::Mars),
            max_distance_delta_au: Some(2.5),
            sum_distance_delta_au: 2.5,
            sum_distance_delta_sq_au: 6.25,
            distance_count: 1,
            median_distance_delta_au: Some(2.5),
            percentile_distance_delta_au: Some(2.5),
        };

        let expected = "samples=1, max Δlon=1.234000000000° (Mars), mean Δlon=1.000000000000°, median Δlon=1.000000000000°, 95th percentile longitude delta: 1.000000000000°, rms Δlon=1.000000000000°, max Δlat=0.500000000000° (Mars), mean Δlat=0.500000000000°, median Δlat=0.500000000000°, 95th percentile latitude delta: 0.500000000000°, rms Δlat=0.500000000000°, max Δdist=2.500000000000 AU (Mars), mean Δdist=2.500000000000 AU, median Δdist=2.500000000000 AU, 95th percentile distance delta: 2.500000000000 AU, rms Δdist=2.500000000000 AU";

        assert_eq!(summary.summary_line(), expected);
        assert_eq!(summary.to_string(), expected);
        assert_eq!(
            format_body_class_comparison_envelope_for_report(&summary),
            expected
        );
    }

    #[test]
    fn body_class_tolerance_summary_reuses_the_typed_formatter() {
        let summary = BodyClassToleranceSummary {
            class: BodyClass::MajorPlanet,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::ReferenceData,
                profile: "phase-1 body-class tolerance",
                max_longitude_delta_deg: 1.5,
                max_latitude_delta_deg: 0.5,
                max_distance_delta_au: Some(3.0),
            },
            body_count: 2,
            sample_count: 2,
            within_tolerance_body_count: 1,
            outside_tolerance_body_count: 1,
            outside_tolerance_sample_count: 1,
            max_longitude_delta_body: Some(CelestialBody::Mars),
            max_longitude_delta_deg: Some(1.0),
            max_latitude_delta_body: Some(CelestialBody::Jupiter),
            max_latitude_delta_deg: Some(0.25),
            max_distance_delta_body: Some(CelestialBody::Saturn),
            max_distance_delta_au: Some(2.5),
            sum_longitude_delta_deg: 2.0,
            sum_longitude_delta_sq_deg: 2.0,
            sum_latitude_delta_deg: 0.5,
            sum_latitude_delta_sq_deg: 0.125,
            sum_distance_delta_au: 3.0,
            sum_distance_delta_sq_au: 4.5,
            distance_count: 2,
            median_longitude_delta_deg: 1.0,
            percentile_longitude_delta_deg: 1.0,
            median_latitude_delta_deg: 0.25,
            percentile_latitude_delta_deg: 0.25,
            median_distance_delta_au: Some(1.5),
            percentile_distance_delta_au: Some(1.5),
            outside_bodies: vec![CelestialBody::Mars],
        };

        let expected = "Major planets: backend family=reference data, profile=phase-1 body-class tolerance, bodies=2, samples=2, within tolerance bodies=1, outside tolerance bodies=1, limit Δlon≤1.500000°, margin Δlon=+0.500000000000°, limit Δlat≤0.500000°, margin Δlat=+0.250000000000°, limit Δdist=3.000000 AU, margin Δdist=+0.500000000000 AU, max Δlon=1.000000000000° (Mars), max Δlat=0.250000000000° (Jupiter), max Δdist=2.500000000000 AU (Saturn)";

        assert!(summary.validate().is_ok());
        assert_eq!(summary.summary_line(), expected);
        assert_eq!(summary.to_string(), expected);
        assert_eq!(
            format_body_class_tolerance_envelope_for_report(&summary),
            expected
        );
    }

    #[test]
    fn body_class_tolerance_summary_rejects_count_drift() {
        let summary = BodyClassToleranceSummary {
            class: BodyClass::MajorPlanet,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::ReferenceData,
                profile: "phase-1 body-class tolerance",
                max_longitude_delta_deg: 1.5,
                max_latitude_delta_deg: 0.5,
                max_distance_delta_au: Some(3.0),
            },
            body_count: 1,
            sample_count: 1,
            within_tolerance_body_count: 1,
            outside_tolerance_body_count: 0,
            outside_tolerance_sample_count: 0,
            max_longitude_delta_body: Some(CelestialBody::Mars),
            max_longitude_delta_deg: Some(1.0),
            max_latitude_delta_body: Some(CelestialBody::Jupiter),
            max_latitude_delta_deg: Some(0.25),
            max_distance_delta_body: Some(CelestialBody::Saturn),
            max_distance_delta_au: Some(2.5),
            sum_longitude_delta_deg: 1.0,
            sum_longitude_delta_sq_deg: 1.0,
            sum_latitude_delta_deg: 0.25,
            sum_latitude_delta_sq_deg: 0.0625,
            sum_distance_delta_au: 2.5,
            sum_distance_delta_sq_au: 6.25,
            distance_count: 1,
            median_longitude_delta_deg: 1.0,
            percentile_longitude_delta_deg: 1.0,
            median_latitude_delta_deg: 0.25,
            percentile_latitude_delta_deg: 0.25,
            median_distance_delta_au: Some(2.5),
            percentile_distance_delta_au: Some(2.5),
            outside_bodies: vec![CelestialBody::Mars, CelestialBody::Jupiter],
        };

        let rendered = format_body_class_tolerance_envelope_for_report(&summary);
        assert!(rendered.contains("body-class tolerance envelope unavailable"));
        assert!(summary.validate().is_err());
    }

    #[test]
    fn cli_help_lists_the_validation_commands() {
        let rendered = render_cli(&["help"]).expect("help should render");
        assert!(rendered.contains("compare-backends"));
        assert!(rendered.contains("compare-backends-audit"));
        assert!(rendered.contains("backend-matrix"));
        assert!(rendered.contains("capability-matrix"));
        assert!(rendered.contains("backend-matrix-summary"));
        assert!(rendered.contains("matrix-summary"));
        assert!(rendered.contains("compatibility-profile"));
        assert!(rendered.contains("Alias for compatibility-profile"));
        assert!(rendered.contains("benchmark [--rounds N]"));
        assert!(rendered.contains("comparison-corpus-summary"));
        assert!(rendered.contains("comparison-corpus-release-guard-summary"));
        assert!(rendered.contains("comparison-corpus-guard-summary"));
        assert!(rendered.contains("benchmark-corpus-summary"));
        assert!(rendered.contains("report [--rounds N]"));
        assert!(rendered.contains("generate-report"));
        assert!(rendered.contains("validation-report-summary [--rounds N]"));
        assert!(rendered.contains("report-summary [--rounds N]"));
        assert!(rendered.contains("validation-summary"));
        assert!(rendered.contains("validate-artifact"));
        assert!(rendered.contains("regenerate-packaged-artifact"));
        assert!(rendered.contains("artifact-summary"));
        assert!(rendered.contains("artifact-boundary-envelope-summary"));
        assert!(rendered.contains("packaged-artifact-output-support-summary"));
        assert!(rendered.contains("packaged-artifact-storage-summary"));
        assert!(rendered.contains("packaged-artifact-production-profile-summary"));
        assert!(rendered.contains("packaged-artifact-target-threshold-summary"));
        assert!(rendered.contains("packaged-artifact-target-threshold-scope-envelopes-summary"));
        assert!(rendered.contains("packaged-artifact-generation-manifest-summary"));
        assert!(rendered.contains("packaged-artifact-generation-policy-summary"));
        assert!(rendered.contains("packaged-artifact-generation-residual-bodies-summary"));
        assert!(rendered.contains("packaged-artifact-regeneration-summary"));
        assert!(rendered.contains("workspace-audit"));
        assert!(rendered.contains("native-dependency-audit"));
        assert!(rendered.contains("native-dependency-audit-summary"));
        assert!(rendered.contains("api-stability"));
        assert!(rendered.contains("api-stability-summary"));
        assert!(rendered.contains("compatibility-profile-summary"));
        assert!(rendered.contains("house-formula-families-summary"));
        assert!(rendered.contains("house-code-aliases-summary"));
        assert!(rendered.contains("house-code-alias-summary"));
        assert!(rendered.contains("delta-t-policy-summary"));
        assert!(rendered.contains("observer-policy-summary"));
        assert!(rendered.contains("apparentness-policy-summary"));
        assert!(rendered.contains("production-generation-boundary-summary"));
        assert!(rendered.contains("production-generation-boundary-request-corpus-summary"));
        assert!(rendered.contains("production-generation-body-class-coverage-summary"));
        assert!(rendered.contains("production-generation-source-window-summary"));
        assert!(rendered.contains("production-generation-summary"));
        assert!(rendered.contains("production-generation-boundary-source-summary"));
        assert!(rendered.contains("production-generation-boundary-window-summary"));
        assert!(rendered.contains("production-generation-source-summary"));
        assert!(rendered.contains("comparison-snapshot-source-window-summary"));
        assert!(rendered.contains("comparison-snapshot-source-summary"));
        assert!(rendered.contains("comparison-snapshot-body-class-coverage-summary"));
        assert!(rendered.contains("comparison-snapshot-manifest-summary"));
        assert!(rendered.contains("comparison-snapshot-summary"));
        assert!(rendered.contains("comparison-snapshot-batch-parity-summary"));
        assert!(rendered.contains("reference-snapshot-source-window-summary"));
        assert!(rendered.contains("reference-snapshot-source-summary"));
        assert!(rendered.contains("reference-snapshot-lunar-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-1749-major-body-boundary-summary"));
        assert!(rendered.contains("1749-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-early-major-body-boundary-summary"));
        assert!(rendered.contains("early-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-1800-major-body-boundary-summary"));
        assert!(rendered.contains("1800-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-2500-major-body-boundary-summary"));
        assert!(rendered.contains("2500-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-major-body-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-mars-jupiter-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-mars-outer-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-major-body-boundary-window-summary"));
        assert!(rendered.contains("reference-snapshot-body-class-coverage-summary"));
        assert!(rendered.contains("reference-snapshot-manifest-summary"));
        assert!(rendered.contains("reference-snapshot-summary"));
        assert!(rendered.contains("reference-snapshot-batch-parity-summary"));
        assert!(rendered.contains("reference-snapshot-equatorial-parity-summary"));
        assert!(rendered.contains("reference-high-curvature-summary"));
        assert!(rendered.contains("high-curvature-summary"));
        assert!(rendered.contains("reference-high-curvature-window-summary"));
        assert!(rendered.contains("high-curvature-window-summary"));
        assert!(rendered.contains("boundary-epoch-coverage-summary"));
        assert!(rendered.contains("interpolation-posture-summary"));
        assert!(rendered.contains("interpolation-quality-summary"));
        assert!(rendered.contains("interpolation-quality-kind-coverage-summary"));
        assert!(rendered.contains("lunar-apparent-comparison-summary"));
        assert!(rendered.contains("lunar-source-window-summary"));
        assert!(rendered.contains("lunar-theory-summary"));
        assert!(rendered.contains("lunar-theory-capability-summary"));
        assert!(rendered.contains("lunar-theory-source-summary"));
        assert!(rendered.contains("selected-asteroid-boundary-summary"));
        assert!(rendered.contains("reference-snapshot-selected-asteroid-terminal-boundary-summary"));
        assert!(rendered.contains("selected-asteroid-source-evidence-summary"));
        assert!(rendered.contains("selected-asteroid-source-summary"));
        assert!(rendered.contains("selected-asteroid-source-window-summary"));
        assert!(rendered.contains("selected-asteroid-batch-parity-summary"));
        assert!(rendered.contains("reference-asteroid-evidence-summary"));
        assert!(rendered.contains("reference-asteroid-equatorial-evidence-summary"));
        assert!(rendered.contains("reference-asteroid-source-window-summary"));
        assert!(rendered.contains("reference-holdout-overlap-summary"));
        assert!(rendered.contains("independent-holdout-source-window-summary"));
        assert!(rendered.contains("independent-holdout-summary"));
        assert!(rendered.contains("independent-holdout-source-summary"));
        assert!(rendered.contains("independent-holdout-body-class-coverage-summary"));
        assert!(rendered.contains("independent-holdout-batch-parity-summary"));
        assert!(rendered.contains("independent-holdout-equatorial-parity-summary"));
        assert!(rendered.contains("release-profile-identifiers-summary"));
        assert!(rendered.contains("mean-obliquity-frame-round-trip-summary"));
        assert!(rendered.contains("request-surface-summary"));
        assert!(rendered.contains("request-policy-summary"));
        assert!(rendered.contains("request-semantics-summary"));
        assert!(rendered.contains("comparison-tolerance-policy-summary"));
        assert!(rendered.contains("pluto-fallback-summary"));
        assert!(rendered.contains("verify-compatibility-profile"));
        assert!(rendered.contains("release-notes"));
        assert!(rendered.contains("release-notes-summary"));
        assert!(rendered.contains("release-checklist"));
        assert!(rendered.contains("release-checklist-summary"));
        assert!(rendered.contains("release-summary"));
        assert!(rendered.contains("packaged-lookup-epoch-policy-summary"));
        assert!(rendered.contains("jpl-batch-error-taxonomy-summary"));
        assert!(rendered.contains("jpl-snapshot-evidence-summary"));
        assert!(rendered.contains("bundle-release --out DIR"));
        assert!(rendered.contains("benchmark report"));
        assert!(rendered.contains("API stability summary"));
        assert!(rendered.contains("profile-summary"));
        assert!(rendered.contains("backend matrix"));
        assert!(rendered.contains("verify-release-bundle"));
    }

    #[test]
    fn regenerate_packaged_artifact_command_checks_and_writes_the_fixture() {
        let check = render_cli(&["regenerate-packaged-artifact", "--check"])
            .expect("packaged artifact regeneration check should render");
        assert!(check.contains("Packaged artifact regeneration check passed"));
        assert!(check.contains("checksum: 0x"));
        assert!(check.contains(&packaged_artifact_regeneration_summary_for_report()));

        let output_dir = unique_temp_dir("pleiades-packaged-artifact-regeneration");
        let output_path = output_dir.join("packaged-artifact.bin");
        let output_path_string = output_path.to_string_lossy().to_string();
        let rendered = render_cli(&["regenerate-packaged-artifact", "--out", &output_path_string])
            .expect("packaged artifact regeneration should write bytes");
        assert!(rendered.contains("Packaged artifact regenerated"));
        assert!(rendered.contains("checksum: 0x"));
        assert_eq!(
            std::fs::read(&output_path).expect("regenerated artifact should exist"),
            packaged_artifact_bytes()
        );
    }

    #[test]
    fn api_stability_command_renders_the_posture() {
        let rendered = render_cli(&["api-stability"]).expect("api posture should render");
        let release_profiles = current_release_profile_identifiers();
        assert!(rendered.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Stable consumer surfaces:"));
        assert!(rendered.contains("Experimental or operational surfaces:"));
        assert!(rendered.contains("Deprecation policy:"));
    }

    #[test]
    fn api_stability_summary_command_renders_the_summary() {
        let rendered =
            render_cli(&["api-stability-summary"]).expect("api stability summary should render");
        let release_profiles = current_release_profile_identifiers();
        let api_stability = current_api_stability_profile();
        assert!(rendered.contains("API stability summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains(&format!(
            "Summary line: API stability posture: {}; stable surfaces: {}; experimental surfaces: {}; deprecation policy items: {}; intentional limits: {}",
            release_profiles.api_stability_profile_id,
            api_stability.stable_surfaces.len(),
            api_stability.experimental_surfaces.len(),
            api_stability.deprecation_policy.len(),
            api_stability.intentional_limits.len()
        )));
        assert!(rendered.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains(&format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered.contains("Stable surfaces:"));
        assert!(rendered.contains("Experimental surfaces:"));
        assert!(rendered.contains("Deprecation policy items:"));
        assert!(rendered.contains("Intentional limits:"));
        assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(
            rendered.contains("See release-summary for the compact one-screen release overview.")
        );
    }

    #[test]
    fn compatibility_profile_command_renders_the_full_profile() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        let release_profiles = current_release_profile_identifiers();
        assert!(rendered.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains("Stage 6 release profile:"));
        assert!(rendered.contains("Target compatibility catalog:"));
        assert!(rendered.contains(
            "the full Swiss-Ephemeris-class house-system catalog remains the long-term compatibility goal."
        ));
        assert!(rendered.contains("Target ayanamsa catalog:"));
        assert!(rendered.contains(
            "the full Swiss-Ephemeris-class ayanamsa catalog remains the long-term compatibility goal."
        ));
        assert!(rendered.contains("Release-specific coverage beyond baseline:"));
        assert!(rendered.contains("Alias mappings for built-in house systems:"));
        assert!(rendered.contains("Source-label aliases for built-in house systems:"));
        assert!(rendered.contains("Source-label aliases for built-in ayanamsas:"));
        assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
        assert!(rendered.contains("Polich Page"));
        assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
        assert!(rendered.contains("Poli-equatorial"));
        assert!(rendered.contains("Poli-Equatorial"));
        assert!(rendered.contains("horizon/azimuth"));
        assert!(rendered.contains("Meridian table of houses"));
        assert!(rendered.contains("Meridian house system"));
        assert!(rendered.contains("Horizon house system"));
        assert!(rendered.contains("Whole-sign"));
        assert!(rendered.contains("Equal Midheaven house system"));
        assert!(rendered.contains("Equal Quadrant"));
        assert!(rendered.contains("Horizontal house system"));
        assert!(rendered.contains("Azimuth house system"));
        assert!(rendered.contains("Azimuthal house system"));
        assert!(rendered.contains("Carter's poli-equatorial"));
        assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
        assert!(rendered.contains("Babylonian Huber"));
        assert!(rendered.contains("Babylonian (House)"));
        assert!(rendered.contains("Babylonian (Sissy)"));
        assert!(rendered.contains("Babylonian (True Topc)"));
        assert!(rendered.contains("Babylonian (True Obs)"));
        assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
        assert!(rendered.contains("True Balarama"));
        assert!(rendered.contains("Aphoric"));
        assert!(rendered.contains("Takra"));
        assert!(rendered.contains("Galactic Equator (True)"));
        assert!(rendered.contains("Galactic Equator mid-Mula, Mula galactic equator, Galactic equator Mula -> Galactic Equator (Mula)"));
        assert!(rendered.contains("Valens Moon ayanamsa"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_recent_release_profile_entries() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Equal (MC) table of houses"));
        assert!(rendered.contains("Equal (MC) house system"));
        assert!(rendered.contains("Equal Midheaven house system"));
        assert!(rendered.contains("Equal from MC"));
        assert!(rendered.contains("Equal (from MC)"));
        assert!(rendered.contains("Equal (from MC) table of houses"));
        assert!(rendered.contains("Equal (1=Aries) table of houses"));
        assert!(rendered.contains("Equal (1=Aries) house system"));
        assert!(rendered.contains("Equal MC"));
        assert!(rendered.contains("Equal Midheaven"));
        assert!(rendered.contains("Babylonian 1"));
        assert!(rendered.contains("Babylonian 2"));
        assert!(rendered.contains("Babylonian 3"));
        assert!(rendered.contains("Vehlow Equal table of houses"));
        assert!(rendered.contains("Vehlow Equal house system"));
        assert!(rendered.contains("Vehlow equal"));
        assert!(rendered.contains("V equal Vehlow, Vehlow, Vehlow equal, Vehlow house system, Vehlow Equal house system, Vehlow-equal, Vehlow-equal table of houses, Vehlow Equal table of houses -> Vehlow Equal"));
        assert!(rendered.contains("Topocentric house system"));
        assert!(rendered.contains("Meridian house system"));
        assert!(rendered.contains("Horizon house system"));
        assert!(rendered.contains("Horizontal house system"));
        assert!(rendered.contains("Azimuth house system"));
        assert!(rendered.contains("Azimuthal house system"));
        assert!(rendered.contains("Carter's poli-equatorial"));
        assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
        assert!(rendered.contains("Albategnius"));
        assert!(rendered.contains("Gauquelin sectors"));
        assert!(rendered.contains("Equal table of houses"));
        assert!(rendered.contains("Equal (cusp 1 = Asc)"));
        assert!(rendered.contains("Whole Sign system"));
        assert!(rendered.contains("Whole Sign house system"));
        assert!(rendered.contains("Whole Sign (house 1 = Aries)"));
        assert!(rendered.contains("Morinus house system"));
        assert!(rendered.contains("Pullen SR (Sinusoidal Ratio) table of houses"));
        assert!(rendered.contains("Pullen SD (Sinusoidal Delta)"));
        assert!(rendered.contains("Pullen SD (Sinusoidal Delta) table of houses"));
        assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
        assert!(rendered.contains("Neo-Porphyry"));
        assert!(rendered.contains("WvA"));
        assert!(rendered.contains("Equal from MC"));
        assert!(rendered.contains("Equal (from MC)"));
        assert!(rendered.contains("Equal (from MC) table of houses"));
        assert!(rendered.contains("Makransky Sunshine"));
        assert!(rendered.contains("True Citra Paksha"));
        assert!(rendered.contains("True Chitra Paksha"));
        assert!(rendered.contains("True Chitrapaksha"));
        assert!(rendered.contains("Galactic Equator (Fiorenza)"));
        assert!(rendered.contains("Nick Anthony Fiorenza"));
        assert!(rendered.contains("Galactic Center (Cochrane)"));
        assert!(rendered.contains("Galactic Center (Gil Brand)"));
        assert!(rendered.contains("Gil Brand"));
        assert!(rendered.contains("P.V.R. Narasimha Rao"));
        assert!(rendered.contains("Bob Makransky"));
        assert!(rendered.contains("Sunshine table of houses, by Bob Makransky"));
        assert!(rendered.contains("Treindl Sunshine"));
        assert!(rendered.contains("Valens Moon"));
        assert!(rendered.contains("Babylonian (House Obs)"));
        assert!(rendered.contains("Sunil Sheoran / Vedic Sheoran / Sheoran ayanamsa spellings"));
        assert!(rendered.contains("True Sheoran"));
        assert!(rendered.contains("Lahiri (VP285)"));
        assert!(rendered.contains("Krishnamurti (VP291)"));
        assert!(rendered.contains("P.V.R. Narasimha Rao"));
        assert!(rendered.contains("B. V. Raman"));
        assert!(rendered.contains("Raman Ayanamsha"));
        assert!(rendered.contains("Raman ayanamsa"));
        assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
        assert!(rendered.contains("Polich/Page"));
        assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
        assert!(rendered.contains("T topocentric"));
        assert!(rendered.contains("Poli-equatorial"));
        assert!(rendered.contains("horizon/azimuth"));
        assert!(rendered.contains("horizon/azimut"));
        assert!(rendered.contains("Horizon/Azimuth table of houses"));
        assert!(rendered.contains("U krusinski-pisa-goelzer"));
        assert!(rendered.contains("X axial rotation system/ Meridian houses"));
        assert!(rendered.contains("Zariel"));
        assert!(rendered.contains("Babylonian Huber"));
        assert!(rendered.contains("Babylonian (True Topc)"));
        assert!(rendered.contains("Babylonian (True Obs)"));
        assert!(rendered.contains("Galactic Equator (True)"));
        assert!(rendered.contains("True galactic equator"));
        assert!(rendered.contains("Galactic equator true"));
        assert!(rendered.contains("Valens Moon ayanamsa"));
        assert!(rendered.contains("Lahiri (ICRC)"));
        assert!(rendered.contains("Lahiri (1940)"));
        assert!(rendered.contains("Yukteshwar"));
        assert!(rendered.contains("True Revati"));
        assert!(rendered.contains("True Pushya"));
        assert!(rendered.contains("Equal/MC = 10th"));
        assert!(rendered.contains("Equal Midheaven table of houses"));
        assert!(rendered.contains("Vehlow Equal table of houses"));
        assert!(rendered.contains("Vehlow equal"));
        assert!(rendered.contains("Wang"));
        assert!(rendered.contains("Aries houses"));
        assert!(rendered.contains("Fagan/Bradley"));
        assert!(rendered.contains("Usha Shashi"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_additional_equal_release_labels() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Equal/MC table of houses"));
        assert!(rendered.contains("Equal/MC house system"));
        assert!(rendered.contains("Equal/1=Aries table of houses"));
        assert!(rendered.contains("Equal/1=Aries house system"));
        assert!(rendered.contains("Equal/1=0 Aries"));
        assert!(rendered.contains("Equal (cusp 1 = 0° Aries)"));
        assert!(rendered.contains("Whole Sign (house 1 = Aries) table of houses"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_reference_frame_and_zero_point_entries() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Suryasiddhanta (499 CE)"));
        assert!(rendered.contains("Aryabhata (499 CE)"));
        assert!(rendered.contains("Sassanian"));
        assert!(rendered.contains("Sasanian"));
        assert!(rendered.contains("Zij al-Shah"));
        assert!(rendered.contains("DeLuce"));
        assert!(rendered.contains("Aryabhata (522 CE)"));
        assert!(rendered.contains("PVR Pushya-paksha"));
        assert!(rendered.contains("Galactic Center (Rgilbrand)"));
        assert!(rendered.contains("Galactic Center (Mardyks)"));
        assert!(rendered.contains("Skydram/Galactic Alignment"));
        assert!(rendered.contains("Skydram (Mardyks)"));
        assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
        assert!(rendered.contains("Galactic Center (Cochrane)"));
        assert!(rendered.contains("Gal. Center = 0 Sag"));
        assert!(rendered.contains("Gal. Center = 0 Cap"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_additional_ayanamsa_transliterations() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Aryabhatan Kaliyuga"));
        assert!(rendered.contains("Krishnamurti-Senthilathiban"));
        assert!(rendered.contains("Sri Yukteshwar"));
        assert!(rendered.contains("Shri Yukteshwar"));
        assert!(rendered.contains("De Luce"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_additional_reference_mode_entries() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Babylonian (Britton)"));
        assert!(rendered.contains("Babylonian/Britton"));
        assert!(rendered.contains("Babylonian (Aldebaran)"));
        assert!(rendered.contains("Babylonian/Aldebaran = 15 Tau"));
        assert!(rendered.contains("Babylonian (Eta Piscium)"));
        assert!(rendered.contains("Babylonian/Eta Piscium"));
        assert!(rendered.contains("Babylonian Eta Piscium"));
        assert!(rendered.contains("Eta Piscium"));
        assert!(rendered.contains("Hipparchus"));
        assert!(rendered.contains("Djwhal Khul"));
        assert!(rendered.contains("Udayagiri"));
        assert!(rendered.contains("True Mula"));
        assert!(rendered.contains("Suryasiddhanta (Mean Sun)"));
        assert!(rendered.contains("Aryabhata (Mean Sun)"));
        assert!(rendered.contains("Galactic Equator (IAU 1958)"));
        assert!(rendered.contains("Galactic Equator (Mula)"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_remaining_ayanamsa_and_reference_aliases() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Suryasiddhanta (Revati)"));
        assert!(rendered.contains("Suryasiddhanta (Citra)"));
        assert!(rendered.contains("True Pushya (PVRN Rao)"));
        assert!(rendered.contains("Dhruva Galactic Center (Middle Mula)"));
        assert!(rendered.contains("Dhruva Galactic Center Middle Mula"));
        assert!(rendered.contains("Dhruva/Gal.Center/Mula (Wilhelm)"));
        assert!(rendered.contains("Mula Wilhelm"));
        assert!(rendered.contains("Wilhelm"));
        assert!(rendered.contains("Middle of Mula"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_house_table_code_spellings() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("A equal, E equal = A"));
        assert!(rendered.contains("D equal / MC"));
        assert!(rendered.contains("N, Equal/1=Aries"));
        assert!(rendered.contains("S, S sripati"));
        assert!(rendered.contains("I, I sunshine"));
        assert!(rendered.contains("W equal, whole sign"));
        assert!(rendered.contains("V equal Vehlow"));
        assert!(rendered.contains("T, Polich-Page"));
        assert!(rendered.contains("U, Krusinski"));
        assert!(rendered.contains("X, Meridian houses"));
        assert!(rendered.contains("Y APC houses"));
        assert!(rendered.contains("M, Morinus houses"));
        assert!(rendered.contains("G, Gauquelin"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_ayanamsa_code_spellings() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("J2000.0 -> J2000"));
        assert!(rendered.contains("J1900.0 -> J1900"));
        assert!(rendered.contains("B1950.0 -> B1950"));
        assert!(rendered.contains(
            "SS Revati, Suryasiddhanta Revati, Surya Siddhanta Revati -> Suryasiddhanta (Revati)"
        ));
        assert!(rendered.contains(
            "SS Citra, Suryasiddhanta Citra, Surya Siddhanta Citra -> Suryasiddhanta (Citra)"
        ));
        assert!(rendered.contains("Galact. Center = 0 Sag, Gal. Center = 0 Sag -> Galactic Center"));
        assert!(rendered.contains("Gal. Eq."));
    }

    #[test]
    fn release_notes_command_renders_the_release_notes() {
        let rendered = render_cli(&["release-notes"]).expect("release notes should render");
        assert!(rendered.contains("Release notes"));
        let release_profiles = current_release_profile_identifiers();
        let profile = current_compatibility_profile();
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains(&format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(
            rendered.contains("Compatibility profile verification: verify-compatibility-profile")
        );
        assert!(rendered.contains("API stability posture:"));
        assert!(rendered.contains("Deprecation policy:"));
        assert!(rendered.contains("Release-specific coverage:"));
        assert!(rendered.contains(&format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        )));
        assert!(rendered.contains("selected asteroid coverage"));
        assert!(rendered.contains("WvA"));
        assert!(rendered.contains("Selected asteroid evidence: 5 exact J2000 samples"));
        assert!(rendered.contains("Selected asteroid batch parity: 5 requests across 5 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); frame mix: 3 ecliptic, 2 equatorial; batch/single parity preserved"));
        assert!(rendered.contains("Reference snapshot coverage: 195 rows across 15 bodies and 18 epochs (61 asteroid rows; JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
        assert!(rendered.contains("Reference snapshot body-class coverage: major bodies: 134 rows across 10 bodies and 17 epochs; major windows: "));
        assert!(rendered.contains(
            "selected asteroids: 61 rows across 5 bodies and 13 epochs; asteroid windows: "
        ));
        assert!(rendered.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_high_curvature_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_manifest_summary_for_report()));
        assert_report_contains_exact_line(
            &rendered,
            "Comparison snapshot coverage: 112 rows across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto",
        );
        assert!(rendered.contains("asteroid:433-Eros"));
        assert!(rendered.contains("Validation reference points:"));
        assert!(rendered.contains("Compatibility caveats:"));
        assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
        assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
        assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
        assert!(rendered.contains("Neo-Porphyry"));
        assert!(rendered.contains("Makransky Sunshine"));
        assert!(rendered.contains("Babylonian Huber"));
        assert!(rendered.contains("Babylonian (Britton)"));
        assert!(rendered.contains("Babylonian (Aldebaran)"));
        assert!(rendered.contains("Babylonian (Eta Piscium)"));
        assert!(rendered.contains("Babylonian (True Geoc)"));
        assert!(rendered.contains("Babylonian (True Topc)"));
        assert!(rendered.contains("Babylonian (True Obs)"));
        assert!(rendered.contains("Babylonian (House Obs)"));
        assert!(rendered.contains("Equal MC"));
        assert!(rendered.contains("Equal/MC house system"));
        assert!(rendered.contains("Equal Midheaven"));
        assert!(rendered.contains("Equal Midheaven house system"));
        assert!(rendered.contains("Babylonian (Kugler 1)"));
        assert!(rendered.contains("Krusinski/Pisa/Goelzer"));
        assert!(rendered.contains("Equal/MC = 10th"));
        assert!(rendered.contains("Galactic Equator (True)"));
        assert!(rendered.contains("Galactic Equator (IAU 1958)"));
        assert!(rendered.contains("Valens Moon ayanamsa"));
    }

    #[test]
    fn compatibility_profile_summary_command_renders_the_summary() {
        let rendered = render_cli(&["compatibility-profile-summary"])
            .expect("compatibility profile summary should render");
        let release_profiles = current_release_profile_identifiers();
        let profile = current_compatibility_profile();
        assert!(rendered.contains("Compatibility profile summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        let coverage = metadata_coverage();
        assert!(rendered.contains("House systems:"));
        assert!(rendered.contains("House systems: 25 total (12 baseline, 13 release-specific)"));
        assert!(rendered.contains(&format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        )));
        assert!(rendered.contains("Ayanamsas:"));
        assert!(rendered.contains("ayanamsa sidereal metadata: 53/59 entries with both a reference epoch and offset; custom-definition-only=6 labels: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs)"));
        assert!(rendered.contains(&format!(
            "House formula families: {}",
            profile.house_formula_families_summary_line()
        )));
        let catalog_inventory_summary = render_cli(&["catalog-inventory-summary"])
            .expect("catalog inventory summary should render");
        assert_eq!(
            catalog_inventory_summary,
            profile
                .validated_catalog_inventory_summary_line()
                .expect("catalog inventory summary should validate")
        );
        assert!(rendered
            .lines()
            .any(|line| line == profile.target_house_scope.join("; ")));
        assert!(rendered
            .lines()
            .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
        assert!(rendered.contains(&coverage.summary_line()));
        assert!(rendered.contains("ayanamsa catalog validation: ok"));
        let ayanamsa_metadata_coverage_summary =
            render_cli(&["ayanamsa-metadata-coverage-summary"])
                .expect("ayanamsa metadata coverage summary should render");
        assert_eq!(
            ayanamsa_metadata_coverage_summary,
            super::format_ayanamsa_metadata_coverage_for_report()
        );
        assert!(
            rendered.contains("Ayanamsa reference offsets: representative zero-point examples:")
        );
        assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
        assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
        assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
        assert!(rendered.contains("Usha Shashi: epoch=JD 2415020.5; offset=18.66096111111111°"));
        assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
        assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
        assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
        assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
        assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
        assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
        assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
        assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
        assert!(rendered.contains("J2000: epoch=JD 2451545; offset=23.85317778°"));
        assert!(rendered.contains("J1900: epoch=JD 2415020; offset=0°"));
        assert!(rendered.contains("B1950: epoch=JD 2433281.5; offset=0°"));
        assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
        assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
        assert!(rendered.contains("True Sheoran: epoch="));
        assert!(rendered.contains("Galactic Center: epoch="));
        assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
        assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
        assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
        assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
        assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
        assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
        assert!(rendered.contains("Aryabhata (522 CE): epoch=JD 1911797.740782; offset=0°"));
        assert!(rendered.contains("Release-specific house-system canonical names: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
        assert!(rendered.contains(&format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        )));
        assert!(rendered.contains("Release-specific ayanamsa canonical names:"));
        assert!(rendered.contains("Release-specific ayanamsa canonical names: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
        assert!(rendered.contains("Custom-definition labels: 9"));
        assert!(rendered.contains(&format!(
            "Custom-definition label names: {}",
            profile.custom_definition_labels.join(", ")
        )));
        assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
        assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(rendered.contains("Compatibility caveats: 2"));
        assert!(
            rendered.contains("Compatibility profile verification: verify-compatibility-profile")
        );
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(
            rendered.contains("See release-summary for the compact one-screen release overview.")
        );
    }

    #[test]
    fn ayanamsa_reference_offsets_summary_command_renders_the_summary() {
        let rendered = render_cli(&["ayanamsa-reference-offsets-summary"])
            .expect("ayanamsa reference offsets summary should render");

        assert!(
            rendered.contains("Ayanamsa reference offsets: representative zero-point examples:")
        );
        assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
        assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
        assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
        assert!(rendered.contains("Usha Shashi: epoch=JD 2415020.5; offset=18.66096111111111°"));
        assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
        assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
        assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
        assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
        assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
        assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
        assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
        assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
        assert!(rendered.contains("J2000: epoch=JD 2451545; offset=23.85317778°"));
        assert!(rendered.contains("J1900: epoch=JD 2415020; offset=0°"));
        assert!(rendered.contains("B1950: epoch=JD 2433281.5; offset=0°"));
        assert!(rendered.contains("True Pushya: epoch=JD 1855769.248315; offset=0°"));
        assert!(rendered.contains("Udayagiri: epoch=JD 1825235.164583; offset=0°"));
        assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
        assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
        assert!(rendered.contains("True Sheoran: epoch="));
        assert!(rendered.contains("Galactic Center: epoch="));
        assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
        assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
        assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
        assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
        assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
        assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
        assert!(rendered.contains("Aryabhata (522 CE): epoch=JD 1911797.740782; offset=0°"));
    }

    #[test]
    fn ayanamsa_metadata_coverage_summary_command_renders_the_summary() {
        let rendered = render_cli(&["ayanamsa-metadata-coverage-summary"])
            .expect("ayanamsa metadata coverage summary should render");

        assert_eq!(rendered, metadata_coverage().summary_line());
        assert!(rendered.contains(
            "ayanamsa sidereal metadata: 53/59 entries with both a reference epoch and offset; custom-definition-only=6 labels: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs)"
        ));
    }

    #[test]
    fn house_validation_summary_command_renders_the_summary() {
        let rendered = render_cli(&["house-validation-summary"])
            .expect("house validation summary should render");

        assert!(rendered.contains("House validation corpus: 5 scenarios"));
        assert!(rendered
            .contains("formula families: Equal, Whole Sign, Quadrant, Equatorial projection"));
        assert!(rendered.contains("latitude-sensitive systems: Koch, Placidus, Topocentric"));
        assert_eq!(
            rendered,
            house_validation_summary_line_for_report(&house_validation_report())
        );
    }

    #[test]
    fn house_formula_families_summary_command_renders_the_family_list() {
        let rendered = render_cli(&["house-formula-families-summary"])
            .expect("house formula families summary should render");

        assert!(rendered.contains("Equal"));
        assert!(rendered.contains("Whole Sign"));
        assert!(rendered.contains("Quadrant"));
        assert_eq!(rendered, format_house_formula_families_for_report());
    }

    #[test]
    fn house_code_aliases_summary_command_renders_the_alias_table() {
        let rendered = render_cli(&["house-code-aliases-summary"])
            .expect("house code aliases summary should render");

        assert!(rendered.contains("P -> Placidus"));
        assert!(rendered.contains("T -> Topocentric"));
        assert!(rendered.contains("X -> Meridian"));
        assert_eq!(rendered, format_house_code_aliases_for_report());
    }

    #[test]
    fn ayanamsa_catalog_validation_summary_command_renders_the_summary() {
        let rendered = render_cli(&["ayanamsa-catalog-validation-summary"])
            .expect("ayanamsa catalog validation summary should render");

        assert!(rendered.contains("ayanamsa catalog validation: ok"));
        assert!(rendered.contains("baseline=5, release=54"));
        assert_eq!(
            rendered,
            ayanamsa_catalog_validation_summary().summary_line()
        );
    }

    #[test]
    fn ayanamsa_reference_offsets_summary_rejects_duplicate_labels() {
        let summary = AyanamsaReferenceOffsetsSummary {
            examples: vec![
                AyanamsaReferenceOffsetExample {
                    canonical_name: "Lahiri",
                    epoch: JulianDay::from_days(2_435_553.5),
                    offset_degrees: Angle::from_degrees(23.245_524_743),
                },
                AyanamsaReferenceOffsetExample {
                    canonical_name: "lahiri",
                    epoch: JulianDay::from_days(2_451_544.5),
                    offset_degrees: Angle::from_degrees(25.0),
                },
            ],
        };

        let error = summary
            .validate()
            .expect_err("duplicate ayanamsa reference labels should fail validation");
        assert!(error
            .to_string()
            .contains("ayanamsa reference offsets contains a case-insensitive duplicate name"));
    }

    #[test]
    fn compatibility_profile_report_helper_validates_the_current_profile() {
        let profile = validated_compatibility_profile_for_report()
            .expect("compatibility profile should validate");
        assert_eq!(
            profile.profile_id,
            current_compatibility_profile().profile_id
        );
    }

    #[test]
    fn compatibility_profile_verification_command_checks_the_catalogs() {
        let rendered = render_cli(&["verify-compatibility-profile"])
            .expect("compatibility profile verification should render");
        let release_profiles = current_release_profile_identifiers();
        let profile = current_compatibility_profile();
        assert!(rendered.contains("Compatibility profile verification"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains("House systems verified: 25 descriptors, 181 labels"));
        assert!(rendered.contains(&format!(
            "House code aliases verified: {} short-form labels",
            profile.house_code_alias_count()
        )));
        assert!(rendered
            .contains("Alias uniqueness checks: exact and case-insensitive labels verified"));
        assert!(rendered.contains(
            "Latitude-sensitive house systems verified: 8 descriptors, 8 labels (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
        assert!(rendered.contains("Ayanamsas verified: 59 descriptors, 242 labels"));
        assert!(rendered.contains("House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"));
        assert!(rendered.contains(
            "Ayanamsa reference metadata verified: 53 descriptors with epoch/offset metadata, 6 metadata gaps"
        ));
        assert!(rendered.contains(&format!(
            "Custom-definition labels verified: {} labels, all remain custom-definition territory",
            profile.custom_definition_labels.len()
        )));
        assert!(rendered.contains("Baseline/release slices:"));
        assert!(rendered.contains("Release-specific house-system canonical names verified: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
        assert!(rendered.contains("Release-specific ayanamsa canonical names verified: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
        assert!(rendered.contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
        assert!(rendered.contains(&format!(
            "Custom-definition label names verified: {}",
            profile.custom_definition_labels.join(", ")
        )));
        assert!(rendered.contains(&format!(
            "Release notes documented: {} entries",
            profile.release_notes.len()
        )));
        assert!(rendered.contains(&format!(
            "Validation reference points documented: {} entries",
            profile.validation_reference_points.len()
        )));
        assert!(rendered.contains(&format!(
            "Custom-definition labels verified: {} labels, all remain custom-definition territory",
            profile.custom_definition_labels.len()
        )));
        assert!(rendered.contains(&format!(
            "Custom-definition label names verified: {}",
            profile.custom_definition_labels.join(", ")
        )));
        assert!(rendered.contains(&format!(
            "Compatibility caveats documented: {}",
            profile.known_gaps.len()
        )));
    }

    #[test]
    fn compatibility_profile_verification_summary_renders_consistently() {
        let summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        let release_profiles = current_release_profile_identifiers();
        let profile = current_compatibility_profile();

        assert_eq!(
            summary.profile_id,
            release_profiles.compatibility_profile_id
        );
        assert_eq!(
            summary.house_system_descriptor_count,
            profile.house_systems.len()
        );
        assert_eq!(
            summary.house_code_alias_count,
            profile.house_code_alias_count()
        );
        assert_eq!(
            summary.house_code_aliases_summary,
            profile.house_code_aliases_summary_line()
        );
        assert_eq!(summary.ayanamsa_descriptor_count, profile.ayanamsas.len());
        assert_eq!(
            summary.baseline_house_system_count,
            profile.baseline_house_systems.len()
        );
        assert_eq!(
            summary.release_house_system_count,
            profile.release_house_systems.len()
        );
        assert_eq!(
            summary.baseline_ayanamsa_count,
            profile.baseline_ayanamsas.len()
        );
        assert_eq!(
            summary.release_ayanamsa_count,
            profile.release_ayanamsas.len()
        );
        assert_eq!(summary.release_note_count, profile.release_notes.len());
        assert_eq!(
            summary.validation_reference_point_count,
            profile.validation_reference_points.len()
        );
        assert_eq!(
            summary.custom_definition_label_count,
            profile.custom_definition_labels.len()
        );
        assert_eq!(
            summary.custom_definition_label_names,
            profile.custom_definition_labels.join(", ")
        );
        assert_eq!(
            summary.house_formula_family_names,
            profile.house_formula_family_names().join(", ")
        );
        assert_eq!(
            summary.ayanamsa_metadata_count,
            profile
                .ayanamsas
                .iter()
                .filter(|entry| entry.has_sidereal_metadata())
                .count()
        );
        assert_eq!(
            summary.ayanamsa_metadata_gap_count,
            profile.ayanamsas.len() - summary.ayanamsa_metadata_count
        );
        assert_eq!(summary.compatibility_caveat_count, profile.known_gaps.len());
        summary
            .validate()
            .expect("fresh compatibility profile verification summary should validate");
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            verify_compatibility_profile().unwrap(),
            summary.summary_line()
        );
        assert!(summary
            .summary_line()
            .contains("Compatibility profile verification"));
        assert!(summary.summary_line().contains(&format!(
            "House code aliases verified: {} short-form labels",
            profile.house_code_alias_count()
        )));
        assert!(summary.summary_line().contains(&format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        )));
        assert!(summary
            .summary_line()
            .contains("House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"));
        assert!(summary.summary_line().contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
        assert!(summary.summary_line().contains(&format!(
            "Ayanamsa reference metadata verified: {} descriptors with epoch/offset metadata, {} metadata gaps",
            profile
                .ayanamsas
                .iter()
                .filter(|entry| entry.has_sidereal_metadata())
                .count(),
            profile.ayanamsas.len() - profile
                .ayanamsas
                .iter()
                .filter(|entry| entry.has_sidereal_metadata())
                .count()
        )));
        assert!(summary.summary_line().contains(&format!(
            "Custom-definition labels verified: {} labels, all remain custom-definition territory",
            profile.custom_definition_labels.len()
        )));
        assert!(summary
            .summary_line()
            .contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
    }

    #[test]
    fn compatibility_profile_verification_summary_validation_rejects_stale_fields() {
        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.release_ayanamsa_canonical_names = "stale summary".to_string();

        let error = summary
            .validate()
            .expect_err("stale compatibility profile verification summary should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("release ayanamsa canonical names mismatch"));

        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.house_code_alias_count = 0;

        let error = summary
            .validate()
            .expect_err("stale house-code alias count should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("house-code alias count mismatch"));

        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.house_code_aliases_summary = "stale summary".to_string();

        let error = summary
            .validate()
            .expect_err("stale house-code alias summary should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("house-code aliases mismatch"));

        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.custom_definition_label_names = "stale summary".to_string();

        let error = summary
            .validate()
            .expect_err("stale custom-definition label names should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("custom-definition label names mismatch"));

        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.house_formula_family_names = "stale summary".to_string();

        let error = summary
            .validate()
            .expect_err("stale house formula families should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("house formula families mismatch"));

        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.release_posture = "stale summary".to_string();

        let error = summary
            .validate()
            .expect_err("stale release posture should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("release posture mismatch"));

        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.ayanamsa_metadata_count = 0;

        let error = summary
            .validate()
            .expect_err("stale ayanamsa metadata counts should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("ayanamsa metadata count mismatch"));
    }

    #[test]
    fn compatibility_profile_verification_summary_validation_rejects_stale_latitude_sensitive_house_systems(
    ) {
        let mut summary = compatibility_profile_verification_summary()
            .expect("compatibility profile verification summary should render");
        summary.latitude_sensitive_house_systems.reverse();

        let error = summary
            .validate()
            .expect_err("stale latitude-sensitive house systems should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("latitude-sensitive house systems mismatch"));
    }

    #[test]
    fn descriptor_names_summary_validation_rejects_blank_entries() {
        let summary = DescriptorNamesSummary {
            names: vec!["Equal (MC)", "   "],
        };

        let error = summary
            .validate()
            .expect_err("blank descriptor names should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("blank name"));
    }

    #[test]
    fn descriptor_names_summary_validation_rejects_case_insensitive_duplicates() {
        let summary = DescriptorNamesSummary {
            names: vec!["Equal (MC)", "equal (mc)"],
        };

        let error = summary
            .validate()
            .expect_err("case-insensitive duplicate descriptor names should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("case-insensitive duplicate name"));
    }

    #[test]
    fn house_formula_families_summary_renders_and_validates() {
        let summary = HouseFormulaFamiliesSummary {
            names: current_compatibility_profile().house_formula_family_names(),
        };
        let expected = "7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)";

        summary
            .validate()
            .expect("current house formula families summary should validate");
        assert_eq!(summary.summary_line(), expected);
        assert_eq!(summary.to_string(), expected);
    }

    #[test]
    fn house_formula_families_summary_validation_rejects_stale_names() {
        let summary = HouseFormulaFamiliesSummary {
            names: vec!["Equal".to_string(), "equal".to_string()],
        };

        let error = summary
            .validate()
            .expect_err("case-insensitive duplicate house formula families should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("case-insensitive duplicate name"));
    }

    #[test]
    fn validate_name_sequence_rejects_whitespace_padded_owned_names() {
        let names = [String::from("Equal (MC)"), String::from(" release family ")];

        let error = validate_name_sequence(
            "compatibility profile house formula families",
            names.iter().map(String::as_str),
        )
        .expect_err("whitespace-padded owned names should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("surrounding whitespace"));
    }

    #[test]
    fn compatibility_profile_partition_checks_cover_the_current_catalog() {
        let profile = current_compatibility_profile();

        verify_profile_catalog_partitions_are_disjoint(
            "house-system",
            profile.baseline_house_systems,
            profile.release_house_systems,
            |entry| entry.canonical_name,
            |entry| entry.aliases,
        )
        .expect("the current house catalog partitions should remain disjoint");

        verify_profile_catalog_partitions_are_disjoint(
            "ayanamsa",
            profile.baseline_ayanamsas,
            profile.release_ayanamsas,
            |entry| entry.canonical_name,
            |entry| entry.aliases,
        )
        .expect("the current ayanamsa catalog partitions should remain disjoint");
    }

    #[test]
    fn compatibility_profile_partition_checks_reject_overlapping_labels() {
        let house_baseline = [pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Placidus",
            &["Placidus house system"],
            "Quadrant system used for partition-overlap coverage.",
            true,
        )];
        let house_release = [pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Koch,
            "Koch",
            &["Placidus"],
            "Quadrant system used for partition-overlap coverage.",
            true,
        )];

        let error = verify_profile_catalog_partitions_are_disjoint(
            "house-system",
            &house_baseline,
            &house_release,
            |entry| entry.canonical_name,
            |entry| entry.aliases,
        )
        .expect_err("overlapping house-system labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("baseline and release slices overlap on label 'Placidus'"));

        let ayanamsa_baseline = [pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &["Lahiri ayanamsa"],
            "Sidereal mode used for partition-overlap coverage.",
            Some(JulianDay::from_days(2_435_553.5)),
            Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
        )];
        let ayanamsa_release = [pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::Raman,
            "Raman",
            &["Lahiri"],
            "Sidereal mode used for partition-overlap coverage.",
            Some(JulianDay::from_days(2_415_020.0)),
            Some(pleiades_core::Angle::from_degrees(21.014_44)),
        )];

        let error = verify_profile_catalog_partitions_are_disjoint(
            "ayanamsa",
            &ayanamsa_baseline,
            &ayanamsa_release,
            |entry| entry.canonical_name,
            |entry| entry.aliases,
        )
        .expect_err("overlapping ayanamsa labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("baseline and release slices overlap on label 'Lahiri'"));
    }

    #[test]
    fn compatibility_profile_partition_checks_reject_case_normalized_alias_overlaps() {
        #[derive(Clone, Copy)]
        struct Entry {
            canonical_name: &'static str,
            aliases: &'static [&'static str],
        }

        let baseline = [Entry {
            canonical_name: "Lahiri",
            aliases: &["Lahiri ayanamsa"],
        }];
        let release = [Entry {
            canonical_name: "Raman",
            aliases: &["lahiri"],
        }];

        let error = verify_profile_catalog_partitions_are_disjoint(
            "ayanamsa",
            &baseline,
            &release,
            |entry| entry.canonical_name,
            |entry| entry.aliases,
        )
        .expect_err("case-normalized overlapping alias labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("baseline and release slices overlap on label 'lahiri'"));
    }

    #[test]
    fn descriptor_names_summary_formats_empty_single_and_multiple_entries() {
        #[derive(Clone, Copy)]
        struct Item(&'static str);

        let empty = summarize_descriptor_names(&[] as &[Item], |item| item.0);
        assert_eq!(empty.summary_line(), "0 (none)");
        assert_eq!(empty.to_string(), "0 (none)");

        let single = summarize_descriptor_names(&[Item("Alpha")], |item| item.0);
        assert_eq!(single.summary_line(), "1 (Alpha)");
        assert_eq!(single.to_string(), "1 (Alpha)");

        let multiple = summarize_descriptor_names(&[Item("Alpha"), Item("Beta")], |item| item.0);
        assert_eq!(multiple.summary_line(), "2 (Alpha, Beta)");
        assert_eq!(multiple.to_string(), "2 (Alpha, Beta)");
    }

    #[test]
    fn compatibility_profile_verification_rejects_duplicate_house_labels() {
        let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Placidus",
            &["Placidus"],
            "Quadrant system used for duplicate-label verification coverage.",
            true,
        )];

        let error = verify_house_system_aliases(&descriptors)
            .expect_err("duplicate labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("house-system labels are not unique"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_case_insensitive_duplicate_house_labels() {
        let descriptors = [
            pleiades_houses::HouseSystemDescriptor::new(
                HouseSystem::Placidus,
                "Placidus",
                &[],
                "Quadrant system used for case-insensitive duplicate-label coverage.",
                true,
            ),
            pleiades_houses::HouseSystemDescriptor::new(
                HouseSystem::Koch,
                "placidus",
                &[],
                "Quadrant system used for case-insensitive duplicate-label coverage.",
                true,
            ),
        ];

        let error = verify_house_system_aliases(&descriptors)
            .expect_err("case-insensitive duplicate labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("labels are not unique ignoring case"));
    }

    #[test]
    fn compatibility_profile_verification_allows_case_insensitive_duplicate_house_aliases_within_entry(
    ) {
        let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Placidus",
            &["placidus"],
            "Quadrant system used for intra-entry duplicate-label coverage.",
            true,
        )];

        let checked = verify_house_system_aliases(&descriptors).expect(
            "case-insensitive duplicate aliases within one descriptor should remain allowed",
        );
        assert_eq!(checked, 2);
    }

    #[test]
    fn compatibility_profile_verification_uses_display_labels_for_alias_mismatches() {
        let house_descriptors = [pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::EqualAries,
            "Equal (MC)",
            &[],
            "Quadrant system used for display-label mismatch coverage.",
            false,
        )];

        let error = verify_house_system_aliases(&house_descriptors)
            .expect_err("mismatched house labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("canonical label 'Equal (MC)' should resolve to Equal (1=Aries)"));
        assert!(!error.message.contains("EqualMidheaven"));
        assert!(!error.message.contains("EqualAries"));

        let ayanamsa_descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::TrueCitra,
            "True Chitra",
            &[],
            "Sidereal mode used for display-label mismatch coverage.",
            Some(JulianDay::from_days(2_451_545.0)),
            Some(pleiades_core::Angle::from_degrees(23.0)),
        )];

        let error = verify_ayanamsa_aliases(&ayanamsa_descriptors)
            .expect_err("mismatched ayanamsa labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("canonical label 'True Chitra' should resolve to True Citra"));
        assert!(!error.message.contains("TrueChitra"));
        assert!(!error.message.contains("TrueCitra"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_missing_descriptor_notes() {
        let descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &[],
            " ",
            Some(JulianDay::from_days(2_435_553.5)),
            Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
        )];

        let error = verify_ayanamsa_aliases(&descriptors)
            .expect_err("missing descriptor notes should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("missing notes metadata"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_whitespace_padded_canonical_names() {
        let error = ensure_profile_descriptor_metadata(
            "house-system",
            " Placidus ",
            "Quadrant system used for whitespace-padded metadata coverage.",
        )
        .expect_err("whitespace-padded canonical names should fail profile verification");

        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("contains surrounding whitespace in its canonical name"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_whitespace_padded_notes() {
        let error = ensure_profile_descriptor_metadata(
            "ayanamsa",
            "Lahiri",
            " whitespace-padded notes metadata ",
        )
        .expect_err("whitespace-padded notes should fail profile verification");

        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("contains surrounding whitespace in its notes metadata"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_whitespace_padded_house_aliases() {
        let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Placidus",
            &[" Placidus alias "],
            "Quadrant system used for whitespace-padded alias coverage.",
            true,
        )];

        let error = verify_house_system_aliases(&descriptors)
            .expect_err("whitespace-padded house aliases should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("contains surrounding whitespace in its label"));
        assert!(error.message.contains(" Placidus alias "));
    }

    #[test]
    fn compatibility_profile_verification_rejects_whitespace_padded_ayanamsa_aliases() {
        let descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &[" Lahiri alias "],
            "Ayanamsa used for whitespace-padded alias coverage.",
            Some(JulianDay::from_days(2_435_553.5)),
            Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
        )];

        let error = verify_ayanamsa_aliases(&descriptors)
            .expect_err("whitespace-padded ayanamsa aliases should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("contains surrounding whitespace in its label"));
        assert!(error.message.contains(" Lahiri alias "));
    }

    #[test]
    fn compatibility_profile_verification_rejects_whitespace_padded_labels() {
        let mut seen_labels = BTreeSet::new();
        let mut seen_labels_case_insensitive = BTreeMap::new();
        let error = ensure_unique_profile_label(
            "custom-definition",
            "  custom delta  ",
            "custom delta",
            &mut seen_labels,
            &mut seen_labels_case_insensitive,
        )
        .expect_err("whitespace-padded labels should fail profile verification");

        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("contains surrounding whitespace in its label"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_case_insensitive_duplicate_ayanamsa_labels() {
        let descriptors = [
            pleiades_ayanamsa::AyanamsaDescriptor::new(
                Ayanamsa::Lahiri,
                "Lahiri",
                &[],
                "Sidereal mode used for case-insensitive duplicate-label coverage.",
                Some(JulianDay::from_days(2_435_553.5)),
                Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
            ),
            pleiades_ayanamsa::AyanamsaDescriptor::new(
                Ayanamsa::TrueRevati,
                "lahiri",
                &[],
                "Sidereal mode used for case-insensitive duplicate-label coverage.",
                Some(JulianDay::from_days(2_444_907.5)),
                Some(pleiades_core::Angle::from_degrees(0.0)),
            ),
        ];

        let error = verify_ayanamsa_aliases(&descriptors)
            .expect_err("case-insensitive duplicate labels should fail profile verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("labels are not unique ignoring case"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_custom_definition_labels_that_resolve_to_builtins(
    ) {
        let labels = ["Placidus"];

        let error = verify_custom_definition_labels(&labels)
            .expect_err("custom-definition labels should stay outside built-ins");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("should remain unresolved as a built-in house system or ayanamsa"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_custom_definition_labels_that_resolve_to_ayanamsas(
    ) {
        let labels = ["Lahiri"];

        let error = verify_custom_definition_labels(&labels)
            .expect_err("custom-definition labels should stay outside built-in ayanamsas");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("should remain unresolved as a built-in house system or ayanamsa"));
    }

    #[test]
    fn compatibility_profile_verification_allows_intentional_ayanamsa_homographs() {
        let labels = INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS;

        let checked = verify_custom_definition_labels(labels)
            .expect("intentional custom-definition homographs should remain allowed");
        assert_eq!(checked, labels.len());
        assert!(is_intentional_custom_definition_ayanamsa_homograph(
            labels[0]
        ));
    }

    #[test]
    fn compatibility_profile_verification_rejects_case_insensitive_duplicate_custom_definition_labels(
    ) {
        let labels = ["custom delta", "Custom Delta"];

        let error = verify_custom_definition_labels(&labels).expect_err(
            "case-insensitive duplicate custom-definition labels should fail verification",
        );
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("custom-definition entries are not unique"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_blank_release_note_entries() {
        let entries = ["release note", "   "];

        let error = verify_profile_text_section("release-note", &entries)
            .expect_err("blank release-note entries should fail verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("entry is blank"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_duplicate_compatibility_caveats() {
        let entries = ["known gap", "known gap"];

        let error = verify_profile_text_section("compatibility-caveat", &entries)
            .expect_err("duplicate caveat entries should fail verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("entries are not unique"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_case_insensitive_duplicate_release_notes() {
        let entries = ["shared release text", "Shared Release Text"];

        let error = verify_profile_text_section("release-note", &entries)
            .expect_err("case-insensitive duplicate release notes should fail verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("entries are not unique ignoring case"));
        assert!(error.message.contains("Shared Release Text"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_duplicate_text_across_sections() {
        let error = verify_profile_text_sections_are_disjoint(&[
            ("release-note", &["shared release text"]),
            ("compatibility-caveat", &["shared release text"]),
        ])
        .expect_err("duplicate prose across sections should fail verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains(
            "duplicate entry 'shared release text' appears in both release-note and compatibility-caveat"
        ));
    }

    #[test]
    fn compatibility_profile_verification_rejects_case_insensitive_duplicate_text_across_sections()
    {
        let error = verify_profile_text_sections_are_disjoint(&[
            ("release-note", &["shared release text"]),
            ("compatibility-caveat", &["Shared Release Text"]),
        ])
        .expect_err("case-insensitive duplicate prose across sections should fail verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("not unique ignoring case"));
        assert!(error.message.contains("release-note"));
        assert!(error.message.contains("compatibility-caveat"));
    }

    #[test]
    fn compatibility_profile_verification_rejects_whitespace_padded_text_across_sections() {
        let error = verify_profile_text_sections_are_disjoint(&[
            ("release-note", &["shared release text "]),
            ("compatibility-caveat", &["shared release text"]),
        ])
        .expect_err("whitespace-padded prose across sections should fail verification");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains("contains surrounding whitespace"));
        assert!(error.message.contains("release-note"));
    }

    #[test]
    fn compatibility_profile_verification_validates_target_scope_sections() {
        let profile = current_compatibility_profile();

        verify_profile_text_section("target-house-scope", profile.target_house_scope)
            .expect("target house scope should validate");
        verify_profile_text_section("target-ayanamsa-scope", profile.target_ayanamsa_scope)
            .expect("target ayanamsa scope should validate");
        verify_profile_text_sections_are_disjoint(&[
            ("target-house-scope", profile.target_house_scope),
            ("target-ayanamsa-scope", profile.target_ayanamsa_scope),
            ("release-note", profile.release_notes),
            (
                "validation-reference-point",
                profile.validation_reference_points,
            ),
            ("compatibility-caveat", profile.known_gaps),
        ])
        .expect("target scope prose should remain disjoint from release prose");
    }

    #[test]
    fn release_notes_summary_command_renders_the_summary() {
        let rendered =
            render_cli(&["release-notes-summary"]).expect("release notes summary should render");
        let release_profiles = current_release_profile_identifiers();
        let profile = current_compatibility_profile();
        assert!(rendered.contains("Release notes summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains(&format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Release-specific coverage:"));
        assert!(rendered.contains("Selected asteroid source evidence: 61 source-backed samples across 5 bodies and 13 epochs (JD 2451545.0 (TDB)..JD 2634167.0 (TDB)); bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros"));
        assert!(rendered.contains("Selected asteroid source windows: 61 source-backed samples across 5 bodies and 13 epochs (JD 2451545.0 (TDB)..JD 2634167.0 (TDB)); windows: Ceres: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Pallas: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Juno: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Vesta: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 13 samples across 13 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB)"));
        assert!(rendered.contains(&selected_asteroid_boundary_summary_for_report()));
        assert!(rendered.contains("Custom-definition labels:"));
        assert!(rendered.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
        assert!(rendered.contains(&format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        )));
        assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
        assert!(rendered.contains("Compatibility caveats:"));
        assert!(rendered.contains(&format!(
            "Custom-definition labels: {}",
            profile.custom_definition_labels.len()
        )));
        assert!(rendered.contains(&format!(
            "Custom-definition label names: {}",
            profile.custom_definition_labels.join(", ")
        )));
        assert!(rendered.contains(&format!(
            "Compatibility caveats: {}",
            profile.known_gaps.len()
        )));
        assert!(rendered
            .lines()
            .any(|line| line == profile.target_house_scope.join("; ")));
        assert!(rendered
            .lines()
            .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
        assert!(rendered.contains("API stability summary line: API stability posture: pleiades-api-stability/0.1.0; stable surfaces: 6; experimental surfaces: 3; deprecation policy items: 4; intentional limits: 3"));
        assert!(rendered.contains("Reference snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01.; geocentric ecliptic J2000; TDB reference epoch JD 2451545.0 (TDB)"));
        assert!(rendered.contains("Reference snapshot body-class coverage: major bodies: 134 rows across 10 bodies and 17 epochs; major windows: "));
        assert!(rendered.contains(
            "selected asteroids: 61 rows across 5 bodies and 13 epochs; asteroid windows: "
        ));
        assert!(rendered.contains("Comparison snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains(&format!(
            "Packaged-artifact access: {}",
            format_packaged_artifact_access_summary()
        )));
        assert!(rendered.contains("Packaged request policy:"));
        assert!(rendered.contains(&format!(
            "Packaged lookup epoch policy: {}",
            packaged_lookup_epoch_policy_summary_for_report()
        )));
        assert!(rendered.lines().any(|line| {
            line == format!(
                "Packaged batch parity: {}",
                packaged_mixed_tt_tdb_batch_parity_summary_for_report()
            )
        }));
        assert!(rendered.contains("Packaged batch parity:"));
        assert!(rendered.contains("Release notes: release-notes"));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert_report_contains_exact_line(
            &rendered,
            "Workspace audit summary: workspace-audit-summary",
        );
        assert!(rendered.contains(profile.target_house_scope.join("; ").as_str()));
        assert!(rendered.contains(profile.target_ayanamsa_scope.join("; ").as_str()));
        assert!(rendered.contains(&format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Reference snapshot coverage: 195 rows across 15 bodies and 18 epochs (61 asteroid rows; JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
        assert_report_contains_exact_line(
            &rendered,
            "  Comparison snapshot coverage: 112 rows across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto",
        );
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Artifact boundary envelope:"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(
            rendered.contains("Compatibility profile verification: verify-compatibility-profile")
        );
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains("See release-notes for the full maintainer-facing artifact."));
        assert!(
            rendered.contains("See release-summary for the compact one-screen release overview.")
        );
    }

    #[test]
    fn release_checklist_summary_helper_reports_expected_posture() {
        let summary = release_checklist_summary();

        assert_eq!(
            summary.release_profile_identifiers,
            current_release_profile_identifiers()
        );
        assert_eq!(
            summary.repository_managed_release_gates,
            release_checklist_repository_managed_release_gates().len()
        );
        assert_eq!(
            summary.manual_bundle_workflow_items,
            release_checklist_manual_bundle_workflow().len()
        );
        assert_eq!(
            summary.bundle_contents_items,
            release_checklist_bundle_contents().len()
        );
        assert_eq!(
            summary.external_publishing_reminders,
            release_checklist_external_publishing_reminders().len()
        );
        assert!(summary.validate().is_ok());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.summary_line().contains("v1 compatibility="));
    }

    #[test]
    fn release_checklist_command_renders_the_release_checklist() {
        let rendered = render_cli(&["release-checklist"]).expect("release checklist should render");
        assert!(rendered.contains("Release checklist"));
        let release_profiles = current_release_profile_identifiers();
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(rendered.contains("API stability summary: api-stability-summary"));
        assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Repository-managed release gates:"));
        assert!(rendered.contains("Manual bundle workflow:"));
        assert!(rendered.contains("Bundle contents:"));
        assert!(rendered.contains("backend-matrix-summary.txt"));
        assert!(rendered.contains("api-stability-summary.txt"));
        assert!(rendered.contains("release-checklist-summary.txt"));
    }

    #[test]
    fn release_checklist_summary_command_renders_the_summary() {
        let rendered = render_cli(&["release-checklist-summary"])
            .expect("release checklist summary should render");
        let release_profiles = current_release_profile_identifiers();
        assert!(rendered.contains("Release checklist summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(rendered.contains("API stability summary: api-stability-summary"));
        assert!(rendered.contains("Zodiac policy:"));
        assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(
            rendered.contains("Compatibility profile verification: verify-compatibility-profile")
        );
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Repository-managed release gates: 7 items"));
        assert!(rendered.contains("Manual bundle workflow: 3 items"));
        assert!(rendered.contains("Bundle contents: 17 items"));
        assert!(rendered.contains("External publishing reminders: 3 items"));
        assert!(rendered.contains("See release-checklist for the full maintainer-facing artifact."));
        assert!(
            rendered.contains("See release-summary for the compact one-screen release overview.")
        );
    }

    #[test]
    fn release_summary_command_renders_the_quick_overview() {
        let rendered = render_cli(&["release-summary"]).expect("release summary should render");
        let release_profiles = current_release_profile_identifiers();
        let profile = current_compatibility_profile();
        assert!(rendered.contains("Release summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert_report_contains_exact_line(
            &rendered,
            &format!(
                "Release profile identifiers: v1 compatibility={}, api-stability={}",
                release_profiles.compatibility_profile_id,
                release_profiles.api_stability_profile_id
            ),
        );
        assert!(rendered
            .lines()
            .any(|line| line == profile.target_house_scope.join("; ")));
        assert!(rendered
            .lines()
            .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
        assert!(rendered.lines().any(|line| line == "Release summary"));
        assert!(rendered.contains("Release summary line:"));
        assert!(rendered.contains(&format!(
            "Packaged lookup epoch policy: {}",
            packaged_lookup_epoch_policy_summary_for_report()
        )));
        assert!(rendered
            .lines()
            .any(|line| line == "Backend matrix summary: backend-matrix-summary"));
        assert_report_contains_exact_line(&rendered, &profile.catalog_inventory_summary_line());
        assert_report_contains_exact_line(
            &rendered,
            "Compatibility catalog inventory: house systems=25 (12 baseline, 13 release-specific, 156 aliases); house-code aliases=22; ayanamsas=59 (5 baseline, 54 release-specific, 183 aliases); custom-definition labels=9; known gaps=2",
        );
        assert_report_contains_exact_line(
            &rendered,
            &format!(
                "House code aliases: {}",
                profile.house_code_aliases_summary_line()
            ),
        );
        assert!(rendered.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.lines().any(|line| {
            line == format!(
                "Packaged batch parity: {}",
                packaged_mixed_tt_tdb_batch_parity_summary_for_report()
            )
        }));
        assert!(rendered
            .lines()
            .any(|line| line == "Backend matrix summary: backend-matrix-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert_report_contains_exact_line(
            &rendered,
            "Workspace audit summary: workspace-audit-summary",
        );
        assert_report_contains_exact_line(
            &rendered,
            "Release checklist summary: release-checklist-summary",
        );
        assert!(rendered.contains("Workspace audit: workspace-audit / audit"));
        assert!(rendered.contains("Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"));
        assert!(rendered.contains("Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"));
        assert!(rendered.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"));
        assert!(rendered.contains("Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"));
        assert!(rendered.contains("Frame policy: ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
        assert!(rendered.contains(&request_surface_summary_for_report()));
        let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");
        assert!(rendered.lines().any(|line| {
            line == format!(
                "Mean-obliquity frame round-trip: {}",
                mean_obliquity_frame_round_trip
            )
        }));
        assert!(rendered.contains("Zodiac policy: tropical only"));
        assert!(rendered.contains("ayanamsa catalog validation: ok"));
        assert!(rendered.contains("House systems:"));
        assert!(rendered.contains("House systems: 25 total (12 baseline, 13 release-specific)"));
        assert!(rendered.contains(&format!(
            "House-code aliases: {}",
            profile.house_code_alias_count()
        )));
        assert!(rendered.contains("Release-specific house-system canonical names: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
        assert!(rendered.contains("Wang"));
        assert!(rendered.contains("Aries houses"));
        assert!(
            rendered.contains("Ayanamsa reference offsets: representative zero-point examples:")
        );
        assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
        assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
        assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
        assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
        assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
        assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
        assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
        assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
        assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
        assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
        assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
        assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
        assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
        assert!(rendered.contains("True Sheoran: epoch="));
        assert!(rendered.contains("Galactic Center: epoch="));
        assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
        assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
        assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
        assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
        assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
        assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
        assert!(rendered.contains("Release-specific ayanamsa canonical names: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
        assert!(
            rendered.contains("Ayanamsa reference offsets: representative zero-point examples:")
        );
        assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
        assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
        assert!(rendered.contains("JN Bhasin"));
        assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
        assert!(rendered.contains("Custom-definition labels: 9"));
        assert!(rendered.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));
        assert!(rendered.contains("Custom-definition ayanamsas:"));
        assert!(rendered.contains("Compatibility caveats: 2"));
        assert!(rendered.contains(&format!(
            "Ayanamsas: {} total ({} baseline, {} release-specific)",
            profile.ayanamsas.len(),
            profile.baseline_ayanamsas.len(),
            profile.release_ayanamsas.len()
        )));
        assert!(rendered.contains("Comparison envelope:"));
        assert!(rendered.contains("median longitude delta:"));
        assert!(rendered.contains("95th percentile longitude delta:"));
        assert!(rendered.contains("median latitude delta:"));
        assert!(rendered.contains("95th percentile latitude delta:"));
        assert_report_contains_exact_line(
            &rendered,
            "Comparison snapshot coverage: 112 rows across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto",
        );
        assert!(rendered.contains("Body-class error envelopes:"));
        assert!(rendered.contains("max Δlon="));
        assert!(rendered.contains("median Δlon="));
        assert!(rendered.contains("rms Δlat="));
        assert!(rendered.contains("max longitude delta:"));
        assert!(rendered.contains("median longitude delta:"));
        assert!(rendered.contains("rms longitude delta:"));
        assert!(rendered.contains("max latitude delta:"));
        assert!(rendered.contains("median latitude delta:"));
        assert!(rendered.contains("95th percentile latitude delta:"));
        assert!(rendered.contains("rms latitude delta:"));
        assert!(rendered.contains("Validation evidence:"));
        assert!(rendered.contains("House validation corpus: 5 scenarios (Mid-latitude reference chart, Equatorial reference chart, Polar stress chart, Southern polar stress chart, Southern hemisphere reference chart), 60 samples, 60 successes, 0 failures; formula families: Equal, Whole Sign, Quadrant, Equatorial projection; latitude-sensitive systems: Koch, Placidus, Topocentric; constraints: Koch [Quadrant system with documented high-latitude pathologies.], Placidus [Quadrant system; can fail or become unstable at extreme latitudes.], Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]"));
        assert!(rendered.contains("comparison samples"));
        assert!(rendered.contains("Time-scale policy:"));
        assert!(rendered.contains("Observer policy:"));
        assert!(rendered.contains("Apparentness policy:"));
        assert!(rendered.contains("Zodiac policy:"));
        assert!(rendered.contains("notable regressions"));
        assert!(rendered.contains("outside-tolerance bodies"));
        assert!(rendered.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
        assert!(rendered.contains("coverage=Luminaries: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence, bodies=2 (Sun, Moon), samples="));
        assert!(rendered.contains("window=JD 2360233.5 (TT) → JD 2634167.0 (TT)"));
        assert!(rendered.contains("frames=Ecliptic"));
        assert!(rendered.contains("Luminaries: Δlon≤7.500°, Δlat≤0.750°, Δdist=0.001 AU"));
        assert!(rendered.contains("Major planets: Δlon≤0.010°, Δlat≤0.010°, Δdist=0.001 AU"));
        assert!(rendered
            .contains("Pluto fallback (approximate): Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU"));
        assert!(rendered.contains("evidence=9 bodies"));
        assert!(rendered.contains("Body-class tolerance posture:"));
        assert!(rendered.contains("Expected tolerance status:"));
        assert!(rendered.contains("Comparison audit: status=clean, bodies checked=9"));
        assert!(rendered.contains("JPL interpolation evidence:"));
        assert!(rendered.contains("Reference/hold-out overlap:"));
        assert!(rendered.contains("JPL independent hold-out:"));
        assert!(rendered.contains("JPL independent hold-out equatorial parity:"));
        assert!(rendered.contains("JPL independent hold-out batch parity:"));
        assert_report_contains_exact_line(
            &rendered,
            "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false",
        );
        assert_report_contains_exact_line(
            &rendered,
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant",
        );
        assert!(rendered.contains("JPL frame treatment: checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"));
        assert!(rendered.contains("Reference snapshot coverage:"));
        assert!(rendered.contains("Selected asteroid evidence:"));
        assert!(rendered.contains("Selected asteroid batch parity:"));
        assert!(rendered.contains("VSOP87 evidence:"));
        assert!(rendered.contains("VSOP87 source-backed body-class envelopes:"));
        assert!(rendered.contains("VSOP87 canonical J2000 equatorial body-class envelopes:"));
        assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
        assert!(rendered.contains("median Δlon="));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("median Δlat="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("median Δdist="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered.contains("Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
        assert!(rendered.contains("VSOP87 source documentation:"));
        assert!(rendered.contains("VSOP87 frame treatment:"));
        assert!(rendered.contains("VSOP87 request policy:"));
        assert!(rendered.contains("VSOP87 source audit:"));
        assert!(rendered.contains("VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"));
        assert!(rendered.contains("VSOP87 canonical J2000 source-backed evidence:"));
        assert!(rendered.contains("VSOP87 canonical J2000 equatorial companion evidence:"));
        assert!(rendered.contains("VSOP87 canonical J2000 batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
        assert!(rendered.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
        assert!(rendered.contains("JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"));
        assert!(rendered.contains("VSOP87 canonical J1900 batch parity:"));
        assert!(rendered.contains("VSOP87 source-backed body evidence:"));
        assert!(rendered.contains("Lunar reference envelope:"));
        assert!(rendered.contains("Lunar equatorial reference envelope:"));
        assert!(rendered.contains("Lunar source windows:"));
        assert!(rendered.contains("JPL interpolation quality:"));
        assert!(rendered.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Artifact boundary envelope:"));
        assert!(rendered.contains(
            &artifact_inspection_summary_for_report()
                .expect("artifact inspection summary should build")
        ));
        assert!(rendered.contains("residual-bearing bodies: Moon"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release gate reminders:"));
        assert!(rendered.contains("verify-compatibility-profile"));
        assert!(rendered.contains("See release-notes and release-checklist"));
    }

    #[test]
    fn backend_matrix_command_renders_the_implemented_catalog() {
        let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
        assert!(rendered.contains("Implemented backend matrices"));
        assert!(rendered.contains("summary: id=jpl-snapshot; version="));
        assert!(rendered.contains("JPL snapshot reference backend"));
        assert!(rendered.contains(
            "selected asteroid coverage: 5 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        ));
        assert!(rendered.contains("Selected asteroid source evidence: 61 source-backed samples across 5 bodies and 13 epochs (JD 2451545.0 (TDB)..JD 2634167.0 (TDB)); bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros"));
        assert!(rendered.contains("Selected asteroid source windows: 61 source-backed samples across 5 bodies and 13 epochs (JD 2451545.0 (TDB)..JD 2634167.0 (TDB)); windows: Ceres: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Pallas: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Juno: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Vesta: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 13 samples across 13 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB)"));
        assert!(rendered.contains(&selected_asteroid_boundary_summary_for_report()));
        assert!(rendered.contains("exact J2000 evidence: 5 bodies at JD 2451545.0"));
        assert!(rendered.contains("nominal range:"));
        assert!(rendered.contains("provenance sources:"));
        assert!(rendered.contains("implementation status: fixture-reference"));
        assert!(rendered.contains("implementation status: partial-source-backed"));
        assert!(rendered.contains("implementation status: preliminary-algorithm"));
        assert!(rendered.contains("implementation status: prototype-artifact"));
        assert!(rendered.contains("implementation status: routing-facade"));
        assert!(rendered.contains("family posture: data-backed"));
        assert!(rendered.contains("family posture: algorithmic"));
        assert!(rendered.contains("family posture: routing"));
        assert!(rendered.contains("implementation note:"));
        assert!(rendered.contains("Sun through Neptune now use generated binary VSOP87B source tables derived from the vendored full-file inputs"));
        assert!(rendered.contains("expected error classes:"));
        assert!(rendered.contains("required external data files:"));
        assert!(rendered.contains("crates/pleiades-jpl/data/reference_snapshot.csv"));
        assert!(rendered.contains("source documentation:"));
        assert!(rendered.contains("source audit:"));
        assert!(rendered.contains("Sun: IMCCE/CELMECH VSOP87B VSOP87B.ear"));
        assert!(rendered.contains("Paul Schlyter-style mean orbital elements for planets"));
        assert!(rendered.contains("body source profiles:"));
        assert!(rendered.contains("VSOP87B.ear"));
        assert!(rendered.contains("geocentric planetary reduction against Earth coefficients"));
        assert!(rendered.contains("solar reduction from Earth coefficients"));
        assert!(rendered.contains("canonical J2000 VSOP87B evidence:"));
        assert!(rendered.contains("Sun: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(rendered.contains("Mercury: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(rendered.contains("Venus: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(rendered.contains("Mars: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(rendered.contains("Jupiter: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(rendered.contains("Saturn: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(rendered.contains("Uranus: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(rendered.contains("Neptune: kind=generated binary VSOP87B, accuracy=Exact"));
        assert!(
            rendered.contains("Pluto: kind=mean orbital elements fallback, accuracy=Approximate")
        );
        assert!(rendered.contains("Meeus-style truncated lunar orbit formulas"));
        assert!(rendered.contains("NASA/JPL Horizons API vector tables (DE441)"));
        assert!(rendered.contains("interpolation quality checks:"));
        assert!(rendered.contains("JPL interpolation posture: source="));
        assert!(rendered.contains("interpolation, bracket span"));
        assert!(rendered.contains("TDB"));
        assert!(rendered.contains("VSOP87 planetary backend"));
        assert!(rendered.contains("Pluto remains the current approximate mean-element fallback special case until a Pluto-specific source path is selected"));
        assert!(rendered.contains("ELP lunar backend (Moon and lunar nodes)"));
        assert!(rendered.contains("specification summary: ELP lunar theory specification:"));
        assert!(rendered.contains("compact lunar and lunar-point formulas provide the current deterministic baseline while documented production lunar-theory ingestion remains open"));
        assert!(rendered.contains("Lunar source windows"));
        assert!(rendered.contains("Lunar high-curvature continuity evidence"));
        assert!(rendered.contains("Lunar high-curvature equatorial continuity evidence"));
        assert!(rendered.contains("unsupported bodies: True Apogee, True Perigee"));
        assert!(rendered.contains("Packaged data backend"));
        assert!(rendered.contains("Composite routed backend"));
    }

    #[test]
    fn backend_matrix_report_rejects_invalid_backend_metadata() {
        let entry = BackendMatrixEntry {
            label: "broken backend",
            metadata: BackendMetadata {
                id: pleiades_core::BackendId::new("broken"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: pleiades_core::BackendProvenance {
                    summary: "  ".to_string(),
                    data_sources: vec![],
                },
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            },
            implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
            status_note: "broken for testing",
            expected_error_kinds: &[],
            required_data_files: &[],
        };

        let error = validate_backend_matrix_entry(&entry)
            .expect_err("invalid backend metadata should be rejected");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .to_string()
            .contains("backend matrix entry `broken backend` has invalid metadata"));
        assert!(error.to_string().contains("provenance summary"));
    }

    #[test]
    fn format_capabilities_reports_unavailable_for_invalid_flag_sets() {
        let capabilities = BackendCapabilities {
            geocentric: true,
            topocentric: false,
            apparent: false,
            mean: false,
            batch: true,
            native_sidereal: false,
        };

        assert_eq!(
            format_capabilities(&capabilities),
            "unavailable (backend capabilities must support mean or apparent output)"
        );
    }

    #[test]
    fn backend_matrix_summary_command_renders_the_summary() {
        let rendered =
            render_cli(&["backend-matrix-summary"]).expect("backend matrix summary should render");
        assert!(rendered.contains("Backend matrix summary"));
        assert!(rendered.contains("Backends: 5"));
        assert!(rendered.contains("Families:"));
        assert!(rendered.contains("Algorithmic: 2"));
        assert!(rendered.contains("ReferenceData: 1"));
        assert!(rendered.contains("CompressedData: 1"));
        assert!(rendered.contains("Composite: 1"));
        assert!(rendered.contains("Implementation statuses:"));
        assert!(rendered.contains("Nominal ranges: bounded: 2, open-ended: 3"));
        assert!(rendered.contains("fixture-reference: 1"));
        assert!(rendered.contains("partial-source-backed: 1"));
        assert!(rendered.contains("preliminary-algorithm: 1"));
        assert!(rendered.contains("prototype-artifact: 1"));
        assert!(rendered.contains("routing-facade: 1"));
        assert!(rendered.contains("Accuracy classes:"));
        assert!(rendered.contains("Exact: 1"));
        assert!(rendered.contains("Approximate: 4"));
        assert!(rendered.contains("VSOP87 source documentation: 8 source specs, 8 source-backed body profiles, 1 fallback mean-element body profile (Pluto); source-backed bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
        assert!(rendered.contains(
            "source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"
        ));
        assert!(rendered.contains(
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        ));
        assert!(rendered.contains("VSOP87 canonical J2000 batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
        assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
        assert!(rendered.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
        assert!(rendered.contains("JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"));
        assert!(rendered
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
        assert!(rendered
            .contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files"));
        assert!(rendered.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(rendered.contains("VSOP87 canonical J2000 interim outliers: none"));
        assert!(
            rendered.contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples")
        );
        assert!(rendered.contains("VSOP87 canonical J1900 batch parity:"));
        assert!(
            rendered.contains("quality counts: Exact=8, Interpolated=0, Approximate=1, Unknown=0")
        );
        assert!(rendered.contains("generated binary VSOP87B"));
        assert!(rendered.contains("generated binary VSOP87B; VSOP87B."));
        assert!(rendered.contains("max Δlon="));
        assert!(rendered.contains("max Δlat="));
        assert!(rendered.contains("max Δdist="));
        assert!(rendered.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
        assert!(rendered.contains(
            "ELP lunar theory specification: Compact Meeus-style truncated lunar baseline [meeus-style-truncated-lunar-baseline; family: Meeus-style truncated analytical baseline; selected key: source identifier=meeus-style-truncated-lunar-baseline]"
        ));
        assert!(rendered.contains(
            "lunar theory catalog: 1 entry, 1 selected entry; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
        assert!(rendered.contains(
            "lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; selected key: source identifier=meeus-style-truncated-lunar-baseline; selected family key: source family=Meeus-style truncated analytical baseline; aliases=1; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)"
        ));
        assert!(rendered.contains("lunar reference error envelope: 9 samples across 5 bodies"));
        assert!(rendered.contains("max Δlon="));
        assert!(rendered.contains("mean Δlon="));
        assert!(rendered.contains("median Δlon="));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("max Δlat="));
        assert!(rendered.contains("mean Δlat="));
        assert!(rendered.contains("median Δlat="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("limits: Δlon≤1e-4°"));
        assert!(rendered.contains("request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"));
        assert!(rendered.contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
        assert!(rendered.contains("date-range note: Validated against the published 1992-04-12 geocentric Moon example, the published 1992-04-12 geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform, the reference-only published 1968-12-24 apparent geocentric Moon comparison datum, the reference-only published 2004-04-01 NASA RP 1349 apparent Moon table row, the reference-only published 2006-09-07 EclipseWise apparent Moon coordinate row, J2000 lunar-point anchors including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example"));
        assert!(rendered.contains("lunar equatorial reference evidence: 3 samples across 1 bodies"));
        assert!(rendered
            .contains("lunar equatorial reference error envelope: 3 samples across 1 bodies"));
        assert!(rendered.contains("mean ΔRA="));
        assert!(rendered.contains("median ΔRA="));
        assert!(rendered.contains("p95 ΔRA="));
        assert!(rendered.contains("limits: ΔRA≤1e-2°"));
        assert!(rendered.contains("Lunar high-curvature continuity evidence"));
        assert!(rendered.contains("Lunar high-curvature equatorial continuity evidence"));
        assert!(rendered
            .contains("lunar high-curvature continuity evidence: 6 samples across 1 bodies"));
        assert!(rendered.contains(
            "lunar high-curvature equatorial continuity evidence: 6 samples across 1 bodies"
        ));
        assert!(rendered.contains("within regression limits=true"));
        assert!(rendered.contains("citation: Jean Meeus"));
        assert!(rendered
            .contains("provenance: Published lunar position, node, and mean-point formulas"));
        assert!(rendered.contains(
            "redistribution: No external coefficient-file redistribution constraints apply"
        ));
        assert!(rendered.contains("license: The current baseline is handwritten pure Rust"));
        assert!(rendered.contains("2 unsupported bodies: True Apogee, True Perigee"));
        assert!(rendered.contains("Distinct bodies covered:"));
        assert!(rendered.contains("Distinct coordinate frames: 2 (Ecliptic, Equatorial)"));
        assert!(rendered.contains("Distinct time scales: 2 (TT, TDB)"));
        assert!(rendered.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        }));
        assert!(rendered.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
        assert!(rendered.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        }));
        assert!(rendered.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
        assert!(rendered.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
        assert_eq!(
            validated_request_policy_summary_for_report()
                .expect("current request policy summary should validate")
                .summary_line(),
            request_policy_summary_for_report()
                .validated_summary_line()
                .expect("current request policy summary should render")
        );
        assert!(rendered.contains(
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        ));
        assert!(rendered.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        }));
        assert!(rendered.contains(
            "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)"
        ));
        assert!(rendered.contains("Frame policy:"));
        let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");
        assert!(rendered.lines().any(|line| {
            line == format!(
                "Mean-obliquity frame round-trip: {}",
                mean_obliquity_frame_round_trip
            )
        }));
        assert!(rendered.contains("Zodiac policy:"));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered.contains("API stability summary: api-stability-summary"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Reference snapshot coverage: 195 rows across 15 bodies and 18 epochs (61 asteroid rows; JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
        assert!(rendered.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_high_curvature_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
        assert!(rendered.contains(&reference_snapshot_manifest_summary_for_report()));
        assert!(rendered.contains("Comparison audit: compare-backends-audit; status="));
        assert!(rendered.contains("within tolerance bodies="));
        assert!(rendered.contains("outside tolerance bodies="));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
        assert!(
            rendered.contains("See release-summary for the compact one-screen release overview.")
        );

        let capability_matrix =
            render_cli(&["capability-matrix"]).expect("capability matrix should render");
        assert_eq!(
            capability_matrix,
            render_cli(&["backend-matrix"]).expect("backend matrix should render")
        );

        let matrix_summary = render_cli(&["matrix-summary"]).expect("matrix summary should render");
        assert_eq!(matrix_summary, rendered);
    }

    #[test]
    fn workspace_audit_reports_a_clean_workspace() {
        let report = workspace_audit_report().expect("workspace audit should render");
        assert!(report.is_clean());
        assert!(report
            .to_string()
            .contains("no mandatory native build hooks detected"));
        assert!(report.to_string().contains("Checked manifests:"));
    }

    #[test]
    fn workspace_audit_summary_reports_a_clean_workspace() {
        let report = workspace_audit_report().expect("workspace audit should render");
        let summary = workspace_audit_summary(&report);
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.summary_line().contains("violations: 0"));
        assert!(summary
            .summary_line()
            .contains("no mandatory native build hooks detected"));

        let rendered = render_cli(&["workspace-audit-summary"])
            .expect("workspace audit summary should render");
        let alias = render_cli(&["native-dependency-audit-summary"])
            .expect("native dependency audit summary should render");
        assert_eq!(rendered, alias);
        assert!(rendered.contains("Workspace audit summary"));
        assert!(rendered.contains("Summary: workspace root:"));
        assert!(rendered.contains("Checked manifests:"));
        assert!(rendered.contains("Checked lockfile:"));
        assert!(rendered.contains("Result: no mandatory native build hooks detected"));
    }

    #[test]
    fn workspace_audit_detects_native_hooks_in_manifests_and_lockfile() {
        let manifest = r#"[package]
name = "example"
build = "build.rs"
links = "example-native"

[dependencies]
cc = "1"
openssl-sys = "0.9"
[target.'cfg(unix)'.dependencies]
bindgen = { version = "0.69" }
renamed-native = { package = "zstd-sys", version = "2" }
"#;
        let manifest_violations = audit_manifest_text(Path::new("/tmp/Cargo.toml"), manifest);
        assert!(manifest_violations
            .iter()
            .any(|violation| violation.rule == "package.build"));
        assert!(manifest_violations
            .iter()
            .any(|violation| violation.rule == "package.links"));
        assert!(manifest_violations
            .iter()
            .any(|violation| violation.detail.contains("cc")));
        assert!(manifest_violations
            .iter()
            .any(|violation| violation.detail.contains("bindgen")));
        assert!(manifest_violations
            .iter()
            .any(|violation| violation.rule == "dependency.native-package"
                && violation.detail.contains("openssl-sys")));
        assert!(manifest_violations
            .iter()
            .any(|violation| violation.rule == "dependency.native-package"
                && violation.detail.contains("zstd-sys")));

        let build_script_dir = unique_temp_dir("pleiades-workspace-audit-build-script");
        let build_script_manifest = build_script_dir.join("Cargo.toml");
        let build_script_path = build_script_dir.join("build.rs");
        std::fs::write(
            &build_script_manifest,
            "[package]\nname = \"example-build-script\"\nversion = \"0.1.0\"\n",
        )
        .expect("manifest should be writable");
        std::fs::write(&build_script_path, "fn main() {}\n").expect("build.rs should be writable");
        let build_script_violation =
            audit_build_script_path(&build_script_manifest).expect("build.rs should be detected");
        assert_eq!(build_script_violation.rule, "package.build-script");
        assert_eq!(build_script_violation.path, build_script_path);
        assert!(build_script_violation.detail.contains("build.rs"));

        let lockfile = r#"[[package]]
name = "openssl-sys"
version = "0.9.0"
"#;
        let lockfile_violations = audit_lockfile_text(Path::new("/tmp/Cargo.lock"), lockfile);
        assert!(lockfile_violations
            .iter()
            .any(|violation| violation.rule == "lockfile.native-package"));
    }

    #[test]
    fn workspace_audit_summary_groups_rule_counts_for_violations() {
        let report = WorkspaceAuditReport {
            workspace_root: PathBuf::from("/workspace"),
            manifest_paths: vec![PathBuf::from("/workspace/Cargo.toml")],
            lockfile_path: PathBuf::from("/workspace/Cargo.lock"),
            violations: vec![
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.toml"),
                    rule: "package.build",
                    detail: "package declares a build script".to_string(),
                },
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.toml"),
                    rule: "package.build",
                    detail: "package declares a build script".to_string(),
                },
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.lock"),
                    rule: "lockfile.native-package",
                    detail: "lockfile package `openssl-sys` suggests a native build dependency and should be reviewed".to_string(),
                },
            ],
        };

        let summary = workspace_audit_summary(&report);
        assert!(summary.summary_line().contains("violations: 3"));
        assert_eq!(
            summary.rule_counts,
            vec![("lockfile.native-package", 1), ("package.build", 2)]
        );
        summary
            .validate()
            .expect("workspace audit summary should validate");

        let rendered = render_workspace_audit_summary_text(&report);
        assert!(rendered.contains("Summary: workspace root:"));
        assert!(rendered.contains("Violations: 3"));
        assert!(rendered.contains("Rule counts:"));
        assert!(rendered.contains("package.build: 2"));
        assert!(rendered.contains("lockfile.native-package: 1"));
        assert!(rendered.contains("Result: violations found"));

        let display = report.to_string();
        assert!(display.contains("Rule counts:"));
        assert!(display.contains("package.build: 2"));
        assert!(display.contains("lockfile.native-package: 1"));
    }

    #[test]
    fn workspace_audit_summary_validate_rejects_incoherent_counts() {
        let summary = WorkspaceAuditSummary {
            workspace_root: PathBuf::from("/workspace"),
            manifest_count: 1,
            lockfile_path: PathBuf::from("/workspace/Cargo.lock"),
            violation_count: 2,
            rule_counts: vec![("package.build", 1)],
            clean: false,
        };

        let error = summary
            .validate()
            .expect_err("incoherent workspace audit summary should fail validation");
        assert!(error
            .to_string()
            .contains("workspace audit summary violation count mismatch"));
    }

    #[test]
    fn release_bundle_writes_expected_artifacts() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        let rendered = render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        assert!(rendered.contains("Release bundle"));
        assert!(rendered.contains("compatibility-profile.txt"));
        assert!(rendered.contains("compatibility-profile-summary.txt"));
        assert!(rendered.contains("validation rounds: 1"));
        assert!(rendered.contains("release-notes.txt"));
        assert!(rendered.contains("release-notes-summary.txt"));
        assert!(rendered.contains("release-checklist.txt"));
        assert!(rendered.contains("backend-matrix.txt"));
        assert!(rendered.contains("API stability posture:"));
        assert!(rendered.contains("api-stability.txt"));
        assert!(rendered.contains("validation-report-summary.txt"));
        assert!(rendered.contains("workspace-audit-summary.txt"));
        assert!(rendered.contains("artifact-summary.txt"));
        assert!(bundle_dir
            .join("packaged-artifact-generation-manifest.txt")
            .exists());
        assert!(rendered.contains("benchmark-report.txt"));
        assert!(rendered.contains("validation-report.txt"));
        assert!(rendered.contains("release-profile-identifiers.txt"));
        assert!(rendered.contains("bundle-manifest.checksum.txt"));
        assert!(rendered.contains("source revision:"));
        assert!(rendered.contains("workspace status:"));
        assert!(rendered.contains("rustc version:"));
        assert!(rendered.contains("checksum: 0x"));

        let profile = std::fs::read_to_string(bundle_dir.join("compatibility-profile.txt"))
            .expect("compatibility profile should be written");
        let profile_summary =
            std::fs::read_to_string(bundle_dir.join("compatibility-profile-summary.txt"))
                .expect("compatibility profile summary should be written");
        let release_notes = std::fs::read_to_string(bundle_dir.join("release-notes.txt"))
            .expect("release notes should be written");
        let release_notes_summary =
            std::fs::read_to_string(bundle_dir.join("release-notes-summary.txt"))
                .expect("release notes summary should be written");
        let release_summary = std::fs::read_to_string(bundle_dir.join("release-summary.txt"))
            .expect("release summary should be written");
        let release_profile_identifiers =
            std::fs::read_to_string(bundle_dir.join("release-profile-identifiers.txt"))
                .expect("release-profile identifiers should be written");
        let release_checklist = std::fs::read_to_string(bundle_dir.join("release-checklist.txt"))
            .expect("release checklist should be written");
        let release_checklist_summary =
            std::fs::read_to_string(bundle_dir.join("release-checklist-summary.txt"))
                .expect("release checklist summary should be written");
        let backend_matrix = std::fs::read_to_string(bundle_dir.join("backend-matrix.txt"))
            .expect("backend matrix should be written");
        let backend_matrix_summary =
            std::fs::read_to_string(bundle_dir.join("backend-matrix-summary.txt"))
                .expect("backend matrix summary should be written");
        let api_stability = std::fs::read_to_string(bundle_dir.join("api-stability.txt"))
            .expect("API stability posture should be written");
        let api_stability_summary =
            std::fs::read_to_string(bundle_dir.join("api-stability-summary.txt"))
                .expect("API stability summary should be written");
        let validation_report_summary =
            std::fs::read_to_string(bundle_dir.join("validation-report-summary.txt"))
                .expect("validation report summary should be written");
        let workspace_audit_summary =
            std::fs::read_to_string(bundle_dir.join("workspace-audit-summary.txt"))
                .expect("workspace audit summary should be written");
        let artifact_summary = std::fs::read_to_string(bundle_dir.join("artifact-summary.txt"))
            .expect("artifact summary should be written");
        let packaged_artifact_generation_manifest =
            std::fs::read_to_string(bundle_dir.join("packaged-artifact-generation-manifest.txt"))
                .expect("packaged artifact generation manifest should be written");
        let compatibility_profile = current_compatibility_profile();
        let house_code_aliases_summary = compatibility_profile.house_code_aliases_summary_line();
        let benchmark_report = std::fs::read_to_string(bundle_dir.join("benchmark-report.txt"))
            .expect("benchmark report should be written");
        let report = std::fs::read_to_string(bundle_dir.join("validation-report.txt"))
            .expect("validation report should be written");
        assert!(benchmark_report.contains("Benchmark report"));
        let manifest = std::fs::read_to_string(bundle_dir.join("bundle-manifest.txt"))
            .expect("manifest should be written");
        let manifest_checksum =
            std::fs::read_to_string(bundle_dir.join("bundle-manifest.checksum.txt"))
                .expect("manifest checksum sidecar should be written");

        let release_profiles = current_release_profile_identifiers();
        assert!(profile.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(profile_summary.contains(&format!(
            "Compatibility profile summary\nProfile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(profile_summary.contains(
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
        assert!(release_notes.contains("Release notes"));
        assert!(release_notes.contains("Release notes summary: release-notes-summary"));
        assert!(release_notes.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(release_notes.contains("Artifact validation: validate-artifact"));
        assert!(release_notes
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_notes.contains("Release summary: release-summary"));
        assert!(release_notes.contains("API stability posture:"));
        assert!(release_notes.contains("Deprecation policy:"));
        assert!(release_notes.contains("Release-specific coverage:"));
        assert!(release_notes.contains("selected asteroid coverage"));
        assert!(release_notes.contains("Selected asteroid evidence: 5 exact J2000 samples"));
        assert!(release_notes.contains("Selected asteroid batch parity: 5 requests across 5 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); frame mix: 3 ecliptic, 2 equatorial; batch/single parity preserved"));
        assert!(release_notes.contains("Selected asteroid equatorial evidence: 5 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros) using a mean-obliquity equatorial transform"));
        assert!(release_notes.contains("asteroid:433-Eros"));
        assert!(release_notes.contains("Validation reference points:"));
        assert!(release_notes.contains("Compatibility caveats:"));
        assert!(release_notes.contains("Bundle provenance:"));
        assert!(release_notes.contains("Rust compiler version"));
        assert!(release_notes_summary.contains("Release notes summary"));
        assert!(release_notes_summary.contains(
            "Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="
        ));
        assert!(release_notes_summary.contains("Release-specific coverage:"));
        assert!(release_notes_summary.contains(
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
        assert!(
            release_notes_summary.contains("API stability summary line: API stability posture:")
        );
        assert!(release_notes_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_notes_summary.lines().any(|line| {
            line == "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
        }));
        assert!(release_notes_summary.lines().any(|line| {
            line == "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        }));
        assert!(release_notes_summary.lines().any(|line| {
            line == "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        }));
        assert!(release_notes_summary.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(release_notes_summary.contains("Release notes: release-notes"));
        assert!(release_notes_summary.contains("Compatibility caveats: 2"));
        assert!(release_notes_summary
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(release_notes_summary
            .contains("Artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(release_notes_summary.contains("Packaged-artifact storage/reconstruction: Quantized linear segments stored in pleiades-compression artifact format; equatorial coordinates are reconstructed at runtime from stored channels"));
        assert_report_contains_exact_line(
            &release_notes_summary,
            &format!(
                "Packaged-artifact generation policy: {}",
                packaged_artifact_generation_policy_summary_for_report()
            ),
        );
        assert_report_contains_exact_line(
            &release_notes_summary,
            &format!(
                "Packaged-artifact generation residual bodies: {}",
                packaged_artifact_generation_residual_bodies_summary_for_report()
            ),
        );
        assert!(release_notes_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_notes_summary.contains("Workspace audit summary: workspace-audit-summary"));
        assert!(release_notes_summary.contains(&format!(
            "House code aliases: {}",
            house_code_aliases_summary
        )));
        assert!(release_summary.contains("Release summary"));
        assert!(release_summary.contains("API stability summary line: API stability posture:"));
        assert!(release_summary.contains(
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
        assert!(release_summary.contains(&format!(
            "House-code aliases: {}",
            current_compatibility_profile().house_code_alias_count()
        )));
        assert!(release_summary.contains(&format!(
            "House code aliases: {}",
            house_code_aliases_summary
        )));
        assert!(release_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(release_summary.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(release_summary.contains("JPL interpolation posture: source="));
        assert!(release_summary.contains(
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        ));
        assert!(release_summary.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        }));
        assert!(release_summary.contains(
            "Validation report summary: validation-report-summary / validation-summary / report-summary"
        ));
        assert!(release_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_summary.contains("Workspace audit summary: workspace-audit-summary"));
        assert!(release_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_summary.contains(
            "Packaged-artifact profile: byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]"
        ));
        assert!(release_summary.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported"
        ));
        assert!(release_summary.contains(
            "Packaged-artifact storage/reconstruction: Quantized linear segments stored in pleiades-compression artifact format; equatorial coordinates are reconstructed at runtime from stored channels"
        ));
        assert!(release_summary.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-prototype; target thresholds: prototype fit envelope recorded; scopes=luminaries, major planets, lunar points, selected asteroids, custom bodies; fit envelope:"));
        assert!(release_summary.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2; fit envelope:"));
        assert!(release_summary.contains(
            "Packaged-artifact generation manifest: Packaged artifact generation manifest:"
        ));
        assert!(release_summary.contains(
            "Artifact profile coverage: stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"
        ));
        assert!(release_summary.contains(&format!(
            "Packaged-artifact access: {}",
            format_packaged_artifact_access_summary()
        )));
        assert_report_contains_exact_line(
            &release_summary,
            &format!(
                "Packaged-artifact generation policy: {}",
                packaged_artifact_generation_policy_summary_for_report()
            ),
        );
        assert_report_contains_exact_line(
            &release_summary,
            &format!(
                "Packaged-artifact generation residual bodies: {}",
                packaged_artifact_generation_residual_bodies_summary_for_report()
            ),
        );
        assert_report_contains_exact_line(
            &release_summary,
            "Packaged lookup epoch policy: TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
        );
        assert!(release_summary.contains(
            "Packaged-artifact regeneration: Packaged artifact regeneration source: label=stage-5 packaged-data prototype"
        ));
        assert!(release_summary.contains("fit envelope:"));
        assert!(release_summary.contains("segment samples across"));
        assert!(release_summary.contains("checksum=0x"));
        assert!(release_summary.contains("generation policy: adjacent same-body linear segments"));
        assert!(release_summary.contains(&format!(
            "artifact version={}",
            pleiades_data::packaged_artifact_regeneration_summary_details().artifact_version
        )));
        assert!(release_profile_identifiers.contains(&format!(
            "Release profile identifiers: {}",
            release_profiles
        )));
        assert!(manifest.contains("release-profile-identifiers.txt"));
        assert!(release_summary.contains("Comparison envelope: samples:"));
        let comparison_report = compare_backends(
            &default_reference_backend(),
            &default_candidate_backend(),
            &release_grade_corpus(),
        )
        .expect("comparison should build");
        assert_report_contains_exact_line(
            &release_summary,
            &format!(
                "Comparison tail envelope: {}",
                comparison_tail_envelope(&comparison_report.samples)
                    .expect("comparison tail envelope should exist")
            ),
        );
        assert!(release_summary.contains("mean longitude delta:"));
        assert!(release_summary.contains("median longitude delta:"));
        assert!(release_summary.contains("mean latitude delta:"));
        assert!(release_summary.contains("median latitude delta:"));
        assert!(release_summary.contains("mean distance delta:"));
        assert!(release_summary.contains("median distance delta:"));
        assert!(release_summary.contains("ELP lunar capability: lunar capability summary:"));
        assert!(release_summary.contains(
            "ELP lunar request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        ));
        assert!(release_summary.contains(
            "ELP frame treatment: Geocentric ecliptic coordinates are produced directly from the truncated lunar series; equatorial coordinates are derived with a mean-obliquity transform"
        ));
        assert!(release_summary.contains(
            "lunar theory catalog: 1 entry, 1 selected entry; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
        assert!(release_summary.contains(
            "lunar source selection: Compact Meeus-style truncated lunar baseline [selected key: source identifier=meeus-style-truncated-lunar-baseline; family key: source family=Meeus-style truncated analytical baseline; family: Meeus-style truncated analytical baseline]; aliases: Meeus-style truncated lunar baseline"
        ));
        assert!(release_summary.contains(
            "lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies, TT requests=5, TDB requests=4, order=preserved, single-query parity=preserved"
        ));
        assert!(release_summary.contains(
            "lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"
        ));
        assert!(release_summary.contains("JPL independent hold-out:"));
        assert!(release_summary.contains("Reference/hold-out overlap:"));
        assert!(release_summary.contains(&independent_holdout_source_summary_for_report()));
        assert_report_contains_exact_line(
            &release_summary,
            &independent_holdout_snapshot_source_window_summary_for_report(),
        );
        assert!(release_summary.contains(&independent_holdout_manifest_summary_for_report()));
        assert!(release_summary.contains("JPL independent hold-out equatorial parity:"));
        assert!(release_summary.contains("JPL independent hold-out batch parity:"));
        assert_report_contains_exact_line(
            &release_summary,
            "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false",
        );
        assert_report_contains_exact_line(
            &release_summary,
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant",
        );
        assert!(release_summary.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
        assert!(release_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
        assert!(release_summary.contains("JPL frame treatment: checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"));
        assert!(release_summary.contains(
            "JPL reference snapshot equatorial parity: 195 rows across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
        assert!(release_summary.contains(
            "JPL reference snapshot batch parity: 195 rows across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
        assert!(release_summary.contains("JPL production-generation coverage:"));
        assert!(release_summary.contains("JPL production-generation source windows:"));
        assert!(release_summary.contains("JPL production-generation body-class coverage:"));
        assert!(release_summary.contains("JPL production-generation boundary overlay:"));
        assert!(release_summary.contains("JPL production-generation boundary body-class coverage:"));
        assert!(release_summary.contains("JPL production-generation boundary windows:"));
        assert!(release_summary.contains("JPL production-generation boundary request corpus:"));
        assert!(release_summary.contains("Production generation boundary overlay source:"));
        assert!(release_summary.contains("Source-backed backend evidence:"));
        assert!(release_summary.contains("Selected asteroid evidence:"));
        assert!(release_summary.contains("Selected asteroid batch parity:"));
        assert!(release_summary.contains("Selected asteroid source windows:"));
        assert!(release_summary.contains("Reference snapshot coverage:"));
        assert!(release_summary.contains("Reference snapshot body-class coverage: major bodies: 134 rows across 10 bodies and 17 epochs; major windows: "));
        assert!(release_summary.contains(&reference_snapshot_high_curvature_summary_for_report()));
        assert!(release_summary
            .contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
        assert!(release_summary
            .contains(&reference_snapshot_high_curvature_window_summary_for_report()));
        assert!(
            release_summary.contains(&comparison_snapshot_body_class_coverage_summary_for_report())
        );
        assert!(release_summary.contains(
            "selected asteroids: 61 rows across 5 bodies and 13 epochs; asteroid windows: "
        ));
        assert!(release_summary.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
        assert!(release_summary.contains(&reference_snapshot_source_summary_for_report()));
        assert!(release_summary.contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(release_summary.contains("Selected asteroid evidence:"));
        assert!(release_summary.contains("Selected asteroid equatorial evidence:"));
        assert!(release_summary.contains("Comparison snapshot coverage:"));
        assert!(release_summary.contains("VSOP87 evidence:"));
        assert!(release_summary.contains("VSOP87 source-backed body-class envelopes:"));
        assert!(release_summary.contains("VSOP87 canonical J2000 equatorial body-class envelopes:"));
        assert!(release_summary.contains("Luminary: samples=1, bodies: Sun"));
        assert!(release_summary.contains("median Δlon="));
        assert!(release_summary.contains("p95 Δlon="));
        assert!(release_summary.contains("median Δlat="));
        assert!(release_summary.contains("p95 Δlat="));
        assert!(release_summary.contains("median Δdist="));
        assert!(release_summary.contains("p95 Δdist="));
        assert!(release_summary.contains("Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
        assert!(release_summary.contains("VSOP87 source documentation:"));
        assert!(release_summary.contains("VSOP87 frame treatment:"));
        assert!(release_summary.contains("VSOP87 request policy:"));
        assert!(release_summary.contains("VSOP87 source audit:"));
        assert!(release_summary.contains("VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"));
        assert!(release_summary.contains("VSOP87 generated binary audit:"));
        assert!(release_summary.contains("VSOP87 canonical J2000 source-backed evidence:"));
        assert!(release_summary.contains("VSOP87 canonical J2000 interim outliers: none"));
        assert!(release_summary.contains("VSOP87 canonical J2000 equatorial companion evidence:"));
        assert!(release_summary.contains("VSOP87 canonical J2000 batch parity:"));
        assert!(release_summary.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
        assert!(release_summary.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
        assert!(release_summary.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
        assert!(release_summary.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
        assert!(release_summary.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
        assert!(release_summary.contains("VSOP87 canonical J1900 batch parity:"));
        assert!(release_summary.contains("VSOP87 source-backed body evidence:"));
        assert!(release_summary.contains("Lunar reference: lunar reference evidence:"));
        assert!(release_summary.contains("Lunar reference envelope:"));
        assert!(release_summary
            .contains("Lunar equatorial reference: lunar equatorial reference evidence:"));
        assert!(release_summary.contains("Lunar equatorial reference envelope:"));
        assert!(release_summary.contains("Lunar source windows: lunar source windows: 7 exact Moon samples across 1 bodies in 2 exact windows; 4 reference-only apparent Moon samples across 1 bodies in 4 apparent windows"));
        assert!(release_summary
            .contains("Lunar apparent comparison: lunar apparent comparison evidence:"));
        assert!(release_summary.contains("Lunar high-curvature continuity evidence"));
        assert!(release_summary.contains("Lunar high-curvature equatorial continuity evidence"));
        assert!(release_summary.contains("|Δlon| mean/median/p95="));
        assert!(release_summary.contains("|ΔDec| mean/median/p95="));
        assert!(release_summary.contains(&packaged_request_policy_summary_for_report()));
        assert!(release_summary.contains(&format!(
            "Packaged lookup epoch policy: {}",
            packaged_lookup_epoch_policy_summary_for_report()
        )));
        assert!(release_summary.contains(&packaged_mixed_tt_tdb_batch_parity_summary_for_report()));
        assert!(release_summary.contains(&format!(
            "Packaged frame treatment: {}",
            packaged_frame_treatment_summary_for_report()
        )));
        assert!(release_summary.contains("Packaged batch parity:"));
        assert!(release_summary.contains("Packaged frame parity:"));
        assert!(release_summary.contains("Artifact boundary envelope:"));
        assert!(release_summary.contains(
            &artifact_inspection_summary_for_report()
                .expect("artifact inspection summary should build")
        ));
        assert!(release_summary.contains("residual-bearing bodies: Moon"));
        assert!(release_summary.contains("applies to 11 bundled bodies"));
        assert!(release_summary.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(release_summary
            .lines()
            .any(|line| line == "Release notes summary: release-notes-summary"));
        assert!(artifact_summary.contains("Artifact summary"));
        assert!(packaged_artifact_generation_manifest
            .contains("Packaged artifact generation manifest:"));
        assert!(artifact_summary.contains("residual-bearing segments: 6"));
        assert!(artifact_summary.contains("residual-bearing bodies: Moon"));
        assert!(artifact_summary.contains("Body classes: luminaries=2; major planets=8; lunar points=0; built-in asteroids=0; custom bodies=1; other bodies=0"));
        assert!(artifact_summary.contains(
            "Artifact profile: byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"
        ));
        assert!(artifact_summary.contains("Generation manifest:"));
        assert!(artifact_summary.contains("Packaged artifact generation manifest:"));
        assert!(artifact_summary.contains("Artifact request policy"));
        assert!(artifact_summary.contains(
            "Artifact storage: Quantized linear segments stored in pleiades-compression artifact format; equatorial coordinates are reconstructed at runtime from stored channels"
        ));
        assert!(artifact_summary.contains(
            "regeneration provenance: Packaged artifact regeneration source: label=stage-5 packaged-data prototype"
        ));
        assert!(artifact_summary.contains("fit envelope:"));
        assert!(artifact_summary.contains("segment samples across"));
        assert!(artifact_summary.contains("checksum=0x"));
        assert!(artifact_summary.contains("generation policy: adjacent same-body linear segments"));
        assert!(artifact_summary.contains("residual bodies: Moon; applies to 1 bundled body"));
        assert!(artifact_summary.contains(&format!(
            "artifact version={}",
            pleiades_data::packaged_artifact_regeneration_summary_details().artifact_version
        )));
        assert!(artifact_summary.lines().any(|line| {
            line == format!(
                "  Packaged frame treatment: {}",
                packaged_frame_treatment_summary_for_report()
            )
        }));
        assert!(artifact_summary.contains("applies to 11 bundled bodies"));
        assert!(artifact_summary.contains("Model error envelope"));
        assert!(artifact_summary.contains("mean longitude delta:"));
        assert!(artifact_summary.contains("median longitude delta:"));
        assert!(artifact_summary.contains("95th percentile longitude delta:"));
        assert!(artifact_summary.contains("rms longitude delta:"));
        assert!(artifact_summary.contains("mean latitude delta:"));
        assert!(artifact_summary.contains("median latitude delta:"));
        assert!(artifact_summary.contains("95th percentile latitude delta:"));
        assert!(artifact_summary.contains("rms latitude delta:"));
        assert!(artifact_summary.contains("mean distance delta:"));
        assert!(artifact_summary.contains("median distance delta:"));
        assert!(artifact_summary.contains("95th percentile distance delta:"));
        assert!(artifact_summary.contains("rms distance delta:"));
        assert!(artifact_summary.contains("Expected tolerance status"));
        assert!(artifact_summary.contains("Comparison tolerance audit"));
        assert!(artifact_summary.contains("bodies checked:"));
        assert!(artifact_summary.contains("Artifact lookup benchmark"));
        assert!(artifact_summary.contains("ns/lookup="));
        assert!(artifact_summary.contains("lookups/s="));
        assert!(artifact_summary.contains("Artifact decode benchmark"));
        assert!(artifact_summary.contains("ns/decode="));
        assert!(artifact_summary.contains("decodes/s="));
        assert!(artifact_summary.contains("Release summary: release-summary"));
        assert!(artifact_summary.contains("Release notes summary: release-notes-summary"));
        assert!(artifact_summary.contains("Workspace audit: workspace-audit / audit"));
        assert!(artifact_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(artifact_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(artifact_summary.contains("Release checklist summary: release-checklist-summary"));
        assert!(artifact_summary.contains("Release bundle verification: verify-release-bundle"));
        assert!(artifact_summary.contains(
            "custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus"
        ));
        assert!(release_checklist.contains("Release checklist"));
        assert!(release_checklist.contains("Repository-managed release gates:"));
        assert!(release_checklist
            .contains("[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile"));
        assert!(release_checklist
            .contains("[x] cargo run -q -p pleiades-validate -- validate-artifact"));
        assert!(release_checklist.contains("Manual bundle workflow:"));
        assert!(release_checklist.contains("bundle-release --out /tmp/pleiades-release"));
        assert!(release_checklist.contains("verify-release-bundle --out /tmp/pleiades-release"));
        assert!(release_checklist.contains("docs/release-reproducibility.md"));
        assert!(release_checklist.contains("Bundle contents:"));
        assert!(release_checklist.contains("compatibility-profile-summary.txt"));
        assert!(release_checklist.contains("release-notes-summary.txt"));
        assert!(release_checklist.contains("workspace-audit-summary.txt"));
        assert!(release_checklist.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(release_checklist.contains("API stability summary: api-stability-summary"));
        assert!(release_checklist.contains("release-summary.txt"));
        assert!(release_checklist.contains("release-checklist-summary.txt"));
        assert!(release_checklist.contains("bundle-manifest.checksum.txt"));
        assert!(release_checklist_summary.contains("Release checklist summary"));
        assert!(release_checklist_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(release_checklist_summary.contains("Artifact validation: validate-artifact"));
        assert!(
            release_checklist_summary.contains("Workspace audit summary: workspace-audit-summary")
        );
        assert!(release_checklist_summary.contains("Workspace audit: workspace-audit / audit"));
        assert!(release_checklist_summary.contains("Repository-managed release gates: 7 items"));
        assert!(release_checklist_summary.contains("Manual bundle workflow: 3 items"));
        assert!(release_checklist_summary.contains("Bundle contents: 17 items"));
        assert!(release_checklist_summary.contains("External publishing reminders: 3 items"));
        assert!(backend_matrix.contains("Implemented backend matrices"));
        assert!(backend_matrix.contains("JPL snapshot reference backend"));
        assert!(backend_matrix.contains(&format!(
            "House code aliases: {}",
            compatibility_profile.house_code_aliases_summary_line()
        )));
        assert!(backend_matrix_summary.contains("Backend matrix summary"));
        assert!(backend_matrix_summary.contains(&format!(
            "House code aliases: {}",
            compatibility_profile.house_code_aliases_summary_line()
        )));
        assert!(backend_matrix_summary.contains(
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        ));
        assert!(
            backend_matrix_summary.lines().any(|line| {
                line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
            })
        );
        assert!(backend_matrix_summary.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(backend_matrix_summary.contains("Backends: 5"));
        assert!(backend_matrix_summary.contains("Algorithmic: 2"));
        assert!(backend_matrix_summary.contains("Composite: 1"));
        assert!(backend_matrix_summary.contains(
            "JPL reference snapshot equatorial parity: 195 rows across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
        assert!(backend_matrix_summary
            .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(backend_matrix_summary.contains("VSOP87 canonical J2000 interim outliers: none"));
        assert!(backend_matrix_summary
            .contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
        assert!(
            backend_matrix_summary.contains("Selected asteroid evidence: 5 exact J2000 samples")
        );
        assert!(backend_matrix_summary.contains("Selected asteroid batch parity:"));
        assert!(backend_matrix_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(backend_matrix.contains(
            "selected asteroid coverage: 5 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        ));
        assert!(backend_matrix.contains(&selected_asteroid_boundary_summary_for_report()));
        assert!(backend_matrix.contains("exact J2000 evidence: 5 bodies at JD 2451545.0"));
        assert!(api_stability.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(api_stability_summary.contains("API stability summary"));
        assert!(api_stability_summary.contains(&format!(
            "Profile: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(api_stability_summary.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(validation_report_summary.contains("Validation report summary"));
        assert!(validation_report_summary.contains("Comparison corpus"));
        assert!(validation_report_summary.contains("Body-class tolerance posture"));
        assert!(validation_report_summary.contains("Tolerance policy catalog"));
        assert!(validation_report_summary.contains("Expected tolerance status"));
        assert!(validation_report_summary.contains("VSOP87 source-backed evidence"));
        assert!(validation_report_summary.contains("VSOP87 source-backed body-class envelopes:"));
        assert!(validation_report_summary
            .contains("VSOP87 canonical J2000 equatorial body-class envelopes:"));
        assert!(workspace_audit_summary.contains("Workspace audit summary"));
        assert!(
            workspace_audit_summary.contains("Result: no mandatory native build hooks detected")
        );
        assert!(validation_report_summary.contains("VSOP87 source documentation: 8 source specs, 8 source-backed body profiles, 1 fallback mean-element body profile (Pluto); source-backed bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
        assert!(validation_report_summary.contains(
            "source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"
        ));
        assert!(validation_report_summary.contains(
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        ));
        assert!(validation_report_summary.contains("VSOP87 request policy:"));
        assert!(validation_report_summary.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
        assert!(validation_report_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
        assert!(validation_report_summary
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
        assert!(validation_report_summary
            .contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files"));
        assert!(validation_report_summary
            .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(validation_report_summary.contains("VSOP87 canonical J2000 batch parity:"));
        assert!(validation_report_summary
            .contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
        assert!(validation_report_summary
            .contains("VSOP87 supported-body J2000 equatorial batch parity:"));
        assert!(validation_report_summary
            .contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
        assert!(validation_report_summary
            .contains("VSOP87 supported-body J1900 equatorial batch parity:"));
        assert!(validation_report_summary.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
        assert!(validation_report_summary
            .contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
        assert!(validation_report_summary.contains("VSOP87 canonical J1900 batch parity:"));
        assert!(validation_report_summary.contains("generated binary VSOP87B"));
        assert!(validation_report_summary.contains("VSOP87 source-backed evidence"));
        assert!(validation_report_summary.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
        assert!(validation_report_summary
            .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(validation_report_summary.contains(
            "ELP lunar theory specification: Compact Meeus-style truncated lunar baseline [meeus-style-truncated-lunar-baseline; family: Meeus-style truncated analytical baseline; selected key: source identifier=meeus-style-truncated-lunar-baseline]"
        ));
        assert!(validation_report_summary.contains(
            "lunar source selection: Compact Meeus-style truncated lunar baseline [selected key: source identifier=meeus-style-truncated-lunar-baseline; family key: source family=Meeus-style truncated analytical baseline; family: Meeus-style truncated analytical baseline]; aliases: Meeus-style truncated lunar baseline"
        ));
        assert!(validation_report_summary.contains(
            "lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies, TT requests=5, TDB requests=4, order=preserved, single-query parity=preserved"
        ));
        assert!(validation_report_summary.contains(
            "lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"
        ));
        assert!(
            validation_report_summary.contains("ELP lunar capability: lunar capability summary:")
        );
        assert!(validation_report_summary.contains(
            "ELP lunar request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        ));
        assert!(validation_report_summary.contains(
            "ELP frame treatment: Geocentric ecliptic coordinates are produced directly from the truncated lunar series; equatorial coordinates are derived with a mean-obliquity transform"
        ));
        assert!(validation_report_summary.contains(
            "lunar theory catalog: 1 entry, 1 selected entry; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
        assert!(validation_report_summary.contains(
            "lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; selected key: source identifier=meeus-style-truncated-lunar-baseline; selected family key: source family=Meeus-style truncated analytical baseline; aliases=1; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)"
        ));
        assert!(validation_report_summary
            .contains("lunar reference error envelope: 9 samples across 5 bodies"));
        assert!(validation_report_summary.contains("max Δlon="));
        assert!(validation_report_summary.contains("mean Δlon="));
        assert!(validation_report_summary.contains("median Δlon="));
        assert!(validation_report_summary.contains("p95 Δlon="));
        assert!(validation_report_summary.contains("max Δlat="));
        assert!(validation_report_summary.contains("mean Δlat="));
        assert!(validation_report_summary.contains("median Δlat="));
        assert!(validation_report_summary.contains("p95 Δlat="));
        assert!(validation_report_summary.contains("limits: Δlon≤1e-4°"));
        assert!(validation_report_summary.contains("request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"));
        assert!(validation_report_summary
            .contains("lookup epoch policy=TT-grid retag without relativistic correction"));
        assert!(validation_report_summary
            .contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
        assert!(validation_report_summary.contains("date-range note: Validated against the published 1992-04-12 geocentric Moon example, the published 1992-04-12 geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform, the reference-only published 1968-12-24 apparent geocentric Moon comparison datum, the reference-only published 2004-04-01 NASA RP 1349 apparent Moon table row, the reference-only published 2006-09-07 EclipseWise apparent Moon coordinate row, J2000 lunar-point anchors including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example"));
        assert!(validation_report_summary
            .contains("lunar equatorial reference error envelope: 3 samples across 1 bodies"));
        assert!(validation_report_summary.contains("mean ΔRA="));
        assert!(validation_report_summary.contains("median ΔRA="));
        assert!(validation_report_summary.contains("p95 ΔRA="));
        assert!(validation_report_summary.contains("limits: ΔRA≤1e-2°"));
        assert!(validation_report_summary.contains("citation: Jean Meeus"));
        assert!(validation_report_summary
            .contains("provenance: Published lunar position, node, and mean-point formulas"));
        assert!(validation_report_summary.contains(
            "redistribution: No external coefficient-file redistribution constraints apply"
        ));
        assert!(validation_report_summary
            .contains("license: The current baseline is handwritten pure Rust"));
        assert!(
            validation_report_summary.contains("2 unsupported bodies: True Apogee, True Perigee")
        );
        assert!(report.contains("Validation report"));
        assert!(report.contains("Expected tolerance status"));
        assert!(report.contains("margin Δlon="));
        assert!(report.contains("margin Δdist="));
        assert!(manifest.contains("Release bundle manifest"));
        assert!(manifest.contains("validation rounds: 1"));
        assert!(manifest.contains("compatibility-profile.txt"));
        assert!(manifest.contains("compatibility-profile-summary.txt"));
        assert!(manifest.contains("release-notes.txt"));
        assert!(manifest.contains("release-notes-summary.txt"));
        assert!(manifest.contains("backend-matrix.txt"));
        assert!(manifest.contains("backend-matrix-summary.txt"));
        assert!(manifest.contains("api-stability.txt"));
        assert!(manifest.contains("api-stability-summary.txt"));
        assert!(manifest.contains("comparison-envelope-summary.txt"));
        assert!(manifest.contains("validation-report-summary.txt"));
        assert!(manifest.contains("artifact-summary.txt"));
        assert!(manifest.contains("packaged-artifact-generation-manifest.txt"));
        assert!(manifest.contains("benchmark-report.txt"));
        assert!(manifest.contains("validation-report.txt"));
        assert!(!manifest.contains("bundle-manifest.checksum.txt"));
        assert!(manifest.contains("source revision:"));
        assert!(manifest.contains("workspace status:"));
        assert!(manifest.contains("rustc version:"));
        assert!(manifest.contains("profile checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("profile summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release notes checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release notes summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release-profile identifiers: release-profile-identifiers.txt"));
        assert!(manifest.contains("release-profile identifiers checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release checklist checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release checklist summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("backend matrix checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("backend matrix summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("api stability checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("comparison-envelope summary checksum (fnv1a-64): 0x"));
        assert!(
            manifest.contains("comparison-corpus release-guard summary checksum (fnv1a-64): 0x")
        );
        assert!(manifest.contains("validation report summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("workspace audit summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("artifact summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("benchmark report checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("validation report checksum (fnv1a-64): 0x"));
        assert!(manifest_checksum.trim().starts_with("0x"));

        let verified = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect("bundle verification should render");
        assert!(verified.contains("Release bundle"));
        assert!(verified.contains("bundle-manifest.txt"));
        assert!(verified.contains("bundle-manifest.checksum.txt"));
        assert!(verified.contains("compatibility-profile-summary.txt"));
        assert!(verified.contains("release-notes-summary.txt"));
        assert!(verified.contains("release-summary.txt"));
        assert!(verified.contains("release-checklist-summary.txt"));
        assert!(verified.contains("comparison-envelope-summary.txt"));
        assert!(verified.contains("comparison-corpus-release-guard-summary.txt"));
        assert!(verified.contains("validation-report-summary.txt"));
        assert!(verified.contains("artifact-summary.txt"));
        assert!(verified.contains("benchmark-report.txt"));
        assert!(verified.contains("source revision:"));
        assert!(verified.contains("workspace status:"));
        assert!(verified.contains("rustc version:"));
        assert!(verified.contains("validation rounds: 1"));
        assert!(verified.contains("release notes checksum: 0x"));
        assert!(verified.contains("release notes summary checksum: 0x"));
        assert!(verified.contains("release-profile identifiers checksum: 0x"));
        assert!(verified.contains("release checklist checksum: 0x"));
        assert!(verified.contains("release checklist summary checksum: 0x"));
        assert!(verified.contains("backend matrix checksum: 0x"));
        assert!(verified.contains("backend matrix summary checksum: 0x"));
        assert!(verified.contains("comparison-envelope summary checksum: 0x"));
        assert!(verified.contains("comparison-corpus release-guard summary checksum: 0x"));
        assert!(verified.contains("validation report summary checksum: 0x"));
        assert!(verified.contains("workspace audit summary checksum: 0x"));
        assert!(verified.contains("artifact summary checksum: 0x"));
        assert!(verified.contains("validation report checksum: 0x"));
        assert!(verified.contains("manifest checksum bytes:"));
        assert!(verified.contains("manifest checksum: 0x"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_missing_source_revision_entry() {
        assert_release_bundle_rejects_missing_manifest_entry(
            "pleiades-release-bundle-missing-source-revision",
            "source revision:",
            &[
                "missing manifest entry: source revision:",
                "missing source revision entry",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_missing_workspace_status_entry() {
        assert_release_bundle_rejects_missing_manifest_entry(
            "pleiades-release-bundle-missing-workspace-status",
            "workspace status:",
            &[
                "missing manifest entry: workspace status:",
                "missing workspace status entry",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_missing_rustc_version_entry() {
        assert_release_bundle_rejects_missing_manifest_entry(
            "pleiades-release-bundle-missing-rustc",
            "rustc version:",
            &[
                "missing manifest entry: rustc version:",
                "missing rustc version entry",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_blank_source_revision_entry() {
        assert_release_bundle_rejects_blank_manifest_value(
            "pleiades-release-bundle-blank-source-revision",
            "source revision:",
            &["missing source revision entry"],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_blank_workspace_status_entry() {
        assert_release_bundle_rejects_blank_manifest_value(
            "pleiades-release-bundle-blank-workspace-status",
            "workspace status:",
            &["missing workspace status entry"],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_blank_rustc_version_entry() {
        assert_release_bundle_rejects_blank_manifest_value(
            "pleiades-release-bundle-blank-rustc",
            "rustc version:",
            &["missing rustc version entry"],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_source_revision_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-source-revision",
            "source revision:",
            &[
                "duplicate entry: source revision:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_workspace_status_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-workspace-status",
            "workspace status:",
            &[
                "duplicate entry: workspace status:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_rustc_version_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-rustc",
            "rustc version:",
            &[
                "duplicate entry: rustc version:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_blank_profile_id_entry() {
        assert_release_bundle_rejects_blank_manifest_value(
            "pleiades-release-bundle-blank-profile-id",
            "profile id:",
            &["missing profile id entry"],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_profile_id_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-profile-id",
            "profile id:",
            &[
                "duplicate entry: profile id:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_api_stability_posture_id_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-api-stability-posture-id",
            "api stability posture id:",
            &[
                "duplicate entry: api stability posture id:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_validation_rounds_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-validation-rounds",
            "validation rounds:",
            &[
                "duplicate entry: validation rounds:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_release_summary_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-release-summary",
            "release summary:",
            &[
                "duplicate entry: release summary:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_duplicate_release_notes_summary_entry() {
        assert_release_bundle_rejects_duplicate_manifest_entry(
            "pleiades-release-bundle-duplicate-release-notes-summary",
            "release notes summary:",
            &[
                "duplicate entry: release notes summary:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_whitespace_source_revision_entry() {
        assert_release_bundle_rejects_whitespace_manifest_entry(
            "pleiades-release-bundle-whitespace-source-revision",
            "source revision:",
            &[
                "unexpected leading or trailing whitespace in manifest entry: source revision:",
                "release bundle verification failed",
            ],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_blank_api_stability_posture_id_entry() {
        assert_release_bundle_rejects_blank_manifest_value(
            "pleiades-release-bundle-blank-api-stability-posture-id",
            "api stability posture id:",
            &["missing API stability posture id entry"],
        );
    }

    #[test]
    fn verify_release_bundle_rejects_checksum_mismatches() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "profile checksum (fnv1a-64):",
            "profile checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a corrupted manifest");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid profile checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_manifest_file() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-manifest");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let tampered = manifest.replace("validation rounds: 1", "validation rounds: 2");
        std::fs::write(&manifest_path, tampered).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a tampered manifest file");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("bundle manifest checksum mismatch")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_invalid_validation_rounds() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-invalid-validation-rounds");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let tampered_manifest = manifest.replace("validation rounds: 1", "validation rounds: nope");
        std::fs::write(&manifest_path, &tampered_manifest).expect("manifest should be writable");

        let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
        std::fs::write(
            &checksum_path,
            format!("0x{:016x}\n", checksum64(&tampered_manifest)),
        )
        .expect("manifest checksum sidecar should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for invalid validation rounds");
        assert!(error.contains("release bundle verification failed"));
        assert!(error.contains("invalid validation rounds entry"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn release_bundle_validate_accepts_rendered_bundle() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-validate-accepts");
        let bundle = render_release_bundle(1, &bundle_dir).expect("release bundle should render");
        bundle
            .validate()
            .expect("rendered release bundle should validate");

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn release_bundle_validate_rejects_whitespace_padded_provenance() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-provenance-padding");
        let mut bundle =
            render_release_bundle(1, &bundle_dir).expect("release bundle should render");
        let source_revision = bundle.source_revision.clone();
        bundle.source_revision = format!(" {source_revision} ");

        let error = bundle
            .validate()
            .expect_err("padded provenance should be rejected by bundle validation");
        let error = error.to_string();
        assert!(error.contains("invalid source revision entry"));
        assert!(error.contains("unexpected leading or trailing whitespace"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn release_bundle_validate_rejects_placeholder_provenance() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-provenance-placeholder");
        let mut bundle =
            render_release_bundle(1, &bundle_dir).expect("release bundle should render");
        bundle.rustc_version = "unknown".to_string();

        let error = bundle
            .validate()
            .expect_err("placeholder provenance should be rejected by bundle validation");
        let error = error.to_string();
        assert!(error.contains("invalid rustc version entry"));
        assert!(error.contains("placeholder values are not allowed"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn release_bundle_validate_rejects_multiline_provenance() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-provenance-multiline");
        let mut bundle =
            render_release_bundle(1, &bundle_dir).expect("release bundle should render");
        bundle.workspace_status = "clean\nmodified".to_string();

        let error = bundle
            .validate()
            .expect_err("multiline provenance should be rejected by bundle validation");
        let error = error.to_string();
        assert!(error.contains("invalid workspace status entry"));
        assert!(error.contains("unexpected line break"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn release_bundle_validate_rejects_manifest_path_drift() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-manifest-path-drift");
        let mut bundle =
            render_release_bundle(1, &bundle_dir).expect("release bundle should render");
        bundle.manifest_path = bundle.output_dir.join("bundle-manifest-drift.txt");

        let error = bundle
            .validate()
            .expect_err("path drift should be rejected by bundle validation");
        let error = error.to_string();
        assert!(error.contains("unexpected bundle manifest file path"));
        assert!(error.contains("bundle-manifest.txt"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_manifest_checksum_sidecar() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-manifest-checksum");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
        let tampered = "0x0000000000000000\n";
        std::fs::write(&checksum_path, tampered).expect("manifest checksum should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a tampered manifest checksum sidecar");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("bundle manifest checksum mismatch")
                || error.contains("invalid bundle manifest checksum sidecar value")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_missing_manifest_checksum_sidecar() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-missing-manifest-checksum");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
        std::fs::remove_file(&checksum_path)
            .expect("manifest checksum sidecar should be removable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a missing manifest checksum sidecar");
        assert!(error.contains("release bundle verification failed"));
        assert!(error.contains("missing bundle manifest checksum sidecar file"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_malformed_manifest_checksum_sidecar() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-malformed-manifest-checksum");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
        std::fs::write(&checksum_path, " 0x0000000000000000 \n")
            .expect("manifest checksum sidecar should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a malformed manifest checksum sidecar");
        assert!(error.contains("release bundle verification failed"));
        assert!(error.contains("invalid bundle manifest checksum sidecar value"));
        assert!(error.contains("unexpected leading or trailing whitespace"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_noncanonical_manifest_checksum_sidecar() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-noncanonical-manifest-checksum");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
        std::fs::write(&checksum_path, "0x1\n")
            .expect("manifest checksum sidecar should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a noncanonical manifest checksum sidecar");
        assert!(error.contains("release bundle verification failed"));
        assert!(error.contains("invalid bundle manifest checksum sidecar value"));
        assert!(error.contains("expected exactly 16 lowercase hex digits"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_noncanonical_manifest_checksum_entry() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-noncanonical-manifest-entry");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let rewritten = manifest
            .lines()
            .map(|line| {
                line.strip_prefix("profile checksum (fnv1a-64): ")
                    .map(|value| {
                        let digits = value.strip_prefix("0x").unwrap_or(value);
                        format!(
                            "profile checksum (fnv1a-64): 0x{}",
                            digits.to_ascii_uppercase()
                        )
                    })
                    .unwrap_or_else(|| line.to_string())
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&manifest_path, format!("{rewritten}\n"))
            .expect("manifest should be writable");

        let checksum = checksum64(
            &std::fs::read_to_string(&manifest_path).expect("manifest should exist after rewrite"),
        );
        std::fs::write(
            bundle_dir.join("bundle-manifest.checksum.txt"),
            format!("0x{checksum:016x}\n"),
        )
        .expect("manifest checksum sidecar should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a noncanonical manifest checksum entry");
        assert!(error.contains("release bundle verification failed"));
        assert!(error.contains("invalid profile checksum (fnv1a-64): value"));
        assert!(error.contains("expected exactly 16 lowercase hex digits"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_unexpected_bundle_entries() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-extra-entry");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        std::fs::write(bundle_dir.join("unexpected.txt"), "spurious bundle content")
            .expect("unexpected file should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for unexpected bundle contents");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("unexpected release bundle directory contents")
        );
        assert!(error.contains("unexpected.txt"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    #[cfg(unix)]
    fn verify_release_bundle_rejects_symlinked_release_summary_file() {
        assert_release_bundle_rejects_symlinked_text_file(
            "pleiades-release-bundle-symlinked-release-summary",
            "release-summary.txt",
            "release-notes.txt",
            "unexpected non-regular release bundle file",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_unexpected_manifest_lines() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-unexpected-manifest-line");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let tampered = format!("{manifest}unexpected manifest note: review required\n");
        std::fs::write(&manifest_path, tampered).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for an unexpected manifest line");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("unexpected release bundle manifest line count")
        );
        assert!(error.contains("unexpected release bundle manifest line count"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_backend_matrix_checksum_mismatches() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-matrix");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "backend matrix checksum (fnv1a-64):",
            "backend matrix checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a corrupted backend matrix checksum");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid backend matrix checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_api_stability_summary_checksum_mismatches() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-api-summary");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "api stability summary checksum (fnv1a-64):",
            "api stability summary checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a corrupted API stability summary checksum");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid api stability summary checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_release_notes_checksum_mismatches() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-notes");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "release notes checksum (fnv1a-64):",
            "release notes checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a corrupted release notes checksum");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid release notes checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_compatibility_profile_summary_file() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-summary");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let summary_path = bundle_dir.join("compatibility-profile-summary.txt");
        let summary = std::fs::read_to_string(&summary_path)
            .expect("compatibility profile summary should exist");
        let tampered = summary.replace(
            "Compatibility profile summary",
            "Tampered compatibility profile summary",
        );
        std::fs::write(&summary_path, tampered).expect("summary should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a tampered compatibility profile summary");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("compatibility profile summary checksum mismatch")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_release_notes_file() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-notes");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let notes_path = bundle_dir.join("release-notes.txt");
        let mut notes = std::fs::read_to_string(&notes_path).expect("release notes should exist");
        notes.push_str("\nTampered for regression coverage.\n");
        std::fs::write(&notes_path, notes).expect("release notes should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a tampered release notes file");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("release notes checksum mismatch")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_backend_matrix_summary_file() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-matrix-summary");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let summary_path = bundle_dir.join("backend-matrix-summary.txt");
        let summary =
            std::fs::read_to_string(&summary_path).expect("backend matrix summary should exist");
        let tampered = summary.replace("Backend matrix summary", "Tampered backend matrix summary");
        std::fs::write(&summary_path, tampered).expect("backend matrix summary should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a tampered backend matrix summary");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("backend matrix summary checksum mismatch")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_release_summary_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-release-summary",
            "release-summary.txt",
            "release summary checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_release_notes_summary_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-release-notes-summary",
            "release-notes-summary.txt",
            "release notes summary checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_profile_alignment_helpers_accept_matching_release_text() {
        let release_profiles = current_release_profile_identifiers();
        let profile_id = release_profiles.compatibility_profile_id;
        let api_stability_posture_id = release_profiles.api_stability_profile_id;
        let release_notes_summary = format!(
            "Release notes summary\nProfile: {profile_id}\nAPI stability posture: {api_stability_posture_id}\n"
        );
        let release_summary = format!(
            "Release summary\nProfile: {profile_id}\nAPI stability posture: {api_stability_posture_id}\n"
        );
        let release_checklist = format!(
            "Release checklist\nProfile: {profile_id}\nAPI stability posture: {api_stability_posture_id}\n"
        );
        let release_profile_identifiers = format!(
            "Release profile identifiers: v1 compatibility={profile_id}, api-stability={api_stability_posture_id}\n"
        );

        ensure_release_profile_line_alignment(
            "release notes",
            &format!("Release notes\nProfile: {profile_id}\n"),
            profile_id,
        )
        .expect("matching release notes should verify");
        ensure_release_profile_summary_alignment(
            "release notes summary",
            &release_notes_summary,
            profile_id,
            api_stability_posture_id,
        )
        .expect("matching release notes summary should verify");
        ensure_release_profile_summary_alignment(
            "release summary",
            &release_summary,
            profile_id,
            api_stability_posture_id,
        )
        .expect("matching release summary should verify");
        ensure_release_profile_summary_alignment(
            "release checklist",
            &release_checklist,
            profile_id,
            api_stability_posture_id,
        )
        .expect("matching release checklist should verify");
        ensure_release_profile_identifiers_alignment(
            &release_profile_identifiers,
            profile_id,
            api_stability_posture_id,
        )
        .expect("matching release profile identifiers should verify");
    }

    #[test]
    fn verify_release_bundle_profile_alignment_helpers_reject_mismatched_release_text() {
        let release_profiles = current_release_profile_identifiers();
        let profile_id = release_profiles.compatibility_profile_id;
        let api_stability_posture_id = release_profiles.api_stability_profile_id;

        let error = ensure_release_profile_summary_alignment(
            "release summary",
            "Release summary\nProfile: incorrect-profile\nAPI stability posture: incorrect-api\n",
            profile_id,
            api_stability_posture_id,
        )
        .expect_err("mismatched release summary should fail");
        assert!(error
            .to_string()
            .contains("release bundle verification failed"));
        assert!(error
            .to_string()
            .contains("release summary profile id mismatch"));

        let error = ensure_release_profile_identifiers_alignment(
            "Release profile identifiers: v1 compatibility=incorrect-profile, api-stability=incorrect-api\n",
            profile_id,
            api_stability_posture_id,
        )
        .expect_err("mismatched release-profile identifiers should fail");
        assert!(error
            .to_string()
            .contains("release-profile identifiers mismatch"));
    }

    #[test]
    fn release_profile_identifier_summary_helper_rejects_drifted_pairs() {
        let compatibility_profile_id = current_compatibility_profile().profile_id;
        let release_profiles = ReleaseProfileIdentifiers {
            compatibility_profile_id,
            api_stability_profile_id: compatibility_profile_id,
        };

        let rendered = format_release_profile_identifiers_summary(&release_profiles);
        assert!(rendered.contains("unavailable"));
        assert!(rendered.contains("release-profile identifiers must be distinct"));
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_artifact_summary_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-artifact-summary",
            "artifact-summary.txt",
            "artifact summary checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_api_stability_summary_file() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-tampered-api-summary");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let summary_path = bundle_dir.join("api-stability-summary.txt");
        let summary =
            std::fs::read_to_string(&summary_path).expect("API stability summary should exist");
        let tampered = summary.replace("API stability summary", "Tampered API stability summary");
        std::fs::write(&summary_path, tampered).expect("API stability summary should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a tampered API stability summary");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("API stability summary checksum mismatch")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_compatibility_profile_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-profile",
            "compatibility-profile.txt",
            "compatibility profile checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_release_checklist_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-checklist",
            "release-checklist.txt",
            "release checklist checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_release_checklist_summary_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-checklist-summary",
            "release-checklist-summary.txt",
            "release checklist summary checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_backend_matrix_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-matrix",
            "backend-matrix.txt",
            "backend matrix checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_api_stability_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-api-stability",
            "api-stability.txt",
            "API stability checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_validation_report_summary_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-validation-report-summary",
            "validation-report-summary.txt",
            "validation report summary checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_validation_report_file() {
        assert_release_bundle_rejects_tampered_text_file(
            "pleiades-release-bundle-tampered-validation-report",
            "validation-report.txt",
            "validation report checksum mismatch",
        );
    }

    #[test]
    fn verify_release_bundle_rejects_release_checklist_checksum_mismatches() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-checklist");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "release checklist checksum (fnv1a-64):",
            "release checklist checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a corrupted release checklist checksum");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid release checklist checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_api_stability_checksum_mismatches() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-api-stability");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "api stability checksum (fnv1a-64):",
            "api stability checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a corrupted API stability checksum");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid api stability checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_validation_report_checksum_mismatches() {
        let bundle_dir = unique_temp_dir("pleiades-release-bundle-corrupt-validation-report");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "validation report checksum (fnv1a-64):",
            "validation report checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect_err("verification should fail for a corrupted validation report checksum");
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid validation report checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn verify_release_bundle_rejects_validation_report_summary_checksum_mismatches() {
        let bundle_dir =
            unique_temp_dir("pleiades-release-bundle-corrupt-validation-report-summary");
        let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
        render_cli(&[
            "bundle-release",
            "--out",
            &bundle_dir_string,
            "--rounds",
            "1",
        ])
        .expect("bundle release should render");

        let manifest_path = bundle_dir.join("bundle-manifest.txt");
        let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
        let corrupted = manifest.replace(
            "validation report summary checksum (fnv1a-64):",
            "validation report summary checksum (fnv1a-64): 0x0000000000000000 #",
        );
        std::fs::write(&manifest_path, corrupted).expect("manifest should be writable");

        let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string]).expect_err(
            "verification should fail for a corrupted validation report summary checksum",
        );
        assert!(
            error.contains("release bundle verification failed")
                || error.contains("invalid validation report summary checksum")
                || error.contains("missing 0x prefix")
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn artifact_validation_report_mentions_boundary_checks() {
        let report = render_artifact_report().expect("artifact report should render");
        assert!(report.contains("Artifact validation report"));
        assert!(report.contains("stage-5 packaged-data prototype"));
        assert!(report.contains("byte order: little-endian"));
        assert!(report.contains("roundtrip decode: ok"));
        assert!(report.contains("checksum verified: ok"));
        assert!(report.contains("Bodies"));
        assert!(report.contains("Sun"));
        assert!(report.contains("Moon"));
        assert!(report.contains("Jupiter"));
        assert!(report.contains("Pluto"));
        assert!(report.contains("boundary checks"));
        assert!(report.contains("Artifact boundary envelope"));
        assert!(report.contains("Artifact request policy"));
        assert!(report.contains("Model error envelope"));
        assert!(report.contains("Body-class error envelopes"));
        assert!(report.contains("Artifact decode benchmark"));
        assert!(report.contains("nanoseconds per decode:"));
        assert!(report.contains("decodes per second:"));
        let body_class_envelopes = report
            .split("Body-class error envelopes")
            .nth(1)
            .expect("artifact report should include body-class error envelopes");
        assert!(body_class_envelopes.contains("max longitude delta:"));
        assert!(body_class_envelopes.contains(" ("));
        assert!(report.contains("Luminaries"));
        assert!(report.contains("Major planets"));
        assert!(report.contains("baseline backend"));
    }

    #[test]
    fn comparison_summary_summary_line_includes_bodies_and_counts() {
        let summary = ComparisonSummary {
            sample_count: 3,
            max_longitude_delta_body: Some(CelestialBody::Sun),
            max_longitude_delta_deg: 0.123_456_789_012,
            mean_longitude_delta_deg: 0.012_345_678_901,
            rms_longitude_delta_deg: 0.023_456_789_012,
            max_latitude_delta_body: Some(CelestialBody::Moon),
            max_latitude_delta_deg: 0.223_456_789_012,
            mean_latitude_delta_deg: 0.032_345_678_901,
            rms_latitude_delta_deg: 0.043_456_789_012,
            max_distance_delta_body: Some(CelestialBody::Mars),
            max_distance_delta_au: Some(0.001_234_567_89),
            mean_distance_delta_au: Some(0.000_234_567_89),
            rms_distance_delta_au: Some(0.000_334_567_89),
        };

        let rendered = summary.summary_line();
        assert!(rendered.contains("samples: 3"));
        assert!(rendered.contains("max longitude delta: 0.123456789012° (Sun)"));
        assert!(rendered.contains("max latitude delta: 0.223456789012° (Moon)"));
        assert!(rendered.contains("max distance delta: 0.001234567890 AU (Mars)"));
        assert_eq!(rendered, format!("{summary}"));
        assert_eq!(summary.validated_summary_line(), Ok(rendered));
        assert!(summary.validate().is_ok());
    }

    #[test]
    fn comparison_summary_validated_summary_line_rejects_zero_sample_drift() {
        let summary = ComparisonSummary {
            sample_count: 0,
            max_longitude_delta_body: Some(CelestialBody::Sun),
            max_longitude_delta_deg: 0.123_456_789_012,
            mean_longitude_delta_deg: 0.012_345_678_901,
            rms_longitude_delta_deg: 0.023_456_789_012,
            max_latitude_delta_body: Some(CelestialBody::Moon),
            max_latitude_delta_deg: 0.223_456_789_012,
            mean_latitude_delta_deg: 0.032_345_678_901,
            rms_latitude_delta_deg: 0.043_456_789_012,
            max_distance_delta_body: Some(CelestialBody::Mars),
            max_distance_delta_au: Some(0.001_234_567_89),
            mean_distance_delta_au: Some(0.000_234_567_89),
            rms_distance_delta_au: Some(0.000_334_567_89),
        };

        let error = summary
            .validated_summary_line()
            .expect_err("zero-sample drift should fail validation");
        assert!(error.to_string().contains(
            "comparison summary with zero samples must not carry per-body or distance extrema"
        ));
    }

    #[test]
    fn comparison_envelope_formatter_rejects_empty_sample_slices() {
        let summary = ComparisonSummary {
            sample_count: 1,
            max_longitude_delta_body: Some(CelestialBody::Sun),
            max_longitude_delta_deg: 0.123_456_789_012,
            mean_longitude_delta_deg: 0.012_345_678_901,
            rms_longitude_delta_deg: 0.023_456_789_012,
            max_latitude_delta_body: Some(CelestialBody::Moon),
            max_latitude_delta_deg: 0.223_456_789_012,
            mean_latitude_delta_deg: 0.032_345_678_901,
            rms_latitude_delta_deg: 0.043_456_789_012,
            max_distance_delta_body: Some(CelestialBody::Mars),
            max_distance_delta_au: Some(0.001_234_567_89),
            mean_distance_delta_au: Some(0.000_234_567_89),
            rms_distance_delta_au: Some(0.000_334_567_89),
        };

        let envelope = format_comparison_envelope_for_report(&summary, &[]);
        assert!(envelope.contains("comparison envelope unavailable"));
        assert!(envelope.contains("sample-count mismatch") || envelope.contains("no samples"));

        let percentile = format_comparison_percentile_envelope_for_report(&[]);
        assert!(percentile.contains("comparison percentile envelope unavailable"));
        assert!(percentile.contains("comparison sample slice is empty"));
    }

    #[test]
    fn comparison_envelope_formatter_rejects_mixed_distance_channels() {
        let summary = ComparisonSummary {
            sample_count: 2,
            max_longitude_delta_body: Some(CelestialBody::Sun),
            max_longitude_delta_deg: 0.123_456_789_012,
            mean_longitude_delta_deg: 0.012_345_678_901,
            rms_longitude_delta_deg: 0.023_456_789_012,
            max_latitude_delta_body: Some(CelestialBody::Moon),
            max_latitude_delta_deg: 0.223_456_789_012,
            mean_latitude_delta_deg: 0.032_345_678_901,
            rms_latitude_delta_deg: 0.043_456_789_012,
            max_distance_delta_body: Some(CelestialBody::Mars),
            max_distance_delta_au: Some(0.001_234_567_89),
            mean_distance_delta_au: Some(0.000_234_567_89),
            rms_distance_delta_au: Some(0.000_334_567_89),
        };
        let samples = vec![
            ComparisonSample {
                body: CelestialBody::Sun,
                reference: EclipticCoordinates::new(
                    Longitude::from_degrees(10.0),
                    Latitude::from_degrees(1.0),
                    Some(1.0),
                ),
                candidate: EclipticCoordinates::new(
                    Longitude::from_degrees(10.1),
                    Latitude::from_degrees(1.1),
                    Some(1.1),
                ),
                longitude_delta_deg: 0.1,
                latitude_delta_deg: 0.1,
                distance_delta_au: Some(0.1),
            },
            ComparisonSample {
                body: CelestialBody::Moon,
                reference: EclipticCoordinates::new(
                    Longitude::from_degrees(20.0),
                    Latitude::from_degrees(2.0),
                    None,
                ),
                candidate: EclipticCoordinates::new(
                    Longitude::from_degrees(20.2),
                    Latitude::from_degrees(2.2),
                    None,
                ),
                longitude_delta_deg: 0.2,
                latitude_delta_deg: 0.2,
                distance_delta_au: None,
            },
        ];

        let envelope = format_comparison_envelope_for_report(&summary, &samples);
        assert!(
            envelope.contains("comparison envelope unavailable")
                || envelope.contains("distance deltas must either all be present or all be absent")
        );
    }

    #[test]
    fn mean_obliquity_frame_round_trip_summary_has_a_displayable_summary_line() {
        let summary = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");

        assert_eq!(summary.summary_line(), summary.to_string());
        assert!(summary.summary_line().contains("6 samples"));
        assert!(summary.summary_line().contains("max |Δlon|="));
        assert!(summary.summary_line().contains("mean |Δlon|="));
        assert!(summary.summary_line().contains("p95 |Δlon|="));
        assert!(summary.validate().is_ok());
    }

    #[test]
    fn mean_obliquity_frame_round_trip_summary_render_cli_command_matches_display() {
        let rendered = render_cli(&["mean-obliquity-frame-round-trip-summary"])
            .expect("frame round-trip summary command should render");
        let summary = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");

        assert_eq!(rendered, summary.to_string());
    }

    #[test]
    fn mean_obliquity_frame_round_trip_sample_corpus_matches_the_canonical_summary() {
        let samples = mean_obliquity_frame_round_trip_sample_corpus();

        assert_eq!(samples.len(), 6);
        assert!(samples
            .iter()
            .any(|(coordinates, _)| coordinates.latitude.degrees() > 80.0));
        assert!(samples
            .iter()
            .any(|(coordinates, _)| coordinates.longitude.degrees() > 350.0));
        assert_eq!(
            samples
                .first()
                .expect("sample corpus should not be empty")
                .1
                .scale,
            TimeScale::Tt
        );
        assert_eq!(
            samples
                .last()
                .expect("sample corpus should not be empty")
                .1
                .scale,
            TimeScale::Tt
        );

        let summary = mean_obliquity_frame_round_trip_summary_from_samples(&samples)
            .expect("canonical sample corpus should remain valid");
        assert_eq!(summary, mean_obliquity_frame_round_trip_summary().unwrap());
    }

    #[test]
    fn mean_obliquity_frame_round_trip_summary_formatter_rejects_drift() {
        let mut summary = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");
        summary.sample_count = 0;

        let rendered = format_mean_obliquity_frame_round_trip_summary_for_report(&summary);
        assert!(rendered.contains("mean-obliquity frame round-trip unavailable"));
        assert!(rendered.contains("mean-obliquity frame round-trip summary has no samples"));
    }

    #[test]
    fn mean_obliquity_frame_round_trip_summary_validate_rejects_metric_drift() {
        let mut summary = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");
        summary.mean_longitude_delta_deg += 0.001;

        let error = summary
            .validate()
            .expect_err("metric drift should fail validation");
        assert!(error.contains("drifted from the canonical sample set"));
    }

    #[test]
    fn mean_obliquity_frame_round_trip_summary_validated_summary_line_matches_display() {
        let summary = mean_obliquity_frame_round_trip_summary()
            .expect("mean-obliquity frame round-trip summary should exist");

        assert_eq!(
            summary.validated_summary_line().unwrap(),
            summary.to_string()
        );
    }

    #[test]
    fn production_generation_boundary_summary_command_renders_the_overlay_summary() {
        let rendered = render_cli(&["production-generation-boundary-summary"])
            .expect("production generation boundary summary should render");

        assert!(rendered.contains("Production generation boundary overlay:"));
        assert!(rendered.contains("boundary overlay"));
        assert_eq!(
            rendered,
            production_generation_boundary_summary_for_report()
        );
    }

    #[test]
    fn production_generation_boundary_request_corpus_summary_command_renders_the_request_corpus_block(
    ) {
        let rendered = render_cli(&["production-generation-boundary-request-corpus-summary"])
            .expect("production generation boundary request corpus summary should render");

        assert!(rendered.contains("Production generation boundary request corpus:"));
        assert!(rendered.contains("request corpus"));
        assert_eq!(
            rendered,
            production_generation_boundary_request_corpus_summary_for_report()
        );
    }

    #[test]
    fn production_generation_source_window_summary_command_renders_the_source_windows_block() {
        let rendered = render_cli(&["production-generation-source-window-summary"])
            .expect("production generation source window summary should render");

        assert!(rendered.contains("Production generation source windows:"));
        assert!(rendered.contains("195 source-backed samples"));
        assert_eq!(
            rendered,
            production_generation_snapshot_window_summary_for_report()
        );
    }

    #[test]
    fn production_generation_summary_command_renders_the_overall_block() {
        let rendered = render_cli(&["production-generation-summary"])
            .expect("production generation summary should render");

        assert!(rendered.contains("Production generation coverage:"));
        assert!(rendered.contains("195 rows across 15 bodies and 18 epochs"));
        assert_eq!(
            rendered,
            production_generation_snapshot_summary_for_report()
        );
    }

    #[test]
    fn production_generation_boundary_source_summary_command_renders_the_source_block() {
        let rendered = render_cli(&["production-generation-boundary-source-summary"])
            .expect("production generation boundary source summary should render");

        assert!(rendered.contains("Production generation boundary overlay source:"));
        assert!(rendered.contains("boundary overlay source"));
        assert_eq!(
            rendered,
            production_generation_boundary_source_summary_for_report()
        );
    }

    #[test]
    fn production_generation_boundary_window_summary_command_renders_the_window_block() {
        let rendered = render_cli(&["production-generation-boundary-window-summary"])
            .expect("production generation boundary window summary should render");

        assert!(rendered.contains("Production generation boundary windows:"));
        assert!(rendered.contains("source-backed samples"));
        assert_eq!(
            rendered,
            production_generation_boundary_window_summary_for_report()
        );
    }

    #[test]
    fn production_generation_source_summary_command_renders_the_merged_source_block() {
        let rendered = render_cli(&["production-generation-source-summary"])
            .expect("production generation source summary should render");

        assert!(rendered.contains("Production generation source:"));
        assert!(rendered.contains("Reference snapshot source:"));
        assert!(rendered.contains("Production generation boundary overlay source:"));
        assert_eq!(rendered, production_generation_source_summary_for_report());
    }

    #[test]
    fn body_class_coverage_summary_commands_render_the_matching_blocks() {
        let production_generation =
            render_cli(&["production-generation-body-class-coverage-summary"])
                .expect("production generation body-class coverage summary should render");
        assert!(production_generation.contains("Production generation body-class coverage:"));
        assert_eq!(
            production_generation,
            production_generation_snapshot_body_class_coverage_summary_for_report()
        );

        let comparison = render_cli(&["comparison-snapshot-body-class-coverage-summary"])
            .expect("comparison snapshot body-class coverage summary should render");
        assert!(comparison.contains("Comparison snapshot body-class coverage:"));
        assert_eq!(
            comparison,
            comparison_snapshot_body_class_coverage_summary_for_report()
        );

        let reference = render_cli(&["reference-snapshot-body-class-coverage-summary"])
            .expect("reference snapshot body-class coverage summary should render");
        assert!(reference.contains("Reference snapshot body-class coverage:"));
        assert_eq!(
            reference,
            reference_snapshot_body_class_coverage_summary_for_report()
        );

        let independent_holdout_source_window =
            render_cli(&["independent-holdout-source-window-summary"])
                .expect("independent hold-out source window summary should render");
        assert!(independent_holdout_source_window.contains("Independent hold-out source windows:"));
        assert!(independent_holdout_source_window.contains("source-backed samples"));
        assert_eq!(
            independent_holdout_source_window,
            independent_holdout_snapshot_source_window_summary_for_report()
        );

        let independent_holdout = render_cli(&["independent-holdout-summary"])
            .expect("independent hold-out summary should render");
        assert!(independent_holdout.contains("JPL independent hold-out:"));
        assert!(independent_holdout.contains("transparency evidence only"));
        assert_eq!(
            independent_holdout,
            jpl_independent_holdout_summary_for_report()
        );

        let independent_holdout_source = render_cli(&["independent-holdout-source-summary"])
            .expect("independent hold-out source summary should render");
        assert!(independent_holdout_source.contains("Independent hold-out source:"));
        assert!(independent_holdout_source.contains("hold-out source"));
        assert_eq!(
            independent_holdout_source,
            independent_holdout_source_summary_for_report()
        );

        let holdout = render_cli(&["independent-holdout-body-class-coverage-summary"])
            .expect("independent hold-out body-class coverage summary should render");
        assert!(holdout.contains("Independent hold-out body-class coverage:"));
        assert_eq!(
            holdout,
            independent_holdout_snapshot_body_class_coverage_summary_for_report()
        );
    }

    #[test]
    fn comparison_snapshot_source_window_summary_command_renders_the_source_windows_block() {
        let rendered = render_cli(&["comparison-snapshot-source-window-summary"])
            .expect("comparison snapshot source window summary should render");

        assert!(rendered.contains("Comparison snapshot source windows:"));
        assert!(rendered.contains("112 source-backed samples"));
        assert_eq!(
            rendered,
            comparison_snapshot_source_window_summary_for_report()
        );
    }

    #[test]
    fn comparison_and_reference_snapshot_source_summary_commands_render_the_source_blocks() {
        let comparison = render_cli(&["comparison-snapshot-source-summary"])
            .expect("comparison snapshot source summary should render");
        assert!(comparison.contains("Comparison snapshot source:"));
        assert_eq!(comparison, comparison_snapshot_source_summary_for_report());

        let reference = render_cli(&["reference-snapshot-source-summary"])
            .expect("reference snapshot source summary should render");
        assert!(reference.contains("Reference snapshot source:"));
        assert_eq!(reference, reference_snapshot_source_summary_for_report());
    }

    #[test]
    fn comparison_and_reference_snapshot_summary_commands_render_the_overall_blocks() {
        let comparison = render_cli(&["comparison-snapshot-summary"])
            .expect("comparison snapshot summary should render");
        assert!(comparison.contains("Comparison snapshot summary"));
        assert!(comparison.contains("Comparison snapshot coverage:"));
        assert_eq!(
            comparison,
            format!(
                "Comparison snapshot summary\n{}\n",
                comparison_snapshot_summary_for_report()
            )
        );

        let reference = render_cli(&["reference-snapshot-summary"])
            .expect("reference snapshot summary should render");
        assert!(reference.contains("Reference snapshot summary"));
        assert!(reference.contains("Reference snapshot coverage:"));
        assert_eq!(
            reference,
            format!(
                "Reference snapshot summary\n{}\n",
                reference_snapshot_summary_for_report()
            )
        );
    }

    #[test]
    fn comparison_and_benchmark_corpus_summary_commands_render_the_corpus_blocks() {
        let comparison = render_cli(&["comparison-corpus-summary"])
            .expect("comparison corpus summary should render");
        assert!(comparison.contains("Comparison corpus summary"));
        assert!(comparison.contains("name: JPL Horizons release-grade comparison window"));
        assert!(comparison.contains("release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"));
        assert_eq!(comparison, render_comparison_corpus_summary_text());

        let guard = render_cli(&["comparison-corpus-release-guard-summary"])
            .expect("comparison corpus release guard summary should render");
        assert!(guard.contains("Comparison corpus release-grade guard summary"));
        assert!(guard.contains("Release-grade guard: Pluto excluded from tolerance evidence; 2451913.5 boundary day stays out of the audit slice"));
        assert_eq!(guard, render_comparison_corpus_release_guard_summary_text());

        let benchmark = render_cli(&["benchmark-corpus-summary"])
            .expect("benchmark corpus summary should render");
        assert!(benchmark.contains("Benchmark corpus summary"));
        assert!(benchmark.contains("name: Representative 1500-2500 window"));
        assert_eq!(benchmark, render_benchmark_corpus_summary_text());
    }

    #[test]
    fn manifest_summary_commands_render_the_matching_blocks() {
        let comparison = render_cli(&["comparison-snapshot-manifest-summary"])
            .expect("comparison snapshot manifest summary should render");
        assert!(comparison.contains("Comparison snapshot manifest:"));
        assert_eq!(
            comparison,
            comparison_snapshot_manifest_summary_for_report()
        );

        let reference = render_cli(&["reference-snapshot-manifest-summary"])
            .expect("reference snapshot manifest summary should render");
        assert!(reference.contains("Reference snapshot manifest:"));
        assert_eq!(reference, reference_snapshot_manifest_summary_for_report());
    }

    #[test]
    fn reference_snapshot_source_window_summary_command_renders_the_source_windows_block() {
        let rendered = render_cli(&["reference-snapshot-source-window-summary"])
            .expect("reference snapshot source window summary should render");

        assert!(rendered.contains("Reference snapshot source windows:"));
        assert!(rendered.contains("source-backed samples"));
        assert_eq!(
            rendered,
            reference_snapshot_source_window_summary_for_report()
        );
    }

    #[test]
    fn reference_snapshot_lunar_boundary_summary_command_renders_the_lunar_boundary_block() {
        let rendered = render_cli(&["reference-snapshot-lunar-boundary-summary"])
            .expect("reference snapshot lunar boundary summary should render");

        assert!(rendered.contains("Reference lunar boundary evidence:"));
        assert_eq!(
            rendered,
            reference_snapshot_lunar_boundary_summary_for_report()
        );
    }

    #[test]
    fn reference_snapshot_1749_major_body_boundary_summary_command_renders_the_1749_boundary_block()
    {
        let rendered = render_cli(&["reference-snapshot-1749-major-body-boundary-summary"])
            .expect("reference snapshot 1749 major-body boundary summary should render");

        assert!(rendered.contains("Reference 1749 major-body boundary evidence:"));
        assert!(rendered.contains("JD 2360233.5 (TDB)"));
        assert_eq!(
            rendered,
            reference_snapshot_1749_major_body_boundary_summary_for_report()
        );
        let alias = render_cli(&["1749-major-body-boundary-summary"])
            .expect("1749 major-body boundary alias should render");
        assert_eq!(alias, rendered);
    }

    #[test]
    fn reference_snapshot_early_major_body_boundary_summary_command_renders_the_early_boundary_block(
    ) {
        let rendered = render_cli(&["reference-snapshot-early-major-body-boundary-summary"])
            .expect("reference snapshot early major-body boundary summary should render");

        assert!(rendered.contains("Reference early major-body boundary evidence:"));
        assert_eq!(
            rendered,
            reference_snapshot_early_major_body_boundary_summary_for_report()
        );
        let alias = render_cli(&["early-major-body-boundary-summary"])
            .expect("early major-body boundary alias should render");
        assert_eq!(alias, rendered);

        let reference_snapshot_1800_major_body_boundary_summary =
            render_cli(&["reference-snapshot-1800-major-body-boundary-summary"])
                .expect("reference snapshot 1800 major-body boundary summary should render");

        assert!(reference_snapshot_1800_major_body_boundary_summary
            .contains("Reference 1800 major-body boundary evidence:"));
        assert_eq!(
            reference_snapshot_1800_major_body_boundary_summary,
            reference_snapshot_1800_major_body_boundary_summary_for_report()
        );
        let alias = render_cli(&["1800-major-body-boundary-summary"])
            .expect("1800 major-body boundary alias should render");
        assert_eq!(alias, reference_snapshot_1800_major_body_boundary_summary);
    }

    #[test]
    fn reference_snapshot_2500_major_body_boundary_summary_command_renders_the_terminal_boundary_block(
    ) {
        let rendered = render_cli(&["reference-snapshot-2500-major-body-boundary-summary"])
            .expect("reference snapshot 2500 major-body boundary summary should render");

        assert!(rendered.contains("Reference 2500 major-body boundary evidence:"));
        assert!(rendered.contains("JD 2500000.0 (TDB)"));
        assert_eq!(
            rendered,
            reference_snapshot_2500_major_body_boundary_summary_for_report()
        );
        let alias = render_cli(&["2500-major-body-boundary-summary"])
            .expect("2500 major-body boundary alias should render");
        assert_eq!(alias, rendered);
    }
}
