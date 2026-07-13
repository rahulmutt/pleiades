//! Relocated reference-snapshot core parity renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::core::parity`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()`/`*_summary()` constructors, `validate()`/`label()` methods,
//! and all release-gate data; jpl's own rendering stays in place until the
//! Task 14 contract sweep.
//!
//! `ReferenceSnapshotBatchParitySummary` and
//! `ReferenceSnapshotMixedTimeScaleBatchParitySummary` both nest a
//! `pleiades_jpl::ReferenceSnapshotSummary` (`snapshot` field), whose struct
//! and rendering live in `general_a.rs` (Slice D Task 8b, not yet copied);
//! only its public fields are read here, so no cross-crate call is needed
//! for that nested type.

use pleiades_jpl::{
    ReferenceSnapshotBatchParitySummary, ReferenceSnapshotEquatorialParitySummary,
    ReferenceSnapshotMixedTimeScaleBatchParitySummary,
};

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
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

/// Compact release-facing equatorial parity summary line. Verbatim copy of
/// `ReferenceSnapshotEquatorialParitySummary::summary_line`
/// (reference_summary/reference_snapshot/core/parity.rs:48).
pub(crate) fn reference_snapshot_equatorial_parity_summary_line(
    s: &ReferenceSnapshotEquatorialParitySummary,
) -> String {
    format!(
        "JPL reference snapshot equatorial parity: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; mean-obliquity transform against the checked-in ecliptic fixture",
        s.row_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(s.bodies),
    )
}

/// Returns the release-facing reference snapshot equatorial parity summary
/// string. Verbatim copy of jpl's
/// `reference_snapshot_equatorial_parity_summary_for_report`
/// (reference_summary/reference_snapshot/core/parity.rs:128), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local).
pub(crate) fn reference_snapshot_equatorial_parity_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_equatorial_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_equatorial_parity_summary_line(&summary),
            Err(error) => {
                format!("JPL reference snapshot equatorial parity: unavailable ({error})")
            }
        },
        None => "JPL reference snapshot equatorial parity: unavailable".to_string(),
    }
}

/// Compact release-facing mixed-frame batch parity summary line. Verbatim
/// copy of `ReferenceSnapshotBatchParitySummary::summary_line`
/// (reference_summary/reference_snapshot/core/parity.rs:340). Reads only the
/// nested `snapshot` (`pleiades_jpl::ReferenceSnapshotSummary`) public
/// fields; that struct's own rendering is not re-homed here (Task 8b).
pub(crate) fn reference_snapshot_batch_parity_summary_line(
    s: &ReferenceSnapshotBatchParitySummary,
) -> String {
    format!(
        "JPL reference snapshot batch parity: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; frame mix: {} ecliptic, {} equatorial; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
        s.snapshot.row_count,
        s.snapshot.body_count,
        s.snapshot.epoch_count,
        format_instant(s.snapshot.earliest_epoch),
        format_instant(s.snapshot.latest_epoch),
        format_bodies(s.snapshot.bodies),
        s.ecliptic_request_count,
        s.equatorial_request_count,
        s.exact_count,
        s.interpolated_count,
        s.approximate_count,
        s.unknown_count,
    )
}

/// Returns the release-facing reference snapshot batch parity summary
/// string. Verbatim copy of jpl's
/// `reference_snapshot_batch_parity_summary_for_report`
/// (reference_summary/reference_snapshot/core/parity.rs:381), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }`.
pub(crate) fn reference_snapshot_batch_parity_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_batch_parity_summary_line(&summary),
            Err(error) => format!("JPL reference snapshot batch parity: unavailable ({error})"),
        },
        None => "JPL reference snapshot batch parity: unavailable".to_string(),
    }
}

/// Returns the validated release-facing reference snapshot batch parity
/// summary string. Verbatim copy of jpl's
/// `validated_reference_snapshot_batch_parity_summary_for_report`
/// (reference_summary/reference_snapshot/core/parity.rs:392), with
/// `summary.validated_summary_line()` rewired to
/// `{ summary.validate().map_err(|e| e.to_string())?; Ok(<local render>) }`.
pub(crate) fn validated_reference_snapshot_batch_parity_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::reference_snapshot_batch_parity_summary()
        .ok_or_else(|| "JPL reference snapshot batch parity: unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(reference_snapshot_batch_parity_summary_line(&summary))
}

/// Compact release-facing mixed TT/TDB batch parity summary line. Verbatim
/// copy of `ReferenceSnapshotMixedTimeScaleBatchParitySummary::summary_line`
/// (reference_summary/reference_snapshot/core/parity.rs:566).
pub(crate) fn reference_snapshot_mixed_time_scale_batch_parity_summary_line(
    s: &ReferenceSnapshotMixedTimeScaleBatchParitySummary,
) -> String {
    let order = if s.order_preserved {
        "preserved"
    } else {
        "needs attention"
    };
    let parity = if s.single_query_parity_preserved {
        "preserved"
    } else {
        "needs attention"
    };
    format!(
        "JPL reference snapshot mixed TT/TDB batch parity: {} requests across {} bodies, TT requests={}, TDB requests={}; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; order={}, single-query parity={}",
        s.request_count,
        s.body_count,
        s.tt_request_count,
        s.tdb_request_count,
        s.exact_count,
        s.interpolated_count,
        s.approximate_count,
        s.unknown_count,
        order,
        parity,
    )
}

