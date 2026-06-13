//! Motion types: [`MotionDirection`], [`Motion`], and [`MotionValidationError`].

use core::fmt;

/// The coarse direction of longitudinal motion.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MotionDirection {
    /// Motion is prograde or direct.
    Direct,
    /// Motion is effectively stationary at the chosen precision.
    Stationary,
    /// Motion is retrograde.
    Retrograde,
}

impl fmt::Display for MotionDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Direct => "Direct",
            Self::Stationary => "Stationary",
            Self::Retrograde => "Retrograde",
        };
        f.write_str(label)
    }
}

/// Apparent motion data for a position sample.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// Longitude speed in degrees per day.
    pub longitude_deg_per_day: Option<f64>,
    /// Latitude speed in degrees per day.
    pub latitude_deg_per_day: Option<f64>,
    /// Distance speed in astronomical units per day.
    pub distance_au_per_day: Option<f64>,
}

/// Errors returned when motion samples contain non-finite values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionValidationError {
    /// A motion component contained a non-finite value.
    NonFiniteSpeed {
        /// Motion field name.
        field: &'static str,
        /// Observed value.
        value: f64,
    },
}

impl fmt::Display for MotionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteSpeed { field, value } => {
                write!(f, "motion field `{field}` must be finite, got {value}")
            }
        }
    }
}

impl std::error::Error for MotionValidationError {}

impl Motion {
    /// Creates a new motion sample.
    pub const fn new(
        longitude_deg_per_day: Option<f64>,
        latitude_deg_per_day: Option<f64>,
        distance_au_per_day: Option<f64>,
    ) -> Self {
        Self {
            longitude_deg_per_day,
            latitude_deg_per_day,
            distance_au_per_day,
        }
    }

    /// Returns the longitudinal motion speed when available.
    pub const fn longitude_speed(self) -> Option<f64> {
        self.longitude_deg_per_day
    }

    /// Returns the latitudinal motion speed when available.
    pub const fn latitude_speed(self) -> Option<f64> {
        self.latitude_deg_per_day
    }

    /// Returns the radial motion speed when available.
    pub const fn distance_speed(self) -> Option<f64> {
        self.distance_au_per_day
    }

    /// Returns a compact one-line summary of the motion sample.
    pub fn summary_line(self) -> String {
        let longitude = self
            .longitude_speed()
            .map(|value| format!("{value} deg/day"))
            .unwrap_or_else(|| "n/a".to_string());
        let latitude = self
            .latitude_speed()
            .map(|value| format!("{value} deg/day"))
            .unwrap_or_else(|| "n/a".to_string());
        let distance = self
            .distance_speed()
            .map(|value| format!("{value} au/day"))
            .unwrap_or_else(|| "n/a".to_string());

        format!("longitude={longitude}; latitude={latitude}; distance={distance}")
    }

    /// Validates that every populated motion component is finite.
    pub fn validate(self) -> Result<(), MotionValidationError> {
        for (field, value) in [
            ("longitude_deg_per_day", self.longitude_deg_per_day),
            ("latitude_deg_per_day", self.latitude_deg_per_day),
            ("distance_au_per_day", self.distance_au_per_day),
        ] {
            if let Some(value) = value {
                if !value.is_finite() {
                    return Err(MotionValidationError::NonFiniteSpeed { field, value });
                }
            }
        }

        Ok(())
    }

    /// Returns the coarse longitudinal motion direction when that speed is available.
    ///
    /// The classification is sign-based: positive speed is direct, negative speed is retrograde,
    /// and an exact zero speed is stationary. Non-finite longitudinal speeds are treated as
    /// unknown until the sample is validated.
    pub fn longitude_direction(self) -> Option<MotionDirection> {
        let speed = self.longitude_speed()?;
        if !speed.is_finite() {
            return None;
        }

        Some(if speed > 0.0 {
            MotionDirection::Direct
        } else if speed < 0.0 {
            MotionDirection::Retrograde
        } else {
            MotionDirection::Stationary
        })
    }
}

impl fmt::Display for Motion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
