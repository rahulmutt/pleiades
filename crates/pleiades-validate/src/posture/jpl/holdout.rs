//! Relocated independent-holdout renderers copied from
//! `pleiades-jpl::reference_summary::holdout` (report-surface relocation
//! program, Slice D). Rendering only — the functional crate keeps the
//! structured evidence structs, their `*_details()` constructors,
//! `validate()`/`label()` methods, and all release-gate data; jpl's own
//! rendering stays in place until the Task 14 contract sweep.

use pleiades_jpl::{
    IndependentHoldoutHighCurvatureSummary, IndependentHoldoutQuarterDayBoundarySummary,
    IndependentHoldoutSnapshotBatchParitySummary,
    IndependentHoldoutSnapshotBodyClassCoverageSummary,
    IndependentHoldoutSnapshotEquatorialParitySummary, IndependentHoldoutSnapshotSourceWindow,
    IndependentHoldoutSnapshotSourceWindowSummary, IndependentHoldoutSnapshotSummary,
    IndependentHoldoutSourceSummary, JplIndependentHoldoutSummary, ReferenceHoldoutOverlapSummary,
};

/// Reproduced from jpl's private (`pub(crate)`, not callable cross-crate)
/// `join_display`/`format_bodies` helpers
/// (`reference_summary/reference_snapshot/core/general_a.rs:502-512`).
fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

// ---------------------------------------------------------------------------
// Re-homed inherent `summary_line` renderers (one free fn per evidence struct).
// ---------------------------------------------------------------------------

/// Verbatim copy of `IndependentHoldoutSnapshotSummary::summary_line`
/// (reference_summary/holdout.rs:202).
pub(crate) fn independent_holdout_snapshot_summary_summary_line(
    s: &IndependentHoldoutSnapshotSummary,
) -> String {
    let bodies = if s.bodies.is_empty() {
        "none".to_string()
    } else {
        s.bodies.join(", ")
    };
    format!(
        "Independent hold-out coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
        s.row_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        bodies,
    )
}

/// Verbatim copy of `IndependentHoldoutSnapshotSourceWindow::summary_line`
/// (reference_summary/holdout.rs:251).
pub(crate) fn independent_holdout_snapshot_source_window_summary_line(
    s: &IndependentHoldoutSnapshotSourceWindow,
) -> String {
    let time_span = if s.earliest_epoch == s.latest_epoch {
        format_instant(s.earliest_epoch)
    } else {
        format!(
            "{}..{}",
            format_instant(s.earliest_epoch),
            format_instant(s.latest_epoch)
        )
    };

    format!(
        "{}: {} samples across {} epochs at {}",
        s.body, s.sample_count, s.epoch_count, time_span
    )
}

