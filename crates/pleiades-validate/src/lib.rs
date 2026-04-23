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

pub use artifact::{render_artifact_report, ArtifactBodyInspection, ArtifactInspectionReport};
pub use house_validation::{
    house_validation_report, HouseValidationReport, HouseValidationSample, HouseValidationScenario,
};

use pleiades_core::{
    current_api_stability_profile, current_compatibility_profile,
    current_release_profile_identifiers, default_chart_bodies, AccuracyClass, Apparentness,
    BackendCapabilities, BackendFamily, BackendMetadata, CelestialBody, CompositeBackend,
    CoordinateFrame, EclipticCoordinates, EphemerisBackend, EphemerisError, EphemerisErrorKind,
    EphemerisRequest, EphemerisResult, Instant, JulianDay, Longitude, TimeRange, TimeScale,
    ZodiacMode,
};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_jpl::{comparison_snapshot, reference_asteroids, JplSnapshotBackend};
use pleiades_vsop87::Vsop87Backend;

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
            .map(|request| request.instant.julian_day.days())
            .collect::<Vec<_>>();
        epochs.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
        epochs.dedup_by(|left, right| (*left - *right).abs() <= f64::EPSILON);

        let mut bodies = Vec::new();
        for request in &self.requests {
            if !bodies.contains(&request.body) {
                bodies.push(request.body.clone());
            }
        }

        CorpusSummary {
            name: self.name.clone(),
            description: self.description,
            apparentness: self.apparentness,
            request_count: self.requests.len(),
            epoch_count: epochs.len(),
            body_count: bodies.len(),
            earliest_julian_day: epochs.first().copied().unwrap_or_default(),
            latest_julian_day: epochs.last().copied().unwrap_or_default(),
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
/// release checklist, backend matrix, API posture, API summary, validation report, and manifest.
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
    /// Path to the generated release checklist file.
    pub release_checklist_path: PathBuf,
    /// Path to the generated backend capability matrix file.
    pub backend_matrix_path: PathBuf,
    /// Path to the generated backend capability matrix summary file.
    pub backend_matrix_summary_path: PathBuf,
    /// Path to the generated API stability posture file.
    pub api_stability_path: PathBuf,
    /// Path to the generated API stability summary file.
    pub api_stability_summary_path: PathBuf,
    /// Path to the generated validation report file.
    pub validation_report_path: PathBuf,
    /// Path to the generated bundle manifest.
    pub manifest_path: PathBuf,
    /// Number of bytes written for the compatibility profile.
    pub compatibility_profile_bytes: usize,
    /// Number of bytes written for the compatibility-profile summary.
    pub compatibility_profile_summary_bytes: usize,
    /// Number of bytes written for the release notes.
    pub release_notes_bytes: usize,
    /// Number of bytes written for the release checklist.
    pub release_checklist_bytes: usize,
    /// Number of bytes written for the backend capability matrix.
    pub backend_matrix_bytes: usize,
    /// Number of bytes written for the backend capability matrix summary.
    pub backend_matrix_summary_bytes: usize,
    /// Number of bytes written for the API stability posture.
    pub api_stability_bytes: usize,
    /// Number of bytes written for the API stability summary.
    pub api_stability_summary_bytes: usize,
    /// Number of bytes written for the validation report.
    pub validation_report_bytes: usize,
    /// Deterministic checksum for the compatibility profile contents.
    pub compatibility_profile_checksum: u64,
    /// Deterministic checksum for the compatibility-profile summary contents.
    pub compatibility_profile_summary_checksum: u64,
    /// Deterministic checksum for the release notes contents.
    pub release_notes_checksum: u64,
    /// Deterministic checksum for the release checklist contents.
    pub release_checklist_checksum: u64,
    /// Deterministic checksum for the backend capability matrix contents.
    pub backend_matrix_checksum: u64,
    /// Deterministic checksum for the backend capability matrix summary contents.
    pub backend_matrix_summary_checksum: u64,
    /// Deterministic checksum for the API stability posture contents.
    pub api_stability_checksum: u64,
    /// Deterministic checksum for the API stability summary contents.
    pub api_stability_summary_checksum: u64,
    /// Deterministic checksum for the validation report contents.
    pub validation_report_checksum: u64,
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
            "  release checklist: {}",
            self.release_checklist_path.display()
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
            "  validation report: {}",
            self.validation_report_path.display()
        )?;
        writeln!(f, "  manifest: {}", self.manifest_path.display())?;
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
            "  release checklist bytes: {}",
            self.release_checklist_bytes
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
            "  release checklist checksum: 0x{:016x}",
            self.release_checklist_checksum
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
            "  validation report bytes: {}",
            self.validation_report_bytes
        )?;
        writeln!(
            f,
            "  validation report checksum: 0x{:016x}",
            self.validation_report_checksum
        )
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
        Some("backend-matrix") => {
            ensure_no_extra_args(&args[1..], "backend-matrix")?;
            render_backend_matrix_report().map_err(render_error)
        }
        Some("backend-matrix-summary") => {
            ensure_no_extra_args(&args[1..], "backend-matrix-summary")?;
            Ok(render_backend_matrix_summary())
        }
        Some("benchmark") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_benchmark_report(rounds).map_err(render_error)
        }
        Some("report") | Some("generate-report") => {
            let rounds = parse_rounds(&args[1..], DEFAULT_BENCHMARK_ROUNDS)?;
            render_validation_report(rounds).map_err(render_error)
        }
        Some("validate-artifact") => {
            ensure_no_extra_args(&args[1..], "validate-artifact")?;
            render_artifact_report().map_err(render_artifact_error)
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
        Some("release-notes") => {
            ensure_no_extra_args(&args[1..], "release-notes")?;
            Ok(render_release_notes_text())
        }
        Some("release-checklist") => {
            ensure_no_extra_args(&args[1..], "release-checklist")?;
            Ok(render_release_checklist_text())
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

fn render_compatibility_profile_summary_text() -> String {
    let profile = current_compatibility_profile();
    let release_profiles = current_release_profile_identifiers();
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
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Known gaps: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');

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

    if !profile.custom_definition_labels.is_empty() {
        text.push_str("Custom-definition labels:\n");
        for label in profile.custom_definition_labels {
            text.push_str("- ");
            text.push_str(label);
            text.push('\n');
        }
        text.push('\n');
    }

    if !profile.known_gaps.is_empty() {
        text.push_str("Known gaps:\n");
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
    text.push('\n');
    text.push_str("Repository-managed release gates:\n");
    for item in [
        "[x] mise run fmt",
        "[x] mise run lint",
        "[x] mise run test",
        "[x] mise run audit",
        "[x] mise run release-smoke",
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
        "[x] release-checklist.txt",
        "[x] backend-matrix.txt",
        "[x] backend-matrix-summary.txt",
        "[x] api-stability.txt",
        "[x] api-stability-summary.txt",
        "[x] validation-report.txt",
        "[x] bundle-manifest.txt",
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
/// release checklist, backend matrix, API posture, API summary, validation report, and a manifest.
pub fn render_release_bundle(
    rounds: usize,
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let profile_text = current_compatibility_profile().to_string();
    let profile_summary_text = render_compatibility_profile_summary_text();
    let release_notes_text = render_release_notes_text();
    let release_checklist_text = render_release_checklist_text();
    let backend_matrix_text = render_backend_matrix_report()?;
    let backend_matrix_summary_text = render_backend_matrix_summary();
    let api_stability_text = current_api_stability_profile().to_string();
    let api_stability_summary_text = render_api_stability_summary();
    let validation_report = render_validation_report(rounds)?;
    let provenance = workspace_provenance();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let profile_summary_path = output_dir.join("compatibility-profile-summary.txt");
    let release_notes_path = output_dir.join("release-notes.txt");
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let compatibility_profile_checksum = checksum64(&profile_text);
    let compatibility_profile_summary_checksum = checksum64(&profile_summary_text);
    let release_notes_checksum = checksum64(&release_notes_text);
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let validation_report_checksum = checksum64(&validation_report);
    let manifest_text = format!(
        "Release bundle manifest\nprofile: compatibility-profile.txt\nprofile checksum (fnv1a-64): 0x{compatibility_profile_checksum:016x}\nprofile summary: compatibility-profile-summary.txt\nprofile summary checksum (fnv1a-64): 0x{compatibility_profile_summary_checksum:016x}\nrelease notes: release-notes.txt\nrelease notes checksum (fnv1a-64): 0x{release_notes_checksum:016x}\nrelease checklist: release-checklist.txt\nrelease checklist checksum (fnv1a-64): 0x{release_checklist_checksum:016x}\nbackend matrix: backend-matrix.txt\nbackend matrix checksum (fnv1a-64): 0x{backend_matrix_checksum:016x}\nbackend matrix summary: backend-matrix-summary.txt\nbackend matrix summary checksum (fnv1a-64): 0x{backend_matrix_summary_checksum:016x}\napi stability posture: api-stability.txt\napi stability checksum (fnv1a-64): 0x{api_stability_checksum:016x}\napi stability summary: api-stability-summary.txt\napi stability summary checksum (fnv1a-64): 0x{api_stability_summary_checksum:016x}\nvalidation report: validation-report.txt\nvalidation report checksum (fnv1a-64): 0x{validation_report_checksum:016x}\nsource revision: {}\nworkspace status: {}\nrustc version: {}\nprofile id: {}\napi stability posture id: {}\nvalidation rounds: {}\n",
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
        &backend_matrix_summary_path,
        backend_matrix_summary_text.as_bytes(),
    )?;
    fs::write(&release_checklist_path, release_checklist_text.as_bytes())?;
    fs::write(&backend_matrix_path, backend_matrix_text.as_bytes())?;
    fs::write(&api_stability_path, api_stability_text.as_bytes())?;
    fs::write(
        &api_stability_summary_path,
        api_stability_summary_text.as_bytes(),
    )?;
    fs::write(&report_path, validation_report.as_bytes())?;
    fs::write(&manifest_path, manifest_text.as_bytes())?;

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
    release_checklist_path: String,
    release_checklist_checksum: u64,
    backend_matrix_path: String,
    backend_matrix_checksum: u64,
    backend_matrix_summary_path: String,
    backend_matrix_summary_checksum: u64,
    api_stability_path: String,
    api_stability_checksum: u64,
    api_stability_summary_path: String,
    api_stability_summary_checksum: u64,
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
            release_checklist_path: parse_manifest_string(text, "release checklist:")?,
            release_checklist_checksum: parse_manifest_checksum(
                text,
                "release checklist checksum (fnv1a-64):",
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
            validation_rounds: parse_manifest_usize(text, "validation rounds:")?,
        })
    }
}

fn ensure_release_bundle_directory_contents(output_dir: &Path) -> Result<(), ReleaseBundleError> {
    let expected_entries: BTreeSet<String> = [
        "compatibility-profile.txt",
        "compatibility-profile-summary.txt",
        "release-notes.txt",
        "release-checklist.txt",
        "backend-matrix.txt",
        "backend-matrix-summary.txt",
        "api-stability.txt",
        "api-stability-summary.txt",
        "validation-report.txt",
        "bundle-manifest.txt",
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

fn verify_release_bundle(
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let profile_summary_path = output_dir.join("compatibility-profile-summary.txt");
    let release_notes_path = output_dir.join("release-notes.txt");
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let validation_report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");

    let profile_text = fs::read_to_string(&profile_path)?;
    let profile_summary_text = fs::read_to_string(&profile_summary_path)?;
    let release_notes_text = fs::read_to_string(&release_notes_path)?;
    let release_checklist_text = fs::read_to_string(&release_checklist_path)?;
    let backend_matrix_text = fs::read_to_string(&backend_matrix_path)?;
    let backend_matrix_summary_text = fs::read_to_string(&backend_matrix_summary_path)?;
    let api_stability_text = fs::read_to_string(&api_stability_path)?;
    let api_stability_summary_text = fs::read_to_string(&api_stability_summary_path)?;
    let validation_report_text = fs::read_to_string(&validation_report_path)?;
    let manifest_text = fs::read_to_string(&manifest_path)?;

    let manifest = ParsedReleaseBundleManifest::parse(&manifest_text)?;
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
    if manifest.release_checklist_path != "release-checklist.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release checklist file entry: {}",
            manifest.release_checklist_path
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
    if manifest.validation_report_path != "validation-report.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected validation report file entry: {}",
            manifest.validation_report_path
        )));
    }

    let compatibility_profile_checksum = checksum64(&profile_text);
    let compatibility_profile_summary_checksum = checksum64(&profile_summary_text);
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let validation_report_checksum = checksum64(&validation_report_text);
    let profile_id = extract_prefixed_value(&profile_text, "Compatibility profile: ")?;
    let api_stability_posture_id =
        extract_prefixed_value(&api_stability_text, "API stability posture: ")?;

    if manifest.release_checklist_checksum != release_checklist_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release checklist checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_checklist_checksum, release_checklist_checksum
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
    if manifest.validation_report_checksum != validation_report_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "validation report checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.validation_report_checksum, validation_report_checksum
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
        release_checklist_path,
        backend_matrix_path,
        backend_matrix_summary_path,
        api_stability_path,
        api_stability_summary_path,
        validation_report_path,
        manifest_path,
        compatibility_profile_bytes: profile_text.len(),
        compatibility_profile_summary_bytes: profile_summary_text.len(),
        release_notes_bytes: release_notes_text.len(),
        release_checklist_bytes: release_checklist_text.len(),
        backend_matrix_bytes: backend_matrix_text.len(),
        backend_matrix_summary_bytes: backend_matrix_summary_text.len(),
        api_stability_bytes: api_stability_text.len(),
        api_stability_summary_bytes: api_stability_summary_text.len(),
        validation_report_bytes: validation_report_text.len(),
        compatibility_profile_checksum,
        compatibility_profile_summary_checksum,
        release_notes_checksum,
        release_checklist_checksum,
        backend_matrix_checksum,
        backend_matrix_summary_checksum,
        api_stability_checksum,
        api_stability_summary_checksum,
        validation_report_checksum,
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

fn parse_manifest_usize(text: &str, prefix: &str) -> Result<usize, ReleaseBundleError> {
    let value = extract_prefixed_value(text, prefix)?;
    value.parse::<usize>().map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {prefix} value: {error}"))
    })
}

