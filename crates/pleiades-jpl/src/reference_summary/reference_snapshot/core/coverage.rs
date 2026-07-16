//! core coverage summaries.

use core::fmt;

use pleiades_types::Instant;

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

/// A compact body-class coverage summary for the checked-in reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotBodyClassCoverageSummary {
    /// Number of major-body rows in the checked-in reference snapshot.
    pub major_body_row_count: usize,
    /// Major bodies covered by the checked-in reference snapshot in first-seen order.
    pub major_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the major-body subset.
    pub major_epoch_count: usize,
    /// Per-body windows covered by the major-body subset in first-seen order.
    pub major_windows: Vec<ReferenceSnapshotSourceWindow>,
    /// Number of selected-asteroid rows in the checked-in reference snapshot.
    pub asteroid_row_count: usize,
    /// Selected asteroids covered by the checked-in reference snapshot in first-seen order.
    pub asteroid_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the selected-asteroid subset.
    pub asteroid_epoch_count: usize,
    /// Per-body windows covered by the selected-asteroid subset in first-seen order.
    pub asteroid_windows: Vec<ReferenceSnapshotSourceWindow>,
}

/// Validation error for a reference snapshot body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceSnapshotBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in body-class coverage.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ReferenceSnapshotBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference snapshot body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotBodyClassCoverageSummaryValidationError {}

impl ReferenceSnapshotBodyClassCoverageSummary {
    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotBodyClassCoverageSummaryValidationError> {
        let Some(expected) = reference_snapshot_body_class_coverage_summary_details() else {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_body_row_count",
                },
            );
        };

        if self.major_body_row_count != expected.major_body_row_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_body_row_count",
                },
            );
        }
        if self.major_bodies != expected.major_bodies {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_bodies",
                },
            );
        }
        if self.major_epoch_count != expected.major_epoch_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_epoch_count",
                },
            );
        }
        if self.major_windows != expected.major_windows {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_windows",
                },
            );
        }
        if self.asteroid_row_count != expected.asteroid_row_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_row_count",
                },
            );
        }
        if self.asteroid_bodies != expected.asteroid_bodies {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_bodies",
                },
            );
        }
        if self.asteroid_epoch_count != expected.asteroid_epoch_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_epoch_count",
                },
            );
        }
        if self.asteroid_windows != expected.asteroid_windows {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_windows",
                },
            );
        }

        Ok(())
    }
}

pub(crate) fn reference_snapshot_body_class_coverage_summary_details(
) -> Option<ReferenceSnapshotBodyClassCoverageSummary> {
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut major_body_row_count = 0usize;
    let mut asteroid_row_count = 0usize;
    let mut major_epochs = BTreeSet::new();
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

    let source_windows = reference_snapshot_source_window_summary_details()?;
    let major_windows = source_windows
        .windows
        .iter()
        .filter(|window| is_comparison_body(&window.body))
        .cloned()
        .collect::<Vec<_>>();
    let asteroid_windows = source_windows
        .windows
        .iter()
        .filter(|window| is_reference_asteroid(&window.body))
        .cloned()
        .collect::<Vec<_>>();

    Some(ReferenceSnapshotBodyClassCoverageSummary {
        major_body_row_count,
        major_bodies: snapshot_bodies()
            .iter()
            .filter(|body| is_comparison_body(body))
            .cloned()
            .collect(),
        major_epoch_count: major_epochs.len(),
        major_windows,
        asteroid_row_count,
        asteroid_bodies: reference_asteroids().to_vec(),
        asteroid_epoch_count: asteroid_epochs.len(),
        asteroid_windows,
    })
}

/// Returns a compact body-class coverage summary for the checked-in reference snapshot.
pub fn reference_snapshot_body_class_coverage_summary(
) -> Option<ReferenceSnapshotBodyClassCoverageSummary> {
    reference_snapshot_body_class_coverage_summary_details()
}

/// A single epoch slice inside the reference snapshot boundary coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotBoundaryEpochCoverage {
    /// The epoch covered by this slice.
    pub epoch: Instant,
    /// Number of bodies covered at the epoch.
    pub body_count: usize,
    /// Bodies covered by the epoch slice in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
}

