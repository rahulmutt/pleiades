//! Structured, fail-closed eclipse errors.

use core::fmt;

/// First instant of the supported window (1900-01-01 TT), Julian Day.
pub const WINDOW_START_JD: f64 = 2_415_020.5;
/// End of the supported window (2101-01-01 TT), Julian Day. The window covers
/// all of 1900–2100 CE inclusive, i.e. every eclipse through 2100-12-31.
pub const WINDOW_END_JD: f64 = 2_488_434.5;

#[derive(Clone, Debug, PartialEq)]
pub enum EclipseError {
    /// A requested instant falls outside the 1900–2100 CE window.
    OutOfWindow { julian_day: f64 },
    /// The backend returned a structured error.
    Backend(String),
    /// The backend produced no ecliptic coordinates for a required body.
    MissingCoordinates {
        body_label: &'static str,
        julian_day: f64,
    },
}

impl fmt::Display for EclipseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EclipseError::OutOfWindow { julian_day } => write!(
                f,
                "instant JD {julian_day} is outside the supported 1900–2100 CE window \
                 (JD {WINDOW_START_JD}..={WINDOW_END_JD})"
            ),
            EclipseError::Backend(message) => write!(f, "backend error: {message}"),
            EclipseError::MissingCoordinates {
                body_label,
                julian_day,
            } => write!(
                f,
                "backend returned no ecliptic coordinates for {body_label} at JD {julian_day}"
            ),
        }
    }
}

impl std::error::Error for EclipseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_window_message_names_the_julian_day() {
        let err = EclipseError::OutOfWindow {
            julian_day: 2_400_000.5,
        };
        assert!(err.to_string().contains("2400000.5"));
        assert!(err.to_string().contains("1900"));
    }

    #[test]
    fn window_constants_match_1900_2100() {
        assert_eq!(WINDOW_START_JD, 2_415_020.5);
        assert_eq!(WINDOW_END_JD, 2_488_434.5);
    }
}
