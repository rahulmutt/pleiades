//! Regression findings, archives, and comparison-audit summaries.

use std::fmt;

use crate::*;

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

impl RegressionFinding {
    /// Returns the compact release-facing summary line for the regression case.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
            self.body,
            self.longitude_delta_deg,
            self.latitude_delta_deg,
            self.distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.note,
        )
    }

    /// Returns the validated compact summary line for the regression case.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for RegressionFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl RegressionFinding {
    /// Returns `Ok(())` when the regression note is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.note.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "regression finding note must not be blank",
            ));
        }

        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "regression finding field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance) = self.distance_delta_au {
            if !distance.is_finite() || distance.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "regression finding distance delta must be a finite non-negative value",
                ));
            }
        }

        Ok(())
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

impl RegressionArchive {
    /// Returns `Ok(())` when the archived regression cases are internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.corpus_name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "regression archive corpus name must not be blank",
            ));
        }

        for (index, case) in self.cases.iter().enumerate() {
            case.validate().map_err(|error| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!("regression archive case #{} is invalid: {error}", index + 1),
                )
            })?;
        }

        Ok(())
    }
}


pub(crate) fn comparison_audit_summary(report: &ComparisonReport) -> ComparisonAuditSummary {
    let tolerance_summaries = report.tolerance_summaries();
    let body_count = tolerance_summaries.len();
    let within_tolerance_body_count = tolerance_summaries
        .iter()
        .filter(|summary| summary.within_tolerance)
        .count();
    let outside_tolerance_body_count = body_count.saturating_sub(within_tolerance_body_count);
    let regression_count = report.notable_regressions().len();

    ComparisonAuditSummary {
        body_count,
        within_tolerance_body_count,
        outside_tolerance_body_count,
        regression_count,
    }
}

pub(crate) fn comparison_audit_summary_for_report(report: &ComparisonReport) -> String {
    let summary = comparison_audit_summary(report);

    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("comparison audit unavailable ({error})"),
    }
}

pub(crate) fn comparison_audit_totals(report: &ComparisonReport) -> (usize, usize, usize, usize) {
    let summary = comparison_audit_summary(report);

    (
        summary.body_count,
        summary.within_tolerance_body_count,
        summary.outside_tolerance_body_count,
        summary.regression_count,
    )
}

pub(crate) fn format_regression_bodies(regressions: &[RegressionFinding]) -> String {
    let mut bodies = regressions
        .iter()
        .map(|finding| finding.body.to_string())
        .collect::<Vec<_>>();
    bodies.sort();
    bodies.dedup();

    if bodies.is_empty() {
        "none".to_string()
    } else {
        bodies.join(", ")
    }
}

pub(crate) fn format_summary_body(body: &Option<CelestialBody>) -> String {
    body.as_ref()
        .map(|body| format!(" ({body})"))
        .unwrap_or_default()
}