fn parse_manifest_checksum(text: &str, prefix: &str) -> Result<u64, ReleaseBundleError> {
    let value = extract_prefixed_value(text, prefix)?;
    let value = value.strip_prefix("0x").ok_or_else(|| {
        ReleaseBundleError::Verification(format!("missing 0x prefix for {prefix}"))
    })?;
    u64::from_str_radix(value, 16).map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {prefix} value: {error}"))
    })
}

fn extract_prefixed_value<'a>(text: &'a str, prefix: &str) -> Result<&'a str, ReleaseBundleError> {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix).map(str::trim))
        .ok_or_else(|| {
            ReleaseBundleError::Verification(format!("missing manifest entry: {prefix}"))
        })
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

/// Renders the workspace audit used by the CLI and release smoke checks.
pub fn workspace_audit_report() -> Result<WorkspaceAuditReport, std::io::Error> {
    let workspace_root = fs::canonicalize(workspace_root())?;
    let manifest_paths = workspace_manifest_paths(&workspace_root)?;
    let lockfile_path = workspace_root.join("Cargo.lock");
    let mut violations = Vec::new();

    for path in &manifest_paths {
        let text = fs::read_to_string(path)?;
        violations.extend(audit_manifest_text(path, &text));
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

/// Renders the validation report used by the CLI.
pub fn render_validation_report(rounds: usize) -> Result<String, EphemerisError> {
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
    }
    .to_string())
}

