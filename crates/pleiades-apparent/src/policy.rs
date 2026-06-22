//! Compact, validated summary of the apparent-place posture this crate implements.

use core::fmt;

/// Canonical one-line apparent-place posture.
pub const CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT: &str =
    "apparent place (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, true equinox of date, release-grade bodies; gravitational light-deflection omitted";

/// Compact summary of the current apparent-place policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ApparentPlacePolicySummary {
    summary: &'static str,
}

/// Validation error for the apparent-place policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentPlacePolicySummaryValidationError {
    BlankSummary,
    WhitespacePaddedSummary,
    EmbeddedLineBreak,
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ApparentPlacePolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("apparent-place policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("apparent-place policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("apparent-place policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => {
                f.write_str("apparent-place policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ApparentPlacePolicySummaryValidationError {}

impl ApparentPlacePolicySummary {
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    pub const fn current() -> Self {
        Self::new(CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT)
    }

    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    pub fn validate(&self) -> Result<(), ApparentPlacePolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ApparentPlacePolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ApparentPlacePolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ApparentPlacePolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT {
            Err(ApparentPlacePolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ApparentPlacePolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ApparentPlacePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_summary_validates() {
        assert_eq!(
            ApparentPlacePolicySummary::current()
                .validated_summary_line()
                .unwrap(),
            CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT
        );
    }

    #[test]
    fn out_of_sync_summary_is_rejected() {
        assert_eq!(
            ApparentPlacePolicySummary::new("stale").validate(),
            Err(ApparentPlacePolicySummaryValidationError::CurrentPolicyOutOfSync)
        );
    }
}
