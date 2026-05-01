//! Full-chart benchmarking helpers used by validation and CLI reporting.
//!
//! The backend benchmark measures raw request throughput. This module adds a
//! smaller but more user-facing full-chart benchmark that exercises chart
//! assembly end to end, including house calculations for representative chart
//! scenarios.

#![forbid(unsafe_code)]

use core::fmt;
use std::time::Instant as StdInstant;

use pleiades_core::{
    default_chart_bodies, Apparentness, BackendMetadata, ChartEngine, ChartRequest,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, HouseSystem,
};

use crate::{house_validation_report, CorpusSummary};

/// Benchmark summary for a full chart assembly workload.
#[derive(Clone, Debug)]
pub struct ChartBenchmarkReport {
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
    /// Total elapsed time for the chart-assembly path.
    pub elapsed: std::time::Duration,
    /// Estimated heap footprint of the benchmark corpus in bytes.
    pub estimated_corpus_heap_bytes: usize,
}

impl ChartBenchmarkReport {
    /// Returns a compact release-facing summary line for the benchmark.
    pub fn summary_line(&self) -> String {
        format!(
            "backend={}; corpus={}; apparentness={}; rounds={}; samples per round={}; chart ns/request={}; charts per second={:.2} req/s; estimated corpus heap footprint={} bytes",
            self.backend.id,
            self.corpus_name,
            self.apparentness,
            self.rounds,
            self.sample_count,
            format_ns(self.nanoseconds_per_chart()),
            self.charts_per_second(),
            self.estimated_corpus_heap_bytes,
        )
    }

    /// Returns the validated compact summary line for the benchmark.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the average number of nanoseconds per chart for the benchmark.
    pub fn nanoseconds_per_chart(&self) -> f64 {
        let total_requests = (self.rounds * self.sample_count) as f64;
        if total_requests == 0.0 {
            return 0.0;
        }

        self.elapsed.as_secs_f64() * 1_000_000_000.0 / total_requests
    }

    /// Returns the average throughput in charts per second.
    pub fn charts_per_second(&self) -> f64 {
        let total_requests = (self.rounds * self.sample_count) as f64;
        if self.elapsed.is_zero() || total_requests == 0.0 {
            return 0.0;
        }

        total_requests / self.elapsed.as_secs_f64()
    }

    /// Returns the benchmark methodology summary.
    pub fn methodology_summary(&self) -> String {
        format!(
            "{} rounds x {} charts per round on the {} corpus; apparentness {}; chart assembly is measured end to end",
            self.rounds, self.sample_count, self.corpus_name, self.apparentness
        )
    }

    /// Validates the benchmark metadata before the report is returned or formatted.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        self.backend.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("chart benchmark backend metadata is invalid: {error}"),
            )
        })?;

        if self.corpus_name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "chart benchmark corpus name must not be blank",
            ));
        }

        if self.rounds == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "chart benchmark rounds must be greater than zero",
            ));
        }

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "chart benchmark sample count must be greater than zero",
            ));
        }

        if self.estimated_corpus_heap_bytes == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "chart benchmark estimated corpus heap footprint must be greater than zero",
            ));
        }

        Ok(())
    }
}

impl fmt::Display for ChartBenchmarkReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chart benchmark report")?;
        writeln!(f, "Summary: {}", self.summary_line())?;
        writeln!(f, "Backend: {}", self.backend.id)?;
        writeln!(f, "Corpus: {}", self.corpus_name)?;
        writeln!(f, "Apparentness: {}", self.apparentness)?;
        writeln!(f, "Rounds: {}", self.rounds)?;
        writeln!(f, "Samples per round: {}", self.sample_count)?;
        writeln!(f, "Methodology: {}", self.methodology_summary())?;
        writeln!(f, "Chart elapsed: {}", super::format_duration(self.elapsed))?;
        writeln!(
            f,
            "Estimated corpus heap footprint: {} bytes",
            self.estimated_corpus_heap_bytes
        )?;
        writeln!(
            f,
            "Nanoseconds per chart: {}",
            format_ns(self.nanoseconds_per_chart())
        )?;
        writeln!(f, "Charts per second: {:.2}", self.charts_per_second())
    }
}

/// Returns the chart-benchmark corpus summary used by validation reports.
pub fn chart_benchmark_corpus_summary() -> CorpusSummary {
    ChartBenchmarkCorpus::new().summary()
}

/// Benchmarks full chart assembly for a backend using a representative chart corpus.
pub fn benchmark_chart_backend<B: EphemerisBackend>(
    backend: B,
    rounds: usize,
) -> Result<ChartBenchmarkReport, EphemerisError> {
    let corpus = ChartBenchmarkCorpus::new();
    let backend_metadata = backend.metadata();
    let engine = ChartEngine::new(backend);

    let start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(engine.chart(request)?);
        }
    }
    let elapsed = start.elapsed();

    let report = ChartBenchmarkReport {
        backend: backend_metadata,
        corpus_name: corpus.name.clone(),
        apparentness: corpus.apparentness,
        rounds,
        sample_count: corpus.requests.len(),
        elapsed,
        estimated_corpus_heap_bytes: corpus.estimated_heap_bytes(),
    };
    report.validate()?;
    Ok(report)
}

#[derive(Clone, Debug)]
struct ChartBenchmarkCorpus {
    name: String,
    description: &'static str,
    apparentness: Apparentness,
    requests: Vec<ChartRequest>,
}

impl ChartBenchmarkCorpus {
    fn new() -> Self {
        let validation = house_validation_report();
        let requests = validation
            .scenarios
            .iter()
            .map(|scenario| {
                ChartRequest::new(scenario.instant)
                    .with_observer(scenario.observer.clone())
                    .with_bodies(default_chart_bodies().to_vec())
                    .with_apparentness(Apparentness::Mean)
                    .with_house_system(HouseSystem::WholeSign)
            })
            .collect();

        Self {
            name: "Representative chart validation scenarios".to_string(),
            description: "Four chart-assembly scenarios that reuse the house-validation observers with Whole Sign houses to measure end-to-end chart latency.",
            apparentness: Apparentness::Mean,
            requests,
        }
    }

    fn summary(&self) -> CorpusSummary {
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
            for body in &request.bodies {
                if !bodies.contains(body) {
                    bodies.push(body.clone());
                }
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

    fn estimated_heap_bytes(&self) -> usize {
        self.requests
            .capacity()
            .saturating_mul(std::mem::size_of::<ChartRequest>())
            .saturating_add(self.name.capacity())
    }
}

fn format_ns(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.2}")
    } else {
        "n/a".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_candidate_backend;

    #[test]
    fn chart_benchmark_summary_line_mentions_the_backend_and_throughput() {
        let report = benchmark_chart_backend(default_candidate_backend(), 1)
            .expect("chart benchmark should produce a report");
        let summary = report.summary_line();
        assert_eq!(report.validated_summary_line(), Ok(summary.clone()));
        assert!(summary.contains("backend="));
        assert!(summary.contains("corpus="));
        assert!(summary.contains("apparentness="));
        assert!(summary.contains("chart ns/request="));
        assert!(summary.contains("charts per second="));
        assert!(summary.contains("estimated corpus heap footprint="));
    }

    #[test]
    fn chart_benchmark_rejects_zero_heap_footprint() {
        let mut report = benchmark_chart_backend(default_candidate_backend(), 1)
            .expect("chart benchmark should produce a report");
        report.estimated_corpus_heap_bytes = 0;

        let error = report
            .validate()
            .expect_err("zero-heap chart benchmarks should be rejected");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("chart benchmark estimated corpus heap footprint must be greater than zero"));
    }
}
