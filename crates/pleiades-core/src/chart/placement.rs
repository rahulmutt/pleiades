use core::fmt;

use pleiades_apparent::ApparentProvenance;
use pleiades_backend::{EphemerisResult, EphemerisResultValidationError};
use pleiades_types::{CelestialBody, Motion, MotionDirection, ZodiacSign};

/// A single body placement within a chart.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyPlacement {
    /// The queried body.
    pub body: CelestialBody,
    /// The raw backend result.
    pub position: EphemerisResult,
    /// The body's zodiac sign in the requested mode, when ecliptic longitude is available.
    pub sign: Option<ZodiacSign>,
    /// The one-based house number, when house placement was requested.
    pub house: Option<usize>,
    /// Apparent-place provenance, when this placement was computed in apparent mode.
    pub apparent: Option<ApparentProvenance>,
}

/// Errors returned when a body placement no longer matches its stored result.
#[derive(Clone, Debug, PartialEq)]
pub enum BodyPlacementValidationError {
    /// The stored backend result is invalid.
    InvalidPosition {
        /// The body represented by the placement.
        body: CelestialBody,
        /// The nested backend-result validation error.
        error: EphemerisResultValidationError,
    },
    /// The stored house number is zero.
    InvalidHouseNumber {
        /// The body represented by the placement.
        body: CelestialBody,
        /// The invalid house number.
        house: usize,
    },
}

impl fmt::Display for BodyPlacementValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPosition { body, error } => {
                write!(f, "body {body} placement result is invalid: {error}")
            }
            Self::InvalidHouseNumber { body, house } => {
                write!(
                    f,
                    "body {body} placement house must be one-based, got {house}"
                )
            }
        }
    }
}

impl std::error::Error for BodyPlacementValidationError {}

impl BodyPlacement {
    /// Returns the backend motion sample when the backend supplied motion data.
    pub fn motion(&self) -> Option<&Motion> {
        self.position.motion.as_ref()
    }

    /// Validates the stored backend result and placement metadata.
    pub fn validate(&self) -> Result<(), BodyPlacementValidationError> {
        self.position.validate().map_err(|error| {
            BodyPlacementValidationError::InvalidPosition {
                body: self.body.clone(),
                error,
            }
        })?;

        if let Some(house) = self.house {
            if house == 0 {
                return Err(BodyPlacementValidationError::InvalidHouseNumber {
                    body: self.body.clone(),
                    house,
                });
            }
        }

        Ok(())
    }

    /// Returns the backend motion sample when the backend supplied motion data.
    pub fn motion_direction(&self) -> Option<MotionDirection> {
        let motion = self.motion()?;
        motion.validate().ok()?;
        motion.longitude_direction()
    }

    /// Returns the longitudinal motion speed when the backend supplied motion data.
    pub fn longitude_speed(&self) -> Option<f64> {
        self.motion()?.longitude_speed()
    }

    /// Returns the latitudinal motion speed when the backend supplied motion data.
    pub fn latitude_speed(&self) -> Option<f64> {
        self.motion()?.latitude_speed()
    }

    /// Returns the radial motion speed when the backend supplied motion data.
    pub fn distance_speed(&self) -> Option<f64> {
        self.motion()?.distance_speed()
    }

    /// Returns a compact one-line summary of the placement row used in chart reports.
    pub fn summary_line(&self) -> String {
        let longitude = self
            .position
            .ecliptic
            .as_ref()
            .map(|coords| format!("{}", coords.longitude))
            .unwrap_or_else(|| "n/a".to_string());
        let sign = self
            .sign
            .map(|sign| sign.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        let house = self
            .house
            .map(|house| house.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        let motion = self
            .motion_direction()
            .map(|direction| direction.to_string())
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "{:<12} {:>9}  {:<10}  {:>3}  {:<10}  {}",
            self.body, longitude, sign, house, motion, self.position.quality,
        )
    }

    /// Returns a compact one-line summary after validating the stored placement.
    pub fn validated_summary_line(&self) -> Result<String, BodyPlacementValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for BodyPlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
