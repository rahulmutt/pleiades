//! Relocated reference-snapshot boundary renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::boundaries::era_b`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()`/`*_summary()` constructors, `validate()`/`label()` methods,
//! and all release-gate data; jpl's own rendering stays in place until the
//! Task 14 contract sweep.
//!
//! This file defines the `Reference1900SelectedBodyBoundarySummary` evidence
//! struct (inherent `summary_line`/`validated_summary_line`, `Display`) and
//! 2 free `*_for_report` renderers of its own
//! (`reference_snapshot_1900_selected_body_boundary_summary_for_report`,
//! `reference_snapshot_2415020_selected_body_boundary_summary_for_report` —
//! a `#[doc(alias)]` compatibility wrapper reusing the same evidence). Its
//! inherent `summary_line` is re-homed below as
//! `reference_1900_selected_body_boundary_summary_line`; its
//! `validated_summary_line`/`validate()` call sites fold into the `match
//! summary.validate() { Ok(()) => <local render>, ... }` rewrite at each of
//! this file's own 2 renderers, per the family recipe (`validate()` stays on
//! the jpl struct; rendering is local). Its `Display` impl (and its sibling
//! `Reference1900SelectedBodyBoundarySummaryValidationError`'s `Display`)
//! are not reproduced as standalone items, consistent with every prior
//! posture module.
//!
//! This file also textually defines
//! `reference_snapshot_1900_selected_body_boundary_entries`/`_summary_details`
//! (data accessors) and the public `_summary()`/`_summary()` alias
//! constructors — none of those are duplicated into validate; every call to
//! one is qualified `pleiades_jpl::<name>()`, consistent with every prior
//! posture module's treatment of `*_summary()`/`*_details()` constructors
//! (even ones textually defined in the same jpl source file as the renderer
//! being copied).
//!
//! `format_selected_body_boundary_summary_line` (a shared formatting helper
//! for this file's 2 renderers) is copied verbatim below.
//!
//! This file also textually defines the
//! `Reference2453000MajorBodyBoundarySummary` struct, but its inherent
//! rendering (`summary_line`/`validated_summary_line`/`Display`) lives in
//! `boundaries/era_c.rs` (Slice D Task 9b, not yet copied) — nothing in this
//! file renders it, so it is not referenced here at all.

use pleiades_jpl::Reference1900SelectedBodyBoundarySummary;

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Compact release-facing summary line for the 1900-01-01 selected-body
/// boundary reference evidence. Verbatim copy of
/// `Reference1900SelectedBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_b.rs:114). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_1900_selected_body_boundary_summary_line(
    s: &Reference1900SelectedBodyBoundarySummary,
) -> String {
    format!(
        "Reference 1900 selected-body boundary evidence: {} exact samples at {} ({}); 1900-01-01 selected-body boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Returns the release-facing 1900 selected-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_1900_selected_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/boundaries/era_b.rs:218), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local), and the data-constructor
/// call qualified `pleiades_jpl::`.
pub(crate) fn reference_snapshot_1900_selected_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_1900_selected_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_1900_selected_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 1900 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1900 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2415020 selected-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2415020_selected_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/boundaries/era_b.rs:238), with
/// `summary.validated_summary_line()` used only as a validation gate rewired
/// to `summary.validate()` (`validate()` stays on the jpl struct; rendering
/// is local), and the data-constructor call qualified `pleiades_jpl::`.
pub fn reference_snapshot_2415020_selected_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2415020_selected_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_selected_body_boundary_summary_line(
                "2415020",
                summary.sample_count,
                &summary.sample_bodies,
                summary.epoch,
                "1900-01-01",
            ),
            Err(error) => {
                format!("Reference 2415020 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2415020 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Verbatim copy of jpl's `pub(crate)` `format_selected_body_boundary_summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_b.rs:256). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn format_selected_body_boundary_summary_line(
    epoch_label: &str,
    sample_count: usize,
    sample_bodies: &[pleiades_backend::CelestialBody],
    epoch: pleiades_types::Instant,
    sample_label: &str,
) -> String {
    format!(
        "Reference {epoch_label} selected-body boundary evidence: {} exact samples at {} ({}); {sample_label} selected-body boundary sample",
        sample_count,
        format_instant(epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(sample_bodies),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_snapshot_1900_selected_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_1900_selected_body_boundary_summary()
            .expect("reference 1900 selected-body boundary summary should exist");
        assert_eq!(summary.sample_count, 4);
        assert_eq!(summary.sample_bodies.len(), 4);
        assert_eq!(summary.epoch.julian_day.days(), 2_415_020.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[1],
            pleiades_backend::CelestialBody::Moon
        );
        assert_eq!(
            summary.sample_bodies[2],
            pleiades_backend::CelestialBody::Mercury
        );
        assert_eq!(
            summary.sample_bodies[3],
            pleiades_backend::CelestialBody::Venus
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_1900_selected_body_boundary_summary_line(&summary),
            "Reference 1900 selected-body boundary evidence: 4 exact samples at JD 2415020.5 (TDB) (Sun, Moon, Mercury, Venus); 1900-01-01 selected-body boundary sample"
        );
        assert_eq!(
            reference_snapshot_1900_selected_body_boundary_summary_for_report(),
            reference_1900_selected_body_boundary_summary_line(&summary)
        );
        assert_eq!(
            reference_snapshot_2415020_selected_body_boundary_summary_for_report(),
            "Reference 2415020 selected-body boundary evidence: 4 exact samples at JD 2415020.5 (TDB) (Sun, Moon, Mercury, Venus); 1900-01-01 selected-body boundary sample"
        );
    }
}
