//! Comparison tolerance policy text and the `BodyToleranceSummary` type.

use std::fmt;

use crate::*;

pub(crate) fn comparison_tolerance_policy_coverage(
    comparison: &ComparisonReport,
) -> Vec<ComparisonToleranceScopeCoverageSummary> {
    let entries = comparison_tolerance_policy_entries(&comparison.candidate_backend.family);
    let tolerance_summaries = comparison.tolerance_summaries();

    entries
        .into_iter()
        .map(|entry| {
            let mut bodies = Vec::new();
            let mut sample_count = 0;

            for summary in &tolerance_summaries {
                if comparison_tolerance_scope_for_body(&summary.body) == entry.scope {
                    bodies.push(summary.body.clone());
                    sample_count += summary.sample_count;
                }
            }

            ComparisonToleranceScopeCoverageSummary {
                entry,
                body_count: bodies.len(),
                bodies,
                sample_count,
            }
        })
        .collect()
}

pub(crate) fn write_tolerance_policy(
    f: &mut fmt::Formatter<'_>,
    comparison: &ComparisonReport,
) -> fmt::Result {
    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let summary = match validated_comparison_tolerance_policy_summary_for_report(comparison) {
        Ok(summary) => summary,
        Err(error) => {
            writeln!(f, "Tolerance policy catalog")?;
            writeln!(f, "  unavailable ({error})")?;
            return Ok(());
        }
    };
    let coordinate_frames = format_frames(&summary.coordinate_frames);
    writeln!(f, "Tolerance policy catalog")?;
    writeln!(f, "  candidate backend family: {}", family_label)?;
    writeln!(
        f,
        "  comparison evidence: {} bodies, {} samples",
        summary.comparison_body_count, summary.comparison_sample_count
    )?;
    writeln!(
        f,
        "  comparison window: {}",
        summary.comparison_window.summary_line()
    )?;
    writeln!(f, "  coordinate frames: {}", coordinate_frames)?;
    for scope_coverage in summary.coverage {
        writeln!(f, "  {}", scope_coverage.summary_line())?;
    }
    Ok(())
}

pub(crate) fn write_tolerance_policy_text(text: &mut String, comparison: &ComparisonReport) {
    use std::fmt::Write as _;

    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let summary = match validated_comparison_tolerance_policy_summary_for_report(comparison) {
        Ok(summary) => summary,
        Err(error) => {
            let _ = writeln!(text, "Tolerance policy catalog");
            let _ = writeln!(text, "  unavailable ({error})");
            return;
        }
    };
    let coordinate_frames = format_frames(&summary.coordinate_frames);
    let _ = writeln!(text, "Tolerance policy catalog");
    let _ = writeln!(text, "  candidate backend family: {}", family_label);
    let _ = writeln!(
        text,
        "  comparison evidence: {} bodies, {} samples",
        summary.comparison_body_count, summary.comparison_sample_count
    );
    let _ = writeln!(
        text,
        "  comparison window: {}",
        summary.comparison_window.summary_line()
    );
    let _ = writeln!(text, "  coordinate frames: {}", coordinate_frames);
    for scope_coverage in summary.coverage {
        let _ = writeln!(text, "  {}", scope_coverage.summary_line());
    }
}

/// Per-body comparison status against the expected tolerance table.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyToleranceSummary {
    /// Body queried for this tolerance summary.
    pub body: CelestialBody,
    /// Expected tolerance for the body.
    pub tolerance: ComparisonTolerance,
    /// Number of samples compared for this body.
    pub sample_count: usize,
    /// Whether all measured deltas are within the expected tolerance.
    pub within_tolerance: bool,
    /// Maximum absolute longitude delta measured for this body.
    pub max_longitude_delta_deg: f64,
    /// Signed margin between the longitude limit and measured maximum.
    pub longitude_margin_deg: f64,
    /// Maximum absolute latitude delta measured for this body.
    pub max_latitude_delta_deg: f64,
    /// Signed margin between the latitude limit and measured maximum.
    pub latitude_margin_deg: f64,
    /// Maximum absolute distance delta measured for this body.
    pub max_distance_delta_au: Option<f64>,
    /// Signed margin between the distance limit and measured maximum.
    pub distance_margin_au: Option<f64>,
}

impl BodyToleranceSummary {
    /// Returns `Ok(())` when the tolerance status is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        validate_comparison_tolerance(&self.tolerance)?;

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} has no samples to compare",
                    self.body
                ),
            ));
        }

        for (label, value) in [
            ("longitude", self.max_longitude_delta_deg),
            ("latitude", self.max_latitude_delta_deg),
            ("longitude margin", self.longitude_margin_deg),
            ("latitude margin", self.latitude_margin_deg),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid {} {}",
                        self.body, label, value
                    ),
                ));
            }
        }

        if let Some(value) = self.max_distance_delta_au {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance delta {}",
                        self.body, value
                    ),
                ));
            }
        }

        if let Some(value) = self.distance_margin_au {
            if !value.is_finite() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance margin {}",
                        self.body, value
                    ),
                ));
            }
        }

        let tolerance = &self.tolerance;
        let distance_margin = self.distance_margin_au;
        let has_distance_limit = tolerance.max_distance_delta_au.is_some();
        let has_distance_measurement = self.max_distance_delta_au.is_some();
        if distance_margin.is_some() != (has_distance_limit && has_distance_measurement) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} distance-margin presence does not match the measured values and tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_longitude_margin =
            tolerance.max_longitude_delta_deg - self.max_longitude_delta_deg;
        if self.longitude_margin_deg != expected_longitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} longitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_latitude_margin =
            tolerance.max_latitude_delta_deg - self.max_latitude_delta_deg;
        if self.latitude_margin_deg != expected_latitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} latitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        if let (Some(measured), Some(limit), Some(margin)) = (
            self.max_distance_delta_au,
            tolerance.max_distance_delta_au,
            self.distance_margin_au,
        ) {
            if margin != limit - measured {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} distance margin drifted from the declared tolerance limit",
                        self.body
                    ),
                ));
            }
        }

        let within_tolerance = self.longitude_margin_deg >= 0.0
            && self.latitude_margin_deg >= 0.0
            && self
                .distance_margin_au
                .map(|value| value >= 0.0)
                .unwrap_or(true);
        if self.within_tolerance != within_tolerance {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} status disagrees with the measured margins",
                    self.body
                ),
            ));
        }

        Ok(())
    }

    /// Renders the compact report wording after validating the summary fields.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Renders the compact report wording for this tolerance status.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: backend family={}, profile={}, samples={}, status={}, limit Δlon≤{:.6}°, margin Δlon={:+.12}°, limit Δlat≤{:.6}°, margin Δlat={:+.12}°, limit Δdist={}, margin Δdist={}",
            self.body,
            tolerance_backend_family_label(&self.tolerance.backend_family),
            self.tolerance.profile,
            self.sample_count,
            if self.within_tolerance { "within" } else { "exceeded" },
            self.tolerance.max_longitude_delta_deg,
            self.longitude_margin_deg,
            self.tolerance.max_latitude_delta_deg,
            self.latitude_margin_deg,
            self.tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.distance_margin_au
                .map(|value| format!("{value:+.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }
}

impl fmt::Display for BodyToleranceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
