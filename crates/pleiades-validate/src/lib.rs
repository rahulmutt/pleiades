//! Validation, comparison, and benchmarking helpers for the workspace.
//!
//! The validation crate compares the algorithmic chart backends against the
//! checked-in JPL Horizons snapshot corpus and renders reproducible reports for
//! stage-4 work.

#![forbid(unsafe_code)]

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant as StdInstant;

mod artifact;
mod house_validation;

pub use artifact::{render_artifact_report, ArtifactBodyInspection, ArtifactInspectionReport};
pub use house_validation::{
    house_validation_report, HouseValidationReport, HouseValidationSample, HouseValidationScenario,
};

use pleiades_core::{
    current_api_stability_profile, current_compatibility_profile, default_chart_bodies,
    Apparentness, BackendCapabilities, BackendMetadata, CelestialBody, CompositeBackend,
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
            requests,
        }
    }

    /// Creates a representative benchmark corpus spanning the target 1500-2500 window.
    pub fn representative_window() -> Self {
        let bodies = default_chart_bodies();
        let instants = [
            Instant::new(JulianDay::from_days(2_268_924.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tt),
        ];

        Self::from_epochs(
            "Representative 1500-2500 window",
            "Three-epoch benchmark corpus that exercises the algorithmic backend across the compression target range.",
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
                    apparent: Apparentness::Mean,
                })
            })
            .collect();

        Self {
            name: name.into(),
            description,
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
}

