//! Shared release-profile identifiers for release-facing outputs.
//!
//! The workspace exposes the compatibility-profile and API-stability posture
//! identifiers separately, but several release-facing tools need both at once.
//! Keeping the pair in one tiny helper reduces the chance that a future refactor
//! updates one identifier path while forgetting the other.

#![forbid(unsafe_code)]

use core::fmt;

use super::{current_api_stability_profile_id, current_compatibility_profile_id};

/// The identifiers that name the current release-facing profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReleaseProfileIdentifiers {
    /// Identifier for the compatibility profile.
    pub compatibility_profile_id: &'static str,
    /// Identifier for the API-stability posture profile.
    pub api_stability_profile_id: &'static str,
}

impl ReleaseProfileIdentifiers {
    /// Returns the compact summary used in release-facing reports.
    pub fn summary_line(&self) -> String {
        format!(
            "compatibility={}, api-stability={}",
            self.compatibility_profile_id, self.api_stability_profile_id
        )
    }

    /// Returns `Ok(())` when the identifiers still match the current release posture.
    pub fn validate(&self) -> Result<(), ReleaseProfileIdentifiersValidationError> {
        if self.compatibility_profile_id != current_compatibility_profile_id() {
            return Err(ReleaseProfileIdentifiersValidationError::CompatibilityProfileIdOutOfSync);
        }
        if self.api_stability_profile_id != current_api_stability_profile_id() {
            return Err(ReleaseProfileIdentifiersValidationError::ApiStabilityProfileIdOutOfSync);
        }

        Ok(())
    }
}

/// Validation error for a release-profile identifier pair that drifted away from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseProfileIdentifiersValidationError {
    /// The compatibility profile identifier no longer matches the current release posture.
    CompatibilityProfileIdOutOfSync,
    /// The API-stability posture identifier no longer matches the current release posture.
    ApiStabilityProfileIdOutOfSync,
}

impl fmt::Display for ReleaseProfileIdentifiersValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompatibilityProfileIdOutOfSync => {
                f.write_str("release-profile compatibility identifier is out of sync")
            }
            Self::ApiStabilityProfileIdOutOfSync => {
                f.write_str("release-profile API-stability identifier is out of sync")
            }
        }
    }
}

impl std::error::Error for ReleaseProfileIdentifiersValidationError {}

impl fmt::Display for ReleaseProfileIdentifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
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
        assert!(identifiers.validate().is_ok());
    }

    #[test]
    fn identifiers_render_as_a_compact_pair_summary() {
        let identifiers = current_release_profile_identifiers();
        let expected = format!(
            "compatibility={}, api-stability={}",
            current_compatibility_profile_id(),
            current_api_stability_profile_id()
        );

        assert_eq!(identifiers.summary_line(), expected);
        assert_eq!(identifiers.to_string(), expected);
    }

    #[test]
    fn validation_rejects_drifted_identifiers() {
        let compatibility_drift = ReleaseProfileIdentifiers {
            compatibility_profile_id: "drifted-compatibility",
            api_stability_profile_id: current_api_stability_profile_id(),
        };
        assert_eq!(
            compatibility_drift.validate().unwrap_err(),
            ReleaseProfileIdentifiersValidationError::CompatibilityProfileIdOutOfSync
        );

        let api_stability_drift = ReleaseProfileIdentifiers {
            compatibility_profile_id: current_compatibility_profile_id(),
            api_stability_profile_id: "drifted-api",
        };
        assert_eq!(
            api_stability_drift.validate().unwrap_err(),
            ReleaseProfileIdentifiersValidationError::ApiStabilityProfileIdOutOfSync
        );
    }
}
