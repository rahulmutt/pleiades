//! House-system validation snapshots used by the stage-4 report output.
//!
//! The validation report keeps a compact, reproducible sample of the baseline
//! house systems at representative chart locations so house-formula changes can
//! be reviewed alongside the backend comparison and benchmark data.

#![forbid(unsafe_code)]

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use pleiades_core::{
    baseline_house_systems, calculate_houses, HouseError, HouseRequest, HouseSnapshot,
    HouseSystemDescriptor, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};
use pleiades_houses::{built_in_house_systems, HighLatitudePolicy, HouseFormulaFamily};

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

/// A compact validation corpus for a house-system catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseValidationReport {
    catalog_label: &'static str,
    house_systems: &'static [HouseSystemDescriptor],
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
    /// A scenario label was duplicated after normalization.
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
    /// Creates a house-validation corpus for the provided catalog.
    pub fn new_with_catalog(
        catalog_label: &'static str,
        house_systems: &'static [HouseSystemDescriptor],
    ) -> Self {
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
                "Western hemisphere reference chart",
                ObserverLocation::new(
                    Latitude::from_degrees(34.0522),
                    Longitude::from_degrees(-118.2437),
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
                "Northern high-latitude stress chart",
                ObserverLocation::new(
                    Latitude::from_degrees(78.0),
                    Longitude::from_degrees(18.9553),
                    Some(0.0),
                ),
            ),
            (
                "Northern high-latitude mountain stress chart",
                ObserverLocation::new(
                    Latitude::from_degrees(78.0),
                    Longitude::from_degrees(18.9553),
                    Some(2_000.0),
                ),
            ),
            (
                "Southern high-latitude mountain stress chart",
                ObserverLocation::new(
                    Latitude::from_degrees(-78.0),
                    Longitude::from_degrees(18.9553),
                    Some(2_000.0),
                ),
            ),
            (
                "Southern polar stress chart",
                ObserverLocation::new(
                    Latitude::from_degrees(-69.6492),
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
            samples: house_systems
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

        Self {
            catalog_label,
            house_systems,
            scenarios,
        }
    }

    /// Creates the default baseline house-validation corpus.
    pub fn new() -> Self {
        Self::new_with_catalog("baseline", baseline_house_systems())
    }

    /// Creates the release house-validation corpus spanning every built-in house system.
    pub fn release() -> Self {
        Self::new_with_catalog("built-in", built_in_house_systems())
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

    /// Returns the distinct formula families represented by the report.
    pub fn formula_families(&self) -> Vec<String> {
        let mut families = Vec::new();
        for family in [
            HouseFormulaFamily::Equal,
            HouseFormulaFamily::WholeSign,
            HouseFormulaFamily::Quadrant,
            HouseFormulaFamily::EquatorialProjection,
            HouseFormulaFamily::GreatCircle,
            HouseFormulaFamily::SolarArc,
            HouseFormulaFamily::Sector,
            HouseFormulaFamily::Custom,
            HouseFormulaFamily::Unknown,
        ] {
            if self
                .scenarios
                .iter()
                .flat_map(|scenario| scenario.samples.iter())
                .any(|sample| sample.descriptor.formula_family() == family)
            {
                families.push(family.to_string());
            }
        }
        families
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

    /// Returns the release-facing constraint notes for latitude-sensitive systems.
    pub fn latitude_sensitive_constraints(&self) -> Vec<String> {
        let mut constraints = BTreeMap::new();
        for scenario in &self.scenarios {
            for sample in &scenario.samples {
                if sample.descriptor.latitude_sensitive {
                    constraints
                        .entry(sample.descriptor.canonical_name)
                        .or_insert(sample.descriptor.notes);
                }
            }
        }

        constraints
            .into_iter()
            .map(|(name, notes)| format!("{name} [{notes}]"))
            .collect()
    }

    /// Returns the number of scenarios whose observer locations fall in each hemisphere bucket.
    ///
    /// Exact-zero latitudes are counted as equatorial rather than northern or southern.
    pub fn hemisphere_coverage(&self) -> (usize, usize, usize) {
        let mut north = 0;
        let mut south = 0;
        let mut equatorial = 0;

        for scenario in &self.scenarios {
            let latitude = scenario.observer.latitude.degrees();
            if latitude > 0.0 {
                north += 1;
            } else if latitude < 0.0 {
                south += 1;
            } else {
                equatorial += 1;
            }
        }

        (north, south, equatorial)
    }

    /// Returns the number of scenarios whose observer longitudes fall on or off the prime meridian.
    ///
    /// `Longitude` values are normalized into `[0, 360)`, so the report can only
    /// distinguish prime-meridian samples from non-prime-meridian samples.
    pub fn longitude_coverage(&self) -> (usize, usize) {
        let mut prime_meridian = 0;
        let mut non_prime_meridian = 0;

        for scenario in &self.scenarios {
            let longitude = scenario.observer.longitude.degrees();
            if longitude == 0.0 {
                prime_meridian += 1;
            } else {
                non_prime_meridian += 1;
            }
        }

        (prime_meridian, non_prime_meridian)
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
        let formula_families = self.formula_families();
        let latitude_sensitive_systems = self.latitude_sensitive_systems();
        let latitude_sensitive_constraints = self.latitude_sensitive_constraints();
        let scenario_labels = self.scenario_labels();
        let (north_hemispheres, south_hemispheres, equatorial_hemispheres) =
            self.hemisphere_coverage();
        let (prime_meridian_longitudes, non_prime_meridian_longitudes) = self.longitude_coverage();
        format!(
            "House validation corpus: {} scenarios ({}), {} samples, {} successes, {} failures; hemisphere coverage: north={}, south={}, equatorial={}; longitude coverage: prime-meridian={}, non-prime-meridian={}; formula families: {}; latitude-sensitive systems: {}; constraints: {}; implementation posture: {} {} systems validated",
            self.scenarios.len(),
            if scenario_labels.is_empty() {
                "none".to_string()
            } else {
                scenario_labels.join(", ")
            },
            self.sample_count(),
            self.success_count(),
            self.failure_count(),
            north_hemispheres,
            south_hemispheres,
            equatorial_hemispheres,
            prime_meridian_longitudes,
            non_prime_meridian_longitudes,
            if formula_families.is_empty() {
                "none".to_string()
            } else {
                formula_families.join(", ")
            },
            if latitude_sensitive_systems.is_empty() {
                "none".to_string()
            } else {
                latitude_sensitive_systems.join(", ")
            },
            if latitude_sensitive_constraints.is_empty() {
                "none".to_string()
            } else {
                latitude_sensitive_constraints.join(", ")
            },
            self.house_systems.len(),
            self.catalog_label
        )
    }

    /// Validates that the report still reflects the expected corpus shape.
    pub fn validate(&self) -> Result<(), HouseValidationReportValidationError> {
        if self.scenarios.is_empty() {
            return Err(HouseValidationReportValidationError::EmptyScenarioList);
        }

        let expected_sample_count = self.house_systems.len();
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

            let normalized_label = label.to_ascii_lowercase();
            if !scenario_labels.insert(normalized_label) {
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

    /// Returns the compact release-facing summary line if validation succeeds.
    pub fn validated_summary_line(&self) -> Result<String, HouseValidationReportValidationError> {
        self.validate()?;
        Ok(self.summary_line())
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

/// Renders the default baseline house-validation corpus.
pub fn house_validation_report() -> HouseValidationReport {
    static CACHE: OnceLock<HouseValidationReport> = OnceLock::new();

    CACHE.get_or_init(HouseValidationReport::new).clone()
}

/// Renders the release house-validation corpus across all built-in systems.
pub fn release_house_validation_report() -> HouseValidationReport {
    static CACHE: OnceLock<HouseValidationReport> = OnceLock::new();

    CACHE.get_or_init(HouseValidationReport::release).clone()
}

/// Returns the compact baseline house-validation summary line.
pub fn house_validation_summary_for_report() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(|| house_validation_summary_line_for_report(&house_validation_report()))
        .clone()
}

/// Returns the compact report-facing summary line, or an unavailable message if validation fails.
pub fn house_validation_summary_line_for_report(report: &HouseValidationReport) -> String {
    match validated_house_validation_summary_line_for_report(report) {
        Ok(summary) => summary,
        Err(error) => format!("House validation corpus unavailable: {error}"),
    }
}

/// Returns the compact release-facing summary line if the release corpus validates.
pub fn validated_release_house_validation_summary_line_for_report(
) -> Result<String, HouseValidationReportValidationError> {
    release_house_validation_report().validated_summary_line()
}

/// Returns the compact release-facing summary line for the release house-validation corpus.
pub fn release_house_validation_summary_for_report() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(|| {
            let summary = match validated_release_house_validation_summary_line_for_report() {
                Ok(summary) => summary,
                Err(error) => return format!("House validation corpus unavailable: {error}"),
            };
            let house_code_aliases = match crate::validated_house_code_aliases_summary_for_report()
            {
                Ok(summary) => summary,
                Err(error) => return format!("House validation corpus unavailable: {error}"),
            };

            format!("{summary}; House code aliases: {house_code_aliases}")
        })
        .clone()
}

/// Returns the compact report-facing summary line if validation succeeds.
pub fn validated_house_validation_summary_line_for_report(
    report: &HouseValidationReport,
) -> Result<String, HouseValidationReportValidationError> {
    report.validated_summary_line()
}

// ── Task 7: house corpus + manifest parsers ───────────────────────────────────
// Task 8 lifts the #[cfg(test)] gating so validate_house_corpus() can use them.

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/houses-corpus/cusps.csv"
));

const CORPUS_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/houses-corpus/manifest.txt"
));

/// A single parsed row from the house-corpus CSV.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HouseCorpusRow {
    /// Unique chart identifier.
    pub(crate) chart_id: String,
    /// Julian Day (UT) for the chart.
    pub(crate) jd_ut: f64,
    /// Observer geodetic latitude, degrees.
    pub(crate) lat_deg: f64,
    /// Observer geodetic longitude, degrees (east-positive).
    pub(crate) lon_deg: f64,
    /// Observer elevation above sea level, metres.
    pub(crate) elev_m: f64,
    /// Pleiades `HouseSystem` variant name (e.g. `"Placidus"`).
    pub(crate) system_code: String,
    /// Twelve house cusps, degrees [0..12].
    pub(crate) cusps: [f64; 12],
    /// Ascendant, degrees.
    pub(crate) asc: f64,
    /// Midheaven (MC), degrees.
    pub(crate) mc: f64,
}

