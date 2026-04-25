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
mod house_validation;

pub use artifact::{
    render_artifact_report, render_artifact_summary, ArtifactBodyInspection,
    ArtifactInspectionReport,
};
pub use house_validation::{
    house_validation_report, HouseValidationReport, HouseValidationSample, HouseValidationScenario,
};

use pleiades_ayanamsa::{
    baseline_ayanamsas, built_in_ayanamsas, metadata_coverage, release_ayanamsas, resolve_ayanamsa,
};
use pleiades_core::{
    current_api_stability_profile, current_compatibility_profile,
    current_release_profile_identifiers, default_chart_bodies, AccuracyClass, Apparentness,
    BackendCapabilities, BackendFamily, BackendMetadata, CelestialBody, CompositeBackend,
    CoordinateFrame, EclipticCoordinates, EphemerisBackend, EphemerisError, EphemerisErrorKind,
    EphemerisRequest, EphemerisResult, Instant, JulianDay, Longitude, TimeRange, TimeScale,
    ZodiacMode,
};
use pleiades_data::{packaged_artifact, PackagedDataBackend};
use pleiades_elp::{
    format_lunar_theory_capability_summary, lunar_reference_evidence,
    lunar_reference_evidence_summary, lunar_reference_evidence_summary_for_report,
    lunar_theory_capability_summary, lunar_theory_specification, lunar_theory_summary, ElpBackend,
};
use pleiades_houses::{
    baseline_house_systems, built_in_house_systems, release_house_systems, resolve_house_system,
};
use pleiades_jpl::{
    comparison_snapshot, interpolation_quality_samples, reference_asteroid_evidence,
    reference_asteroids, reference_snapshot_summary, JplSnapshotBackend,
};
use pleiades_vsop87::{
    body_source_profiles, frame_treatment_summary, source_audit_summary, source_audits,
    source_documentation_summary, source_specifications, Vsop87Backend,
};

const DEFAULT_BENCHMARK_ROUNDS: usize = 10_000;
const BANNER: &str = "pleiades-validate stage 4 tool";
const REGRESSION_LONGITUDE_THRESHOLD_DEG: f64 = 45.0;
const REGRESSION_LATITUDE_THRESHOLD_DEG: f64 = 1.0;
const REGRESSION_DISTANCE_THRESHOLD_AU: f64 = 0.25;

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

impl ValidationCorpus {
    /// Creates the default JPL snapshot corpus.
    pub fn jpl_snapshot() -> Self {
        let requests = comparison_snapshot()
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(entry.epoch.julian_day, TimeScale::Tt),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect();

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
#[derive(Clone, Debug, Default)]
pub struct ComparisonSummary {
    /// Number of samples compared.
    pub sample_count: usize,
    /// Maximum absolute longitude delta.
    pub max_longitude_delta_deg: f64,
    /// Mean absolute longitude delta.
    pub mean_longitude_delta_deg: f64,
    /// Maximum absolute latitude delta.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta.
    pub mean_latitude_delta_deg: f64,
    /// Maximum absolute distance delta.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta.
    pub mean_distance_delta_au: Option<f64>,
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
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute latitude delta.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta.
    pub mean_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Maximum absolute distance delta.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta.
    pub mean_distance_delta_au: Option<f64>,
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

/// Per-body comparison status against the expected tolerance table.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyToleranceSummary {
    /// Body queried for this tolerance summary.
    pub body: CelestialBody,
    /// Expected tolerance for this body.
    pub tolerance: ComparisonTolerance,
    /// Number of samples compared for this body.
    pub sample_count: usize,
    /// Whether all measured deltas are within the expected tolerance.
    pub within_tolerance: bool,
    /// Maximum absolute longitude delta measured for this body.
    pub max_longitude_delta_deg: f64,
    /// Maximum absolute latitude delta measured for this body.
    pub max_latitude_delta_deg: f64,
    /// Maximum absolute distance delta measured for this body.
    pub max_distance_delta_au: Option<f64>,
}

/// A comparison report generated by the validation tooling.
#[derive(Clone, Debug)]
pub struct ComparisonReport {
    /// Corpus name.
    pub corpus_name: String,
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
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: f64,
    sum_latitude_delta_deg: f64,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    sum_distance_delta_au: f64,
    distance_count: usize,
}

#[derive(Clone, Debug)]
struct BodyClassToleranceSummary {
    class: BodyClass,
    body_count: usize,
    sample_count: usize,
    within_tolerance_body_count: usize,
    outside_tolerance_body_count: usize,
    max_longitude_delta_body: Option<CelestialBody>,
    max_longitude_delta_deg: Option<f64>,
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: Option<f64>,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    outside_bodies: Vec<CelestialBody>,
}

impl BodyClassToleranceSummary {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  {}", self.class.label())?;
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
            writeln!(f, "    max longitude delta: {:.12}° ({})", value, body)?;
        }
        if let (Some(body), Some(value)) = (
            self.max_latitude_delta_body.as_ref(),
            self.max_latitude_delta_deg,
        ) {
            writeln!(f, "    max latitude delta: {:.12}° ({})", value, body)?;
        }
        if let (Some(body), Some(value)) = (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_au,
        ) {
            writeln!(f, "    max distance delta: {:.12} AU ({})", value, body)?;
        }

        Ok(())
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

    fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sample_count == 0 {
            return Ok(());
        }

        writeln!(f, "  {}", self.class.label())?;
        writeln!(f, "    samples: {}", self.sample_count)?;
        writeln!(
            f,
            "    max longitude delta: {:.12}°",
            self.max_longitude_delta_deg
        )?;
        writeln!(
            f,
            "    mean longitude delta: {:.12}°",
            self.mean_longitude_delta_deg()
        )?;
        writeln!(
            f,
            "    max latitude delta: {:.12}°",
            self.max_latitude_delta_deg
        )?;
        writeln!(
            f,
            "    mean latitude delta: {:.12}°",
            self.mean_latitude_delta_deg()
        )?;
        if let Some(value) = self.max_distance_delta_au {
            writeln!(f, "    max distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.mean_distance_delta_au() {
            writeln!(f, "    mean distance delta: {:.12} AU", value)?;
        }

        Ok(())
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
    /// Total elapsed time.
    pub elapsed: std::time::Duration,
}

impl BenchmarkReport {
    /// Returns the average number of nanoseconds per request.
    pub fn nanoseconds_per_request(&self) -> f64 {
        let total_requests = (self.rounds * self.sample_count) as f64;
        if total_requests == 0.0 {
            return 0.0;
        }

        self.elapsed.as_secs_f64() * 1_000_000_000.0 / total_requests
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
}

/// A generated release bundle containing the compatibility profile, release notes,
/// release checklist, backend matrix, API posture, API stability summary, validation report summary,
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
    /// Path to the generated validation report summary file.
    pub validation_report_summary_path: PathBuf,
    /// Path to the generated artifact summary file.
    pub artifact_summary_path: PathBuf,
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
    /// Number of bytes written for the validation report summary.
    pub validation_report_summary_bytes: usize,
    /// Number of bytes written for the artifact summary.
    pub artifact_summary_bytes: usize,
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
    /// Deterministic checksum for the validation report summary contents.
    pub validation_report_summary_checksum: u64,
    /// Deterministic checksum for the artifact summary contents.
    pub artifact_summary_checksum: u64,
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

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation report")?;
        writeln!(f)?;
        let release_profiles = current_release_profile_identifiers();
        writeln!(f, "Compatibility profile")?;
        writeln!(f, "  id: {}", release_profiles.compatibility_profile_id)?;
        writeln!(f, "{}", current_compatibility_profile())?;
        writeln!(f)?;
        writeln!(f, "API stability posture")?;
        writeln!(f, "  id: {}", release_profiles.api_stability_profile_id)?;
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
        writeln!(f)?;
        writeln!(f, "Benchmark corpus")?;
        write_corpus_summary(f, &self.benchmark_corpus)?;
        writeln!(f)?;
        writeln!(f, "Packaged-data benchmark corpus")?;
        write_corpus_summary(f, &self.packaged_benchmark_corpus)?;
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
        write_comparison_summary(f, &self.comparison.summary)?;
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
        write_tolerance_policy(f, &self.comparison.candidate_backend)?;
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
        writeln!(f, "Benchmark summaries")?;
        writeln!(f, "Reference benchmark")?;
        writeln!(f, "  corpus: {}", self.reference_benchmark.corpus_name)?;
        writeln!(
            f,
            "  ns/request: {}",
            format_ns(self.reference_benchmark.nanoseconds_per_request())
        )?;
        writeln!(f)?;
        writeln!(f, "Candidate benchmark")?;
        writeln!(f, "  corpus: {}", self.candidate_benchmark.corpus_name)?;
        writeln!(
            f,
            "  ns/request: {}",
            format_ns(self.candidate_benchmark.nanoseconds_per_request())
        )?;
        writeln!(f)?;
        writeln!(f, "Packaged-data benchmark")?;
        writeln!(f, "  corpus: {}", self.packaged_benchmark.corpus_name)?;
        writeln!(
            f,
            "  ns/request: {}",
            format_ns(self.packaged_benchmark.nanoseconds_per_request())
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
            "  validation report summary: {}",
            self.validation_report_summary_path.display()
        )?;
        writeln!(
            f,
            "  artifact summary: {}",
            self.artifact_summary_path.display()
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
            "  validation report summary bytes: {}",
            self.validation_report_summary_bytes
        )?;
        writeln!(
            f,
            "  artifact summary bytes: {}",
            self.artifact_summary_bytes
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
            "  validation report summary checksum: 0x{:016x}",
            self.validation_report_summary_checksum
        )?;
        writeln!(
            f,
            "  artifact summary checksum: 0x{:016x}",
            self.artifact_summary_checksum
        )?;
        writeln!(
            f,
            "  validation report checksum: 0x{:016x}",
            self.validation_report_checksum
        )?;
        writeln!(f, "  manifest checksum: 0x{:016x}", self.manifest_checksum)
    }
}

/// Builds the default validation corpus.
pub fn default_corpus() -> ValidationCorpus {
    ValidationCorpus::jpl_snapshot()
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
            Ok(current_compatibility_profile().to_string())
        }
        Some("benchmark") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_benchmark_report(rounds).map_err(render_error)
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
        Some("artifact-summary") | Some("artifact-posture-summary") => {
            ensure_no_extra_args(&args[1..], "artifact-summary")?;
            render_artifact_summary().map_err(render_artifact_error)
        }
        Some("workspace-audit") | Some("audit") => {
            ensure_no_extra_args(&args[1..], "workspace-audit")?;
            let report = workspace_audit_report().map_err(|error| error.to_string())?;
            if report.is_clean() {
                Ok(report.to_string())
            } else {
                Err(format!("workspace audit failed:\n{report}"))
            }
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
            distance_count += 1;
            summary.max_distance_delta_au = Some(
                summary
                    .max_distance_delta_au
                    .map_or(delta, |current| current.max(delta)),
            );
        }

        summary.sample_count += 1;
        summary.max_longitude_delta_deg = summary.max_longitude_delta_deg.max(longitude_delta_deg);
        summary.mean_longitude_delta_deg += longitude_delta_deg;
        summary.max_latitude_delta_deg = summary.max_latitude_delta_deg.max(latitude_delta_deg);
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
        summary.mean_latitude_delta_deg /= sample_count;
    }
    if distance_count > 0 {
        summary.mean_distance_delta_au = Some(distance_sum / distance_count as f64);
    }

    Ok(ComparisonReport {
        corpus_name: corpus.name.clone(),
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
    let start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(backend.position(request)?);
        }
    }

    Ok(BenchmarkReport {
        backend: backend.metadata(),
        corpus_name: corpus.name.clone(),
        apparentness: corpus.apparentness,
        rounds,
        sample_count: corpus.requests.len(),
        elapsed: start.elapsed(),
    })
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

