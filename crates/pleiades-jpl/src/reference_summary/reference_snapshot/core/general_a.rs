//! reference snapshot core general_a summaries.

use core::fmt;
use std::sync::OnceLock;

use pleiades_types::{Apparentness, CoordinateFrame, Instant, TimeScale, ZodiacMode};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

/// A compact coverage summary for the checked-in reference snapshot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceSnapshotSummary {
    /// Total number of parsed snapshot rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the snapshot.
    pub body_count: usize,
    /// Bodies covered by the snapshot in first-seen order.
    pub bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs covered by the snapshot.
    pub epoch_count: usize,
    /// Number of rows that belong to the reference asteroid subset.
    pub asteroid_row_count: usize,
    /// Earliest epoch represented in the snapshot.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the snapshot.
    pub latest_epoch: Instant,
}

/// Structured validation errors for a reference snapshot coverage summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotSummaryValidationError {
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
    /// The summary body order drifted from the checked-in reference snapshot.
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
    /// The asteroid row count exceeded the total row count.
    AsteroidRowCountExceedsRowCount {
        asteroid_row_count: usize,
        row_count: usize,
    },
    /// The summary's epoch count drifted away from the checked-in derived evidence.
    EpochCountMismatch {
        epoch_count: usize,
        derived_epoch_count: usize,
    },
    /// The summary's asteroid row count drifted away from the checked-in derived evidence.
    AsteroidRowCountMismatch {
        asteroid_row_count: usize,
        derived_asteroid_row_count: usize,
    },
    /// The summary's epoch range drifted away from the checked-in derived evidence.
    EpochRangeMismatch {
        earliest_epoch: Instant,
        latest_epoch: Instant,
        derived_earliest_epoch: Instant,
        derived_latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl ReferenceSnapshotSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::AsteroidRowCountExceedsRowCount { .. } => "asteroid row count exceeds row count",
            Self::EpochCountMismatch { .. } => "epoch count mismatch",
            Self::AsteroidRowCountMismatch { .. } => "asteroid row count mismatch",
            Self::EpochRangeMismatch { .. } => "epoch range mismatch",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ReferenceSnapshotSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBodies => f.write_str(self.label()),
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(
                f,
                "body count {body_count} does not match body list length {bodies_len}"
            ),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(
                f,
                "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "body order mismatch at index {index}: expected '{expected}' but found '{found}'"
            ),
            Self::MissingEpochs => f.write_str(self.label()),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range: earliest {} is after latest {}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            Self::AsteroidRowCountExceedsRowCount {
                asteroid_row_count,
                row_count,
            } => write!(
                f,
                "asteroid row count {asteroid_row_count} exceeds row count {row_count}"
            ),
            Self::EpochCountMismatch {
                epoch_count,
                derived_epoch_count,
            } => write!(
                f,
                "epoch count {epoch_count} does not match derived epoch count {derived_epoch_count}"
            ),
            Self::AsteroidRowCountMismatch {
                asteroid_row_count,
                derived_asteroid_row_count,
            } => write!(
                f,
                "asteroid row count {asteroid_row_count} does not match derived asteroid row count {derived_asteroid_row_count}"
            ),
            Self::EpochRangeMismatch {
                earliest_epoch,
                latest_epoch,
                derived_earliest_epoch,
                derived_latest_epoch,
            } => write!(
                f,
                "epoch range {}..{} does not match derived range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
                format_instant(*derived_earliest_epoch),
                format_instant(*derived_latest_epoch),
            ),
            Self::DerivedSummaryMismatch => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ReferenceSnapshotSummaryValidationError {}

/// Returns a compact coverage summary for the checked-in reference snapshot.
pub fn reference_snapshot_summary() -> Option<ReferenceSnapshotSummary> {
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut asteroid_row_count = 0usize;
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;
    let reference_asteroids = reference_asteroids();

    for entry in entries {
        bodies.insert(entry.body.to_string());
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if reference_asteroids.contains(&entry.body) {
            asteroid_row_count += 1;
        }
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }
    }

    Some(ReferenceSnapshotSummary {
        row_count: entries.len(),
        body_count: bodies.len(),
        bodies: snapshot_bodies(),
        epoch_count: epochs.len(),
        asteroid_row_count,
        earliest_epoch,
        latest_epoch,
    })
}

