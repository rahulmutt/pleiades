//! Per-entry compatibility claim tier shared by the house and ayanamsa catalogs.
#![forbid(unsafe_code)]

/// The compatibility claim a catalog makes for one built-in entry.
///
/// Two tiers only — no ambiguous middle state. `ReleaseGradeNumeric` asserts the
/// entry is exercised by an SE numeric gate against a corpus-validated ceiling
/// and passes it; `DescriptorOnly` asserts catalogue/metadata presence with no
/// numeric compatibility claim.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityClaimTier {
    /// Numeric compatibility is asserted and backed by passing gate evidence.
    ReleaseGradeNumeric,
    /// Catalogued with metadata/aliases only; no numeric compatibility claim.
    DescriptorOnly,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_are_distinct() {
        assert_ne!(
            CompatibilityClaimTier::ReleaseGradeNumeric,
            CompatibilityClaimTier::DescriptorOnly
        );
    }
}
