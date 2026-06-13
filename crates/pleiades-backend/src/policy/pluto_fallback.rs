use core::fmt;

/// Compact summary of the current Pluto fallback posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlutoFallbackSummary {
    summary: &'static str,
}

/// Validation error for the current Pluto fallback summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlutoFallbackSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current Pluto fallback posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for PlutoFallbackSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("Pluto fallback summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("Pluto fallback summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("Pluto fallback summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("Pluto fallback summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for PlutoFallbackSummaryValidationError {}

impl PlutoFallbackSummary {
    /// Creates a new Pluto fallback summary from backend-owned prose.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the Pluto fallback posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current Pluto fallback posture.
    pub const fn current() -> Self {
        Self::new(super::CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), PlutoFallbackSummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(PlutoFallbackSummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(PlutoFallbackSummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(PlutoFallbackSummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != super::CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT {
            Err(PlutoFallbackSummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, PlutoFallbackSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PlutoFallbackSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