/// Verbatim copy of
/// `IndependentHoldoutSnapshotSourceWindowSummary::summary_line`
/// (reference_summary/holdout.rs:288), with the nested
/// `IndependentHoldoutSnapshotSourceWindow::summary_line` call rewired to the
/// local `independent_holdout_snapshot_source_window_summary_line`.
pub(crate) fn independent_holdout_snapshot_source_window_summary_summary_line(
    s: &IndependentHoldoutSnapshotSourceWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(independent_holdout_snapshot_source_window_summary_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Independent hold-out source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Verbatim copy of `IndependentHoldoutQuarterDayBoundarySummary::summary_line`
/// (reference_summary/holdout.rs:526).
pub(crate) fn independent_holdout_quarter_day_boundary_summary_summary_line(
    s: &IndependentHoldoutQuarterDayBoundarySummary,
) -> String {
    format!(
        "Independent hold-out quarter-day boundary samples: {} rows across {} bodies and {} epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); bodies: {}",
        s.row_count,
        s.body_count,
        s.epoch_count,
        format_bodies(&s.bodies),
    )
}

/// Verbatim copy of `IndependentHoldoutHighCurvatureSummary::summary_line`
/// (reference_summary/holdout.rs:695).
pub(crate) fn independent_holdout_high_curvature_summary_summary_line(
    s: &IndependentHoldoutHighCurvatureSummary,
) -> String {
    format!(
        "JPL independent hold-out high-curvature evidence: {} exact samples across {} bodies and {} epochs ({}..{}); bodies: {}; high-curvature interpolation window",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Verbatim copy of `ReferenceHoldoutOverlapSummary::summary_line`
/// (reference_summary/holdout.rs:900).
pub(crate) fn reference_holdout_overlap_summary_summary_line(
    s: &ReferenceHoldoutOverlapSummary,
) -> String {
    if s.shared_sample_count == 0 {
        return "Reference/hold-out overlap: 0 shared body-epoch pairs (reference snapshot and independent hold-out remain disjoint)".to_string();
    }

    format!(
        "Reference/hold-out overlap: {} shared body-epoch pairs across {} bodies and {} epochs; bodies: {}",
        s.shared_sample_count,
        s.shared_bodies.len(),
        s.shared_epoch_count,
        format_bodies(&s.shared_bodies),
    )
}

/// Verbatim copy of
/// `IndependentHoldoutSnapshotBodyClassCoverageSummary::summary_line`
/// (reference_summary/holdout.rs:1102), with the nested
/// `IndependentHoldoutSnapshotSourceWindow::summary_line` call rewired to the
/// local `independent_holdout_snapshot_source_window_summary_line`.
pub(crate) fn independent_holdout_snapshot_body_class_coverage_summary_summary_line(
    s: &IndependentHoldoutSnapshotBodyClassCoverageSummary,
) -> String {
    let windows = s
        .windows
        .iter()
        .map(independent_holdout_snapshot_source_window_summary_line)
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Independent hold-out body-class coverage: {} rows across {} bodies and {} epochs; bodies: {}; windows: {}",
        s.row_count,
        s.bodies.len(),
        s.epoch_count,
        format_bodies(&s.bodies),
        windows,
    )
}

/// Verbatim copy of
/// `IndependentHoldoutSnapshotBatchParitySummary::summary_line`
/// (reference_summary/holdout.rs:1430).
pub(crate) fn independent_holdout_snapshot_batch_parity_summary_summary_line(
    s: &IndependentHoldoutSnapshotBatchParitySummary,
) -> String {
    let order = if s.parity_preserved {
        "preserved"
    } else {
        "needs attention"
    };
    format!(
        "JPL independent hold-out batch parity: {} requests across {} bodies ({}) and {} epochs ({}..{}); TT requests={}, TDB requests={}; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; order={}, single-query parity={}",
        s.snapshot.row_count,
        s.snapshot.body_count,
        if s.snapshot.bodies.is_empty() {
            "none".to_string()
        } else {
            s.snapshot.bodies.join(", ")
        },
        s.snapshot.epoch_count,
        format_instant(s.snapshot.earliest_epoch),
        format_instant(s.snapshot.latest_epoch),
        s.tt_request_count,
        s.tdb_request_count,
        s.exact_count,
        s.interpolated_count,
        s.approximate_count,
        s.unknown_count,
        order,
        order,
    )
}

/// Verbatim copy of
/// `IndependentHoldoutSnapshotEquatorialParitySummary::summary_line`
/// (reference_summary/holdout.rs:1534).
pub(crate) fn independent_holdout_snapshot_equatorial_parity_summary_summary_line(
    s: &IndependentHoldoutSnapshotEquatorialParitySummary,
) -> String {
    format!(
        "JPL independent hold-out equatorial parity: {} rows across {} bodies and {} epochs ({}..{}); mean-obliquity transform against the checked-in ecliptic fixture",
        s.row_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
    )
}

/// Verbatim copy of `IndependentHoldoutSourceSummary::summary_line`
/// (reference_summary/holdout.rs:1820).
pub(crate) fn independent_holdout_source_summary_summary_line(
    s: &IndependentHoldoutSourceSummary,
) -> String {
    format!(
        "Independent hold-out source: {}; evidence class={}; coverage={}; columns={}; redistribution={}; checksum=0x{:016x}; {}; time scale={}",
        s.source, s.evidence_class, s.coverage, s.columns, s.redistribution, s.checksum, s.frame_treatment, s.time_scale
    )
}

/// Verbatim copy of `JplIndependentHoldoutSummary::summary_line`
/// (reference_summary/holdout.rs:2171).
pub(crate) fn jpl_independent_holdout_summary_summary_line(
    s: &JplIndependentHoldoutSummary,
) -> String {
    fn format_body_epoch_suffix(body: &str, epoch: pleiades_types::Instant) -> String {
        if body.is_empty() {
            String::new()
        } else {
            format!(" ({body} @ {})", format_instant(epoch))
        }
    }

    format!(
        "JPL independent hold-out: {} exact rows across {} bodies ({}) and {} epochs ({} → {}); max Δlon={:.12}°{}; mean Δlon={:.12}°; median Δlon={:.12}°; p95 Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}°{}; mean Δlat={:.12}°; median Δlat={:.12}°; p95 Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU{}; mean Δdist={:.12} AU; median Δdist={:.12} AU; p95 Δdist={:.12} AU; rms Δdist={:.12} AU; transparency evidence only, not a production tolerance envelope; independent JPL Horizons rows held out from the main snapshot corpus",
        s.sample_count,
        s.body_count,
        if s.bodies.is_empty() {
            "none".to_string()
        } else {
            s.bodies.join(", ")
        },
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        s.max_longitude_error_deg,
        format_body_epoch_suffix(&s.max_longitude_error_body, s.max_longitude_error_epoch),
        s.mean_longitude_error_deg,
        s.median_longitude_error_deg,
        s.percentile_longitude_error_deg,
        s.rms_longitude_error_deg,
        s.max_latitude_error_deg,
        format_body_epoch_suffix(&s.max_latitude_error_body, s.max_latitude_error_epoch),
        s.mean_latitude_error_deg,
        s.median_latitude_error_deg,
        s.percentile_latitude_error_deg,
        s.rms_latitude_error_deg,
        s.max_distance_error_au,
        format_body_epoch_suffix(&s.max_distance_error_body, s.max_distance_error_epoch),
        s.mean_distance_error_au,
        s.median_distance_error_au,
        s.percentile_distance_error_au,
        s.rms_distance_error_au,
    )
}

/// Reproduced from jpl's `format_jpl_independent_holdout_summary`
/// (reference_summary/holdout.rs:2332); a rendering helper (not one of the 13
/// `_for_report` renderers) that jpl's `jpl_independent_holdout_summary_for_report`
/// depends on. `summary.validated_summary_line()` is expanded to
/// `validate()` (which stays in jpl) plus the local render fn.
pub(crate) fn format_jpl_independent_holdout_summary(s: &JplIndependentHoldoutSummary) -> String {
    match s.validate() {
        Ok(()) => jpl_independent_holdout_summary_summary_line(s),
        Err(error) => format!("JPL independent hold-out: unavailable ({error})"),
    }
}

/// Reproduced from jpl's `format_independent_holdout_snapshot_batch_parity_summary`
/// (reference_summary/holdout.rs:1467); a thin rendering wrapper (not one of
/// the 13 `_for_report` renderers) exercised directly by the copied report
/// test.
pub(crate) fn format_independent_holdout_snapshot_batch_parity_summary(
    s: &IndependentHoldoutSnapshotBatchParitySummary,
) -> String {
    independent_holdout_snapshot_batch_parity_summary_summary_line(s)
}

// ---------------------------------------------------------------------------
// The 13 free `*_for_report` renderers, copied verbatim (validate()/gates stay
// in jpl and are called cross-crate; rendering is local).
// ---------------------------------------------------------------------------

/// Returns the release-facing independent hold-out source window summary string.
/// Verbatim copy of jpl's
/// `independent_holdout_snapshot_source_window_summary_for_report`
/// (reference_summary/holdout.rs:474).
pub fn independent_holdout_snapshot_source_window_summary_for_report() -> String {
    match pleiades_jpl::independent_holdout_snapshot_source_window_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => independent_holdout_snapshot_source_window_summary_summary_line(&summary),
            Err(error) => format!("Independent hold-out source windows: unavailable ({error})"),
        },
        None => "Independent hold-out source windows: unavailable".to_string(),
    }
}

/// Returns the compact quarter-day boundary sample summary for release-facing
/// reporting. Verbatim copy of jpl's
/// `independent_holdout_snapshot_quarter_day_boundary_summary_for_report`
/// (reference_summary/holdout.rs:643). The `*_details()` data constructor was
/// promoted to `pub` in jpl (Slice D Task 5) so it can be called cross-crate.
pub(crate) fn independent_holdout_snapshot_quarter_day_boundary_summary_for_report() -> String {
    match pleiades_jpl::independent_holdout_quarter_day_boundary_summary_details() {
        Some(summary) => match summary.validate() {
            Ok(()) => independent_holdout_quarter_day_boundary_summary_summary_line(&summary),
            Err(error) => {
                format!("Independent hold-out quarter-day boundary samples: unavailable ({error})")
            }
        },
        None => "Independent hold-out quarter-day boundary samples: unavailable".to_string(),
    }
}

/// Returns the release-facing independent hold-out high-curvature summary
/// string. Verbatim copy of jpl's
/// `independent_holdout_high_curvature_summary_for_report`
/// (reference_summary/holdout.rs:852).
pub fn independent_holdout_high_curvature_summary_for_report() -> String {
    match pleiades_jpl::independent_holdout_high_curvature_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => independent_holdout_high_curvature_summary_summary_line(&summary),
            Err(error) => {
                format!("JPL independent hold-out high-curvature evidence: unavailable ({error})")
            }
        },
        None => "JPL independent hold-out high-curvature evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing reference/hold-out overlap summary string.
/// Verbatim copy of jpl's `reference_holdout_overlap_summary_for_report`
/// (reference_summary/holdout.rs:1027).
pub fn reference_holdout_overlap_summary_for_report() -> String {
    match validated_reference_holdout_overlap_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference/hold-out overlap: unavailable ({error})"),
    }
}

