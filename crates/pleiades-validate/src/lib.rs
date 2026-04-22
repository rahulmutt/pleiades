//! Validation, comparison, and benchmarking helpers for the workspace.
//!
//! The validation crate compares the algorithmic chart backends against the
//! checked-in JPL Horizons snapshot corpus and renders reproducible reports for
//! stage-4 work.

#![forbid(unsafe_code)]

use std::fmt;
use std::time::Instant as StdInstant;

mod artifact;

pub use artifact::{render_artifact_report, ArtifactBodyInspection, ArtifactInspectionReport};

use pleiades_core::{
    default_chart_bodies, Apparentness, BackendCapabilities, BackendMetadata, CelestialBody,
    CompositeBackend, CoordinateFrame, EclipticCoordinates, EphemerisBackend, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, EphemerisResult, Instant, JulianDay, Longitude,
    TimeScale, ZodiacMode,
};
use pleiades_elp::ElpBackend;
use pleiades_jpl::{comparison_snapshot, JplSnapshotBackend};
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

/// A full validation report containing comparison and benchmark data.
#[derive(Clone, Debug)]
pub struct ValidationReport {
    /// Comparison corpus summary.
    pub comparison_corpus: CorpusSummary,
    /// Benchmark corpus summary.
    pub benchmark_corpus: CorpusSummary,
    /// Comparison output.
    pub comparison: ComparisonReport,
    /// Archived regression cases preserved from the comparison corpus.
    pub archived_regressions: RegressionArchive,
    /// Benchmark output for the reference backend.
    pub reference_benchmark: BenchmarkReport,
    /// Benchmark output for the candidate backend.
    pub candidate_benchmark: BenchmarkReport,
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation report")?;
        writeln!(f)?;
        writeln!(f, "Comparison corpus")?;
        write_corpus_summary(f, &self.comparison_corpus)?;
        writeln!(f)?;
        writeln!(f, "Benchmark corpus")?;
        write_corpus_summary(f, &self.benchmark_corpus)?;
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
                sample.body.built_in_name().unwrap_or("Custom"),
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
                sample.body.built_in_name().unwrap_or("Custom"),
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
            finding.body.built_in_name().unwrap_or("Custom"),
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
            finding.body.built_in_name().unwrap_or("Custom"),
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

fn format_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(|body| body.built_in_name().unwrap_or("Custom"))
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
        "{banner}\n\nCommands:\n  compare-backends          Compare the JPL snapshot against the algorithmic composite backend\n  benchmark [--rounds N]    Benchmark the candidate backend on the representative 1500-2500 window corpus\n  report [--rounds N]       Render the full validation report\n  validate-artifact         Inspect and validate the bundled compressed artifact\n  help                      Show this help text\n\nDefault benchmark rounds: {DEFAULT_BENCHMARK_ROUNDS}\nDefault comparison corpus size: {corpus_size}",
        banner = banner(),
        corpus_size = corpus_size,
    )
}

fn render_error(error: EphemerisError) -> String {
    error.to_string()
}

fn render_artifact_error(error: crate::artifact::ArtifactInspectionError) -> String {
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
        assert!(report.contains("Comparison corpus"));
        assert!(report.contains("JPL Horizons comparison window"));
        assert!(report.contains("Benchmark corpus"));
        assert!(report.contains("Representative 1500-2500 window"));
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
        assert!(rendered.contains("benchmark [--rounds N]"));
        assert!(rendered.contains("report [--rounds N]"));
        assert!(rendered.contains("validate-artifact"));
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
        assert!(report.contains("boundary checks"));
        assert!(report.contains("Model error envelope"));
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