/// Errors produced while parsing the house-corpus CSV or manifest, or while
/// running the numeric-residual gate.
#[derive(Clone, Debug, PartialEq)]
pub enum HouseCorpusError {
    /// A CSV data row could not be parsed.
    MalformedRow {
        /// One-based data-row number (skipping header/comment lines).
        row: usize,
        /// The raw CSV line.
        line: String,
        /// Description of what was malformed.
        reason: String,
    },
    /// The manifest text could not be parsed.
    MalformedManifest {
        /// Description of what was malformed.
        reason: String,
    },
    /// The corpus CSV checksum does not match the manifest pin.
    ChecksumMismatch {
        /// Pinned checksum from the manifest.
        expected: u64,
        /// Recomputed checksum of the embedded CSV.
        actual: u64,
    },
    /// A manifest field does not match the corpus shape.
    ManifestDrift {
        /// The field that drifted (e.g. "rows", "completeness").
        field: String,
        /// Expected value.
        expected: String,
        /// Actual value.
        actual: String,
    },
    /// A corpus row references a system code that cannot be resolved to a HouseSystem.
    UnknownSystemCode {
        /// One-based data-row number.
        row: usize,
        /// Unrecognised system code string.
        code: String,
    },
    /// A cusp residual exceeded the per-family ceiling.
    ///
    /// `cusp` encoding: 1–12 = house cusp number; 0 = Ascendant; 13 = Midheaven.
    CeilingExceeded {
        /// One-based data-row number.
        row: usize,
        /// System name (e.g. "Placidus").
        system: String,
        /// Angle slot: 1–12 = house cusp; 0 = Ascendant; 13 = Midheaven.
        cusp: usize,
        /// Recomputed pleiades value, degrees.
        got: f64,
        /// SE reference value, degrees.
        want: f64,
        /// Circular residual, arcseconds.
        residual_arcsec: f64,
        /// Ceiling that was exceeded, arcseconds.
        ceiling_arcsec: f64,
    },
    /// A corpus row's `calculate_houses` call returned an error.
    CalculationFailed {
        /// One-based data-row number.
        row: usize,
        /// System name (e.g. "Placidus").
        system: String,
        /// Description of the calculation failure.
        reason: String,
    },
    /// A latitude-sensitive system did not reject a strictly-out-of-bounds latitude.
    MissingStrictRejection {
        /// System name (e.g. "Placidus").
        system: String,
        /// Observer latitude that should have triggered rejection (degrees).
        lat: f64,
    },
    /// The SE-compat fallback produced cusps that differ from a direct Porphyry calculation.
    FallbackMismatch {
        /// System name (e.g. "Koch").
        system: String,
        /// Observer latitude at which the fallback was exercised (degrees).
        lat: f64,
        /// Human-readable explanation of the mismatch.
        reason: String,
    },
}

