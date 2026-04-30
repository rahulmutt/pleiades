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
///
/// # Example
///
/// ```
/// use pleiades_core::{current_release_profile_identifiers, ReleaseProfileIdentifiers};
///
/// let identifiers = current_release_profile_identifiers();
/// assert_eq!(ReleaseProfileIdentifiers::schema_version(), 1);
/// assert!(identifiers.validate().is_ok());
/// assert!(identifiers.summary_line().starts_with("v1 compatibility="));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReleaseProfileIdentifiers {
    /// Identifier for the compatibility profile.
    pub compatibility_profile_id: &'static str,
    /// Identifier for the API-stability posture profile.
    pub api_stability_profile_id: &'static str,
}

impl ReleaseProfileIdentifiers {
    /// Schema version for the compact release-profile identifier payload.
    pub const fn schema_version() -> u8 {
        1
    }

    /// Returns the compact summary used in release-facing reports.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::current_release_profile_identifiers;
    ///
    /// let identifiers = current_release_profile_identifiers();
    /// assert!(identifiers.summary_line().contains("api-stability="));
    /// ```
    pub fn summary_line(&self) -> String {
        format!(
            "v{} compatibility={}, api-stability={}",
            Self::schema_version(),
            self.compatibility_profile_id,
            self.api_stability_profile_id
        )
    }

    /// Returns `Ok(())` when the identifiers still match the current release posture.
    pub fn validate(&self) -> Result<(), ReleaseProfileIdentifiersValidationError> {
        validate_release_profile_identifiers(
            self.compatibility_profile_id,
            self.api_stability_profile_id,
        )
    }
}

/// Validation error for a release-profile identifier pair that drifted away from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseProfileIdentifiersValidationError {
    /// The compatibility profile and API-stability posture identifiers unexpectedly match.
    IdentifiersAreNotDistinct,
    /// The compatibility profile identifier no longer matches the current release posture.
    CompatibilityProfileIdOutOfSync,
    /// The API-stability posture identifier no longer matches the current release posture.
    ApiStabilityProfileIdOutOfSync,
}

impl fmt::Display for ReleaseProfileIdentifiersValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifiersAreNotDistinct => {
                f.write_str("release-profile identifiers must be distinct")
            }
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

fn contains_line_breaks(value: &str) -> bool {
    value
        .chars()
        .any(|character| character == '\n' || character == '\r')
}

fn validate_release_profile_identifiers(
    compatibility_profile_id: &str,
    api_stability_profile_id: &str,
) -> Result<(), ReleaseProfileIdentifiersValidationError> {
    if compatibility_profile_id.trim().is_empty()
        || compatibility_profile_id.trim() != compatibility_profile_id
        || contains_line_breaks(compatibility_profile_id)
    {
        return Err(ReleaseProfileIdentifiersValidationError::CompatibilityProfileIdOutOfSync);
    }

    if api_stability_profile_id.trim().is_empty()
        || api_stability_profile_id.trim() != api_stability_profile_id
        || contains_line_breaks(api_stability_profile_id)
    {
        return Err(ReleaseProfileIdentifiersValidationError::ApiStabilityProfileIdOutOfSync);
    }

    if compatibility_profile_id == api_stability_profile_id {
        return Err(ReleaseProfileIdentifiersValidationError::IdentifiersAreNotDistinct);
    }

    if compatibility_profile_id != current_compatibility_profile_id() {
        return Err(ReleaseProfileIdentifiersValidationError::CompatibilityProfileIdOutOfSync);
    }
    if api_stability_profile_id != current_api_stability_profile_id() {
        return Err(ReleaseProfileIdentifiersValidationError::ApiStabilityProfileIdOutOfSync);
    }

    Ok(())
}

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
    fn identifiers_render_as_a_versioned_compact_pair_summary() {
        let identifiers = current_release_profile_identifiers();
        let expected = format!(
            "v{} compatibility={}, api-stability={}",
            ReleaseProfileIdentifiers::schema_version(),
            current_compatibility_profile_id(),
            current_api_stability_profile_id()
        );

        assert_eq!(ReleaseProfileIdentifiers::schema_version(), 1);
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

    #[test]
    fn validation_rejects_matching_profile_identifiers() {
        let error = validate_release_profile_identifiers(
            current_compatibility_profile_id(),
            current_compatibility_profile_id(),
        )
        .unwrap_err();

        assert_eq!(
            error,
            ReleaseProfileIdentifiersValidationError::IdentifiersAreNotDistinct
        );
        assert_eq!(
            error.to_string(),
            "release-profile identifiers must be distinct"
        );
    }

    #[test]
    fn validation_rejects_blank_whitespace_padded_or_multiline_identifiers() {
        let compatibility_blank = ReleaseProfileIdentifiers {
            compatibility_profile_id: "",
            api_stability_profile_id: current_api_stability_profile_id(),
        };
        assert_eq!(
            compatibility_blank.validate().unwrap_err(),
            ReleaseProfileIdentifiersValidationError::CompatibilityProfileIdOutOfSync
        );

        let api_stability_padded = ReleaseProfileIdentifiers {
            compatibility_profile_id: current_compatibility_profile_id(),
            api_stability_profile_id: " pleiades-api-stability/0.1.0 ",
        };
        assert_eq!(
            api_stability_padded.validate().unwrap_err(),
            ReleaseProfileIdentifiersValidationError::ApiStabilityProfileIdOutOfSync
        );

        let compatibility_multiline = ReleaseProfileIdentifiers {
            compatibility_profile_id: "pleiades-compatibility/0.1.0\nrelease",
            api_stability_profile_id: current_api_stability_profile_id(),
        };
        assert_eq!(
            compatibility_multiline.validate().unwrap_err(),
            ReleaseProfileIdentifiersValidationError::CompatibilityProfileIdOutOfSync
        );

        let api_stability_multiline = ReleaseProfileIdentifiers {
            compatibility_profile_id: current_compatibility_profile_id(),
            api_stability_profile_id: "pleiades-api-stability/0.1.0\r\nrelease",
        };
        assert_eq!(
            api_stability_multiline.validate().unwrap_err(),
            ReleaseProfileIdentifiersValidationError::ApiStabilityProfileIdOutOfSync
        );
    }
}