impl ReferenceSnapshotSummary {
    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotSummaryValidationError> {
        if self.bodies.is_empty() {
            return Err(ReferenceSnapshotSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(ReferenceSnapshotSummaryValidationError::BodyCountMismatch {
                body_count: self.body_count,
                bodies_len: self.bodies.len(),
            });
        }
        let mut seen_bodies = BTreeMap::new();
        for (index, body) in self.bodies.iter().enumerate() {
            let body_label = body.to_string();
            if let Some(first_index) = seen_bodies.insert(body_label.clone(), index) {
                return Err(ReferenceSnapshotSummaryValidationError::DuplicateBody {
                    first_index,
                    second_index: index,
                    body: body_label,
                });
            }
        }

        let expected_bodies = reference_bodies();
        if self.bodies != expected_bodies {
            let mismatch_index = self
                .bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.bodies.len().min(expected_bodies.len()));
            return Err(ReferenceSnapshotSummaryValidationError::BodyOrderMismatch {
                index: mismatch_index,
                expected: expected_bodies
                    .get(mismatch_index)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "<end of reference body list>".to_string()),
                found: self
                    .bodies
                    .get(mismatch_index)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "<end of summary body list>".to_string()),
            });
        }

        let derived_summary = reference_snapshot_summary()
            .ok_or(ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch)?;
        if self.row_count != derived_summary.row_count {
            return Err(ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch);
        }
        if self.epoch_count == 0 {
            return Err(ReferenceSnapshotSummaryValidationError::MissingEpochs);
        }
        if self.epoch_count != derived_summary.epoch_count {
            return Err(
                ReferenceSnapshotSummaryValidationError::EpochCountMismatch {
                    epoch_count: self.epoch_count,
                    derived_epoch_count: derived_summary.epoch_count,
                },
            );
        }
        if self.asteroid_row_count > self.row_count {
            return Err(
                ReferenceSnapshotSummaryValidationError::AsteroidRowCountExceedsRowCount {
                    asteroid_row_count: self.asteroid_row_count,
                    row_count: self.row_count,
                },
            );
        }
        if self.asteroid_row_count != derived_summary.asteroid_row_count {
            return Err(
                ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                    asteroid_row_count: self.asteroid_row_count,
                    derived_asteroid_row_count: derived_summary.asteroid_row_count,
                },
            );
        }
        if self.earliest_epoch != derived_summary.earliest_epoch
            || self.latest_epoch != derived_summary.latest_epoch
        {
            return Err(
                ReferenceSnapshotSummaryValidationError::EpochRangeMismatch {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                    derived_earliest_epoch: derived_summary.earliest_epoch,
                    derived_latest_epoch: derived_summary.latest_epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot coverage: {} rows across {} bodies and {} epochs ({} asteroid rows; {}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            self.asteroid_row_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(self.bodies),
        )
    }

    /// Returns a compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in reference snapshot coverage for release-facing reporting.
pub fn format_reference_snapshot_summary(summary: &ReferenceSnapshotSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing reference snapshot coverage summary string.
pub fn reference_snapshot_summary_for_report() -> String {
    let summary_line = match reference_snapshot_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference snapshot coverage: unavailable ({error})"),
        },
        None => "Reference snapshot coverage: unavailable".to_string(),
    };

    let summary_lines = [
        reference_snapshot_1500_selected_body_boundary_summary_for_report(),
        reference_snapshot_2268932_selected_body_boundary_summary_for_report(),
        reference_snapshot_1600_selected_body_boundary_summary_for_report(),
        reference_snapshot_2305457_selected_body_boundary_summary_for_report(),
        reference_snapshot_source_summary_for_report(),
        reference_snapshot_source_window_summary_for_report(),
        reference_snapshot_equatorial_parity_summary_for_report(),
        reference_snapshot_major_body_bridge_summary_for_report(),
        reference_snapshot_batch_parity_summary_for_report(),
        reference_snapshot_1749_major_body_boundary_summary_for_report(),
        reference_snapshot_2360233_major_body_boundary_summary_for_report(),
        reference_snapshot_early_major_body_boundary_summary_for_report(),
        reference_snapshot_1750_selected_body_boundary_summary_for_report(),
        reference_snapshot_1750_major_body_interior_summary_for_report(),
        reference_snapshot_1800_major_body_boundary_summary_for_report(),
        reference_snapshot_2378499_major_body_boundary_summary_for_report(),
        reference_snapshot_1900_selected_body_boundary_summary_for_report(),
        reference_snapshot_2415020_selected_body_boundary_summary_for_report(),
        reference_snapshot_lunar_boundary_summary_for_report(),
        reference_snapshot_high_curvature_summary_for_report(),
        reference_snapshot_high_curvature_window_summary_for_report(),
        reference_snapshot_high_curvature_epoch_coverage_summary_for_report(),
        reference_snapshot_2400000_major_body_boundary_summary_for_report(),
        reference_snapshot_2451545_major_body_boundary_summary_for_report(),
        reference_snapshot_major_body_boundary_summary_for_report(),
        reference_snapshot_exact_j2000_evidence_summary_for_report(),
        reference_snapshot_2360234_major_body_interior_summary_for_report(),
        reference_snapshot_2451910_major_body_boundary_summary_for_report(),
        reference_snapshot_2451911_major_body_boundary_summary_for_report(),
        reference_snapshot_2451912_major_body_boundary_summary_for_report(),
        reference_snapshot_2451913_major_body_boundary_summary_for_report(),
        reference_snapshot_2451914_major_body_boundary_summary_for_report(),
        reference_snapshot_2451914_major_body_pre_bridge_summary_for_report(),
        reference_snapshot_bridge_day_summary_for_report(),
        reference_snapshot_2451914_bridge_day_summary_for_report(),
        reference_snapshot_2451914_major_body_bridge_day_summary_for_report(),
        reference_snapshot_2451914_major_body_bridge_summary_for_report(),
        reference_snapshot_2451915_major_body_boundary_summary_for_report(),
        reference_snapshot_2451915_major_body_bridge_summary_for_report(),
        reference_snapshot_2451917_major_body_bridge_summary_for_report(),
        reference_snapshot_2451917_major_body_boundary_summary_for_report(),
        reference_snapshot_2451916_major_body_interior_summary_for_report(),
        reference_snapshot_2451916_major_body_dense_boundary_summary_for_report(),
        reference_snapshot_2451916_major_body_boundary_summary_for_report(),
        reference_snapshot_dense_boundary_summary_for_report(),
        reference_snapshot_sparse_boundary_summary_for_report(),
        reference_snapshot_pre_bridge_boundary_summary_for_report(),
        reference_snapshot_boundary_epoch_coverage_summary_for_report(),
        reference_snapshot_major_body_boundary_window_summary_for_report(),
        reference_snapshot_mars_jupiter_boundary_summary_for_report(),
        reference_snapshot_2451918_major_body_boundary_summary_for_report(),
        reference_snapshot_2451919_major_body_boundary_summary_for_report(),
        reference_snapshot_2451920_major_body_interior_summary_for_report(),
        reference_snapshot_2453000_major_body_boundary_summary_for_report(),
        reference_snapshot_2500000_major_body_boundary_summary_for_report(),
        reference_snapshot_2500_major_body_boundary_summary_for_report(),
        selected_asteroid_boundary_summary_for_report(),
        selected_asteroid_bridge_summary_for_report(),
        selected_asteroid_dense_boundary_summary_for_report(),
        selected_asteroid_terminal_boundary_summary_for_report(),
        selected_asteroid_source_evidence_summary_for_report(),
        selected_asteroid_source_window_summary_for_report(),
        selected_asteroid_source_2451917_summary_for_report(),
        selected_asteroid_source_2453000_summary_for_report(),
        selected_asteroid_source_2500000_summary_for_report(),
        selected_asteroid_source_2634167_summary_for_report(),
        reference_asteroid_evidence_summary_for_report(),
        reference_asteroid_equatorial_evidence_summary_for_report(),
        reference_asteroid_source_window_summary_for_report(),
        reference_snapshot_2200_selected_body_boundary_summary_for_report(),
        reference_snapshot_2524593_selected_body_boundary_summary_for_report(),
        reference_snapshot_mars_outer_boundary_summary_for_report(),
        reference_snapshot_2600000_major_body_boundary_summary_for_report(),
        reference_snapshot_2500_selected_body_boundary_summary_for_report(),
        reference_snapshot_2634167_selected_body_boundary_summary_for_report(),
    ];

    let mut report = summary_line;
    for summary in summary_lines {
        report.push('\n');
        report.push_str("  ");
        report.push_str(&summary);
    }
    report
}

pub(crate) fn strip_report_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

/// Computes a deterministic 64-bit checksum for report text.
pub(crate) fn checksum64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub(crate) const INDEPENDENT_HOLDOUT_QUARTER_DAY_EPOCHS: [f64; 2] = [2451915.25, 2451915.75];

pub(crate) const COMPARISON_SNAPSHOT_SOURCE_EXPECTED: &str =
    "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.";

pub(crate) const COMPARISON_SNAPSHOT_COVERAGE_EXPECTED: &str =
    "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.";

pub(crate) const COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED: &str =
    "repository-checked regression fixtures, not a broad public corpus.";

pub(crate) const COMPARISON_SNAPSHOT_REDISTRIBUTION_FALLBACK: &str = "unknown";

pub(crate) const COMPARISON_SNAPSHOT_COLUMNS: &str = "body, x_km, y_km, z_km";

#[cfg(test)]
pub(crate) fn format_validated_source_summary_for_report(
    label: &'static str,
    manifest: &SnapshotManifest,
    render: impl FnOnce() -> String,
) -> String {
    match manifest.validate() {
        Ok(()) => render(),
        Err(error) => format!("{label}: unavailable ({error})"),
    }
}

#[cfg(test)]
pub(crate) fn format_manifest_summary_for_report(
    label: &str,
    manifest: &SnapshotManifest,
) -> String {
    match manifest.validate() {
        Ok(()) => manifest.summary_line(label),
        Err(error) => format!("{label}: unavailable ({error})"),
    }
}

pub(crate) fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    join_display(bodies)
}

