//! House-system catalog definitions and compatibility metadata.
//!
//! This crate currently focuses on the catalog layer and the first chart MVP
//! house-placement helpers: it enumerates the baseline built-in house systems,
//! their common aliases, and a few notes about latitude-sensitive behavior, and
//! it now exposes a small calculation path for the simpler baseline systems.
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

/// Returns the baseline built-in house-system catalog.
pub const fn baseline_house_systems() -> &'static [HouseSystemDescriptor] {
    BASELINE_HOUSE_SYSTEMS
}

/// Finds the descriptor for a typed house-system selection.
pub fn descriptor(system: &HouseSystem) -> Option<&'static HouseSystemDescriptor> {
    BASELINE_HOUSE_SYSTEMS
        .iter()
        .find(|entry| entry.system == *system)
}

/// Resolves a house-system label to a built-in type.
pub fn resolve_house_system(label: &str) -> Option<HouseSystem> {
    BASELINE_HOUSE_SYSTEMS
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
    }
}
