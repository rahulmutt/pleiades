//! Versioned API stability posture for the current release line.
//!
//! The release plan requires a clear statement of what consumers can treat as
//! stable versus what remains tooling- or release-internal. This profile keeps
//! that posture explicit without pretending that the entire workspace is a
//! frozen interchange format.

#![forbid(unsafe_code)]

use core::fmt;

/// The current API-stability posture identifier.
pub const CURRENT_API_STABILITY_PROFILE_ID: &str = "pleiades-api-stability/0.1.0";

/// Returns the current API-stability posture identifier.
pub const fn current_api_stability_profile_id() -> &'static str {
    CURRENT_API_STABILITY_PROFILE_ID
}

/// A release-scoped API stability profile.
#[derive(Clone, Copy, Debug)]
pub struct ApiStabilityProfile {
    /// Stable profile identifier.
    pub profile_id: &'static str,
    /// Human-readable summary of the current stability posture.
    pub summary: &'static str,
    /// Public surfaces that consumers can treat as stable.
    pub stable_surfaces: &'static [&'static str],
    /// Public surfaces that are documented but still allowed to evolve.
    pub experimental_surfaces: &'static [&'static str],
    /// Explicit policy for API deprecation and removal.
    pub deprecation_policy: &'static [&'static str],
    /// Intentional limitations that keep the façade thin and predictable.
    pub intentional_limits: &'static [&'static str],
}

impl ApiStabilityProfile {
    /// Returns a short stability note string.
    pub const fn stability_note(&self) -> &'static str {
        self.summary
    }

    /// Validates the profile's internal release-facing metadata.
    pub fn validate(&self) -> Result<(), ApiStabilityProfileValidationError> {
        validate_profile_identifier(self.profile_id)?;
        validate_profile_summary(self.summary)?;
        validate_text_section("stable surfaces", self.stable_surfaces)?;
        validate_text_section("experimental surfaces", self.experimental_surfaces)?;
        validate_text_section("deprecation policy", self.deprecation_policy)?;
        validate_text_section("intentional limits", self.intentional_limits)?;
        Ok(())
    }

    /// Returns a compact release-facing summary line for the stability posture.
    pub fn summary_line(&self) -> String {
        format!(
            "API stability posture: {}; stable surfaces: {}; experimental surfaces: {}; deprecation policy items: {}; intentional limits: {}",
            self.profile_id,
            self.stable_surfaces.len(),
            self.experimental_surfaces.len(),
            self.deprecation_policy.len(),
            self.intentional_limits.len()
        )
    }
}

/// A validation error emitted when the API stability profile's internal
/// release-facing metadata drifts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiStabilityProfileValidationError {
    /// The profile identifier is blank or whitespace-padded.
    BlankProfileIdentifier,
    /// The summary is blank or whitespace-padded.
    BlankSummary,
    /// A text section has no entries.
    EmptyTextSection {
        /// Section that failed validation.
        section_label: &'static str,
    },
    /// A text section contains a blank entry.
    BlankTextSectionEntry {
        /// Section that failed validation.
        section_label: &'static str,
    },
    /// A text section contains an entry with surrounding whitespace.
    WhitespaceTextSectionEntry {
        /// Section that failed validation.
        section_label: &'static str,
    },
}

impl fmt::Display for ApiStabilityProfileValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankProfileIdentifier => {
                f.write_str("API stability profile identifier is blank")
            }
            Self::BlankSummary => f.write_str("API stability summary is blank"),
            Self::EmptyTextSection { section_label } => {
                write!(f, "API stability {section_label} section is empty")
            }
            Self::BlankTextSectionEntry { section_label } => write!(
                f,
                "API stability {section_label} section contains a blank entry"
            ),
            Self::WhitespaceTextSectionEntry { section_label } => write!(
                f,
                "API stability {section_label} section contains an entry with surrounding whitespace"
            ),
        }
    }
}

impl std::error::Error for ApiStabilityProfileValidationError {}

