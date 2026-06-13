//! Comparison report, error envelopes, body-class tolerance, and sample validation.

use std::fmt;
use std::sync::OnceLock;

use crate::*;

/// Renders the comparison report used by the CLI.
pub fn render_comparison_report() -> Result<String, EphemerisError> {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    Ok(compare_backends(&reference, &candidate, &corpus)?.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonMedianEnvelope {
    pub(crate) longitude_delta_deg: f64,
    pub(crate) latitude_delta_deg: f64,
    pub(crate) distance_delta_au: Option<f64>,
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
    pub(crate) longitude_delta_deg: f64,
    pub(crate) latitude_delta_deg: f64,
    pub(crate) distance_delta_au: Option<f64>,
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
    pub(crate) summary: ComparisonSummary,
    pub(crate) median: ComparisonMedianEnvelope,
    pub(crate) percentile: ComparisonPercentileEnvelope,
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

pub(crate) fn median_value(values: &mut [f64]) -> Option<f64> {
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

pub(crate) fn percentile_value(values: &mut [f64], percentile: f64) -> Option<f64> {
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

pub(crate) fn comparison_median_envelope_for_samples(
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

pub(crate) fn comparison_percentile_envelope(
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

pub(crate) fn format_comparison_percentile_envelope_for_report(
    samples: &[ComparisonSample],
) -> String {
    match comparison_tail_envelope(samples) {
        Ok(envelope) => envelope.summary_line(),
        Err(error) => format!("comparison percentile envelope unavailable ({error})"),
    }
}

pub(crate) fn format_comparison_envelope_for_report(
    summary: &ComparisonSummary,
    samples: &[ComparisonSample],
) -> String {
    let envelope = comparison_envelope_summary(summary, samples);
    match envelope.validated_summary_line(samples) {
        Ok(rendered) => rendered,
        Err(error) => format!("comparison envelope unavailable ({error})"),
    }
}

pub(crate) fn format_body_class_comparison_envelope_for_report(
    summary: &BodyClassSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body-class error envelope unavailable ({error})"),
    }
}

pub(crate) fn comparison_body_class_error_envelope_summaries_for_report(
) -> Result<Vec<BodyClassSummary>, String> {
    let report = comparison_report_for_default_render()?;
    let summaries = report.body_class_summaries();

    if summaries.is_empty() {
        return Err("comparison report did not produce any body-class error envelopes".to_string());
    }

    for summary in &summaries {
        summary.validate().map_err(|error| error.to_string())?;
    }

    Ok(summaries)
}

pub(crate) fn comparison_body_class_error_envelope_summary_for_report() -> String {
    match comparison_body_class_error_envelope_summaries_for_report() {
        Ok(summaries) => format!("{} classes checked", summaries.len()),
        Err(error) => format!("body-class error envelopes unavailable ({error})"),
    }
}

pub(crate) fn render_comparison_body_class_error_envelope_summary_text_from_summaries(
    summaries: Result<Vec<BodyClassSummary>, String>,
) -> String {
    use std::fmt::Write as _;

    let summaries = match summaries {
        Ok(summaries) => summaries,
        Err(error) => {
            return format!(
                "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable ({error})\n"
            );
        }
    };

    if summaries.is_empty() {
        return "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable (comparison report did not produce any body-class error envelopes)\n".to_string();
    }

    for summary in &summaries {
        if let Err(error) = summary.validate() {
            return format!(
                "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable ({error})\n"
            );
        }
    }

    let mut text = String::from("Comparison body-class error envelope summary\n");
    let _ = writeln!(text, "Body-class error envelopes: {}", summaries.len());
    for summary in summaries {
        let _ = writeln!(
            text,
            "  {}: {}",
            summary.class.label(),
            summary.summary_line()
        );
    }
    text
}

pub(crate) fn render_comparison_body_class_error_envelope_summary_text() -> String {
    render_comparison_body_class_error_envelope_summary_text_from_summaries(
        comparison_body_class_error_envelope_summaries_for_report(),
    )
}

pub(crate) fn format_body_class_tolerance_envelope_for_report(
    summary: &BodyClassToleranceSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("body-class tolerance envelope unavailable ({error})"),
    }
}

pub(crate) fn comparison_report_for_default_render() -> Result<ComparisonReport, String> {
    compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &default_corpus(),
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn validated_comparison_body_class_tolerance_posture_line(
    report: &ComparisonReport,
) -> Result<String, String> {
    use std::fmt::Write as _;

    let summaries = report.body_class_tolerance_summaries();
    if summaries.is_empty() {
        return Err(
            "comparison report did not produce any body-class tolerance summaries".to_string(),
        );
    }

    let outlier_class_count = summaries
        .iter()
        .filter(|summary| summary.outside_tolerance_body_count > 0)
        .count();
    let outlier_bodies = summaries
        .iter()
        .flat_map(|summary| summary.outside_bodies.iter().cloned())
        .collect::<Vec<_>>();

    let mut text = String::new();
    let _ = write!(
        text,
        "body-class tolerance posture: {} classes checked, {} classes with outlier bodies, outlier bodies: {}",
        summaries.len(),
        outlier_class_count,
        if outlier_bodies.is_empty() {
            "none".to_string()
        } else {
            format_bodies(&outlier_bodies)
        }
    );
    Ok(text)
}

pub(crate) fn validated_comparison_body_class_tolerance_posture_for_report(
) -> Result<String, String> {
    let report = comparison_report_for_default_render()?;
    validated_comparison_body_class_tolerance_posture_line(&report)
}

pub(crate) fn format_body_class_tolerance_posture_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();

    SUMMARY
        .get_or_init(|| {
            validated_comparison_body_class_tolerance_posture_for_report().unwrap_or_else(|error| {
                format!("body-class tolerance posture unavailable ({error})")
            })
        })
        .clone()
}

pub(crate) fn validate_comparison_sample_distance_channels(
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

pub(crate) fn validate_comparison_samples_for_report(
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

pub(crate) fn comparison_tolerance_policy_summary_details(
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

pub(crate) fn validated_comparison_tolerance_policy_summary_for_report(
    comparison: &ComparisonReport,
) -> Result<ComparisonTolerancePolicySummary, String> {
    let summary = comparison_tolerance_policy_summary_details(comparison);
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

pub(crate) fn format_comparison_tolerance_policy_for_report(
    comparison: &ComparisonReport,
) -> String {
    let summary = comparison_tolerance_policy_summary_details(comparison);
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("comparison tolerance policy unavailable ({error})"),
    }
}

pub(crate) fn format_comparison_tolerance_limits_for_report(
    entries: &[ComparisonToleranceEntry],
) -> String {
    entries
        .iter()
        .map(format_comparison_tolerance_limit_for_report)
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn format_comparison_tolerance_limit_for_report(
    entry: &ComparisonToleranceEntry,
) -> String {
    match entry.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("{} unavailable ({error})", entry.scope.label()),
    }
}

pub(crate) fn comparison_coordinate_frames(comparison: &ComparisonReport) -> &[CoordinateFrame] {
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

/// Renders the compact release-grade comparison-audit summary used by the CLI.
pub fn render_comparison_audit_summary() -> Result<String, String> {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let comparison =
        compare_backends(&reference, &candidate, &corpus).map_err(|error| error.to_string())?;

    Ok(comparison_audit_summary_for_report(&comparison))
}

pub(crate) fn comparison_audit_result_label(regression_count: usize) -> &'static str {
    if regression_count == 0 {
        "clean"
    } else {
        "regressions found"
    }
}
