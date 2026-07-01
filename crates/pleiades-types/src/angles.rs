//! Angular quantity primitives: [`Angle`], [`Longitude`], and [`Latitude`].

use core::fmt;

/// An angular quantity measured in degrees.
///
/// `Angle` is intentionally neutral: it does not assume a normalization range.
/// Use [`Angle::normalized_0_360`] or [`Angle::normalized_signed`] when a
/// canonical wrap is required.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Angle(f64);

impl Angle {
    /// Creates a new angle measured in degrees.
    pub const fn from_degrees(degrees: f64) -> Self {
        Self(degrees)
    }

    /// Creates a new angle measured in radians.
    pub fn from_radians(radians: f64) -> Self {
        Self(radians.to_degrees())
    }

    /// Returns the underlying angle in degrees.
    pub const fn degrees(self) -> f64 {
        self.0
    }

    /// Returns the angle in radians.
    pub fn radians(self) -> f64 {
        self.0.to_radians()
    }

    /// Returns the angle normalized into the half-open range `[0, 360)`.
    pub fn normalized_0_360(self) -> Self {
        Self(self.0.rem_euclid(360.0))
    }

    /// Returns the angle normalized into the signed range `[-180, 180)`.
    pub fn normalized_signed(self) -> Self {
        let degrees = self.normalized_0_360().degrees();
        if degrees >= 180.0 {
            Self(degrees - 360.0)
        } else {
            Self(degrees)
        }
    }

    /// Returns `true` when the underlying numeric value is finite.
    pub const fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.0)
    }
}

/// A canonical ecliptic or longitude-like angle normalized into `[0, 360)`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Longitude(Angle);

impl Longitude {
    /// Creates a longitude normalized into `[0, 360)`.
    pub fn from_degrees(degrees: f64) -> Self {
        Self(Angle::from_degrees(degrees).normalized_0_360())
    }

    /// Returns the longitude in degrees, already normalized into `[0, 360)`.
    pub const fn degrees(self) -> f64 {
        self.0.degrees()
    }

    /// Returns the underlying angle wrapper.
    pub const fn angle(self) -> Angle {
        self.0
    }
}

impl From<Angle> for Longitude {
    fn from(value: Angle) -> Self {
        Self::from_degrees(value.degrees())
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A signed latitude-like angle measured in degrees, north-positive.
///
/// Positive values lie north of the equator (or the relevant reference plane)
/// and negative values lie south. Latitude values are not automatically
/// clamped; the caller is expected to provide values consistent with the
/// relevant coordinate system.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Latitude(Angle);

impl Latitude {
    /// Creates a latitude measured in degrees.
    pub const fn from_degrees(degrees: f64) -> Self {
        Self(Angle::from_degrees(degrees))
    }

    /// Returns the latitude in degrees.
    pub const fn degrees(self) -> f64 {
        self.0.degrees()
    }

    /// Returns the underlying angle wrapper.
    pub const fn angle(self) -> Angle {
        self.0
    }
}

impl From<Angle> for Latitude {
    fn from(value: Angle) -> Self {
        Self::from_degrees(value.degrees())
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