fn validate_profile_identifier(
    profile_id: &'static str,
) -> Result<(), ApiStabilityProfileValidationError> {
    if profile_id.trim().is_empty() {
        return Err(ApiStabilityProfileValidationError::BlankProfileIdentifier);
    }

    if profile_id.trim() != profile_id {
        return Err(ApiStabilityProfileValidationError::BlankProfileIdentifier);
    }

    Ok(())
}

fn validate_profile_summary(
    summary: &'static str,
) -> Result<(), ApiStabilityProfileValidationError> {
    if summary.trim().is_empty() {
        return Err(ApiStabilityProfileValidationError::BlankSummary);
    }

    if summary.trim() != summary {
        return Err(ApiStabilityProfileValidationError::BlankSummary);
    }

    Ok(())
}

fn validate_text_section(
    section_label: &'static str,
    entries: &'static [&'static str],
) -> Result<(), ApiStabilityProfileValidationError> {
    if entries.is_empty() {
        return Err(ApiStabilityProfileValidationError::EmptyTextSection { section_label });
    }

    for entry in entries {
        if entry.trim().is_empty() {
            return Err(ApiStabilityProfileValidationError::BlankTextSectionEntry {
                section_label,
            });
        }

        if entry.trim() != *entry {
            return Err(
                ApiStabilityProfileValidationError::WhitespaceTextSectionEntry { section_label },
            );
        }
    }

    Ok(())
}

/// Returns the current API stability posture.
pub const fn current_api_stability_profile() -> ApiStabilityProfile {
    ApiStabilityProfile {
        profile_id: CURRENT_API_STABILITY_PROFILE_ID,
        summary: "The stable consumer surface is the shared domain model, backend contract, and chart/compatibility façade; validation and release-tooling formats are documented but still allowed to evolve as hardening continues. ChartSnapshot's summary_line helper, apparentness, direct, stationary, unknown-motion, retrograde, sign summary, dominant sign summary, house summary, dominant house summary, motion summary, and aspect summary helpers, plus the generic motion-direction placement filter, are part of that stable chart surface.",
        stable_surfaces: &[
            "pleiades-types defines the stable units, identifiers, and request/response primitives.",
            "pleiades-backend's EphemerisBackend trait and metadata model are the primary backend-facing contract.",
            "pleiades-core's ChartEngine, ChartRequest, ChartSnapshot, and compatibility-profile helpers are the stable façade used by consumers. ChartSnapshot's summary_line helper gives the chart façade a compact release-facing snapshot summary.",
            "ChartSnapshot exposes the apparentness used for backend position queries so mean/apparent chart mode stays explicit in reports and downstream consumers.",
            "ChartSnapshot body-placement helpers include direct lookup, sign lookup, house lookup, sign-scoped iteration, house-scoped iteration, motion-direction classification, direct, stationary, unknown-motion, and retrograde placement helpers, the placements_with_motion_direction filter, sign summaries, dominant sign summaries, house summaries, dominant house summaries, motion summaries, aspect summaries, retrograde summaries, and aspect helpers for backend motion data when present.",
            "House-system and ayanamsa resolution helpers are stable lookup surfaces for built-ins and custom entries.",
        ],
        experimental_surfaces: &[
            "pleiades-validate report text, release bundle layout, and validation-corpus composition remain operational tooling rather than a public interchange format.",
            "CLI command names and text formatting are documented but may evolve with the release tooling.",
            "Backend-specific helper modules remain backend-owned and may add convenience APIs without a compatibility promise.",
        ],
        deprecation_policy: &[
            "Breaking changes to stable public APIs require a major-version bump or a documented transition period.",
            "Additive changes should preserve source compatibility where practical.",
            "Experimental surfaces may change without deprecation, but changes should be called out in release notes.",
            "Deprecated APIs should stay available for at least one minor release unless a safety or correctness issue requires faster removal.",
        ],
        intentional_limits: &[
            "The façade stays thin; callers that need backend details are expected to use the backend trait directly.",
            "No stable promise is made about CLI output formatting beyond the documented commands and flags.",
            "Validation report text is intended for maintainers and release automation, not as a machine-stable interchange format.",
        ],
    }
}

fn write_section(f: &mut fmt::Formatter<'_>, title: &str, lines: &[&'static str]) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for line in lines {
        writeln!(f, "- {}", line)?;
    }
    Ok(())
}

