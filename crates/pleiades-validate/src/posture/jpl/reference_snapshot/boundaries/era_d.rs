//! Relocated reference-snapshot boundary renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::boundaries::era_d`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()`/`*_summary()` constructors, `validate()`/`label()` methods,
//! and all release-gate data; jpl's own rendering for these structs was
//! deleted in the Task 14b contract sweep.
//!
//! This file textually defines the rendering (`summary_line`/
//! `validated_summary_line`/`Display`) for 10 evidence structs — 9 of them
//! (`Reference2451919MajorBodyBoundarySummary`,
//! `Reference2451916MajorBodyInteriorSummary`,
//! `Reference2451920MajorBodyInteriorSummary`, `ReferenceMajorBodyBoundaryWindow`,
//! `ReferenceMajorBodyBoundaryWindowSummary`, `ReferenceHighCurvatureWindow`,
//! `ReferenceHighCurvatureWindowSummary`, `ReferenceHighCurvatureEpochCoverage`,
//! `ReferenceHighCurvatureEpochCoverageSummary`) with their struct/enum also
//! textually defined here, plus one (`Reference2451917MajorBodyBoundarySummary`)
//! whose struct is textually defined in `boundaries/era_c.rs` (Slice D Task
//! 9b, already copied) — that file's own doc comment already flags this
//! split. Each struct's inherent `summary_line` (10) is re-homed below as
//! `pub(crate) fn <struct_snake>_summary_line` (the two non-`Summary`-suffixed
//! window/coverage structs — `ReferenceMajorBodyBoundaryWindow`,
//! `ReferenceHighCurvatureWindow`, `ReferenceHighCurvatureEpochCoverage` — use
//! a bare `<struct_snake>_line` name to avoid colliding with their owning
//! `*Summary` struct's `<struct_snake>_summary_line`, mirroring
//! `general_b.rs`'s `ReferenceSnapshotSourceWindow`/`...WindowSummary` pair).
//! Every `validated_summary_line` (7 — the two bare window/coverage structs
//! and `Reference2451917MajorBodyBoundarySummary` have none, see below) is the
//! simple `{ self.validate()?; Ok(self.summary_line()) }` pattern (verified
//! per struct) — consistent with the family recipe, so no standalone re-home
//! is needed. None of these 10 structs has a local free `*_for_report`
//! renderer in this file (their owning wrappers live in
//! `reference_snapshot/core/{general_b,coverage}.rs`, Slice D Task 8, already
//! copied, still calling jpl's still-present inherent methods — validate→jpl
//! is allowed and byte-identical; Task 13 repoints them to the free fns
//! below), so — consistent with `era_a.rs`'s orphaned `validated_summary_line`
//! methods — no standalone free fn is added to fold validation into a local
//! render call site. The `Display` impls (10 evidence + 6 sibling
//! `*ValidationError`, all pure `f.write_str(&self.summary_line())`
//! forwarders for the evidence structs) are not reproduced as standalone
//! items, consistent with every prior posture module — `Display` on a
//! foreign jpl type cannot be re-implemented here anyway (orphan rule); the
//! free-fn equivalent is the re-homed rendering surface, and
//! `*ValidationError` formatting stays in jpl untouched (it is gate/error
//! logic, not report rendering). `Reference2451917MajorBodyBoundarySummaryValidationError`'s
//! `Display` lives in `boundaries/era_c.rs` alongside its struct — not
//! referenced here.
//!
//! This file also textually defines 4 free `*_for_report` renderers, copied
//! verbatim below with `self`→`s` and cross-crate data-accessor calls
//! qualified `pleiades_jpl::` (`reference_snapshot_bridge_day_summary`,
//! `reference_snapshot_2451914_major_body_bridge_day_summary` — both data
//! accessors, never duplicated into validate, per every prior posture
//! module's treatment): `reference_snapshot_bridge_day_summary_for_report`,
//! `validated_reference_snapshot_bridge_day_summary_for_report`,
//! `reference_snapshot_2451914_bridge_day_summary_for_report` (same-file
//! nested call, kept local), `reference_snapshot_2451914_major_body_bridge_day_summary_for_report`.
//! The first two operate on `ReferenceSnapshotBridgeDaySummary` — a struct
//! defined in `reference_snapshot/core/general_b.rs` (Slice D Task 8, already
//! copied) — via a cross-file call to the local
//! `reference_snapshot_bridge_day_summary_line` free fn (repointed in Slice D
//! Task 13; jpl's own inherent method was deleted in the Task 14b contract
//! sweep).
//!
//! The jpl test file (`reference_summary/reference_snapshot/tests.rs`) had 4
//! bridge-day tests deferred by Slice D Task 8c because they exercise this
//! file's free renderers, which had not yet been copied:
//! `reference_snapshot_bridge_day_summary_reports_the_bridge_day` (:348),
//! `reference_snapshot_bridge_day_summary_validation_rejects_drift` (:396),
//! `reference_snapshot_bridge_day_summary_validation_rejects_body_drift`
//! (:422), and `reference_snapshot_2451914_and_2451915_boundary_aliases_match_the_generic_reports`
//! (:923). All 4 are copied into this file's test module below, pointed at
//! this file's local free fns and (for renderers still owned by
//! `general_b.rs`/`era_a.rs`) their already-copied `pub(crate)` counterparts
//! via `crate::posture::jpl::reference_snapshot::core::general_b::` —
//! every renderer each test references is now copied somewhere in validate.
//! This file's own test module additionally pins byte-identity between jpl's
//! still-present inherent `summary_line()` and the free-fn copies for the 10
//! evidence structs (mirroring Task 2's `posture/jpl/backend.rs` golden
//! pattern) — no separate golden entry is added for the two bare
//! window/coverage structs since their rendering is already exercised
//! transitively through their owning `*Summary` aggregate's golden assertion,
//! mirroring `general_b.rs`'s treatment of `ReferenceSnapshotSourceWindow`.

