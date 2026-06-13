//! Zodiac mode and sign types: [`ZodiacMode`] and [`ZodiacSign`].

use core::fmt;

use crate::angles::Longitude;
use crate::ayanamsa::Ayanamsa;

/// Whether coordinates should be interpreted in tropical or sidereal mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ZodiacMode {
    /// Tropical zodiac.
    Tropical,
    /// Sidereal zodiac using the selected ayanamsa definition.
    Sidereal { ayanamsa: Ayanamsa },
}

impl fmt::Display for ZodiacMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tropical => f.write_str("Tropical"),
            Self::Sidereal { ayanamsa } => write!(f, "Sidereal ({ayanamsa})"),
        }
    }
}

/// One of the twelve zodiac signs.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ZodiacSign {
    /// Aries 0°–30°.
    Aries,
    /// Taurus 30°–60°.
    Taurus,
    /// Gemini 60°–90°.
    Gemini,
    /// Cancer 90°–120°.
    Cancer,
    /// Leo 120°–150°.
    Leo,
    /// Virgo 150°–180°.
    Virgo,
    /// Libra 180°–210°.
    Libra,
    /// Scorpio 210°–240°.
    Scorpio,
    /// Sagittarius 240°–270°.
    Sagittarius,
    /// Capricorn 270°–300°.
    Capricorn,
    /// Aquarius 300°–330°.
    Aquarius,
    /// Pisces 330°–360°.
    Pisces,
}

impl ZodiacSign {
    /// Returns the sign corresponding to a normalized ecliptic longitude.
    pub fn from_longitude(longitude: Longitude) -> Self {
        match (longitude.degrees() / 30.0).floor() as usize % 12 {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            _ => Self::Pisces,
        }
    }

    /// Returns the sign's display name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aries => "Aries",
            Self::Taurus => "Taurus",
            Self::Gemini => "Gemini",
            Self::Cancer => "Cancer",
            Self::Leo => "Leo",
            Self::Virgo => "Virgo",
            Self::Libra => "Libra",
            Self::Scorpio => "Scorpio",
            Self::Sagittarius => "Sagittarius",
            Self::Capricorn => "Capricorn",
            Self::Aquarius => "Aquarius",
            Self::Pisces => "Pisces",
        }
    }
}

impl fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
