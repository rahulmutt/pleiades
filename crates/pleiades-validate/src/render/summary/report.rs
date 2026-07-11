//! The validation report summary spine (`render_validation_report_summary_text`).

use crate::*;

pub(crate) fn render_validation_report_summary_text(report: &ValidationReport) -> String {
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
        validated_release_profile_identifiers_summary_for_report(&release_profiles)
    );
    let _ = writeln!(text, "Time-scale policy: {}", request_policy.time_scale);
    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );
    let utc_convenience_policy =
        crate::posture::backend_policy::validated_utc_convenience_policy_summary_for_report();
    let _ = writeln!(text, "UTC convenience policy: {}", utc_convenience_policy);
    let _ = writeln!(text, "Observer policy: {}", request_policy.observer);
    let _ = writeln!(text, "Apparentness policy: {}", request_policy.apparentness);
    let native_sidereal_policy =
        crate::posture::backend_policy::validated_native_sidereal_policy_summary_for_report();
    let _ = writeln!(text, "Native sidereal policy: {}", native_sidereal_policy);
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
        validated_zodiac_policy_summary_for_report()
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
    let _ = writeln!(text, "  {}", format_comparison_snapshot_manifest_summary());
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => return format!("Comparison corpus summary unavailable ({error})"),
    };
    let _ = writeln!(text, "  release-grade guard: {release_grade_guard}");
    let _ = writeln!(
        text,
        "  Source corpus: {}",
        source_corpus_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Source corpus posture: {}",
        source_corpus_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Reference snapshot");
    let _ = writeln!(text, "  {}", reference_snapshot_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451911_major_body_boundary_summary_for_report()
    );
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
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_dense_boundary_summary_for_report()
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
    let _ = writeln!(
        text,
        "  {}",
        render_reference_holdout_overlap_summary_text()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_bridge_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451916_major_body_interior_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451918_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451919_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451920_major_body_interior_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_mars_jupiter_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_window_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        independent_holdout_snapshot_batch_parity_summary_text()
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
        crate::posture::elp::catalog::lunar_theory_capability_summary_for_report()
    );
    let _ = writeln!(
        text,
        "ELP lunar request policy: {}",
        crate::posture::elp::lib_summaries::lunar_theory_request_policy_summary()
    );
    let _ = writeln!(
        text,
        "ELP frame treatment: {}",
        format_lunar_frame_treatment_summary()
    );
    let _ = writeln!(
        text,
        "ELP lunar theory limitations: {}",
        crate::posture::elp::catalog::lunar_theory_limitations_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::lib_summaries::lunar_theory_source_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar reference");
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_reference_evidence_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_reference_evidence_envelope_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar equatorial reference");
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_equatorial_reference_evidence_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_equatorial_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_equatorial_reference_evidence_envelope_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar apparent comparison");
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_apparent_comparison_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar source windows");
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_source_window_summary_for_report()
    );
    let _ = writeln!(text, "Lunar high-curvature continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_high_curvature_continuity_evidence_for_report()
    );
    let _ = writeln!(text, "Lunar high-curvature equatorial continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::evidence::lunar_high_curvature_equatorial_continuity_evidence_for_report()
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
    let _ = writeln!(text, "{}", format_ayanamsa_catalog_validation_for_report());
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
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::vsop87::audit::generated_binary_audit_summary_for_report()
    );
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
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::catalog::lunar_theory_catalog_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        validated_lunar_theory_catalog_validation_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::lib_summaries::lunar_theory_source_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        crate::posture::elp::lib_summaries::lunar_theory_summary_for_report()
    );
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
        "  Packaged-artifact speed policy: {}",
        format_packaged_artifact_speed_policy_summary()
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
        format_packaged_artifact_generation_policy_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact normalized intermediates: {}",
        packaged_artifact_normalized_intermediate_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation residual bodies: {}",
        match validated_packaged_artifact_generation_residual_bodies_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => return format!("Validation report summary unavailable ({error})"),
        }
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target thresholds: {}",
        validated_packaged_artifact_target_threshold_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target-threshold state: {}",
        validated_packaged_artifact_target_threshold_state_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit envelope: {}",
        packaged_artifact_fit_envelope_summary_for_report()
    );
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
    let _ = writeln!(
        text,
        "  Packaged-artifact fit margins: {}",
        fit_margin_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit threshold violation count: {}",
        fit_threshold_violation_count_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit threshold violations: {}",
        fit_threshold_violation_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit sample classes: {}",
        packaged_artifact_fit_sample_classes_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit outliers: {}",
        packaged_artifact_fit_outlier_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target-threshold scope envelopes: {}",
        validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact source-fit and hold-out sync: {}",
        validated_packaged_artifact_source_fit_holdout_sync_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact phase-2 corpus alignment: {}",
        validated_packaged_artifact_phase2_corpus_alignment_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation manifest: {}",
        packaged_artifact_generation_manifest_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact size: {} bytes",
        report.artifact_decode_benchmark.encoded_bytes
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