/// Verifies that the release compatibility profile stays synchronized with the
/// canonical house-system and ayanamsa catalogs.
pub fn verify_compatibility_profile() -> Result<String, EphemerisError> {
    let profile = current_compatibility_profile();
    let release_profiles = current_release_profile_identifiers();

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

    let house_labels_checked = verify_house_system_aliases(profile.house_systems)?;
    let ayanamsa_labels_checked = verify_ayanamsa_aliases(profile.ayanamsas)?;
    let custom_definition_labels_checked =
        verify_custom_definition_labels(profile.custom_definition_labels)?;

    let mut text = String::new();
    text.push_str("Compatibility profile verification\n");
    text.push_str("Profile: ");
    text.push_str(profile.profile_id);
    text.push('\n');
    text.push_str("House systems verified: ");
    text.push_str(&profile.house_systems.len().to_string());
    text.push_str(" descriptors, ");
    text.push_str(&house_labels_checked.to_string());
    text.push_str(" labels\n");
    text.push_str("Ayanamsas verified: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" descriptors, ");
    text.push_str(&ayanamsa_labels_checked.to_string());
    text.push_str(" labels\n");
    text.push_str("Baseline/release slices: ");
    text.push_str(&profile.baseline_house_systems.len().to_string());
    text.push_str(" house baseline + ");
    text.push_str(&profile.release_house_systems.len().to_string());
    text.push_str(" house release, ");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" ayanamsa baseline + ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" ayanamsa release\n");
    text.push_str("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented\n");
    text.push_str("Custom-definition labels verified: ");
    text.push_str(&custom_definition_labels_checked.to_string());
    text.push_str(" labels, all remain custom-definition territory\n");
    text.push_str("Compatibility caveats documented: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    Ok(text)
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

fn verify_house_system_aliases(
    entries: &[pleiades_houses::HouseSystemDescriptor],
) -> Result<usize, EphemerisError> {
    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for entry in entries {
        ensure_profile_descriptor_metadata("house-system", entry.canonical_name, entry.notes)?;

        labels_checked += 1;
        ensure_unique_profile_label("house-system", entry.canonical_name, &mut seen_labels)?;
        if resolve_house_system(entry.canonical_name) != Some(entry.system.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile house-system alias mismatch: canonical label '{}' should resolve to {:?}",
                    entry.canonical_name, entry.system
                ),
            ));
        }

        for alias in entry.aliases {
            labels_checked += 1;
            ensure_unique_profile_label("house-system", alias, &mut seen_labels)?;
            if resolve_house_system(alias) != Some(entry.system.clone()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile house-system alias mismatch: alias '{}' should resolve to {:?}",
                        alias, entry.system
                    ),
                ));
            }
        }
    }

    Ok(labels_checked)
}

fn verify_ayanamsa_aliases(
    entries: &[pleiades_ayanamsa::AyanamsaDescriptor],
) -> Result<usize, EphemerisError> {
    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for entry in entries {
        ensure_profile_descriptor_metadata("ayanamsa", entry.canonical_name, entry.notes)?;

        labels_checked += 1;
        ensure_unique_profile_label("ayanamsa", entry.canonical_name, &mut seen_labels)?;
        if resolve_ayanamsa(entry.canonical_name) != Some(entry.ayanamsa.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile ayanamsa alias mismatch: canonical label '{}' should resolve to {:?}",
                    entry.canonical_name, entry.ayanamsa
                ),
            ));
        }

        for alias in entry.aliases {
            labels_checked += 1;
            ensure_unique_profile_label("ayanamsa", alias, &mut seen_labels)?;
            if resolve_ayanamsa(alias) != Some(entry.ayanamsa.clone()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile ayanamsa alias mismatch: alias '{}' should resolve to {:?}",
                        alias, entry.ayanamsa
                    ),
                ));
            }
        }
    }

    Ok(labels_checked)
}

fn verify_custom_definition_labels(labels: &[&'static str]) -> Result<usize, EphemerisError> {
    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for label in labels {
        labels_checked += 1;
        ensure_unique_profile_label("custom-definition", label, &mut seen_labels)?;
        if resolve_house_system(label).is_some() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile custom-definition label '{}' should remain unresolved as a built-in house system",
                    label
                ),
            ));
        }
    }

    Ok(labels_checked)
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

    if notes.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{}' is missing notes metadata",
                canonical_name
            ),
        ));
    }

    Ok(())
}

fn ensure_unique_profile_label(
    catalog_label: &str,
    label: &str,
    seen_labels: &mut BTreeSet<String>,
) -> Result<(), EphemerisError> {
    if label.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("compatibility profile {catalog_label} descriptor contains a blank label"),
        ));
    }

    let normalized = label.trim().to_string();
    if !seen_labels.insert(normalized) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} labels are not unique: duplicate label '{}'",
                label
            ),
        ));
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

