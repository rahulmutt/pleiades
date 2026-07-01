use core::fmt;

use pleiades_types::{Angle, CelestialBody, Longitude};

/// A summary of major aspect matches in a chart snapshot.
///
/// # Example
///
/// ```
/// use pleiades_core::AspectSummary;
///
/// let summary = AspectSummary {
///     conjunction: 0,
///     sextile: 1,
///     square: 0,
///     trine: 2,
///     opposition: 0,
/// };
///
/// assert_eq!(summary.summary_line(), "1 Sextile, 2 Trine");
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert_eq!(summary.validate(3), Ok(()));
/// assert!(summary.has_known_aspects());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AspectSummary {
    /// Placements matched by conjunction.
    pub conjunction: usize,
    /// Placements matched by sextile.
    pub sextile: usize,
    /// Placements matched by square.
    pub square: usize,
    /// Placements matched by trine.
    pub trine: usize,
    /// Placements matched by opposition.
    pub opposition: usize,
}

impl AspectSummary {
    pub(super) fn from_matches(matches: &[AspectMatch]) -> Self {
        let mut summary = Self::default();

        for aspect in matches {
            summary.increment(aspect.kind);
        }

        summary
    }

    fn increment(&mut self, kind: AspectKind) {
        match kind {
            AspectKind::Conjunction => self.conjunction += 1,
            AspectKind::Sextile => self.sextile += 1,
            AspectKind::Square => self.square += 1,
            AspectKind::Trine => self.trine += 1,
            AspectKind::Opposition => self.opposition += 1,
        }
    }

    /// Returns `true` when the snapshot contains at least one major aspect match.
    pub fn has_known_aspects(self) -> bool {
        self.conjunction + self.sextile + self.square + self.trine + self.opposition > 0
    }

    /// Validates that the summary covers the expected number of aspect matches.
    pub fn validate(self, aspect_count: usize) -> Result<(), AspectSummaryValidationError> {
        let actual = self.conjunction + self.sextile + self.square + self.trine + self.opposition;
        if actual == aspect_count {
            Ok(())
        } else {
            Err(AspectSummaryValidationError::PlacementCountMismatch {
                expected: aspect_count,
                actual,
            })
        }
    }

    /// Returns a compact one-line summary of the major aspect families in the snapshot.
    pub fn summary_line(self) -> String {
        let mut summary = String::new();
        for (count, aspect) in [
            (self.conjunction, AspectKind::Conjunction),
            (self.sextile, AspectKind::Sextile),
            (self.square, AspectKind::Square),
            (self.trine, AspectKind::Trine),
            (self.opposition, AspectKind::Opposition),
        ] {
            if count == 0 {
                continue;
            }
            if !summary.is_empty() {
                summary.push_str(", ");
            }
            summary.push_str(&count.to_string());
            summary.push(' ');
            summary.push_str(aspect.name());
        }

        if summary.is_empty() {
            summary.push_str("no major aspects");
        }

        summary
    }
}

/// Errors returned when a major-aspect summary no longer matches the chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectSummaryValidationError {
    /// The total aspect-summary count did not match the chart aspect count.
    PlacementCountMismatch {
        /// Expected number of aspect matches.
        expected: usize,
        /// Aspect-summary total that was observed.
        actual: usize,
    },
}

impl fmt::Display for AspectSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlacementCountMismatch { expected, actual } => write!(
                f,
                "aspect summary placement count mismatch: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for AspectSummaryValidationError {}

impl fmt::Display for AspectSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A major aspect family in the chart façade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AspectKind {
    /// Bodies are aligned near the same longitude.
    Conjunction,
    /// Bodies are separated by roughly 60 degrees.
    Sextile,
    /// Bodies are separated by roughly 90 degrees.
    Square,
    /// Bodies are separated by roughly 120 degrees.
    Trine,
    /// Bodies are separated by roughly 180 degrees.
    Opposition,
}

impl AspectKind {
    /// Returns the canonical display name for the aspect family.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Conjunction => "Conjunction",
            Self::Sextile => "Sextile",
            Self::Square => "Square",
            Self::Trine => "Trine",
            Self::Opposition => "Opposition",
        }
    }
}

impl fmt::Display for AspectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Validation failure for a configurable aspect definition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AspectDefinitionValidationError {
    /// The exact aspect angle was not finite or fell outside the supported half-circle range.
    ExactDegreesOutOfRange {
        /// The rejected exact aspect angle, in degrees.
        exact_degrees: f64,
    },
    /// The orb was not finite or fell outside the supported half-circle range.
    OrbDegreesOutOfRange {
        /// The rejected orb half-width, in degrees.
        orb_degrees: f64,
    },
}

impl AspectDefinitionValidationError {
    /// Returns a compact one-line summary of the validation failure.
    pub fn summary_line(self) -> String {
        match self {
            Self::ExactDegreesOutOfRange { exact_degrees } => format!(
                "aspect definition exact_degrees must be finite and between 0 and 180 degrees; found {exact_degrees}"
            ),
            Self::OrbDegreesOutOfRange { orb_degrees } => format!(
                "aspect definition orb_degrees must be finite and between 0 and 180 degrees; found {orb_degrees}"
            ),
        }
    }
}

