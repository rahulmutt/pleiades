use core::fmt;
use std::sync::OnceLock;

use pleiades_backend::{CelestialBody, EphemerisRequest};
use pleiades_types::CoordinateFrame;

use crate::*;

pub(crate) const PRODUCTION_GENERATION_BOUNDARY_COVERAGE: &str =
    "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.";

pub(crate) const PRODUCTION_GENERATION_QUARTER_DAY_EPOCHS: [f64; 2] = [2_451_915.25, 2_451_915.75];
pub(crate) fn production_generation_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            independent_holdout_snapshot_entries()
                .into_iter()
                .flatten()
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

pub(crate) fn production_generation_snapshot_body_list() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = production_generation_snapshot_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

pub(crate) fn production_generation_boundary_body_list() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = production_generation_boundary_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

pub(crate) fn production_generation_snapshot_bodies() -> &'static [CelestialBody] {
    production_generation_snapshot_body_list()
}

fn extend_unique_snapshot_entries(merged: &mut Vec<SnapshotEntry>, entries: &[SnapshotEntry]) {
    let mut seen = merged
        .iter()
        .map(|entry| {
            (
                entry.body.to_string(),
                entry.epoch.julian_day.days().to_bits(),
            )
        })
        .collect::<BTreeSet<_>>();

    for entry in entries {
        let key = (
            entry.body.to_string(),
            entry.epoch.julian_day.days().to_bits(),
        );
        if seen.insert(key) {
            merged.push(entry.clone());
        }
    }
}

/// Returns the production-generation snapshot corpus used for boundary-coverage checks.
///
/// The merged corpus keeps the checked-in reference rows first, then appends the
/// boundary overlay while skipping overlapping body/epoch pairs that are already
/// covered by the reference snapshot.
pub fn production_generation_snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            let mut merged = Vec::new();
            if let Some(reference_entries) = snapshot_entries() {
                extend_unique_snapshot_entries(&mut merged, reference_entries);
            }
            if let Some(boundary_entries) = production_generation_boundary_entries() {
                extend_unique_snapshot_entries(&mut merged, boundary_entries);
            }
            merged
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Returns the production-generation request corpus in the requested frame.
pub fn production_generation_snapshot_requests(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_snapshot_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`production_generation_snapshot_requests`].
#[doc(alias = "production_generation_snapshot_requests")]
pub fn production_generation_snapshot_request_corpus(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_snapshot_requests(frame)
}

/// Returns the production-generation boundary-overlay request corpus in the requested frame.
pub fn production_generation_boundary_requests(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_boundary_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`production_generation_boundary_requests`].
#[doc(alias = "production_generation_boundary_requests")]
pub fn production_generation_boundary_request_corpus(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_boundary_requests(frame)
}

/// A compact coverage summary for the production-generation boundary overlay.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProductionGenerationBoundarySummary {
    /// Total number of parsed boundary-overlay rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the boundary overlay.
    pub body_count: usize,
    /// Bodies covered by the boundary overlay in first-seen order.
    pub bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs covered by the boundary overlay.
    pub epoch_count: usize,
    /// Earliest epoch represented in the boundary overlay.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the boundary overlay.
    pub latest_epoch: Instant,
}

