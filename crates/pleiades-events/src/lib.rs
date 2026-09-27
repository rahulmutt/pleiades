//! Ephemeris event-finding for the `pleiades` workspace: longitude crossings of
//! the Sun, Moon, and planets, derived from pleiades' validated body positions.
//!
//! The engine is generic over any [`pleiades_backend::EphemerisBackend`] and,
//! like `pleiades-eclipse`, works in TDB over the 1900–2100 CE packaged window.
//! Every returned event instant is TDB, tagged `TimeScale::Tdb`. The
//! observer-local surfaces (rise/set/transit, horizontal coordinates) read the
//! `TimeScale` tag on their query instants and evaluate Earth rotation at the
//! UT1 re-expression of each sampled day; see [`EventEngine::next_rise_set`].
//!
//! Lunar occultations of a planet (Mercury–Pluto) or a curated fixed star are
//! covered by [`EventEngine::occultation`] (local circumstances),
//! [`EventEngine::next_occultation`]/[`EventEngine::previous_occultation`]
//! (per-observer search), and [`EventEngine::next_global_occultation`]
//! (global search for the central-observation point).
//!
//! ## Example
//!
//! ```rust
//! use pleiades_data::packaged_backend;
//! use pleiades_events::{CrossingFrame, EventEngine};
//! use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};
//!
//! let engine = EventEngine::new(packaged_backend());
//! let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
//! // When does the Sun next reach 0° (the March equinox point)?
//! let next = engine
//!     .next_sun_crossing(Longitude::from_degrees(0.0), after)
//!     .unwrap();
//! assert!(next.is_some());
//! ```
//!
//! ```rust
//! // When does the Sun next rise for a mid-latitude observer?
//! use pleiades_data::packaged_backend;
//! use pleiades_events::{EventEngine, RiseSetEvent, RiseSetOptions, RiseSetTarget};
//! use pleiades_apparent::Atmosphere;
//! use pleiades_types::{
//!     CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
//! };
//!
//! let engine = EventEngine::new(packaged_backend());
//! let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(-74.0), None);
//! let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
//! let rise = engine
//!     .next_rise_set(
//!         RiseSetTarget::Body(CelestialBody::Sun),
//!         RiseSetEvent::Rise,
//!         obs,
//!         Atmosphere::default(),
//!         RiseSetOptions::default(),
//!         after,
//!     )
//!     .unwrap();
//! assert!(rise.is_some());
//! ```
//!
//! ```rust
//! // Which sunrise began the Hindu civil day containing an instant? That is
//! // the last sunrise at or before it: `previous_rise_set`.
//! use pleiades_data::packaged_backend;
//! use pleiades_events::{EventEngine, RiseSetEvent, RiseSetOptions, RiseSetTarget};
//! use pleiades_apparent::Atmosphere;
//! use pleiades_types::{
//!     CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
//! };
//!
//! let engine = EventEngine::new(packaged_backend());
//! let chennai = ObserverLocation::new(
//!     Latitude::from_degrees(13.0827),
//!     Longitude::from_degrees(80.2707),
//!     None,
//! );
//! let birth = Instant::new(JulianDay::from_days(2_458_849.9), TimeScale::Tdb);
//! let day_start = engine
//!     .previous_rise_set(
//!         RiseSetTarget::Body(CelestialBody::Sun),
//!         RiseSetEvent::Rise,
//!         chennai,
//!         Atmosphere::default(),
//!         RiseSetOptions { hindu: true, ..RiseSetOptions::default() },
//!         birth,
//!     )
//!     .unwrap()
//!     .expect("the Sun rises daily in Chennai");
//! assert!(day_start.instant.julian_day.days() <= birth.julian_day.days());
//! assert!(day_start.instant.julian_day.days() > birth.julian_day.days() - 1.0);
//! ```
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod crossings;
mod ephemeris;
mod error;
mod fixstar;
mod horizontal;
mod magnitude;
mod mean_elements;
mod nod_aps;
mod occult;
mod pheno;
mod rise_trans;
mod root;
mod semidiameter;
mod time_scale;

#[allow(deprecated)]
pub use crossings::{Crossing, CrossingEngine, CrossingFrame, EventEngine};
pub use error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
pub use fixstar::{fixed_star_apparent, fixed_star_entry, FixedStarEntry};
pub use horizontal::{Horizontal, HorizontalInput};
pub use nod_aps::{ApsisConvention, NodApsMethod, NodApsPoint, NodesApsides};
pub use occult::{
    GlobalOccultation, LocalOccultation, OccultStageDiagnostics, OccultTarget, OccultationContact,
    OccultationType,
};
pub use pheno::PhenoData;
pub use rise_trans::{DiscMode, RiseSet, RiseSetEvent, RiseSetOptions, RiseSetTarget};