/// Returns the validated release-facing reference/hold-out overlap summary
/// string. Verbatim copy of jpl's
/// `validated_reference_holdout_overlap_summary_for_report`
/// (reference_summary/holdout.rs:1035).
pub(crate) fn validated_reference_holdout_overlap_summary_for_report() -> Result<String, String> {
    let summary = pleiades_jpl::reference_holdout_overlap_summary()
        .ok_or_else(|| "reference/hold-out overlap unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(reference_holdout_overlap_summary_summary_line(&summary))
}

/// Returns the release-facing independent hold-out coverage summary string.
/// Verbatim copy of jpl's `independent_holdout_snapshot_summary_for_report`
/// (reference_summary/holdout.rs:1051). `independent_holdout_snapshot_error`
/// was promoted to `pub` in jpl (Slice D Task 5) so the `None` fallback can be
/// rendered cross-crate.
pub(crate) fn independent_holdout_snapshot_summary_for_report() -> String {
    match pleiades_jpl::independent_holdout_snapshot_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => independent_holdout_snapshot_summary_summary_line(&summary),
            Err(error) => format!("Independent hold-out coverage: unavailable ({error})"),
        },
        None => match pleiades_jpl::independent_holdout_snapshot_error() {
            Some(error) => format!("Independent hold-out coverage: unavailable ({error})"),
            None => "Independent hold-out coverage: unavailable".to_string(),
        },
    }
}

