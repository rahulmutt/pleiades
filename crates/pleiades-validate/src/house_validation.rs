//! House-system validation snapshots used by the stage-4 report output.
//!
//! The validation report keeps a compact, reproducible sample of the baseline
//! house systems at representative chart locations so house-formula changes can
//! be reviewed alongside the backend comparison and benchmark data.

#![forbid(unsafe_code)]

use core::fmt;
use std::collections::BTreeSet;

use pleiades_core::{
    baseline_house_systems, calculate_houses, HouseError, HouseRequest, HouseSnapshot,
    HouseSystemDescriptor, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};

/// A house-validation sample for one system in one chart scenario.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseValidationSample {
    /// Descriptor for the validated house system.
    pub descriptor: HouseSystemDescriptor,
    /// Calculation outcome for the sample.
    pub result: Result<HouseSnapshot, HouseError>,
}

/// A representative validation scenario.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseValidationScenario {
    /// Human-readable scenario label.
    pub label: &'static str,
    /// Instant used for the sample chart.
    pub instant: Instant,
    /// Observer location used for the sample chart.
    pub observer: ObserverLocation,
    /// Per-system validation samples.
    pub samples: Vec<HouseValidationSample>,
}

/// A compact validation corpus for baseline house systems.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseValidationReport {
    /// Scenarios included in the report.
    pub scenarios: Vec<HouseValidationScenario>,
}

/// Errors produced while validating a house-validation report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HouseValidationReportValidationError {
    /// The report does not contain any scenarios.
    EmptyScenarioList,
    /// A scenario label was blank or whitespace only.
    BlankScenarioLabel { scenario_index: usize },
    /// A scenario label was duplicated.
    DuplicateScenarioLabel { label: &'static str },
    /// A scenario does not contain any samples.
    EmptyScenarioSamples {
        scenario_index: usize,
        label: &'static str,
    },
    /// A scenario does not match the baseline house-system coverage.
    ScenarioSampleCountMismatch {
        scenario_index: usize,
        label: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for HouseValidationReportValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScenarioList => {
                f.write_str("house validation report does not contain any scenarios")
            }
            Self::BlankScenarioLabel { scenario_index } => {
                write!(f, "house validation scenario #{scenario_index} has a blank label")
            }
            Self::DuplicateScenarioLabel { label } => {
                write!(f, "house validation scenario label '{label}' is duplicated")
            }
            Self::EmptyScenarioSamples {
                scenario_index,
                label,
            } => write!(
                f,
                "house validation scenario #{scenario_index} ('{label}') does not contain any samples"
            ),
            Self::ScenarioSampleCountMismatch {
                scenario_index,
                label,
                expected,
                actual,
            } => write!(
                f,
                "house validation scenario #{scenario_index} ('{label}') has {actual} samples but expected {expected}"
            ),
        }
    }
}

impl std::error::Error for HouseValidationReportValidationError {}

impl HouseValidationReport {
    /// Creates the default house-validation corpus.
    pub fn new() -> Self {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let scenarios = [
            (
                "Mid-latitude reference chart",
                ObserverLocation::new(
                    Latitude::from_degrees(51.5074),
                    Longitude::from_degrees(0.0),
                    None,
                ),
            ),
            (
                "Equatorial reference chart",
                ObserverLocation::new(
                    Latitude::from_degrees(0.0),
                    Longitude::from_degrees(0.0),
                    None,
                ),
            ),
            (
                "Polar stress chart",
                ObserverLocation::new(
                    Latitude::from_degrees(69.6492),
                    Longitude::from_degrees(18.9553),
                    Some(0.0),
                ),
            ),
            (
                "Southern hemisphere reference chart",
                ObserverLocation::new(
                    Latitude::from_degrees(-33.8688),
                    Longitude::from_degrees(151.2093),
                    None,
                ),
            ),
        ]
        .into_iter()
        .map(|(label, observer)| HouseValidationScenario {
            label,
            instant,
            observer: observer.clone(),
            samples: baseline_house_systems()
                .iter()
                .map(|descriptor| HouseValidationSample {
                    descriptor: descriptor.clone(),
                    result: calculate_houses(&HouseRequest::new(
                        instant,
                        observer.clone(),
                        descriptor.system.clone(),
                    )),
                })
                .collect(),
        })
        .collect();

        Self { scenarios }
    }