impl fmt::Display for AspectDefinitionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for AspectDefinitionValidationError {}

/// An aspect definition used for configurable matching.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectDefinition {
    /// The aspect family.
    pub kind: AspectKind,
    /// The exact angular separation that defines the aspect.
    pub exact_degrees: f64,
    /// The maximum allowed orb in degrees.
    pub orb_degrees: f64,
}

impl AspectDefinition {
    /// Creates a new aspect definition.
    pub const fn new(kind: AspectKind, exact_degrees: f64, orb_degrees: f64) -> Self {
        Self {
            kind,
            exact_degrees,
            orb_degrees,
        }
    }

    /// Returns a compact one-line rendering of the configured aspect.
    ///
    /// This keeps the configurable aspect vocabulary aligned with the other
    /// typed chart summaries while making the angle parameters easy to audit.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::{AspectDefinition, AspectKind};
    ///
    /// let definition = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
    /// assert_eq!(definition.summary_line(), "kind=Sextile; exact_degrees=60°; orb_degrees=4°");
    /// assert_eq!(definition.to_string(), definition.summary_line());
    /// ```
    pub fn summary_line(self) -> String {
        format!(
            "kind={}; exact_degrees={}°; orb_degrees={}°",
            self.kind, self.exact_degrees, self.orb_degrees
        )
    }

    /// Validates that the configured exact angle and orb remain finite and within range.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::{AspectDefinition, AspectKind};
    ///
    /// let definition = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
    /// assert!(definition.validate().is_ok());
    /// ```
    pub fn validate(self) -> Result<(), AspectDefinitionValidationError> {
        if !self.exact_degrees.is_finite() || !(0.0..=180.0).contains(&self.exact_degrees) {
            return Err(AspectDefinitionValidationError::ExactDegreesOutOfRange {
                exact_degrees: self.exact_degrees,
            });
        }

        if !self.orb_degrees.is_finite() || !(0.0..=180.0).contains(&self.orb_degrees) {
            return Err(AspectDefinitionValidationError::OrbDegreesOutOfRange {
                orb_degrees: self.orb_degrees,
            });
        }

        Ok(())
    }
}

impl fmt::Display for AspectDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validates a slice of aspect definitions.
///
/// # Example
///
/// ```
/// use pleiades_core::{validate_aspect_definitions, AspectDefinition, AspectKind};
///
/// let definitions = [
///     AspectDefinition::new(AspectKind::Conjunction, 0.0, 8.0),
///     AspectDefinition::new(AspectKind::Opposition, 180.0, 8.0),
/// ];
///
/// assert!(validate_aspect_definitions(&definitions).is_ok());
/// ```
pub fn validate_aspect_definitions(
    definitions: &[AspectDefinition],
) -> Result<(), AspectDefinitionValidationError> {
    definitions
        .iter()
        .copied()
        .try_for_each(AspectDefinition::validate)
}

/// A matched aspect between two bodies.
#[derive(Clone, Debug, PartialEq)]
pub struct AspectMatch {
    /// The body on the left side of the pairing.
    pub left: CelestialBody,
    /// The body on the right side of the pairing.
    pub right: CelestialBody,
    /// The matched aspect family.
    pub kind: AspectKind,
    /// The measured angular separation between the bodies.
    pub separation: Angle,
    /// The absolute orb from the exact aspect angle.
    pub orb: Angle,
}

impl fmt::Display for AspectMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} (separation: {}, orb: {})",
            self.left, self.kind, self.right, self.separation, self.orb
        )
    }
}

pub(super) const DEFAULT_ASPECT_DEFINITIONS: [AspectDefinition; 5] = [
    AspectDefinition::new(AspectKind::Conjunction, 0.0, 8.0),
    AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0),
    AspectDefinition::new(AspectKind::Square, 90.0, 6.0),
    AspectDefinition::new(AspectKind::Trine, 120.0, 6.0),
    AspectDefinition::new(AspectKind::Opposition, 180.0, 8.0),
];

pub(super) fn default_aspect_definitions() -> &'static [AspectDefinition] {
    &DEFAULT_ASPECT_DEFINITIONS
}

pub(super) fn angle_separation(left: Longitude, right: Longitude) -> Angle {
    let difference = (right.degrees() - left.degrees()).rem_euclid(360.0);
    let separation = if difference > 180.0 {
        360.0 - difference
    } else {
        difference
    };
    Angle::from_degrees(separation)
}

pub(super) fn best_aspect_definition(
    separation: Angle,
    definitions: &[AspectDefinition],
) -> Option<AspectDefinition> {
    definitions
        .iter()
        .copied()
        .filter(|definition| {
            (separation.degrees() - definition.exact_degrees).abs() <= definition.orb_degrees
        })
        .min_by(|left, right| {
            let left_orb = (separation.degrees() - left.exact_degrees).abs();
            let right_orb = (separation.degrees() - right.exact_degrees).abs();
            left_orb
                .partial_cmp(&right_orb)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
}
