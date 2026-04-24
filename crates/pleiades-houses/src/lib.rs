//! House-system catalog definitions and compatibility metadata.
//!
//! This crate focuses on the catalog layer and the first chart MVP house-
//! placement helpers: it enumerates the baseline built-in house systems, their
//! common aliases, latitude-sensitive notes, and the Stage 3 baseline house
//! formulas that power the chart workflow. It also carries the first Stage 6
//! compatibility-expansion additions so release profiles can distinguish the
//! baseline milestone from newer catalog breadth. The resolver additionally
//! accepts the common Swiss Ephemeris house-system letter codes used by
//! interoperability tables.
//!
//! # Examples
//!
//! ```
//! use pleiades_houses::{baseline_house_systems, resolve_house_system};
//!
//! let systems = baseline_house_systems();
//! assert!(systems.iter().any(|entry| entry.canonical_name == "Placidus"));
//!
//! assert_eq!(resolve_house_system("Polich-Page"), Some(pleiades_types::HouseSystem::Topocentric));
//! ```
//!
//! ```
//! use pleiades_houses::{calculate_houses, HouseRequest};
//! use pleiades_types::{HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
//!
//! let request = HouseRequest::new(
//!     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
//!     ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), None),
//!     HouseSystem::WholeSign,
//! );
//! let houses = calculate_houses(&request).expect("house calculation should work");
//! assert_eq!(houses.cusps.len(), 12);
//! ```

#![forbid(unsafe_code)]

mod houses;

pub use houses::{
    calculate_houses, house_for_longitude, HouseAngles, HouseError, HouseErrorKind, HouseRequest,
    HouseSnapshot,
};

use pleiades_types::HouseSystem;

/// A catalog entry for a built-in house system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseSystemDescriptor {
    /// The strongly typed system identifier.
    pub system: HouseSystem,
    /// The canonical name used in compatibility profiles.
    pub canonical_name: &'static str,
    /// Alternate names or software-specific aliases.
    pub aliases: &'static [&'static str],
    /// Short notes about formula family or interoperability constraints.
    pub notes: &'static str,
    /// Whether the system is known to have latitude-sensitive failure modes.
    pub latitude_sensitive: bool,
}

impl HouseSystemDescriptor {
    /// Creates a new descriptor.
    pub const fn new(
        system: HouseSystem,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        latitude_sensitive: bool,
    ) -> Self {
        Self {
            system,
            canonical_name,
            aliases,
            notes,
            latitude_sensitive,
        }
    }

    /// Returns `true` if the provided label matches the canonical name or one
    /// of the documented aliases.
    pub fn matches_label(&self, label: &str) -> bool {
        self.canonical_name.eq_ignore_ascii_case(label)
            || self
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(label))
    }
}

const BASELINE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
    HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system", "Placidus table of houses"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["Koch houses", "Koch house system", "house system of the birth place", "Koch table of houses", "W. Koch", "W Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Porphyry,
        "Porphyry",
        &[
            "Equal Quadrant",
            "Porphyry house system",
            "Porphyry table of houses",
        ],
        "Simple quadrant division used as a robust fallback.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &[
            "Regiomontanus houses",
            "Regiomontanus house system",
            "Regiomontanus table of houses",
        ],
        "Classical quadrant system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Campanus,
        "Campanus",
        &[
            "Campanus houses",
            "Campanus house system",
            "Campanus table of houses",
        ],
        "Great-circle division system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Equal",
        &[
            "A equal",
            "E equal = A",
            "Equal houses",
            "Equal house system",
            "Equal House",
            "Equal table of houses",
            "Wang",
            "Equal (cusp 1 = Asc)",
        ],
        "Equal-house system anchored on the ascendant; Wang and the Swiss Ephemeris \"Equal (cusp 1 = Asc)\" label are treated as interoperability aliases for the equal-house-from-Ascendant convention.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::WholeSign,
        "Whole Sign",
        &[
            "W equal, whole sign",
            "Whole Sign houses",
            "Whole Sign table of houses",
            "Whole-sign",
            "Whole Sign system",
            "Whole Sign house system",
        ],
        "Whole-sign system anchored on the rising sign.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &[
            "Alcabitius houses",
            "Alcabitius house system",
            "Alcabitius table of houses",
        ],
        "Classical semi-arc family system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Meridian,
        "Meridian",
        &[
            "Meridian houses",
            "Meridian table of houses",
            "Meridian house system",
            "ARMC",
            "Axial Rotation",
            "Axial rotation system",
            "Zariel",
            "X axial rotation system/ Meridian houses",
        ],
        "Meridian-style systems and documented axial variants.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants", "A"],
        "Documented axial variants used by some astrology packages.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Topocentric,
        "Topocentric",
        &[
            "Polich-Page",
            "Polich/Page",
            "Polich Page",
            "Polich-Page \"topocentric\" table of houses",
            "T Polich/Page (\"topocentric\")",
            "T topocentric",
            "Topocentric house system",
            "Topocentric table of houses",
        ],
        "Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses", "Morinus house system"],
        "Morinus house system with historical interoperability value.",
        false,
    ),
];