impl fmt::Display for HouseCorpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRow { row, line, reason } => {
                write!(
                    f,
                    "house corpus row {row} is malformed ({reason}): {line:?}"
                )
            }
            Self::MalformedManifest { reason } => {
                write!(f, "house corpus manifest is malformed: {reason}")
            }
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "house corpus checksum mismatch: expected {expected}, got {actual}"
                )
            }
            Self::ManifestDrift {
                field,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "house corpus manifest drift on field '{field}': expected {expected:?}, got {actual:?}"
                )
            }
            Self::UnknownSystemCode { row, code } => {
                write!(f, "house corpus row {row}: unknown system code {code:?}")
            }
            Self::CeilingExceeded {
                row,
                system,
                cusp,
                got,
                want,
                residual_arcsec,
                ceiling_arcsec,
            } => {
                let slot_label = match *cusp {
                    0 => "Ascendant".to_string(),
                    13 => "Midheaven".to_string(),
                    n => format!("cusp {n}"),
                };
                write!(
                    f,
                    "house corpus row {row} ({system} {slot_label}): residual {residual_arcsec:.3}\u{2033} > ceiling {ceiling_arcsec:.1}\u{2033} (got={got:.6}\u{00b0}, want={want:.6}\u{00b0})"
                )
            }
            Self::CalculationFailed {
                row,
                system,
                reason,
            } => {
                write!(
                    f,
                    "house corpus row {row} ({system}): calculate_houses failed: {reason}"
                )
            }
            Self::MissingStrictRejection { system, lat } => {
                write!(
                    f,
                    "house gate: {system} at lat={lat}\u{00b0} should be rejected by the strict high-latitude policy but was not"
                )
            }
            Self::FallbackMismatch {
                system,
                lat,
                reason,
            } => {
                write!(
                    f,
                    "house gate: {system} SE-compat fallback at lat={lat}\u{00b0} mismatch: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for HouseCorpusError {}

/// Parse the house-corpus CSV, skipping comment (`#`) and blank lines and the
/// header row (the line beginning with `chart_id,`).
///
/// Fails closed: any malformed or unparseable data row returns `Err(MalformedRow)`.
pub(crate) fn parse_house_corpus(csv: &str) -> Result<Vec<HouseCorpusRow>, HouseCorpusError> {
    let mut rows = Vec::new();
    let mut data_row = 0usize;

    for line in csv.lines() {
        let trimmed = line.trim();
        // Skip comment lines and blank lines.
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        // Skip the header row.
        if trimmed.starts_with("chart_id,") {
            continue;
        }

        data_row += 1;
        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() != 20 {
            return Err(HouseCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("expected 20 comma-separated fields, got {}", parts.len()),
            });
        }

        let chart_id = parts[0].trim().to_string();

        let jd_ut: f64 = parts[1]
            .trim()
            .parse()
            .map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("jd_ut {:?} is not a valid float", parts[1]),
            })?;

        let lat_deg: f64 = parts[2]
            .trim()
            .parse()
            .map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("lat_deg {:?} is not a valid float", parts[2]),
            })?;

        let lon_deg: f64 = parts[3]
            .trim()
            .parse()
            .map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("lon_deg {:?} is not a valid float", parts[3]),
            })?;

        let elev_m: f64 = parts[4]
            .trim()
            .parse()
            .map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("elev_m {:?} is not a valid float", parts[4]),
            })?;

        let system_code = parts[5].trim().to_string();

        let mut cusps = [0.0f64; 12];
        for (i, cusp) in cusps.iter_mut().enumerate() {
            let field = parts[6 + i];
            *cusp = field
                .trim()
                .parse()
                .map_err(|_| HouseCorpusError::MalformedRow {
                    row: data_row,
                    line: line.to_string(),
                    reason: format!("cusp field[{}] {:?} is not a valid float", 6 + i, field),
                })?;
        }

        let asc: f64 = parts[18]
            .trim()
            .parse()
            .map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("asc {:?} is not a valid float", parts[18]),
            })?;

        let mc: f64 = parts[19]
            .trim()
            .parse()
            .map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row,
                line: line.to_string(),
                reason: format!("mc {:?} is not a valid float", parts[19]),
            })?;

        rows.push(HouseCorpusRow {
            chart_id,
            jd_ut,
            lat_deg,
            lon_deg,
            elev_m,
            system_code,
            cusps,
            asc,
            mc,
        });
    }

    Ok(rows)
}

