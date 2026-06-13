use core::fmt;

/// Compact summary of the current shared observer policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObserverPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared observer-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObserverPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ObserverPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("observer policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("observer policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("observer policy summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("observer policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ObserverPolicySummaryValidationError {}

impl ObserverPolicySummary {
    /// Creates a new observer policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the observer policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared observer policy posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_OBSERVER_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), ObserverPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ObserverPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ObserverPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ObserverPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_OBSERVER_POLICY_SUMMARY_TEXT {
            Err(ObserverPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ObserverPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ObserverPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
