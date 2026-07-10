//! Civil-time conversion: Gregorian calendar, leap seconds, Delta-T, and
//! TT/TDB output with typed provenance.
//!
//! # Examples
//!
//! Convert a civil UTC datetime to Terrestrial Time. Every result carries a
//! tiered quality marker — [`ConversionQuality::Exact`] for leap-second-exact
//! UTC (1972 onward), [`Observed`](ConversionQuality::Observed) from the Delta-T
//! table, or [`Predicted`](ConversionQuality::Predicted) from Delta-T
//! extrapolation — so a modelled offset is never mistaken for an exact one.
//! Sidereal time is then taken from the UT1 Julian day recovered via Delta-T
//! (sidereal time is a function of Earth rotation, i.e. UT1, not TT):
//!
//! ```
//! use pleiades_time::{
//!     gmst_degrees, tt_from_utc_civil, ut1_jd_from_tt, CivilDateTime, ConversionQuality,
//! };
//! use pleiades_types::TimeScale;
//!
//! // 2000-01-01 12:00:00 UTC -> TT. UTC from 1972 onward is leap-second-exact.
//! let civil = CivilDateTime::new(2000, 1, 1, 12, 0, 0.0);
//! let tt = tt_from_utc_civil(civil).expect("inside the 1900-2100 support window");
//! assert_eq!(tt.instant.scale, TimeScale::Tt);
//! assert_eq!(tt.provenance.quality, ConversionQuality::Exact);
//! assert_eq!(tt.provenance.tai_minus_utc, Some(32)); // TAI - UTC at 2000-01-01
//!
//! // Greenwich mean sidereal time (degrees, normalized to [0, 360)).
//! let jd_ut1 = ut1_jd_from_tt(tt.instant.julian_day.days()).expect("Delta-T available");
//! let gmst = gmst_degrees(jd_ut1);
//! assert!((0.0..360.0).contains(&gmst));
//! ```
#![deny(missing_docs)]

mod calendar;
mod convert;
pub mod deltat;
mod error;
pub mod leap;
pub mod sidereal;
pub mod tdb;

pub use calendar::CivilDateTime;
pub use convert::{
    tdb_from_ut1_civil, tdb_from_utc_civil, to_terrestrial, tt_from_ut1_civil, tt_from_utc_civil,
    ut1_jd_from_tt, CivilInstant, ConversionPath, ConversionProvenance, ConversionQuality,
    SUPPORT_END_JD, SUPPORT_START_JD,
};
pub use deltat::DeltaTQuality;
pub use error::CivilTimeError;
pub use sidereal::{gmst_degrees, gmst_degrees_raw};

/// Deterministic 64-bit content checksum (FNV-1a), byte-identical to
/// `pleiades_jpl::spk::corpus_manifest::corpus_checksum64`. Used to detect drift
/// between a checked-in data table and its pinned checksum. Not cryptographic.
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
