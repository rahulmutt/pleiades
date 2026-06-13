use core::fmt;

/// Compact summary of the current shared time-scale policy.
///
/// # Example
///
/// ```
/// use pleiades_backend::TimeScalePolicySummary;
///
/// let summary = TimeScalePolicySummary::current();
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert!(summary.summary_line().contains("TT/TDB"));
/// assert!(summary.validate().is_ok());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimeScalePolicySummary {
    summary: &'static str,
}

/// Validation error for a time-scale policy summary that drifted away from a compact release-facing line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeScalePolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for TimeScalePolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("time-scale policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("time-scale policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("time-scale policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => {
                f.write_str("time-scale policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for TimeScalePolicySummaryValidationError {}

impl TimeScalePolicySummary {
    /// Creates a new time-scale policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the time-scale policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared time-scale policy posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), TimeScalePolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(TimeScalePolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(TimeScalePolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(TimeScalePolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT {
            Err(TimeScalePolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, TimeScalePolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for TimeScalePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