/// Returns the release-facing body-class coverage summary string for the
/// independent hold-out snapshot. Verbatim copy of jpl's
/// `independent_holdout_snapshot_body_class_coverage_summary_for_report`
/// (reference_summary/holdout.rs:1200).
pub(crate) fn independent_holdout_snapshot_body_class_coverage_summary_for_report() -> String {
    match pleiades_jpl::independent_holdout_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => {
                independent_holdout_snapshot_body_class_coverage_summary_summary_line(&summary)
            }
            Err(error) => {
                format!("Independent hold-out body-class coverage: unavailable ({error})")
            }
        },
        None => "Independent hold-out body-class coverage: unavailable".to_string(),
    }
}

/// Returns the release-facing independent hold-out mixed-scale batch parity
/// summary string. Verbatim copy of jpl's
/// `independent_holdout_snapshot_batch_parity_summary_for_report`
/// (reference_summary/holdout.rs:1474).
pub fn independent_holdout_snapshot_batch_parity_summary_for_report() -> String {
    match pleiades_jpl::independent_holdout_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => independent_holdout_snapshot_batch_parity_summary_summary_line(&summary),
            Err(error) => format!("JPL independent hold-out batch parity: unavailable ({error})"),
        },
        None => "JPL independent hold-out batch parity: unavailable".to_string(),
    }
}

