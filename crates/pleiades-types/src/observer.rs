//! Observer location types: [`ObserverLocation`] and [`ObserverLocationValidationError`].

use core::fmt;

use crate::angles::{Latitude, Longitude};

/// A geographic observer location.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ObserverLocation {
    /// Geographic latitude.
    pub latitude: Latitude,
    /// Geographic longitude, expressed in degrees east of Greenwich.
    pub longitude: Longitude,
    /// Optional elevation above sea level in meters.
    pub elevation_m: Option<f64>,
}

impl ObserverLocation {
    /// Creates a new observer location.
    pub const fn new(latitude: Latitude, longitude: Longitude, elevation_m: Option<f64>) -> Self {
        Self {
            latitude,
            longitude,
            elevation_m,
        }
    }

    /// Validates that the stored observer location is finite and within the
    /// latitude range expected by house calculations.
    pub fn validate(&self) -> Result<(), ObserverLocationValidationError> {
        let latitude = self.latitude.degrees();
        if !latitude.is_finite() {
            return Err(ObserverLocationValidationError::NonFiniteLatitude { value: latitude });
        }
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(ObserverLocationValidationError::LatitudeOutOfRange { value: latitude });
        }

        let longitude = self.longitude.degrees();
        if !longitude.is_finite() {
            return Err(ObserverLocationValidationError::NonFiniteLongitude { value: longitude });
        }

        if let Some(elevation_m) = self.elevation_m {
            if !elevation_m.is_finite() {
                return Err(ObserverLocationValidationError::NonFiniteElevation {
                    value: elevation_m,
                });
            }
        }

        Ok(())
    }

    /// Returns a compact one-line rendering of the observer location.
    pub fn summary_line(&self) -> String {
        let elevation = self
            .elevation_m
            .map(|value| format!("{value:.3} m"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "latitude={}, longitude={}, elevation={}",
            self.latitude, self.longitude, elevation
        )
    }

    /// Returns a compact one-line rendering after validating the stored data.
    pub fn validated_summary_line(&self) -> Result<String, ObserverLocationValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ObserverLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation errors for observer locations.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ObserverLocationValidationError {
    /// The observer latitude was not finite.
    NonFiniteLatitude {
        /// The offending value.
        value: f64,
    },
    /// The observer latitude fell outside the valid range.
    LatitudeOutOfRange {
        /// The offending value.
        value: f64,
    },
    /// The observer longitude was not finite.
    NonFiniteLongitude {
        /// The offending value.
        value: f64,
    },
    /// The observer elevation was not finite.
    NonFiniteElevation {
        /// The offending value.
        value: f64,
    },
}

impl ObserverLocationValidationError {
    /// Returns a compact one-line rendering of the validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonFiniteLatitude { value } => {
                format!("observer latitude must be finite, got {value}")
            }
            Self::LatitudeOutOfRange { value } => {
                format!("observer latitude must stay within [-90, 90], got {value}")
            }
            Self::NonFiniteLongitude { value } => {
                format!("observer longitude must be finite, got {value}")
            }
            Self::NonFiniteElevation { value } => {
                format!("observer elevation must be finite, got {value}")
            }
        }
    }
}

impl fmt::Display for ObserverLocationValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for ObserverLocationValidationError {}
