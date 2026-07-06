//! Celestial body identifiers: [`CelestialBodyClass`] and [`CelestialBody`].

use core::fmt;

use crate::custom_bodies::{CustomBodyId, CustomDefinitionValidationError};

/// A coarse classification for a celestial body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CelestialBodyClass {
    /// The Sun or Moon.
    Luminary,
    /// Mercury through Pluto.
    MajorPlanet,
    /// Lunar nodes and apsides.
    LunarPoint,
    /// Ceres, Pallas, Juno, and Vesta.
    BuiltInAsteroid,
    /// A hypothetical/fictitious body defined by osculating orbital elements
    /// (Uranian planets, Transpluto, Vulcan, historical pre-discovery predictions).
    Fictitious,
    /// A structured custom body identifier.
    Custom,
}

impl CelestialBodyClass {
    /// Returns a stable human-readable label for the class.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "luminary",
            Self::MajorPlanet => "major planet",
            Self::LunarPoint => "lunar point",
            Self::BuiltInAsteroid => "built-in asteroid",
            Self::Fictitious => "fictitious body",
            Self::Custom => "custom body",
        }
    }
}

impl fmt::Display for CelestialBodyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// The built-in and custom body identifiers recognized by the shared API.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CelestialBody {
    /// The Sun.
    Sun,
    /// The Moon.
    Moon,
    /// Mercury.
    Mercury,
    /// Venus.
    Venus,
    /// Mars.
    Mars,
    /// Jupiter.
    Jupiter,
    /// Saturn.
    Saturn,
    /// Uranus.
    Uranus,
    /// Neptune.
    Neptune,
    /// Pluto.
    Pluto,
    /// The mean lunar node.
    MeanNode,
    /// The true lunar node.
    TrueNode,
    /// The mean lunar apogee.
    MeanApogee,
    /// The true lunar apogee.
    TrueApogee,
    /// The mean lunar perigee.
    MeanPerigee,
    /// The true lunar perigee.
    TruePerigee,
    /// Ceres.
    Ceres,
    /// Pallas.
    Pallas,
    /// Juno.
    Juno,
    /// Vesta.
    Vesta,
    /// Cupido (Uranian/Hamburg, Witte) — SE fictitious body 40.
    Cupido,
    /// Hades (Uranian/Hamburg, Witte) — SE 41.
    Hades,
    /// Zeus (Uranian/Hamburg, Sieggrün) — SE 42.
    Zeus,
    /// Kronos (Uranian/Hamburg, Sieggrün) — SE 43.
    Kronos,
    /// Apollon (Uranian/Hamburg, Sieggrün) — SE 44.
    Apollon,
    /// Admetos (Uranian/Hamburg, Sieggrün) — SE 45.
    Admetos,
    /// Vulkanus (Uranian/Hamburg, Sieggrün) — SE 46.
    Vulkanus,
    /// Poseidon (Uranian/Hamburg, Sieggrün) — SE 47.
    Poseidon,
    /// Isis / Transpluto — SE 48.
    Transpluto,
    /// Nibiru — SE 49.
    Nibiru,
    /// Harrington — SE 50.
    Harrington,
    /// Neptune (Leverrier historical prediction) — SE 51.
    NeptuneLeverrier,
    /// Neptune (Adams historical prediction) — SE 52.
    NeptuneAdams,
    /// Pluto (Lowell historical prediction) — SE 53.
    PlutoLowell,
    /// Pluto (Pickering historical prediction) — SE 54.
    PlutoPickering,
    /// Vulcan (intramercurial) — SE 55.
    Vulcan,
    /// White Moon / Selena (geocentric orbit) — SE 56.
    WhiteMoon,
    /// Proserpina — SE 57.
    Proserpina,
    /// Waldemath (hypothetical second Earth moon, geocentric orbit) — SE 58.
    Waldemath,
    /// A body that is not yet one of the built-in identifiers.
    Custom(CustomBodyId),
}

impl CelestialBody {
    /// Returns the coarse class for this body.
    pub const fn class(&self) -> CelestialBodyClass {
        match self {
            Self::Sun | Self::Moon => CelestialBodyClass::Luminary,
            Self::Mercury
            | Self::Venus
            | Self::Mars
            | Self::Jupiter
            | Self::Saturn
            | Self::Uranus
            | Self::Neptune
            | Self::Pluto => CelestialBodyClass::MajorPlanet,
            Self::MeanNode
            | Self::TrueNode
            | Self::MeanApogee
            | Self::TrueApogee
            | Self::MeanPerigee
            | Self::TruePerigee => CelestialBodyClass::LunarPoint,
            Self::Ceres | Self::Pallas | Self::Juno | Self::Vesta => {
                CelestialBodyClass::BuiltInAsteroid
            }
            Self::Cupido
            | Self::Hades
            | Self::Zeus
            | Self::Kronos
            | Self::Apollon
            | Self::Admetos
            | Self::Vulkanus
            | Self::Poseidon
            | Self::Transpluto
            | Self::Nibiru
            | Self::Harrington
            | Self::NeptuneLeverrier
            | Self::NeptuneAdams
            | Self::PlutoLowell
            | Self::PlutoPickering
            | Self::Vulcan
            | Self::WhiteMoon
            | Self::Proserpina
            | Self::Waldemath => CelestialBodyClass::Fictitious,
            Self::Custom(_) => CelestialBodyClass::Custom,
        }
    }