/// Structured validation errors for the production-generation boundary overlay summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationBoundarySummaryValidationError {
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary body order drifted from the checked-in boundary overlay.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl ProductionGenerationBoundarySummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ProductionGenerationBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match body list length {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(f, "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(f, "body order mismatch at index {index}: expected {expected}, found {found}"),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "epoch range {}..{} is invalid",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundarySummaryValidationError {}

impl ProductionGenerationBoundarySummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ProductionGenerationBoundarySummaryValidationError> {
        if self.body_count == 0 {
            return Err(ProductionGenerationBoundarySummaryValidationError::MissingBodies);
        }
        if self.bodies.is_empty() {
            return Err(ProductionGenerationBoundarySummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                ProductionGenerationBoundarySummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    ProductionGenerationBoundarySummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .unwrap(),
                        second_index: index,
                        body: body.to_string(),
                    },
                );
            }
        }

        let expected_bodies = production_generation_boundary_body_list();
        if self.bodies != expected_bodies {
            let mismatch_index = self
                .bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.bodies.len().min(expected_bodies.len()));
            return Err(
                ProductionGenerationBoundarySummaryValidationError::BodyOrderMismatch {
                    index: mismatch_index,
                    expected: expected_bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of boundary body list>".to_string()),
                    found: self
                        .bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of summary body list>".to_string()),
                },
            );
        }

        if self.epoch_count == 0 {
            return Err(ProductionGenerationBoundarySummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                ProductionGenerationBoundarySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        if production_generation_boundary_summary().as_ref() != Some(self) {
            return Err(ProductionGenerationBoundarySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Production generation boundary overlay: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(self.bodies),
        )
    }

    /// Returns a compact summary line after validating the boundary overlay summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a compact coverage summary for the production-generation boundary overlay.
pub fn production_generation_boundary_summary() -> Option<ProductionGenerationBoundarySummary> {
    static SUMMARY: OnceLock<ProductionGenerationBoundarySummary> = OnceLock::new();
    Some(*SUMMARY.get_or_init(|| {
        let entries = production_generation_boundary_entries()
            .expect("production generation boundary entries should exist");

        let mut earliest_epoch = entries[0].epoch;
        let mut latest_epoch = entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        ProductionGenerationBoundarySummary {
            row_count: entries.len(),
            body_count: production_generation_boundary_body_list().len(),
            bodies: production_generation_boundary_body_list(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        }
    }))
}

/// Formats the production-generation boundary overlay for release-facing reporting.
pub fn format_production_generation_boundary_summary(
    summary: &ProductionGenerationBoundarySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing production-generation boundary overlay summary string.
pub fn production_generation_boundary_summary_for_report() -> String {
    match production_generation_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Production generation boundary overlay: unavailable ({error})"),
        },
        None => "Production generation boundary overlay: unavailable".to_string(),
    }
}

/// Returns the provenance summary shared by the production-generation boundary overlay.
#[doc(alias = "independent_holdout_source_summary")]
pub fn production_generation_boundary_source_summary() -> IndependentHoldoutSourceSummary {
    independent_holdout_source_summary()
}

/// Formats the provenance summary for the production-generation boundary overlay.
pub fn format_production_generation_boundary_source_summary(
    summary: &IndependentHoldoutSourceSummary,
) -> String {
    format!(
        "Production generation boundary overlay source: {}; evidence class={}; coverage={}; columns={}; redistribution={}; checksum=0x{:016x}; {}; time scale={}",
        summary.source,
        summary.evidence_class,
        summary.coverage,
        summary.columns,
        summary.redistribution,
        independent_holdout_snapshot_checksum(),
        summary.frame_treatment,
        summary.time_scale,
    )
}

fn independent_holdout_snapshot_checksum() -> u64 {
    static CHECKSUM: OnceLock<u64> = OnceLock::new();
    *CHECKSUM.get_or_init(|| {
        fnv1a64(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/independent_holdout_snapshot.csv"
            ))
            .as_bytes(),
        )
    })
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// Returns the release-facing provenance summary string for the production-generation boundary overlay.
pub fn production_generation_boundary_source_summary_for_report() -> String {
    let summary = production_generation_boundary_source_summary();
    match summary.validate() {
        Ok(()) => format_production_generation_boundary_source_summary(&summary),
        Err(error) => {
            format!("Production generation boundary overlay source: unavailable ({error})")
        }
    }
}

/// A single body-window slice inside the production-generation boundary overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryWindow {
    /// The boundary-overlay body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ProductionGenerationBoundaryWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

/// Compact release-facing summary for the production-generation boundary windows.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryWindowSummary {
    /// Number of boundary-overlay samples in the expanded source slice.
    pub sample_count: usize,
    /// Bodies covered by the expanded source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the expanded source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the expanded source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the expanded source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ProductionGenerationBoundaryWindow>,
}

/// Structured validation errors for a production-generation boundary window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductionGenerationBoundaryWindowSummaryValidationError {
    /// The summary no longer matches the checked-in boundary window slice.
    DerivedSummaryMismatch,
}

