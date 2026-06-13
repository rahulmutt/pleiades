use core::fmt;

use pleiades_types::{
    CelestialBody, HouseSystem, ObserverLocation, ObserverLocationValidationError,
};

/// Default bodies included in a basic chart report.
pub const fn default_chart_bodies() -> &'static [CelestialBody] {
    &[
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
        CelestialBody::Pluto,
    ]
}

pub(super) fn render_house_system_label(house_system: &HouseSystem) -> String {
    match house_system {
        HouseSystem::Custom(custom) => custom.to_string(),
        other => crate::house_system_descriptor(other)
            .map(|descriptor| descriptor.canonical_name.to_string())
            .unwrap_or_else(|| "Custom".to_string()),
    }
}

pub(super) fn render_observer_location_label(observer: Option<&ObserverLocation>) -> String {
    observer
        .map(ObserverLocation::summary_line)
        .unwrap_or_else(|| "none".to_string())
}

/// Observer-policy summary for chart assembly and chart snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObserverPolicy {
    /// Body positions are queried geocentrically.
    Geocentric,
    /// The observer is used for house calculations only.
    HouseOnly,
}

impl ObserverPolicy {
    /// Returns the stable display label for the observer policy.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Geocentric => "geocentric",
            Self::HouseOnly => "house-only",
        }
    }

    /// Returns the compact one-line rendering of the observer policy.
    ///
    /// This mirrors [`core::fmt::Display`] so request and snapshot summaries can reuse a
    /// single typed vocabulary without duplicating the policy label table.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ObserverPolicy;
    ///
    /// assert_eq!(ObserverPolicy::HouseOnly.summary_line(), "house-only");
    /// assert_eq!(ObserverPolicy::Geocentric.to_string(), "geocentric");
    /// ```
    pub const fn summary_line(self) -> &'static str {
        self.as_str()
    }
}

impl fmt::Display for ObserverPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Observer-policy and location summary for chart requests and snapshots.
///
/// The house-observer location and the optional body-position observer are
/// kept as separate typed fields so report text can distinguish them clearly.
///
/// # Example
///
/// ```
/// use pleiades_core::{ObserverPolicy, ObserverSummary};
/// use pleiades_types::{Latitude, Longitude, ObserverLocation};
///
/// let summary = ObserverSummary::new(
///     ObserverPolicy::HouseOnly,
///     Some(ObserverLocation::new(
///         Latitude::from_degrees(12.5),
///         Longitude::from_degrees(45.0),
///         Some(100.0),
///     )),
/// )
/// .with_body_location(Some(ObserverLocation::new(
///     Latitude::from_degrees(-33.9),
///     Longitude::from_degrees(151.2),
///     None,
/// )));
///
/// let rendered = summary
///     .validated_summary_line()
///     .expect("valid observer summary");
/// assert!(rendered.contains("observer=house-only"));
/// assert!(rendered.contains("observer location=latitude=12.5°"));
/// assert!(rendered.contains("body observer=latitude=-33.9°"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ObserverSummary {
    /// The observer posture implied by the chart shape.
    pub policy: ObserverPolicy,
    /// The optional observer location rendered in report output.
    pub location: Option<ObserverLocation>,
    /// The optional topocentric body-position observer rendered in report output.
    pub body_location: Option<ObserverLocation>,
}

/// Errors returned when an observer summary no longer matches the chart observer posture.
#[derive(Clone, Debug, PartialEq)]
pub enum ObserverSummaryValidationError {
    /// A house-only summary no longer carries an observer location.
    HouseOnlyMissingObserver,
    /// The rendered observer location is invalid.
    InvalidObserverLocation(ObserverLocationValidationError),
    /// The rendered body-position observer location is invalid.
    InvalidBodyPosition(ObserverLocationValidationError),
}

impl fmt::Display for ObserverSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HouseOnlyMissingObserver => {
                f.write_str("observer summary for house-only posture requires an observer location")
            }
            Self::InvalidObserverLocation(error) => {
                write!(f, "observer summary location is invalid: {error}")
            }
            Self::InvalidBodyPosition(error) => {
                write!(f, "observer summary body location is invalid: {error}")
            }
        }
    }
}

impl std::error::Error for ObserverSummaryValidationError {}

impl ObserverSummary {
    /// Creates a new observer summary from the typed policy and optional location.
    pub const fn new(policy: ObserverPolicy, location: Option<ObserverLocation>) -> Self {
        Self {
            policy,
            location,
            body_location: None,
        }
    }

    /// Sets the optional body-position observer rendered in report output.
    pub fn with_body_location(mut self, body_location: Option<ObserverLocation>) -> Self {
        self.body_location = body_location;
        self
    }

    /// Returns the stored observer location as a compact label.
    pub fn location_label(&self) -> String {
        render_observer_location_label(self.location.as_ref())
    }

    /// Returns the stored body-position observer location as a compact label.
    pub fn body_location_label(&self) -> String {
        render_observer_location_label(self.body_location.as_ref())
    }

    /// Validates that the observer summary still matches the current observer posture.
    pub fn validate(&self) -> Result<(), ObserverSummaryValidationError> {
        if self.policy == ObserverPolicy::HouseOnly && self.location.is_none() {
            return Err(ObserverSummaryValidationError::HouseOnlyMissingObserver);
        }

        if let Some(observer) = &self.location {
            observer
                .validate()
                .map_err(ObserverSummaryValidationError::InvalidObserverLocation)?;
        }

        if let Some(observer) = &self.body_location {
            observer
                .validate()
                .map_err(ObserverSummaryValidationError::InvalidBodyPosition)?;
        }

        Ok(())
    }

    /// Returns the stored observer location as a compact label after validation.
    pub fn validated_location_label(&self) -> Result<String, ObserverSummaryValidationError> {
        self.validate()?;
        Ok(self.location_label())
    }

    /// Returns the stored body-position observer location as a compact label after validation.
    pub fn validated_body_location_label(&self) -> Result<String, ObserverSummaryValidationError> {
        self.validate()?;
        Ok(self.body_location_label())
    }

    /// Returns a compact one-line rendering of the observer posture and location.
    pub fn summary_line(&self) -> String {
        format!(
            "observer={}; observer location={}; body observer={}",
            self.policy,
            self.location_label(),
            self.body_location_label(),
        )
    }

    /// Returns a compact one-line rendering after validating the observer summary.
    pub fn validated_summary_line(&self) -> Result<String, ObserverSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ObserverSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
