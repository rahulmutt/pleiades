//! Coordinate types: [`EclipticCoordinates`], [`EquatorialCoordinates`], and [`CoordinateValidationError`].

use core::fmt;

use crate::angles::{Angle, Latitude, Longitude};

/// Ecliptic position data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EclipticCoordinates {
    /// Ecliptic longitude.
    pub longitude: Longitude,
    /// Ecliptic latitude.
    pub latitude: Latitude,
    /// Distance in astronomical units when available.
    pub distance_au: Option<f64>,
}

impl EclipticCoordinates {
    /// Creates a new ecliptic coordinate sample.
    pub const fn new(longitude: Longitude, latitude: Latitude, distance_au: Option<f64>) -> Self {
        Self {
            longitude,
            latitude,
            distance_au,
        }
    }

    /// Converts this ecliptic position into an equatorial position using the supplied obliquity.
    ///
    /// The transform is a pure geometric rotation: longitude/latitude are interpreted in the
    /// ecliptic frame, right ascension is normalized into `[0, 360)`, declination is signed, and
    /// any available distance is preserved. Round-tripping through
    /// [`EquatorialCoordinates::to_ecliptic`] with the same obliquity should stay numerically
    /// stable within normal floating-point tolerance, including near wraparound and high-latitude
    /// cases.
    pub fn to_equatorial(self, obliquity: Angle) -> EquatorialCoordinates {
        let longitude = self.longitude.degrees().to_radians();
        let latitude = self.latitude.degrees().to_radians();
        let obliquity = obliquity.radians();

        let x = longitude.cos() * latitude.cos();
        let y =
            longitude.sin() * latitude.cos() * obliquity.cos() - latitude.sin() * obliquity.sin();
        let z =
            longitude.sin() * latitude.cos() * obliquity.sin() + latitude.sin() * obliquity.cos();

        EquatorialCoordinates::new(
            Angle::from_degrees(y.atan2(x).to_degrees()).normalized_0_360(),
            Latitude::from_degrees(z.atan2((x * x + y * y).sqrt()).to_degrees()),
            self.distance_au,
        )
    }

    /// Validates that the sample is finite and physically sensible for frame conversions.
    pub fn validate(&self) -> Result<(), CoordinateValidationError> {
        validate_finite_coordinate_value("ecliptic", "longitude", self.longitude.degrees())?;
        validate_finite_coordinate_value("ecliptic", "latitude", self.latitude.degrees())?;
        validate_latitude_range("ecliptic", "latitude", self.latitude.degrees())?;
        if let Some(distance_au) = self.distance_au {
            validate_distance("ecliptic", distance_au)?;
        }
        Ok(())
    }
}

/// Equatorial position data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EquatorialCoordinates {
    /// Right ascension.
    pub right_ascension: Angle,
    /// Declination.
    pub declination: Latitude,
    /// Distance in astronomical units when available.
    pub distance_au: Option<f64>,
}

impl EquatorialCoordinates {
    /// Creates a new equatorial coordinate sample.
    pub const fn new(
        right_ascension: Angle,
        declination: Latitude,
        distance_au: Option<f64>,
    ) -> Self {
        Self {
            right_ascension,
            declination,
            distance_au,
        }
    }

    /// Validates that the sample is finite and normalized enough for release-facing frame checks.
    pub fn validate(&self) -> Result<(), CoordinateValidationError> {
        validate_finite_coordinate_value(
            "equatorial",
            "right_ascension",
            self.right_ascension.degrees(),
        )?;
        validate_right_ascension_range(self.right_ascension.degrees())?;
        validate_finite_coordinate_value("equatorial", "declination", self.declination.degrees())?;
        validate_latitude_range("equatorial", "declination", self.declination.degrees())?;
        if let Some(distance_au) = self.distance_au {
            validate_distance("equatorial", distance_au)?;
        }
        Ok(())
    }