/// Returns the release-facing mixed TT/TDB reference snapshot batch parity
/// summary string. Verbatim copy of jpl's
/// `reference_snapshot_mixed_time_scale_batch_parity_summary_for_report`
/// (reference_summary/reference_snapshot/core/parity.rs:675), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }`.
pub(crate) fn reference_snapshot_mixed_time_scale_batch_parity_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_mixed_time_scale_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_mixed_time_scale_batch_parity_summary_line(&summary),
            Err(error) => {
                format!("JPL reference snapshot mixed TT/TDB batch parity: unavailable ({error})")
            }
        },
        None => "JPL reference snapshot mixed TT/TDB batch parity: unavailable".to_string(),
    }
}

/// Returns the validated release-facing mixed TT/TDB reference snapshot
/// batch parity summary string. Verbatim copy of jpl's
/// `validated_reference_snapshot_mixed_time_scale_batch_parity_summary_for_report`
/// (reference_summary/reference_snapshot/core/parity.rs:689), with
/// `summary.validated_summary_line()` rewired to
/// `{ summary.validate().map_err(|e| e.to_string())?; Ok(<local render>) }`.
pub(crate) fn validated_reference_snapshot_mixed_time_scale_batch_parity_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::reference_snapshot_mixed_time_scale_batch_parity_summary()
        .ok_or_else(|| {
            "JPL reference snapshot mixed TT/TDB batch parity: unavailable".to_string()
        })?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(reference_snapshot_mixed_time_scale_batch_parity_summary_line(&summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::reference_snapshot_equatorial_parity_summary()
            .expect("reference snapshot equatorial parity summary should exist");
        assert_eq!(summary.row_count, 277);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.bodies, pleiades_jpl::reference_bodies());
        assert_eq!(summary.epoch_count, 23);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "JPL reference snapshot equatorial parity: 277 rows across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; mean-obliquity transform against the checked-in ecliptic fixture",
                format_bodies(pleiades_jpl::reference_bodies())
            )
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_snapshot_equatorial_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_equatorial_parity_summary_validation_rejects_body_count_drift() {
        let summary = pleiades_jpl::ReferenceSnapshotEquatorialParitySummary {
            row_count: 2,
            body_count: 3,
            bodies: pleiades_jpl::reference_bodies(),
            epoch_count: 1,
            earliest_epoch: pleiades_jpl::reference_instant(),
            latest_epoch: pleiades_jpl::reference_instant(),
        };

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceSnapshotEquatorialParitySummaryValidationError::Snapshot(
                    pleiades_jpl::ReferenceSnapshotSummaryValidationError::BodyCountMismatch {
                        body_count: 3,
                        bodies_len: 16,
                    }
                )
            )
        ));
    }

    #[test]
    fn reference_snapshot_batch_parity_summary_validation_rejects_derived_summary_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_batch_parity_summary()
            .expect("reference snapshot batch parity summary should exist");
        summary.snapshot.asteroid_row_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceSnapshotBatchParitySummaryValidationError::Snapshot(
                    pleiades_jpl::ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                        asteroid_row_count: 96,
                        derived_asteroid_row_count: 95,
                    }
                )
            )
        ));
    }

    #[test]
    fn reference_snapshot_batch_parity_summary_validation_rejects_request_count_mismatches() {
        let mut summary = pleiades_jpl::reference_snapshot_batch_parity_summary()
            .expect("reference snapshot batch parity summary should exist");
        summary.equatorial_request_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(pleiades_jpl::ReferenceSnapshotBatchParitySummaryValidationError::RequestCountMismatch { .. })
        ));
    }

    #[test]
    fn reference_snapshot_mixed_time_scale_batch_parity_summary_reports_the_mixed_request_slice() {
        let summary = pleiades_jpl::reference_snapshot_mixed_time_scale_batch_parity_summary()
            .expect("reference snapshot mixed TT/TDB batch parity summary should exist");

        assert_eq!(summary.request_count, summary.snapshot.row_count);
        assert_eq!(summary.body_count, summary.snapshot.body_count);
        assert!(summary.tt_request_count > 0);
        assert!(summary.tdb_request_count > 0);
        assert_eq!(
            summary.tt_request_count + summary.tdb_request_count,
            summary.request_count
        );
        assert_eq!(
            summary.exact_count
                + summary.interpolated_count
                + summary.approximate_count
                + summary.unknown_count,
            summary.request_count
        );
        assert!(summary.order_preserved);
        assert!(summary.single_query_parity_preserved);
        assert!(summary.validate().is_ok());
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            validated_reference_snapshot_mixed_time_scale_batch_parity_summary_for_report(),
            Ok(summary.summary_line())
        );
        assert_eq!(
            reference_snapshot_mixed_time_scale_batch_parity_summary_for_report(),
            summary.summary_line()
        );
        assert!(
            reference_snapshot_mixed_time_scale_batch_parity_summary_for_report()
                .starts_with("JPL reference snapshot mixed TT/TDB batch parity: ")
        );
    }
}