/// Parsed metadata from the house-corpus manifest.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HouseManifest {
    /// The reference engine used to generate the corpus (e.g. `"SwissEphemeris 2.10.03"`).
    pub(crate) reference_engine: String,
    /// The cross-check engine used (e.g. `"not-run"`).
    pub(crate) crosscheck: String,
    /// Number of data rows recorded in the manifest.
    pub(crate) rows: usize,
    /// FNV-1a-64 checksum of the corpus CSV.
    pub(crate) checksum: u64,
}

/// Parse the house-corpus manifest text.
///
/// Reads `#Reference-Engine:` and `#CrossCheck-Engine:` comment values, and the
/// `slice cusps file=cusps.csv role=cusps rows=<n> checksum=<u64>` line.
///
/// Fails closed on any missing or malformed field.
pub(crate) fn parse_house_manifest(text: &str) -> Result<HouseManifest, HouseCorpusError> {
    let mut reference_engine: Option<String> = None;
    let mut crosscheck: Option<String> = None;
    let mut rows: Option<usize> = None;
    let mut checksum: Option<u64> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("#Reference-Engine:") {
            reference_engine = Some(rest.trim().to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("#CrossCheck-Engine:") {
            crosscheck = Some(rest.trim().to_string());
            continue;
        }
        // Parse the `slice cusps file=cusps.csv role=cusps rows=<n> checksum=<u64>` line.
        if trimmed.starts_with("slice ") {
            for token in trimmed.split_whitespace() {
                if let Some(val) = token.strip_prefix("rows=") {
                    rows = Some(val.parse::<usize>().map_err(|_| {
                        HouseCorpusError::MalformedManifest {
                            reason: format!("rows value {val:?} is not a valid usize"),
                        }
                    })?);
                } else if let Some(val) = token.strip_prefix("checksum=") {
                    checksum = Some(val.parse::<u64>().map_err(|_| {
                        HouseCorpusError::MalformedManifest {
                            reason: format!("checksum value {val:?} is not a valid u64"),
                        }
                    })?);
                }
            }
        }
    }

    let reference_engine = reference_engine.ok_or_else(|| HouseCorpusError::MalformedManifest {
        reason: "#Reference-Engine comment not found".to_string(),
    })?;
    let crosscheck = crosscheck.ok_or_else(|| HouseCorpusError::MalformedManifest {
        reason: "#CrossCheck-Engine comment not found".to_string(),
    })?;
    let rows = rows.ok_or_else(|| HouseCorpusError::MalformedManifest {
        reason: "rows= key not found in slice line".to_string(),
    })?;
    let checksum = checksum.ok_or_else(|| HouseCorpusError::MalformedManifest {
        reason: "checksum= key not found in slice line".to_string(),
    })?;

    Ok(HouseManifest {
        reference_engine,
        crosscheck,
        rows,
        checksum,
    })
}

// ── Task 8: numeric-residual gate ─────────────────────────────────────────────

/// Summary report produced by [`validate_house_corpus`].
#[derive(Clone, Debug, PartialEq)]
pub struct HouseCorpusReport {
    /// Number of corpus rows validated.
    pub rows_validated: usize,
    /// Number of distinct house systems checked.
    pub systems_checked: usize,
    /// Maximum circular cusp residual (arcseconds) across all rows and cusps.
    pub max_cusp_residual_arcsec: f64,
    /// Cross-check engine field from the manifest.
    pub crosscheck: String,
    /// Compact one-line summary.
    pub summary_line: String,
    /// The distinct typed house systems the corpus validated.
    pub validated_systems: Vec<pleiades_core::HouseSystem>,
}

impl HouseCorpusReport {
    /// Returns the compact one-line summary.
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }

    /// Returns the distinct house systems validated by the corpus.
    pub fn validated_systems(&self) -> &[pleiades_core::HouseSystem] {
        &self.validated_systems
    }
}

impl fmt::Display for HouseCorpusReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

/// Maps the corpus `system_code` string (the pleiades `HouseSystem` variant name)
/// to the corresponding typed `HouseSystem`.
///
/// Uses the exact variant names emitted by the SE harness (Task 6 schema decision:
/// strings, not chars, to disambiguate Meridian vs Axial which both map to SE 'X').
fn system_for_code(code: &str) -> Option<pleiades_core::HouseSystem> {
    use pleiades_core::HouseSystem;
    match code {
        "Placidus" => Some(HouseSystem::Placidus),
        "Koch" => Some(HouseSystem::Koch),
        "Porphyry" => Some(HouseSystem::Porphyry),
        "Regiomontanus" => Some(HouseSystem::Regiomontanus),
        "Campanus" => Some(HouseSystem::Campanus),
        "Equal" => Some(HouseSystem::Equal),
        "WholeSign" => Some(HouseSystem::WholeSign),
        "Alcabitius" => Some(HouseSystem::Alcabitius),
        "Meridian" => Some(HouseSystem::Meridian),
        "Axial" => Some(HouseSystem::Axial),
        "Topocentric" => Some(HouseSystem::Topocentric),
        "Morinus" => Some(HouseSystem::Morinus),
        _ => None,
    }
}

