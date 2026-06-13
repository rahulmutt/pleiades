//! The comparison audit report text spine (`render_comparison_audit_report_text`).

use crate::*;

pub(crate) fn render_comparison_audit_report_text(report: &ComparisonReport) -> String {
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
    let body_class_tolerance_posture =
        match validated_comparison_body_class_tolerance_posture_line(report) {
            Ok(line) => line,
            Err(error) => format!("body-class tolerance posture unavailable ({error})"),
        };
    let _ = writeln!(text, "  {}", body_class_tolerance_posture);
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
