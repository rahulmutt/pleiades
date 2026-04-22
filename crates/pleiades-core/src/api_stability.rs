//! Versioned API stability posture for the current release line.
//!
//! The release plan requires a clear statement of what consumers can treat as
//! stable versus what remains tooling- or release-internal. This profile keeps
//! that posture explicit without pretending that the entire workspace is a
//! frozen interchange format.

#![forbid(unsafe_code)]

use core::fmt;

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
}

/// Returns the current API stability posture.
pub const fn current_api_stability_profile() -> ApiStabilityProfile {
    ApiStabilityProfile {
        profile_id: "pleiades-api-stability/0.1.0",
        summary: "The stable consumer surface is the shared domain model, backend contract, and chart/compatibility façade; validation and release-tooling formats are documented but still allowed to evolve as hardening continues.",
        stable_surfaces: &[
            "pleiades-types defines the stable units, identifiers, and request/response primitives.",
            "pleiades-backend's EphemerisBackend trait and metadata model are the primary backend-facing contract.",
            "pleiades-core's ChartEngine, ChartRequest, ChartSnapshot, and compatibility-profile helpers are the stable façade used by consumers.",
            "ChartSnapshot body-placement helpers include motion-direction classification for backend motion data when present.",
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
        assert!(profile.to_string().contains("API stability posture:"));
    }
}
