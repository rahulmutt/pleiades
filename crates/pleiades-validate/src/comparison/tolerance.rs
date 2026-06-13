//! Comparison tolerance catalog data types.

use std::fmt;

use pleiades_core::{
    BackendFamily, CelestialBody, CoordinateFrame, EphemerisError, EphemerisErrorKind, TimeRange,
};

use crate::{
    backend_family_label, format_bodies, format_comparison_tolerance_limits_for_report,
    format_frames, has_surrounding_whitespace, tolerance_backend_family_label,
};

/// Expected comparison tolerance for a body or body class.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonTolerance {
    /// Backend family this tolerance profile is currently scoped to.
    pub backend_family: BackendFamily,
    /// Human-readable tolerance profile label.
    pub profile: &'static str,
    /// Maximum accepted absolute longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Maximum accepted absolute latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Maximum accepted absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
}

fn validate_comparison_tolerance_profile(profile: &str) -> Result<(), EphemerisError> {
    if profile.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison tolerance profile must not be blank",
        ));
    }

    if has_surrounding_whitespace(profile) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "comparison tolerance profile '{}' contains surrounding whitespace",
                profile
            ),
        ));
    }

    Ok(())
}

pub(crate) fn validate_comparison_tolerance(
    tolerance: &ComparisonTolerance,
) -> Result<(), EphemerisError> {
    validate_comparison_tolerance_profile(tolerance.profile)?;

    for (label, value) in [
        ("longitude", tolerance.max_longitude_delta_deg),
        ("latitude", tolerance.max_latitude_delta_deg),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance '{}' has invalid {} limit {}",
                    tolerance.profile, label, value
                ),
            ));
        }
    }

    if let Some(value) = tolerance.max_distance_delta_au {
        if !value.is_finite() || value < 0.0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance '{}' has invalid distance limit {}",
                    tolerance.profile, value
                ),
            ));
        }
    }

    Ok(())
}

/// Expected tolerance scope used by the validation catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ComparisonToleranceScope {
    /// Luminary body-class scope.
    Luminary,
    /// Major-planet body-class scope.
    MajorPlanet,
    /// Lunar-point body-class scope.
    LunarPoint,
    /// Asteroid body-class scope.
    Asteroid,
    /// Custom-body body-class scope.
    Custom,
    /// Pluto-specific approximate fallback scope.
    Pluto,
}

impl ComparisonToleranceScope {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminaries",
            Self::MajorPlanet => "Major planets",
            Self::LunarPoint => "Lunar points",
            Self::Asteroid => "Asteroids",
            Self::Custom => "Custom bodies",
            Self::Pluto => "Pluto fallback (approximate)",
        }
    }
}

/// One expected tolerance entry in the validation catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonToleranceEntry {
    /// Scope covered by this tolerance entry.
    pub scope: ComparisonToleranceScope,
    /// Expected tolerance for the scope.
    pub tolerance: ComparisonTolerance,
}

impl ComparisonToleranceEntry {
    /// Validates the tolerance entry before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        validate_comparison_tolerance(&self.tolerance)
    }

    /// Renders the compact report wording for this tolerance entry.
    pub fn summary_line(&self) -> String {
        let tolerance = &self.tolerance;
        format!(
            "{}: Δlon≤{:.3}°, Δlat≤{:.3}°, Δdist={}",
            self.scope.label(),
            tolerance.max_longitude_delta_deg,
            tolerance.max_latitude_delta_deg,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.3} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }

    /// Returns the compact summary line after validating the entry.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonToleranceEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured comparison tolerance policy details for a validation report.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonTolerancePolicySummary {
    /// Backend family used to select the comparison tolerance catalog.
    pub backend_family: BackendFamily,
    /// Tolerance entries in catalog order.
    pub entries: Vec<ComparisonToleranceEntry>,
    /// Scope coverage rows in catalog order.
    pub coverage: Vec<ComparisonToleranceScopeCoverageSummary>,
    /// Number of unique bodies covered by the comparison corpus.
    pub comparison_body_count: usize,
    /// Number of comparison samples in the corpus.
    pub comparison_sample_count: usize,
    /// Comparison corpus time window.
    pub comparison_window: TimeRange,
    /// Candidate backend coordinate frames covered by the comparison corpus.
    pub coordinate_frames: Vec<CoordinateFrame>,
}

