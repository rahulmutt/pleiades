//! House-system catalog definitions and compatibility metadata.
//!
//! This crate focuses on the catalog layer and the baseline chart house-
//! placement helpers: it enumerates the built-in house systems, their common
//! aliases, formula-family tags, latitude-sensitive notes, and the Stage 3
//! baseline house formulas that power the chart workflow. It also carries the first Stage 6
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

mod catalog;
mod error;
mod systems;
pub mod thresholds;

// Re-export the public surface from the catalog module.
pub use catalog::{
    baseline_house_systems, built_in_house_systems, descriptor, house_catalog_validation_summary,
    house_formula_families, house_formula_families_summary_line, house_system_code_aliases,
    house_system_code_aliases_summary_line, latitude_sensitive_house_failure_modes,
    latitude_sensitive_house_failure_modes_summary_line, release_house_systems,
    resolve_house_system, validate_house_catalog, validate_house_system_code_aliases,
    validated_house_system_code_aliases_summary_line, HouseCatalogValidationError,
    HouseCatalogValidationSummary, HouseFormulaFamily, HouseSystemCodeAlias,
    HouseSystemCodeAliasValidationError, HouseSystemDescriptor,
};

// Re-export the public surface from the error module.
pub use error::{HouseError, HouseErrorKind};

// Re-export the public surface from the systems module.
pub use systems::{
    calculate_houses, house_for_longitude, HighLatitudePolicy, HouseAngles, HouseRequest,
    HouseSnapshot,
};
