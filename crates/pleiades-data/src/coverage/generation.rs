use super::*;

/// Structured generation policy for the packaged artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedArtifactGenerationPolicy {
    /// Same-body source epochs are fit with adjacent quadratic windows.
    AdjacentSameBodyQuadraticWindows,
}

/// Validation error for the packaged-artifact generation policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactGenerationPolicyValidationError {
    /// A policy field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactGenerationPolicyValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact generation policy field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactGenerationPolicyValidationError {}

impl PackagedArtifactGenerationPolicy {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::AdjacentSameBodyQuadraticWindows => "adjacent same-body quadratic windows",
        }
    }

    /// Returns the explanatory note used in release-facing summaries.
    pub fn note(self) -> &'static str {
        match self {
            Self::AdjacentSameBodyQuadraticWindows => {
                packaged_artifact_generation_policy_note_text()
            }
        }
    }

    /// Returns the segment-strategy text used in release-facing summaries.
    pub fn segment_strategy(self) -> &'static str {
        self.note()
    }

    /// Returns the compact release-facing summary for the generation policy.
    pub fn summary_line(self) -> String {
        format!("{}; {}", self.label(), self.note())
    }

    /// Returns `Ok(())` when the generation policy still matches the current packaged-artifact posture.
    pub fn validate(self) -> Result<(), PackagedArtifactGenerationPolicyValidationError> {
        if self != Self::AdjacentSameBodyQuadraticWindows {
            return Err(
                PackagedArtifactGenerationPolicyValidationError::FieldOutOfSync { field: "policy" },
            );
        }

        Ok(())
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Structured summary for the packaged-artifact generation policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedArtifactGenerationPolicySummary {
    /// Policy describing how the packaged artifact is generated.
    pub policy: PackagedArtifactGenerationPolicy,
}

/// Validation error for the packaged-artifact generation policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactGenerationPolicySummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactGenerationPolicySummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact generation policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactGenerationPolicySummaryValidationError {}

pub(crate) fn validate_packaged_artifact_generation_policy_residual_bodies(
    policy: PackagedArtifactGenerationPolicy,
    residual_bodies: &[CelestialBody],
) -> Result<(), PackagedArtifactGenerationPolicySummaryValidationError> {
    match policy {
        PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows => {
            let expected_residual_bodies = packaged_artifact().residual_bodies();
            if residual_bodies != expected_residual_bodies.as_slice() {
                return Err(
                    PackagedArtifactGenerationPolicySummaryValidationError::FieldOutOfSync {
                        field: "residual_bodies",
                    },
                );
            }
        }
    }

    Ok(())
}

impl PackagedArtifactGenerationPolicySummary {
    /// Returns the packaged-artifact generation policy as a compact human-readable line.
    pub fn summary_line(self) -> String {
        self.policy.summary_line()
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactGenerationPolicySummaryValidationError> {
        self.policy.validate().map_err(|error| match error {
            PackagedArtifactGenerationPolicyValidationError::FieldOutOfSync { field } => {
                PackagedArtifactGenerationPolicySummaryValidationError::FieldOutOfSync { field }
            }
        })?;

        validate_packaged_artifact_generation_policy_residual_bodies(
            self.policy,
            &packaged_artifact().residual_bodies(),
        )
    }
}

impl fmt::Display for PackagedArtifactGenerationPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const PACKAGED_ARTIFACT_GENERATION_POLICY_SUMMARY: PackagedArtifactGenerationPolicySummary =
    PackagedArtifactGenerationPolicySummary {
        policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
    };

/// Returns the current packaged-artifact generation policy summary record.
///
/// # Examples
///
/// ```
/// use pleiades_data::packaged_artifact_generation_policy_summary_details;
///
/// let summary = packaged_artifact_generation_policy_summary_details();
/// assert!(summary.validate().is_ok());
/// ```
pub fn packaged_artifact_generation_policy_summary_details(
) -> PackagedArtifactGenerationPolicySummary {
    let summary = PACKAGED_ARTIFACT_GENERATION_POLICY_SUMMARY;
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact residual-bearing body coverage summary record.
pub fn packaged_artifact_generation_residual_bodies_summary_details(
) -> ArtifactResidualBodyCoverageSummary {
    let artifact = packaged_artifact();
    let summary = artifact.residual_body_coverage_summary();
    debug_assert!(summary.validate(artifact).is_ok());
    summary
}

/// Returns the current packaged-artifact generation policy summary.
pub fn packaged_artifact_generation_policy_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_generation_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.to_string(),
                Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
            }
        })
        .as_str()
}