const RELEASE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
    HouseSystemDescriptor::new(
        HouseSystem::EqualMidheaven,
        "Equal (MC)",
        &[
            "D equal / MC",
            "Equal from MC",
            "Equal (from MC)",
            "Equal (from MC) table of houses",
            "Equal (MC) table of houses",
            "Equal/MC table of houses",
            "Equal (MC) house system",
            "Equal MC",
            "Equal/MC",
            "Equal Midheaven",
            "Equal Midheaven house system",
            "Equal Midheaven table of houses",
            "Equal/MC = 10th",
        ],
        "Equal houses anchored at the Midheaven instead of the Ascendant.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::EqualAries,
        "Equal (1=Aries)",
        &[
            "N",
            "Equal/1=Aries",
            "Equal Aries",
            "Aries houses",
            "Whole Sign (house 1 = Aries)",
            "Whole Sign (house 1 = Aries) table of houses",
            "Equal (1=Aries) table of houses",
            "Equal/1=Aries table of houses",
            "Equal (1=Aries) house system",
            "N whole sign houses, 1. house = Aries",
            "Whole sign houses, 1. house = Aries",
            "Equal/1=0 Aries",
            "Equal (cusp 1 = 0° Aries)",
        ],
        "Fixed zodiac-sign houses anchored at 0° Aries.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Vehlow,
        "Vehlow Equal",
        &[
            "V equal Vehlow",
            "Vehlow",
            "Vehlow equal",
            "Vehlow house system",
            "Vehlow Equal house system",
            "Vehlow-equal",
            "Vehlow-equal table of houses",
            "Vehlow Equal table of houses",
        ],
        "Equal-house variant with the Ascendant centered in house 1.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sripati,
        "Sripati",
        &["S sripati", "Śrīpati", "Sripati house system", "Sripati table of houses"],
        "Midpoint variant of the Porphyry quadrants used in Jyotiṣa.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Carter,
        "Carter (poli-equatorial)",
        &[
            "Carter",
            "Carter's poli-equatorial",
            "Carter's poli-equatorial table of houses",
            "Poli-Equatorial",
            "Poli-equatorial",
        ],
        "Equal right-ascension segments anchored on the Ascendant's meridian.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Horizon,
        "Horizon/Azimuth",
        &[
            "Horizon",
            "Azimuth",
            "Horizontal",
            "Azimuthal",
            "Horizon house system",
            "Horizon/Azimuth house system",
            "Horizontal house system",
            "Azimuth house system",
            "Horizon/Azimuth table of houses",
            "Azimuthal house system",
            "horizon/azimut",
            "horizon/azimuth",
        ],
        "Azimuthal house system that anchors house 1 due East and house 10 at the MC.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Apc,
        "APC",
        &[
            "Ram school",
            "Ram's school",
            "Ramschool",
            "WvA",
            "Y APC houses",
            "APC houses",
            "APC, also known as “Ram school”, table of houses",
            "APC house system",
            "Ascendant Parallel Circle",
        ],
        "APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::KrusinskiPisaGoelzer,
        "Krusinski-Pisa-Goelzer",
        &[
            "Krusinski",
            "Krusinski-Pisa",
            "Krusinski Pisa",
            "Krusinski/Pisa/Goelzer",
            "Krusinski-Pisa-Goelzer table of houses",
            "U krusinski-pisa-goelzer",
            "Krusinski/Pisa/Goelzer house system",
            "Pisa-Goelzer",
        ],
        "Great-circle house system centered on the ascendant and zenith; latitude-sensitive near the poles.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Albategnius,
        "Albategnius",
        &["Savard-A", "Savard A", "Savard's Albategnius"],
        "Quartered latitude-circle variant associated with Savard's Albategnius proposal.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::PullenSd,
        "Pullen SD",
        &[
            "Neo-Porphyry",
            "Pullen (Sinusoidal Delta)",
            "Pullen sinusoidal delta",
            "Pullen SD (Neo-Porphyry) table of houses",
        ],
        "Sinusoidal-delta variant that smooths quadrant spacing toward the angles.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::PullenSr,
        "Pullen SR",
        &[
            "Pullen (Sinusoidal Ratio)",
            "Pullen sinusoidal ratio",
            "Pullen SR (Sinusoidal Ratio) table of houses",
        ],
        "Sinusoidal-ratio variant with ratio-derived house spacing.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sunshine,
        "Sunshine",
        &[
            "I sunshine",
            "Sunshine houses",
            "Sunshine house system",
            "Sunshine table of houses",
            "Sunshine table of houses, by Bob Makransky",
            "Makransky Sunshine",
            "Bob Makransky",
            "Treindl Sunshine",
        ],
        "Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Gauquelin,
        "Gauquelin sectors",
        &["G", "Gauquelin", "Gauquelin sector", "Gauquelin table of sectors"],
        "Thirty-six sectors used by the Gauquelin-sector family.",
        true,
    ),
];

