//! Comparison sample and summary data types.

use std::fmt;

use pleiades_core::{CelestialBody, EclipticCoordinates, EphemerisError, EphemerisErrorKind};

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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComparisonSummary {
    /// Number of samples compared.
    pub sample_count: usize,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute longitude delta.
    pub max_longitude_delta_deg: f64,
    /// Mean absolute longitude delta.
    pub mean_longitude_delta_deg: f64,
    /// Root-mean-square absolute longitude delta.
    pub rms_longitude_delta_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute latitude delta.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta.
    pub mean_latitude_delta_deg: f64,
    /// Root-mean-square absolute latitude delta.
    pub rms_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Maximum absolute distance delta.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta.
    pub mean_distance_delta_au: Option<f64>,
    /// Root-mean-square absolute distance delta.
    pub rms_distance_delta_au: Option<f64>,
}

impl ComparisonSummary {
    /// Returns the compact release-facing summary line for the aggregate comparison envelope.
    pub fn summary_line(&self) -> String {
        format!(
            "samples: {}, max longitude delta: {:.12}°{}, mean longitude delta: {:.12}°, rms longitude delta: {:.12}°, max latitude delta: {:.12}°{}, mean latitude delta: {:.12}°, rms latitude delta: {:.12}°, max distance delta: {}{}, mean distance delta: {}, rms distance delta: {}",
            self.sample_count,
            self.max_longitude_delta_deg,
            self.max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_longitude_delta_deg,
            self.rms_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_latitude_delta_deg,
            self.rms_latitude_delta_deg,
            self.max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.rms_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
        )
    }

    /// Returns the compact comparison summary line after validating the aggregate envelope.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the aggregate envelope is finite and self-consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.sample_count == 0 {
            if self.max_longitude_delta_body.is_some()
                || self.max_latitude_delta_body.is_some()
                || self.max_distance_delta_body.is_some()
                || self.max_distance_delta_au.is_some()
                || self.mean_distance_delta_au.is_some()
                || self.rms_distance_delta_au.is_some()
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison summary with zero samples must not carry per-body or distance extrema",
                ));
            }

            for (label, value) in [
                ("max_longitude_delta_deg", self.max_longitude_delta_deg),
                ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
                ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
                ("max_latitude_delta_deg", self.max_latitude_delta_deg),
                ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
                ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
            ] {
                if value != 0.0 {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!("comparison summary field `{label}` must be zero when there are no samples"),
                    ));
                }
            }
        } else if self.max_longitude_delta_body.is_none() || self.max_latitude_delta_body.is_none()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison summary with samples must name the bodies that drive the longitude and latitude maxima",
            ));
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison summary field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        match (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_au,
            self.mean_distance_delta_au,
            self.rms_distance_delta_au,
        ) {
            (None, None, None, None) => {}
            (Some(_), Some(max), Some(mean), Some(rms)) => {
                for (label, value) in [
                    ("max_distance_delta_au", max),
                    ("mean_distance_delta_au", mean),
                    ("rms_distance_delta_au", rms),
                ] {
                    if !value.is_finite() || value.is_sign_negative() {
                        return Err(EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            format!("comparison summary field `{label}` must be a finite non-negative value"),
                        ));
                    }
                }
            }
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison summary distance metrics must either all be present or all be absent",
                ));
            }
        }

        Ok(())
    }
}

impl fmt::Display for ComparisonSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Summary statistics for the comparison-audit gate over a comparison run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComparisonAuditSummary {
    /// Number of compared bodies.
    pub body_count: usize,
    /// Number of bodies whose measured deltas stayed within tolerance.
    pub within_tolerance_body_count: usize,
    /// Number of bodies whose measured deltas exceeded tolerance.
    pub outside_tolerance_body_count: usize,
    /// Number of notable regressions in the comparison corpus.
    pub regression_count: usize,
}