pub(crate) fn format_coordinate_frames(frames: &[CoordinateFrame]) -> String {
    join_display(frames)
}

pub(crate) fn format_time_scales(time_scales: &[TimeScale]) -> String {
    join_display(time_scales)
}

pub(crate) fn format_zodiac_modes(zodiac_modes: &[ZodiacMode]) -> String {
    join_display(zodiac_modes)
}

pub(crate) fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    join_display(modes)
}

pub(crate) const ASTEROID_EQUATORIAL_TOLERANCE_DEGREES: f64 = 1e-12;

pub(crate) const ASTEROID_DISTANCE_TOLERANCE_AU: f64 = 1e-12;

pub(crate) const SELECTED_ASTEROID_SOURCE_2453000_EPOCH: f64 = 2_453_000.5;

pub(crate) const SELECTED_ASTEROID_SOURCE_2500000_EPOCH: f64 = 2_500_000.0;

pub(crate) const SELECTED_ASTEROID_SOURCE_2634167_EPOCH: f64 = 2_634_167.0;

pub(crate) const SELECTED_ASTEROID_BRIDGE_EPOCH: f64 = 2_451_915.0;

pub(crate) const SELECTED_ASTEROID_BOUNDARY_EPOCHS: &[f64] =
    &[2_451_914.5, 2_451_915.5, 2_451_918.5, 2_451_919.5];

