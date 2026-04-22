//! House-system catalog definitions and compatibility metadata.
//!
//! This crate focuses on the catalog layer and the first chart MVP house-
//! placement helpers: it enumerates the baseline built-in house systems, their
//! common aliases, latitude-sensitive notes, and the Stage 3 baseline house
//! formulas that power the chart workflow. It also carries the first Stage 6
//! compatibility-expansion additions so release profiles can distinguish the
//! baseline milestone from newer catalog breadth.
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
        &["Placidus house system"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["W. Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Porphyry,
        "Porphyry",
        &["Equal Quadrant"],
        "Simple quadrant division used as a robust fallback.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &["Regiomontanus houses"],
        "Classical quadrant system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Campanus,
        "Campanus",
        &["Campanus houses"],
        "Great-circle division system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Equal",
        &["Equal houses"],
        "Equal-house system anchored on the ascendant.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::WholeSign,
        "Whole Sign",
        &["Whole Sign houses", "Whole-sign"],
        "Whole-sign system anchored on the rising sign.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &["Alcabitius houses"],
        "Classical semi-arc family system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Meridian,
        "Meridian",
        &["Meridian houses"],
        "Meridian-style systems and documented axial variants.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants"],
        "Documented axial variants used by some astrology packages.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Topocentric,
        "Topocentric",
        &["Polich-Page", "Polich Page"],
        "Topocentric (Polich-Page) house system.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses"],
        "Morinus house system with historical interoperability value.",
        false,
    ),
];

const RELEASE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
    HouseSystemDescriptor::new(
        HouseSystem::EqualMidheaven,
        "Equal (MC)",
        &["Equal from MC", "Equal (from MC)"],
        "Equal houses anchored at the Midheaven instead of the Ascendant.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::EqualAries,
        "Equal (1=Aries)",
        &["Equal/1=Aries", "Equal Aries", "Aries houses"],
        "Fixed zodiac-sign houses anchored at 0° Aries.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Vehlow,
        "Vehlow Equal",
        &["Vehlow", "Vehlow equal"],
        "Equal-house variant with the Ascendant centered in house 1.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sripati,
        "Sripati",
        &["Śrīpati"],
        "Midpoint variant of the Porphyry quadrants used in Jyotiṣa.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Carter,
        "Carter (poli-equatorial)",
        &["Carter", "Poli-Equatorial", "Poli-equatorial"],
        "Equal right-ascension segments anchored on the Ascendant's meridian.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Horizon,
        "Horizon/Azimuth",
        &["Horizon", "Azimuth", "horizon/azimut"],
        "Azimuthal house system that anchors house 1 due East and house 10 at the MC.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Apc,
        "APC",
        &["Ram school", "Ramschool", "APC houses"],
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
        &["Neo-Porphyry", "Pullen sinusoidal delta"],
        "Sinusoidal-delta variant that smooths quadrant spacing toward the angles.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::PullenSr,
        "Pullen SR",
        &["Pullen sinusoidal ratio"],
        "Sinusoidal-ratio variant with ratio-derived house spacing.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sunshine,
        "Sunshine",
        &["Sunshine houses", "Makransky Sunshine", "Treindl Sunshine"],
        "Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.",
        true,
    ),
];

static BUILT_IN_HOUSE_SYSTEMS: [HouseSystemDescriptor; 24] = [
    HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["W. Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Porphyry,
        "Porphyry",
        &["Equal Quadrant"],
        "Simple quadrant division used as a robust fallback.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &["Regiomontanus houses"],
        "Classical quadrant system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Campanus,
        "Campanus",
        &["Campanus houses"],
        "Great-circle division system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Carter,
        "Carter (poli-equatorial)",
        &["Carter", "Poli-Equatorial", "Poli-equatorial"],
        "Equal right-ascension segments anchored on the Ascendant's meridian.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Horizon,
        "Horizon/Azimuth",
        &["Horizon", "Azimuth", "horizon/azimut"],
        "Azimuthal house system that anchors house 1 due East and house 10 at the MC.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Apc,
        "APC",
        &["Ram school", "Ramschool", "APC houses"],
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
        &["Neo-Porphyry", "Pullen sinusoidal delta"],
        "Sinusoidal-delta variant that smooths quadrant spacing toward the angles.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::PullenSr,
        "Pullen SR",
        &["Pullen sinusoidal ratio"],
        "Sinusoidal-ratio variant with ratio-derived house spacing.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Equal",
        &["Equal houses"],
        "Equal-house system anchored on the ascendant.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::WholeSign,
        "Whole Sign",
        &["Whole Sign houses", "Whole-sign"],
        "Whole-sign system anchored on the rising sign.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &["Alcabitius houses"],
        "Classical semi-arc family system.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Meridian,
        "Meridian",
        &["Meridian houses"],
        "Meridian-style systems and documented axial variants.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants"],
        "Documented axial variants used by some astrology packages.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Topocentric,
        "Topocentric",
        &["Polich-Page", "Polich Page"],
        "Topocentric (Polich-Page) house system.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses"],
        "Morinus house system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sunshine,
        "Sunshine",
        &["Sunshine houses", "Makransky Sunshine", "Treindl Sunshine"],
        "Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::EqualMidheaven,
        "Equal (MC)",
        &["Equal from MC", "Equal (from MC)"],
        "Equal houses anchored at the Midheaven instead of the Ascendant.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::EqualAries,
        "Equal (1=Aries)",
        &["Equal/1=Aries", "Equal Aries", "Aries houses"],
        "Fixed zodiac-sign houses anchored at 0° Aries.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Vehlow,
        "Vehlow Equal",
        &["Vehlow", "Vehlow equal"],
        "Equal-house variant with the Ascendant centered in house 1.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sripati,
        "Sripati",
        &["Śrīpati"],
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

/// Resolves a house-system label to a built-in type.
pub fn resolve_house_system(label: &str) -> Option<HouseSystem> {
    built_in_house_systems()
        .iter()
        .find(|entry| entry.matches_label(label))
        .map(|entry| entry.system.clone())
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
            resolve_house_system("whole sign houses"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(resolve_house_system("w. koch"), Some(HouseSystem::Koch));
        assert_eq!(resolve_house_system("Carter"), Some(HouseSystem::Carter));
        assert_eq!(
            resolve_house_system("Equal (from MC)"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(resolve_house_system("vehlow"), Some(HouseSystem::Vehlow));
        assert_eq!(resolve_house_system("Azimuth"), Some(HouseSystem::Horizon));
        assert_eq!(resolve_house_system("Ram school"), Some(HouseSystem::Apc));
        assert_eq!(
            resolve_house_system("Krusinski"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(resolve_house_system("Śrīpati"), Some(HouseSystem::Sripati));
        assert_eq!(
            resolve_house_system("Sunshine"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Treindl Sunshine"),
            Some(HouseSystem::Sunshine)
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
            resolve_house_system("Pullen sinusoidal ratio"),
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
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }
}
