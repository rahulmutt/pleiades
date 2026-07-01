//! Structured, fail-closed errors for apparent-place computation.

use core::fmt;

/// Error returned when an apparent-place correction cannot be performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentPlaceError {
    /// The light-time iteration did not converge within the iteration cap.
    NonConvergentLightTime {
        /// Number of iteration steps taken before giving up.
        iterations: u8,
    },
    /// A position lacked the geocentric distance light-time needs.
    MissingDistance,
    /// A computed correction was not finite (defensive).
    NonFiniteCorrection {
        /// Correction stage that produced the non-finite value (e.g. `"nutation"`).
        stage: &'static str,
    },
    /// A pinned model table failed its checksum/freshness gate.
    StaleModelData {
        /// Model table that failed the gate (e.g. `"nutation"`).
        kind: &'static str,
    },
}

impl ApparentPlaceError {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonConvergentLightTime { iterations } => {
                format!("light-time iteration did not converge after {iterations} step(s)")
            }
            Self::MissingDistance => {
                "apparent place requires a geocentric distance the position did not carry"
                    .to_string()
            }
            Self::NonFiniteCorrection { stage } => {
                format!("apparent-place correction stage `{stage}` produced a non-finite value")
            }
            Self::StaleModelData { kind } => {
                format!("{kind} apparent-model table failed its checksum/freshness gate")
            }
        }
    }
}

impl fmt::Display for ApparentPlaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for ApparentPlaceError {}

/// Error from the light-time iterator: either the caller's position query failed
/// (`Query`) or an apparent-place correction failed (`Apparent`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentLightTimeError<E> {
    /// The caller-supplied position query returned an error.
    Query(E),
    /// An apparent-place correction failed.
    Apparent(ApparentPlaceError),
}

impl<E: fmt::Display> fmt::Display for ApparentLightTimeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(error) => write!(f, "apparent-place position query failed: {error}"),
            Self::Apparent(error) => write!(f, "{error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ApparentLightTimeError<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_lines_are_distinct_and_nonempty() {
        let errors = [
            ApparentPlaceError::NonConvergentLightTime { iterations: 5 },
            ApparentPlaceError::MissingDistance,
            ApparentPlaceError::NonFiniteCorrection {
                stage: "aberration",
            },
            ApparentPlaceError::StaleModelData { kind: "nutation" },
        ];
        let mut seen = std::collections::HashSet::new();
        for e in errors {
            assert!(!e.summary_line().is_empty());
            assert_eq!(e.to_string(), e.summary_line());
            assert!(
                seen.insert(e.summary_line()),
                "duplicate summary: {}",
                e.summary_line()
            );
        }
    }

    #[test]
    fn light_time_error_wraps_query_and_apparent() {
        let q: ApparentLightTimeError<&str> = ApparentLightTimeError::Query("boom");
        assert!(q.to_string().contains("boom"));
        let a: ApparentLightTimeError<&str> =
            ApparentLightTimeError::Apparent(ApparentPlaceError::MissingDistance);
        assert!(a.to_string().contains("distance"));
    }
}
