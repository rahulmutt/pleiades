//! Typed, drift-checked summary of the civil-time conversion posture.

use core::fmt;

/// The canonical one-line civil-time posture. Update with the implementation if
/// the supported window or tier model changes; the validator fails closed on drift.
pub const CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT: &str =
    "Civil UTC/UT1 input converts to TT/TDB over 1900-2100: leap-second-exact UTC, observed/extrapolated Delta-T elsewhere, each result tagged exact/observed/predicted";

/// Validation error for the civil-time policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CivilTimePolicyError {
    /// The summary text is empty or whitespace-only.
    BlankSummary,
    /// The summary text has leading or trailing whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded `\n` or `\r`.
    EmbeddedLineBreak,
    /// The summary text does not match the current canonical posture (drift).
    CurrentPolicyOutOfSync,
}

impl fmt::Display for CivilTimePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BlankSummary => "civil-time policy summary is blank",
            Self::WhitespacePaddedSummary => "civil-time policy summary has surrounding whitespace",
            Self::EmbeddedLineBreak => "civil-time policy summary contains a line break",
            Self::CurrentPolicyOutOfSync => {
                "civil-time policy summary is out of sync with the current posture"
            }
        })
    }
}

impl std::error::Error for CivilTimePolicyError {}

/// Compact summary of the current civil-time conversion posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CivilTimePolicySummary {
    summary: &'static str,
}

impl CivilTimePolicySummary {
    /// Wraps an arbitrary summary string (unvalidated) for later checking.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }
    /// The canonical current posture summary (`CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT`).
    pub const fn current() -> Self {
        Self::new(CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT)
    }
    /// The wrapped summary string, without validating it.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }
    /// Fails closed if the summary is blank, whitespace-padded, contains a line
    /// break, or has drifted from the canonical current posture.
    pub fn validate(&self) -> Result<(), CivilTimePolicyError> {
        if self.summary.trim().is_empty() {
            Err(CivilTimePolicyError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(CivilTimePolicyError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(CivilTimePolicyError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT {
            Err(CivilTimePolicyError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }
    /// Returns the summary string only after `validate` succeeds.
    pub fn validated_summary_line(&self) -> Result<&'static str, CivilTimePolicyError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for CivilTimePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_summary_validates() {
        assert!(CivilTimePolicySummary::current().validate().is_ok());
    }

    #[test]
    fn drifted_summary_is_rejected() {
        assert_eq!(
            CivilTimePolicySummary::new("stale").validate(),
            Err(CivilTimePolicyError::CurrentPolicyOutOfSync)
        );
    }
}
