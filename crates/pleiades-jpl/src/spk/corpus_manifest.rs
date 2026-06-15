//! Corpus manifest: provenance plus per-slice FNV content checksums and row
//! counts, rendered in the repo's manifest-text style and parsed back.

/// Deterministic 64-bit content checksum (FNV-1a) used to detect drift between
/// a checked-in slice file and the manifest. Not cryptographic; pairs with the
/// gated regenerate-and-value-compare path for kernel-identity assurance.
///
/// Kept byte-identical to `reference_summary::...::checksum64`; if you change
/// one, change both.
pub fn corpus_checksum64(text: &str) -> u64 {
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
    fn checksum_is_deterministic_and_sensitive() {
        assert_eq!(corpus_checksum64("abc"), corpus_checksum64("abc"));
        assert_ne!(corpus_checksum64("abc"), corpus_checksum64("abd"));
    }
}