impl fmt::Display for ProductionGenerationBoundaryWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DerivedSummaryMismatch => write!(
                f,
                "the production-generation boundary window summary is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundaryWindowSummaryValidationError {}

impl ProductionGenerationBoundaryWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ProductionGenerationBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Production generation boundary windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the summary still matches the checked-in boundary window slice.
    pub fn validate(&self) -> Result<(), ProductionGenerationBoundaryWindowSummaryValidationError> {
        let Some(expected) = production_generation_boundary_window_summary_details() else {
            return Err(
                ProductionGenerationBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        };

        if self != &expected {
            return Err(
                ProductionGenerationBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns the validated summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundaryWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundaryWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn production_generation_boundary_window_summary_details(
) -> Option<ProductionGenerationBoundaryWindowSummary> {
    let entries = production_generation_boundary_entries()?;
    let mut windows = Vec::new();
    for body in production_generation_boundary_body_list() {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(ProductionGenerationBoundaryWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("production generation boundary windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("production generation boundary windows should not be empty after collection");

    Some(ProductionGenerationBoundaryWindowSummary {
        sample_count: entries.len(),
        sample_bodies: production_generation_boundary_body_list().to_vec(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the production-generation boundary windows.
pub fn production_generation_boundary_window_summary(
) -> Option<ProductionGenerationBoundaryWindowSummary> {
    static SUMMARY: OnceLock<ProductionGenerationBoundaryWindowSummary> = OnceLock::new();
    Some(
        SUMMARY
            .get_or_init(|| {
                production_generation_boundary_window_summary_details()
                    .expect("production generation boundary windows should exist")
            })
            .clone(),
    )
}

/// Returns the release-facing production-generation boundary window summary string.
pub fn production_generation_boundary_window_summary_for_report() -> String {
    match production_generation_boundary_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Production generation boundary windows: unavailable ({error})"),
        },
        None => "Production generation boundary windows: unavailable".to_string(),
    }
}

/// A compact body-class coverage summary for the production-generation boundary overlay used by validation and generation tooling.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryBodyClassCoverageSummary {
    /// Number of rows in the boundary overlay.
    pub row_count: usize,
    /// Number of major-body rows in the boundary overlay.
    pub major_body_row_count: usize,
    /// Major bodies covered by the boundary overlay in first-seen order.
    pub major_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the major-body subset.
    pub major_epoch_count: usize,
    /// Per-body windows covered by the major-body subset in first-seen order.
    pub major_windows: Vec<ProductionGenerationBoundaryWindow>,
    /// Number of selected-asteroid rows in the boundary overlay.
    pub asteroid_row_count: usize,
    /// Selected asteroids covered by the boundary overlay in first-seen order.
    pub asteroid_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the selected-asteroid subset.
    pub asteroid_epoch_count: usize,
    /// Per-body windows covered by the selected-asteroid subset in first-seen order.
    pub asteroid_windows: Vec<ProductionGenerationBoundaryWindow>,
}

/// Validation error for a production-generation boundary body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in boundary body-class coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the production-generation boundary body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError {}

impl ProductionGenerationBoundaryBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let major_windows = self
            .major_windows
            .iter()
            .map(ProductionGenerationBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let asteroid_windows = self
            .asteroid_windows
            .iter()
            .map(ProductionGenerationBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Production generation boundary body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
            self.major_body_row_count,
            self.major_bodies.len(),
            self.major_epoch_count,
            major_windows,
            self.asteroid_row_count,
            self.asteroid_bodies.len(),
            self.asteroid_epoch_count,
            asteroid_windows,
        )
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError> {
        let Some(expected) = production_generation_boundary_body_class_coverage_summary_details()
        else {
            return Err(
                ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self != &expected {
            return Err(
                ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundaryBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn production_generation_boundary_body_class_coverage_summary_details(
) -> Option<ProductionGenerationBoundaryBodyClassCoverageSummary> {
    let summary = production_generation_boundary_summary()?;
    let source_windows = production_generation_boundary_window_summary_details()?;
    let entries = production_generation_boundary_entries()?;

    let mut major_body_row_count = 0usize;
    let mut major_epochs = BTreeSet::new();
    let mut asteroid_row_count = 0usize;
    let mut asteroid_epochs = BTreeSet::new();

    for entry in entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        if is_comparison_body(&entry.body) {
            major_body_row_count += 1;
            major_epochs.insert(epoch_bits);
        }
        if is_reference_asteroid(&entry.body) {
            asteroid_row_count += 1;
            asteroid_epochs.insert(epoch_bits);
        }
    }

    Some(ProductionGenerationBoundaryBodyClassCoverageSummary {
        row_count: summary.row_count,
        major_body_row_count,
        major_bodies: summary
            .bodies
            .iter()
            .filter(|body| is_comparison_body(body))
            .cloned()
            .collect(),
        major_epoch_count: major_epochs.len(),
        major_windows: source_windows
            .windows
            .iter()
            .filter(|window| is_comparison_body(&window.body))
            .cloned()
            .collect(),
        asteroid_row_count,
        asteroid_bodies: summary
            .bodies
            .iter()
            .filter(|body| is_reference_asteroid(body))
            .cloned()
            .collect(),
        asteroid_epoch_count: asteroid_epochs.len(),
        asteroid_windows: source_windows
            .windows
            .iter()
            .filter(|window| is_reference_asteroid(&window.body))
            .cloned()
            .collect(),
    })
}

/// Returns the compact body-class coverage summary for the production-generation boundary overlay.
pub fn production_generation_boundary_body_class_coverage_summary(
) -> Option<ProductionGenerationBoundaryBodyClassCoverageSummary> {
    production_generation_boundary_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the production-generation boundary overlay.
pub fn production_generation_boundary_body_class_coverage_summary_for_report() -> String {
    match production_generation_boundary_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Production generation boundary body-class coverage: unavailable ({error})")
            }
        },
        None => "Production generation boundary body-class coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the production-generation boundary request corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryRequestCorpusSummary {
    /// Total number of generated requests.
    pub request_count: usize,
    /// Number of distinct bodies covered by the request corpus.
    pub body_count: usize,
    /// Bodies covered by the request corpus in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the request corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the request corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the request corpus.
    pub latest_epoch: Instant,
    /// Coordinate frame requested by the corpus.
    pub frame: CoordinateFrame,
    /// Time scale requested by the corpus.
    pub time_scale: TimeScale,
    /// Zodiac mode requested by the corpus.
    pub zodiac_mode: ZodiacMode,
    /// Apparentness requested by the corpus.
    pub apparentness: Apparentness,
}

/// Structured validation errors for the production-generation boundary request corpus summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationBoundaryRequestCorpusSummaryValidationError {
    /// The summary reported a field that is out of sync with the checked-in boundary request corpus.
    FieldOutOfSync { field: &'static str },
    /// The summary body order drifted from the checked-in boundary request corpus.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl ProductionGenerationBoundaryRequestCorpusSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::FieldOutOfSync { .. } => "field out of sync",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ProductionGenerationBoundaryRequestCorpusSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the production-generation boundary request corpus summary field `{field}` is out of sync with the current slice"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(f, "body order mismatch at index {index}: expected {expected}, found {found}"),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "epoch range {}..{} is invalid",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            Self::DerivedSummaryMismatch => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundaryRequestCorpusSummaryValidationError {}

impl ProductionGenerationBoundaryRequestCorpusSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Production generation boundary request corpus: {} requests (frame={}; time scale={}; zodiac mode={}; apparentness={}; observerless) across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.request_count,
            self.frame,
            self.time_scale,
            self.zodiac_mode,
            self.apparentness,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current boundary request corpus.
    pub fn validate(
        &self,
    ) -> Result<(), ProductionGenerationBoundaryRequestCorpusSummaryValidationError> {
        let Some(expected) =
            production_generation_boundary_request_corpus_summary_details(self.frame)
        else {
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::DerivedSummaryMismatch);
        };

        if self.request_count != expected.request_count {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "request_count",
                },
            );
        }
        if self.body_count != expected.body_count {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.bodies.as_slice() != expected.bodies.as_slice() {
            for (index, (expected, found)) in
                expected.bodies.iter().zip(self.bodies.iter()).enumerate()
            {
                if expected != found {
                    return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::BodyOrderMismatch {
                        index,
                        expected: expected.to_string(),
                        found: found.to_string(),
                    });
                }
            }
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.time_scale != expected.time_scale {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "time_scale",
                },
            );
        }
        if self.zodiac_mode != expected.zodiac_mode {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "zodiac_mode",
                },
            );
        }
        if self.apparentness != expected.apparentness {
            return Err(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "apparentness",
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current request corpus.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundaryRequestCorpusSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundaryRequestCorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn production_generation_boundary_request_corpus_summary_details(
    frame: CoordinateFrame,
) -> Option<ProductionGenerationBoundaryRequestCorpusSummary> {
    let entries = production_generation_boundary_entries()?;
    let requests = production_generation_boundary_requests(frame)?;
    if requests.is_empty() {
        return None;
    }

    let mut bodies = Vec::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = requests[0].instant;
    let mut latest_epoch = requests[0].instant;
    let time_scale = requests[0].instant.scale;

    for (request, entry) in requests.iter().zip(entries.iter()) {
        if request.body != entry.body
            || request.instant.julian_day.days() != entry.epoch.julian_day.days()
            || request.instant.scale != time_scale
            || request.frame != frame
            || request.zodiac_mode != ZodiacMode::Tropical
            || request.apparent != Apparentness::Mean
            || request.observer.is_some()
        {
            return None;
        }

        if !bodies.contains(&request.body) {
            bodies.push(request.body.clone());
        }
        epochs.insert(request.instant.julian_day.days().to_bits());
        if request.instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = request.instant;
        }
        if request.instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = request.instant;
        }
    }

    Some(ProductionGenerationBoundaryRequestCorpusSummary {
        request_count: requests.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
        frame,
        time_scale,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
    })
}

/// Returns the production-generation boundary request corpus summary in the requested frame.
pub fn production_generation_boundary_request_corpus_summary(
    frame: CoordinateFrame,
) -> Option<ProductionGenerationBoundaryRequestCorpusSummary> {
    static ECLIPTIC_SUMMARY: OnceLock<ProductionGenerationBoundaryRequestCorpusSummary> =
        OnceLock::new();
    if frame == CoordinateFrame::Ecliptic {
        Some(
            ECLIPTIC_SUMMARY
                .get_or_init(|| {
                    production_generation_boundary_request_corpus_summary_details(frame)
                        .expect("production generation boundary request corpus should exist")
                })
                .clone(),
        )
    } else {
        production_generation_boundary_request_corpus_summary_details(frame)
    }
}

/// Formats the production-generation boundary request corpus for release-facing reporting.
pub fn format_production_generation_boundary_request_corpus_summary(
    summary: &ProductionGenerationBoundaryRequestCorpusSummary,
) -> String {
    summary.summary_line()
}

/// Returns the validated release-facing production-generation boundary request corpus summary string.
fn validated_production_generation_boundary_request_corpus_summary_for_frame(
    frame: CoordinateFrame,
) -> Result<String, String> {
    let summary = production_generation_boundary_request_corpus_summary(frame)
        .ok_or_else(|| "production generation boundary request corpus unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the release-facing production-generation boundary request corpus summary string.
pub fn production_generation_boundary_request_corpus_summary_for_report() -> String {
    validated_production_generation_boundary_request_corpus_summary_for_frame(
        CoordinateFrame::Ecliptic,
    )
    .unwrap_or_else(|error| {
        format!("Production generation boundary request corpus: unavailable ({error})")
    })
}

/// Returns the release-facing equatorial production-generation boundary request corpus summary string.
pub fn production_generation_boundary_request_corpus_equatorial_summary_for_report() -> String {
    validated_production_generation_boundary_request_corpus_summary_for_frame(
        CoordinateFrame::Equatorial,
    )
    .unwrap_or_else(|error| {
        format!("Production generation boundary request corpus: unavailable ({error})")
    })
}

/// Returns the validated release-facing equatorial production-generation boundary request corpus summary string.
pub fn validated_production_generation_boundary_request_corpus_equatorial_summary_for_report(
) -> Result<String, String> {
    validated_production_generation_boundary_request_corpus_summary_for_frame(
        CoordinateFrame::Equatorial,
    )
}