fn render_compatibility_profile_summary_text() -> String {
    let profile = current_compatibility_profile();
    let release_profiles = current_release_profile_identifiers();
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
    text.push_str("Ayanamsas: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str("Ayanamsa sidereal metadata: ");
    text.push_str(&coverage.with_sidereal_metadata.to_string());
    text.push('/');
    text.push_str(&coverage.total.to_string());
    text.push_str(" entries with both a reference epoch and offset\n");
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
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
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

fn render_release_notes_text() -> String {
    let profile = current_compatibility_profile();
    let release_profiles = current_release_profile_identifiers();
    let api_stability = current_api_stability_profile();
    let mut text = String::new();

    text.push_str("Release notes\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Summary:\n");
    text.push_str(profile.summary);
    text.push('\n');
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push('\n');

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

    if !profile.release_notes.is_empty() {
        text.push_str("Release-specific coverage:\n");
        for note in profile.release_notes {
            text.push_str("- ");
            text.push_str(note);
            text.push('\n');
        }
        text.push('\n');
    }
    text.push_str(&format_reference_asteroid_evidence_summary());
    text.push('\n');
    text.push_str(&format_reference_snapshot_summary());
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
    let profile = current_compatibility_profile();
    let release_profiles = current_release_profile_identifiers();
    let api_stability = current_api_stability_profile();
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
    text.push_str(&format_reference_asteroid_evidence_summary());
    text.push('\n');
    text.push_str(&format_reference_snapshot_summary());
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    text.push_str("API stability summary line: ");
    text.push_str(api_stability.summary);
    text.push('\n');
    text.push_str("Release notes: release-notes\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release summary: release-summary\n");
    text.push('\n');
    text.push_str("See release-notes for the full maintainer-facing artifact.\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

fn render_release_checklist_text() -> String {
    let release_profiles = current_release_profile_identifiers();
    let mut text = String::new();

    text.push_str("Release checklist\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(release_profiles.api_stability_profile_id);
    text.push('\n');
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary\n");
    text.push('\n');
    text.push_str("Repository-managed release gates:\n");
    for item in [
        "[x] mise run fmt",
        "[x] mise run lint",
        "[x] mise run test",
        "[x] mise run audit",
        "[x] mise run release-smoke",
        "[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile",
        "[x] cargo run -q -p pleiades-validate -- validate-artifact",
    ] {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str("Manual bundle workflow:\n");
    for item in [
        "[x] cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release",
        "[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release",
        "[x] docs/release-reproducibility.md",
    ] {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str("Bundle contents:\n");
    for item in [
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
        "[x] validation-report.txt",
        "[x] bundle-manifest.txt",
        "[x] bundle-manifest.checksum.txt",
        "[x] verify-release-bundle",
    ] {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }
    text.push('\n');
    text.push_str("External publishing reminders:\n");
    for item in [
        "[ ] tag and archive the release commit",
        "[ ] publish or attach the release bundle outside the workspace",
        "[ ] review any documented compatibility gaps before announcing the release",
    ] {
        text.push_str("- ");
        text.push_str(item);
        text.push('\n');
    }

    text
}

fn render_release_summary_text() -> String {
    let profile = current_compatibility_profile();
    let release_profiles = current_release_profile_identifiers();
    let api_stability = current_api_stability_profile();
    let mut text = String::new();

    text.push_str("Release summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(release_profiles.api_stability_profile_id);
    text.push('\n');
    text.push_str("Release summary line: ");
    text.push_str(profile.summary);
    text.push('\n');
    text.push_str("House systems: ");
    text.push_str(&profile.house_systems.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_house_systems.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_house_systems.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str("Ayanamsas: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
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
    if let Ok(report) = build_validation_report(0) {
        let tolerance_outside_bodies: usize = report
            .comparison
            .body_class_tolerance_summaries()
            .iter()
            .map(|summary| summary.outside_tolerance_body_count)
            .sum();
        text.push_str("Validation evidence: ");
        text.push_str(&report.comparison.summary.sample_count.to_string());
        text.push_str(" comparison samples, ");
        text.push_str(&report.comparison.notable_regressions().len().to_string());
        text.push_str(" notable regressions, ");
        text.push_str(&tolerance_outside_bodies.to_string());
        text.push_str(" outside-tolerance bodies\n");
    }
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Workspace audit: workspace-audit / audit\n");
    text.push_str("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Packaged-artifact profile: ");
    text.push_str(&format_packaged_artifact_profile_summary());
    text.push('\n');
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push('\n');
    text.push_str("API stability summary line: ");
    text.push_str(api_stability.summary);
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
    let release_profiles = current_release_profile_identifiers();
    let mut text = String::new();

    text.push_str("Release checklist summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("API stability posture: ");
    text.push_str(release_profiles.api_stability_profile_id);
    text.push('\n');
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Backend matrix summary: backend-matrix-summary\n");
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Workspace audit: workspace-audit / audit\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Artifact validation: validate-artifact\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary\n");
    text.push_str("Repository-managed release gates: 7 items\n");
    text.push_str("Manual bundle workflow: 3 items\n");
    text.push_str("Bundle contents: 16 items\n");
    text.push_str("External publishing reminders: 3 items\n");
    text.push('\n');
    text.push_str("See release-checklist for the full maintainer-facing artifact.\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

#[derive(Clone, Debug)]
struct WorkspaceProvenance {
    source_revision: String,
    workspace_status: String,
    rustc_version: String,
}

fn workspace_provenance() -> WorkspaceProvenance {
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

/// Writes a release bundle containing the compatibility profile, release notes,
/// release notes summary, release summary, release checklist, release checklist summary,
/// backend matrix, API posture, API stability summary, validation report summary, artifact summary,
/// validation report, and a manifest.
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
    let release_checklist_text = render_release_checklist_text();
    let release_checklist_summary_text = render_release_checklist_summary_text();
    let backend_matrix_text = render_backend_matrix_report()?;
    let backend_matrix_summary_text = render_backend_matrix_summary();
    let api_stability_text = current_api_stability_profile().to_string();
    let api_stability_summary_text = render_api_stability_summary();
    let validation_report = build_validation_report(rounds)?;
    let validation_report_text = validation_report.to_string();
    let validation_report_summary_text = render_validation_report_summary_text(&validation_report);
    let artifact_summary_text = render_artifact_summary()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let provenance = workspace_provenance();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let profile_summary_path = output_dir.join("compatibility-profile-summary.txt");
    let release_notes_path = output_dir.join("release-notes.txt");
    let release_notes_summary_path = output_dir.join("release-notes-summary.txt");
    let release_summary_path = output_dir.join("release-summary.txt");
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let release_checklist_summary_path = output_dir.join("release-checklist-summary.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let validation_report_summary_path = output_dir.join("validation-report-summary.txt");
    let artifact_summary_path = output_dir.join("artifact-summary.txt");
    let report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let manifest_checksum_path = output_dir.join("bundle-manifest.checksum.txt");
    let compatibility_profile_checksum = checksum64(&profile_text);
    let compatibility_profile_summary_checksum = checksum64(&profile_summary_text);
    let release_notes_checksum = checksum64(&release_notes_text);
    let release_notes_summary_text = render_release_notes_summary_text();
    let release_notes_summary_checksum = checksum64(&release_notes_summary_text);
    let release_summary_checksum = checksum64(&release_summary_text);
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let release_checklist_summary_checksum = checksum64(&release_checklist_summary_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let validation_report_summary_checksum = checksum64(&validation_report_summary_text);
    let artifact_summary_checksum = checksum64(&artifact_summary_text);
    let validation_report_checksum = checksum64(&validation_report_text);
    let manifest_text = format!(
        "Release bundle manifest\nprofile: compatibility-profile.txt\nprofile checksum (fnv1a-64): 0x{compatibility_profile_checksum:016x}\nprofile summary: compatibility-profile-summary.txt\nprofile summary checksum (fnv1a-64): 0x{compatibility_profile_summary_checksum:016x}\nrelease notes: release-notes.txt\nrelease notes checksum (fnv1a-64): 0x{release_notes_checksum:016x}\nrelease notes summary: release-notes-summary.txt\nrelease notes summary checksum (fnv1a-64): 0x{release_notes_summary_checksum:016x}\nrelease summary: release-summary.txt\nrelease summary checksum (fnv1a-64): 0x{release_summary_checksum:016x}\nrelease checklist: release-checklist.txt\nrelease checklist checksum (fnv1a-64): 0x{release_checklist_checksum:016x}\nrelease checklist summary: release-checklist-summary.txt\nrelease checklist summary checksum (fnv1a-64): 0x{release_checklist_summary_checksum:016x}\nbackend matrix: backend-matrix.txt\nbackend matrix checksum (fnv1a-64): 0x{backend_matrix_checksum:016x}\nbackend matrix summary: backend-matrix-summary.txt\nbackend matrix summary checksum (fnv1a-64): 0x{backend_matrix_summary_checksum:016x}\napi stability posture: api-stability.txt\napi stability checksum (fnv1a-64): 0x{api_stability_checksum:016x}\napi stability summary: api-stability-summary.txt\napi stability summary checksum (fnv1a-64): 0x{api_stability_summary_checksum:016x}\nvalidation report summary: validation-report-summary.txt\nvalidation report summary checksum (fnv1a-64): 0x{validation_report_summary_checksum:016x}\nartifact summary: artifact-summary.txt\nartifact summary checksum (fnv1a-64): 0x{artifact_summary_checksum:016x}\nvalidation report: validation-report.txt\nvalidation report checksum (fnv1a-64): 0x{validation_report_checksum:016x}\nsource revision: {}\nworkspace status: {}\nrustc version: {}\nprofile id: {}\napi stability posture id: {}\nvalidation rounds: {}\n",
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
        &validation_report_summary_path,
        validation_report_summary_text.as_bytes(),
    )?;
    let manifest_checksum = checksum64(&manifest_text);
    let manifest_checksum_text = format!("0x{manifest_checksum:016x}\n");
    fs::write(&artifact_summary_path, artifact_summary_text.as_bytes())?;
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
    validation_report_summary_path: String,
    validation_report_summary_checksum: u64,
    artifact_summary_path: String,
    artifact_summary_checksum: u64,
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
            validation_report_summary_path: parse_manifest_string(
                text,
                "validation report summary:",
            )?,
            validation_report_summary_checksum: parse_manifest_checksum(
                text,
                "validation report summary checksum (fnv1a-64):",
            )?,
            artifact_summary_path: parse_manifest_string(text, "artifact summary:")?,
            artifact_summary_checksum: parse_manifest_checksum(
                text,
                "artifact summary checksum (fnv1a-64):",
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
        "release-checklist.txt",
        "release-checklist-summary.txt",
        "backend-matrix.txt",
        "backend-matrix-summary.txt",
        "api-stability.txt",
        "api-stability-summary.txt",
        "validation-report-summary.txt",
        "artifact-summary.txt",
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
    const EXPECTED_MANIFEST_LINES: [&str; 35] = [
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
        "validation report summary:",
        "validation report summary checksum (fnv1a-64):",
        "artifact summary:",
        "artifact summary checksum (fnv1a-64):",
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
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let release_checklist_summary_path = output_dir.join("release-checklist-summary.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let validation_report_summary_path = output_dir.join("validation-report-summary.txt");
    let artifact_summary_path = output_dir.join("artifact-summary.txt");
    let validation_report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let manifest_checksum_path = output_dir.join("bundle-manifest.checksum.txt");

    for (path, label) in [
        (&profile_path, "compatibility profile"),
        (&profile_summary_path, "compatibility profile summary"),
        (&release_notes_path, "release notes"),
        (&release_notes_summary_path, "release notes summary"),
        (&release_summary_path, "release summary"),
        (&release_checklist_path, "release checklist"),
        (&release_checklist_summary_path, "release checklist summary"),
        (&backend_matrix_path, "backend matrix"),
        (&backend_matrix_summary_path, "backend matrix summary"),
        (&api_stability_path, "API stability"),
        (&api_stability_summary_path, "API stability summary"),
        (&validation_report_summary_path, "validation report summary"),
        (&artifact_summary_path, "artifact summary"),
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
    let validation_report_summary_text =
        read_required_bundle_text(&validation_report_summary_path, "validation report summary")?;
    let artifact_summary_text =
        read_required_bundle_text(&artifact_summary_path, "artifact summary")?;
    let validation_report_text =
        read_required_bundle_text(&validation_report_path, "validation report")?;
    let manifest_text = read_required_bundle_text(&manifest_path, "bundle manifest")?;
    let manifest_checksum_text =
        read_required_bundle_text(&manifest_checksum_path, "bundle manifest checksum sidecar")?;

    let manifest = ParsedReleaseBundleManifest::parse(&manifest_text)?;
    ensure_release_bundle_manifest_is_canonical(&manifest_text)?;
    ensure_release_bundle_directory_contents(output_dir)?;
    ensure_non_empty_manifest_value(&manifest.source_revision, "source revision")?;
    ensure_non_empty_manifest_value(&manifest.workspace_status, "workspace status")?;
    ensure_non_empty_manifest_value(&manifest.rustc_version, "rustc version")?;
    ensure_non_empty_manifest_value(&manifest.profile_id, "profile id")?;
    ensure_non_empty_manifest_value(
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
    if manifest.validation_report_summary_path != "validation-report-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected validation report summary file entry: {}",
            manifest.validation_report_summary_path
        )));
    }
    if manifest.artifact_summary_path != "artifact-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected artifact summary file entry: {}",
            manifest.artifact_summary_path
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
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let release_checklist_summary_checksum = checksum64(&release_checklist_summary_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let validation_report_summary_checksum = checksum64(&validation_report_summary_text);
    let artifact_summary_checksum = checksum64(&artifact_summary_text);
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
    if manifest.validation_report_summary_checksum != validation_report_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "validation report summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.validation_report_summary_checksum, validation_report_summary_checksum
        )));
    }
    if manifest.artifact_summary_checksum != artifact_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "artifact summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.artifact_summary_checksum, artifact_summary_checksum
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

    Ok(ReleaseBundle {
        source_revision: manifest.source_revision,
        workspace_status: manifest.workspace_status,
        rustc_version: manifest.rustc_version,
        output_dir: output_dir.to_path_buf(),
        compatibility_profile_path: profile_path,
        compatibility_profile_summary_path: profile_summary_path,
        release_notes_path,
        release_notes_summary_path,
        release_summary_path,
        release_checklist_path,
        release_checklist_summary_path,
        backend_matrix_path,
        backend_matrix_summary_path,
        api_stability_path,
        api_stability_summary_path,
        validation_report_summary_path,
        artifact_summary_path,
        validation_report_path,
        manifest_path,
        manifest_checksum_path,
        compatibility_profile_bytes: profile_text.len(),
        compatibility_profile_summary_bytes: profile_summary_text.len(),
        release_notes_bytes: release_notes_text.len(),
        release_notes_summary_bytes: release_notes_summary_text.len(),
        release_summary_bytes: release_summary_text.len(),
        release_checklist_bytes: release_checklist_text.len(),
        release_checklist_summary_bytes: release_checklist_summary_text.len(),
        backend_matrix_bytes: backend_matrix_text.len(),
        backend_matrix_summary_bytes: backend_matrix_summary_text.len(),
        api_stability_bytes: api_stability_text.len(),
        api_stability_summary_bytes: api_stability_summary_text.len(),
        validation_report_summary_bytes: validation_report_summary_text.len(),
        artifact_summary_bytes: artifact_summary_text.len(),
        validation_report_bytes: validation_report_text.len(),
        manifest_checksum_bytes: manifest_checksum_text.len(),
        compatibility_profile_checksum,
        compatibility_profile_summary_checksum,
        release_notes_checksum,
        release_notes_summary_checksum,
        release_summary_checksum,
        release_checklist_checksum,
        release_checklist_summary_checksum,
        backend_matrix_checksum,
        backend_matrix_summary_checksum,
        api_stability_checksum,
        api_stability_summary_checksum,
        validation_report_summary_checksum,
        artifact_summary_checksum,
        validation_report_checksum,
        manifest_checksum: manifest_checksum_value,
        validation_rounds: manifest.validation_rounds,
    })
}

fn parse_manifest_string(text: &str, prefix: &str) -> Result<String, ReleaseBundleError> {
    extract_prefixed_value(text, prefix).map(|value| value.to_string())
}

fn ensure_non_empty_manifest_value(
    value: &str,
    field_name: &str,
) -> Result<(), ReleaseBundleError> {
    if value.is_empty() {
        Err(ReleaseBundleError::Verification(format!(
            "missing {field_name} entry"
        )))
    } else {
        Ok(())
    }
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
    let comparison_corpus = default_corpus();
    let benchmark_corpus = benchmark_corpus();
    let packaged_benchmark_corpus = artifact::packaged_artifact_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let packaged = PackagedDataBackend::new();
    let comparison = compare_backends(&reference, &candidate, &comparison_corpus)?;
    let reference_benchmark = benchmark_backend(&reference, &comparison_corpus, rounds)?;
    let candidate_benchmark = benchmark_backend(&candidate, &benchmark_corpus, rounds)?;
    let packaged_benchmark = benchmark_backend(&packaged, &packaged_benchmark_corpus, rounds)?;
    let archived_regressions = comparison.regression_archive();

    Ok(ValidationReport {
        comparison_corpus: comparison_corpus.summary(),
        benchmark_corpus: benchmark_corpus.summary(),
        packaged_benchmark_corpus: packaged_benchmark_corpus.summary(),
        house_validation: house_validation_report(),
        comparison,
        archived_regressions,
        reference_benchmark,
        candidate_benchmark,
        packaged_benchmark,
    })
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

fn comparison_audit_totals(report: &ComparisonReport) -> (usize, usize, usize, usize) {
    let tolerance_summaries = report.tolerance_summaries();
    let body_count = tolerance_summaries.len();
    let within_tolerance_body_count = tolerance_summaries
        .iter()
        .filter(|summary| summary.within_tolerance)
        .count();
    let outside_tolerance_body_count = body_count.saturating_sub(within_tolerance_body_count);
    let regression_count = report.notable_regressions().len();

    (
        body_count,
        within_tolerance_body_count,
        outside_tolerance_body_count,
        regression_count,
    )
}

fn format_regression_bodies(regressions: &[RegressionFinding]) -> String {
    let bodies = regressions
        .iter()
        .map(|finding| finding.body.to_string())
        .collect::<BTreeSet<_>>();

    if bodies.is_empty() {
        "none".to_string()
    } else {
        bodies.into_iter().collect::<Vec<_>>().join(", ")
    }
}

/// Renders a release-grade comparison tolerance audit used by the CLI.
pub fn render_comparison_audit_report() -> Result<String, String> {
    let corpus = default_corpus();
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
        report.reference_backend.id, report.reference_backend.provenance.summary
    );
    let _ = writeln!(
        text,
        "  candidate backend: {} ({})",
        report.candidate_backend.id, report.candidate_backend.provenance.summary
    );
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
        if regression_count == 0 {
            "clean"
        } else {
            "regressions found"
        }
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(text, "  samples: {}", report.summary.sample_count);
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°",
        report.summary.max_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°",
        report.summary.max_latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}",
        report
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string())
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
        if let Some(value) = summary.max_distance_delta_au {
            let _ = writeln!(text, "    max distance delta: {:.12} AU", value);
        }
        if summary.distance_count > 0 {
            let _ = writeln!(
                text,
                "    mean distance delta: {:.12} AU",
                summary.sum_distance_delta_au / summary.distance_count as f64
            );
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
    write_tolerance_policy_text(&mut text, &report.candidate_backend);
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
    Ok(benchmark_backend(&candidate, &corpus, rounds)?.to_string())
}

fn vsop87_canonical_body_evidence() -> Option<Vec<pleiades_vsop87::Vsop87CanonicalBodyEvidence>> {
    pleiades_vsop87::canonical_epoch_body_evidence()
}

fn vsop87_canonical_evidence_summary() -> Option<pleiades_vsop87::Vsop87CanonicalEvidenceSummary> {
    pleiades_vsop87::canonical_epoch_evidence_summary()
}

fn format_vsop87_canonical_evidence_summary() -> String {
    match vsop87_canonical_evidence_summary() {
        Some(summary) => format!(
            "VSOP87 canonical J2000 source-backed evidence: {} samples, status {}, max Δlon={:.12}° ({}; {}; {}), max Δlat={:.12}° ({}; {}; {}), max Δdist={:.12} AU ({}; {}; {})",
            summary.sample_count,
            if summary.within_interim_limits {
                "within interim limits"
            } else {
                "outside interim limits"
            },
            summary.max_longitude_delta_deg,
            summary.max_longitude_delta_body,
            summary.max_longitude_delta_source_kind.label(),
            summary.max_longitude_delta_source_file,
            summary.max_latitude_delta_deg,
            summary.max_latitude_delta_body,
            summary.max_latitude_delta_source_kind.label(),
            summary.max_latitude_delta_source_file,
            summary.max_distance_delta_au,
            summary.max_distance_delta_body,
            summary.max_distance_delta_source_kind.label(),
            summary.max_distance_delta_source_file,
        ),
        None => "VSOP87 canonical J2000 source-backed evidence: unavailable".to_string(),
    }
}

fn format_vsop87_body_evidence_summary() -> String {
    match vsop87_canonical_body_evidence() {
        Some(evidence) => {
            let within_interim_limits = evidence
                .iter()
                .filter(|row| row.within_interim_limits)
                .count();
            let vendored_count = evidence
                .iter()
                .filter(|row| {
                    row.source_kind == pleiades_vsop87::Vsop87BodySourceKind::VendoredVsop87b
                })
                .count();
            let generated_count = evidence
                .iter()
                .filter(|row| {
                    row.source_kind == pleiades_vsop87::Vsop87BodySourceKind::GeneratedBinaryVsop87b
                })
                .count();
            let truncated_count = evidence
                .iter()
                .filter(|row| {
                    row.source_kind == pleiades_vsop87::Vsop87BodySourceKind::TruncatedVsop87b
                })
                .count();
            if generated_count == 0 && truncated_count == 0 {
                format!(
                    "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file), {} within interim limits",
                    evidence.len(),
                    vendored_count,
                    within_interim_limits,
                )
            } else if generated_count > 0 && truncated_count == 0 {
                format!(
                    "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary), {} within interim limits",
                    evidence.len(),
                    vendored_count,
                    generated_count,
                    within_interim_limits,
                )
            } else if generated_count == 0 {
                format!(
                    "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} truncated slice), {} within interim limits",
                    evidence.len(),
                    vendored_count,
                    truncated_count,
                    within_interim_limits,
                )
            } else {
                format!(
                    "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary, {} truncated slice), {} within interim limits",
                    evidence.len(),
                    vendored_count,
                    generated_count,
                    truncated_count,
                    within_interim_limits,
                )
            }
        }
        None => "VSOP87 source-backed body evidence: unavailable".to_string(),
    }
}

fn format_vsop87_source_documentation_summary() -> String {
    let summary = source_documentation_summary();
    format!(
        "VSOP87 source documentation: {} source specs, {} source-backed body profiles, {} fallback mean-element body profile{}; source-backed breakdown: {} generated binary, {} vendored full-file, {} truncated slice",
        summary.source_specification_count,
        summary.source_backed_profile_count,
        summary.fallback_profile_count,
        if summary.fallback_profile_count == 1 {
            ""
        } else {
            "s"
        },
        summary.generated_binary_profile_count,
        summary.vendored_full_file_profile_count,
        summary.truncated_profile_count,
    )
}

fn format_vsop87_frame_treatment_summary() -> String {
    frame_treatment_summary().to_string()
}

fn format_vsop87_source_audit_summary() -> String {
    let audit = source_audit_summary();
    format!(
        "VSOP87 source audit: {} source-backed bodies, {} vendored full-file inputs, {} total terms, max source size {} bytes / {} lines, {} deterministic fingerprints",
        audit.source_count,
        audit.vendored_full_file_count,
        audit.total_term_count,
        audit.max_byte_length,
        audit.max_line_count,
        audit.fingerprint_count
    )
}

fn format_reference_asteroid_evidence_summary() -> String {
    let evidence = reference_asteroid_evidence();
    if evidence.is_empty() {
        "Selected asteroid evidence: unavailable".to_string()
    } else {
        format!(
            "Selected asteroid evidence: {} exact J2000 samples at JD {:.1} ({})",
            evidence.len(),
            evidence[0].epoch.julian_day.days(),
            format_bodies(reference_asteroids())
        )
    }
}

fn format_reference_snapshot_summary() -> String {
    match reference_snapshot_summary() {
        Some(summary) => format!(
            "Reference snapshot coverage: {} rows across {} bodies and {} epochs ({} asteroid rows; JD {:.1}..{:.1})",
            summary.row_count,
            summary.body_count,
            summary.epoch_count,
            summary.asteroid_row_count,
            summary.earliest_epoch.julian_day.days(),
            summary.latest_epoch.julian_day.days(),
        ),
        None => "Reference snapshot coverage: unavailable".to_string(),
    }
}

fn format_packaged_artifact_profile_summary() -> String {
    let artifact = packaged_artifact();
    artifact
        .header
        .profile
        .summary_for_body_count(artifact.bodies.len())
}

fn render_validation_report_summary_text(report: &ValidationReport) -> String {
    use std::fmt::Write as _;

    let release_profiles = current_release_profile_identifiers();
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
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(
        text,
        "  samples: {}",
        report.comparison.summary.sample_count
    );
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°",
        report.comparison.summary.max_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°",
        report.comparison.summary.max_latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}",
        report
            .comparison
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string())
    );
    let _ = writeln!(text, "  notable regressions: {}", comparison_regressions);
    let _ = writeln!(
        text,
        "  regression bodies: {}",
        format_regression_bodies(&report.comparison.notable_regressions())
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "JPL interpolation quality");
    let _ = writeln!(
        text,
        "  {}",
        format_jpl_interpolation_quality_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar reference evidence");
    let _ = writeln!(text, "  {}", lunar_reference_evidence_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Body comparison summaries");
    for summary in report.comparison.body_summaries() {
        let _ = writeln!(
            text,
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, max Δdist={}{}, mean Δdist={}",
            summary.body,
            summary.sample_count,
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_latitude_delta_deg,
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
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, max Δdist={}{}, mean Δdist={}",
            summary.class.label(),
            summary.sample_count,
            summary.max_longitude_delta_deg,
            max_longitude_body,
            summary.mean_longitude_delta_deg(),
            summary.max_latitude_delta_deg,
            max_latitude_body,
            summary.mean_latitude_delta_deg(),
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_distance_body,
            summary
                .mean_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class tolerance posture");
    for summary in report.comparison.body_class_tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}: bodies={}, samples={}, within tolerance bodies={}, outside tolerance bodies={}, max Δlon={}{}, max Δlat={}{}, max Δdist={}{}",
            summary.class.label(),
            summary.body_count,
            summary.sample_count,
            summary.within_tolerance_body_count,
            summary.outside_tolerance_body_count,
            summary
                .max_longitude_delta_deg
                .map(|value| format!("{value:.12}°"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary
                .max_latitude_delta_deg
                .map(|value| format!("{value:.12}°"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default()
        );
        if !summary.outside_bodies.is_empty() {
            let _ = writeln!(
                text,
                "    outside bodies: {}",
                format_bodies(&summary.outside_bodies)
            );
        }
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Tolerance policy");
    write_tolerance_policy_text(&mut text, &report.comparison.candidate_backend);
    let _ = writeln!(text);
    let _ = writeln!(text, "Expected tolerance status");
    for summary in report.comparison.tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}: profile={}, status={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}, measured max Δlon={:.12}°, max Δlat={:.12}°, max Δdist={}",
            summary.body,
            summary.tolerance.profile,
            if summary.within_tolerance { "within" } else { "exceeded" },
            summary.tolerance.max_longitude_delta_deg,
            summary.tolerance.max_latitude_delta_deg,
            summary
                .tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
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
    let _ = writeln!(text, "House validation corpus");
    let _ = writeln!(
        text,
        "  scenarios: {}",
        report.house_validation.scenarios.len()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "VSOP87 source-backed evidence");
    let _ = writeln!(text, "  {}", format_vsop87_source_documentation_summary());
    let _ = writeln!(text, "  {}", format_vsop87_frame_treatment_summary());
    let _ = writeln!(text, "  {}", format_vsop87_source_audit_summary());
    let _ = writeln!(text, "  {}", format_vsop87_canonical_evidence_summary());
    let _ = writeln!(text, "  {}", format_vsop87_body_evidence_summary());
    let _ = writeln!(text);
    let _ = writeln!(text, "ELP lunar theory specification");
    let _ = writeln!(text, "  {}", lunar_theory_summary());
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-artifact profile");
    let _ = writeln!(text, "  {}", format_packaged_artifact_profile_summary());
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
        "  ns/request: {}",
        format_ns(report.reference_benchmark.nanoseconds_per_request())
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
        "  ns/request: {}",
        format_ns(report.candidate_benchmark.nanoseconds_per_request())
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
        "  ns/request: {}",
        format_ns(report.packaged_benchmark.nanoseconds_per_request())
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
    let release_profiles = current_release_profile_identifiers();
    let catalog = implemented_backend_catalog();
    let mut family_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut bodies: Vec<String> = Vec::new();
    let mut frames: Vec<String> = Vec::new();
    let mut time_scales: Vec<String> = Vec::new();
    let mut deterministic_count = 0usize;
    let mut offline_count = 0usize;
    let mut batch_count = 0usize;
    let mut native_sidereal_count = 0usize;
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
            push_unique(&mut frames, format!("{:?}", frame));
        }
        for scale in &entry.metadata.supported_time_scales {
            push_unique(&mut time_scales, format!("{:?}", scale));
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
    text.push_str(&format_reference_asteroid_evidence_summary());
    text.push('\n');
    text.push_str(&format_reference_snapshot_summary());
    text.push('\n');
    text.push_str("Backends with external data sources: ");
    text.push_str(&data_source_count.to_string());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_summary());
    text.push('\n');
    text.push_str(&format_vsop87_frame_treatment_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_audit_summary());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_body_evidence_summary());
    text.push('\n');
    text.push_str(&lunar_theory_summary());
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
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
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
    let profile = current_api_stability_profile();
    let release_profiles = current_release_profile_identifiers();
    let mut text = String::new();

    text.push_str("API stability summary\n");
    text.push_str("Profile: ");
    text.push_str(profile.profile_id);
    text.push('\n');
    text.push_str("Compatibility profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
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

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn backend_family_label(family: &BackendFamily) -> String {
    match family {
        BackendFamily::Algorithmic => "Algorithmic".to_string(),
        BackendFamily::ReferenceData => "ReferenceData".to_string(),
        BackendFamily::CompressedData => "CompressedData".to_string(),
        BackendFamily::Composite => "Composite".to_string(),
        BackendFamily::Other(value) => format!("Other({value})"),
        _ => "Other(unknown)".to_string(),
    }
}

/// Renders a backend capability matrix for the implemented backend catalog.
pub fn render_backend_matrix_report() -> Result<String, EphemerisError> {
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

    for entry in implemented_backend_catalog() {
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

struct BackendMatrixDisplay<'a>(&'a BackendMatrixEntry);

impl fmt::Display for BackendMatrixDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_backend_catalog_entry(f, self.0)
    }
}

impl fmt::Display for ComparisonReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Comparison report")?;
        writeln!(f, "Corpus: {}", self.corpus_name)?;
        writeln!(f, "Apparentness: {}", self.apparentness)?;
        writeln!(f, "Reference backend: {}", self.reference_backend.id)?;
        writeln!(f, "Candidate backend: {}", self.candidate_backend.id)?;
        writeln!(f)?;
        write_comparison_summary(f, &self.summary)?;
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
        writeln!(f, "Backend: {}", self.backend.id)?;
        writeln!(f, "Corpus: {}", self.corpus_name)?;
        writeln!(f, "Apparentness: {}", self.apparentness)?;
        writeln!(f, "Rounds: {}", self.rounds)?;
        writeln!(f, "Samples per round: {}", self.sample_count)?;
        writeln!(f, "Elapsed: {:?}", self.elapsed)?;
        writeln!(
            f,
            "Nanoseconds per request: {}",
            format_ns(self.nanoseconds_per_request())
        )
    }
}

impl ComparisonReport {
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

    /// Returns per-body-class tolerance posture preserving first-seen class order.
    pub(crate) fn body_class_tolerance_summaries(&self) -> Vec<BodyClassToleranceSummary> {
        body_class_tolerance_summaries(&self.samples, &self.candidate_backend.family)
    }

    /// Returns the samples that exceed the built-in regression thresholds.
    pub fn notable_regressions(&self) -> Vec<RegressionFinding> {
        self.samples.iter().filter_map(regression_finding).collect()
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
    latitude_sum_deg: f64,
    distance_sum_au: f64,
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
            latitude_sum_deg: 0.0,
            distance_sum_au: 0.0,
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
        if sample.longitude_delta_deg >= self.max_longitude_delta_deg {
            self.max_longitude_delta_deg = sample.longitude_delta_deg;
            self.max_longitude_delta_body = Some(sample.body.clone());
        }
        self.latitude_sum_deg += sample.latitude_delta_deg;
        if sample.latitude_delta_deg >= self.max_latitude_delta_deg {
            self.max_latitude_delta_deg = sample.latitude_delta_deg;
            self.max_latitude_delta_body = Some(sample.body.clone());
        }
        if let Some(delta) = sample.distance_delta_au {
            self.distance_sum_au += delta;
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
            max_latitude_delta_body: self.max_latitude_delta_body,
            max_latitude_delta_deg: self.max_latitude_delta_deg,
            mean_latitude_delta_deg: if self.sample_count > 0 {
                self.latitude_sum_deg / sample_count
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
    latitude_sum_deg: f64,
    distance_sum_au: f64,
    distance_count: usize,
    max_longitude_delta_deg: f64,
    max_longitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: f64,
    max_latitude_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    max_distance_delta_body: Option<CelestialBody>,
}

impl BodyClassAccumulator {
    const fn new(class: BodyClass) -> Self {
        Self {
            class,
            sample_count: 0,
            longitude_sum_deg: 0.0,
            latitude_sum_deg: 0.0,
            distance_sum_au: 0.0,
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
        if sample.longitude_delta_deg >= self.max_longitude_delta_deg {
            self.max_longitude_delta_deg = sample.longitude_delta_deg;
            self.max_longitude_delta_body = Some(sample.body.clone());
        }
        self.latitude_sum_deg += sample.latitude_delta_deg;
        if sample.latitude_delta_deg >= self.max_latitude_delta_deg {
            self.max_latitude_delta_deg = sample.latitude_delta_deg;
            self.max_latitude_delta_body = Some(sample.body.clone());
        }
        if let Some(delta) = sample.distance_delta_au {
            self.distance_sum_au += delta;
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
        BodyClassSummary {
            class: self.class,
            sample_count: self.sample_count,
            max_longitude_delta_body: self.max_longitude_delta_body,
            max_longitude_delta_deg: self.max_longitude_delta_deg,
            sum_longitude_delta_deg: self.longitude_sum_deg,
            max_latitude_delta_body: self.max_latitude_delta_body,
            max_latitude_delta_deg: self.max_latitude_delta_deg,
            sum_latitude_delta_deg: self.latitude_sum_deg,
            max_distance_delta_body: self.max_distance_delta_body,
            max_distance_delta_au: self.max_distance_delta_au,
            sum_distance_delta_au: self.distance_sum_au,
            distance_count: self.distance_count,
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
    body_count: usize,
    sample_count: usize,
    within_tolerance_body_count: usize,
    outside_tolerance_body_count: usize,
    max_longitude_delta_body: Option<CelestialBody>,
    max_longitude_delta_deg: Option<f64>,
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: Option<f64>,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    outside_bodies: Vec<CelestialBody>,
}

impl BodyClassToleranceAccumulator {
    const fn new(class: BodyClass) -> Self {
        Self {
            class,
            body_count: 0,
            sample_count: 0,
            within_tolerance_body_count: 0,
            outside_tolerance_body_count: 0,
            max_longitude_delta_body: None,
            max_longitude_delta_deg: None,
            max_latitude_delta_body: None,
            max_latitude_delta_deg: None,
            max_distance_delta_body: None,
            max_distance_delta_au: None,
            outside_bodies: Vec::new(),
        }
    }

    fn push(&mut self, summary: &BodyToleranceSummary) {
        self.body_count += 1;
        self.sample_count += summary.sample_count;
        match self.max_longitude_delta_deg {
            Some(current) if summary.max_longitude_delta_deg < current => {}
            _ => {
                self.max_longitude_delta_deg = Some(summary.max_longitude_delta_deg);
                self.max_longitude_delta_body = Some(summary.body.clone());
            }
        }
        match self.max_latitude_delta_deg {
            Some(current) if summary.max_latitude_delta_deg < current => {}
            _ => {
                self.max_latitude_delta_deg = Some(summary.max_latitude_delta_deg);
                self.max_latitude_delta_body = Some(summary.body.clone());
            }
        }
        if let Some(delta) = summary.max_distance_delta_au {
            match self.max_distance_delta_au {
                Some(current) if delta < current => {}
                _ => {
                    self.max_distance_delta_au = Some(delta);
                    self.max_distance_delta_body = Some(summary.body.clone());
                }
            }
        }
        if summary.within_tolerance {
            self.within_tolerance_body_count += 1;
        } else {
            self.outside_tolerance_body_count += 1;
            self.outside_bodies.push(summary.body.clone());
        }
    }

    fn finish(self) -> BodyClassToleranceSummary {
        BodyClassToleranceSummary {
            class: self.class,
            body_count: self.body_count,
            sample_count: self.sample_count,
            within_tolerance_body_count: self.within_tolerance_body_count,
            outside_tolerance_body_count: self.outside_tolerance_body_count,
            max_longitude_delta_body: self.max_longitude_delta_body,
            max_longitude_delta_deg: self.max_longitude_delta_deg,
            max_latitude_delta_body: self.max_latitude_delta_body,
            max_latitude_delta_deg: self.max_latitude_delta_deg,
            max_distance_delta_body: self.max_distance_delta_body,
            max_distance_delta_au: self.max_distance_delta_au,
            outside_bodies: self.outside_bodies,
        }
    }
}

fn body_class_tolerance_summaries(
    samples: &[ComparisonSample],
    backend_family: &BackendFamily,
) -> Vec<BodyClassToleranceSummary> {
    let body_summaries = body_comparison_summaries(samples)
        .into_iter()
        .map(|summary| body_tolerance_summary(summary, backend_family))
        .collect::<Vec<_>>();
    let mut accumulators = BodyClass::ALL.map(BodyClassToleranceAccumulator::new);

    for summary in body_summaries {
        accumulators[body_class(&summary.body).index()].push(&summary);
    }

    accumulators
        .into_iter()
        .filter(|summary| summary.body_count > 0)
        .map(BodyClassToleranceAccumulator::finish)
        .collect()
}

fn comparison_tolerance_for_class(
    class: BodyClass,
    backend_family: &BackendFamily,
) -> ComparisonTolerance {
    match class {
        BodyClass::Luminary | BodyClass::MajorPlanet => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 full-file VSOP87B planetary evidence",
            max_longitude_delta_deg: REGRESSION_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: REGRESSION_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(REGRESSION_DISTANCE_THRESHOLD_AU),
        },
        BodyClass::LunarPoint => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 compact-ELP lunar evidence",
            max_longitude_delta_deg: REGRESSION_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: REGRESSION_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(REGRESSION_DISTANCE_THRESHOLD_AU),
        },
        BodyClass::Asteroid => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 asteroid comparison evidence",
            max_longitude_delta_deg: REGRESSION_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: REGRESSION_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(REGRESSION_DISTANCE_THRESHOLD_AU),
        },
        BodyClass::Custom => ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 uncategorized comparison evidence",
            max_longitude_delta_deg: REGRESSION_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: REGRESSION_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(REGRESSION_DISTANCE_THRESHOLD_AU),
        },
    }
}

fn comparison_tolerance_for_body(
    body: &CelestialBody,
    backend_family: &BackendFamily,
) -> ComparisonTolerance {
    if matches!(body, CelestialBody::Pluto) {
        return ComparisonTolerance {
            backend_family: backend_family.clone(),
            profile: "phase-1 Pluto mean-elements fallback evidence",
            max_longitude_delta_deg: REGRESSION_LONGITUDE_THRESHOLD_DEG,
            max_latitude_delta_deg: REGRESSION_LATITUDE_THRESHOLD_DEG,
            max_distance_delta_au: Some(REGRESSION_DISTANCE_THRESHOLD_AU),
        };
    }

    comparison_tolerance_for_class(body_class(body), backend_family)
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

    BodyToleranceSummary {
        body: summary.body,
        tolerance,
        sample_count: summary.sample_count,
        within_tolerance,
        max_longitude_delta_deg: summary.max_longitude_delta_deg,
        max_latitude_delta_deg: summary.max_latitude_delta_deg,
        max_distance_delta_au: summary.max_distance_delta_au,
    }
}

fn write_corpus_summary(f: &mut fmt::Formatter<'_>, corpus: &CorpusSummary) -> fmt::Result {
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

fn write_backend_matrix(f: &mut fmt::Formatter<'_>, backend: &BackendMetadata) -> fmt::Result {
    writeln!(f, "  id: {}", backend.id)?;
    writeln!(f, "  version: {}", backend.version)?;
    writeln!(f, "  family: {:?}", backend.family)?;
    writeln!(f, "  accuracy: {:?}", backend.accuracy)?;
    writeln!(f, "  deterministic: {}", backend.deterministic)?;
    writeln!(f, "  offline: {}", backend.offline)?;
    writeln!(
        f,
        "  nominal range: {}",
        format_time_range(&backend.nominal_range)
    )?;
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
    writeln!(f, "  provenance: {}", backend.provenance.summary)?;
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
            writeln!(
                f,
                "    {}: {:?}, {:?}, {}",
                profile.body, profile.kind, profile.accuracy, profile.provenance
            )?;
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

        writeln!(f, "  canonical J2000 VSOP87B evidence:")?;
        match vsop87_canonical_body_evidence() {
            Some(body_evidence) => {
                for evidence in body_evidence {
                    writeln!(
                        f,
                        "    {}: {:?} from {} — {} — Δlon={:.12}°, Δlat={:.12}°, Δdist={:.12} AU",
                        evidence.body,
                        evidence.source_kind,
                        evidence.source_file,
                        if evidence.within_interim_limits {
                            evidence.provenance
                        } else {
                            "outside interim limits"
                        },
                        evidence.longitude_delta_deg,
                        evidence.latitude_delta_deg,
                        evidence.distance_delta_au
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
        writeln!(f, "    model: {}", theory.model_name)?;
        writeln!(
            f,
            "    source family: {}",
            pleiades_elp::lunar_theory_source_family().label()
        )?;
        writeln!(
            f,
            "    capability summary: {}",
            format_lunar_theory_capability_summary(&lunar_theory_capability_summary())
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
            "    request policy: frames={}, time scales={}, zodiac modes={}, apparentness={}, topocentric observer={}",
            format_frames(theory.request_policy.supported_frames),
            format_time_scales(theory.request_policy.supported_time_scales),
            format_zodiac_modes(theory.request_policy.supported_zodiac_modes),
            format_apparentness_modes(theory.request_policy.supported_apparentness),
            theory.request_policy.supports_topocentric_observer,
        )?;
        writeln!(
            f,
            "    validation window: {}",
            format_time_range(&theory.validation_window)
        )?;
        writeln!(f, "    date-range note: {}", theory.date_range_note)?;
        writeln!(f, "    frame note: {}", theory.frame_note)?;
        write_lunar_reference_evidence(f)?;
    }
    if entry.metadata.id.as_str() == "jpl-snapshot" {
        write_jpl_interpolation_quality(f)?;
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
        "    note: expanded public-input leave-one-out checks report current runtime interpolation error against held-out exact rows; they are not production tolerances"
    )?;
    for sample in interpolation_quality_samples() {
        writeln!(
            f,
            "    {} at JD {:.1}: bracket span {:.1} d, |Δlon|={:.12}°, |Δlat|={:.12}°, |Δdist|={:.12} AU",
            sample.body,
            sample.epoch.julian_day.days(),
            sample.bracket_span_days,
            sample.longitude_error_deg,
            sample.latitude_error_deg,
            sample.distance_error_au
        )?;
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

fn format_jpl_interpolation_quality_summary_for_report() -> String {
    match jpl_interpolation_quality_summary() {
        Some(summary) => format_jpl_interpolation_quality_summary(&summary),
        None => "JPL interpolation quality: unavailable".to_string(),
    }
}

fn write_lunar_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar reference evidence:")?;
    let Some(summary) = lunar_reference_evidence_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        pleiades_elp::format_lunar_reference_evidence_summary(&summary)
    )?;
    for sample in lunar_reference_evidence() {
        writeln!(
            f,
            "    {} at JD {:.1}: lon={:.12}°, lat={:.12}°, dist={}, note={}",
            sample.body,
            sample.epoch.julian_day.days(),
            sample.longitude_deg,
            sample.latitude_deg,
            sample
                .distance_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            sample.note
        )?;
    }
    Ok(())
}

fn write_comparison_summary(
    f: &mut fmt::Formatter<'_>,
    summary: &ComparisonSummary,
) -> fmt::Result {
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
        "  max latitude delta: {:.12}°",
        summary.max_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean latitude delta: {:.12}°",
        summary.mean_latitude_delta_deg
    )?;
    if let Some(value) = summary.max_distance_delta_au {
        writeln!(f, "  max distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.mean_distance_delta_au {
        writeln!(f, "  mean distance delta: {:.12} AU", value)?;
    }
    Ok(())
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
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, max Δdist={}{}, mean Δdist={}",
            summary.body,
            summary.sample_count,
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_latitude_delta_deg,
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
                .unwrap_or_else(|| "n/a".to_string())
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

fn write_tolerance_policy(
    f: &mut fmt::Formatter<'_>,
    candidate_backend: &BackendMetadata,
) -> fmt::Result {
    let family_label = tolerance_backend_family_label(&candidate_backend.family);
    writeln!(f, "Tolerance policy")?;
    writeln!(f, "  candidate backend family: {}", family_label)?;
    for class in [
        BodyClass::Luminary,
        BodyClass::MajorPlanet,
        BodyClass::LunarPoint,
        BodyClass::Asteroid,
        BodyClass::Custom,
    ] {
        let tolerance = comparison_tolerance_for_class(class, &candidate_backend.family);
        writeln!(
            f,
            "  {}: backend family={}, profile={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}",
            class.label(),
            tolerance_backend_family_label(&tolerance.backend_family),
            tolerance.profile,
            tolerance.max_longitude_delta_deg,
            tolerance.max_latitude_delta_deg,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )?;
    }

    let pluto = comparison_tolerance_for_body(&CelestialBody::Pluto, &candidate_backend.family);
    writeln!(
        f,
        "  Pluto override: backend family={}, profile={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}",
        tolerance_backend_family_label(&pluto.backend_family),
        pluto.profile,
        pluto.max_longitude_delta_deg,
        pluto.max_latitude_delta_deg,
        pluto
            .max_distance_delta_au
            .map(|value| format!("{value:.6} AU"))
            .unwrap_or_else(|| "n/a".to_string())
    )?;

    Ok(())
}

fn write_tolerance_policy_text(text: &mut String, candidate_backend: &BackendMetadata) {
    use std::fmt::Write as _;

    let family_label = tolerance_backend_family_label(&candidate_backend.family);
    let _ = writeln!(text, "Tolerance policy");
    let _ = writeln!(text, "  candidate backend family: {}", family_label);
    for class in [
        BodyClass::Luminary,
        BodyClass::MajorPlanet,
        BodyClass::LunarPoint,
        BodyClass::Asteroid,
        BodyClass::Custom,
    ] {
        let tolerance = comparison_tolerance_for_class(class, &candidate_backend.family);
        let _ = writeln!(
            text,
            "  {}: backend family={}, profile={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}",
            class.label(),
            tolerance_backend_family_label(&tolerance.backend_family),
            tolerance.profile,
            tolerance.max_longitude_delta_deg,
            tolerance.max_latitude_delta_deg,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let pluto = comparison_tolerance_for_body(&CelestialBody::Pluto, &candidate_backend.family);
    let _ = writeln!(
        text,
        "  Pluto override: backend family={}, profile={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}",
        tolerance_backend_family_label(&pluto.backend_family),
        pluto.profile,
        pluto.max_longitude_delta_deg,
        pluto.max_latitude_delta_deg,
        pluto
            .max_distance_delta_au
            .map(|value| format!("{value:.6} AU"))
            .unwrap_or_else(|| "n/a".to_string())
    );
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
        writeln!(
            f,
            "  {}: backend family={}, profile={}, samples={}, status={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}, measured max Δlon={:.12}°, max Δlat={:.12}°, max Δdist={}",
            summary.body,
            tolerance_backend_family_label(&summary.tolerance.backend_family),
            summary.tolerance.profile,
            summary.sample_count,
            if summary.within_tolerance { "within" } else { "exceeded" },
            summary.tolerance.max_longitude_delta_deg,
            summary.tolerance.max_latitude_delta_deg,
            summary
                .tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary.max_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )?;
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
        writeln!(
            f,
            "  {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
            finding.body,
            finding.longitude_delta_deg,
            finding.latitude_delta_deg,
            finding
                .distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            finding.note
        )?;
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
        writeln!(
            f,
            "  {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
            finding.body,
            finding.longitude_delta_deg,
            finding.latitude_delta_deg,
            finding
                .distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            finding.note
        )?;
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

fn regression_finding(sample: &ComparisonSample) -> Option<RegressionFinding> {
    let mut notes = Vec::new();
    if sample.longitude_delta_deg >= REGRESSION_LONGITUDE_THRESHOLD_DEG {
        notes.push(format!(
            "longitude delta exceeds {:.1}°",
            REGRESSION_LONGITUDE_THRESHOLD_DEG
        ));
    }
    if sample.latitude_delta_deg >= REGRESSION_LATITUDE_THRESHOLD_DEG {
        notes.push(format!(
            "latitude delta exceeds {:.1}°",
            REGRESSION_LATITUDE_THRESHOLD_DEG
        ));
    }
    if sample
        .distance_delta_au
        .is_some_and(|value| value >= REGRESSION_DISTANCE_THRESHOLD_AU)
    {
        notes.push(format!(
            "distance delta exceeds {:.2} AU",
            REGRESSION_DISTANCE_THRESHOLD_AU
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
            status_note: "checked-in public-input derivative fixture with exact lookup and cubic interpolation on four-sample windows when available, with quadratic and linear fallbacks for sparser bodies; expanded hold-out coverage now spans an additional epoch, while a larger reference corpus remains planned",
            expected_error_kinds: JPL_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
        BackendMatrixEntry {
            label: "VSOP87 planetary backend",
            metadata: Vsop87Backend::new().metadata(),
            implementation_status: BackendImplementationStatus::PartialSourceBacked,
            status_note: "Sun, Mercury, Venus, and Mars now use generated binary VSOP87B source tables, Jupiter, Saturn, Uranus, and Neptune still use vendored full-file source files, and Pluto remains a mean-element fallback pending a selected source path",
            expected_error_kinds: VSOP87_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "ELP lunar backend (Moon and lunar nodes)",
            metadata: ElpBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
            status_note: "compact lunar and lunar-point formulas provide deterministic interim behavior pending documented production lunar-theory ingestion",
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
        .map(|frame| match frame {
            CoordinateFrame::Ecliptic => "Ecliptic",
            CoordinateFrame::Equatorial => "Equatorial",
            _ => "Other",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_time_scales(scales: &[TimeScale]) -> String {
    scales
        .iter()
        .map(|scale| match scale {
            TimeScale::Utc => "UTC",
            TimeScale::Ut1 => "UT1",
            TimeScale::Tt => "TT",
            TimeScale::Tdb => "TDB",
            _ => "Other",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_zodiac_modes(modes: &[ZodiacMode]) -> String {
    modes
        .iter()
        .map(|mode| match mode {
            ZodiacMode::Tropical => "Tropical".to_string(),
            ZodiacMode::Sidereal { ayanamsa } => format!("Sidereal ({ayanamsa:?})"),
            _ => "Other".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    modes
        .iter()
        .map(|mode| mode.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_capabilities(capabilities: &BackendCapabilities) -> String {
    format!(
        "geocentric={}, topocentric={}, apparent={}, mean={}, batch={}, native_sidereal={}",
        capabilities.geocentric,
        capabilities.topocentric,
        capabilities.apparent,
        capabilities.mean,
        capabilities.batch,
        capabilities.native_sidereal
    )
}

fn format_error_kinds(kinds: &[EphemerisErrorKind]) -> String {
    kinds
        .iter()
        .map(|kind| format!("{:?}", kind))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_data_files(files: &[&str]) -> String {
    files.join("; ")
}

fn format_time_range(range: &TimeRange) -> String {
    match (range.start, range.end) {
        (Some(start), Some(end)) => format!("{} → {}", format_instant(start), format_instant(end)),
        (Some(start), None) => format!("from {}", format_instant(start)),
        (None, Some(end)) => format!("through {}", format_instant(end)),
        (None, None) => "unbounded".to_string(),
    }
}

fn format_instant(instant: Instant) -> String {
    let scale = match instant.scale {
        TimeScale::Utc => "UTC",
        TimeScale::Ut1 => "UT1",
        TimeScale::Tt => "TT",
        TimeScale::Tdb => "TDB",
        _ => "Other",
    };
    format!("JD {:.1} ({scale})", instant.julian_day.days())
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
        "{banner}\n\nCommands:\n  compare-backends          Compare the JPL snapshot against the algorithmic composite backend\n  compare-backends-audit    Compare the JPL snapshot against the algorithmic composite backend and fail if the tolerance audit reports regressions\n  backend-matrix            Print the implemented backend capability matrices\n  capability-matrix         Alias for backend-matrix\n  backend-matrix-summary    Print the compact backend capability matrix summary\n  matrix-summary            Alias for backend-matrix-summary\n  compatibility-profile     Print the release compatibility profile\n  profile                   Alias for compatibility-profile\n  benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus with guard epochs\n  report [--rounds N]       Render the full validation report\n  generate-report           Alias for report\n  validation-report-summary [--rounds N]  Render a compact validation report summary\n  report-summary [--rounds N]  Alias for validation-report-summary\n  validation-summary        Alias for validation-report-summary\n  validate-artifact         Inspect and validate the bundled compressed artifact\n  artifact-summary          Print the compact packaged-artifact summary\n  artifact-posture-summary  Alias for artifact-summary\n  workspace-audit           Check the workspace for mandatory native build hooks\n  audit                     Alias for workspace-audit\n  api-stability             Print the release API stability posture\n  api-posture               Alias for api-stability\n  api-stability-summary     Print the compact API stability summary\n  api-posture-summary       Alias for api-stability-summary\n  compatibility-profile-summary  Print the compact compatibility profile summary\n  profile-summary           Alias for compatibility-profile-summary\n  verify-compatibility-profile  Verify the release compatibility profile against the canonical catalogs\n  release-notes             Print the release compatibility notes\n  release-notes-summary     Print the compact release notes summary\n  release-checklist         Print the release maintainer checklist\n  release-checklist-summary Print the compact release checklist summary\n  checklist-summary        Alias for release-checklist-summary\n  release-summary           Print the compact release summary\n  bundle-release --out DIR  Write the release compatibility profile, profile summary, release notes, release notes summary, release summary, release checklist, release checklist summary, backend matrix, backend matrix summary, API posture, API stability summary, validation report summary, artifact summary, validation report, manifest, and manifest checksum sidecar\n  verify-release-bundle     Read a staged release bundle back and verify its manifest checksums\n  help                      Show this help text\n\nDefault benchmark rounds: {DEFAULT_BENCHMARK_ROUNDS}\nDefault comparison corpus size: {corpus_size}",
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

fn render_error(error: EphemerisError) -> String {
    error.to_string()
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
        current_release_profile_identifiers, sidereal_longitude, Apparentness, Ayanamsa,
        CoordinateFrame, HouseSystem, JulianDay, TimeScale, ZodiacMode,
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
        assert_eq!(corpus.requests.len(), 41);
        assert_eq!(summary.epoch_count, 6);
        assert_eq!(summary.epochs.len(), 6);
        assert!(summary
            .epochs
            .iter()
            .all(|epoch| epoch.scale == TimeScale::Tt));
        assert_eq!(summary.epochs[0].julian_day.days(), 2_378_499.0);
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
            .any(|request| request.instant.julian_day.days() == 2_378_499.0));
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
    fn comparison_report_uses_the_snapshot_backend() {
        let report = render_comparison_report().expect("comparison should render");
        assert!(report.contains("JPL Horizons comparison window"));
        assert!(report.contains("Apparentness: Mean"));
        assert!(report.contains("Reference backend:"));
        assert!(report.contains("Candidate backend:"));
    }

    #[test]
    fn comparison_audit_command_reports_regressions() {
        let error = render_cli(&["compare-backends-audit"])
            .expect_err("comparison audit should fail while regressions remain");
        assert!(error.contains("comparison audit failed"));
        assert!(error.contains("Comparison tolerance audit"));
        assert!(error.contains("Body-class error envelopes"));
        assert!(error.contains("Body-class tolerance posture"));
        assert!(error.contains("Tolerance policy"));
        assert!(error.contains("Notable regressions"));
        assert!(error.contains("regression bodies: Pluto"));
        assert!(error.contains("Pluto"));
    }

    #[test]
    fn benchmark_report_renders_a_time_summary() {
        let report = render_benchmark_report(10).expect("benchmark should render");
        assert!(report.contains("Benchmark report"));
        assert!(report.contains("Representative 1500-2500 window"));
        assert!(report.contains("Apparentness: Mean"));
        assert!(report.contains("Nanoseconds per request:"));
    }

    #[test]
    fn validation_report_includes_corpus_metadata() {
        let report = render_validation_report(10).expect("validation report should render");
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
        assert!(report.contains("Implemented backend matrices"));
        assert!(report.contains("Selected asteroid coverage"));
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
        assert!(report.contains("JPL Horizons comparison window"));
        assert!(report.contains("Apparentness: Mean"));
        assert!(report.contains("Benchmark corpus"));
        assert!(report.contains("Representative 1500-2500 window"));
        assert!(report.contains("House validation corpus"));
        assert!(report.contains("Mid-latitude reference chart"));
        assert!(report.contains("Polar stress chart"));
        assert!(report.contains("Equatorial reference chart"));
        assert!(report.contains("Southern hemisphere reference chart"));
        assert!(report.contains("Reference backend"));
        assert!(report.contains("Candidate backend"));
        assert!(report.contains("Comparison summary"));
        assert!(report.contains("Body-class error envelopes"));
        let body_class_envelopes = report
            .split("Body-class error envelopes")
            .nth(1)
            .expect("report should include body-class error envelopes");
        assert!(body_class_envelopes.contains("max Δlon="));
        assert!(body_class_envelopes.contains(" ("));
        assert!(report.contains("Body-class tolerance posture"));
        assert!(report.contains("Tolerance policy"));
        assert!(report.contains("candidate backend family: composite"));
        assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
        assert!(report.contains(
            "Pluto override: backend family=composite, profile=phase-1 Pluto mean-elements fallback evidence"
        ));
        assert!(report.contains("Luminaries"));
        assert!(report.contains("Major planets"));
        assert!(report.contains("interpolation quality checks:"));
        assert!(report.contains("JPL interpolation quality: 21 samples across 10 bodies"));
        assert!(report.contains("Lunar reference evidence"));
        assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5..2459278.5"
        ));
        assert!(report.contains("exact J2000 evidence: 5 bodies at JD 2451545.0"));
        assert!(report.contains("Body comparison summaries"));
        assert!(report.contains("Sun: samples="));
        assert!(report.contains("Notable regressions"));
        assert!(report.contains("Archived regression cases"));
        assert!(report.contains("Reference benchmark"));
        assert!(report.contains("Candidate benchmark"));
        assert!(report.contains("Packaged-data benchmark corpus"));
        assert!(report.contains("Packaged-data benchmark"));
    }

    #[test]
    fn validation_report_summary_renders_a_compact_overview() {
        let report =
            render_validation_report_summary(10).expect("validation summary should render");
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
        assert!(report.contains("epoch labels: JD 2378499.0 (TT)"));
        assert!(report.contains("Comparison summary"));
        assert!(report.contains("Tolerance policy"));
        assert!(report.contains("candidate backend family: composite"));
        assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
        assert!(report.contains(
            "Pluto override: backend family=composite, profile=phase-1 Pluto mean-elements fallback evidence"
        ));
        assert!(report.contains("Comparison tolerance audit"));
        assert!(report.contains("command: compare-backends-audit"));
        assert!(report.contains("regressions found"));
        assert!(report.contains("regression bodies: Pluto"));
        assert!(report.contains("JPL interpolation quality"));
        assert!(report.contains("JPL interpolation quality: 21 samples across 10 bodies"));
        assert!(report.contains("leave-one-out runtime interpolation evidence"));
        assert!(report.contains("@ JD"));
        assert!(report.contains("Lunar reference evidence"));
        assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5..2459278.5"
        ));
        assert!(report.contains("Body comparison summaries"));
        assert!(report.contains("Body-class error envelopes"));
        let body_class_envelopes = report
            .split("Body-class error envelopes")
            .nth(1)
            .expect("report should include body-class error envelopes");
        assert!(body_class_envelopes.contains("max Δlon="));
        assert!(body_class_envelopes.contains(" ("));
        assert!(report.contains("VSOP87 source-backed evidence"));
        assert!(report
            .contains("VSOP87 source audit: 8 source-backed bodies, 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
        assert!(report.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(report.contains("generated binary VSOP87B"));
        assert!(report.contains("generated binary VSOP87B; VSOP87B."));
        assert!(report.contains("max Δlon="));
        assert!(report.contains("max Δlat="));
        assert!(report.contains("max Δdist="));
        assert!(report.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), 8 within interim limits"
        ));
        assert!(report.contains("House validation corpus"));
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
        assert!(rendered.contains("Body comparison summaries"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(rendered.contains("Packaged-artifact profile"));
        assert!(rendered.contains("Benchmark summaries"));

        let validation_report_summary =
            render_cli(&["validation-report-summary", "--rounds", "10"])
                .expect("validation-report-summary should render");
        assert!(validation_report_summary.contains("Validation report summary"));
        assert!(validation_report_summary.contains("Comparison corpus"));
        assert!(validation_report_summary.contains("Body comparison summaries"));
        assert!(validation_report_summary.contains("Body-class error envelopes"));
        assert!(validation_report_summary.contains("Body-class tolerance posture"));
        assert!(validation_report_summary.contains("Expected tolerance status"));
        assert!(validation_report_summary.contains("Comparison tolerance audit"));
        assert!(validation_report_summary.contains("command: compare-backends-audit"));
        assert!(validation_report_summary.contains("regressions found"));
        assert!(validation_report_summary.contains("regression bodies: Pluto"));
        assert!(validation_report_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(validation_report_summary.contains("Release notes summary: release-notes-summary"));
        assert!(validation_report_summary.contains("Packaged-artifact profile"));
        assert!(validation_report_summary.contains("Benchmark summaries"));
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
        assert_eq!(report.scenarios.len(), 4);
        assert!(report
            .scenarios
            .iter()
            .any(|scenario| scenario.label == "Southern hemisphere reference chart"));
    }

    #[test]
    fn comparison_report_surfaces_regressions() {
        let corpus = default_corpus();
        let report = compare_backends(
            &default_reference_backend(),
            &default_candidate_backend(),
            &corpus,
        )
        .expect("comparison should succeed");

        let regressions = report.notable_regressions();
        assert!(!regressions.is_empty());
        assert!(regressions
            .iter()
            .any(|finding| finding.body == CelestialBody::Pluto));
        assert!(!regressions
            .iter()
            .any(|finding| finding.body == CelestialBody::Neptune));

        let body_summaries = report.body_summaries();
        assert_eq!(body_summaries.len(), comparison_bodies().len());
        assert!(body_summaries
            .iter()
            .all(|summary| summary.sample_count > 0));
        assert!(body_summaries
            .iter()
            .any(|summary| summary.body == CelestialBody::Jupiter
                && summary.max_longitude_delta_deg > 0.0
                && summary.max_longitude_delta_deg < 0.01));

        let archive = report.regression_archive();
        assert_eq!(archive.corpus_name, corpus.name);
        assert_eq!(archive.cases.len(), regressions.len());
        assert!(archive
            .cases
            .iter()
            .any(|finding| finding.body == CelestialBody::Pluto));
        let tolerance_summaries = report.tolerance_summaries();
        assert_eq!(tolerance_summaries.len(), body_summaries.len());
        assert!(tolerance_summaries.iter().any(|summary| {
            summary.body == CelestialBody::Jupiter
                && summary.tolerance.profile.contains("full-file VSOP87B")
        }));
        assert!(tolerance_summaries
            .iter()
            .any(|summary| summary.body == CelestialBody::Pluto && !summary.within_tolerance));
        let body_class_tolerance_summaries = report.body_class_tolerance_summaries();
        assert!(body_class_tolerance_summaries.iter().any(|summary| {
            summary.class == BodyClass::MajorPlanet
                && summary.body_count >= 1
                && summary.sample_count >= summary.body_count
                && summary.outside_tolerance_body_count >= 1
                && summary.outside_bodies.contains(&CelestialBody::Pluto)
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
        assert!(rendered.contains("report [--rounds N]"));
        assert!(rendered.contains("generate-report"));
        assert!(rendered.contains("validation-report-summary [--rounds N]"));
        assert!(rendered.contains("report-summary [--rounds N]"));
        assert!(rendered.contains("validation-summary"));
        assert!(rendered.contains("validate-artifact"));
        assert!(rendered.contains("artifact-summary"));
        assert!(rendered.contains("workspace-audit"));
        assert!(rendered.contains("api-stability"));
        assert!(rendered.contains("api-stability-summary"));
        assert!(rendered.contains("compatibility-profile-summary"));
        assert!(rendered.contains("verify-compatibility-profile"));
        assert!(rendered.contains("release-notes"));
        assert!(rendered.contains("release-notes-summary"));
        assert!(rendered.contains("release-checklist"));
        assert!(rendered.contains("release-checklist-summary"));
        assert!(rendered.contains("release-summary"));
        assert!(rendered.contains("bundle-release --out DIR"));
        assert!(rendered.contains("API stability summary"));
        assert!(rendered.contains("profile-summary"));
        assert!(rendered.contains("backend matrix"));
        assert!(rendered.contains("verify-release-bundle"));
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
        assert!(rendered.contains("API stability summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains(&format!(
            "Compatibility profile: {}",
            release_profiles.compatibility_profile_id
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
        assert!(rendered.contains("Release-specific coverage beyond baseline:"));
        assert!(rendered.contains("Alias mappings for built-in house systems:"));
        assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
        assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
        assert!(rendered.contains("Poli-equatorial"));
        assert!(rendered.contains("horizon/azimuth"));
        assert!(rendered.contains("Babylonian Huber"));
        assert!(rendered.contains("Galactic Equator (True)"));
        assert!(rendered.contains("Valens Moon ayanamsa"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_recent_release_profile_entries() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Equal (MC) table of houses"));
        assert!(rendered.contains("Equal (1=Aries) table of houses"));
        assert!(rendered.contains("Equal table of houses"));
        assert!(rendered.contains("Whole Sign system"));
        assert!(rendered.contains("Morinus house system"));
        assert!(rendered.contains("Pullen SR (Sinusoidal Ratio) table of houses"));
        assert!(rendered.contains("True Citra Paksha"));
        assert!(rendered.contains("Galactic Equator (Fiorenza)"));
        assert!(rendered.contains("Valens Moon"));
        assert!(rendered.contains("Babylonian (House Obs)"));
        assert!(rendered.contains("Sunil Sheoran / Vedic Sheoran / Sheoran ayanamsa spellings"));
        assert!(rendered.contains("P.V.R. Narasimha Rao"));
        assert!(rendered.contains("B. V. Raman"));
        assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
        assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
        assert!(rendered.contains("Poli-equatorial"));
        assert!(rendered.contains("horizon/azimuth"));
        assert!(rendered.contains("Babylonian Huber"));
        assert!(rendered.contains("Galactic Equator (True)"));
        assert!(rendered.contains("Valens Moon ayanamsa"));
    }

    #[test]
    fn compatibility_profile_command_surfaces_reference_frame_and_zero_point_entries() {
        let rendered =
            render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
        assert!(rendered.contains("Suryasiddhanta (499 CE)"));
        assert!(rendered.contains("Aryabhata (499 CE)"));
        assert!(rendered.contains("Sassanian"));
        assert!(rendered.contains("PVR Pushya-paksha"));
        assert!(rendered.contains("Galactic Center (Rgilbrand)"));
        assert!(rendered.contains("Galactic Center (Mardyks)"));
        assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
        assert!(rendered.contains("Galactic Center (Cochrane)"));
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
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(
            rendered.contains("Compatibility profile verification: verify-compatibility-profile")
        );
        assert!(rendered.contains("API stability posture:"));
        assert!(rendered.contains("Deprecation policy:"));
        assert!(rendered.contains("Release-specific coverage:"));
        assert!(rendered.contains("selected asteroid coverage"));
        assert!(rendered.contains("Selected asteroid evidence: 5 exact J2000 samples"));
        assert!(rendered.contains("Reference snapshot coverage: 46 rows across 15 bodies and 6 epochs (5 asteroid rows; JD 2378499.0..2634167.0)"));
        assert!(rendered.contains("asteroid:433-Eros"));
        assert!(rendered.contains("Validation reference points:"));
        assert!(rendered.contains("Compatibility caveats:"));
        assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
        assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
        assert!(rendered.contains("Babylonian Huber"));
        assert!(rendered.contains("Galactic Equator (True)"));
        assert!(rendered.contains("Valens Moon ayanamsa"));
    }

    #[test]
    fn compatibility_profile_summary_command_renders_the_summary() {
        let rendered = render_cli(&["compatibility-profile-summary"])
            .expect("compatibility profile summary should render");
        let release_profiles = current_release_profile_identifiers();
        assert!(rendered.contains("Compatibility profile summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        let coverage = metadata_coverage();
        assert!(rendered.contains("House systems:"));
        assert!(rendered.contains("Ayanamsas:"));
        assert!(rendered.contains(&format!(
            "Ayanamsa sidereal metadata: {}/{} entries with both a reference epoch and offset",
            coverage.with_sidereal_metadata, coverage.total
        )));
        assert!(rendered.contains("Custom-definition labels:"));
        assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
        assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(rendered.contains("Compatibility caveats:"));
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
        assert!(rendered.contains("House systems verified:"));
        assert!(rendered.contains("Ayanamsas verified:"));
        assert!(rendered.contains("Baseline/release slices:"));
        assert!(rendered.contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
        assert!(rendered.contains(&format!(
            "Custom-definition labels verified: {} labels, all remain custom-definition territory",
            profile.custom_definition_labels.len()
        )));
        assert!(rendered.contains(&format!(
            "Compatibility caveats documented: {}",
            profile.known_gaps.len()
        )));
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
    fn compatibility_profile_verification_rejects_custom_definition_labels_that_resolve_to_builtins(
    ) {
        let labels = ["Placidus"];

        let error = verify_custom_definition_labels(&labels)
            .expect_err("custom-definition labels should stay outside built-ins");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("should remain unresolved as a built-in house system"));
    }

    #[test]
    fn release_notes_summary_command_renders_the_summary() {
        let rendered =
            render_cli(&["release-notes-summary"]).expect("release notes summary should render");
        let release_profiles = current_release_profile_identifiers();
        assert!(rendered.contains("Release notes summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Release-specific coverage:"));
        assert!(rendered.contains("Custom-definition labels:"));
        assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
        assert!(rendered.contains("Compatibility caveats:"));
        assert!(rendered.contains("API stability summary line:"));
        assert!(rendered.contains("Release notes: release-notes"));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered.contains("Reference snapshot coverage: 46 rows across 15 bodies and 6 epochs (5 asteroid rows; JD 2378499.0..2634167.0)"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
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
        assert!(rendered.contains("Validation report summary: validation-report-summary / validation-summary / report-summary"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
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
        assert!(rendered.contains("Validation report summary: validation-report-summary / validation-summary / report-summary"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(
            rendered.contains("Compatibility profile verification: verify-compatibility-profile")
        );
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(rendered.contains("Release summary: release-summary"));
        assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Repository-managed release gates: 7 items"));
        assert!(rendered.contains("Manual bundle workflow: 3 items"));
        assert!(rendered.contains("Bundle contents: 16 items"));
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
        assert!(rendered.contains("Release summary"));
        assert!(rendered.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(rendered.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(rendered.contains("Release summary line:"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(rendered.contains("Workspace audit: workspace-audit / audit"));
        assert!(rendered.contains("House systems:"));
        assert!(rendered.contains("Ayanamsas:"));
        assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
        assert!(rendered.contains("Custom-definition labels:"));
        assert!(rendered.contains("Custom-definition ayanamsas:"));
        assert!(rendered.contains("Compatibility caveats:"));
        assert!(rendered.contains("Validation evidence:"));
        assert!(rendered.contains("comparison samples"));
        assert!(rendered.contains("notable regressions"));
        assert!(rendered.contains("outside-tolerance bodies"));
        assert!(rendered.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release gate reminders:"));
        assert!(rendered.contains("verify-compatibility-profile"));
        assert!(rendered.contains("See release-notes and release-checklist"));
    }

    #[test]
    fn backend_matrix_command_renders_the_implemented_catalog() {
        let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
        assert!(rendered.contains("Implemented backend matrices"));
        assert!(rendered.contains("JPL snapshot reference backend"));
        assert!(rendered.contains(
            "selected asteroid coverage: 5 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        ));
        assert!(rendered.contains("exact J2000 evidence: 5 bodies at JD 2451545.0"));
        assert!(rendered.contains("nominal range:"));
        assert!(rendered.contains("provenance sources:"));
        assert!(rendered.contains("implementation status: fixture-reference"));
        assert!(rendered.contains("implementation status: partial-source-backed"));
        assert!(rendered.contains("implementation status: preliminary-algorithm"));
        assert!(rendered.contains("implementation status: prototype-artifact"));
        assert!(rendered.contains("implementation status: routing-facade"));
        assert!(rendered.contains("implementation note:"));
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
        assert!(rendered.contains("Sun: GeneratedBinaryVsop87b from VSOP87B.ear"));
        assert!(rendered.contains("Mercury: GeneratedBinaryVsop87b from VSOP87B.mer"));
        assert!(rendered.contains("Venus: GeneratedBinaryVsop87b from VSOP87B.ven"));
        assert!(rendered.contains("Mars: GeneratedBinaryVsop87b from VSOP87B.mar"));
        assert!(rendered.contains("Jupiter: GeneratedBinaryVsop87b"));
        assert!(rendered.contains("Saturn: GeneratedBinaryVsop87b"));
        assert!(rendered.contains("Uranus: GeneratedBinaryVsop87b"));
        assert!(rendered.contains("Neptune: GeneratedBinaryVsop87b"));
        assert!(rendered.contains("Pluto: MeanOrbitalElements"));
        assert!(rendered.contains("Meeus-style truncated lunar orbit formulas"));
        assert!(rendered.contains("NASA/JPL Horizons API vector tables (DE441)"));
        assert!(rendered.contains("interpolation quality checks:"));
        assert!(rendered.contains("expanded public-input leave-one-out checks"));
        assert!(rendered.contains("Mars at JD 2451545.0"));
        assert!(rendered.contains("VSOP87 planetary backend"));
        assert!(rendered.contains("ELP lunar backend (Moon and lunar nodes)"));
        assert!(rendered.contains("unsupported bodies: True Apogee, True Perigee"));
        assert!(rendered.contains("Packaged data backend"));
        assert!(rendered.contains("Composite routed backend"));
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
        assert!(rendered.contains("fixture-reference: 1"));
        assert!(rendered.contains("partial-source-backed: 1"));
        assert!(rendered.contains("preliminary-algorithm: 1"));
        assert!(rendered.contains("prototype-artifact: 1"));
        assert!(rendered.contains("routing-facade: 1"));
        assert!(rendered.contains("Accuracy classes:"));
        assert!(rendered.contains("Exact: 1"));
        assert!(rendered.contains("Approximate: 4"));
        assert!(rendered.contains("VSOP87 source documentation: 8 source specs, 8 source-backed body profiles, 1 fallback mean-element body profile"));
        assert!(rendered.contains(
            "source-backed breakdown: 8 generated binary, 0 vendored full-file, 0 truncated slice"
        ));
        assert!(rendered.contains(
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        ));
        assert!(rendered
            .contains("VSOP87 source audit: 8 source-backed bodies, 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
        assert!(rendered.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(rendered.contains("generated binary VSOP87B"));
        assert!(rendered.contains("generated binary VSOP87B; VSOP87B."));
        assert!(rendered.contains("max Δlon="));
        assert!(rendered.contains("max Δlat="));
        assert!(rendered.contains("max Δdist="));
        assert!(rendered.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), 8 within interim limits"
        ));
        assert!(rendered.contains(
            "ELP lunar theory specification: Compact Meeus-style truncated lunar baseline [meeus-style-truncated-lunar-baseline; family: Meeus-style truncated analytical baseline]"
        ));
        assert!(rendered.contains("request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"));
        assert!(rendered.contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
        assert!(rendered.contains("date-range note: Validated against the published 1992-04-12 geocentric Moon example, J2000 lunar-point anchors, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example"));
        assert!(rendered.contains("citation: Jean Meeus"));
        assert!(rendered
            .contains("provenance: Published lunar position, node, and mean-point formulas"));
        assert!(rendered.contains(
            "redistribution: No external coefficient-file redistribution constraints apply"
        ));
        assert!(rendered.contains("license: The current baseline is handwritten pure Rust"));
        assert!(rendered.contains("2 unsupported bodies: True Apogee, True Perigee"));
        assert!(rendered.contains("Distinct bodies covered:"));
        assert!(rendered.contains("Distinct coordinate frames:"));
        assert!(rendered.contains("Distinct time scales:"));
        assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(rendered.contains("API stability summary: api-stability-summary"));
        assert!(rendered.contains("Release notes summary: release-notes-summary"));
        assert!(rendered.contains("Reference snapshot coverage: 46 rows across 15 bodies and 6 epochs (5 asteroid rows; JD 2378499.0..2634167.0)"));
        assert!(rendered
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
        assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
        assert!(
            rendered.contains("Validation report summary: validation-report-summary / validation-summary / report-summary")
        );
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
    fn workspace_audit_detects_native_hooks_in_manifests_and_lockfile() {
        let manifest = r#"[package]
name = "example"
build = "build.rs"
links = "example-native"

[dependencies]
cc = "1"
[target.'cfg(unix)'.dependencies]
bindgen = { version = "0.69" }
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
        assert!(rendered.contains("artifact-summary.txt"));
        assert!(rendered.contains("validation-report.txt"));
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
        let artifact_summary = std::fs::read_to_string(bundle_dir.join("artifact-summary.txt"))
            .expect("artifact summary should be written");
        let report = std::fs::read_to_string(bundle_dir.join("validation-report.txt"))
            .expect("validation report should be written");
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
        assert!(release_notes.contains("asteroid:433-Eros"));
        assert!(release_notes.contains("Validation reference points:"));
        assert!(release_notes.contains("Compatibility caveats:"));
        assert!(release_notes.contains("Bundle provenance:"));
        assert!(release_notes.contains("Rust compiler version"));
        assert!(release_notes_summary.contains("Release notes summary"));
        assert!(release_notes_summary.contains("Release-specific coverage:"));
        assert!(release_notes_summary.contains("API stability summary line:"));
        assert!(release_notes_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_notes_summary.contains("Compact summary views: backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(release_notes_summary.contains("Release notes: release-notes"));
        assert!(release_notes_summary
            .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(release_notes_summary
            .contains("Artifact summary: artifact-summary / artifact-posture-summary"));
        assert!(release_notes_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_summary.contains("Release summary"));
        assert!(release_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(release_summary.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(release_summary.contains(
            "Validation report summary: validation-report-summary / validation-summary / report-summary"
        ));
        assert!(release_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(release_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_summary.contains(
            "Packaged-artifact profile: stored channels: [Longitude, Latitude, DistanceAu]"
        ));
        assert!(release_summary.contains("applies to 11 bundled bodies"));
        assert!(release_summary.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
        assert!(release_summary.contains("Release notes summary: release-notes-summary"));
        assert!(artifact_summary.contains("Artifact summary"));
        assert!(artifact_summary
            .contains("Artifact profile: stored channels: [Longitude, Latitude, DistanceAu]"));
        assert!(artifact_summary.contains("applies to 11 bundled bodies"));
        assert!(artifact_summary.contains("Model error envelope"));
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
        assert!(release_checklist.contains("Backend matrix summary: backend-matrix-summary"));
        assert!(release_checklist.contains("API stability summary: api-stability-summary"));
        assert!(release_checklist.contains("release-summary.txt"));
        assert!(release_checklist.contains("release-checklist-summary.txt"));
        assert!(release_checklist.contains("bundle-manifest.checksum.txt"));
        assert!(release_checklist_summary.contains("Release checklist summary"));
        assert!(release_checklist_summary
            .contains("Compatibility profile summary: compatibility-profile-summary"));
        assert!(release_checklist_summary.contains("Artifact validation: validate-artifact"));
        assert!(release_checklist_summary.contains("Workspace audit: workspace-audit / audit"));
        assert!(release_checklist_summary.contains("Repository-managed release gates: 7 items"));
        assert!(release_checklist_summary.contains("Manual bundle workflow: 3 items"));
        assert!(release_checklist_summary.contains("Bundle contents: 16 items"));
        assert!(release_checklist_summary.contains("External publishing reminders: 3 items"));
        assert!(backend_matrix.contains("Implemented backend matrices"));
        assert!(backend_matrix.contains("JPL snapshot reference backend"));
        assert!(backend_matrix_summary.contains("Backend matrix summary"));
        assert!(backend_matrix_summary.contains(&format!(
            "Profile: {}",
            release_profiles.compatibility_profile_id
        )));
        assert!(backend_matrix_summary.contains("Backends: 5"));
        assert!(backend_matrix_summary.contains("Algorithmic: 2"));
        assert!(backend_matrix_summary.contains("Composite: 1"));
        assert!(backend_matrix_summary
            .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(
            backend_matrix_summary.contains("Selected asteroid evidence: 5 exact J2000 samples")
        );
        assert!(backend_matrix_summary
            .contains("Compatibility profile verification: verify-compatibility-profile"));
        assert!(backend_matrix.contains(
            "selected asteroid coverage: 5 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        ));
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
        assert!(validation_report_summary.contains("Expected tolerance status"));
        assert!(validation_report_summary.contains("VSOP87 source-backed evidence"));
        assert!(validation_report_summary.contains("VSOP87 source documentation: 8 source specs, 8 source-backed body profiles, 1 fallback mean-element body profile"));
        assert!(validation_report_summary.contains(
            "source-backed breakdown: 8 generated binary, 0 vendored full-file, 0 truncated slice"
        ));
        assert!(validation_report_summary.contains(
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        ));
        assert!(validation_report_summary
            .contains("VSOP87 source audit: 8 source-backed bodies, 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
        assert!(validation_report_summary
            .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(validation_report_summary.contains("generated binary VSOP87B"));
        assert!(validation_report_summary.contains("VSOP87 source-backed evidence"));
        assert!(validation_report_summary.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), 8 within interim limits"
        ));
        assert!(validation_report_summary
            .contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
        assert!(validation_report_summary.contains(
            "ELP lunar theory specification: Compact Meeus-style truncated lunar baseline [meeus-style-truncated-lunar-baseline; family: Meeus-style truncated analytical baseline]"
        ));
        assert!(validation_report_summary.contains("request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"));
        assert!(validation_report_summary
            .contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
        assert!(validation_report_summary.contains("date-range note: Validated against the published 1992-04-12 geocentric Moon example, J2000 lunar-point anchors, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example"));
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
        assert!(manifest.contains("validation-report-summary.txt"));
        assert!(manifest.contains("artifact-summary.txt"));
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
        assert!(manifest.contains("release checklist checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release checklist summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("backend matrix checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("backend matrix summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("api stability checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("validation report summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("artifact summary checksum (fnv1a-64): 0x"));
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
        assert!(verified.contains("validation-report-summary.txt"));
        assert!(verified.contains("artifact-summary.txt"));
        assert!(verified.contains("source revision:"));
        assert!(verified.contains("workspace status:"));
        assert!(verified.contains("rustc version:"));
        assert!(verified.contains("validation rounds: 1"));
        assert!(verified.contains("release notes checksum: 0x"));
        assert!(verified.contains("release notes summary checksum: 0x"));
        assert!(verified.contains("release checklist checksum: 0x"));
        assert!(verified.contains("release checklist summary checksum: 0x"));
        assert!(verified.contains("backend matrix checksum: 0x"));
        assert!(verified.contains("backend matrix summary checksum: 0x"));
        assert!(verified.contains("validation report summary checksum: 0x"));
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
        assert!(report.contains("roundtrip decode: ok"));
        assert!(report.contains("checksum verified: ok"));
        assert!(report.contains("Bodies"));
        assert!(report.contains("Sun"));
        assert!(report.contains("Moon"));
        assert!(report.contains("Jupiter"));
        assert!(report.contains("Pluto"));
        assert!(report.contains("boundary checks"));
        assert!(report.contains("Model error envelope"));
        assert!(report.contains("Body-class error envelopes"));
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
    fn sidereal_conversion_remains_available_above_the_backend_layer() {
        let longitude = sidereal_longitude(
            Longitude::from_degrees(120.0),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
            &ZodiacMode::Sidereal {
                ayanamsa: Ayanamsa::Lahiri,
            },
        )
        .expect("sidereal conversion should succeed");
        assert!(longitude.degrees().is_finite());
    }
}