impl fmt::Display for ApiStabilityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "API stability posture: {}", self.profile_id)?;
        writeln!(f, "{}", self.summary)?;
        writeln!(f)?;
        write_section(f, "Stable consumer surfaces:", self.stable_surfaces)?;
        writeln!(f)?;
        write_section(
            f,
            "Experimental or operational surfaces:",
            self.experimental_surfaces,
        )?;
        writeln!(f)?;
        write_section(f, "Deprecation policy:", self.deprecation_policy)?;
        writeln!(f)?;
        write_section(f, "Intentional limits:", self.intentional_limits)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_calls_out_stable_and_experimental_surfaces() {
        let profile = current_api_stability_profile();
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("ChartEngine")));
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("sign lookup")));
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("house-scoped iteration")));
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("sign summaries")));
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("house summaries")));
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("motion summaries")));
        assert!(profile.summary.contains("sign summary"));
        assert!(profile.summary.contains("dominant sign summary"));
        assert!(profile.summary.contains("house summary"));
        assert!(profile.summary.contains("dominant house summary"));
        assert!(profile.summary.contains("motion summary"));
        assert!(profile.summary.contains("apparentness"));
        assert!(profile
            .summary
            .contains("motion-direction placement filter"));
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("placements_with_motion_direction")));
        assert!(
            profile
                .stable_surfaces
                .iter()
                .any(|line| line.contains("unknown-motion placement helpers"))
                || profile.stable_surfaces.iter().any(|line| line.contains(
                    "direct, stationary, unknown-motion, and retrograde placement helpers"
                ))
        );
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("aspect summaries")));
        assert!(profile
            .stable_surfaces
            .iter()
            .any(|line| line.contains("aspect helpers")));
        assert!(profile
            .experimental_surfaces
            .iter()
            .any(|line| line.contains("validation-corpus")));
        assert!(profile
            .deprecation_policy
            .iter()
            .any(|line| line.contains("major-version bump")));
        assert!(profile
            .intentional_limits
            .iter()
            .any(|line| line.contains("Validation report text")));
        let summary_line = profile.summary_line();
        assert!(summary_line.contains("API stability posture:"));
        assert!(summary_line.contains(CURRENT_API_STABILITY_PROFILE_ID));
        assert!(summary_line.contains(&format!(
            "stable surfaces: {}",
            profile.stable_surfaces.len()
        )));
        assert!(summary_line.contains(&format!(
            "experimental surfaces: {}",
            profile.experimental_surfaces.len()
        )));
        assert!(summary_line.contains(&format!(
            "deprecation policy items: {}",
            profile.deprecation_policy.len()
        )));
        assert!(summary_line.contains(&format!(
            "intentional limits: {}",
            profile.intentional_limits.len()
        )));
        assert!(profile.to_string().contains("API stability posture:"));
        profile.validate().expect("current profile should validate");
    }

    #[test]
    fn profile_validation_rejects_whitespace_padded_metadata() {
        let profile = ApiStabilityProfile {
            profile_id: " pleiades-api-stability/0.1.0 ",
            summary: "valid summary",
            stable_surfaces: &["stable surface"],
            experimental_surfaces: &["experimental surface"],
            deprecation_policy: &["deprecation policy"],
            intentional_limits: &["intentional limit"],
        };

        let error = profile
            .validate()
            .expect_err("whitespace-padded profile identifier should be rejected");
        assert_eq!(
            error,
            ApiStabilityProfileValidationError::BlankProfileIdentifier
        );
    }

    #[test]
    fn profile_validation_rejects_empty_sections() {
        let profile = ApiStabilityProfile {
            profile_id: CURRENT_API_STABILITY_PROFILE_ID,
            summary: "valid summary",
            stable_surfaces: &[],
            experimental_surfaces: &["experimental surface"],
            deprecation_policy: &["deprecation policy"],
            intentional_limits: &["intentional limit"],
        };

        let error = profile
            .validate()
            .expect_err("empty stable surface section should be rejected");
        assert_eq!(
            error,
            ApiStabilityProfileValidationError::EmptyTextSection {
                section_label: "stable surfaces"
            }
        );
    }
}