/// Returns the validated release-facing independent hold-out mixed-scale batch
/// parity summary string. Verbatim copy of jpl's
/// `validated_independent_holdout_snapshot_batch_parity_summary_for_report`
/// (reference_summary/holdout.rs:1485).
pub(crate) fn validated_independent_holdout_snapshot_batch_parity_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::independent_holdout_snapshot_batch_parity_summary()
        .ok_or_else(|| "JPL independent hold-out batch parity: unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(independent_holdout_snapshot_batch_parity_summary_summary_line(&summary))
}

/// Returns the release-facing independent hold-out equatorial parity summary
/// string. Verbatim copy of jpl's
/// `independent_holdout_snapshot_equatorial_parity_summary_for_report`
/// (reference_summary/holdout.rs:1652).
pub fn independent_holdout_snapshot_equatorial_parity_summary_for_report() -> String {
    match pleiades_jpl::independent_holdout_snapshot_equatorial_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => independent_holdout_snapshot_equatorial_parity_summary_summary_line(&summary),
            Err(error) => {
                format!("JPL independent hold-out equatorial parity: unavailable ({error})")
            }
        },
        None => "JPL independent hold-out equatorial parity: unavailable".to_string(),
    }
}

/// Returns the source-material summary for the checked-in hold-out snapshot.
/// Verbatim copy of jpl's `independent_holdout_source_summary_for_report`
/// (reference_summary/holdout.rs:1933).
pub fn independent_holdout_source_summary_for_report() -> String {
    if let Err(error) = pleiades_jpl::independent_holdout_snapshot_manifest().validate() {
        return format!("Independent hold-out source: unavailable ({error})");
    }

    let summary = pleiades_jpl::independent_holdout_source_summary();
    match summary.validate() {
        Ok(()) => independent_holdout_source_summary_summary_line(&summary),
        Err(error) => format!("Independent hold-out source: unavailable ({error})"),
    }
}

/// Returns the manifest summary for the checked-in hold-out snapshot. Verbatim
/// copy of jpl's `independent_holdout_manifest_summary_for_report`
/// (reference_summary/holdout.rs:1956), with the manifest header gate called
/// cross-crate (`pleiades_jpl::validate_snapshot_manifest_header_structure`,
/// `pub` from Task 4b), the footprint gate called cross-crate
/// (`pleiades_jpl::validate_snapshot_manifest_footprint`, promoted to `pub` in
/// Task 5), and the final manifest line delegated to Task 2's
/// `crate::posture::jpl::backend::snapshot_manifest_summary_line`.
///
/// The `manifest_text` `include_str!` reaches one directory over to jpl's
/// checked-in copy of the same CSV (jpl's own `env!("CARGO_MANIFEST_DIR")`
/// resolves against jpl's manifest dir, not validate's); the bytes read are
/// identical either way (established precedent — comparison.rs:284 and
/// validate/src/corpus/production.rs:218 do the same).
pub fn independent_holdout_manifest_summary_for_report() -> String {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../pleiades-jpl/data/independent_holdout_snapshot.csv"
    ));
    if let Err(error) = pleiades_jpl::validate_snapshot_manifest_header_structure(
        manifest_text,
        "Independent JPL Horizons hold-out snapshot used only for interpolation validation.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.",
        Some("repository-checked regression fixtures, not a broad public corpus."),
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        return format!("Independent hold-out manifest: unavailable ({error})");
    }

    let summary = pleiades_jpl::independent_holdout_manifest_summary();
    match summary.validate_with_expected_metadata(
        "Independent JPL Horizons hold-out snapshot used only for interpolation validation.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        Ok(()) => match pleiades_jpl::validate_snapshot_manifest_footprint(
            "independent hold-out snapshot",
            pleiades_jpl::independent_holdout_snapshot_entries(),
            66,
            16,
            12,
        ) {
            Ok(()) => crate::posture::jpl::backend::snapshot_manifest_summary_line(&summary),
            Err(error) => format!("Independent hold-out manifest: unavailable ({error})"),
        },
        Err(error) => format!("Independent hold-out manifest: unavailable ({error})"),
    }
}

