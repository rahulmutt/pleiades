//! Relocated reference-snapshot boundary renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::boundaries::era_a`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()` constructors, `validate()`/`label()` methods, and all
//! release-gate data; jpl's own rendering stays in place until the Task 14
//! contract sweep.
//!
//! This file defines exactly 5 evidence structs
//! (`ReferenceLunarBoundarySummary`, `ReferenceHighCurvatureSummary`,
//! `ReferenceMajorBodyBoundarySummary`, `ReferenceMajorBodyBridgeSummary`,
//! `ReferenceMarsJupiterBoundarySummary`) and defines **no** free
//! `*_for_report` renderers of its own — each struct's free `*_for_report`
//! wrapper lives in `reference_snapshot/core/general_a.rs` (Slice D Task 8,
//! already copied), whose `.validated_summary_line()` calls are still left
//! on jpl's still-present inherent methods (validate→jpl is allowed and
//! byte-identical; Task 13 repoints them to call the free fns below). Each
//! struct's inherent `summary_line` (5) is re-homed below as `pub(crate) fn
//! <struct_snake>_summary_line`. None has a local caller of its inherent
//! `validated_summary_line` (no free `*_for_report` exists in this file to
//! fold it into), so — consistent with `general_b.rs`'s orphaned
//! `validated_summary_line` methods — no standalone free fn is added for it
//! here; `validate()` stays on the jpl struct, and Task 13 folds it into a
//! `match summary.validate() { Ok(()) => <local render>, ... }` rewrite at
//! each (currently jpl-side) call site. The `Display` impls (5 evidence +
//! 5 sibling `*ValidationError`, all pure `f.write_str(&self.summary_line())`
//! forwarders for the evidence structs) are not reproduced as standalone
//! items, consistent with every prior posture module — `Display` on a
//! foreign jpl type cannot be re-implemented here anyway (orphan rule); the
//! free-fn equivalent is the re-homed rendering surface, and
//! `*ValidationError` formatting stays in jpl untouched (it is gate/error
//! logic, not report rendering).
//!
//! The jpl test file (`reference_summary/reference_snapshot/tests.rs`)
//! already had its literal-string assertions for these 5 structs' rendering
//! copied into `reference_snapshot/core/general_a.rs`'s test module in Task
//! 8 (testing that file's own `*_for_report` wrappers, which still fall
//! back to jpl's inherent methods). There is no leftover jpl test that names
//! this file's new local free fns directly (they did not exist before this
//! task), so this file's own test module instead pins byte-identity between
//! jpl's still-present inherent `summary_line()` and the free-fn copies
//! below, mirroring Task 2's `posture/jpl/backend.rs` golden pattern.

use pleiades_jpl::{
    ReferenceHighCurvatureSummary, ReferenceLunarBoundarySummary,
    ReferenceMajorBodyBoundarySummary, ReferenceMajorBodyBridgeSummary,
    ReferenceMarsJupiterBoundarySummary,
};

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Compact release-facing summary line for the Moon high-curvature reference
/// window. Verbatim copy of `ReferenceLunarBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_a.rs:27).
pub(crate) fn reference_lunar_boundary_summary_line(s: &ReferenceLunarBoundarySummary) -> String {
    format!(
        "Reference lunar boundary evidence: {} exact Moon samples at {}..{}; high-curvature interpolation window",
        s.sample_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
    )
}

/// Compact release-facing summary line for the major-body high-curvature
/// reference window. Verbatim copy of
/// `ReferenceHighCurvatureSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_a.rs:135). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_high_curvature_summary_line(s: &ReferenceHighCurvatureSummary) -> String {
    format!(
        "Reference major-body high-curvature evidence: {} exact samples across {} bodies and {} epochs ({}..{}); bodies: {}; high-curvature interpolation window",
        s.sample_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.bodies),
    )
}

/// Compact release-facing summary line for the major-body boundary-day
/// reference evidence. Verbatim copy of
/// `ReferenceMajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_a.rs:313). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_major_body_boundary_summary_line(
    s: &ReferenceMajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference major-body boundary evidence: {} exact samples at {} ({}); 2001-01-08 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the major-body bridge-day
/// reference evidence. Verbatim copy of
/// `ReferenceMajorBodyBridgeSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_a.rs:467). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_major_body_bridge_summary_line(
    s: &ReferenceMajorBodyBridgeSummary,
) -> String {
    format!(
        "Reference major-body bridge evidence: {} exact samples at {} ({}); bridge sample across the major-body boundary window",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the Mars/Jupiter boundary
/// reference evidence. Verbatim copy of
/// `ReferenceMarsJupiterBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_a.rs:621). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_mars_jupiter_boundary_summary_line(
    s: &ReferenceMarsJupiterBoundarySummary,
) -> String {
    format!(
        "Reference Mars/Jupiter boundary evidence: {} exact samples at {} ({}); 2001-01-09 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

#[cfg(test)]
mod golden {
    use super::*;

    // jpl's inherent renderers are still present through the contract sweep
    // (Task 14); these fail closed on any drift in the validate copies. Task
    // 14 replaces the `summary.summary_line()` comparisons with the captured
    // literals when the jpl methods are deleted.

    #[test]
    fn reference_lunar_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_lunar_boundary_summary()
            .expect("reference lunar boundary summary should exist");
        assert_eq!(
            reference_lunar_boundary_summary_line(&summary),
            summary.summary_line()
        );
        assert_eq!(
            reference_lunar_boundary_summary_line(&summary),
            "Reference lunar boundary evidence: 2 exact Moon samples at JD 2451911.5 (TDB)..JD 2451912.5 (TDB); high-curvature interpolation window"
        );
    }

    #[test]
    fn reference_high_curvature_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_high_curvature_summary()
            .expect("reference high-curvature summary should exist");
        assert_eq!(
            reference_high_curvature_summary_line(&summary),
            summary.summary_line()
        );
        assert_eq!(
            reference_high_curvature_summary_line(&summary),
            "Reference major-body high-curvature evidence: 50 exact samples across 10 bodies and 5 epochs (JD 2451911.5 (TDB)..JD 2451916.5 (TDB)); bodies: Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter; high-curvature interpolation window"
        );
    }

    #[test]
    fn reference_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist");
        assert_eq!(
            reference_major_body_boundary_summary_line(&summary),
            summary.summary_line()
        );
        assert_eq!(
            reference_major_body_boundary_summary_line(&summary),
            "Reference major-body boundary evidence: 10 exact samples at JD 2451917.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-08 boundary sample"
        );
    }

    #[test]
    fn reference_major_body_bridge_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_major_body_bridge_summary()
            .expect("reference major-body bridge summary should exist");
        assert_eq!(
            reference_major_body_bridge_summary_line(&summary),
            summary.summary_line()
        );
        assert_eq!(
            reference_major_body_bridge_summary_line(&summary),
            "Reference major-body bridge evidence: 10 exact samples at JD 2451915.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); bridge sample across the major-body boundary window"
        );
    }

    #[test]
    fn reference_mars_jupiter_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_mars_jupiter_boundary_summary()
            .expect("reference Mars/Jupiter boundary summary should exist");
        assert_eq!(
            reference_mars_jupiter_boundary_summary_line(&summary),
            summary.summary_line()
        );
        assert_eq!(
            reference_mars_jupiter_boundary_summary_line(&summary),
            "Reference Mars/Jupiter boundary evidence: 16 exact samples at JD 2451918.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2001-01-09 boundary sample"
        );
    }
}
