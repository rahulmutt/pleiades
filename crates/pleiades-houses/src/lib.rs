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
//!
//! House-system selection: pick a system (here Placidus), compute cusps from an
//! [`Instant`](pleiades_types::Instant) plus
//! [`ObserverLocation`](pleiades_types::ObserverLocation), and read the exposed
//! AscMc chart points.
//!
//! ```
//! use pleiades_houses::{calculate_houses, HouseRequest};
//! use pleiades_types::{HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
//!
//! // London, mid-latitude, at J2000.0 (TT).
//! let request = HouseRequest::new(
//!     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
//!     ObserverLocation::new(
//!         Latitude::from_degrees(51.4769),
//!         Longitude::from_degrees(-0.0005),
//!         None,
//!     ),
//!     HouseSystem::Placidus,
//! );
//! let houses = calculate_houses(&request).expect("house calculation should work");
//!
//! // Every house system yields the full ring of twelve cusps.
//! assert_eq!(houses.cusps.len(), 12);
//!
//! // The AscMc chart points are exposed alongside the cusps. The descendant and
//! // imum coeli sit opposite the ascendant and midheaven, respectively.
//! let asc_mc = houses.asc_mc;
//! let opposite = |deg: f64| (deg + 180.0) % 360.0;
//! assert!((asc_mc.descendant.degrees() - opposite(asc_mc.ascendant.degrees())).abs() < 1e-6);
//! assert!((asc_mc.imum_coeli.degrees() - opposite(asc_mc.midheaven.degrees())).abs() < 1e-6);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod catalog;
mod error;
mod systems;
pub mod thresholds;

// Re-export the public surface from the catalog module.
pub use catalog::{
    baseline_house_systems, built_in_house_systems, descriptor, house_catalog_validation_summary,
    house_formula_families, house_system_code_aliases, latitude_sensitive_house_failure_modes,
    release_house_systems, resolve_house_system, validate_house_catalog,
    validate_house_system_code_aliases, HouseCatalogValidationError, HouseCatalogValidationSummary,
    HouseFormulaFamily, HouseSystemCodeAlias, HouseSystemCodeAliasValidationError,
    HouseSystemDescriptor,
};

// Re-export the public surface from the error module.
pub use error::{HouseError, HouseErrorKind};

// Re-export the public surface from the systems module.
pub use systems::{
    calculate_houses, chart_points, chart_points_from_armc, house_for_longitude, AscMc,
    HighLatitudePolicy, HouseAngles, HouseRequest, HouseSnapshot,
};
