use core::fmt;

/// A summary of motion-direction classifications in a chart snapshot.
///
/// The counts track the current chart placements in the order used by the
/// compact report text: direct, stationary, retrograde, then unknown.
///
/// # Example
///
/// ```
/// use pleiades_core::MotionSummary;
///
/// let summary = MotionSummary {
///     direct: 2,
///     stationary: 1,
///     retrograde: 3,
///     unknown: 0,
/// };
///
/// assert_eq!(summary.summary_line(), "2 direct, 1 stationary, 3 retrograde, 0 unknown");
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert_eq!(summary.validate(6), Ok(()));
/// assert!(summary.has_known_motion());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MotionSummary {
    /// Placements classified as direct.
    pub direct: usize,
    /// Placements classified as stationary.
    pub stationary: usize,
    /// Placements classified as retrograde.
    pub retrograde: usize,
    /// Placements without enough motion data to classify.
    pub unknown: usize,
}

/// Errors returned when a motion summary no longer matches the chart placements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionSummaryValidationError {
    /// The total motion-summary count did not match the chart placement count.
    PlacementCountMismatch {
        /// Expected number of placements.
        expected: usize,
        /// Motion-summary total that was observed.
        actual: usize,
    },
}

impl fmt::Display for MotionSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlacementCountMismatch { expected, actual } => write!(
                f,
                "motion summary placement count mismatch: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for MotionSummaryValidationError {}

impl MotionSummary {
    /// Returns `true` when the snapshot contains at least one known motion classification.
    pub fn has_known_motion(self) -> bool {
        self.direct + self.stationary + self.retrograde > 0
    }

    /// Validates that the summary covers the expected number of placements.
    pub fn validate(self, placement_count: usize) -> Result<(), MotionSummaryValidationError> {
        let actual = self.direct + self.stationary + self.retrograde + self.unknown;
        if actual == placement_count {
            Ok(())
        } else {
            Err(MotionSummaryValidationError::PlacementCountMismatch {
                expected: placement_count,
                actual,
            })
        }
    }

    /// Returns a compact one-line summary of the motion classifications in the snapshot.
    pub fn summary_line(self) -> String {
        format!(
            "{} direct, {} stationary, {} retrograde, {} unknown",
            self.direct, self.stationary, self.retrograde, self.unknown
        )
    }
}

impl fmt::Display for MotionSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
