//! Global geocentric solar and lunar eclipse computation for the `pleiades`
//! workspace, derived entirely from pleiades' validated Sun and Moon positions.
//!
//! ## Scope
//!
//! - **Window:** 1900-01-01 (JD 2 415 020.5 TDB) through 2100-01-01
//!   (JD 2 488 069.5 TDB), bounded by the packaged ephemeris data. Four
//!   NASA-canon eclipses falling in mid/late 2100 are uncomputable with the
//!   packaged data and are excluded.
//! - **Coverage:** global / geocentric, plus per-observer local circumstances
//!   (contact times, magnitude/obscuration, az/alt, visibility) via
//!   [`EclipseEngine::local_circumstances`] and `next/previous_local_eclipse`.
//! - **Outputs per eclipse:** type, instant of greatest eclipse, magnitude,
//!   gamma, Saros series, eclipsed longitude (apparent tropical ecliptic of
//!   date; no ayanamsa), and (solar only) geographic location of greatest
//!   eclipse. Lunar eclipses have no greatest-eclipse location.
//! - **Validation:** the fail-closed `validate-eclipses` gate recomputes every
//!   in-window NASA-canon eclipse and compares against ≤ 60 s (time),
//!   ≤ 0.01 (magnitude), exact type, exact Saros, and ≤ 1.0″ (eclipsed
//!   longitude). Of the 909 in-window NASA-canon rows, 908 pass all five
//!   tolerances; one documented knife-edge eclipse (1948-05-09, Saros 137,
//!   annular vs hybrid at magnitude ≈ 1.0) is allowlisted for the exact-type
//!   check only (its four other tolerances are still verified).
//!
//! ## Example
//!
//! ```rust
//! use pleiades_data::packaged_backend;
//! use pleiades_eclipse::{EclipseEngine, EclipseFilter};
//! use pleiades_types::{Instant, JulianDay, TimeScale};
//!
//! let engine = EclipseEngine::new(packaged_backend());
//! let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
//! let next = engine.next_eclipse(after, EclipseFilter::All).unwrap();
//! assert!(next.is_some());
//! ```
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod engine;
mod ephemeris;
mod error;
mod geometry;
mod local;
mod saros;
mod syzygy;
mod types;

pub use engine::EclipseEngine;
pub use error::{EclipseError, WINDOW_END_JD, WINDOW_START_JD};
pub use local::{
    LocalCircumstances, LocalContact, LocalLunarCircumstances, LocalSolarCircumstances,
};
pub use types::{
    Eclipse, EclipseFilter, EclipseKind, EclipseType, GeoLocation, LunarEclipseType, Node,
    SolarEclipseType,
};
