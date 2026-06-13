//! Body classification and per-class comparison summaries.

use std::fmt;

use pleiades_core::CelestialBody;

use crate::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BodyClass {
    Luminary,
    MajorPlanet,
    LunarPoint,
    Asteroid,
    Custom,
}

impl BodyClass {
    pub(crate) const ALL: [Self; 5] = [
        Self::Luminary,
        Self::MajorPlanet,
        Self::LunarPoint,
        Self::Asteroid,
        Self::Custom,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminaries",
            Self::MajorPlanet => "Major planets",
            Self::LunarPoint => "Lunar points",
            Self::Asteroid => "Asteroids",
            Self::Custom => "Custom bodies",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Luminary => 0,
            Self::MajorPlanet => 1,
            Self::LunarPoint => 2,
            Self::Asteroid => 3,
            Self::Custom => 4,
        }
    }
}

pub(crate) fn body_class(body: &CelestialBody) -> BodyClass {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => BodyClass::Luminary,
        CelestialBody::Mercury
        | CelestialBody::Venus
        | CelestialBody::Mars
        | CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune
        | CelestialBody::Pluto => BodyClass::MajorPlanet,
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => BodyClass::LunarPoint,
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => BodyClass::Asteroid,
        CelestialBody::Custom(_) => BodyClass::Custom,
        _ => BodyClass::Custom,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BodyClassSummary {
    pub(crate) class: BodyClass,
    pub(crate) sample_count: usize,
    pub(crate) max_longitude_delta_body: Option<CelestialBody>,
    pub(crate) max_longitude_delta_deg: f64,
    pub(crate) sum_longitude_delta_deg: f64,
    pub(crate) sum_longitude_delta_sq_deg: f64,
    pub(crate) median_longitude_delta_deg: f64,
    pub(crate) percentile_longitude_delta_deg: f64,
    pub(crate) max_latitude_delta_body: Option<CelestialBody>,
    pub(crate) max_latitude_delta_deg: f64,
    pub(crate) sum_latitude_delta_deg: f64,
    pub(crate) sum_latitude_delta_sq_deg: f64,
    pub(crate) median_latitude_delta_deg: f64,
    pub(crate) percentile_latitude_delta_deg: f64,
    pub(crate) max_distance_delta_body: Option<CelestialBody>,
    pub(crate) max_distance_delta_au: Option<f64>,
    pub(crate) sum_distance_delta_au: f64,
    pub(crate) sum_distance_delta_sq_au: f64,
    pub(crate) distance_count: usize,
    pub(crate) median_distance_delta_au: Option<f64>,
    pub(crate) percentile_distance_delta_au: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BodyClassSummaryValidationError {
    MissingSamples,
    FieldOutOfSync { class: BodyClass },
}

impl fmt::Display for BodyClassSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSamples => f.write_str("body-class summary is unavailable"),
            Self::FieldOutOfSync { class } => write!(
                f,
                "the body-class summary for {} is out of sync with the current evidence",
                class.label()
            ),
        }
    }
}

impl std::error::Error for BodyClassSummaryValidationError {}

#[derive(Clone, Debug)]
pub(crate) struct BodyClassToleranceSummary {
    pub(crate) class: BodyClass,
    pub(crate) tolerance: ComparisonTolerance,
    pub(crate) body_count: usize,
    pub(crate) sample_count: usize,
    pub(crate) within_tolerance_body_count: usize,
    pub(crate) outside_tolerance_body_count: usize,
    pub(crate) outside_tolerance_sample_count: usize,
    pub(crate) max_longitude_delta_body: Option<CelestialBody>,
    pub(crate) max_longitude_delta_deg: Option<f64>,
    pub(crate) max_latitude_delta_body: Option<CelestialBody>,
    pub(crate) max_latitude_delta_deg: Option<f64>,
    pub(crate) max_distance_delta_body: Option<CelestialBody>,
    pub(crate) max_distance_delta_au: Option<f64>,
    pub(crate) sum_longitude_delta_deg: f64,
    pub(crate) sum_longitude_delta_sq_deg: f64,
    pub(crate) sum_latitude_delta_deg: f64,
    pub(crate) sum_latitude_delta_sq_deg: f64,
    pub(crate) sum_distance_delta_au: f64,
    pub(crate) sum_distance_delta_sq_au: f64,
    pub(crate) distance_count: usize,
    pub(crate) median_longitude_delta_deg: f64,
    pub(crate) percentile_longitude_delta_deg: f64,
    pub(crate) median_latitude_delta_deg: f64,
    pub(crate) percentile_latitude_delta_deg: f64,
    pub(crate) median_distance_delta_au: Option<f64>,
    pub(crate) percentile_distance_delta_au: Option<f64>,
    pub(crate) outside_bodies: Vec<CelestialBody>,
}