impl ComparisonAuditSummary {
    fn status_label(&self) -> &'static str {
        if self.regression_count == 0 {
            "clean"
        } else {
            "regressions found"
        }
    }

    /// Returns the compact release-facing comparison-audit summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "status={}, bodies checked={}, within tolerance bodies={}, outside tolerance bodies={}, notable regressions={}",
            self.status_label(),
            self.body_count,
            self.within_tolerance_body_count,
            self.outside_tolerance_body_count,
            self.regression_count,
        )
    }

    /// Returns the compact comparison-audit summary line after validating the
    /// counted bodies.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the comparison-audit counts before the summary is rendered or reused.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison audit summary must include at least one compared body",
            ));
        }

        let within_and_outside = self
            .within_tolerance_body_count
            .saturating_add(self.outside_tolerance_body_count);

        if within_and_outside != self.body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison audit summary body-count mismatch: expected {}, found within tolerance {} + outside tolerance {} = {}",
                    self.body_count,
                    self.within_tolerance_body_count,
                    self.outside_tolerance_body_count,
                    within_and_outside
                ),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for ComparisonAuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Summary statistics for a single body within a comparison run.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyComparisonSummary {
    /// Body queried for this summary.
    pub body: CelestialBody,
    /// Number of samples compared for this body.
    pub sample_count: usize,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute longitude delta.
    pub max_longitude_delta_deg: f64,
    /// Mean absolute longitude delta.
    pub mean_longitude_delta_deg: f64,
    /// Root-mean-square absolute longitude delta.
    pub rms_longitude_delta_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: Option<CelestialBody>,
    /// Maximum absolute latitude delta.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta.
    pub mean_latitude_delta_deg: f64,
    /// Root-mean-square absolute latitude delta.
    pub rms_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Maximum absolute distance delta.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta.
    pub mean_distance_delta_au: Option<f64>,
    /// Root-mean-square absolute distance delta.
    pub rms_distance_delta_au: Option<f64>,
}

impl BodyComparisonSummary {
    /// Returns the compact release-facing summary line for one body.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, rms Δdist={}",
            self.body,
            self.sample_count,
            self.max_longitude_delta_deg,
            self.max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_longitude_delta_deg,
            self.rms_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_latitude_delta_deg,
            self.rms_latitude_delta_deg,
            self.max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.mean_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.rms_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }

    /// Returns the compact body-specific summary line after validating the envelope.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the body-specific envelope is finite and self-consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body comparison summary for {} field `{label}` must be a finite non-negative value",
                        self.body
                    ),
                ));
            }
        }

        match self.sample_count {
            0 => {
                if self.max_longitude_delta_body.is_some()
                    || self.max_latitude_delta_body.is_some()
                    || self.max_distance_delta_body.is_some()
                    || self.max_distance_delta_au.is_some()
                    || self.mean_distance_delta_au.is_some()
                    || self.rms_distance_delta_au.is_some()
                {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "body comparison summary for {} with zero samples must not carry per-body or distance extrema",
                            self.body
                        ),
                    ));
                }
            }
            _ => {
                if self.max_longitude_delta_body.as_ref() != Some(&self.body)
                    || self.max_latitude_delta_body.as_ref() != Some(&self.body)
                {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "body comparison summary for {} must name that body as the longitude and latitude extrema driver",
                            self.body
                        ),
                    ));
                }

                match (
                    self.max_distance_delta_body.as_ref(),
                    self.max_distance_delta_au,
                    self.mean_distance_delta_au,
                    self.rms_distance_delta_au,
                ) {
                    (None, None, None, None) => {}
                    (Some(body), Some(max), Some(mean), Some(rms)) => {
                        if body != &self.body {
                            return Err(EphemerisError::new(
                                EphemerisErrorKind::InvalidRequest,
                                format!(
                                    "body comparison summary for {} must name that body as the distance extrema driver",
                                    self.body
                                ),
                            ));
                        }

                        for (label, value) in [
                            ("max_distance_delta_au", max),
                            ("mean_distance_delta_au", mean),
                            ("rms_distance_delta_au", rms),
                        ] {
                            if !value.is_finite() || value.is_sign_negative() {
                                return Err(EphemerisError::new(
                                    EphemerisErrorKind::InvalidRequest,
                                    format!(
                                        "body comparison summary for {} field `{label}` must be a finite non-negative value",
                                        self.body
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            format!(
                                "body comparison summary for {} distance metrics must either all be present or all be absent",
                                self.body
                            ),
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for BodyComparisonSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