/// Returns the corpus string key for a `HouseSystem` (inverse of `system_for_code`).
fn code_for_system(system: &pleiades_core::HouseSystem) -> &'static str {
    use pleiades_core::HouseSystem;
    match system {
        HouseSystem::Placidus => "Placidus",
        HouseSystem::Koch => "Koch",
        HouseSystem::Porphyry => "Porphyry",
        HouseSystem::Regiomontanus => "Regiomontanus",
        HouseSystem::Campanus => "Campanus",
        HouseSystem::Equal => "Equal",
        HouseSystem::WholeSign => "WholeSign",
        HouseSystem::Alcabitius => "Alcabitius",
        HouseSystem::Meridian => "Meridian",
        HouseSystem::Axial => "Axial",
        HouseSystem::Topocentric => "Topocentric",
        HouseSystem::Morinus => "Morinus",
        // Non-baseline systems: return empty string (no corpus rows expected).
        _ => "",
    }
}

/// Circular absolute residual in arcseconds between two ecliptic degree values.
fn wrap_arcsec(got: f64, want: f64) -> f64 {
    let mut d = (got - want).abs();
    if d > 180.0 {
        d = 360.0 - d;
    }
    d * 3600.0
}

/// Recomputes pleiades house cusps for a corpus row and returns the snapshot.
fn recompute_pleiades(
    row_num: usize,
    row: &HouseCorpusRow,
    system: &pleiades_core::HouseSystem,
) -> Result<pleiades_core::HouseSnapshot, HouseCorpusError> {
    let observer = ObserverLocation::new(
        Latitude::from_degrees(row.lat_deg),
        Longitude::from_degrees(row.lon_deg),
        Some(row.elev_m),
    );
    // The corpus JD is recorded as UT (from the SE harness).
    // The pleiades engine accepts JD via TimeScale::Tt for the obliquity
    // calculation; for house cusps, which are analytic (no nutation table),
    // TT ≈ UT at this precision. This matches how all prior phase-5 tests
    // set up their instants (J2000 TT = JD 2451545.0).
    let instant = Instant::new(JulianDay::from_days(row.jd_ut), TimeScale::Tt);
    let request = HouseRequest::new(instant, observer, system.clone());
    calculate_houses(&request).map_err(|e| HouseCorpusError::CalculationFailed {
        row: row_num,
        system: row.system_code.clone(),
        reason: e.to_string(),
    })
}

/// Returns the fixed gate instant (J2000.0 TT = JD 2451545.0).
///
/// Used for strict-rejection and SE-compat fallback assertions inside
/// [`validate_house_corpus`].
fn gate_instant() -> Instant {
    Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
}

