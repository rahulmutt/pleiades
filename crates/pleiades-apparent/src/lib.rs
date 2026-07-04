//! Apparent-place corrections: light-time, precession-to-date, annual
//! aberration, and nutation-in-longitude, with typed provenance.
//! Gravitational light-deflection and atmospheric refraction are omitted.
//!
//! # Examples
//!
//! Turn a mean J2000 ecliptic position into an apparent place of date with
//! [`apparent_position`]. The `query` closure returns the body's mean/J2000
//! geocentric position at a (light-time-retarded) instant; the routine applies
//! light-time, precession to the equinox of date, annual aberration, and
//! nutation-in-longitude. Gravitational light-deflection and atmospheric
//! refraction are **not** applied. One century after J2000 precession alone
//! shifts ecliptic longitude by roughly 1.4°:
//!
//! ```
//! use pleiades_apparent::{apparent_position, ApparentPlaceError, DEFAULT_MAX_ITERATIONS};
//! use pleiades_types::{EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, TimeScale};
//!
//! let instant = Instant::new(JulianDay::from_days(2_451_545.0 + 36_525.0), TimeScale::Tt);
//! let mean_lon = 100.0;
//! let out = apparent_position::<_, ApparentPlaceError>(
//!     instant,
//!     280.0, // Sun's true longitude of date, of-date, for the aberration term
//!     DEFAULT_MAX_ITERATIONS,
//!     |_| {
//!         Ok(EclipticCoordinates::new(
//!             Longitude::from_degrees(mean_lon),
//!             Latitude::from_degrees(0.0),
//!             Some(1.0),
//!         ))
//!     },
//! )
//! .expect("apparent place");
//! let shift_deg = out.ecliptic.longitude.degrees() - mean_lon;
//! assert!((shift_deg - 1.397).abs() < 0.05, "apparent-vs-mean shift {shift_deg} deg");
//! assert!(out.provenance.corrections.nutation_longitude);
//! ```
//!
//! Apply the topocentric correction (diurnal parallax + diurnal aberration) to a
//! geocentric apparent place with [`topocentric_position`]. For the Moon the
//! diurnal parallax is large (of order 1°):
//!
//! ```
//! use pleiades_apparent::topocentric_position;
//! use pleiades_types::{EclipticCoordinates, Latitude, Longitude, ObserverLocation};
//!
//! let geocentric = EclipticCoordinates::new(
//!     Longitude::from_degrees(100.0),
//!     Latitude::from_degrees(0.0),
//!     Some(0.002_57), // Moon distance, in AU
//! );
//! let observer = ObserverLocation::new(
//!     Latitude::from_degrees(0.0),
//!     Longitude::from_degrees(0.0),
//!     Some(0.0),
//! );
//! // local apparent sidereal time = 100°, true obliquity of date ≈ 23.4°.
//! let topo = topocentric_position(geocentric, &observer, 100.0, 23.4).expect("topocentric");
//! let shift_deg = topo
//!     .provenance
//!     .parallax_longitude_arcsec
//!     .hypot(topo.provenance.parallax_latitude_arcsec)
//!     / 3600.0;
//! assert!(shift_deg > 0.3, "Moon parallax {shift_deg} deg too small");
//! ```

#![deny(missing_docs)]

mod error;

pub use error::{ApparentLightTimeError, ApparentPlaceError};

pub mod nutation;

pub use nutation::Nutation;

pub mod equatorial;

pub use equatorial::{apparent_equatorial_of_date, true_obliquity_degrees};

pub mod sidereal;

pub use sidereal::{
    equation_of_equinoxes, equation_of_equinoxes_degrees, greenwich_mean_sidereal_time_degrees,
    sidereal_time, SiderealTime,
};

pub mod aberration;

pub use aberration::AberrationOffset;

pub mod lighttime;

pub use lighttime::{LightTimePosition, LIGHT_TIME_DAYS_PER_AU};

pub mod precession;

pub use precession::{
    precess_ecliptic_date_to_j2000, precess_ecliptic_j2000_to_date, PrecessedEcliptic,
};

pub mod parallax;

pub use parallax::{ObserverGeocentric, AU_IN_EARTH_RADII};

mod provenance;

pub use provenance::{ApparentProvenance, CorrectionSet, TopocentricProvenance, MODEL_SOURCES};

mod apparent;

pub use apparent::{
    apparent_apsis_position, apparent_position, apparent_sun_position, ApparentPosition,
    DEFAULT_MAX_ITERATIONS,
};

mod topocentric;

pub use topocentric::{topocentric_position, TopocentricPosition, DIURNAL_ABERRATION_ARCSEC};

pub mod policy;

pub use policy::{
    ApparentPlacePolicySummary, ApparentPlacePolicySummaryValidationError,
    CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT,
};

pub mod refraction;

pub use refraction::{apparent_from_true, true_from_apparent, Atmosphere};

/// Deterministic 64-bit content checksum (FNV-1a), byte-identical to
/// `pleiades_time::fnv1a64`. Detects drift between a checked-in data table and
/// its pinned checksum. Not cryptographic.
pub fn fnv1a64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_is_deterministic_and_sensitive() {
        assert_eq!(fnv1a64("abc"), fnv1a64("abc"));
        assert_ne!(fnv1a64("abc"), fnv1a64("abd"));
    }
}