impl ReferenceSnapshotBoundaryEpochCoverage {}

/// Compact release-facing summary for the reference snapshot boundary-window epoch coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotBoundaryEpochCoverageSummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Number of distinct epochs covered by the boundary slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the boundary slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the boundary slice.
    pub latest_epoch: Instant,
    /// Per-epoch body breakdown in first-seen order.
    pub windows: Vec<ReferenceSnapshotBoundaryEpochCoverage>,
}

/// Validation error for a boundary-window epoch summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in epoch coverage.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference snapshot boundary epoch coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError {}

impl ReferenceSnapshotBoundaryEpochCoverageSummary {
    /// Returns `Ok(())` when the epoch coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError> {
        let Some(expected) = reference_snapshot_boundary_epoch_coverage_summary_details() else {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }
}

pub(crate) fn reference_snapshot_boundary_epoch_coverage_summary_details(
) -> Option<ReferenceSnapshotBoundaryEpochCoverageSummary> {
    let entries = reference_snapshot()
        .iter()
        .filter(|entry| (2_451_912.5..=2_451_919.5).contains(&entry.epoch.julian_day.days()))
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return None;
    }

    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .expect("reference snapshot boundary epoch coverage should not be empty after collection")
        .epoch;
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .expect("reference snapshot boundary epoch coverage should not be empty after collection")
        .epoch;

    let mut windows = BTreeMap::<u64, ReferenceSnapshotBoundaryEpochCoverage>::new();
    for entry in &entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        let window =
            windows
                .entry(epoch_bits)
                .or_insert_with(|| ReferenceSnapshotBoundaryEpochCoverage {
                    epoch: entry.epoch,
                    body_count: 0,
                    bodies: Vec::new(),
                });
        if !window.bodies.contains(&entry.body) {
            window.bodies.push(entry.body.clone());
            window.body_count = window.bodies.len();
        }
    }

    Some(ReferenceSnapshotBoundaryEpochCoverageSummary {
        sample_count: entries.len(),
        epoch_count: windows.len(),
        earliest_epoch,
        latest_epoch,
        windows: windows.into_values().collect(),
    })
}

/// Returns the compact typed summary for the reference snapshot boundary-window epoch coverage.
pub fn reference_snapshot_boundary_epoch_coverage_summary(
) -> Option<ReferenceSnapshotBoundaryEpochCoverageSummary> {
    reference_snapshot_boundary_epoch_coverage_summary_details()
}

pub(crate) fn reference_snapshot_high_curvature_epoch_coverage_summary_details(
) -> Option<ReferenceHighCurvatureEpochCoverageSummary> {
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

    let mut windows = BTreeMap::<u64, ReferenceHighCurvatureEpochCoverage>::new();
    for entry in entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        let window =
            windows
                .entry(epoch_bits)
                .or_insert_with(|| ReferenceHighCurvatureEpochCoverage {
                    epoch: entry.epoch,
                    body_count: 0,
                    bodies: Vec::new(),
                });
        if !window.bodies.contains(&entry.body) {
            window.bodies.push(entry.body.clone());
            window.body_count = window.bodies.len();
        }
    }

    Some(ReferenceHighCurvatureEpochCoverageSummary {
        sample_count: entries.len(),
        epoch_count: windows.len(),
        earliest_epoch,
        latest_epoch,
        windows: windows.into_values().collect(),
    })
}

/// Returns the compact typed summary for the major-body high-curvature epoch coverage.
pub fn reference_snapshot_high_curvature_epoch_coverage_summary(
) -> Option<ReferenceHighCurvatureEpochCoverageSummary> {
    reference_snapshot_high_curvature_epoch_coverage_summary_details()
}

impl ReferenceSnapshotBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let major_windows = self
            .major_windows
            .iter()
            .map(ReferenceSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let asteroid_windows = self
            .asteroid_windows
            .iter()
            .map(ReferenceSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Reference snapshot body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
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
}
