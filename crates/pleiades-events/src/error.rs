//! Structured, fail-closed event errors.

use core::fmt;
use pleiades_types::TimeScale;

/// First instant of the supported window (1900-01-01 TT), Julian Day.
pub const WINDOW_START_JD: f64 = 2_415_020.5;
/// Last instant of the supported window (2100-01-01 TT), Julian Day — the end of
/// the packaged backend's Sun/Moon/planet coverage.
pub const WINDOW_END_JD: f64 = 2_488_069.5;

/// Errors returned by the event engine; all variants fail closed.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum EventError {
    /// A requested instant falls outside the 1900–2100 CE window, or cannot
    /// be served from inside it.
    ///
    /// The second case is an apparent geocentric place within a light-time
    /// of the window start: the body is read a light-time before the
    /// instant (1.3 s for the Moon, up to about 0.3 day for Pluto), which
    /// there falls before the window. The Sun, the lunar points and the
    /// mean-of-date and heliocentric frames need no such read and are served
    /// from the window's first instant.
    ///
    /// A rise, set or transit search also returns this when the window ends
    /// before its search span does and no event lies inside the window: the
    /// event may exist but cannot be computed. `julian_day` is then the first
    /// instant past the window that the search needed.
    OutOfWindow {
        /// The out-of-window instant, as a Julian Day.
        julian_day: f64,
    },
    /// The backend returned a structured error (message forwarded verbatim).
    Backend(String),
    /// The backend produced no ecliptic coordinates for a body.
    MissingCoordinates {
        /// Human-readable label of the body that was missing (e.g. `"Sun"`).
        body_label: &'static str,
        /// The Julian Day at which coordinates were requested.
        julian_day: f64,
    },
    /// The backend produced ecliptic coordinates but no distance for a body
    /// that needs one. Every body except the lunar orbit points (mean and true
    /// node, apogee and perigee) needs a distance in every frame; the lunar
    /// points are directions and are served without one by some backends.
    MissingDistance {
        /// Human-readable label of the body (e.g. `"Sun"`).
        body_label: &'static str,
        /// The Julian Day at which coordinates were requested.
        julian_day: f64,
    },
    /// A frame/body combination that is not defined (e.g. heliocentric Sun/Moon).
    UnsupportedFrame {
        /// Human-readable explanation.
        detail: String,
    },
    /// The observer location failed validation (non-finite / out of range).
    InvalidObserver {
        /// Human-readable detail.
        detail: String,
    },
    /// A fixed-star name not present in the curated catalog.
    UnknownFixedStar {
        /// The requested name.
        name: String,
    },
    /// Atmosphere parameters were non-finite.
    InvalidAtmosphere {
        /// Human-readable detail.
        detail: String,
    },
    /// A body/method combination `nod_aps` does not support (fail-closed).
    UnsupportedNodAps {
        /// Human-readable explanation.
        detail: String,
    },
    /// The orbit geometry was too degenerate to form nodes/apsides.
    DegenerateNodAps {
        /// Human-readable explanation.
        detail: String,
    },
    /// A body that cannot be occulted this way (Sun or Moon as the target).
    UnsupportedOccultTarget {
        /// Human-readable explanation.
        detail: String,
    },
    /// An instant tagged with a time scale the observer-local surfaces
    /// (rise/set/transit, horizontal coordinates) cannot convert to TDB.
    UnsupportedTimeScale {
        /// The unsupported scale.
        scale: TimeScale,
    },
    /// The backend reported no finite longitude speed for a body, so its
    /// stations cannot be found.
    MissingSpeed {
        /// Human-readable label of the body (e.g. `"Mercury"`).
        body_label: &'static str,
        /// The Julian Day at which the speed was requested.
        julian_day: f64,
    },
    /// An aspect request that is not defined: a non-finite angle, an angle
    /// outside 0–180 degrees, or the same body twice.
    InvalidAspect {
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
            EventError::MissingDistance {
                body_label,
                julian_day,
            } => write!(
                f,
                "backend returned ecliptic coordinates but no distance for {body_label} at JD \
                 {julian_day}"
            ),
            EventError::UnsupportedFrame { detail } => {
                write!(f, "unsupported crossing frame: {detail}")
            }
            EventError::InvalidObserver { detail } => write!(f, "invalid observer: {detail}"),
            EventError::UnknownFixedStar { name } => write!(f, "unknown fixed star: {name}"),
            EventError::InvalidAtmosphere { detail } => write!(f, "invalid atmosphere: {detail}"),
            EventError::UnsupportedNodAps { detail } => {
                write!(f, "unsupported nod_aps request: {detail}")
            }
            EventError::DegenerateNodAps { detail } => {
                write!(f, "degenerate nod_aps geometry: {detail}")
            }
            EventError::UnsupportedOccultTarget { detail } => {
                write!(f, "unsupported occultation target: {detail}")
            }
            EventError::UnsupportedTimeScale { scale } => write!(
                f,
                "unsupported time scale {scale:?}: rise/set/transit and horizontal \
                 coordinates accept TDB, TT, UT1, and UTC instants"
            ),
            EventError::MissingSpeed {
                body_label,
                julian_day,
            } => write!(
                f,
                "backend reported no finite longitude speed for {body_label} at JD {julian_day}"
            ),
            EventError::InvalidAspect { detail } => write!(f, "invalid aspect: {detail}"),
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

    #[test]
    fn nod_aps_errors_render_their_detail() {
        let err = EventError::UnsupportedNodAps {
            detail: "mean elements for Pluto".into(),
        };
        assert!(err.to_string().contains("mean elements for Pluto"));
        let err = EventError::DegenerateNodAps {
            detail: "node ill-defined".into(),
        };
        assert!(err.to_string().contains("node ill-defined"));
    }

    #[test]
    fn unsupported_time_scale_names_the_scale() {
        let err = EventError::UnsupportedTimeScale {
            scale: TimeScale::Utc,
        };
        assert!(err.to_string().contains("Utc"));
        assert!(err.to_string().contains("TDB, TT, UT1, and UTC"));
    }

    #[test]
    fn missing_speed_names_the_body_and_the_julian_day() {
        let err = EventError::MissingSpeed {
            body_label: "Mercury",
            julian_day: 2_451_545.0,
        };
        let text = err.to_string();
        assert!(text.contains("Mercury"), "{text}");
        assert!(text.contains("2451545"), "{text}");
        assert!(text.contains("longitude speed"), "{text}");
    }

    #[test]
    fn invalid_aspect_renders_its_detail() {
        let err = EventError::InvalidAspect {
            detail: "got 200".into(),
        };
        assert_eq!(err.to_string(), "invalid aspect: got 200");
    }
}