/// Returns the release-facing independent hold-out interpolation summary
/// string. Verbatim copy of jpl's `jpl_independent_holdout_summary_for_report`
/// (reference_summary/holdout.rs:2340). `independent_holdout_snapshot_error`
/// was promoted to `pub` in jpl (Slice D Task 5) so the `None` fallback can be
/// rendered cross-crate.
pub fn jpl_independent_holdout_summary_for_report() -> String {
    match pleiades_jpl::jpl_independent_holdout_summary() {
        Some(summary) => format_jpl_independent_holdout_summary(&summary),
        None => match pleiades_jpl::independent_holdout_snapshot_error() {
            Some(error) => format!("JPL independent hold-out: unavailable ({error})"),
            None => "JPL independent hold-out: unavailable".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_holdout_overlap_summary_reports_the_current_overlap() {
        let summary = pleiades_jpl::reference_holdout_overlap_summary()
            .expect("reference/hold-out overlap summary should exist");

        assert_eq!(summary.shared_sample_count, 66);
        assert_eq!(summary.shared_epoch_count, 12);
        assert_eq!(summary.shared_bodies.len(), 16);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_holdout_overlap_summary_for_report(),
            reference_holdout_overlap_summary_summary_line(&summary)
        );
        assert_eq!(
            validated_reference_holdout_overlap_summary_for_report(),
            Ok(reference_holdout_overlap_summary_summary_line(&summary))
        );
        assert_eq!(
            reference_holdout_overlap_summary_summary_line(&summary),
            format!(
                "Reference/hold-out overlap: 66 shared body-epoch pairs across 16 bodies and 12 epochs; bodies: {}",
                format_bodies(&summary.shared_bodies)
            )
        );
    }

    #[test]
    fn independent_holdout_source_summary_reports_the_expected_provenance() {
        let summary = pleiades_jpl::independent_holdout_source_summary();

        assert_eq!(
            summary.source,
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
        );
        assert_eq!(
            summary.coverage,
            "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs."
        );
        // `INDEPENDENT_HOLDOUT_*` are `pub(crate)` consts in jpl's not-yet-copied
        // general_b.rs; their literal values are inlined here (jpl's retained test
        // still asserts the const equality directly).
        assert_eq!(summary.evidence_class, "hold-out");
        assert_eq!(summary.columns, "epoch_jd, body, x_km, y_km, z_km");
        assert_eq!(summary.frame_treatment, "geocentric ecliptic J2000");
        assert_eq!(summary.time_scale, "TDB");
        assert!(
            independent_holdout_source_summary_summary_line(&summary).contains("time scale=TDB")
        );
        assert_eq!(
            summary.redistribution,
            "repository-checked regression fixtures, not a broad public corpus."
        );
        assert!(independent_holdout_source_summary_summary_line(&summary)
            .contains("evidence class=hold-out"));
        assert!(
            independent_holdout_source_summary_summary_line(&summary).contains(
                "redistribution=repository-checked regression fixtures, not a broad public corpus."
            )
        );
        assert!(independent_holdout_source_summary_summary_line(&summary)
            .contains(&format!("checksum=0x{:016x}", summary.checksum)));
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            independent_holdout_source_summary_for_report(),
            independent_holdout_source_summary_summary_line(&summary)
        );
    }

    #[test]
    fn independent_holdout_snapshot_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::independent_holdout_snapshot_summary()
            .expect("independent hold-out summary should exist");
        assert_eq!(summary.row_count, 66);
        assert_eq!(summary.body_count, 16);
        assert_eq!(
            summary.bodies,
            vec![
                "Mars",
                "Jupiter",
                "Mercury",
                "Venus",
                "Saturn",
                "Uranus",
                "Neptune",
                "Sun",
                "Pluto",
                "Moon",
                "Ceres",
                "Pallas",
                "Juno",
                "Vesta",
                "asteroid:433-Eros",
                "asteroid:99942-Apophis",
            ]
        );
        assert_eq!(summary.epoch_count, 12);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            independent_holdout_snapshot_summary_summary_line(&summary),
            "Independent hold-out coverage: 66 rows across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"
        );
        assert_eq!(
            independent_holdout_snapshot_summary_for_report(),
            independent_holdout_snapshot_summary_summary_line(&summary)
        );
    }

    #[test]
    fn independent_holdout_snapshot_source_window_summary_reports_the_expected_windows() {
        let summary = pleiades_jpl::independent_holdout_snapshot_source_window_summary()
            .expect("independent hold-out source window summary should exist");
        assert_eq!(summary.sample_count, 66);
        assert_eq!(summary.sample_bodies.len(), 16);
        // (jpl's retained test additionally asserts `sample_bodies ==
        // independent_holdout_bodies()`, which is `pub(crate)` and not callable
        // cross-crate; the rendered-string byte-identity below is the concern here.)
        assert_eq!(summary.epoch_count, 12);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.windows.len(), 16);
        assert_eq!(
            summary.windows[0].body,
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(summary.windows[0].sample_count, 5);
        assert_eq!(summary.windows[0].epoch_count, 5);
        assert_eq!(
            summary.windows[0].earliest_epoch.julian_day.days(),
            2_451_545.0
        );
        assert_eq!(
            summary.windows[0].latest_epoch.julian_day.days(),
            2_451_915.5
        );
        assert_eq!(
            summary.windows[8].body,
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.windows[8].sample_count, 2);
        assert_eq!(summary.windows[8].epoch_count, 2);
        assert_eq!(
            summary.windows[8].earliest_epoch.julian_day.days(),
            2_451_545.0
        );
        assert_eq!(
            summary.windows[8].latest_epoch.julian_day.days(),
            2_451_915.5
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            independent_holdout_snapshot_source_window_summary_for_report(),
            independent_holdout_snapshot_source_window_summary_summary_line(&summary)
        );
        assert!(
            independent_holdout_snapshot_source_window_summary_summary_line(&summary).contains(
                "Independent hold-out source windows: 66 source-backed samples across 16 bodies and 12 epochs"
            )
        );
    }

    #[test]
    fn independent_holdout_quarter_day_boundary_summary_reports_the_expected_window() {
        let summary = pleiades_jpl::independent_holdout_quarter_day_boundary_summary_details()
            .expect("independent hold-out quarter-day boundary summary should exist");
        assert_eq!(summary.row_count, 8);
        assert_eq!(summary.body_count, 4);
        assert_eq!(
            summary.bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
            ]
        );
        assert_eq!(summary.epoch_count, 2);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_915.25);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_915.75);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            independent_holdout_snapshot_quarter_day_boundary_summary_for_report(),
            independent_holdout_quarter_day_boundary_summary_summary_line(&summary)
        );
        assert!(
            independent_holdout_quarter_day_boundary_summary_summary_line(&summary).contains(
                "Independent hold-out quarter-day boundary samples: 8 rows across 4 bodies and 2 epochs"
            )
        );
    }

    #[test]
    fn independent_holdout_high_curvature_summary_reports_the_expected_window() {
        let summary = pleiades_jpl::independent_holdout_high_curvature_summary()
            .expect("independent hold-out high-curvature summary should exist");
        assert_eq!(summary.sample_count, 8);
        assert_eq!(summary.sample_bodies.len(), 4);
        assert_eq!(
            summary.sample_bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
            ]
        );
        assert_eq!(summary.epoch_count, 2);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_915.25);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_915.75);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            independent_holdout_high_curvature_summary_for_report(),
            independent_holdout_high_curvature_summary_summary_line(&summary)
        );
        assert!(
            independent_holdout_high_curvature_summary_summary_line(&summary).contains(
                "JPL independent hold-out high-curvature evidence: 8 exact samples across 4 bodies and 2 epochs"
            )
        );
    }

    #[test]
    fn independent_holdout_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::independent_holdout_snapshot_equatorial_parity_summary()
            .expect("independent hold-out equatorial parity summary should exist");
        assert_eq!(summary.row_count, 66);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.epoch_count, 12);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            independent_holdout_snapshot_equatorial_parity_summary_summary_line(&summary),
            "JPL independent hold-out equatorial parity: 66 rows across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); mean-obliquity transform against the checked-in ecliptic fixture"
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            independent_holdout_snapshot_equatorial_parity_summary_for_report(),
            independent_holdout_snapshot_equatorial_parity_summary_summary_line(&summary)
        );
    }

    #[test]
    fn independent_holdout_summary_reports_the_expected_envelope() {
        let summary = pleiades_jpl::jpl_independent_holdout_summary()
            .expect("independent hold-out summary should exist");
        assert_eq!(summary.sample_count, 66);
        assert_eq!(summary.body_count, 16);
        assert_eq!(
            summary.bodies,
            vec![
                "Mars",
                "Jupiter",
                "Mercury",
                "Venus",
                "Saturn",
                "Uranus",
                "Neptune",
                "Sun",
                "Pluto",
                "Moon",
                "Ceres",
                "Pallas",
                "Juno",
                "Vesta",
                "asteroid:433-Eros",
                "asteroid:99942-Apophis",
            ]
        );
        assert_eq!(summary.epoch_count, 12);
        assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
        assert!(summary.max_longitude_error_deg.is_finite());
        assert!(summary.mean_longitude_error_deg.is_finite());
        assert!(summary.median_longitude_error_deg.is_finite());
        assert!(summary.percentile_longitude_error_deg.is_finite());
        assert!(summary.rms_longitude_error_deg.is_finite());
        assert!(summary.max_latitude_error_deg.is_finite());
        assert!(summary.mean_latitude_error_deg.is_finite());
        assert!(summary.median_latitude_error_deg.is_finite());
        assert!(summary.percentile_latitude_error_deg.is_finite());
        assert!(summary.rms_latitude_error_deg.is_finite());
        assert!(summary.max_distance_error_au.is_finite());
        assert!(summary.mean_distance_error_au.is_finite());
        assert!(summary.median_distance_error_au.is_finite());
        assert!(summary.percentile_distance_error_au.is_finite());
        assert!(summary.rms_distance_error_au.is_finite());
        assert!(!summary.max_longitude_error_body.is_empty());
        assert!(!summary.max_latitude_error_body.is_empty());
        assert!(!summary.max_distance_error_body.is_empty());

        let rendered = format_jpl_independent_holdout_summary(&summary);
        assert!(rendered.contains("JPL independent hold-out:"));
        assert!(rendered.contains(
            "66 exact rows across 16 bodies (Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) and 12 epochs"
        ));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(
            rendered.contains("transparency evidence only, not a production tolerance envelope")
        );
        assert!(rendered
            .contains("independent JPL Horizons rows held out from the main snapshot corpus"));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_longitude_error_body,
            format_instant(summary.max_longitude_error_epoch)
        )));
    }

    #[test]
    fn batch_query_preserves_independent_holdout_mixed_scale_order_and_single_query_parity() {
        let summary = pleiades_jpl::independent_holdout_snapshot_batch_parity_summary()
            .expect("independent hold-out batch parity summary should exist");
        assert_eq!(summary.snapshot.row_count, 66);
        assert_eq!(summary.snapshot.body_count, 16);
        assert_eq!(summary.tt_request_count, 33);
        assert_eq!(summary.tdb_request_count, 33);
        assert!(summary.parity_preserved);
        assert_eq!(
            summary.exact_count
                + summary.interpolated_count
                + summary.approximate_count
                + summary.unknown_count,
            summary.snapshot.row_count,
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            validated_independent_holdout_snapshot_batch_parity_summary_for_report(),
            Ok(independent_holdout_snapshot_batch_parity_summary_summary_line(&summary))
        );

        let rendered = format_independent_holdout_snapshot_batch_parity_summary(&summary);
        assert!(rendered.contains("JPL independent hold-out batch parity:"));
        assert!(rendered.contains(
            "66 requests across 16 bodies (Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) and 12 epochs"
        ));
        assert!(rendered.contains("TT requests=33, TDB requests=33"));
        assert!(rendered.contains("quality counts:"));
        assert!(rendered.contains("order=preserved, single-query parity=preserved"));
    }

    #[test]
    fn independent_holdout_summary_validated_summary_line_rejects_drift() {
        let mut summary =
            pleiades_jpl::jpl_independent_holdout_summary().expect("summary should exist");
        summary.sample_count += 1;
        assert_eq!(
            format_jpl_independent_holdout_summary(&summary),
            "JPL independent hold-out: unavailable (summary no longer matches the derived interpolation evidence)"
        );
    }
}
