//! Time interval types: [`TimeRange`] and [`TimeRangeValidationError`].

use core::fmt;

use crate::time::Instant;

/// A Julian-day interval.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    /// Inclusive lower bound.
    pub start: Option<Instant>,
    /// Inclusive upper bound.
    pub end: Option<Instant>,
}

impl TimeRange {
    /// Creates a new time range.
    pub const fn new(start: Option<Instant>, end: Option<Instant>) -> Self {
        Self { start, end }
    }

    /// Returns `true` if the given instant is inside the range.
    ///
    /// Containment requires the instant to share each present bound's
    /// [`TimeScale`](crate::TimeScale): an instant tagged with a different time
    /// scale than a present bound, or one carrying a non-finite Julian day, is
    /// treated as not contained. An unbounded side imposes no constraint.
    pub fn contains(&self, instant: Instant) -> bool {
        let after_start = self.start.is_none_or(|start| {
            same_scale_and_jd(instant, start)
                && instant.julian_day.days() >= start.julian_day.days()
        });
        let before_end = self.end.is_none_or(|end| {
            same_scale_and_jd(instant, end) && instant.julian_day.days() <= end.julian_day.days()
        });
        after_start && before_end
    }

    /// Validates the range bounds and ordering.
    ///
    /// Unbounded ranges are valid. When one or both bounds are present, the
    /// finite Julian-day requirement applies to each bound, the two bounds must
    /// use the same time scale, and the upper bound must not precede the lower
    /// bound.
    pub fn validate(self) -> Result<(), TimeRangeValidationError> {
        if let Some(start) = self.start {
            if !start.julian_day.days().is_finite() {
                return Err(TimeRangeValidationError::non_finite_bound("start", start));
            }
        }
        if let Some(end) = self.end {
            if !end.julian_day.days().is_finite() {
                return Err(TimeRangeValidationError::non_finite_bound("end", end));
            }
        }
        if let (Some(start), Some(end)) = (self.start, self.end) {
            if start.scale != end.scale {
                return Err(TimeRangeValidationError::scale_mismatch(start, end));
            }
            if start.julian_day.days() > end.julian_day.days() {
                return Err(TimeRangeValidationError::out_of_order(start, end));
            }
        }

        Ok(())
    }

    /// Returns a compact one-line rendering of the range.
    pub fn summary_line(&self) -> String {
        match (self.start, self.end) {
            (Some(start), Some(end)) => format!(
                "{} → {}",
                format_time_range_instant(start),
                format_time_range_instant(end)
            ),
            (Some(start), None) => format!("from {}", format_time_range_instant(start)),
            (None, Some(end)) => format!("through {}", format_time_range_instant(end)),
            (None, None) => "unbounded".to_string(),
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn format_time_range_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

pub(crate) fn same_scale_and_jd(a: Instant, b: Instant) -> bool {
    a.scale == b.scale && a.julian_day.days().is_finite() && b.julian_day.days().is_finite()
}

/// Shared validation errors for [`TimeRange`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeRangeValidationError {
    /// A required bound contains a non-finite Julian day.
    NonFiniteBound {
        /// Which bound failed validation.
        bound: &'static str,
        /// The offending instant.
        instant: Instant,
    },
    /// The lower and upper bounds use different time scales.
    ScaleMismatch {
        /// The lower bound.
        start: Instant,
        /// The upper bound.
        end: Instant,
    },
    /// The upper bound precedes the lower bound.
    OutOfOrder {
        /// The lower bound.
        start: Instant,
        /// The upper bound.
        end: Instant,
    },
}

impl TimeRangeValidationError {
    pub(crate) const fn non_finite_bound(bound: &'static str, instant: Instant) -> Self {
        Self::NonFiniteBound { bound, instant }
    }

    pub(crate) const fn scale_mismatch(start: Instant, end: Instant) -> Self {
        Self::ScaleMismatch { start, end }
    }

    pub(crate) const fn out_of_order(start: Instant, end: Instant) -> Self {
        Self::OutOfOrder { start, end }
    }

    /// Returns a compact one-line rendering of the range validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonFiniteBound { bound, instant } => format!(
                "time range bound `{bound}` must be finite: {}",
                format_time_range_instant(*instant)
            ),
            Self::ScaleMismatch { start, end } => format!(
                "time range bounds must use the same time scale: start={}; end={}",
                format_time_range_instant(*start),
                format_time_range_instant(*end)
            ),
            Self::OutOfOrder { start, end } => format!(
                "time range end must not precede the start: start={}; end={}",
                format_time_range_instant(*start),
                format_time_range_instant(*end)
            ),
        }
    }
}

impl fmt::Display for TimeRangeValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for TimeRangeValidationError {}
