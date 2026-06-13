//! Validation and benchmark report types and their report rendering.

use std::fmt;

use crate::*;

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
            validated_release_profile_identifiers_summary_for_report(&release_profiles)
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
            match validated_comparison_corpus_release_guard_summary_for_report() {
                Ok(summary) => summary,
                Err(_) => return Err(fmt::Error),
            }
        )?;
        writeln!(f, "  {}", comparison_snapshot_summary_for_report())?;
        writeln!(
            f,
            "  {}",
            comparison_snapshot_body_class_coverage_summary_for_report()
        )?;
        writeln!(f, "  {}", comparison_snapshot_batch_parity_summary_text())?;
        writeln!(f, "  Source corpus: {}", source_corpus_summary_for_report())?;
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