/// A generated release bundle containing the compatibility profile, backend matrix,
/// API posture, validation report, and manifest.
#[derive(Clone, Debug)]
pub struct ReleaseBundle {
    /// Output directory chosen by the caller.
    pub output_dir: PathBuf,
    /// Path to the generated compatibility profile file.
    pub compatibility_profile_path: PathBuf,
    /// Path to the generated backend capability matrix file.
    pub backend_matrix_path: PathBuf,
    /// Path to the generated API stability posture file.
    pub api_stability_path: PathBuf,
    /// Path to the generated validation report file.
    pub validation_report_path: PathBuf,
    /// Path to the generated bundle manifest.
    pub manifest_path: PathBuf,
    /// Number of bytes written for the compatibility profile.
    pub compatibility_profile_bytes: usize,
    /// Number of bytes written for the backend capability matrix.
    pub backend_matrix_bytes: usize,
    /// Number of bytes written for the API stability posture.
    pub api_stability_bytes: usize,
    /// Number of bytes written for the validation report.
    pub validation_report_bytes: usize,
    /// Deterministic checksum for the compatibility profile contents.
    pub compatibility_profile_checksum: u64,
    /// Deterministic checksum for the backend capability matrix contents.
    pub backend_matrix_checksum: u64,
    /// Deterministic checksum for the API stability posture contents.
    pub api_stability_checksum: u64,
    /// Deterministic checksum for the validation report contents.
    pub validation_report_checksum: u64,
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

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation report")?;
        writeln!(f)?;
        writeln!(f, "Compatibility profile")?;
        writeln!(f, "{}", current_compatibility_profile())?;
        writeln!(f)?;
        writeln!(f, "API stability posture")?;
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
            "  backend matrix: {}",
            self.backend_matrix_path.display()
        )?;
        writeln!(
            f,
            "  API stability posture: {}",
            self.api_stability_path.display()
        )?;
        writeln!(
            f,
            "  validation report: {}",
            self.validation_report_path.display()
        )?;
        writeln!(f, "  manifest: {}", self.manifest_path.display())?;
        writeln!(
            f,
            "  compatibility profile bytes: {}",
            self.compatibility_profile_bytes
        )?;
        writeln!(
            f,
            "  compatibility profile checksum: 0x{:016x}",
            self.compatibility_profile_checksum
        )?;
        writeln!(f, "  backend matrix bytes: {}", self.backend_matrix_bytes)?;
        writeln!(
            f,
            "  backend matrix checksum: 0x{:016x}",
            self.backend_matrix_checksum
        )?;
        writeln!(
            f,
            "  API stability posture bytes: {}",
            self.api_stability_bytes
        )?;
        writeln!(
            f,
            "  API stability posture checksum: 0x{:016x}",
            self.api_stability_checksum
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
        Some("api-stability") | Some("api-posture") => {
            ensure_no_extra_args(&args[1..], "api-stability")?;
            Ok(current_api_stability_profile().to_string())
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

/// Writes a release bundle containing the compatibility profile, backend matrix,
/// API posture, validation report, and a manifest.
pub fn render_release_bundle(
    rounds: usize,
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let profile_text = current_compatibility_profile().to_string();
    let backend_matrix_text = render_backend_matrix_report()?;
    let api_stability_text = current_api_stability_profile().to_string();
    let validation_report = render_validation_report(rounds)?;
    let profile_path = output_dir.join("compatibility-profile.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let compatibility_profile_checksum = checksum64(&profile_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let validation_report_checksum = checksum64(&validation_report);
    let manifest_text = format!(
        "Release bundle manifest\nprofile: compatibility-profile.txt\nprofile checksum (fnv1a-64): 0x{compatibility_profile_checksum:016x}\nbackend matrix: backend-matrix.txt\nbackend matrix checksum (fnv1a-64): 0x{backend_matrix_checksum:016x}\napi stability posture: api-stability.txt\napi stability checksum (fnv1a-64): 0x{api_stability_checksum:016x}\nvalidation report: validation-report.txt\nvalidation report checksum (fnv1a-64): 0x{validation_report_checksum:016x}\nprofile id: {}\napi stability posture id: {}\nvalidation rounds: {}\n",
        current_compatibility_profile().profile_id,
        current_api_stability_profile().profile_id,
        rounds,
    );

    fs::write(&profile_path, profile_text.as_bytes())?;
    fs::write(&backend_matrix_path, backend_matrix_text.as_bytes())?;
    fs::write(&api_stability_path, api_stability_text.as_bytes())?;
    fs::write(&report_path, validation_report.as_bytes())?;
    fs::write(&manifest_path, manifest_text.as_bytes())?;

    verify_release_bundle(output_dir)
}

#[derive(Debug)]
struct ParsedReleaseBundleManifest {
    profile_path: String,
    profile_checksum: u64,
    backend_matrix_path: String,
    backend_matrix_checksum: u64,
    api_stability_path: String,
    api_stability_checksum: u64,
    validation_report_path: String,
    validation_report_checksum: u64,
    profile_id: String,
    api_stability_posture_id: String,
    _validation_rounds: usize,
}

impl ParsedReleaseBundleManifest {
    fn parse(text: &str) -> Result<Self, ReleaseBundleError> {
        Ok(Self {
            profile_path: parse_manifest_string(text, "profile:")?,
            profile_checksum: parse_manifest_checksum(text, "profile checksum (fnv1a-64):")?,
            backend_matrix_path: parse_manifest_string(text, "backend matrix:")?,
            backend_matrix_checksum: parse_manifest_checksum(
                text,
                "backend matrix checksum (fnv1a-64):",
            )?,
            api_stability_path: parse_manifest_string(text, "api stability posture:")?,
            api_stability_checksum: parse_manifest_checksum(
                text,
                "api stability checksum (fnv1a-64):",
            )?,
            validation_report_path: parse_manifest_string(text, "validation report:")?,
            validation_report_checksum: parse_manifest_checksum(
                text,
                "validation report checksum (fnv1a-64):",
            )?,
            profile_id: parse_manifest_string(text, "profile id:")?,
            api_stability_posture_id: parse_manifest_string(text, "api stability posture id:")?,
            _validation_rounds: parse_manifest_usize(text, "validation rounds:")?,
        })
    }
}

fn verify_release_bundle(
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let validation_report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");

    let profile_text = fs::read_to_string(&profile_path)?;
    let backend_matrix_text = fs::read_to_string(&backend_matrix_path)?;
    let api_stability_text = fs::read_to_string(&api_stability_path)?;
    let validation_report_text = fs::read_to_string(&validation_report_path)?;
    let manifest_text = fs::read_to_string(&manifest_path)?;

    let manifest = ParsedReleaseBundleManifest::parse(&manifest_text)?;
    if manifest.profile_path != "compatibility-profile.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected profile file entry: {}",
            manifest.profile_path
        )));
    }
    if manifest.backend_matrix_path != "backend-matrix.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected backend matrix file entry: {}",
            manifest.backend_matrix_path
        )));
    }
    if manifest.api_stability_path != "api-stability.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected API stability file entry: {}",
            manifest.api_stability_path
        )));
    }
    if manifest.validation_report_path != "validation-report.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected validation report file entry: {}",
            manifest.validation_report_path
        )));
    }

    let compatibility_profile_checksum = checksum64(&profile_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let validation_report_checksum = checksum64(&validation_report_text);
    let profile_id = extract_prefixed_value(&profile_text, "Compatibility profile: ")?;
    let api_stability_posture_id =
        extract_prefixed_value(&api_stability_text, "API stability posture: ")?;

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
    if manifest.backend_matrix_checksum != backend_matrix_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "backend matrix checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.backend_matrix_checksum, backend_matrix_checksum
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
    if manifest.validation_report_checksum != validation_report_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "validation report checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.validation_report_checksum, validation_report_checksum
        )));
    }

    Ok(ReleaseBundle {
        output_dir: output_dir.to_path_buf(),
        compatibility_profile_path: profile_path,
        backend_matrix_path,
        api_stability_path,
        validation_report_path,
        manifest_path,
        compatibility_profile_bytes: profile_text.len(),
        backend_matrix_bytes: backend_matrix_text.len(),
        api_stability_bytes: api_stability_text.len(),
        validation_report_bytes: validation_report_text.len(),
        compatibility_profile_checksum,
        backend_matrix_checksum,
        api_stability_checksum,
        validation_report_checksum,
    })
}