static BUILT_IN_HOUSE_SYSTEMS: [HouseSystemDescriptor; 25] = [
    HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system", "Placidus table of houses"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["Koch houses", "Koch house system", "house system of the birth place", "Koch table of houses", "W. Koch", "W Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Porphyry,
        "Porphyry",
        &[
            "Equal Quadrant",
            "Porphyry house system",
            "Porphyry table of houses",
        ],
        "Simple quadrant division used as a robust fallback.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &[
            "Regiomontanus houses",
            "Regiomontanus house system",
            "Regiomontanus table of houses",
        ],
        "Classical quadrant system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Campanus,
        "Campanus",
        &[
            "Campanus houses",
            "Campanus house system",
            "Campanus table of houses",
        ],
        "Great-circle division system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Carter,
        "Carter (poli-equatorial)",
        &[
            "Carter",
            "Carter's poli-equatorial",
            "Carter's poli-equatorial table of houses",
            "Poli-Equatorial",
            "Poli-equatorial",
        ],
        "Equal right-ascension segments anchored on the Ascendant's meridian.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Horizon,
        "Horizon/Azimuth",
        &[
            "Horizon",
            "Azimuth",
            "Horizontal",
            "Azimuthal",
            "Horizon house system",
            "Horizon/Azimuth house system",
            "Horizontal house system",
            "Azimuth house system",
            "Horizon/Azimuth table of houses",
            "Azimuthal house system",
            "horizon/azimut",
            "horizon/azimuth",
        ],
        "Azimuthal house system that anchors house 1 due East and house 10 at the MC.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Apc,
        "APC",
        &[
            "Ram school",
            "Ram's school",
            "Ramschool",
            "WvA",
            "Y APC houses",
            "APC houses",
            "APC, also known as “Ram school”, table of houses",
            "APC house system",
            "Ascendant Parallel Circle",
        ],
        "APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::KrusinskiPisaGoelzer,
        "Krusinski-Pisa-Goelzer",
        &[
            "Krusinski",
            "Krusinski-Pisa",
            "Krusinski Pisa",
            "Krusinski/Pisa/Goelzer",
            "Krusinski-Pisa-Goelzer table of houses",
            "U krusinski-pisa-goelzer",
            "Krusinski/Pisa/Goelzer house system",
            "Pisa-Goelzer",
        ],
        "Great-circle house system centered on the ascendant and zenith; latitude-sensitive near the poles.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Albategnius,
        "Albategnius",
        &["Savard-A", "Savard A", "Savard's Albategnius"],
        "Quartered latitude-circle variant associated with Savard's Albategnius proposal.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::PullenSd,
        "Pullen SD",
        &[
            "Neo-Porphyry",
            "Pullen (Sinusoidal Delta)",
            "Pullen sinusoidal delta",
            "Pullen SD (Neo-Porphyry) table of houses",
        ],
        "Sinusoidal-delta variant that smooths quadrant spacing toward the angles.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::PullenSr,
        "Pullen SR",
        &[
            "Pullen (Sinusoidal Ratio)",
            "Pullen sinusoidal ratio",
            "Pullen SR (Sinusoidal Ratio) table of houses",
        ],
        "Sinusoidal-ratio variant with ratio-derived house spacing.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Equal",
        &[
            "A equal",
            "E equal = A",
            "Equal houses",
            "Equal house system",
            "Equal House",
            "Equal table of houses",
            "Wang",
            "Equal (cusp 1 = Asc)",
        ],
        "Equal-house system anchored on the ascendant; Wang and the Swiss Ephemeris \"Equal (cusp 1 = Asc)\" label are treated as interoperability aliases for the equal-house-from-Ascendant convention.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::WholeSign,
        "Whole Sign",
        &[
            "W equal, whole sign",
            "Whole Sign houses",
            "Whole Sign table of houses",
            "Whole-sign",
            "Whole Sign system",
            "Whole Sign house system",
        ],
        "Whole-sign system anchored on the rising sign.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &[
            "Alcabitius houses",
            "Alcabitius house system",
            "Alcabitius table of houses",
        ],
        "Classical semi-arc family system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Meridian,
        "Meridian",
        &[
            "Meridian houses",
            "Meridian table of houses",
            "Meridian house system",
            "ARMC",
            "Axial Rotation",
            "Axial rotation system",
            "Zariel",
            "X axial rotation system/ Meridian houses",
        ],
        "Meridian-style systems and documented axial variants.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants", "A"],
        "Documented axial variants used by some astrology packages.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Topocentric,
        "Topocentric",
        &[
            "Polich-Page",
            "Polich/Page",
            "Polich Page",
            "Polich-Page \"topocentric\" table of houses",
            "T Polich/Page (\"topocentric\")",
            "T topocentric",
            "Topocentric house system",
            "Topocentric table of houses",
        ],
        "Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses", "Morinus house system"],
        "Morinus house system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sunshine,
        "Sunshine",
        &[
            "I sunshine",
            "Sunshine houses",
            "Sunshine house system",
            "Sunshine table of houses",
            "Sunshine table of houses, by Bob Makransky",
            "Makransky Sunshine",
            "Bob Makransky",
            "Treindl Sunshine",
        ],
        "Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Gauquelin,
        "Gauquelin sectors",
        &["G", "Gauquelin", "Gauquelin sector", "Gauquelin table of sectors"],
        "Thirty-six sectors used by the Gauquelin-sector family.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::EqualMidheaven,
        "Equal (MC)",
        &[
            "D equal / MC",
            "Equal from MC",
            "Equal (from MC)",
            "Equal (from MC) table of houses",
            "Equal (MC) table of houses",
            "Equal/MC table of houses",
            "Equal (MC) house system",
            "Equal MC",
            "Equal/MC",
            "Equal Midheaven",
            "Equal Midheaven house system",
            "Equal Midheaven table of houses",
            "Equal/MC = 10th",
        ],
        "Equal houses anchored at the Midheaven instead of the Ascendant.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::EqualAries,
        "Equal (1=Aries)",
        &[
            "N",
            "Equal/1=Aries",
            "Equal Aries",
            "Aries houses",
            "Whole Sign (house 1 = Aries)",
            "Whole Sign (house 1 = Aries) table of houses",
            "Equal (1=Aries) table of houses",
            "Equal/1=Aries table of houses",
            "Equal (1=Aries) house system",
            "N whole sign houses, 1. house = Aries",
            "Whole sign houses, 1. house = Aries",
            "Equal/1=0 Aries",
            "Equal (cusp 1 = 0° Aries)",
        ],
        "Fixed zodiac-sign houses anchored at 0° Aries.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Vehlow,
        "Vehlow Equal",
        &[
            "V equal Vehlow",
            "Vehlow",
            "Vehlow equal",
            "Vehlow house system",
            "Vehlow Equal house system",
            "Vehlow-equal",
            "Vehlow-equal table of houses",
            "Vehlow Equal table of houses",
        ],
        "Equal-house variant with the Ascendant centered in house 1.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sripati,
        "Sripati",
        &["S sripati", "Śrīpati", "Sripati house system", "Sripati table of houses"],
        "Midpoint variant of the Porphyry quadrants used in Jyotiṣa.",
        false,
    ),
];