/// Runs the house-corpus numeric-residual gate.
///
/// Steps:
/// 1. Verify the embedded corpus CSV checksum against the manifest pin.
/// 2. Verify the row count matches the manifest.
/// 3. Verify completeness: every baseline system has ≥1 corpus row.
/// 4. For each row, recompute pleiades cusps and check every cusp residual
///    against the per-formula-family ceiling.
///
/// Returns a [`HouseCorpusReport`] on success, or the first [`HouseCorpusError`]
/// encountered on failure.
pub fn validate_house_corpus() -> Result<HouseCorpusReport, HouseCorpusError> {
    // 1. Checksum gate.
    let actual_checksum = pleiades_apparent::fnv1a64(CORPUS_CSV);
    let manifest = parse_house_manifest(CORPUS_MANIFEST)?;
    if actual_checksum != manifest.checksum {
        return Err(HouseCorpusError::ChecksumMismatch {
            expected: manifest.checksum,
            actual: actual_checksum,
        });
    }

    // 2. Row count gate.
    let rows = parse_house_corpus(CORPUS_CSV)?;
    if rows.len() != manifest.rows {
        return Err(HouseCorpusError::ManifestDrift {
            field: "rows".into(),
            expected: manifest.rows.to_string(),
            actual: rows.len().to_string(),
        });
    }

    // 3. Completeness: every baseline system must appear at least once.
    let baseline = pleiades_houses::baseline_house_systems();
    for descriptor in baseline {
        let code = code_for_system(&descriptor.system);
        if code.is_empty() {
            // System has no corpus code mapping — skip (not a baseline corpus system).
            continue;
        }
        if !rows.iter().any(|r| r.system_code == code) {
            return Err(HouseCorpusError::ManifestDrift {
                field: "completeness".into(),
                expected: format!("rows for {code}"),
                actual: "missing".into(),
            });
        }
    }

    // 4. Numeric residual loop.
    let mut max_cusp_residual_arcsec = 0.0_f64;
    for (idx, row) in rows.iter().enumerate() {
        let data_row = idx + 1;
        let system = system_for_code(&row.system_code).ok_or_else(|| {
            HouseCorpusError::UnknownSystemCode {
                row: data_row,
                code: row.system_code.clone(),
            }
        })?;

        let family = pleiades_houses::descriptor(&system)
            .map(|d| d.formula_family())
            .unwrap_or(pleiades_houses::HouseFormulaFamily::Unknown);
        let ceiling = pleiades_houses::thresholds::house_family_ceiling(family);

        let snapshot = recompute_pleiades(data_row, row, &system)?;

        // Check the 12 house cusps.
        for (i, &want) in row.cusps.iter().enumerate() {
            let got = snapshot.cusps[i].degrees();
            let resid = wrap_arcsec(got, want);
            if resid > max_cusp_residual_arcsec {
                max_cusp_residual_arcsec = resid;
            }
            if resid > ceiling.cusp_arcsec {
                return Err(HouseCorpusError::CeilingExceeded {
                    row: data_row,
                    system: row.system_code.clone(),
                    cusp: i + 1,
                    got,
                    want,
                    residual_arcsec: resid,
                    ceiling_arcsec: ceiling.cusp_arcsec,
                });
            }
        }

        // Check Ascendant against the angle ceiling.
        // cusp encoding: 0 = Ascendant (see CeilingExceeded doc comment).
        let asc_got = snapshot.angles.ascendant.degrees();
        let asc_resid = wrap_arcsec(asc_got, row.asc);
        if asc_resid > max_cusp_residual_arcsec {
            max_cusp_residual_arcsec = asc_resid;
        }
        if asc_resid > ceiling.angle_arcsec {
            return Err(HouseCorpusError::CeilingExceeded {
                row: data_row,
                system: row.system_code.clone(),
                cusp: 0, // 0 = Ascendant
                got: asc_got,
                want: row.asc,
                residual_arcsec: asc_resid,
                ceiling_arcsec: ceiling.angle_arcsec,
            });
        }
        // Check Midheaven against the angle ceiling.
        // cusp encoding: 13 = Midheaven (see CeilingExceeded doc comment).
        let mc_got = snapshot.angles.midheaven.degrees();
        let mc_resid = wrap_arcsec(mc_got, row.mc);
        if mc_resid > max_cusp_residual_arcsec {
            max_cusp_residual_arcsec = mc_resid;
        }
        if mc_resid > ceiling.angle_arcsec {
            return Err(HouseCorpusError::CeilingExceeded {
                row: data_row,
                system: row.system_code.clone(),
                cusp: 13, // 13 = Midheaven
                got: mc_got,
                want: row.mc,
                residual_arcsec: mc_resid,
                ceiling_arcsec: ceiling.angle_arcsec,
            });
        }
    }

    // 5. Strict-rejection assertions: every latitude-sensitive baseline system
    //    must reject beyond its bound under the default Strict policy.
    //
    // 6. SE-compat fallback assertions: the SwissEphemerisFallback policy must
    //    succeed and produce cusps equal to a direct Porphyry calculation.
    for descriptor in baseline_house_systems() {
        if descriptor.max_abs_latitude_deg.is_none() {
            continue;
        }
        for lat in [70.0_f64, 80.0] {
            let observer = ObserverLocation::new(
                Latitude::from_degrees(lat),
                Longitude::from_degrees(0.0),
                Some(0.0),
            );
            let req =
                HouseRequest::new(gate_instant(), observer.clone(), descriptor.system.clone());
            // Strict (default) policy must reject.
            if calculate_houses(&req).is_ok() {
                return Err(HouseCorpusError::MissingStrictRejection {
                    system: format!("{:?}", descriptor.system),
                    lat,
                });
            }
            // SE-compat fallback must succeed and equal Porphyry.
            let fb_req = req
                .clone()
                .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);
            let fb = calculate_houses(&fb_req).map_err(|e| HouseCorpusError::FallbackMismatch {
                system: format!("{:?}", descriptor.system),
                lat,
                reason: format!("fallback calculate_houses failed: {e}"),
            })?;
            let po = calculate_houses(&HouseRequest::new(
                gate_instant(),
                observer.clone(),
                pleiades_core::HouseSystem::Porphyry,
            ))
            .map_err(|e| HouseCorpusError::FallbackMismatch {
                system: "Porphyry".into(),
                lat,
                reason: format!("porphyry calculate_houses failed: {e}"),
            })?;
            if fb.cusps != po.cusps {
                return Err(HouseCorpusError::FallbackMismatch {
                    system: format!("{:?}", descriptor.system),
                    lat,
                    reason: "fallback cusps differ from Porphyry".into(),
                });
            }
        }
    }

    let systems_checked = baseline.len();
    let summary_line = format!(
        "House gate: {} rows / {} systems, max cusp residual {:.3}\u{2033}, cross-check {}",
        rows.len(),
        systems_checked,
        max_cusp_residual_arcsec,
        manifest.crosscheck,
    );

    let mut validated_systems: Vec<pleiades_core::HouseSystem> = Vec::new();
    for row in &rows {
        if let Some(sys) = system_for_code(&row.system_code) {
            if !validated_systems.iter().any(|s| *s == sys) {
                validated_systems.push(sys);
            }
        }
    }

    Ok(HouseCorpusReport {
        rows_validated: rows.len(),
        systems_checked,
        max_cusp_residual_arcsec,
        crosscheck: manifest.crosscheck,
        summary_line,
        validated_systems,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_line_reports_scenario_and_latitude_sensitive_counts() {
        let report = house_validation_report();

        assert_eq!(report.scenarios.len(), 9);
        assert_eq!(
            report.sample_count(),
            report.scenarios.len() * baseline_house_systems().len()
        );
        assert_eq!(
            report.latitude_sensitive_systems(),
            vec!["Koch", "Placidus", "Topocentric"]
        );
        assert_eq!(
            report.formula_families(),
            vec!["Equal", "Whole Sign", "Quadrant", "Equatorial projection"]
        );
        assert_eq!(
            report.latitude_sensitive_constraints(),
            vec![
                "Koch [Quadrant system with documented high-latitude pathologies.]",
                "Placidus [Quadrant system; can fail or become unstable at extreme latitudes.]",
                "Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]",
            ]
        );
        assert_eq!(report.hemisphere_coverage(), (5, 3, 1));
        assert_eq!(report.longitude_coverage(), (2, 7));
        assert_eq!(
            report.scenario_labels(),
            vec![
                "Mid-latitude reference chart",
                "Western hemisphere reference chart",
                "Equatorial reference chart",
                "Polar stress chart",
                "Northern high-latitude stress chart",
                "Northern high-latitude mountain stress chart",
                "Southern high-latitude mountain stress chart",
                "Southern polar stress chart",
                "Southern hemisphere reference chart",
            ]
        );

        assert_eq!(
            report
                .scenarios
                .iter()
                .find(|scenario| scenario.label == "Northern high-latitude mountain stress chart")
                .expect("mountain scenario should exist")
                .observer
                .elevation_m,
            Some(2_000.0)
        );
        assert_eq!(
            report.summary_line(),
            "House validation corpus: 9 scenarios (Mid-latitude reference chart, Western hemisphere reference chart, Equatorial reference chart, Polar stress chart, Northern high-latitude stress chart, Northern high-latitude mountain stress chart, Southern high-latitude mountain stress chart, Southern polar stress chart, Southern hemisphere reference chart), 108 samples, 93 successes, 15 failures; hemisphere coverage: north=5, south=3, equatorial=1; longitude coverage: prime-meridian=2, non-prime-meridian=7; formula families: Equal, Whole Sign, Quadrant, Equatorial projection; latitude-sensitive systems: Koch, Placidus, Topocentric; constraints: Koch [Quadrant system with documented high-latitude pathologies.], Placidus [Quadrant system; can fail or become unstable at extreme latitudes.], Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]; implementation posture: 12 baseline systems validated"
        );
        assert_eq!(
            house_validation_summary_line_for_report(&report),
            report.summary_line()
        );
        assert_eq!(report.validated_summary_line(), Ok(report.summary_line()));
        assert_eq!(
            validated_house_validation_summary_line_for_report(&report),
            Ok(report.summary_line())
        );
        assert!(release_house_validation_summary_for_report().starts_with(
            &house_validation_summary_line_for_report(&release_house_validation_report())
        ));
        assert!(release_house_validation_summary_for_report().contains("House code aliases:"));
        assert_eq!(
            validated_release_house_validation_summary_line_for_report(),
            Ok(release_house_validation_report().summary_line())
        );
        assert_eq!(report.validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_case_insensitive_duplicate_scenario_labels() {
        let mut report = HouseValidationReport::new();
        report.scenarios[1].label = "mid-latitude reference chart";

        let error = report
            .validate()
            .expect_err("case-insensitive duplicate scenario labels should fail validation");

        assert!(matches!(
            error,
            HouseValidationReportValidationError::DuplicateScenarioLabel {
                label: "mid-latitude reference chart"
            }
        ));
        assert_eq!(
            house_validation_summary_line_for_report(&report),
            "House validation corpus unavailable: house validation scenario label 'mid-latitude reference chart' is duplicated"
        );
        assert_eq!(
            validated_house_validation_summary_line_for_report(&report),
            Err(error)
        );
    }

    #[test]
    fn release_report_expands_to_all_built_in_house_systems() {
        let report = release_house_validation_report();

        assert_eq!(report.scenarios.len(), 9);
        assert_eq!(
            report.sample_count(),
            report.scenarios.len() * built_in_house_systems().len()
        );
        assert_eq!(report.failure_count(), 40);
        assert_eq!(report.validate(), Ok(()));
        assert_eq!(
            report.latitude_sensitive_systems(),
            vec![
                "APC",
                "Gauquelin sectors",
                "Horizon/Azimuth",
                "Koch",
                "Krusinski-Pisa-Goelzer",
                "Placidus",
                "Sunshine",
                "Topocentric",
            ]
        );
        assert_eq!(
            report.formula_families(),
            vec![
                "Equal",
                "Whole Sign",
                "Quadrant",
                "Equatorial projection",
                "Great-circle",
                "Solar arc",
                "Sector",
            ]
        );
        assert_eq!(
            report.summary_line(),
            "House validation corpus: 9 scenarios (Mid-latitude reference chart, Western hemisphere reference chart, Equatorial reference chart, Polar stress chart, Northern high-latitude stress chart, Northern high-latitude mountain stress chart, Southern high-latitude mountain stress chart, Southern polar stress chart, Southern hemisphere reference chart), 225 samples, 185 successes, 40 failures; hemisphere coverage: north=5, south=3, equatorial=1; longitude coverage: prime-meridian=2, non-prime-meridian=7; formula families: Equal, Whole Sign, Quadrant, Equatorial projection, Great-circle, Solar arc, Sector; latitude-sensitive systems: APC, Gauquelin sectors, Horizon/Azimuth, Koch, Krusinski-Pisa-Goelzer, Placidus, Sunshine, Topocentric; constraints: APC [APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.], Gauquelin sectors [Thirty-six sectors used by the Gauquelin-sector family.], Horizon/Azimuth [Azimuthal house system that anchors house 1 due East and house 10 at the MC.], Koch [Quadrant system with documented high-latitude pathologies.], Krusinski-Pisa-Goelzer [Great-circle house system centered on the ascendant and zenith; latitude-sensitive near the poles.], Placidus [Quadrant system; can fail or become unstable at extreme latitudes.], Sunshine [Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.], Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]; implementation posture: 25 built-in systems validated"
        );
    }

    #[test]
    fn validate_rejects_drifted_corpus_shapes() {
        let report = HouseValidationReport {
            catalog_label: "baseline",
            house_systems: baseline_house_systems(),
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

    // ── Task 7 tests: corpus parser + manifest parser ─────────────────────────

    const SAMPLE: &str = "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc\n\
c0,2451545,0,0,0,Placidus,1,2,3,4,5,6,7,8,9,10,11,12,1.5,10.5\n";

    #[test]
    fn parses_a_well_formed_row() {
        let rows = parse_house_corpus(SAMPLE).expect("valid");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].system_code, "Placidus");
        assert_eq!(rows[0].cusps[0], 1.0);
        assert_eq!(rows[0].cusps[11], 12.0);
        assert_eq!(rows[0].asc, 1.5);
    }

    #[test]
    fn rejects_short_row() {
        let bad = "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc\nc0,1,2,3\n";
        assert!(matches!(
            parse_house_corpus(bad),
            Err(HouseCorpusError::MalformedRow { .. })
        ));
    }

    #[test]
    fn parses_manifest_fields() {
        let m = "#Pleiades House Reference Corpus Manifest\n#Reference-Engine: SwissEphemeris 2.10.03\n#CrossCheck-Engine: not-run\nslice cusps file=cusps.csv role=cusps rows=55 checksum=12345\n";
        let parsed = parse_house_manifest(m).expect("valid manifest");
        assert_eq!(parsed.rows, 55);
        assert_eq!(parsed.checksum, 12345);
        assert_eq!(parsed.crosscheck, "not-run");
    }

    #[test]
    fn committed_corpus_parses_and_matches_manifest_row_count() {
        let rows =
            parse_house_corpus(CORPUS_CSV).expect("committed corpus CSV must be well-formed");
        let manifest =
            parse_house_manifest(CORPUS_MANIFEST).expect("committed manifest must be well-formed");

        assert!(!rows.is_empty(), "committed corpus must contain data rows");
        assert_eq!(
            rows.len(),
            manifest.rows,
            "manifest row count must match parsed corpus row count"
        );
        assert_eq!(manifest.reference_engine, "SwissEphemeris 2.10.03");
        // CrossCheck-Engine is opaque provenance text; the gate never fails on its value.
        // Updated from "not-run" when Astrolog 7.70 cross-check was run (Task 12).
        assert_eq!(
            manifest.crosscheck,
            "Astrolog 7.70 (patched, hardeningDisable=all)"
        );
    }

    // ── Task 8 tests: numeric-residual gate ───────────────────────────────────

    #[test]
    fn gate_passes_over_committed_corpus() {
        let report =
            validate_house_corpus().expect("committed house corpus must validate within ceilings");
        assert_eq!(
            report.rows_validated, 60,
            "corpus must have exactly 60 rows"
        );
        assert!(
            report.max_cusp_residual_arcsec.is_finite(),
            "max cusp residual must be finite"
        );
        assert!(
            report.max_cusp_residual_arcsec < 30.0,
            "max cusp residual {:.3}\" must be below the 30\" sanity bound",
            report.max_cusp_residual_arcsec
        );
    }

    #[test]
    fn checksum_drift_fails_closed() {
        let actual = pleiades_apparent::fnv1a64(CORPUS_CSV);
        let manifest = parse_house_manifest(CORPUS_MANIFEST).unwrap();
        assert_eq!(
            actual, manifest.checksum,
            "corpus checksum drifted from manifest pin"
        );
    }

    #[test]
    fn strict_rejection_fires_above_polar_circle() {
        use pleiades_core::{calculate_houses, HouseRequest, HouseSystem};
        for lat in [70.0_f64, 80.0] {
            let observer = ObserverLocation::new(
                Latitude::from_degrees(lat),
                Longitude::from_degrees(0.0),
                None,
            );
            let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
            let req = HouseRequest::new(instant, observer, HouseSystem::Placidus);
            assert!(
                calculate_houses(&req).is_err(),
                "Placidus at {lat}\u{00b0} must be rejected (strict high-latitude policy)"
            );
        }
    }

    // ── Task 9 tests: strict-rejection + SE-compat fallback in the gate ───────

    #[test]
    fn gate_asserts_strict_and_fallback_paths() {
        // The full gate must internally exercise strict rejection + fallback;
        // a regression in either must fail the gate.
        let report = validate_house_corpus().expect("gate passes");
        assert!(report.rows_validated > 0);
    }

    #[test]
    fn fallback_equals_porphyry_in_gate_helper() {
        use pleiades_core::HouseSystem;
        use pleiades_houses::{
            calculate_houses as ph_calculate_houses, HighLatitudePolicy,
            HouseRequest as PhHouseRequest,
        };
        let obs = ObserverLocation::new(
            Latitude::from_degrees(80.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let fb = ph_calculate_houses(
            &PhHouseRequest::new(gate_instant(), obs.clone(), HouseSystem::Koch)
                .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback),
        )
        .expect("SE-compat fallback must succeed beyond bound");
        let po = ph_calculate_houses(&PhHouseRequest::new(
            gate_instant(),
            obs,
            HouseSystem::Porphyry,
        ))
        .expect("Porphyry is defined at all latitudes");
        assert_eq!(fb.cusps, po.cusps);
    }

    #[test]
    fn corpus_report_exposes_twelve_validated_systems() {
        let report = validate_house_corpus().expect("house corpus gate passes");
        assert_eq!(report.validated_systems().len(), 12);
        assert!(report
            .validated_systems()
            .iter()
            .any(|s| *s == pleiades_core::HouseSystem::Placidus));
    }
}
