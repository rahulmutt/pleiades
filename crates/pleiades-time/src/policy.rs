//! Typed, drift-checked summary of the civil-time conversion posture.

use core::fmt;

/// The canonical one-line civil-time posture. Update with the implementation if
/// the supported window or tier model changes; the validator fails closed on drift.
pub const CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT: &str =
    "Civil UTC/UT1 input converts to TT/TDB over 1900–2100: leap-second-exact UTC, observed/extrapolated Delta-T elsewhere, each result tagged exact/observed/predicted.";

/// Validation error for the civil-time policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CivilTimePolicyError {
    BlankSummary,
    WhitespacePaddedSummary,
    EmbeddedLineBreak,
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
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }
    pub const fn current() -> Self {
        Self::new(CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT)
    }
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }
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