/// Returns the baseline built-in house-system catalog.
pub const fn baseline_house_systems() -> &'static [HouseSystemDescriptor] {
    BASELINE_HOUSE_SYSTEMS
}

/// Returns the release-specific house-system additions beyond the baseline milestone.
pub const fn release_house_systems() -> &'static [HouseSystemDescriptor] {
    RELEASE_HOUSE_SYSTEMS
}

/// Returns the full built-in house-system catalog shipped by this release line.
pub const fn built_in_house_systems() -> &'static [HouseSystemDescriptor] {
    &BUILT_IN_HOUSE_SYSTEMS
}

/// Finds the descriptor for a typed house-system selection.
pub fn descriptor(system: &HouseSystem) -> Option<&'static HouseSystemDescriptor> {
    built_in_house_systems()
        .iter()
        .find(|entry| entry.system == *system)
}

fn resolve_house_system_code(label: &str) -> Option<HouseSystem> {
    match label.trim() {
        "P" | "p" => Some(HouseSystem::Placidus),
        "K" | "k" => Some(HouseSystem::Koch),
        "R" | "r" => Some(HouseSystem::Regiomontanus),
        "C" | "c" => Some(HouseSystem::Campanus),
        "O" | "o" => Some(HouseSystem::Porphyry),
        "D" | "d" => Some(HouseSystem::EqualMidheaven),
        "E" | "e" => Some(HouseSystem::Equal),
        "W" | "w" => Some(HouseSystem::WholeSign),
        "V" | "v" => Some(HouseSystem::Vehlow),
        "A" | "a" => Some(HouseSystem::Axial),
        "H" | "h" => Some(HouseSystem::Horizon),
        "B" | "b" => Some(HouseSystem::Alcabitius),
        "M" | "m" => Some(HouseSystem::Morinus),
        "S" | "s" => Some(HouseSystem::Sripati),
        "I" | "i" => Some(HouseSystem::Sunshine),
        "G" | "g" => Some(HouseSystem::Gauquelin),
        "T" | "t" => Some(HouseSystem::Topocentric),
        "U" | "u" => Some(HouseSystem::KrusinskiPisaGoelzer),
        "Axial Rotation" | "axial rotation" | "Axial rotation system" | "axial rotation system" => {
            Some(HouseSystem::Meridian)
        }
        "X" | "x" => Some(HouseSystem::Meridian),
        "Y" | "y" => Some(HouseSystem::Apc),
        _ => None,
    }
}

