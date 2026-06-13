use core::fmt;

/// Compact summary of the current shared apparentness policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ApparentnessPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared apparentness-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentnessPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ApparentnessPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("apparentness policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("apparentness policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("apparentness policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => {
                f.write_str("apparentness policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ApparentnessPolicySummaryValidationError {}

impl ApparentnessPolicySummary {
    /// Creates a new apparentness policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the apparentness policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared apparentness policy posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), ApparentnessPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ApparentnessPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ApparentnessPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ApparentnessPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT {
            Err(ApparentnessPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ApparentnessPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ApparentnessPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
