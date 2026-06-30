//! Shared primitive and domain-adjacent types used across the workspace.
//!
//! These types define the vocabulary for angles, time scales, celestial body
//! identifiers, observer locations, coordinate frames, and catalog selections.
//! Higher-level crates build on these semantics without re-labelling the same
//! concepts in backend-specific ways.
//!
//! Enable the optional `serde` feature to serialize and deserialize the public
//! type vocabulary for interchange or caching workflows.
//!
//! # Examples
//!
//! ```
//! use pleiades_types::{Angle, Longitude};
//!
//! let angle = Angle::from_degrees(-30.0);
//! assert_eq!(angle.normalized_0_360().degrees(), 330.0);
//!
//! let lon = Longitude::from_degrees(390.0);
//! assert_eq!(lon.degrees(), 30.0);
//! ```

#![forbid(unsafe_code)]

mod angles;
mod ayanamsa;
mod bodies;
mod compatibility_claim;
mod coordinates;
mod custom_bodies;
mod frames;
mod house_systems;
mod motion;
mod observer;
mod time;
mod time_range;
mod zodiac;

pub use angles::{Angle, Latitude, Longitude};
pub use ayanamsa::{Ayanamsa, CustomAyanamsa};
pub use bodies::{CelestialBody, CelestialBodyClass};
pub use compatibility_claim::CompatibilityClaimTier;
pub use coordinates::{CoordinateValidationError, EclipticCoordinates, EquatorialCoordinates};
pub use custom_bodies::{CustomBodyId, CustomDefinitionValidationError};
pub use frames::{Apparentness, CoordinateFrame};
pub use house_systems::{CustomHouseSystem, HouseSystem};
pub use motion::{Motion, MotionDirection, MotionValidationError};
pub use observer::{ObserverLocation, ObserverLocationValidationError};
pub use time::{
    Instant, JulianDay, TimeScale, TimeScaleConversion, TimeScaleConversionError,
    OBLIQUITY_J2000_DEG, SECONDS_PER_DAY,
};
pub use time_range::{TimeRange, TimeRangeValidationError};
pub use zodiac::{ZodiacMode, ZodiacSign};

#[cfg(test)]
mod tests;
