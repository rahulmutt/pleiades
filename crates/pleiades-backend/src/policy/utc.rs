use core::fmt;

/// Compact summary of the current shared UTC-convenience policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UtcConveniencePolicySummary {
    summary: &'static str,
}

/// Validation error for the shared UTC-convenience policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UtcConveniencePolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for UtcConveniencePolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("UTC convenience policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("UTC convenience policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("UTC convenience policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => f.write_str(
                "UTC convenience policy summary is out of sync with the current posture",
            ),
        }
    }
}

impl std::error::Error for UtcConveniencePolicySummaryValidationError {}

impl UtcConveniencePolicySummary {
    /// Creates a new UTC-convenience policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the UTC-convenience policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared UTC-convenience policy posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), UtcConveniencePolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(UtcConveniencePolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(UtcConveniencePolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(UtcConveniencePolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT {
            Err(UtcConveniencePolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, UtcConveniencePolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for UtcConveniencePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
