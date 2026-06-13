use core::fmt;

/// Compact summary of the current native sidereal policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativeSiderealPolicySummary {
    summary: &'static str,
}

/// Validation error for the current native sidereal policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeSiderealPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current native sidereal posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for NativeSiderealPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("native sidereal policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("native sidereal policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("native sidereal policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => f.write_str(
                "native sidereal policy summary is out of sync with the current posture",
            ),
        }
    }
}

impl std::error::Error for NativeSiderealPolicySummaryValidationError {}

impl NativeSiderealPolicySummary {
    /// Creates a new native sidereal policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the native sidereal policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current native sidereal policy posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), NativeSiderealPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(NativeSiderealPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(NativeSiderealPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(NativeSiderealPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT {
            Err(NativeSiderealPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, NativeSiderealPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for NativeSiderealPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
