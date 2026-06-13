use core::fmt;

/// Compact summary of the current shared zodiac policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ZodiacPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared zodiac-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZodiacPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current zodiac posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ZodiacPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("zodiac policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("zodiac policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("zodiac policy summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("zodiac policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ZodiacPolicySummaryValidationError {}

impl ZodiacPolicySummary {
    /// Creates a new zodiac policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the zodiac policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared zodiac policy posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_ZODIAC_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), ZodiacPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ZodiacPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ZodiacPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ZodiacPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_ZODIAC_POLICY_SUMMARY_TEXT {
            Err(ZodiacPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ZodiacPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ZodiacPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
