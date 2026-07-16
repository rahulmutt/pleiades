//! Relocated reference-snapshot core coverage renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::core::coverage`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()` constructors, `validate()`/`label()` methods, and all
//! release-gate data; jpl's own rendering stays in place until the Task 14
//! contract sweep.
//!
//! The major-body high-curvature epoch coverage renderer
//! (`reference_snapshot_high_curvature_epoch_coverage_summary_for_report`)
//! renders a `ReferenceHighCurvatureEpochCoverageSummary`, whose struct and
//! inherent rendering live in
//! `reference_summary/reference_snapshot/boundaries/era_d.rs` (Slice D Task
//! 9, already copied); its `validated_summary_line()` call is rewired to
//! `match summary.validate() { Ok(()) => <local render>, ... }`, calling the
//! local `reference_high_curvature_epoch_coverage_summary_line` (Slice D Task
//! 14a4). Likewise, the body-class coverage renderer's nested
//! `ReferenceSnapshotSourceWindow::summary_line` call is left pointing at
//! jpl (that struct's rendering moves in Task 8c's `general_b.rs` slice).

use pleiades_jpl::{
    ReferenceSnapshotBodyClassCoverageSummary, ReferenceSnapshotBoundaryEpochCoverage,
    ReferenceSnapshotBoundaryEpochCoverageSummary, ReferenceSnapshotSourceWindow,
};

#[allow(unused_imports)]
use crate::posture::jpl::*;

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling
/// `comparison.rs`, `holdout.rs`, `jpl_posture.rs`, and `reference_asteroid.rs`
/// posture modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Reproduced from jpl's `pub(crate)` `format_bodies`
/// (`reference_summary/reference_snapshot/core/general_a.rs:510`), which is
/// not callable cross-crate. Per-module duplicate accepted (Slice D expand)
/// — already reproduced identically in the sibling posture modules.
fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Compact release-facing body-class summary line. Verbatim copy of
/// `ReferenceSnapshotBodyClassCoverageSummary::summary_line`
/// (reference_summary/reference_snapshot/core/coverage.rs:58). The nested
/// `ReferenceSnapshotSourceWindow::summary_line` calls are left as jpl method
/// calls — that struct's rendering has not moved to validate yet (Task 8c's
/// `general_b.rs` slice); validate→jpl is allowed and byte-identical here.
pub(crate) fn reference_snapshot_body_class_coverage_summary_line(
    s: &ReferenceSnapshotBodyClassCoverageSummary,
) -> String {
    let major_windows = s
        .major_windows
        .iter()
        .map(ReferenceSnapshotSourceWindow::summary_line)
        .collect::<Vec<_>>()
        .join("; ");
    let asteroid_windows = s
        .asteroid_windows
        .iter()
        .map(ReferenceSnapshotSourceWindow::summary_line)
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Reference snapshot body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
        s.major_body_row_count,
        s.major_bodies.len(),
        s.major_epoch_count,
        major_windows,
        s.asteroid_row_count,
        s.asteroid_bodies.len(),
        s.asteroid_epoch_count,
        asteroid_windows,
    )
}

/// Returns the release-facing body-class coverage summary string for the
/// checked-in reference snapshot. Verbatim copy of jpl's
/// `reference_snapshot_body_class_coverage_summary_for_report`
/// (reference_summary/reference_snapshot/core/coverage.rs:231), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local).
pub(crate) fn reference_snapshot_body_class_coverage_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_body_class_coverage_summary_line(&summary),
            Err(error) => format!("Reference snapshot body-class coverage: unavailable ({error})"),
        },
        None => "Reference snapshot body-class coverage: unavailable".to_string(),
    }
}

