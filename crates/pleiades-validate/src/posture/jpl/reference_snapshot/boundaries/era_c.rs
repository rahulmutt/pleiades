//! Relocated reference-snapshot boundary renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::boundaries::era_c`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `validate()`/`label()` methods, and all release-gate data; jpl's own
//! rendering stays in place until the Task 14 contract sweep.
//!
//! This file defines **no** free `*_for_report` renderers of its own — every
//! one of this file's structs has its free `*_for_report` wrapper living in
//! `reference_snapshot/core/{general_a,general_b}.rs` (Slice D Task 8,
//! already copied), whose `.validated_summary_line()`/`.validate()` calls are
//! still left on jpl's still-present inherent methods (validate→jpl is
//! allowed and byte-identical; Task 13 repoints them to the free fns below).
//!
//! This file textually defines the rendering (`summary_line`/
//! `validated_summary_line`/`Display`) for exactly 9 evidence structs — 8 of
//! them (`Reference2451545MajorBodyBoundarySummary`,
//! `Reference2451910MajorBodyBoundarySummary`,
//! `Reference2451911MajorBodyBoundarySummary`,
//! `Reference2451912MajorBodyBoundarySummary`,
//! `Reference2451913MajorBodyBoundarySummary`,
//! `Reference2451914MajorBodyBoundarySummary`,
//! `Reference2451915MajorBodyBoundarySummary`,
//! `Reference2451917MajorBodyBridgeSummary`) with their struct/enum also
//! textually defined here, plus one
//! (`Reference2453000MajorBodyBoundarySummary`) whose struct is textually
//! defined in `boundaries/era_b.rs` (Slice D Task 9a — that file's own doc
//! comment already flags this split). Each struct's inherent `summary_line`
//! (9) is re-homed below as `pub(crate) fn <struct_snake>_summary_line`.
//! Every `validated_summary_line` (9) is the simple `{ self.validate()?;
//! Ok(self.summary_line()) }` pattern (verified per struct) — consistent
//! with the family recipe, so no standalone re-home is needed; the pattern
//! reconstructs at each cross-file `*_for_report` call site when Task 13
//! repoints it. The `Display` impls (9 evidence + 9 sibling
//! `*ValidationError`, all pure `f.write_str(&self.summary_line())`
//! forwarders for the evidence structs) are not reproduced as standalone
//! items, consistent with every prior posture module — `Display` on a
//! foreign jpl type cannot be re-implemented here anyway (orphan rule); the
//! free-fn equivalent is the re-homed rendering surface, and
//! `*ValidationError` formatting stays in jpl untouched (it is gate/error
//! logic, not report rendering).
//!
//! This file also textually defines the
//! `Reference2451917MajorBodyBoundarySummary` struct (and its
//! `*ValidationError` enum plus that enum's `Display`), but its inherent
//! rendering (`summary_line`/`validated_summary_line`/`Display`) lives in
//! `boundaries/era_d.rs` (Slice D Task 9c, not yet copied) — nothing in this
//! file renders it, so it is not referenced here at all.
//!
//! The jpl test file (`reference_summary/reference_snapshot/tests.rs`)
//! already had its literal-string assertions for these 9 structs' owning
//! `*_for_report` wrappers copied into `reference_snapshot/core/
//! {general_a,general_b}.rs`'s test modules in Task 8 (testing those files'
//! own `*_for_report` wrappers, which still fall back to jpl's inherent
//! methods). There is no leftover jpl test that names this file's new local
//! free fns directly (they did not exist before this task), so this file's
//! own test module instead pins byte-identity between jpl's still-present
//! inherent `summary_line()` and the free-fn copies below, mirroring Task
//! 2's `posture/jpl/backend.rs` / Task 9a's `boundaries/era_a.rs` golden
//! pattern.

use pleiades_jpl::{
    Reference2451545MajorBodyBoundarySummary, Reference2451910MajorBodyBoundarySummary,
    Reference2451911MajorBodyBoundarySummary, Reference2451912MajorBodyBoundarySummary,
    Reference2451913MajorBodyBoundarySummary, Reference2451914MajorBodyBoundarySummary,
    Reference2451915MajorBodyBoundarySummary, Reference2451917MajorBodyBridgeSummary,
    Reference2453000MajorBodyBoundarySummary,
};

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Compact release-facing summary line for the 2453000.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2453000MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:75). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2453000_major_body_boundary_summary_line(
    s: &Reference2453000MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2453000 major-body boundary evidence: {} exact samples at {} ({}); 2453000.5 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451545.0 major-body boundary
/// (J2000) reference evidence. Verbatim copy of
/// `Reference2451545MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:229). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451545_major_body_boundary_summary_line(
    s: &Reference2451545MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451545 major-body boundary evidence: {} exact samples at {} ({}); J2000 reference sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451910.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451910MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:383). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451910_major_body_boundary_summary_line(
    s: &Reference2451910MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451910 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-01 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451911.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451911MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:537). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451911_major_body_boundary_summary_line(
    s: &Reference2451911MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451911 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-02 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451912.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451912MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:691). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451912_major_body_boundary_summary_line(
    s: &Reference2451912MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451912 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-03 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451913.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451913MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:845). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451913_major_body_boundary_summary_line(
    s: &Reference2451913MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451913 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-04 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451914.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451914MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:999). Reuses the
/// canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451914_major_body_boundary_summary_line(
    s: &Reference2451914MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451914 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-05 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451915.5 major-body boundary
/// reference evidence. Verbatim copy of
/// `Reference2451915MajorBodyBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:1153). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451915_major_body_boundary_summary_line(
    s: &Reference2451915MajorBodyBoundarySummary,
) -> String {
    format!(
        "Reference 2451915 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-06 boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451917.0 major-body bridge
/// reference evidence. Verbatim copy of
/// `Reference2451917MajorBodyBridgeSummary::summary_line`
/// (reference_summary/reference_snapshot/boundaries/era_c.rs:1307). Reuses
/// the canonical `format_bodies` copy in `core::general_a` rather than
/// reproducing it locally.
pub(crate) fn reference_2451917_major_body_bridge_summary_line(
    s: &Reference2451917MajorBodyBridgeSummary,
) -> String {
    format!(
        "Reference 2451917 major-body bridge evidence: {} exact samples at {} ({}); bridge sample across the major-body boundary window",
        s.sample_count,
        format_instant(s.epoch),
        crate::posture::jpl::reference_snapshot::core::general_a::format_bodies(&s.sample_bodies),
    )
}

#[cfg(test)]
mod golden {
    use super::*;

    // Task 14b (contract sweep) deleted these structs' jpl inherent
    // `summary_line` renderers, so the byte-identity `summary.summary_line()`
    // comparisons are gone; the captured literals below are the standing
    // regression guard for the validate copies.

    #[test]
    fn reference_2453000_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2453000_major_body_boundary_summary()
            .expect("reference 2453000 major-body boundary summary should exist");
        assert_eq!(
            reference_2453000_major_body_boundary_summary_line(&summary),
            "Reference 2453000 major-body boundary evidence: 10 exact samples at JD 2453000.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2453000.5 boundary sample"
        );
    }

    #[test]
    fn reference_2451545_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451545_major_body_boundary_summary()
            .expect("reference 2451545 major-body boundary summary should exist");
        assert_eq!(
            reference_2451545_major_body_boundary_summary_line(&summary),
            "Reference 2451545 major-body boundary evidence: 10 exact samples at JD 2451545.0 (TDB) (Jupiter, Mars, Mercury, Moon, Neptune, Pluto, Saturn, Sun, Uranus, Venus); J2000 reference sample"
        );
    }

    #[test]
    fn reference_2451910_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451910_major_body_boundary_summary()
            .expect("reference 2451910 major-body boundary summary should exist");
        assert_eq!(
            reference_2451910_major_body_boundary_summary_line(&summary),
            "Reference 2451910 major-body boundary evidence: 10 exact samples at JD 2451910.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-01 boundary sample"
        );
    }

    #[test]
    fn reference_2451911_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451911_major_body_boundary_summary()
            .expect("reference 2451911 major-body boundary summary should exist");
        assert_eq!(
            reference_2451911_major_body_boundary_summary_line(&summary),
            "Reference 2451911 major-body boundary evidence: 10 exact samples at JD 2451911.5 (TDB) (Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter); 2001-01-02 boundary sample"
        );
    }

    #[test]
    fn reference_2451912_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451912_major_body_boundary_summary()
            .expect("reference 2451912 major-body boundary summary should exist");
        assert_eq!(
            reference_2451912_major_body_boundary_summary_line(&summary),
            "Reference 2451912 major-body boundary evidence: 10 exact samples at JD 2451912.5 (TDB) (Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter); 2001-01-03 boundary sample"
        );
    }

    #[test]
    fn reference_2451913_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451913_major_body_boundary_summary()
            .expect("reference 2451913 major-body boundary summary should exist");
        assert_eq!(
            reference_2451913_major_body_boundary_summary_line(&summary),
            "Reference 2451913 major-body boundary evidence: 10 exact samples at JD 2451913.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-04 boundary sample"
        );
    }

    #[test]
    fn reference_2451914_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451914_major_body_boundary_summary()
            .expect("reference 2451914 major-body boundary summary should exist");
        assert_eq!(
            reference_2451914_major_body_boundary_summary_line(&summary),
            "Reference 2451914 major-body boundary evidence: 10 exact samples at JD 2451914.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-05 boundary sample"
        );
    }

    #[test]
    fn reference_2451915_major_body_boundary_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451915_major_body_boundary_summary()
            .expect("reference 2451915 major-body boundary summary should exist");
        assert_eq!(
            reference_2451915_major_body_boundary_summary_line(&summary),
            "Reference 2451915 major-body boundary evidence: 10 exact samples at JD 2451915.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-06 boundary sample"
        );
    }

    #[test]
    fn reference_2451917_major_body_bridge_summary_line_byte_identical() {
        let summary = pleiades_jpl::reference_snapshot_2451917_major_body_bridge_summary()
            .expect("reference 2451917 major-body bridge summary should exist");
        assert_eq!(
            reference_2451917_major_body_bridge_summary_line(&summary),
            "Reference 2451917 major-body bridge evidence: 10 exact samples at JD 2451917.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); bridge sample across the major-body boundary window"
        );
    }
}
