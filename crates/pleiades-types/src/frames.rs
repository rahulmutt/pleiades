//! Coordinate frame and apparentness selectors: [`CoordinateFrame`] and [`Apparentness`].

use core::fmt;

/// The coordinate frame requested from a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CoordinateFrame {
    /// Ecliptic longitude/latitude coordinates.
    Ecliptic,
    /// Equatorial right ascension/declination coordinates.
    Equatorial,
}

impl fmt::Display for CoordinateFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Ecliptic => "Ecliptic",
            Self::Equatorial => "Equatorial",
        };
        f.write_str(label)
    }
}

/// Whether a backend should prefer apparent or mean values where both exist.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Apparentness {
    /// Apparent values, including light-time and related corrections when available.
    Apparent,
    /// Mean values.
    Mean,
}

impl fmt::Display for Apparentness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Apparent => "Apparent",
            Self::Mean => "Mean",
        };
        f.write_str(label)
    }
}