/// Compact release-facing epoch-coverage summary line for a single epoch
/// slice. Verbatim copy of
/// `ReferenceSnapshotBoundaryEpochCoverage::summary_line`
/// (reference_summary/reference_snapshot/core/coverage.rs:254). Uses the
/// promoted `REFERENCE_PRE_BRIDGE_BOUNDARY_EPOCH_JD`/
/// `REFERENCE_SPARSE_BOUNDARY_EPOCH_JD` constants and the promoted
/// `reference_snapshot_sparse_boundary_missing_bodies` data helper
/// (`reference_summary/reference_snapshot/core/general_b.rs`, Slice D Task 8a
/// promotion).
pub(crate) fn reference_snapshot_boundary_epoch_coverage_line(
    s: &ReferenceSnapshotBoundaryEpochCoverage,
) -> String {
    let pre_bridge_note =
        if s.epoch.julian_day.days() == pleiades_jpl::REFERENCE_PRE_BRIDGE_BOUNDARY_EPOCH_JD {
            "; pre-bridge boundary day"
        } else {
            ""
        };
    let sparse_note =
        if s.epoch.julian_day.days() == pleiades_jpl::REFERENCE_SPARSE_BOUNDARY_EPOCH_JD {
            let missing_bodies =
                pleiades_jpl::reference_snapshot_sparse_boundary_missing_bodies(&s.bodies);
            if missing_bodies.is_empty() {
                String::new()
            } else {
                format!(
                    "; sparse boundary day; missing bodies: {}",
                    format_bodies(&missing_bodies)
                )
            }
        } else {
            String::new()
        };

    format!(
        "{}: {} bodies ({}{}){}",
        format_instant(s.epoch),
        s.body_count,
        format_bodies(&s.bodies),
        pre_bridge_note,
        sparse_note,
    )
}

/// Compact release-facing summary line for the reference snapshot
/// boundary-window epoch coverage. Verbatim copy of
/// `ReferenceSnapshotBoundaryEpochCoverageSummary::summary_line`
/// (reference_summary/reference_snapshot/core/coverage.rs:332). The nested
/// `ReferenceSnapshotBoundaryEpochCoverage::summary_line(w)` call is rewired
/// to the local `reference_snapshot_boundary_epoch_coverage_line` free fn
/// (same-file struct, already re-homed above).
pub(crate) fn reference_snapshot_boundary_epoch_coverage_summary_line(
    s: &ReferenceSnapshotBoundaryEpochCoverageSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(reference_snapshot_boundary_epoch_coverage_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Reference snapshot boundary epoch coverage: {} exact samples across {} epochs ({}..{}); epochs: {}",
        s.sample_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Returns the release-facing reference snapshot boundary-window epoch
/// coverage summary string. Verbatim copy of jpl's
/// `reference_snapshot_boundary_epoch_coverage_summary_for_report`
/// (reference_summary/reference_snapshot/core/coverage.rs:479), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }`.
pub(crate) fn reference_snapshot_boundary_epoch_coverage_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_boundary_epoch_coverage_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_boundary_epoch_coverage_summary_line(&summary),
            Err(error) => {
                format!("Reference snapshot boundary epoch coverage: unavailable ({error})")
            }
        },
        None => "Reference snapshot boundary epoch coverage: unavailable".to_string(),
    }
}