/// Resolves a house-system label to a built-in type.
pub fn resolve_house_system(label: &str) -> Option<HouseSystem> {
    resolve_house_system_code(label).or_else(|| {
        built_in_house_systems()
            .iter()
            .find(|entry| entry.matches_label(label))
            .map(|entry| entry.system.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_catalog_includes_required_milestone_entries() {
        let names: Vec<_> = baseline_house_systems()
            .iter()
            .map(|entry| entry.canonical_name)
            .collect();

        for expected in [
            "Placidus",
            "Koch",
            "Porphyry",
            "Regiomontanus",
            "Campanus",
            "Equal",
            "Whole Sign",
            "Alcabitius",
            "Meridian",
            "Axial",
            "Topocentric",
            "Morinus",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn aliases_resolve_to_builtin_systems() {
        assert_eq!(
            resolve_house_system("Polich-Page"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Polich/Page"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Topocentric house system"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Topocentric table of houses"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Polich-Page \"topocentric\" table of houses"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Equal table of houses"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("Equal (from MC) table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal (MC) table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal/MC table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal (MC) house system"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Whole Sign table of houses"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Whole Sign (house 1 = Aries) table of houses"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (1=Aries) table of houses"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal/1=Aries table of houses"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (1=Aries) house system"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Vehlow-equal table of houses"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Vehlow Equal table of houses"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Carter's poli-equatorial table of houses"),
            Some(HouseSystem::Carter)
        );
        assert_eq!(
            resolve_house_system("Carter's poli-equatorial"),
            Some(HouseSystem::Carter)
        );
        assert_eq!(
            resolve_house_system("APC, also known as “Ram school”, table of houses"),
            Some(HouseSystem::Apc)
        );
        assert_eq!(
            resolve_house_system("Krusinski-Pisa-Goelzer table of houses"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Sunshine table of houses"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Sunshine table of houses, by Bob Makransky"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("I sunshine"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Gauquelin table of sectors"),
            Some(HouseSystem::Gauquelin)
        );
        assert_eq!(
            resolve_house_system("whole sign houses"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Whole Sign system"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Whole Sign house system"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Placidus table of houses"),
            Some(HouseSystem::Placidus)
        );
        assert_eq!(
            resolve_house_system("Koch table of houses"),
            Some(HouseSystem::Koch)
        );
        assert_eq!(resolve_house_system("w. koch"), Some(HouseSystem::Koch));
        assert_eq!(resolve_house_system("Koch houses"), Some(HouseSystem::Koch));
        assert_eq!(
            resolve_house_system("house system of the birth place"),
            Some(HouseSystem::Koch)
        );
        assert_eq!(resolve_house_system("W Koch"), Some(HouseSystem::Koch));
        assert_eq!(resolve_house_system("ARMC"), Some(HouseSystem::Meridian));
        assert_eq!(
            resolve_house_system("Axial Rotation"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(
            resolve_house_system("Axial rotation system"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(resolve_house_system("Zariel"), Some(HouseSystem::Meridian));
        assert_eq!(
            resolve_house_system("Meridian house system"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(resolve_house_system("D"), Some(HouseSystem::EqualMidheaven));
        assert_eq!(resolve_house_system("A equal"), Some(HouseSystem::Equal));
        assert_eq!(
            resolve_house_system("D equal / MC"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("E equal = A"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("W equal, whole sign"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("V equal Vehlow"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("X axial rotation system/ Meridian houses"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(resolve_house_system("Y APC houses"), Some(HouseSystem::Apc));
        assert_eq!(
            resolve_house_system("T Polich/Page (\"topocentric\")"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(resolve_house_system("P"), Some(HouseSystem::Placidus));
        assert_eq!(resolve_house_system("K"), Some(HouseSystem::Koch));
        assert_eq!(resolve_house_system("R"), Some(HouseSystem::Regiomontanus));
        assert_eq!(resolve_house_system("C"), Some(HouseSystem::Campanus));
        assert_eq!(resolve_house_system("O"), Some(HouseSystem::Porphyry));
        assert_eq!(resolve_house_system("E"), Some(HouseSystem::Equal));
        assert_eq!(resolve_house_system("W"), Some(HouseSystem::WholeSign));
        assert_eq!(resolve_house_system("N"), Some(HouseSystem::EqualAries));
        assert_eq!(resolve_house_system("V"), Some(HouseSystem::Vehlow));
        assert_eq!(resolve_house_system("A"), Some(HouseSystem::Axial));
        assert_eq!(resolve_house_system("H"), Some(HouseSystem::Horizon));
        assert_eq!(resolve_house_system("B"), Some(HouseSystem::Alcabitius));
        assert_eq!(resolve_house_system("M"), Some(HouseSystem::Morinus));
        assert_eq!(resolve_house_system("S"), Some(HouseSystem::Sripati));
        assert_eq!(resolve_house_system("I"), Some(HouseSystem::Sunshine));
        assert_eq!(resolve_house_system("G"), Some(HouseSystem::Gauquelin));
        assert_eq!(resolve_house_system("T"), Some(HouseSystem::Topocentric));
        assert_eq!(
            resolve_house_system("U"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(resolve_house_system("X"), Some(HouseSystem::Meridian));
        assert_eq!(resolve_house_system("Y"), Some(HouseSystem::Apc));
        assert_eq!(resolve_house_system("Carter"), Some(HouseSystem::Carter));
        assert_eq!(
            resolve_house_system("Carter's poli-equatorial"),
            Some(HouseSystem::Carter)
        );
        assert_eq!(
            resolve_house_system("T topocentric"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("U krusinski-pisa-goelzer"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Equal (from MC)"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal MC"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal/MC"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Midheaven"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Midheaven house system"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Midheaven table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal (MC)"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal/MC = 10th"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal/1=Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal/1=0 Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (cusp 1 = 0° Aries)"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(resolve_house_system("vehlow"), Some(HouseSystem::Vehlow));
        assert_eq!(
            resolve_house_system("Vehlow house system"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Vehlow Equal house system"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Vehlow-equal"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(resolve_house_system("Wang"), Some(HouseSystem::Equal));
        assert_eq!(
            resolve_house_system("Equal house system"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("Equal House"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("Whole Sign (house 1 = Aries)"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("N whole sign houses, 1. house = Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Whole sign houses, 1. house = Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (cusp 1 = Asc)"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(resolve_house_system("Azimuth"), Some(HouseSystem::Horizon));
        assert_eq!(
            resolve_house_system("Horizontal"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuthal"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizontal house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuth house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("horizon/azimuth"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("horizon/azimut"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(resolve_house_system("Ram school"), Some(HouseSystem::Apc));
        assert_eq!(resolve_house_system("Ram's school"), Some(HouseSystem::Apc));
        assert_eq!(
            resolve_house_system("APC house system"),
            Some(HouseSystem::Apc)
        );
        assert_eq!(resolve_house_system("WvA"), Some(HouseSystem::Apc));
        assert_eq!(
            resolve_house_system("Ascendant Parallel Circle"),
            Some(HouseSystem::Apc)
        );
        assert_eq!(
            resolve_house_system("Krusinski"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Krusinski/Pisa/Goelzer"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Krusinski/Pisa/Goelzer house system"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Horizon house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizon/Azimuth house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizontal house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuth house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizon/Azimuth table of houses"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuthal house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Sunshine house system"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(resolve_house_system("Śrīpati"), Some(HouseSystem::Sripati));
        assert_eq!(
            resolve_house_system("S sripati"),
            Some(HouseSystem::Sripati)
        );
        assert_eq!(
            resolve_house_system("Sripati house system"),
            Some(HouseSystem::Sripati)
        );
        assert_eq!(
            resolve_house_system("Sripati table of houses"),
            Some(HouseSystem::Sripati)
        );
        assert_eq!(
            resolve_house_system("Sunshine"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Bob Makransky"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Treindl Sunshine"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(resolve_house_system("G"), Some(HouseSystem::Gauquelin));
        assert_eq!(
            resolve_house_system("Gauquelin sectors"),
            Some(HouseSystem::Gauquelin)
        );
        assert_eq!(
            resolve_house_system("Savard-A"),
            Some(HouseSystem::Albategnius)
        );
        assert_eq!(
            resolve_house_system("Neo-Porphyry"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen (Sinusoidal Delta)"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen SD (Neo-Porphyry) table of houses"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen (Sinusoidal Ratio)"),
            Some(HouseSystem::PullenSr)
        );
        assert_eq!(
            resolve_house_system("Pullen sinusoidal ratio"),
            Some(HouseSystem::PullenSr)
        );
        assert_eq!(
            resolve_house_system("Pullen SR (Sinusoidal Ratio) table of houses"),
            Some(HouseSystem::PullenSr)
        );
    }

    #[test]
    fn release_additions_are_merged_into_the_built_in_catalog() {
        let names: Vec<_> = built_in_house_systems()
            .iter()
            .map(|entry| entry.canonical_name)
            .collect();

        for expected in [
            "Equal (MC)",
            "Equal (1=Aries)",
            "Vehlow Equal",
            "Sripati",
            "Carter (poli-equatorial)",
            "Horizon/Azimuth",
            "APC",
            "Krusinski-Pisa-Goelzer",
            "Albategnius",
            "Pullen SD",
            "Pullen SR",
            "Sunshine",
            "Gauquelin sectors",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn release_descriptor_aliases_do_not_repeat_canonical_labels() {
        assert!(built_in_house_systems()
            .iter()
            .all(|entry| { !entry.aliases.contains(&entry.canonical_name) }));
    }

    #[test]
    fn house_catalog_round_trips_all_built_ins_and_aliases() {
        use std::collections::HashSet;

        let built_in = built_in_house_systems();
        let mut unique_names = HashSet::new();

        assert_eq!(
            built_in.len(),
            baseline_house_systems().len() + release_house_systems().len()
        );

        for entry in baseline_house_systems()
            .iter()
            .chain(release_house_systems().iter())
        {
            assert!(
                unique_names.insert(entry.canonical_name),
                "duplicate canonical house-system name {}",
                entry.canonical_name
            );
            assert_eq!(
                descriptor(&entry.system).map(|d| d.canonical_name),
                Some(entry.canonical_name)
            );
            assert_eq!(
                resolve_house_system(entry.canonical_name),
                Some(entry.system.clone())
            );
            for alias in entry.aliases {
                assert_eq!(resolve_house_system(alias), Some(entry.system.clone()));
            }
        }

        for entry in built_in {
            assert!(unique_names.contains(entry.canonical_name));
        }
    }
}