pub(crate) const SELECTED_ASTEROID_DENSE_BOUNDARY_EPOCH: f64 = 2_451_916.5;

pub(crate) const SELECTED_ASTEROID_TERMINAL_BOUNDARY_EPOCH_JD: f64 = 2_500_000.0;

pub(crate) const REFERENCE_LUNAR_BOUNDARY_EPOCHS: [f64; 2] = [2_451_911.5, 2_451_912.5];

pub(crate) const REFERENCE_HIGH_CURVATURE_EPOCHS: [f64; 5] = [
    2_451_911.5,
    2_451_912.5,
    2_451_913.5,
    2_451_914.5,
    2_451_916.5,
];

pub(crate) const REFERENCE_MAJOR_BODY_BOUNDARY_EPOCH: f64 = 2_451_917.5;

pub(crate) const REFERENCE_MARS_JUPITER_BOUNDARY_EPOCH: f64 = 2_451_918.5;

pub(crate) fn reference_snapshot_lunar_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    entry.body == pleiades_backend::CelestialBody::Moon
                        && REFERENCE_LUNAR_BOUNDARY_EPOCHS.contains(&entry.epoch.julian_day.days())
                })
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

pub(crate) fn reference_snapshot_high_curvature_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && REFERENCE_HIGH_CURVATURE_EPOCHS.contains(&entry.epoch.julian_day.days())
                })
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

