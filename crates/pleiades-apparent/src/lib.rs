//! Apparent-place corrections: light-time, precession-to-date, annual
//! aberration, and nutation-in-longitude, with typed provenance.

mod error;

pub use error::{ApparentLightTimeError, ApparentPlaceError};

pub mod nutation;

pub use nutation::Nutation;

pub mod aberration;

pub use aberration::AberrationOffset;

pub mod lighttime;

pub use lighttime::{LightTimePosition, LIGHT_TIME_DAYS_PER_AU};

pub mod precession;

pub use precession::{precess_ecliptic_j2000_to_date, PrecessedEcliptic};

mod provenance;

pub use provenance::{ApparentProvenance, CorrectionSet, MODEL_SOURCES};

mod apparent;

pub use apparent::{apparent_position, ApparentPosition, DEFAULT_MAX_ITERATIONS};

pub mod policy;

pub use policy::{
    ApparentPlacePolicySummary, ApparentPlacePolicySummaryValidationError,
    CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT,
};

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
