use core::fmt;

/// Compact summary of the current shared Delta T policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeltaTPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared Delta T policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeltaTPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for DeltaTPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("Delta T policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("Delta T policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("Delta T policy summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("Delta T policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for DeltaTPolicySummaryValidationError {}

impl DeltaTPolicySummary {
    /// Creates a new Delta T policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the Delta T policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared Delta T policy posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_DELTA_T_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), DeltaTPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(DeltaTPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(DeltaTPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(DeltaTPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_DELTA_T_POLICY_SUMMARY_TEXT {
            Err(DeltaTPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, DeltaTPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for DeltaTPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