impl BodyClassToleranceSummary {
    pub(crate) fn validate(&self) -> Result<(), EphemerisError> {
        if self.body_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "body-class tolerance summary must include at least one body",
            ));
        }

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "body-class tolerance summary must include at least one sample",
            ));
        }

        let within_and_outside = self
            .within_tolerance_body_count
            .saturating_add(self.outside_tolerance_body_count);
        if within_and_outside != self.body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary body-count mismatch: expected {}, found within tolerance {} + outside tolerance {} = {}",
                    self.body_count,
                    self.within_tolerance_body_count,
                    self.outside_tolerance_body_count,
                    within_and_outside
                ),
            ));
        }

        if self.outside_tolerance_sample_count > self.sample_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary outside-sample count {} exceeds sample count {}",
                    self.outside_tolerance_sample_count, self.sample_count
                ),
            ));
        }

        if self.outside_bodies.len() != self.outside_tolerance_body_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary outside-body list mismatch: expected {}, found {}",
                    self.outside_tolerance_body_count,
                    self.outside_bodies.len()
                ),
            ));
        }

        let mut seen_outside_bodies = BTreeSet::new();
        for body in &self.outside_bodies {
            if !seen_outside_bodies.insert(body.to_string()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body-class tolerance summary contains a duplicate outside body: {body}"
                    ),
                ));
            }
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            (
                "median_longitude_delta_deg",
                Some(self.median_longitude_delta_deg),
            ),
            (
                "percentile_longitude_delta_deg",
                Some(self.percentile_longitude_delta_deg),
            ),
            (
                "median_latitude_delta_deg",
                Some(self.median_latitude_delta_deg),
            ),
            (
                "percentile_latitude_delta_deg",
                Some(self.percentile_latitude_delta_deg),
            ),
            (
                "sum_longitude_delta_deg",
                Some(self.sum_longitude_delta_deg),
            ),
            (
                "sum_longitude_delta_sq_deg",
                Some(self.sum_longitude_delta_sq_deg),
            ),
            ("sum_latitude_delta_deg", Some(self.sum_latitude_delta_deg)),
            (
                "sum_latitude_delta_sq_deg",
                Some(self.sum_latitude_delta_sq_deg),
            ),
            ("sum_distance_delta_au", Some(self.sum_distance_delta_au)),
            (
                "sum_distance_delta_sq_au",
                Some(self.sum_distance_delta_sq_au),
            ),
        ] {
            if let Some(value) = value {
                if !value.is_finite() || value.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "body-class tolerance summary for {} field `{label}` must be a finite non-negative value",
                            self.class.label()
                        ),
                    ));
                }
            }
        }

        if let Some(value) = self.max_distance_delta_au {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body-class tolerance summary for {} field `max_distance_delta_au` must be a finite non-negative value",
                        self.class.label()
                    ),
                ));
            }
        }

        if self.distance_count > self.sample_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary distance count {} exceeds sample count {}",
                    self.distance_count, self.sample_count
                ),
            ));
        }

        let distance_fields_present = self.max_distance_delta_body.is_some()
            || self.max_distance_delta_au.is_some()
            || self.median_distance_delta_au.is_some()
            || self.percentile_distance_delta_au.is_some();
        if self.distance_count == 0 {
            if distance_fields_present {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body-class tolerance summary for {} must not carry distance metrics when no distance samples are present",
                        self.class.label()
                    ),
                ));
            }
        } else if self.max_distance_delta_body.is_none()
            || self.max_distance_delta_au.is_none()
            || self.median_distance_delta_au.is_none()
            || self.percentile_distance_delta_au.is_none()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary for {} must carry distance extrema and spread metrics when distance samples are present",
                    self.class.label()
                ),
            ));
        }

        if self.max_longitude_delta_body.is_none() || self.max_latitude_delta_body.is_none() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body-class tolerance summary for {} must name the longitude and latitude extrema drivers",
                    self.class.label()
                ),
            ));
        }

        Ok(())
    }

    pub(crate) fn mean_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_longitude_delta_deg / self.sample_count as f64
        }
    }

    pub(crate) fn rms_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_longitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    pub(crate) fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    pub(crate) fn rms_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_latitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    pub(crate) fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    pub(crate) fn rms_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some((self.sum_distance_delta_sq_au / self.distance_count as f64).sqrt())
        }
    }

    pub(crate) fn summary_line(&self) -> String {
        let tolerance = &self.tolerance;
        let max_longitude_body = self
            .max_longitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_latitude_body = self
            .max_latitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_distance_body = self
            .max_distance_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let longitude_margin = self
            .max_longitude_delta_deg
            .map(|value| format!("{:+.12}°", tolerance.max_longitude_delta_deg - value))
            .unwrap_or_else(|| "n/a".to_string());
        let latitude_margin = self
            .max_latitude_delta_deg
            .map(|value| format!("{:+.12}°", tolerance.max_latitude_delta_deg - value))
            .unwrap_or_else(|| "n/a".to_string());
        let distance_margin = self
            .max_distance_delta_au
            .zip(tolerance.max_distance_delta_au)
            .map(|(value, limit)| format!("{:+.12} AU", limit - value))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "{}: backend family={}, profile={}, bodies={}, samples={}, within tolerance bodies={}, outside tolerance bodies={}, limit Δlon≤{:.6}°, margin Δlon={}, limit Δlat≤{:.6}°, margin Δlat={}, limit Δdist={}, margin Δdist={}, max Δlon={}{}, max Δlat={}{}, max Δdist={}{}",
            self.class.label(),
            tolerance_backend_family_label(&tolerance.backend_family),
            tolerance.profile,
            self.body_count,
            self.sample_count,
            self.within_tolerance_body_count,
            self.outside_tolerance_body_count,
            tolerance.max_longitude_delta_deg,
            longitude_margin,
            tolerance.max_latitude_delta_deg,
            latitude_margin,
            tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            distance_margin,
            self.max_longitude_delta_deg
                .map(|value| format!("{value:.12}°"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_longitude_body,
            self.max_latitude_delta_deg
                .map(|value| format!("{value:.12}°"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_latitude_body,
            self.max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_distance_body
        )
    }

    pub(crate) fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  {}", self.class.label())?;
        writeln!(
            f,
            "    backend family: {}",
            tolerance_backend_family_label(&self.tolerance.backend_family)
        )?;
        writeln!(f, "    profile: {}", self.tolerance.profile)?;
        writeln!(f, "    bodies: {}", self.body_count)?;
        writeln!(f, "    samples: {}", self.sample_count)?;
        writeln!(
            f,
            "    within tolerance bodies: {}",
            self.within_tolerance_body_count
        )?;
        writeln!(
            f,
            "    outside tolerance bodies: {}",
            self.outside_tolerance_body_count
        )?;
        writeln!(
            f,
            "    outside tolerance samples: {}",
            self.outside_tolerance_sample_count
        )?;
        if !self.outside_bodies.is_empty() {
            writeln!(
                f,
                "    outside bodies: {}",
                format_bodies(&self.outside_bodies)
            )?;
        }
        if let (Some(body), Some(value)) = (
            self.max_longitude_delta_body.as_ref(),
            self.max_longitude_delta_deg,
        ) {
            writeln!(
                f,
                "    max longitude delta: {:.12}° ({}) | limit Δlon≤{:.6}°, margin Δlon={:+.12}°",
                value,
                body,
                self.tolerance.max_longitude_delta_deg,
                self.tolerance.max_longitude_delta_deg - value
            )?;
        }
        writeln!(
            f,
            "    mean longitude delta: {:.12}°",
            self.mean_longitude_delta_deg()
        )?;
        writeln!(
            f,
            "    median longitude delta: {:.12}°",
            self.median_longitude_delta_deg
        )?;
        writeln!(
            f,
            "    95th percentile longitude delta: {:.12}°",
            self.percentile_longitude_delta_deg
        )?;
        writeln!(
            f,
            "    rms longitude delta: {:.12}°",
            self.rms_longitude_delta_deg()
        )?;
        if let (Some(body), Some(value)) = (
            self.max_latitude_delta_body.as_ref(),
            self.max_latitude_delta_deg,
        ) {
            writeln!(
                f,
                "    max latitude delta: {:.12}° ({}) | limit Δlat≤{:.6}°, margin Δlat={:+.12}°",
                value,
                body,
                self.tolerance.max_latitude_delta_deg,
                self.tolerance.max_latitude_delta_deg - value
            )?;
        }
        writeln!(
            f,
            "    mean latitude delta: {:.12}°",
            self.mean_latitude_delta_deg()
        )?;
        writeln!(
            f,
            "    median latitude delta: {:.12}°",
            self.median_latitude_delta_deg
        )?;
        writeln!(
            f,
            "    95th percentile latitude delta: {:.12}°",
            self.percentile_latitude_delta_deg
        )?;
        writeln!(
            f,
            "    rms latitude delta: {:.12}°",
            self.rms_latitude_delta_deg()
        )?;
        if let (Some(body), Some(value)) = (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_au,
        ) {
            let limit = self
                .tolerance
                .max_distance_delta_au
                .expect("distance tolerance should exist for class summaries");
            writeln!(
                f,
                "    max distance delta: {:.12} AU ({}) | limit Δdist={:.6} AU, margin Δdist={:+.12} AU",
                value,
                body,
                limit,
                limit - value
            )?;
        }
        if let Some(value) = self.mean_distance_delta_au() {
            writeln!(f, "    mean distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.median_distance_delta_au {
            writeln!(f, "    median distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.percentile_distance_delta_au {
            writeln!(f, "    95th percentile distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.rms_distance_delta_au() {
            writeln!(f, "    rms distance delta: {:.12} AU", value)?;
        }

        Ok(())
    }
}

impl fmt::Display for BodyClassToleranceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl BodyClassSummary {
    pub(crate) fn validate(&self) -> Result<(), BodyClassSummaryValidationError> {
        if self.sample_count == 0 {
            return Err(BodyClassSummaryValidationError::MissingSamples);
        }

        for (_label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("sum_longitude_delta_deg", self.sum_longitude_delta_deg),
            (
                "sum_longitude_delta_sq_deg",
                self.sum_longitude_delta_sq_deg,
            ),
            (
                "median_longitude_delta_deg",
                self.median_longitude_delta_deg,
            ),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("sum_latitude_delta_deg", self.sum_latitude_delta_deg),
            ("sum_latitude_delta_sq_deg", self.sum_latitude_delta_sq_deg),
            ("median_latitude_delta_deg", self.median_latitude_delta_deg),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            ("sum_distance_delta_au", self.sum_distance_delta_au),
            ("sum_distance_delta_sq_au", self.sum_distance_delta_sq_au),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(BodyClassSummaryValidationError::FieldOutOfSync { class: self.class });
            }
        }

        if self.max_longitude_delta_body.is_none() || self.max_latitude_delta_body.is_none() {
            return Err(BodyClassSummaryValidationError::FieldOutOfSync { class: self.class });
        }

        if self.distance_count == 0 {
            if self.max_distance_delta_body.is_some()
                || self.max_distance_delta_au.is_some()
                || self.median_distance_delta_au.is_some()
                || self.percentile_distance_delta_au.is_some()
            {
                return Err(BodyClassSummaryValidationError::FieldOutOfSync { class: self.class });
            }
        } else {
            if self.max_distance_delta_body.is_none()
                || self.max_distance_delta_au.is_none()
                || self.median_distance_delta_au.is_none()
                || self.percentile_distance_delta_au.is_none()
            {
                return Err(BodyClassSummaryValidationError::FieldOutOfSync { class: self.class });
            }
        }

        Ok(())
    }

    pub(crate) fn validated_summary_line(&self) -> Result<String, BodyClassSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    pub(crate) fn mean_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_longitude_delta_deg / self.sample_count as f64
        }
    }

    pub(crate) fn rms_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_longitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    pub(crate) fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    pub(crate) fn rms_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            (self.sum_latitude_delta_sq_deg / self.sample_count as f64).sqrt()
        }
    }

    pub(crate) fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    pub(crate) fn rms_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some((self.sum_distance_delta_sq_au / self.distance_count as f64).sqrt())
        }
    }

    pub(crate) fn summary_line(&self) -> String {
        if self.sample_count == 0 {
            return String::new();
        }

        let max_distance = self
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let mean_distance = self
            .mean_distance_delta_au()
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let median_distance = self
            .median_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let percentile_distance = self
            .percentile_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());
        let rms_distance = self
            .rms_distance_delta_au()
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, median Δlon={:.12}°, 95th percentile longitude delta: {:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, median Δlat={:.12}°, 95th percentile latitude delta: {:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, median Δdist={}, 95th percentile distance delta: {}, rms Δdist={}",
            self.sample_count,
            self.max_longitude_delta_deg,
            format_summary_body(&self.max_longitude_delta_body),
            self.mean_longitude_delta_deg(),
            self.median_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.rms_longitude_delta_deg(),
            self.max_latitude_delta_deg,
            format_summary_body(&self.max_latitude_delta_body),
            self.mean_latitude_delta_deg(),
            self.median_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            self.rms_latitude_delta_deg(),
            max_distance,
            format_summary_body(&self.max_distance_delta_body),
            mean_distance,
            median_distance,
            percentile_distance,
            rms_distance,
        )
    }

    pub(crate) fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sample_count == 0 {
            return Ok(());
        }

        writeln!(f, "  {}", self.class.label())?;
        match self.validated_summary_line() {
            Ok(summary_line) => writeln!(f, "    {}", summary_line)?,
            Err(error) => writeln!(f, "    body-class error envelope unavailable ({error})")?,
        }

        Ok(())
    }
}

impl fmt::Display for BodyClassSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