    /// Returns the total number of scenario/sample calculations.
    pub fn sample_count(&self) -> usize {
        self.scenarios
            .iter()
            .map(|scenario| scenario.samples.len())
            .sum()
    }

    /// Returns the number of successful calculations in the report.
    pub fn success_count(&self) -> usize {
        self.scenarios
            .iter()
            .flat_map(|scenario| scenario.samples.iter())
            .filter(|sample| sample.result.is_ok())
            .count()
    }

    /// Returns the number of failing calculations in the report.
    pub fn failure_count(&self) -> usize {
        self.sample_count().saturating_sub(self.success_count())
    }

    /// Returns the distinct latitude-sensitive house systems represented by the report.
    pub fn latitude_sensitive_systems(&self) -> Vec<&'static str> {
        let mut systems = BTreeSet::new();
        for scenario in &self.scenarios {
            for sample in &scenario.samples {
                if sample.descriptor.latitude_sensitive {
                    systems.insert(sample.descriptor.canonical_name);
                }
            }
        }
        systems.into_iter().collect()
    }

    /// Returns the scenario labels represented by the report.
    pub fn scenario_labels(&self) -> Vec<&'static str> {
        self.scenarios
            .iter()
            .map(|scenario| scenario.label)
            .collect()
    }

    /// Returns a compact release-facing summary line.
    pub fn summary_line(&self) -> String {
        let latitude_sensitive_systems = self.latitude_sensitive_systems();
        let scenario_labels = self.scenario_labels();
        format!(
            "House validation corpus: {} scenarios ({}), {} samples, {} successes, {} failures; latitude-sensitive systems: {}",
            self.scenarios.len(),
            if scenario_labels.is_empty() {
                "none".to_string()
            } else {
                scenario_labels.join(", ")
            },
            self.sample_count(),
            self.success_count(),
            self.failure_count(),
            if latitude_sensitive_systems.is_empty() {
                "none".to_string()
            } else {
                latitude_sensitive_systems.join(", ")
            }
        )
    }

    /// Validates that the report still reflects the expected baseline corpus shape.
    pub fn validate(&self) -> Result<(), HouseValidationReportValidationError> {
        if self.scenarios.is_empty() {
            return Err(HouseValidationReportValidationError::EmptyScenarioList);
        }

        let expected_sample_count = baseline_house_systems().len();
        let mut scenario_labels = BTreeSet::new();

        for (index, scenario) in self.scenarios.iter().enumerate() {
            let label = scenario.label.trim();
            if label.is_empty() {
                return Err(HouseValidationReportValidationError::BlankScenarioLabel {
                    scenario_index: index + 1,
                });
            }
            if label != scenario.label {
                return Err(HouseValidationReportValidationError::BlankScenarioLabel {
                    scenario_index: index + 1,
                });
            }
            if !scenario_labels.insert(scenario.label) {
                return Err(
                    HouseValidationReportValidationError::DuplicateScenarioLabel {
                        label: scenario.label,
                    },
                );
            }
            if scenario.samples.is_empty() {
                return Err(HouseValidationReportValidationError::EmptyScenarioSamples {
                    scenario_index: index + 1,
                    label: scenario.label,
                });
            }
            if scenario.samples.len() != expected_sample_count {
                return Err(
                    HouseValidationReportValidationError::ScenarioSampleCountMismatch {
                        scenario_index: index + 1,
                        label: scenario.label,
                        expected: expected_sample_count,
                        actual: scenario.samples.len(),
                    },
                );
            }
        }

        Ok(())
    }

    fn success_count_for(samples: &[HouseValidationSample]) -> usize {
        samples
            .iter()
            .filter(|sample| sample.result.is_ok())
            .count()
    }

    fn failure_names(samples: &[HouseValidationSample]) -> Vec<&'static str> {
        samples
            .iter()
            .filter(|sample| sample.result.is_err())
            .map(|sample| sample.descriptor.canonical_name)
            .collect()
    }
}

