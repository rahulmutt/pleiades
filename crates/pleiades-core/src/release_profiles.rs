//! Shared release-profile identifiers for release-facing outputs.
//!
//! The workspace exposes the compatibility-profile and API-stability posture
//! identifiers separately, but several release-facing tools need both at once.
//! Keeping the pair in one tiny helper reduces the chance that a future refactor
//! updates one identifier path while forgetting the other.

#![forbid(unsafe_code)]

use super::{current_api_stability_profile_id, current_compatibility_profile_id};

/// The identifiers that name the current release-facing profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReleaseProfileIdentifiers {
    /// Identifier for the compatibility profile.
    pub compatibility_profile_id: &'static str,
    /// Identifier for the API-stability posture profile.
    pub api_stability_profile_id: &'static str,
}

/// Returns the current release-profile identifiers.
pub const fn current_release_profile_identifiers() -> ReleaseProfileIdentifiers {
    ReleaseProfileIdentifiers {
        compatibility_profile_id: current_compatibility_profile_id(),
        api_stability_profile_id: current_api_stability_profile_id(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_match_the_individual_profile_accessors() {
        let identifiers = current_release_profile_identifiers();
        assert_eq!(
            identifiers.compatibility_profile_id,
            current_compatibility_profile_id()
        );
        assert_eq!(
            identifiers.api_stability_profile_id,
            current_api_stability_profile_id()
        );
        assert_ne!(
            identifiers.compatibility_profile_id,
            identifiers.api_stability_profile_id
        );
    }
}