pub(crate) fn reference_snapshot_lunar_boundary_summary_details(
) -> Option<ReferenceLunarBoundarySummary> {
    let entries = reference_snapshot_lunar_boundary_entries()?;
    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference lunar boundary evidence should not be empty after collection");
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference lunar boundary evidence should not be empty after collection");

    Some(ReferenceLunarBoundarySummary {
        sample_count: entries.len(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Returns the compact typed summary for the Moon high-curvature reference window.
pub fn reference_snapshot_lunar_boundary_summary() -> Option<ReferenceLunarBoundarySummary> {
    reference_snapshot_lunar_boundary_summary_details()
}

/// Returns the release-facing Moon high-curvature reference window summary string.
pub fn reference_snapshot_lunar_boundary_summary_for_report() -> String {
    match reference_snapshot_lunar_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference lunar boundary evidence: unavailable ({error})"),
        },
        None => "Reference lunar boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_high_curvature_summary_details(
) -> Option<ReferenceHighCurvatureSummary> {
    let entries = reference_snapshot_high_curvature_entries()?;
    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");

    let mut bodies = Vec::new();
    let mut epochs = BTreeSet::new();
    for entry in entries {
        if !bodies.contains(&entry.body) {
            bodies.push(entry.body.clone());
        }
        epochs.insert(entry.epoch.julian_day.days().to_bits());
    }

    Some(ReferenceHighCurvatureSummary {
        sample_count: entries.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Returns the compact typed summary for the major-body high-curvature reference window.
pub fn reference_snapshot_high_curvature_summary() -> Option<ReferenceHighCurvatureSummary> {
    reference_snapshot_high_curvature_summary_details()
}

/// Returns the release-facing major-body high-curvature reference window summary string.
pub fn reference_snapshot_high_curvature_summary_for_report() -> String {
    match reference_snapshot_high_curvature_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body high-curvature evidence: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_major_body_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days() == REFERENCE_MAJOR_BODY_BOUNDARY_EPOCH
                })
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

pub(crate) fn reference_snapshot_major_body_boundary_summary_details(
) -> Option<ReferenceMajorBodyBoundarySummary> {
    let evidence = reference_snapshot_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceMajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the major-body boundary-day reference evidence.
pub fn reference_snapshot_major_body_boundary_summary() -> Option<ReferenceMajorBodyBoundarySummary>
{
    reference_snapshot_major_body_boundary_summary_details()
}

/// Returns the release-facing major-body boundary-day summary string.
pub fn reference_snapshot_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) const REFERENCE_MAJOR_BODY_BRIDGE_EPOCH: f64 = 2_451_915.0;

pub(crate) fn reference_snapshot_major_body_bridge_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days() == REFERENCE_MAJOR_BODY_BRIDGE_EPOCH
                })
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

pub(crate) fn reference_snapshot_major_body_bridge_summary_details(
) -> Option<ReferenceMajorBodyBridgeSummary> {
    let evidence = reference_snapshot_major_body_bridge_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceMajorBodyBridgeSummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the major-body bridge-day reference evidence.
pub fn reference_snapshot_major_body_bridge_summary() -> Option<ReferenceMajorBodyBridgeSummary> {
    reference_snapshot_major_body_bridge_summary_details()
}

/// Returns the release-facing major-body bridge-day summary string.
pub fn reference_snapshot_major_body_bridge_summary_for_report() -> String {
    match reference_snapshot_major_body_bridge_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference major-body bridge evidence: unavailable ({error})"),
        },
        None => "Reference major-body bridge evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_mars_jupiter_boundary_entries() -> Option<&'static [SnapshotEntry]>
{
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    entry.epoch.julian_day.days() == REFERENCE_MARS_JUPITER_BOUNDARY_EPOCH
                })
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

pub(crate) fn reference_snapshot_mars_jupiter_boundary_summary_details(
) -> Option<ReferenceMarsJupiterBoundarySummary> {
    let evidence = reference_snapshot_mars_jupiter_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceMarsJupiterBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the Mars/Jupiter boundary reference evidence.
pub fn reference_snapshot_mars_jupiter_boundary_summary(
) -> Option<ReferenceMarsJupiterBoundarySummary> {
    reference_snapshot_mars_jupiter_boundary_summary_details()
}

/// Returns the release-facing Mars/Jupiter boundary summary string.
pub fn reference_snapshot_mars_jupiter_boundary_summary_for_report() -> String {
    match reference_snapshot_mars_jupiter_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference Mars/Jupiter boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference Mars/Jupiter boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2451918 major-body boundary reference evidence.
///
/// This is a compatibility alias for the Mars/Jupiter boundary slice.
pub fn reference_snapshot_2451918_major_body_boundary_summary(
) -> Option<ReferenceMarsJupiterBoundarySummary> {
    reference_snapshot_mars_jupiter_boundary_summary_details()
}

/// Returns the release-facing 2451918 major-body boundary summary string.
///
/// This is a compatibility alias for the Mars/Jupiter boundary slice with
/// explicit 2451918 wording for release-facing reports.
pub fn reference_snapshot_2451918_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451918_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line.replacen(
                "Reference Mars/Jupiter boundary evidence",
                "Reference 2451918 major-body boundary evidence",
                1,
            ),
            Err(error) => {
                format!("Reference 2451918 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451918 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_1749_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_1749_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_1749_major_body_boundary_summary_details(
) -> Option<Reference1749MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_1749_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1749MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1749 major-body boundary reference evidence.
pub fn reference_snapshot_1749_major_body_boundary_summary(
) -> Option<Reference1749MajorBodyBoundarySummary> {
    reference_snapshot_1749_major_body_boundary_summary_details()
}

/// Returns the release-facing 1749 major-body boundary summary string.
pub fn reference_snapshot_1749_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1749_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1749 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1749 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2360233 major-body boundary reference evidence.
#[doc(alias = "reference_snapshot_1749_major_body_boundary_summary")]
pub fn reference_snapshot_2360233_major_body_boundary_summary(
) -> Option<Reference1749MajorBodyBoundarySummary> {
    reference_snapshot_1749_major_body_boundary_summary()
}

/// Returns the release-facing 2360233 major-body boundary summary string.
pub fn reference_snapshot_2360233_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2360233_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line.replacen(
                "Reference 1749 major-body boundary evidence",
                "Reference 2360233 major-body boundary evidence",
                1,
            ),
            Err(error) => {
                format!("Reference 2360233 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2360233 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_1750_major_body_interior_entries(
) -> Option<&'static [SnapshotEntry]> {
    reference_snapshot_1750_selected_body_boundary_entries()
}

pub(crate) fn reference_snapshot_1750_major_body_interior_summary_details(
) -> Option<Reference1750MajorBodyInteriorSummary> {
    let evidence = reference_snapshot_1750_major_body_interior_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1750MajorBodyInteriorSummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1750 major-body interior reference evidence.
pub fn reference_snapshot_1750_major_body_interior_summary(
) -> Option<Reference1750MajorBodyInteriorSummary> {
    reference_snapshot_1750_major_body_interior_summary_details()
}

/// Returns the release-facing 1750 major-body interior summary string.
pub fn reference_snapshot_1750_major_body_interior_summary_for_report() -> String {
    match reference_snapshot_1750_major_body_interior_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1750 major-body interior evidence: unavailable ({error})")
            }
        },
        None => "Reference 1750 major-body interior evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2360234_major_body_interior_entries(
) -> Option<&'static [SnapshotEntry]> {
    reference_snapshot_1750_selected_body_boundary_entries()
}

pub(crate) fn reference_snapshot_2360234_major_body_interior_summary_details(
) -> Option<Reference2360234MajorBodyInteriorSummary> {
    let evidence = reference_snapshot_2360234_major_body_interior_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2360234MajorBodyInteriorSummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2360234 major-body interior comparison reference evidence.
pub fn reference_snapshot_2360234_major_body_interior_summary(
) -> Option<Reference2360234MajorBodyInteriorSummary> {
    reference_snapshot_2360234_major_body_interior_summary_details()
}

/// Returns the release-facing 2360234 major-body interior comparison summary string.
pub fn reference_snapshot_2360234_major_body_interior_summary_for_report() -> String {
    match reference_snapshot_2360234_major_body_interior_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2360234 major-body interior comparison evidence: unavailable ({error})")
            }
        },
        None => {
            "Reference 2360234 major-body interior comparison evidence: unavailable".to_string()
        }
    }
}

pub(crate) fn reference_snapshot_early_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_REFERENCE_ONLY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_early_major_body_boundary_summary_details(
) -> Option<ReferenceEarlyMajorBodyBoundarySummary> {
    let evidence = reference_snapshot_early_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceEarlyMajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the early major-body boundary reference evidence.
pub fn reference_snapshot_early_major_body_boundary_summary(
) -> Option<ReferenceEarlyMajorBodyBoundarySummary> {
    reference_snapshot_early_major_body_boundary_summary_details()
}

/// Returns the release-facing early major-body boundary summary string.
pub fn reference_snapshot_early_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_early_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference early major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference early major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2378498 major-body boundary reference evidence.
#[doc(alias = "reference_snapshot_early_major_body_boundary_summary")]
pub fn reference_snapshot_2378498_major_body_boundary_summary(
) -> Option<ReferenceEarlyMajorBodyBoundarySummary> {
    reference_snapshot_early_major_body_boundary_summary()
}

/// Returns the release-facing 2378498 major-body boundary summary string.
pub fn reference_snapshot_2378498_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2378498_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line.replacen(
                "Reference early major-body boundary evidence",
                "Reference 2378498 major-body boundary evidence",
                1,
            ),
            Err(error) => {
                format!("Reference 2378498 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2378498 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_1800_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_1800_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_1800_major_body_boundary_summary_details(
) -> Option<Reference1800MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_1800_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1800MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1800 major-body boundary reference evidence.
pub fn reference_snapshot_1800_major_body_boundary_summary(
) -> Option<Reference1800MajorBodyBoundarySummary> {
    reference_snapshot_1800_major_body_boundary_summary_details()
}

/// Returns the release-facing 1800 major-body boundary summary string.
pub fn reference_snapshot_1800_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1800_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1800 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1800 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2378499 major-body boundary reference evidence.
#[doc(alias = "reference_snapshot_1800_major_body_boundary_summary")]
pub fn reference_snapshot_2378499_major_body_boundary_summary(
) -> Option<Reference1800MajorBodyBoundarySummary> {
    reference_snapshot_1800_major_body_boundary_summary()
}

/// Returns the release-facing 2378499 major-body boundary summary string.
pub fn reference_snapshot_2378499_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2378499_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line.replacen(
                "Reference 1800 major-body boundary evidence",
                "Reference 2378499 major-body boundary evidence",
                1,
            ),
            Err(error) => {
                format!("Reference 2378499 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2378499 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2500_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2500_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2500_major_body_boundary_summary_details(
) -> Option<Reference2500MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2500_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2500MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2500 major-body boundary reference evidence.
pub fn reference_snapshot_2500_major_body_boundary_summary(
) -> Option<Reference2500MajorBodyBoundarySummary> {
    reference_snapshot_2500_major_body_boundary_summary_details()
}

/// Returns the release-facing 2500 major-body boundary summary string.
pub fn reference_snapshot_2500_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2500_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2500 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2500 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2453000_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2453000_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2453000_major_body_boundary_summary_details(
) -> Option<Reference2453000MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2453000_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2453000MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2453000 major-body boundary reference evidence.
pub fn reference_snapshot_2453000_major_body_boundary_summary(
) -> Option<Reference2453000MajorBodyBoundarySummary> {
    reference_snapshot_2453000_major_body_boundary_summary_details()
}

/// Returns the release-facing 2453000 major-body boundary summary string.
pub fn reference_snapshot_2453000_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2453000_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2453000 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2453000 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2500000_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2500000_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2500000_major_body_boundary_summary_details(
) -> Option<Reference2500000MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2500000_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2500000MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2500000 major-body boundary reference evidence.
pub fn reference_snapshot_2500000_major_body_boundary_summary(
) -> Option<Reference2500000MajorBodyBoundarySummary> {
    reference_snapshot_2500000_major_body_boundary_summary_details()
}

/// Returns the release-facing 2500000 major-body boundary summary string.
pub fn reference_snapshot_2500000_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2500000_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2500000 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2500000 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2400000_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2400000_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2400000_major_body_boundary_summary_details(
) -> Option<Reference2400000MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2400000_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2400000MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2400000 major-body boundary reference evidence.
pub fn reference_snapshot_2400000_major_body_boundary_summary(
) -> Option<Reference2400000MajorBodyBoundarySummary> {
    reference_snapshot_2400000_major_body_boundary_summary_details()
}

/// Returns the release-facing 2400000 major-body boundary summary string.
pub fn reference_snapshot_2400000_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2400000_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2400000 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2400000 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451545_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451545_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451545_major_body_boundary_summary_details(
) -> Option<Reference2451545MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451545_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451545MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451545 major-body boundary reference evidence.
pub fn reference_snapshot_2451545_major_body_boundary_summary(
) -> Option<Reference2451545MajorBodyBoundarySummary> {
    reference_snapshot_2451545_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451545 major-body boundary summary string.
pub fn reference_snapshot_2451545_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451545_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451545 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451545 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451910_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451910_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451910_major_body_boundary_summary_details(
) -> Option<Reference2451910MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451910_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451910MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451910 major-body boundary reference evidence.
pub fn reference_snapshot_2451910_major_body_boundary_summary(
) -> Option<Reference2451910MajorBodyBoundarySummary> {
    reference_snapshot_2451910_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451910 major-body boundary summary string.
pub fn reference_snapshot_2451910_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451910_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451910 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451910 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451911_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451911_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451911_major_body_boundary_summary_details(
) -> Option<Reference2451911MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451911_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451911MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451911 major-body boundary reference evidence.
pub fn reference_snapshot_2451911_major_body_boundary_summary(
) -> Option<Reference2451911MajorBodyBoundarySummary> {
    reference_snapshot_2451911_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451911 major-body boundary summary string.
pub fn reference_snapshot_2451911_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451911_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451911 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451911 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451912_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451912_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451912_major_body_boundary_summary_details(
) -> Option<Reference2451912MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451912_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451912MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451912 major-body boundary reference evidence.
pub fn reference_snapshot_2451912_major_body_boundary_summary(
) -> Option<Reference2451912MajorBodyBoundarySummary> {
    reference_snapshot_2451912_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451912 major-body boundary summary string.
pub fn reference_snapshot_2451912_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451912_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451912 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451912 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451913_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451913_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451913_major_body_boundary_summary_details(
) -> Option<Reference2451913MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451913_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451913MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451913 major-body boundary reference evidence.
pub fn reference_snapshot_2451913_major_body_boundary_summary(
) -> Option<Reference2451913MajorBodyBoundarySummary> {
    reference_snapshot_2451913_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451913 major-body boundary summary string.
pub fn reference_snapshot_2451913_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451913_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451913 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451913 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451914_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451914_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451914_major_body_boundary_summary_details(
) -> Option<Reference2451914MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451914_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451914MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451914 major-body boundary reference evidence.
pub fn reference_snapshot_2451914_major_body_boundary_summary(
) -> Option<Reference2451914MajorBodyBoundarySummary> {
    reference_snapshot_2451914_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451914 major-body boundary summary string.
pub fn reference_snapshot_2451914_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451914_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451914 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451914 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451915_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451915_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451915_major_body_boundary_summary_details(
) -> Option<Reference2451915MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451915_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451915MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451915 major-body boundary reference evidence.
pub fn reference_snapshot_2451915_major_body_boundary_summary(
) -> Option<Reference2451915MajorBodyBoundarySummary> {
    reference_snapshot_2451915_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451915 major-body boundary summary string.
pub fn reference_snapshot_2451915_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451915_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451915 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451915 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451917_major_body_bridge_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451917_MAJOR_BODY_BRIDGE_EPOCH_JD
                })
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

pub(crate) fn reference_snapshot_2451917_major_body_bridge_summary_details(
) -> Option<Reference2451917MajorBodyBridgeSummary> {
    let evidence = reference_snapshot_2451917_major_body_bridge_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451917MajorBodyBridgeSummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451917 major-body bridge reference evidence.
pub fn reference_snapshot_2451917_major_body_bridge_summary(
) -> Option<Reference2451917MajorBodyBridgeSummary> {
    reference_snapshot_2451917_major_body_bridge_summary_details()
}

/// Returns the release-facing 2451917 major-body bridge summary string.
pub fn reference_snapshot_2451917_major_body_bridge_summary_for_report() -> String {
    match reference_snapshot_2451917_major_body_bridge_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451917 major-body bridge evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451917 major-body bridge evidence: unavailable".to_string(),
    }
}