impl Default for HouseValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for HouseValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "House validation corpus")?;
        for scenario in &self.scenarios {
            let success_count = Self::success_count_for(&scenario.samples);
            let failure_names = Self::failure_names(&scenario.samples);

            writeln!(f, "{}", scenario.label)?;
            writeln!(f, "  instant: {}", scenario.instant.julian_day)?;
            writeln!(
                f,
                "  observer: lat={:.4}°, lon={:.4}°{}",
                scenario.observer.latitude.degrees(),
                scenario.observer.longitude.degrees(),
                scenario
                    .observer
                    .elevation_m
                    .map(|elevation| format!(", elev={elevation:.1} m"))
                    .unwrap_or_default()
            )?;
            writeln!(f, "  systems: {}", scenario.samples.len())?;
            writeln!(f, "  successes: {}", success_count)?;
            if failure_names.is_empty() {
                writeln!(f, "  failures: none")?;
            } else {
                writeln!(f, "  failures: {}", failure_names.join(", "))?;
            }

            for sample in &scenario.samples {
                let request = HouseRequest::new(
                    scenario.instant,
                    scenario.observer.clone(),
                    sample.descriptor.system.clone(),
                );

                writeln!(f, "  request: {}", request)?;
                match &sample.result {
                    Ok(snapshot) => {
                        writeln!(
                            f,
                            "  {}{}: asc={}, mc={}, cusp1={}, cusp10={}",
                            sample.descriptor.canonical_name,
                            if sample.descriptor.latitude_sensitive {
                                " [latitude-sensitive]"
                            } else {
                                ""
                            },
                            snapshot.angles.ascendant,
                            snapshot.angles.midheaven,
                            snapshot
                                .cusp(1)
                                .map(|cusp| cusp.to_string())
                                .unwrap_or_else(|| "n/a".to_string()),
                            snapshot
                                .cusp(10)
                                .map(|cusp| cusp.to_string())
                                .unwrap_or_else(|| "n/a".to_string())
                        )?;
                    }
                    Err(error) => {
                        writeln!(
                            f,
                            "  {}{}: {}",
                            sample.descriptor.canonical_name,
                            if sample.descriptor.latitude_sensitive {
                                " [latitude-sensitive]"
                            } else {
                                ""
                            },
                            error
                        )?;
                    }
                }
            }

            writeln!(f)?;
        }
        Ok(())
    }
}

/// Renders the default house-validation corpus.
pub fn house_validation_report() -> HouseValidationReport {
    HouseValidationReport::new()
}

/// Returns the compact report-facing summary line, or an unavailable message if validation fails.
pub fn house_validation_summary_line_for_report(report: &HouseValidationReport) -> String {
    match report.validate() {
        Ok(()) => report.summary_line(),
        Err(error) => format!("House validation corpus unavailable: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_line_reports_scenario_and_latitude_sensitive_counts() {
        let report = house_validation_report();

        assert_eq!(report.scenarios.len(), 4);
        assert_eq!(
            report.sample_count(),
            report.scenarios.len() * baseline_house_systems().len()
        );
        assert_eq!(
            report.latitude_sensitive_systems(),
            vec!["Koch", "Placidus", "Topocentric"]
        );
        assert_eq!(
            report.scenario_labels(),
            vec![
                "Mid-latitude reference chart",
                "Equatorial reference chart",
                "Polar stress chart",
                "Southern hemisphere reference chart",
            ]
        );

        assert_eq!(
            report.summary_line(),
            "House validation corpus: 4 scenarios (Mid-latitude reference chart, Equatorial reference chart, Polar stress chart, Southern hemisphere reference chart), 48 samples, 48 successes, 0 failures; latitude-sensitive systems: Koch, Placidus, Topocentric"
        );
        assert_eq!(
            house_validation_summary_line_for_report(&report),
            report.summary_line()
        );
        assert_eq!(report.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_drifted_corpus_shapes() {
        let report = HouseValidationReport {
            scenarios: vec![HouseValidationScenario {
                label: "",
                instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
                observer: ObserverLocation::new(
                    Latitude::from_degrees(0.0),
                    Longitude::from_degrees(0.0),
                    None,
                ),
                samples: Vec::new(),
            }],
        };

        assert!(matches!(
            report.validate(),
            Err(HouseValidationReportValidationError::BlankScenarioLabel { .. })
        ));
        assert_eq!(
            house_validation_summary_line_for_report(&report),
            "House validation corpus unavailable: house validation scenario #1 has a blank label"
        );
    }
}
