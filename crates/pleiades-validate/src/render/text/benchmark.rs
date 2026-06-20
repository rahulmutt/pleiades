//! Benchmark report and benchmark matrix summary rendering.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::*;

/// Renders a benchmark report used by the CLI.
pub fn render_benchmark_report(rounds: usize) -> Result<String, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("benchmark report cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = render_benchmark_report_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

pub(crate) fn render_benchmark_report_uncached(rounds: usize) -> Result<String, EphemerisError> {
    let corpus = benchmark_timing_corpus();
    let candidate = default_candidate_backend();
    let backend_report = benchmark_backend(&candidate, &corpus, rounds)?;
    let artifact_lookup_report =
        artifact::benchmark_packaged_artifact_lookup(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let artifact_decode_report =
        artifact::benchmark_packaged_artifact_decode(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let chart_report = benchmark_chart_backend(default_candidate_backend(), rounds)?;
    Ok(format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}",
        benchmark_provenance_text(),
        backend_report,
        artifact_lookup_report,
        artifact_decode_report,
        chart_report
    ))
}

/// Renders a compact benchmark matrix summary used by the CLI.
pub fn render_benchmark_matrix_summary(rounds: usize) -> Result<String, EphemerisError> {
    let report = build_validation_report(rounds)?;
    Ok(render_benchmark_matrix_summary_text(&report))
}

pub(crate) fn report_summary_payload(summary: String, prefix: &str) -> String {
    summary
        .strip_prefix(prefix)
        .unwrap_or(summary.as_str())
        .to_string()
}

/// Returns a non-gating latency budget summary string comparing measured artifact
/// latency against the published `PACKAGED_BUDGETS` targets.
///
/// Non-gating: always returns a string, never panics or asserts on slow timing.
/// The opt-in enforcement gate lives in the `tests` module under `#[ignore]`.
pub fn render_packaged_artifact_latency_budget_summary() -> String {
    const ROUNDS: usize = 16;

    let budgets = &pleiades_data::thresholds::PACKAGED_BUDGETS;

    // --- decode ---
    let decode_line = match artifact::benchmark_packaged_artifact_decode(ROUNDS) {
        Ok(report) => {
            let measured_ms = report.elapsed.as_secs_f64() * 1_000.0 / report.rounds as f64;
            let target_ms = budgets.decode_latency_target_ms;
            let margin_ms = target_ms - measured_ms;
            format!(
                "decode {measured_ms:.1}ms / target {target_ms:.1}ms (margin {margin_ms:+.1}ms)"
            )
        }
        Err(e) => format!("decode error: {e}"),
    };

    // --- single lookup ---
    let lookup_line = match artifact::benchmark_packaged_artifact_lookup(ROUNDS) {
        Ok(report) => {
            let per_lookup_ms = report.elapsed.as_secs_f64() * 1_000.0
                / (report.rounds as f64 * report.sample_count as f64);
            let target_ms = budgets.single_lookup_target_ms;
            let margin_ms = target_ms - per_lookup_ms;
            format!(
                "single-lookup {per_lookup_ms:.1}ms / target {target_ms:.1}ms (margin {margin_ms:+.1}ms)"
            )
        }
        Err(e) => format!("single-lookup error: {e}"),
    };

    // --- batch throughput ---
    let batch_line = match artifact::benchmark_packaged_artifact_batch_lookup(ROUNDS) {
        Ok(report) => {
            let per_lookup_s =
                report.elapsed.as_secs_f64() / (report.rounds as f64 * report.batch_size as f64);
            let throughput_per_s = if per_lookup_s > 0.0 {
                1.0 / per_lookup_s
            } else {
                0.0
            };
            let target_per_s = budgets.batch_throughput_target_per_s;
            let margin = throughput_per_s - target_per_s;
            format!(
                "batch {throughput_per_s:.0}/s / target {target_per_s:.0}/s (margin {margin:+.0}/s)"
            )
        }
        Err(e) => format!("batch error: {e}"),
    };

    format!(
        "Packaged-artifact latency budget summary\n  {decode_line}\n  {lookup_line}\n  {batch_line}"
    )
}

pub(crate) fn render_benchmark_matrix_summary_text(report: &ValidationReport) -> String {
    use std::fmt::Write as _;

    let mut text = String::from("Benchmark matrix summary\n");
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text, "Benchmark corpora");
    let _ = writeln!(
        text,
        "  comparison corpus: {}",
        report.comparison_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  benchmark corpus: {}",
        report.benchmark_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-data benchmark corpus: {}",
        report.packaged_benchmark_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  chart benchmark corpus: {}",
        report.chart_benchmark_corpus.summary_line()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark rows");
    let _ = writeln!(
        text,
        "  reference benchmark: {}",
        report.reference_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  candidate benchmark: {}",
        report.candidate_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-data benchmark: {}",
        report.packaged_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  chart benchmark: {}",
        report.chart_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  artifact decode benchmark: {}",
        report.artifact_decode_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-artifact size: {} bytes",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let fit_envelope_summary = packaged_artifact_fit_envelope_summary_for_report();
    let fit_sample_classes_summary = packaged_artifact_fit_sample_classes_summary_for_report();
    let fit_outlier_summary = packaged_artifact_fit_outlier_summary_for_report();
    let fit_thresholds_summary = packaged_artifact_fit_threshold_summary_for_report();
    let target_threshold_summary =
        validated_packaged_artifact_target_threshold_summary_for_report();
    let target_threshold_state_summary =
        validated_packaged_artifact_target_threshold_state_for_report();
    let target_threshold_scope_envelopes_summary =
        validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report();
    let fit_margin_summary = report_summary_payload(
        packaged_artifact_fit_margin_summary_for_report(),
        "fit margins: ",
    );
    let fit_threshold_violation_count_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_count_for_report(),
        "fit threshold violations: ",
    );
    let fit_threshold_violation_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_summary_for_report(),
        "fit threshold violations: ",
    );
    let fit_envelope = fit_envelope_summary
        .strip_prefix("fit envelope: ")
        .unwrap_or(&fit_envelope_summary);
    let fit_sample_classes = fit_sample_classes_summary
        .strip_prefix("fit sample classes: ")
        .unwrap_or(&fit_sample_classes_summary);
    let fit_outliers = fit_outlier_summary
        .strip_prefix("fit outliers: ")
        .unwrap_or(&fit_outlier_summary);
    let fit_thresholds = fit_thresholds_summary
        .strip_prefix("fit thresholds: ")
        .unwrap_or(&fit_thresholds_summary);
    let target_threshold_scope_envelopes = target_threshold_scope_envelopes_summary
        .strip_prefix("scope envelopes: ")
        .unwrap_or(&target_threshold_scope_envelopes_summary);

    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-artifact fit posture");
    let _ = writeln!(text, "  fit envelope: {}", fit_envelope);
    let _ = writeln!(text, "  fit margins: {}", fit_margin_summary);
    let _ = writeln!(
        text,
        "  fit threshold violation count: {}",
        fit_threshold_violation_count_summary
    );
    let _ = writeln!(
        text,
        "  fit threshold violations: {}",
        fit_threshold_violation_summary
    );
    let _ = writeln!(text, "  fit sample classes: {}", fit_sample_classes);
    let _ = writeln!(text, "  fit outliers: {}", fit_outliers);
    let _ = writeln!(text, "  fit thresholds: {}", fit_thresholds);
    let _ = writeln!(text, "  target thresholds: {}", target_threshold_summary);
    let _ = writeln!(
        text,
        "  target-threshold state: {}",
        target_threshold_state_summary
    );
    let _ = writeln!(
        text,
        "  target-threshold scope envelopes: {}",
        target_threshold_scope_envelopes
    );
    text
}