/// Renders the comparison report used by the CLI.
pub fn render_comparison_report() -> Result<String, EphemerisError> {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    Ok(compare_backends(&reference, &candidate, &corpus)?.to_string())
}

/// Renders a benchmark report used by the CLI.
pub fn render_benchmark_report(rounds: usize) -> Result<String, EphemerisError> {
    let corpus = benchmark_corpus();
    let candidate = default_candidate_backend();
    Ok(benchmark_backend(&candidate, &corpus, rounds)?.to_string())
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

    for entry in &catalog {
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
    text.push_str("Backends with external data sources: ");
    text.push_str(&data_source_count.to_string());
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

    text
}

/// Renders a compact summary of the API stability posture.
pub fn render_api_stability_summary() -> String {
    render_api_stability_summary_text()
}

fn render_api_stability_summary_text() -> String {
    let profile = current_api_stability_profile();
    let mut text = String::new();

    text.push_str("API stability summary\n");
    text.push_str("Profile: ");
    text.push_str(profile.profile_id);
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

fn write_corpus_summary(f: &mut fmt::Formatter<'_>, corpus: &CorpusSummary) -> fmt::Result {
    writeln!(f, "  name: {}", corpus.name)?;
    writeln!(f, "  description: {}", corpus.description)?;
    writeln!(f, "  Apparentness: {}", corpus.apparentness)?;
    writeln!(f, "  requests: {}", corpus.request_count)?;
    writeln!(f, "  epochs: {}", corpus.epoch_count)?;
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
            expected_error_kinds: JPL_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
        BackendMatrixEntry {
            label: "VSOP87 planetary backend",
            metadata: Vsop87Backend::new().metadata(),
            expected_error_kinds: VSOP87_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "ELP lunar backend (Moon and lunar nodes)",
            metadata: ElpBackend::new().metadata(),
            expected_error_kinds: ELP_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Packaged data backend",
            metadata: PackagedDataBackend::new().metadata(),
            expected_error_kinds: PACKAGED_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Composite routed backend",
            metadata: default_candidate_backend().metadata(),
            expected_error_kinds: COMPOSITE_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
    ]
}

struct BackendMatrixEntry {
    label: &'static str,
    metadata: BackendMetadata,
    expected_error_kinds: &'static [EphemerisErrorKind],
    required_data_files: &'static [&'static str],
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
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(rounds)
}

fn help_text() -> String {
    let corpus_size = default_corpus().requests.len();
    format!(
        "{banner}\n\nCommands:\n  compare-backends          Compare the JPL snapshot against the algorithmic composite backend\n  backend-matrix            Print the implemented backend capability matrices\n  backend-matrix-summary    Print the compact backend capability matrix summary\n  benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus with guard epochs\n  report [--rounds N]       Render the full validation report\n  generate-report           Alias for report\n  validate-artifact         Inspect and validate the bundled compressed artifact\n  workspace-audit           Check the workspace for mandatory native build hooks\n  audit                     Alias for workspace-audit\n  api-stability             Print the release API stability posture\n  api-posture               Alias for api-stability\n  api-stability-summary     Print the compact API stability summary\n  api-posture-summary       Alias for api-stability-summary\n  compatibility-profile-summary  Print the compact compatibility profile summary\n  profile-summary           Alias for compatibility-profile-summary\n  release-notes             Print the release compatibility notes\n  release-checklist         Print the release maintainer checklist\n  bundle-release --out DIR  Write the release compatibility profile, profile summary, release notes, release checklist, backend matrix, backend matrix summary, API posture, API summary, validation report, and manifest\n  verify-release-bundle     Read a staged release bundle back and verify its manifest checksums\n  help                      Show this help text\n\nDefault benchmark rounds: {DEFAULT_BENCHMARK_ROUNDS}\nDefault comparison corpus size: {corpus_size}",
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
        CoordinateFrame, JulianDay, TimeScale, ZodiacMode,
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

    #[test]
    fn default_corpus_covers_the_comparison_snapshot() {
        let corpus = default_corpus();
        let summary = corpus.summary();
        assert_eq!(corpus.requests.len(), 20);
        assert_eq!(summary.epoch_count, 3);
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
        assert!(report.contains("Ceres"));
        assert!(report.contains("Pallas"));
        assert!(report.contains("Juno"));
        assert!(report.contains("Vesta"));
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
        assert!(report.contains("Southern hemisphere reference chart"));
        assert!(report.contains("Reference backend"));
        assert!(report.contains("Candidate backend"));
        assert!(report.contains("Comparison summary"));
        assert!(report.contains("Notable regressions"));
        assert!(report.contains("Archived regression cases"));
        assert!(report.contains("Reference benchmark"));
        assert!(report.contains("Candidate benchmark"));
        assert!(report.contains("Packaged-data benchmark corpus"));
        assert!(report.contains("Packaged-data benchmark"));
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
        assert_eq!(summary.body_count, default_chart_bodies().len());
        assert!(summary.request_count > 0);
        assert!(summary.earliest_julian_day <= summary.latest_julian_day);
    }

    #[test]
    fn house_validation_report_includes_southern_hemisphere_scenario() {
        let report = house_validation_report();
        assert_eq!(report.scenarios.len(), 3);
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
            .any(|finding| finding.body == CelestialBody::Mars));
        assert!(regressions
            .iter()
            .any(|finding| finding.body == CelestialBody::Neptune));
        assert!(regressions
            .iter()
            .any(|finding| finding.body == CelestialBody::Pluto));

        let archive = report.regression_archive();
        assert_eq!(archive.corpus_name, corpus.name);
        assert_eq!(archive.cases.len(), regressions.len());
        assert!(archive
            .cases
            .iter()
            .any(|finding| finding.body == CelestialBody::Mars));
        assert!(report.to_string().contains("Notable regressions"));
    }

    #[test]
    fn cli_help_lists_the_validation_commands() {
        let rendered = render_cli(&["help"]).expect("help should render");
        assert!(rendered.contains("compare-backends"));
        assert!(rendered.contains("backend-matrix"));
        assert!(rendered.contains("backend-matrix-summary"));
        assert!(rendered.contains("benchmark [--rounds N]"));
        assert!(rendered.contains("report [--rounds N]"));
        assert!(rendered.contains("generate-report"));
        assert!(rendered.contains("validate-artifact"));
        assert!(rendered.contains("workspace-audit"));
        assert!(rendered.contains("api-stability"));
        assert!(rendered.contains("api-stability-summary"));
        assert!(rendered.contains("compatibility-profile-summary"));
        assert!(rendered.contains("release-notes"));
        assert!(rendered.contains("release-checklist"));
        assert!(rendered.contains("bundle-release --out DIR"));
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
        assert!(rendered.contains("Stable surfaces:"));
        assert!(rendered.contains("Experimental surfaces:"));
        assert!(rendered.contains("Deprecation policy items:"));
        assert!(rendered.contains("Intentional limits:"));
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
        assert!(rendered.contains("House systems:"));
        assert!(rendered.contains("Ayanamsas:"));
        assert!(rendered.contains("Custom-definition labels:"));
        assert!(rendered.contains("Known gaps:"));
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
        assert!(rendered.contains("API stability posture:"));
        assert!(rendered.contains("Deprecation policy:"));
        assert!(rendered.contains("Release-specific coverage:"));
        assert!(rendered.contains("Known gaps:"));
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
        assert!(rendered.contains("Repository-managed release gates:"));
        assert!(rendered.contains("Manual bundle workflow:"));
        assert!(rendered.contains("Bundle contents:"));
        assert!(rendered.contains("backend-matrix-summary.txt"));
        assert!(rendered.contains("api-stability-summary.txt"));
    }

    #[test]
    fn backend_matrix_command_renders_the_implemented_catalog() {
        let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
        assert!(rendered.contains("Implemented backend matrices"));
        assert!(rendered.contains("JPL snapshot reference backend"));
        assert!(
            rendered.contains("selected asteroid coverage: 4 bodies (Ceres, Pallas, Juno, Vesta)")
        );
        assert!(rendered.contains("nominal range:"));
        assert!(rendered.contains("provenance sources:"));
        assert!(rendered.contains("expected error classes:"));
        assert!(rendered.contains("required external data files:"));
        assert!(rendered.contains("crates/pleiades-jpl/data/reference_snapshot.csv"));
        assert!(
            rendered.contains("Paul Schlyter-style mean orbital elements for the Sun and planets")
        );
        assert!(rendered.contains("Meeus-style truncated lunar orbit formulas"));
        assert!(rendered.contains("NASA/JPL Horizons API vector tables (DE441)"));
        assert!(rendered.contains("VSOP87 planetary backend"));
        assert!(rendered.contains("ELP lunar backend (Moon and lunar nodes)"));
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
        assert!(rendered.contains("Accuracy classes:"));
        assert!(rendered.contains("Exact: 1"));
        assert!(rendered.contains("Approximate: 4"));
        assert!(rendered.contains("Distinct bodies covered:"));
        assert!(rendered.contains("Distinct coordinate frames:"));
        assert!(rendered.contains("Distinct time scales:"));
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
        assert!(rendered.contains("release-checklist.txt"));
        assert!(rendered.contains("backend-matrix.txt"));
        assert!(rendered.contains("API stability posture:"));
        assert!(rendered.contains("api-stability.txt"));
        assert!(rendered.contains("validation-report.txt"));
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
        let release_checklist = std::fs::read_to_string(bundle_dir.join("release-checklist.txt"))
            .expect("release checklist should be written");
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
        let report = std::fs::read_to_string(bundle_dir.join("validation-report.txt"))
            .expect("validation report should be written");
        let manifest = std::fs::read_to_string(bundle_dir.join("bundle-manifest.txt"))
            .expect("manifest should be written");

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
        assert!(release_notes.contains("API stability posture:"));
        assert!(release_notes.contains("Deprecation policy:"));
        assert!(release_notes.contains("Release-specific coverage:"));
        assert!(release_notes.contains("Known gaps:"));
        assert!(release_notes.contains("Bundle provenance:"));
        assert!(release_notes.contains("Rust compiler version"));
        assert!(release_checklist.contains("Release checklist"));
        assert!(release_checklist.contains("Manual bundle workflow:"));
        assert!(release_checklist.contains("bundle-release --out /tmp/pleiades-release"));
        assert!(release_checklist.contains("verify-release-bundle --out /tmp/pleiades-release"));
        assert!(release_checklist.contains("docs/release-reproducibility.md"));
        assert!(release_checklist.contains("Bundle contents:"));
        assert!(release_checklist.contains("compatibility-profile-summary.txt"));
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
        assert!(backend_matrix
            .contains("selected asteroid coverage: 4 bodies (Ceres, Pallas, Juno, Vesta)"));
        assert!(api_stability.contains(&format!(
            "API stability posture: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(api_stability_summary.contains("API stability summary"));
        assert!(api_stability_summary.contains(&format!(
            "Profile: {}",
            release_profiles.api_stability_profile_id
        )));
        assert!(report.contains("Validation report"));
        assert!(manifest.contains("Release bundle manifest"));
        assert!(manifest.contains("validation rounds: 1"));
        assert!(manifest.contains("compatibility-profile.txt"));
        assert!(manifest.contains("compatibility-profile-summary.txt"));
        assert!(manifest.contains("release-notes.txt"));
        assert!(manifest.contains("backend-matrix.txt"));
        assert!(manifest.contains("backend-matrix-summary.txt"));
        assert!(manifest.contains("api-stability.txt"));
        assert!(manifest.contains("api-stability-summary.txt"));
        assert!(manifest.contains("validation-report.txt"));
        assert!(manifest.contains("source revision:"));
        assert!(manifest.contains("workspace status:"));
        assert!(manifest.contains("rustc version:"));
        assert!(manifest.contains("profile checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("profile summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release notes checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("release checklist checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("backend matrix checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("backend matrix summary checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("api stability checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("validation report checksum (fnv1a-64): 0x"));

        let verified = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect("bundle verification should render");
        assert!(verified.contains("Release bundle"));
        assert!(verified.contains("bundle-manifest.txt"));
        assert!(verified.contains("compatibility-profile-summary.txt"));
        assert!(verified.contains("source revision:"));
        assert!(verified.contains("workspace status:"));
        assert!(verified.contains("rustc version:"));
        assert!(verified.contains("validation rounds: 1"));
        assert!(verified.contains("release notes checksum: 0x"));
        assert!(verified.contains("release checklist checksum: 0x"));
        assert!(verified.contains("backend matrix checksum: 0x"));
        assert!(verified.contains("backend matrix summary checksum: 0x"));
        assert!(verified.contains("validation report checksum: 0x"));

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
    fn verify_release_bundle_rejects_blank_profile_id_entry() {
        assert_release_bundle_rejects_blank_manifest_value(
            "pleiades-release-bundle-blank-profile-id",
            "profile id:",
            &["missing profile id entry"],
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