use pleiades_jpl::{
    Reference2451916MajorBodyInteriorSummary, Reference2451917MajorBodyBoundarySummary,
    Reference2451919MajorBodyBoundarySummary, Reference2451920MajorBodyInteriorSummary,
    ReferenceHighCurvatureEpochCoverage, ReferenceHighCurvatureEpochCoverageSummary,
    ReferenceHighCurvatureWindow, ReferenceHighCurvatureWindowSummary,
    ReferenceMajorBodyBoundaryWindow, ReferenceMajorBodyBoundaryWindowSummary,
};

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Compact release-facing summary line for the 2451917.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451917MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:15). The struct
/// itself is textually defined in `boundaries/era_c.rs`; this file only
/// re-homes its rendering, reading the struct's public fields directly.
/// Reuses the canonical `format_bodies` copy in `core::general_a` rather
/// than reproducing it locally.
pub(crate) fn reference_2451917_major_body_boundary_summary_line(
    s: &Reference2451917MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451917 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-08 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451919.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451919MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:169). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451919_major_body_boundary_summary_line(
    s: &Reference2451919MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451919 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-10 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451916.0 major-body interior
/// reference evidence. Verbatim copy of
/// `Reference2451916MajorBodyInteriorSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:323). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451916_major_body_interior_summary_line(
    s: &Reference2451916MajorBodyInteriorSummary,
) -> String {
    format!(
        "Reference 2451916 major-body interior evidence: {} exact samples at {} ({}); 2001-01-06 interior reference sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451920.5 major-body interior
/// reference evidence. Verbatim copy of
/// `Reference2451920MajorBodyInteriorSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:477). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451920_major_body_interior_summary_line(
    s: &Reference2451920MajorBodyInteriorSummary,
) -> String {
    format!(
        "Reference 2451920 major-body interior evidence: {} exact samples at {} ({}); 2001-01-13 interior reference sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing body-window summary line used inside the
/// major-body boundary-day window aggregate. Verbatim copy of
/// `ReferenceMajorBodyBoundaryWindow::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:574).
pub(crate) fn reference_major_body_boundary_window_line(
    s: &ReferenceMajorBodyBoundaryWindow,
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

/// Compact release-facing summary line for the major-body boundary-day
/// reference coverage windows. Verbatim copy of
/// `ReferenceMajorBodyBoundaryWindowSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:637). Same-file
/// nested call (`ReferenceMajorBodyBoundaryWindow::summary_line`) rewritten
/// to the local `reference_major_body_boundary_window_line` (per the
/// recipe).
pub(crate) fn reference_major_body_boundary_window_summary_line(
    s: &ReferenceMajorBodyBoundaryWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(reference_major_body_boundary_window_line)
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Reference major-body boundary windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Compact release-facing body-window summary line used inside the
/// major-body high-curvature window aggregate. Verbatim copy of
/// `ReferenceHighCurvatureWindow::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:802).
pub(crate) fn reference_high_curvature_window_line(s: &ReferenceHighCurvatureWindow) -> String {
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

/// Compact release-facing summary line for the major-body high-curvature
/// reference coverage. Verbatim copy of
/// `ReferenceHighCurvatureWindowSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:845). Same-file
/// nested call (`ReferenceHighCurvatureWindow::summary_line`) rewritten to
/// the local `reference_high_curvature_window_line` (per the recipe).
pub(crate) fn reference_high_curvature_window_summary_line(
    s: &ReferenceHighCurvatureWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(reference_high_curvature_window_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Reference major-body high-curvature windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Compact release-facing epoch-coverage summary line used inside the
/// major-body high-curvature epoch coverage aggregate. Verbatim copy of
/// `ReferenceHighCurvatureEpochCoverage::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:970). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_high_curvature_epoch_coverage_line(
    s: &ReferenceHighCurvatureEpochCoverage,
) -> String {
    format!(
        "{}: {} bodies ({})",
        format_instant(s.epoch),
        s.body_count,
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.bodies),
    )
}

/// Compact release-facing summary line for the major-body high-curvature
/// epoch coverage. Verbatim copy of
/// `ReferenceHighCurvatureEpochCoverageSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:1003). Same-file
/// nested call (`ReferenceHighCurvatureEpochCoverage::summary_line`)
/// rewritten to the local `reference_high_curvature_epoch_coverage_line` (per
/// the recipe).
pub(crate) fn reference_high_curvature_epoch_coverage_summary_line(
    s: &ReferenceHighCurvatureEpochCoverageSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(reference_high_curvature_epoch_coverage_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Reference major-body high-curvature epoch coverage: {} exact samples across {} epochs ({}..{}); epochs: {}",
        s.sample_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Returns the release-facing bridge day summary string. Verbatim copy of
/// jpl's `reference_snapshot_bridge_day_summary_for_report`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:731), with the
/// data-constructor call qualified `pleiades_jpl::`. `summary.validated_summary_line()`
/// (a cross-file struct-method call — `ReferenceSnapshotBridgeDaySummary` is
/// defined in `core/general_b.rs`) is rewired to `match summary.validate() {
/// Ok(()) => <local render>, ... }`, calling the local
/// `reference_snapshot_bridge_day_summary_line` (Slice D Task 13b; `validate()`
/// stays on the jpl struct, rendering is local).
pub fn reference_snapshot_bridge_day_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_bridge_day_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => {
                crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(&summary)
            }
            Err(error) => format!("Reference snapshot bridge day: unavailable ({error})"),
        },
        None => "Reference snapshot bridge day: unavailable".to_string(),
    }
}

/// Returns the validated release-facing bridge day summary string. Verbatim
/// copy of jpl's `validated_reference_snapshot_bridge_day_summary_for_report`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:742), with the
/// data-constructor call qualified `pleiades_jpl::`. `summary.validated_summary_line()`
/// is the same cross-file struct-method call as above, rewired the same way
/// (Slice D Task 13b).
pub(crate) fn validated_reference_snapshot_bridge_day_summary_for_report() -> Result<String, String>
{
    let summary = pleiades_jpl::reference_snapshot_bridge_day_summary()
        .ok_or_else(|| "reference snapshot bridge day unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(
        crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(&summary),
    )
}

/// Returns the release-facing 2451914 bridge-day summary string. Verbatim
/// copy of jpl's `reference_snapshot_2451914_bridge_day_summary_for_report`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:757). Same-file
/// nested call, kept local.
pub(crate) fn reference_snapshot_2451914_bridge_day_summary_for_report() -> String {
    reference_snapshot_bridge_day_summary_for_report()
}

/// Returns the release-facing 2451914 major-body bridge-day summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451914_major_body_bridge_day_summary_for_report`
/// (reference_summary/reference_snapshot/boundaries/era_d.rs:768), with the
/// data-constructor call qualified `pleiades_jpl::` and `format_bodies`
/// reused from the canonical `core::general_a` copy.
pub fn reference_snapshot_2451914_major_body_bridge_day_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451914_major_body_bridge_day_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format!(
                "Reference 2451914 major-body bridge-day evidence: {} exact samples at {} ({}); 2451914 major-body bridge-day sample",
                summary.sample_count,
                format_instant(summary.epoch),
                crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&summary.sample_bodies),
            ),
            Err(error) => {
                format!("Reference 2451914 major-body bridge-day evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451914 major-body bridge-day evidence: unavailable".to_string(),
    }
}

#[cfg(test)]
mod golden {
    use super::*;

    // Task 14b (contract sweep) deleted these structs' jpl inherent
    // `summary_line` renderers, so the byte-identity `summary.summary_line()`
    // comparisons are gone; the captured literals below are the standing
    // regression guard for the validate copies.

    #[test]
    fn reference_2451917_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451917_major_body_boundary_summary()
            .expect("reference 2451917 major-body boundary summary should exist");
        assert_eq!(
            reference_2451917_major_body_boundary_summary_line(&summary),
            "Reference 2451917 major-body boundary evidence: 10 exact samples at JD 2451917.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-08 boundary sample"
        );
    }

    #[test]
    fn reference_2451919_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451919_major_body_boundary_summary()
            .expect("reference 2451919 major-body boundary summary should exist");
        assert_eq!(
            reference_2451919_major_body_boundary_summary_line(&summary),
            "Reference 2451919 major-body boundary evidence: 10 exact samples at JD 2451919.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-10 boundary sample"
        );
    }

    #[test]
    fn reference_2451916_major_body_interior_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451916_major_body_interior_summary()
            .expect("reference 2451916 major-body interior summary should exist");
        assert_eq!(
            reference_2451916_major_body_interior_summary_line(&summary),
            "Reference 2451916 major-body interior evidence: 10 exact samples at JD 2451916.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-06 interior reference sample"
        );
    }

    #[test]
    fn reference_2451920_major_body_interior_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451920_major_body_interior_summary()
            .expect("reference 2451920 major-body interior summary should exist");
        assert_eq!(
            reference_2451920_major_body_interior_summary_line(&summary),
            "Reference 2451920 major-body interior evidence: 10 exact samples at JD 2451920.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-13 interior reference sample"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The following 4 tests were deferred by Slice D Task 8c
    // (reference_summary/reference_snapshot/tests.rs:348, :396, :422, :923)
    // because they exercise this file's free renderers, which had not yet
    // been copied. All renderers they reference are now copied (either in
    // this file or in `core::general_b`/`core::era_a`, Task 8), so they are
    // copied here verbatim, pointed at the local/already-copied fns.

    #[test]
    fn reference_snapshot_bridge_day_summary_reports_the_bridge_day() {
        let summary = pleiades_jpl::reference_snapshot_bridge_day_summary()
            .expect("reference snapshot bridge day summary should exist");
        assert_eq!(summary.sample_count, 15);
        assert_eq!(summary.sample_bodies.len(), 15);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_914.0);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(&summary),
            "Reference snapshot bridge day: 15 exact samples at JD 2451914.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); bridge sample across the reference boundary window"
        );
        assert_eq!(
            reference_snapshot_bridge_day_summary_for_report(),
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(&summary)
        );
        assert_eq!(
            validated_reference_snapshot_bridge_day_summary_for_report(),
            Ok(crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(&summary))
        );
        assert_eq!(
            pleiades_jpl::reference_snapshot_2451914_bridge_day_summary(),
            Some(summary.clone())
        );
        assert_eq!(
            reference_snapshot_2451914_bridge_day_summary_for_report(),
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(&summary)
        );
        assert_eq!(
            pleiades_jpl::reference_snapshot_2451914_bridge_day_summary(),
            Some(summary.clone())
        );
        assert_eq!(
            pleiades_jpl::reference_snapshot_2451914_major_body_bridge_day_summary(),
            Some(summary.clone())
        );
        assert_eq!(
            pleiades_jpl::reference_snapshot_2451914_major_body_bridge_summary(),
            Some(summary.clone())
        );
        assert_eq!(
            reference_snapshot_2451914_major_body_bridge_day_summary_for_report(),
            "Reference 2451914 major-body bridge-day evidence: 15 exact samples at JD 2451914.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); 2451914 major-body bridge-day sample"
        );
    }

    #[test]
    fn reference_snapshot_bridge_day_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_bridge_day_summary()
            .expect("reference snapshot bridge day summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted bridge day summary should fail validation");

        assert!(matches!(
            error,
            pleiades_jpl::ReferenceSnapshotBridgeDaySummaryValidationError::SampleCountMismatch {
                sample_count: 16,
                derived_sample_count: 15
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_bridge_day_summary_for_report(),
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(
                &pleiades_jpl::reference_snapshot_bridge_day_summary()
                    .expect("reference snapshot bridge day summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_bridge_day_summary_validation_rejects_body_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_bridge_day_summary()
            .expect("reference snapshot bridge day summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted bridge day summary should fail validation");

        assert!(matches!(
            error,
            pleiades_jpl::ReferenceSnapshotBridgeDaySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_bridge_day_summary_for_report(),
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(
                &pleiades_jpl::reference_snapshot_bridge_day_summary()
                    .expect("reference snapshot bridge day summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451914_and_2451915_boundary_aliases_match_the_generic_reports() {
        assert_eq!(
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_2451914_major_body_pre_bridge_summary_for_report(),
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_pre_bridge_boundary_summary_for_report()
        );
        assert_eq!(
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_pre_bridge_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451914_major_body_pre_bridge_summary()
                    .expect("reference 2451914 major-body pre-bridge summary should exist")
            ),
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_pre_bridge_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_pre_bridge_boundary_summary()
                    .expect("reference pre-bridge boundary summary should exist")
            )
        );

        assert_eq!(
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_2451914_major_body_bridge_summary_for_report(),
            reference_snapshot_bridge_day_summary_for_report()
        );
        assert_eq!(
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(
                &pleiades_jpl::reference_snapshot_2451914_major_body_bridge_summary()
                    .expect("reference 2451914 major-body bridge summary should exist")
            ),
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_bridge_day_summary_line(
                &pleiades_jpl::reference_snapshot_bridge_day_summary()
                    .expect("reference bridge day summary should exist")
            )
        );

        let summary = pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary()
            .expect("reference 2451915 major-body bridge summary should exist");
        assert_eq!(
            crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_2451915_major_body_bridge_summary_for_report(),
            format!(
                "Reference 2451915 major-body bridge evidence: {} exact samples at {} ({}); 2451915 major-body bridge sample",
                summary.sample_count,
                format_instant(summary.epoch),
                crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&summary.sample_bodies),
            )
        );
        assert_eq!(
            crate::posture::jpl::reference_snapshot::boundaries::era_a::reference_major_body_bridge_summary_line(&summary),
            crate::posture::jpl::reference_snapshot::boundaries::era_a::reference_major_body_bridge_summary_line(
                &pleiades_jpl::reference_snapshot_major_body_bridge_summary()
                    .expect("reference major-body bridge summary should exist")
            )
        );
    }
}
