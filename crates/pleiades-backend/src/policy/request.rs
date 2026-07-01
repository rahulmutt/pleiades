use core::fmt;

/// Compact summary of the current shared request-policy posture.
///
/// # Example
///
/// ```
/// use pleiades_backend::RequestPolicySummary;
///
/// let summary = RequestPolicySummary::current();
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert!(summary.summary_line().contains("time-scale="));
/// assert!(summary.summary_line().contains("observer="));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestPolicySummary {
    /// Time-scale policy wording.
    pub time_scale: &'static str,
    /// Observer policy wording.
    pub observer: &'static str,
    /// Apparentness policy wording.
    pub apparentness: &'static str,
    /// Frame policy wording.
    pub frame: &'static str,
}

impl RequestPolicySummary {
    /// Returns the current shared request-policy posture.
    pub const fn current() -> Self {
        Self {
            time_scale: super::CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT,
            observer: super::CURRENT_OBSERVER_POLICY_SUMMARY_TEXT,
            apparentness: super::CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT,
            frame: super::CURRENT_FRAME_POLICY_SUMMARY_TEXT,
        }
    }

    /// Returns a compact one-line rendering of the shared request-policy posture.
    pub fn summary_line(&self) -> String {
        format!(
            "time-scale={}; observer={}; apparentness={}; frame={}",
            self.time_scale, self.observer, self.apparentness, self.frame
        )
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(&self) -> Result<String, RequestPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for RequestPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for the shared request-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestPolicySummaryValidationError {
    /// A summary field was blank or whitespace-only.
    BlankField {
        /// Name of the offending summary field.
        field: &'static str,
    },
    /// A summary field had surrounding whitespace.
    WhitespacePaddedField {
        /// Name of the offending summary field.
        field: &'static str,
    },
    /// A summary field contained an embedded line break.
    EmbeddedLineBreak {
        /// Name of the offending summary field.
        field: &'static str,
    },
    /// A summary field is out of sync with the current request-policy posture.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the current posture.
        field: &'static str,
    },
}

impl fmt::Display for RequestPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankField { field } => {
                write!(f, "the request-policy summary field `{field}` is blank")
            }
            Self::WhitespacePaddedField { field } => write!(
                f,
                "the request-policy summary field `{field}` has surrounding whitespace"
            ),
            Self::EmbeddedLineBreak { field } => write!(
                f,
                "the request-policy summary field `{field}` contains a line break"
            ),
            Self::FieldOutOfSync { field } => write!(
                f,
                "the request-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for RequestPolicySummaryValidationError {}

impl RequestPolicySummary {
    /// Returns `Ok(())` when the shared request-policy wording still matches the current posture.
    pub fn validate(&self) -> Result<(), RequestPolicySummaryValidationError> {
        let current = Self::current();
        for (field, value, expected) in [
            ("time_scale", self.time_scale, current.time_scale),
            ("observer", self.observer, current.observer),
            ("apparentness", self.apparentness, current.apparentness),
            ("frame", self.frame, current.frame),
        ] {
            if value.trim().is_empty() {
                return Err(RequestPolicySummaryValidationError::BlankField { field });
            }
            if value.trim() != value {
                return Err(RequestPolicySummaryValidationError::WhitespacePaddedField { field });
            }
            if value.contains('\n') || value.contains('\r') {
                return Err(RequestPolicySummaryValidationError::EmbeddedLineBreak { field });
            }
            if value != expected {
                return Err(RequestPolicySummaryValidationError::FieldOutOfSync { field });
            }
        }

        Ok(())
    }
}