    /// Converts this equatorial position back into an ecliptic position using the supplied obliquity.
    ///
    /// The transform is the inverse geometric rotation of [`EclipticCoordinates::to_equatorial`]:
    /// right ascension is interpreted as a normalized angle, declination is signed, and any
    /// available distance is preserved. Using the same obliquity as the forward transform should
    /// recover the original ecliptic position within normal floating-point tolerance.
    pub fn to_ecliptic(self, obliquity: Angle) -> EclipticCoordinates {
        let right_ascension = self.right_ascension.degrees().to_radians();
        let declination = self.declination.degrees().to_radians();
        let obliquity = obliquity.radians();

        let x = right_ascension.cos() * declination.cos();
        let y = right_ascension.sin() * declination.cos() * obliquity.cos()
            + declination.sin() * obliquity.sin();
        let z = -right_ascension.sin() * declination.cos() * obliquity.sin()
            + declination.sin() * obliquity.cos();

        EclipticCoordinates::new(
            Longitude::from_degrees(y.atan2(x).to_degrees()),
            Latitude::from_degrees(z.atan2((x * x + y * y).sqrt()).to_degrees()),
            self.distance_au,
        )
    }
}

/// Validation errors for shared coordinate samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordinateValidationError {
    /// A stored numeric field was not finite.
    NonFiniteValue {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The field that failed validation.
        field: &'static str,
        /// The offending value.
        value: f64,
    },
    /// A stored latitude-like field fell outside the expected closed range.
    LatitudeOutOfRange {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The field that failed validation.
        field: &'static str,
        /// The offending value.
        value: f64,
    },
    /// A stored right-ascension field fell outside the expected half-open range.
    RightAscensionOutOfRange {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The field that failed validation.
        field: &'static str,
        /// The offending value.
        value: f64,
    },
    /// A stored distance channel was negative.
    NegativeDistance {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The offending value.
        value: f64,
    },
}

impl CoordinateValidationError {
    /// Returns a compact one-line rendering of the validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonFiniteValue {
                coordinate,
                field,
                value,
            } => format!("{coordinate} coordinate field `{field}` must be finite, got {value}"),
            Self::LatitudeOutOfRange {
                coordinate,
                field,
                value,
            } => format!(
                "{coordinate} coordinate field `{field}` must stay within [-90, 90], got {value}"
            ),
            Self::RightAscensionOutOfRange {
                coordinate,
                field,
                value,
            } => format!(
                "{coordinate} coordinate field `{field}` must stay within [0, 360), got {value}"
            ),
            Self::NegativeDistance { coordinate, value } => format!(
                "{coordinate} coordinate field `distance_au` must be non-negative, got {value}"
            ),
        }
    }
}

impl fmt::Display for CoordinateValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for CoordinateValidationError {}

pub(crate) fn validate_finite_coordinate_value(
    coordinate: &'static str,
    field: &'static str,
    value: f64,
) -> Result<(), CoordinateValidationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(CoordinateValidationError::NonFiniteValue {
            coordinate,
            field,
            value,
        })
    }
}

pub(crate) fn validate_latitude_range(
    coordinate: &'static str,
    field: &'static str,
    value: f64,
) -> Result<(), CoordinateValidationError> {
    if (-90.0..=90.0).contains(&value) {
        Ok(())
    } else {
        Err(CoordinateValidationError::LatitudeOutOfRange {
            coordinate,
            field,
            value,
        })
    }
}

pub(crate) fn validate_right_ascension_range(value: f64) -> Result<(), CoordinateValidationError> {
    if (0.0..360.0).contains(&value) {
        Ok(())
    } else {
        Err(CoordinateValidationError::RightAscensionOutOfRange {
            coordinate: "equatorial",
            field: "right_ascension",
            value,
        })
    }
}

pub(crate) fn validate_distance(
    coordinate: &'static str,
    value: f64,
) -> Result<(), CoordinateValidationError> {
    if !value.is_finite() {
        return Err(CoordinateValidationError::NonFiniteValue {
            coordinate,
            field: "distance_au",
            value,
        });
    }
    if value < 0.0 {
        Err(CoordinateValidationError::NegativeDistance { coordinate, value })
    } else {
        Ok(())
    }
}
