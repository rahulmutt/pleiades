//! Structured, fail-closed event errors.

use core::fmt;

/// First instant of the supported window (1900-01-01 TT), Julian Day.
pub const WINDOW_START_JD: f64 = 2_415_020.5;
/// Last instant of the supported window (2100-01-01 TT), Julian Day — the end of
/// the packaged backend's Sun/Moon/planet coverage.
pub const WINDOW_END_JD: f64 = 2_488_069.5;

/// Errors returned by the event engine; all variants fail closed.
#[derive(Clone, Debug, PartialEq)]
pub enum EventError {
    /// A requested instant falls outside the 1900–2100 CE window.
    OutOfWindow {
        /// The out-of-window instant, as a Julian Day.
        julian_day: f64,
    },
    /// The backend returned a structured error (message forwarded verbatim).
    Backend(String),
    /// The backend produced no ecliptic coordinates (or no distance) for a body.
    MissingCoordinates {
        /// Human-readable label of the body that was missing (e.g. `"Sun"`).
        body_label: &'static str,
        /// The Julian Day at which coordinates were requested.
        julian_day: f64,
    },
    /// A frame/body combination that is not defined (e.g. heliocentric Sun/Moon).
    UnsupportedFrame {
        /// Human-readable explanation.
        detail: String,
    },
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::OutOfWindow { julian_day } => write!(
                f,
                "instant JD {julian_day} is outside the supported 1900–2100 CE window \
                 (JD {WINDOW_START_JD}..={WINDOW_END_JD})"
            ),
            EventError::Backend(message) => write!(f, "backend error: {message}"),
            EventError::MissingCoordinates {
                body_label,
                julian_day,
            } => write!(
                f,
                "backend returned no ecliptic coordinates for {body_label} at JD {julian_day}"
            ),
            EventError::UnsupportedFrame { detail } => {
                write!(f, "unsupported crossing frame: {detail}")
            }
        }
    }
}

impl std::error::Error for EventError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_window_message_names_the_julian_day() {
        let err = EventError::OutOfWindow {
            julian_day: 2_400_000.5,
        };
        assert!(err.to_string().contains("2400000.5"));
        assert!(err.to_string().contains("1900"));
    }

    #[test]
    fn window_constants_match_1900_2100() {
        assert_eq!(WINDOW_START_JD, 2_415_020.5);
        assert_eq!(WINDOW_END_JD, 2_488_069.5);
    }
}
