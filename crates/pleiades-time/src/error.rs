//! Structured, fail-closed errors for civil-time conversion.

use core::fmt;

use pleiades_types::TimeScale;

/// Error returned when a civil-time conversion cannot be performed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CivilTimeError {
    /// A calendar field (month/day/hour/minute/second) was out of range or non-finite.
    InvalidCivilDate { field: &'static str },
    /// A UTC-tagged instant fell before the 1972 leap-second epoch.
    UtcBeforeLeapEpoch,
    /// The instant fell outside the documented support window (1900–2100).
    BeyondHorizon { jd: f64 },
    /// The requested source/target scale pair is not a civil conversion this crate performs.
    UnsupportedScale {
        source: TimeScale,
        target: TimeScale,
    },
    /// A pinned data table failed its checksum/freshness gate.
    StaleTimeData { kind: &'static str },
    /// A computed offset was not finite (defensive).
    NonFiniteOffset,
}

impl CivilTimeError {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        match self {
            Self::InvalidCivilDate { field } => {
                format!("invalid civil date field: {field}")
            }
            Self::UtcBeforeLeapEpoch => {
                "UTC civil input is undefined before 1972-01-01; tag pre-1972 input as UT1"
                    .to_string()
            }
            Self::BeyondHorizon { jd } => {
                format!("civil instant JD {jd} is outside the supported 1900–2100 window")
            }
            Self::UnsupportedScale { source, target } => {
                format!("unsupported civil conversion: {source} -> {target}")
            }
            Self::StaleTimeData { kind } => {
                format!("{kind} time-data table failed its checksum/freshness gate")
            }
            Self::NonFiniteOffset => "computed time-scale offset was not finite".to_string(),
        }
    }
}

impl fmt::Display for CivilTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for CivilTimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_lines_are_distinct_and_nonempty() {
        let errors = [
            CivilTimeError::InvalidCivilDate { field: "month" },
            CivilTimeError::UtcBeforeLeapEpoch,
            CivilTimeError::BeyondHorizon { jd: 1.0 },
            CivilTimeError::UnsupportedScale {
                source: TimeScale::Tt,
                target: TimeScale::Utc,
            },
            CivilTimeError::StaleTimeData {
                kind: "leap-second",
            },
            CivilTimeError::NonFiniteOffset,
        ];
        for e in &errors {
            assert!(!e.summary_line().is_empty());
            assert_eq!(e.to_string(), e.summary_line());
        }
        use std::collections::HashSet;
        let lines: Vec<String> = errors.iter().map(|e| e.summary_line()).collect();
        let unique: HashSet<&String> = lines.iter().collect();
        assert_eq!(unique.len(), lines.len(), "summary_lines must be distinct");
    }
}
