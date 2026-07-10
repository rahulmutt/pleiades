use core::fmt;

/// Compact summary of a backend's frame-treatment posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameTreatmentSummary {
    summary: &'static str,
}

/// Validation error for a frame-treatment summary that drifted away from a compact release-facing line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameTreatmentSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
}

impl fmt::Display for FrameTreatmentSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("frame-treatment summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("frame-treatment summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("frame-treatment summary contains a line break"),
        }
    }
}

impl std::error::Error for FrameTreatmentSummaryValidationError {}

impl FrameTreatmentSummary {
    /// Creates a new frame-treatment summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the frame-treatment posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns `Ok(())` when the summary still contains a compact canonical line.
    pub fn validate(&self) -> Result<(), FrameTreatmentSummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(FrameTreatmentSummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(FrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(FrameTreatmentSummaryValidationError::EmbeddedLineBreak)
        } else {
            Ok(())
        }
    }

    /// Returns the compact one-line rendering after validation.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, FrameTreatmentSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for FrameTreatmentSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}