fn parse_manifest_string(text: &str, prefix: &str) -> Result<String, ReleaseBundleError> {
    extract_prefixed_value(text, prefix).map(|value| value.to_string())
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

/// Renders the validation report used by the CLI.
pub fn render_validation_report(rounds: usize) -> Result<String, EphemerisError> {
    let comparison_corpus = default_corpus();
    let benchmark_corpus = benchmark_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let comparison = compare_backends(&reference, &candidate, &comparison_corpus)?;
    let reference_benchmark = benchmark_backend(&reference, &comparison_corpus, rounds)?;
    let candidate_benchmark = benchmark_backend(&candidate, &benchmark_corpus, rounds)?;
    let archived_regressions = comparison.regression_archive();

    Ok(ValidationReport {
        comparison_corpus: comparison_corpus.summary(),
        benchmark_corpus: benchmark_corpus.summary(),
        house_validation: house_validation_report(),
        comparison,
        archived_regressions,
        reference_benchmark,
        candidate_benchmark,
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
            format_args!("{}\n\n", BackendMatrixDisplay(&entry.metadata)),
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

struct BackendMatrixDisplay<'a>(&'a BackendMetadata);

impl fmt::Display for BackendMatrixDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_backend_matrix(f, self.0)
    }
}

impl fmt::Display for ComparisonReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Comparison report")?;
        writeln!(f, "Corpus: {}", self.corpus_name)?;
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
    writeln!(f, "  frames: {}", format_frames(&backend.supported_frames))?;
    writeln!(
        f,
        "  capabilities: {}",
        format_capabilities(&backend.capabilities)
    )?;
    writeln!(f, "  provenance: {}", backend.provenance.summary)
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

fn implemented_backend_catalog() -> Vec<BackendMatrixEntry> {
    vec![
        BackendMatrixEntry {
            label: "JPL snapshot reference backend",
            metadata: default_reference_backend().metadata(),
        },
        BackendMatrixEntry {
            label: "VSOP87 planetary backend",
            metadata: Vsop87Backend::new().metadata(),
        },
        BackendMatrixEntry {
            label: "ELP lunar backend",
            metadata: ElpBackend::new().metadata(),
        },
        BackendMatrixEntry {
            label: "Packaged data backend",
            metadata: PackagedDataBackend::new().metadata(),
        },
        BackendMatrixEntry {
            label: "Composite routed backend",
            metadata: default_candidate_backend().metadata(),
        },
    ]
}

struct BackendMatrixEntry {
    label: &'static str,
    metadata: BackendMetadata,
}

fn write_backend_catalog(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    catalog: &[BackendMatrixEntry],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for entry in catalog {
        writeln!(f, "{}", entry.label)?;
        write_backend_matrix(f, &entry.metadata)?;
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
        "{banner}\n\nCommands:\n  compare-backends          Compare the JPL snapshot against the algorithmic composite backend\n  backend-matrix            Print the implemented backend capability matrices\n  benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus\n  report [--rounds N]       Render the full validation report\n  generate-report           Alias for report\n  validate-artifact         Inspect and validate the bundled compressed artifact\n  api-stability             Print the release API stability posture\n  api-posture               Alias for api-stability\n  bundle-release --out DIR  Write the release compatibility profile, API posture, validation report, and manifest\n  verify-release-bundle     Read a staged release bundle back and verify its manifest checksums\n  help                      Show this help text\n\nDefault benchmark rounds: {DEFAULT_BENCHMARK_ROUNDS}\nDefault comparison corpus size: {corpus_size}",
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
    use super::*;
    use pleiades_core::{
        sidereal_longitude, Apparentness, Ayanamsa, CoordinateFrame, JulianDay, TimeScale,
        ZodiacMode,
    };
    use pleiades_jpl::comparison_bodies;

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
        assert_eq!(corpus.requests[0].apparent, Apparentness::Mean);
    }

    #[test]
    fn comparison_report_uses_the_snapshot_backend() {
        let report = render_comparison_report().expect("comparison should render");
        assert!(report.contains("JPL Horizons comparison window"));
        assert!(report.contains("Reference backend:"));
        assert!(report.contains("Candidate backend:"));
    }

    #[test]
    fn benchmark_report_renders_a_time_summary() {
        let report = render_benchmark_report(10).expect("benchmark should render");
        assert!(report.contains("Benchmark report"));
        assert!(report.contains("Representative 1500-2500 window"));
        assert!(report.contains("Nanoseconds per request:"));
    }

    #[test]
    fn validation_report_includes_corpus_metadata() {
        let report = render_validation_report(10).expect("validation report should render");
        assert!(report.contains("Validation report"));
        assert!(report.contains("Compatibility profile"));
        assert!(report.contains("API stability posture"));
        assert!(report.contains("Implemented backend matrices"));
        assert!(report.contains("Selected asteroid coverage"));
        assert!(report.contains("Ceres"));
        assert!(report.contains("Pallas"));
        assert!(report.contains("Juno"));
        assert!(report.contains("Vesta"));
        assert!(report.contains("JPL snapshot reference backend"));
        assert!(report.contains("VSOP87 planetary backend"));
        assert!(report.contains("ELP lunar backend"));
        assert!(report.contains("Packaged data backend"));
        assert!(report.contains("Composite routed backend"));
        assert!(report.contains("Target compatibility catalog:"));
        assert!(report.contains("Comparison corpus"));
        assert!(report.contains("JPL Horizons comparison window"));
        assert!(report.contains("Benchmark corpus"));
        assert!(report.contains("Representative 1500-2500 window"));
        assert!(report.contains("House validation corpus"));
        assert!(report.contains("Mid-latitude reference chart"));
        assert!(report.contains("Polar stress chart"));
        assert!(report.contains("Reference backend"));
        assert!(report.contains("Candidate backend"));
        assert!(report.contains("Comparison summary"));
        assert!(report.contains("Notable regressions"));
        assert!(report.contains("Archived regression cases"));
        assert!(report.contains("Reference benchmark"));
        assert!(report.contains("Candidate benchmark"));
    }

    #[test]
    fn benchmark_corpus_spans_the_target_window() {
        let corpus = benchmark_corpus();
        let summary = corpus.summary();
        assert_eq!(summary.epoch_count, 3);
        assert_eq!(summary.body_count, default_chart_bodies().len());
        assert_eq!(summary.request_count, 30);
        assert!(summary.earliest_julian_day < summary.latest_julian_day);
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
        assert!(rendered.contains("benchmark [--rounds N]"));
        assert!(rendered.contains("report [--rounds N]"));
        assert!(rendered.contains("generate-report"));
        assert!(rendered.contains("validate-artifact"));
        assert!(rendered.contains("api-stability"));
        assert!(rendered.contains("bundle-release --out DIR"));
        assert!(rendered.contains("verify-release-bundle"));
    }

    #[test]
    fn api_stability_command_renders_the_posture() {
        let rendered = render_cli(&["api-stability"]).expect("api posture should render");
        assert!(rendered.contains("API stability posture: pleiades-api-stability/0.1.0"));
        assert!(rendered.contains("Stable consumer surfaces:"));
        assert!(rendered.contains("Experimental or operational surfaces:"));
        assert!(rendered.contains("Deprecation policy:"));
    }

    #[test]
    fn backend_matrix_command_renders_the_implemented_catalog() {
        let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
        assert!(rendered.contains("Implemented backend matrices"));
        assert!(rendered.contains("JPL snapshot reference backend"));
        assert!(rendered.contains("nominal range:"));
        assert!(rendered.contains("VSOP87 planetary backend"));
        assert!(rendered.contains("ELP lunar backend"));
        assert!(rendered.contains("Packaged data backend"));
        assert!(rendered.contains("Composite routed backend"));
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
        assert!(rendered.contains("backend-matrix.txt"));
        assert!(rendered.contains("API stability posture:"));
        assert!(rendered.contains("api-stability.txt"));
        assert!(rendered.contains("validation-report.txt"));
        assert!(rendered.contains("checksum: 0x"));

        let profile = std::fs::read_to_string(bundle_dir.join("compatibility-profile.txt"))
            .expect("compatibility profile should be written");
        let backend_matrix = std::fs::read_to_string(bundle_dir.join("backend-matrix.txt"))
            .expect("backend matrix should be written");
        let api_stability = std::fs::read_to_string(bundle_dir.join("api-stability.txt"))
            .expect("API stability posture should be written");
        let report = std::fs::read_to_string(bundle_dir.join("validation-report.txt"))
            .expect("validation report should be written");
        let manifest = std::fs::read_to_string(bundle_dir.join("bundle-manifest.txt"))
            .expect("manifest should be written");

        assert!(profile.contains("Compatibility profile: pleiades-compatibility-profile/0.6.11"));
        assert!(backend_matrix.contains("Implemented backend matrices"));
        assert!(backend_matrix.contains("JPL snapshot reference backend"));
        assert!(api_stability.contains("API stability posture: pleiades-api-stability/0.1.0"));
        assert!(report.contains("Validation report"));
        assert!(manifest.contains("Release bundle manifest"));
        assert!(manifest.contains("validation rounds: 1"));
        assert!(manifest.contains("compatibility-profile.txt"));
        assert!(manifest.contains("backend-matrix.txt"));
        assert!(manifest.contains("api-stability.txt"));
        assert!(manifest.contains("validation-report.txt"));
        assert!(manifest.contains("profile checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("backend matrix checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("api stability checksum (fnv1a-64): 0x"));
        assert!(manifest.contains("validation report checksum (fnv1a-64): 0x"));

        let verified = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
            .expect("bundle verification should render");
        assert!(verified.contains("Release bundle"));
        assert!(verified.contains("bundle-manifest.txt"));
        assert!(verified.contains("backend matrix checksum: 0x"));
        assert!(verified.contains("validation report checksum: 0x"));

        let _ = std::fs::remove_dir_all(&bundle_dir);
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
