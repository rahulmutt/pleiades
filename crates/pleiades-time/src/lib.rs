//! Civil-time conversion: Gregorian calendar, leap seconds, Delta-T, and
//! TT/TDB output with typed provenance.

mod calendar;
mod error;

pub use calendar::CivilDateTime;
pub use error::CivilTimeError;

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
