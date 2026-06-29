//! Global geocentric solar and lunar eclipse computation, derived entirely from
//! pleiades' validated Sun and Moon positions. Scope: 1900–2100 CE, geocentric
//! circumstances only (no per-observer local circumstances).
#![forbid(unsafe_code)]

mod ephemeris;
mod error;
mod types;

pub use error::{EclipseError, WINDOW_END_JD, WINDOW_START_JD};
pub use types::{
    Eclipse, EclipseFilter, EclipseKind, EclipseType, GeoLocation, LunarEclipseType, Node,
    SolarEclipseType,
};