/// Returns the release-facing major-body high-curvature epoch coverage
/// summary string. Verbatim copy of jpl's
/// `reference_snapshot_high_curvature_epoch_coverage_summary_for_report`
/// (reference_summary/reference_snapshot/core/coverage.rs:548). The
/// `ReferenceHighCurvatureEpochCoverageSummary` struct and its rendering live
/// in `reference_summary/reference_snapshot/boundaries/era_d.rs` (Slice D
/// Task 9, already copied); `summary.validated_summary_line()` is rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }`, calling the
/// local `reference_high_curvature_epoch_coverage_summary_line` (Slice D Task
/// 14a4; `validate()` stays on the jpl struct, rendering is local).
pub(crate) fn reference_snapshot_high_curvature_epoch_coverage_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_high_curvature_epoch_coverage_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_high_curvature_epoch_coverage_summary_line(&summary),
            Err(error) => {
                format!("Reference major-body high-curvature epoch coverage: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature epoch coverage: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_snapshot_body_class_coverage_summary_reports_the_expected_body_classes() {
        let summary = pleiades_jpl::reference_snapshot_body_class_coverage_summary()
            .expect("reference snapshot body-class coverage summary should exist");

        assert_eq!(summary.major_body_row_count, 182);
        assert_eq!(summary.major_bodies.len(), 10);
        assert_eq!(
            summary.major_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.major_bodies[9],
            pleiades_backend::CelestialBody::Uranus
        );
        assert_eq!(summary.major_epoch_count, 20);
        assert_eq!(summary.asteroid_row_count, 95);
        assert_eq!(summary.asteroid_bodies.len(), 6);
        assert_eq!(
            summary.asteroid_bodies[0],
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.asteroid_bodies[4],
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid", "433-Eros"
            ))
        );
        assert_eq!(
            summary.asteroid_bodies[5],
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid",
                "99942-Apophis"
            ))
        );
        assert_eq!(summary.asteroid_epoch_count, 17);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_snapshot_body_class_coverage_summary_for_report(),
            reference_snapshot_body_class_coverage_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_body_class_coverage_summary_validation_rejects_row_count_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_body_class_coverage_summary()
            .expect("reference snapshot body-class coverage summary should exist");
        summary.major_body_row_count += 1;

        assert_eq!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_body_row_count",
                }
            )
        );
    }

    #[test]
    fn reference_snapshot_boundary_epoch_coverage_summary_reports_the_sparse_epochs() {
        let summary = pleiades_jpl::reference_snapshot_boundary_epoch_coverage_summary()
            .expect("reference snapshot boundary epoch coverage summary should exist");
        assert_eq!(summary.sample_count, 183);
        assert_eq!(summary.epoch_count, 14);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_912.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_919.5);
        assert_eq!(summary.windows.len(), 14);
        assert_eq!(summary.windows[0].body_count, 15);
        assert_eq!(summary.windows[3].body_count, 15);
        assert_eq!(
            summary.windows[3].bodies[0],
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.windows[3].bodies[14],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert!(
            reference_snapshot_boundary_epoch_coverage_summary_line(&summary)
                .contains("Reference snapshot boundary epoch coverage: 183 exact samples across 14 epochs (JD 2451912.5 (TDB)..JD 2451919.5 (TDB)); epochs:")
        );
        assert!(reference_snapshot_boundary_epoch_coverage_summary_line(&summary).contains(
            "JD 2451914.0 (TDB): 15 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        ));
        assert!(reference_snapshot_boundary_epoch_coverage_summary_line(&summary)
            .contains("JD 2451915.5 (TDB): 16 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"));
        assert!(reference_snapshot_boundary_epoch_coverage_summary_line(&summary)
            .contains("JD 2451916.0 (TDB): 10 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto)"));
        assert!(reference_snapshot_boundary_epoch_coverage_summary_line(&summary).contains(
            "JD 2451919.5 (TDB): 16 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        ));

        assert_eq!(
            reference_snapshot_boundary_epoch_coverage_summary_for_report(),
            reference_snapshot_boundary_epoch_coverage_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_boundary_epoch_coverage_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_boundary_epoch_coverage_summary()
            .expect("reference snapshot boundary epoch coverage summary should exist");
        summary.windows[2].body_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted boundary epoch coverage summary should fail validation");

        assert!(matches!(
            error,
            pleiades_jpl::ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                field: "windows"
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_boundary_epoch_coverage_summary_for_report(),
            reference_snapshot_boundary_epoch_coverage_summary_line(
                &pleiades_jpl::reference_snapshot_boundary_epoch_coverage_summary()
                    .expect("reference snapshot boundary epoch coverage summary should exist")
            )
        );
    }
}