    /// Returns a stable human-readable name for built-in bodies.
    pub const fn built_in_name(&self) -> Option<&'static str> {
        match self {
            Self::Sun => Some("Sun"),
            Self::Moon => Some("Moon"),
            Self::Mercury => Some("Mercury"),
            Self::Venus => Some("Venus"),
            Self::Mars => Some("Mars"),
            Self::Jupiter => Some("Jupiter"),
            Self::Saturn => Some("Saturn"),
            Self::Uranus => Some("Uranus"),
            Self::Neptune => Some("Neptune"),
            Self::Pluto => Some("Pluto"),
            Self::MeanNode => Some("Mean Node"),
            Self::TrueNode => Some("True Node"),
            Self::MeanApogee => Some("Mean Apogee"),
            Self::TrueApogee => Some("True Apogee"),
            Self::MeanPerigee => Some("Mean Perigee"),
            Self::TruePerigee => Some("True Perigee"),
            Self::Ceres => Some("Ceres"),
            Self::Pallas => Some("Pallas"),
            Self::Juno => Some("Juno"),
            Self::Vesta => Some("Vesta"),
            Self::Cupido => Some("Cupido"),
            Self::Hades => Some("Hades"),
            Self::Zeus => Some("Zeus"),
            Self::Kronos => Some("Kronos"),
            Self::Apollon => Some("Apollon"),
            Self::Admetos => Some("Admetos"),
            Self::Vulkanus => Some("Vulkanus"),
            Self::Poseidon => Some("Poseidon"),
            Self::Transpluto => Some("Transpluto"),
            Self::Nibiru => Some("Nibiru"),
            Self::Harrington => Some("Harrington"),
            Self::NeptuneLeverrier => Some("Neptune (Leverrier)"),
            Self::NeptuneAdams => Some("Neptune (Adams)"),
            Self::PlutoLowell => Some("Pluto (Lowell)"),
            Self::PlutoPickering => Some("Pluto (Pickering)"),
            Self::Vulcan => Some("Vulcan"),
            Self::WhiteMoon => Some("White Moon"),
            Self::Proserpina => Some("Proserpina"),
            Self::Waldemath => Some("Waldemath"),
            Self::Custom(_) => None,
        }
    }

    /// Validates the custom body identifier when this is a custom body.
    ///
    /// Built-in bodies are always valid; only the structured custom body
    /// identifier is checked. This keeps user-defined catalog entries from
    /// drifting into request or profile surfaces with blank or padded fields.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate(),
            _ => Ok(()),
        }
    }
}

impl fmt::Display for CelestialBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sun => f.write_str("Sun"),
            Self::Moon => f.write_str("Moon"),
            Self::Mercury => f.write_str("Mercury"),
            Self::Venus => f.write_str("Venus"),
            Self::Mars => f.write_str("Mars"),
            Self::Jupiter => f.write_str("Jupiter"),
            Self::Saturn => f.write_str("Saturn"),
            Self::Uranus => f.write_str("Uranus"),
            Self::Neptune => f.write_str("Neptune"),
            Self::Pluto => f.write_str("Pluto"),
            Self::MeanNode => f.write_str("Mean Node"),
            Self::TrueNode => f.write_str("True Node"),
            Self::MeanApogee => f.write_str("Mean Apogee"),
            Self::TrueApogee => f.write_str("True Apogee"),
            Self::MeanPerigee => f.write_str("Mean Perigee"),
            Self::TruePerigee => f.write_str("True Perigee"),
            Self::Ceres => f.write_str("Ceres"),
            Self::Pallas => f.write_str("Pallas"),
            Self::Juno => f.write_str("Juno"),
            Self::Vesta => f.write_str("Vesta"),
            Self::Cupido => f.write_str("Cupido"),
            Self::Hades => f.write_str("Hades"),
            Self::Zeus => f.write_str("Zeus"),
            Self::Kronos => f.write_str("Kronos"),
            Self::Apollon => f.write_str("Apollon"),
            Self::Admetos => f.write_str("Admetos"),
            Self::Vulkanus => f.write_str("Vulkanus"),
            Self::Poseidon => f.write_str("Poseidon"),
            Self::Transpluto => f.write_str("Transpluto"),
            Self::Nibiru => f.write_str("Nibiru"),
            Self::Harrington => f.write_str("Harrington"),
            Self::NeptuneLeverrier => f.write_str("Neptune (Leverrier)"),
            Self::NeptuneAdams => f.write_str("Neptune (Adams)"),
            Self::PlutoLowell => f.write_str("Pluto (Lowell)"),
            Self::PlutoPickering => f.write_str("Pluto (Pickering)"),
            Self::Vulcan => f.write_str("Vulcan"),
            Self::WhiteMoon => f.write_str("White Moon"),
            Self::Proserpina => f.write_str("Proserpina"),
            Self::Waldemath => f.write_str("Waldemath"),
            Self::Custom(custom) => fmt::Display::fmt(custom, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fictitious_bodies_map_to_fictitious_class() {
        for body in [
            CelestialBody::Cupido,
            CelestialBody::Transpluto,
            CelestialBody::Vulcan,
            CelestialBody::WhiteMoon,
            CelestialBody::Waldemath,
        ] {
            assert_eq!(body.class(), CelestialBodyClass::Fictitious);
            assert!(body.built_in_name().is_some());
            assert!(!body.to_string().is_empty());
        }
        assert_eq!(CelestialBody::Transpluto.built_in_name(), Some("Transpluto"));
        assert_eq!(CelestialBody::WhiteMoon.to_string(), "White Moon");
    }
}
