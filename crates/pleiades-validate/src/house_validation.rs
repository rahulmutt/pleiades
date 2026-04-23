//! House-system validation snapshots used by the stage-4 report output.
//!
//! The validation report keeps a compact, reproducible sample of the baseline
//! house systems at representative chart locations so house-formula changes can
//! be reviewed alongside the backend comparison and benchmark data.

#![forbid(unsafe_code)]

use core::fmt;

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
                "Polar stress chart",
                ObserverLocation::new(
                    Latitude::from_degrees(69.6492),
                    Longitude::from_degrees(18.9553),
                    Some(0.0),
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

    fn success_count(samples: &[HouseValidationSample]) -> usize {
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
            let success_count = Self::success_count(&scenario.samples);
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