impl ComparisonTolerancePolicySummary {
    /// Validates the policy summary before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.entries.len() != self.coverage.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary entry/coverage mismatch: expected {} coverage rows for {} entries, found {}",
                    self.entries.len(),
                    self.entries.len(),
                    self.coverage.len()
                ),
            ));
        }

        let mut seen_bodies: Vec<(CelestialBody, ComparisonToleranceScope)> = Vec::new();

        for (index, entry) in self.entries.iter().enumerate() {
            if self.entries[..index]
                .iter()
                .any(|prior| prior.scope == entry.scope)
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary has duplicate scope '{}'",
                        entry.scope.label()
                    ),
                ));
            }

            entry.validate()?;

            let coverage = &self.coverage[index];
            if coverage.entry.scope != entry.scope {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary scope mismatch at index {}: expected {}, found {}",
                        index + 1,
                        entry.scope.label(),
                        coverage.entry.scope.label()
                    ),
                ));
            }

            if coverage.entry.tolerance.backend_family != self.backend_family {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary backend family mismatch at index {}: expected {}, found {}",
                        index + 1,
                        backend_family_label(&self.backend_family),
                        backend_family_label(&coverage.entry.tolerance.backend_family)
                    ),
                ));
            }

            coverage.validate()?;

            for body in &coverage.bodies {
                if let Some((prior_body, prior_scope)) =
                    seen_bodies.iter().find(|(seen_body, _)| seen_body == body)
                {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison tolerance policy summary body '{}' appears in multiple scope rows: first seen in {}, repeated in {}",
                            prior_body,
                            prior_scope.label(),
                            entry.scope.label()
                        ),
                    ));
                }
            }
            seen_bodies.extend(
                coverage
                    .bodies
                    .iter()
                    .cloned()
                    .map(|body| (body, entry.scope)),
            );
        }

        let comparison_body_count = self
            .coverage
            .iter()
            .map(|coverage| coverage.body_count)
            .sum::<usize>();
        if comparison_body_count != self.comparison_body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary body-count mismatch: expected {}, found {}",
                    self.comparison_body_count, comparison_body_count
                ),
            ));
        }

        let comparison_sample_count = self
            .coverage
            .iter()
            .map(|coverage| coverage.sample_count)
            .sum::<usize>();
        if comparison_sample_count != self.comparison_sample_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary sample-count mismatch: expected {}, found {}",
                    self.comparison_sample_count, comparison_sample_count
                ),
            ));
        }

        if self.coordinate_frames.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison tolerance policy summary has no coordinate frames",
            ));
        }

        self.comparison_window.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance policy summary has invalid comparison window: {error}"
                ),
            )
        })?;

        for (index, frame) in self.coordinate_frames.iter().enumerate() {
            if self.coordinate_frames[..index].contains(frame) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance policy summary has duplicate coordinate frame '{frame}'"
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Renders the compact report wording for this tolerance policy.
    pub fn summary_line(&self) -> String {
        let scopes = self
            .entries
            .iter()
            .map(|entry| entry.scope.label())
            .collect::<Vec<_>>()
            .join(", ");
        let limits = format_comparison_tolerance_limits_for_report(&self.entries);
        let coverage = self
            .coverage
            .iter()
            .map(ComparisonToleranceScopeCoverageSummary::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let coordinate_frames = format_frames(&self.coordinate_frames);
        let window = self.comparison_window.summary_line();

        format!(
            "backend family={}; scopes={} ({scopes}); limits={limits}; coverage={coverage}; window={window}; frames={coordinate_frames}; evidence={} bodies, {} samples",
            backend_family_label(&self.backend_family),
            self.entries.len(),
            self.comparison_body_count,
            self.comparison_sample_count,
        )
    }

    /// Validates the policy summary and returns its compact report line.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonTolerancePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Scope coverage row within a comparison tolerance policy summary.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonToleranceScopeCoverageSummary {
    /// Tolerance entry covered by this scope summary.
    pub entry: ComparisonToleranceEntry,
    /// Bodies assigned to this tolerance scope in first-seen order.
    pub bodies: Vec<CelestialBody>,
    /// Number of distinct bodies assigned to this tolerance scope.
    pub body_count: usize,
    /// Total number of samples covered by this tolerance scope.
    pub sample_count: usize,
}

impl ComparisonToleranceScopeCoverageSummary {
    /// Validates the coverage row before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body_count != self.bodies.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison tolerance coverage row '{}' body-count mismatch: expected {}, found {}",
                    self.entry.scope.label(),
                    self.body_count,
                    self.bodies.len()
                ),
            ));
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].contains(body) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison tolerance coverage row '{}' has duplicate body '{}': first-seen order must be unique",
                        self.entry.scope.label(),
                        body
                    ),
                ));
            }
        }

        self.entry.validate()?;
        Ok(())
    }

    /// Renders the compact report wording for this tolerance coverage row.
    pub fn summary_line(&self) -> String {
        let bodies = if self.bodies.is_empty() {
            "none".to_string()
        } else {
            format_bodies(&self.bodies)
        };
        let tolerance = &self.entry.tolerance;
        format!(
            "{}: backend family={}, profile={}, bodies={} ({}), samples={}, limit Δlon≤{:.6}°, limit Δlat≤{:.6}°, limit Δdist={}",
            self.entry.scope.label(),
            tolerance_backend_family_label(&tolerance.backend_family),
            tolerance.profile,
            self.body_count,
            bodies,
            self.sample_count,
            tolerance.max_longitude_delta_deg,
            tolerance.max_latitude_delta_deg,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }

    /// Returns the compact summary line after validating the coverage row.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonToleranceScopeCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
