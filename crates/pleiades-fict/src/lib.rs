//! Fictitious/hypothetical bodies from osculating orbital elements.
//!
//! Computes the Swiss-Ephemeris default `seorbel.txt` fictitious body set as an
//! unperturbed Kepler orbit, rotated to the J2000 mean ecliptic and assembled to
//! a geocentric place. Definitional: parity with SE, gated by `validate-fictitious`.

pub mod elements;
pub mod frame;
pub mod kepler;

/// Crate/backend identifier used in backend metadata and results.
pub const PACKAGE_NAME: &str = "pleiades-fict";

/// Julian Day (TT) of the J2000.0 epoch.
pub const J2000_JD: f64 = 2_451_545.0;
