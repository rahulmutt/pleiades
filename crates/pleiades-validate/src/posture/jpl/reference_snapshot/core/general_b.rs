//! Relocated reference-snapshot core renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::core::general_b`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()`/`*_summary()` constructors, `validate()`/`label()` methods,
//! and all release-gate data; jpl's own rendering stays in place until the
//! Task 14 contract sweep.
//!
//! This file defines exactly 7 evidence structs:
//! `ReferenceSnapshotSparseBoundarySummary`, `ReferenceSnapshotPreBridgeBoundarySummary`,
//! `ReferenceSnapshotBridgeDaySummary`, `ReferenceSnapshotDenseBoundarySummary`,
//! `ReferenceSnapshotSourceSummary`, `ReferenceSnapshotSourceWindow`, and
//! `ReferenceSnapshotSourceWindowSummary`. Their inherent `summary_line`
//! (7) is re-homed below as `pub(crate) fn <struct_snake>_summary_line`;
//! their inherent `validated_summary_line` (6 — `ReferenceSnapshotBridgeDaySummary`
//! has no local caller of its `validated_summary_line`, see the note on
//! `reference_snapshot_bridge_day_summary_line` below) fold into the `match
//! summary.validate() { Ok(()) => <local render>, ... }` rewrite at each
//! call site, per the family recipe (`validate()` stays on the jpl struct;
//! rendering is local). Their `Display` impls (7, all pure
//! `f.write_str(&self.summary_line())` forwarders) are not reproduced as
//! standalone items, consistent with every prior posture module.
//!
//! Several of this file's 18 free `*_for_report` renderers operate on
//! `Option<Struct>` values whose *own* struct definitions and inherent
//! rendering live in `reference_summary/reference_snapshot/boundaries/
//! {era_a,era_c,era_d}.rs` (Slice D Task 9, already copied) even though the
//! *data accessor* function that produces the `Option<Struct>` is textually
//! defined in this jpl source file — those `.validated_summary_line()`/
//! `.validate()` calls are rewired (Slice D Task 13b) to `match
//! summary.validate() { Ok(()) => <local render>, ... }`, calling the
//! `boundaries/era_d.rs` free renderers directly (`validate()` stays on the
//! jpl struct; rendering is local). Every `*_summary()`/`*_details()` data
//! accessor (even the ones textually defined in this same jpl source file)
//! is never duplicated into validate; every call to one is qualified
//! `pleiades_jpl::<name>()`. Two renderers
//! (`reference_snapshot_2451914_major_body_bridge_summary_for_report`,
//! `reference_snapshot_2451914_major_body_pre_bridge_summary_for_report`'s
//! sibling `reference_snapshot_2451914_major_body_bridge_summary_for_report`)
//! delegate to a free renderer defined in `boundaries/era_d.rs`
//! (`reference_snapshot_bridge_day_summary_for_report`, Slice D Task 9,
//! already copied) — rewired to the local copy (Slice D Task 13b).
//!
//! The manifest renderer (`reference_snapshot_manifest_summary_for_report`)
//! delegates its final line to Task 2's
//! `crate::posture::jpl::backend::snapshot_manifest_summary_line`, matching
//! the sibling `comparison.rs`/`holdout.rs` manifest renderers; its
//! `include_str!` reaches one directory over to jpl's checked-in copy of the
//! same CSV (established precedent — `comparison.rs:284`, `holdout.rs:489`).
//!
//! `format_bodies`/`format_instant` are consumed from `super::general_a`
//! where general_a is the canonical home (Task 8b); `format_instant` itself
//! is reproduced locally (jpl's private `lib.rs:66`, per-module duplicate
//! accepted, consistent with every sibling posture module) since general_a's
//! copy is not `pub(crate)` (module-private in that file).
//!
//! `reference_snapshot_source_checksum` and 6 of the `REFERENCE_SNAPSHOT_*`
//! label constants (`EVIDENCE_CLASS`, `SOURCE_EXPECTED`, `COVERAGE_FALLBACK`,
//! `REDISTRIBUTION_FALLBACK`, `FRAME_TREATMENT`, `TIME_SCALE`, `COLUMNS`)
//! were promoted `pub` in jpl (Slice D expand 8c) — they are data/label
//! accessors (not rendering) needed by this file's copied
//! `reference_snapshot_source_summary_validation_reports_blank_fields` test,
//! which constructs `ReferenceSnapshotSourceSummary` literals directly.
//! `REFERENCE_SNAPSHOT_SOURCE_FALLBACK` was not promoted (no copied consumer
//! — only used inside jpl's own `reference_snapshot_source_summary()`
//! constructor, which stays in jpl).

use pleiades_jpl::{
    ReferenceSnapshotBridgeDaySummary, ReferenceSnapshotDenseBoundarySummary,
    ReferenceSnapshotPreBridgeBoundarySummary, ReferenceSnapshotSourceSummary,
    ReferenceSnapshotSourceWindow, ReferenceSnapshotSourceWindowSummary,
    ReferenceSnapshotSparseBoundarySummary,
};

// Slice D Task 13b: reach `boundaries/era_d.rs`'s local renderers for the
// evidence structs this file's `*_for_report` wrappers unwrap, without a
// `pleiades_jpl::` detour (those structs/rendering are textually defined in
// `era_d.rs`, already copied by Task 9).
use super::super::boundaries::era_d::{
    reference_2451916_major_body_interior_summary_line,
    reference_2451917_major_body_boundary_summary_line,
    reference_2451919_major_body_boundary_summary_line,
    reference_2451920_major_body_interior_summary_line,
    reference_high_curvature_window_summary_line,
    reference_major_body_boundary_window_summary_line,
    reference_snapshot_bridge_day_summary_for_report,
};

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Compact release-facing summary line for the sparse asteroid-only boundary
/// day and its remaining coverage gap. Verbatim copy of
/// `ReferenceSnapshotSparseBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/core/general_b.rs:502).
pub(crate) fn reference_snapshot_sparse_boundary_summary_line(
    s: &ReferenceSnapshotSparseBoundarySummary,
) -> String {
    if s.missing_bodies.is_empty() {
        format!(
            "Reference snapshot boundary day: {} exact samples at {} ({})",
            s.sample_count,
            format_instant(s.epoch),
            super::general_a::format_bodies(&s.sample_bodies),
        )
    } else {
        format!(
            "Reference snapshot boundary day: {} exact samples at {} ({}); sparse boundary day; missing bodies: {}",
            s.sample_count,
            format_instant(s.epoch),
            super::general_a::format_bodies(&s.sample_bodies),
            super::general_a::format_bodies(&s.missing_bodies),
        )
    }
}

/// Compact release-facing summary line for the pre-bridge 2451914.5 boundary
/// day. Verbatim copy of `ReferenceSnapshotPreBridgeBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/core/general_b.rs:738).
pub(crate) fn reference_snapshot_pre_bridge_boundary_summary_line(
    s: &ReferenceSnapshotPreBridgeBoundarySummary,
) -> String {
    format!(
        "Reference snapshot pre-bridge boundary day: {} exact samples at {} ({}); pre-bridge boundary day",
        s.sample_count,
        format_instant(s.epoch),
        super::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the 2451914.0 bridge day.
/// Verbatim copy of `ReferenceSnapshotBridgeDaySummary::summary_line`
/// (reference_summary/reference_snapshot/core/general_b.rs:963). No renderer
/// in this file calls it locally — the only local user,
/// `reference_snapshot_2451914_major_body_bridge_summary_for_report`,
/// delegates wholesale to `reference_snapshot_bridge_day_summary_for_report`
/// (`boundaries/era_d.rs`, Slice D Task 9, already copied), which itself
/// calls this function directly (Slice D Task 13b) — this function was
/// re-homed here per the family recipe's "re-home all inherent rendering of
/// this file's evidence structs" requirement ahead of that call site landing.
pub(crate) fn reference_snapshot_bridge_day_summary_line(
    s: &ReferenceSnapshotBridgeDaySummary,
) -> String {
    format!(
        "Reference snapshot bridge day: {} exact samples at {} ({}); bridge sample across the reference boundary window",
        s.sample_count,
        format_instant(s.epoch),
        super::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the dense 2451916.5 boundary day.
/// Verbatim copy of `ReferenceSnapshotDenseBoundarySummary::summary_line`
/// (reference_summary/reference_snapshot/core/general_b.rs:1152).
pub(crate) fn reference_snapshot_dense_boundary_summary_line(
    s: &ReferenceSnapshotDenseBoundarySummary,
) -> String {
    format!(
        "Reference snapshot dense boundary day: {} exact samples at {} ({}); dense boundary day",
        s.sample_count,
        format_instant(s.epoch),
        super::general_a::format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the checked-in reference snapshot
/// source-material provenance. Verbatim copy of
/// `ReferenceSnapshotSourceSummary::summary_line`
/// (reference_summary/reference_snapshot/core/general_b.rs:1627).
pub(crate) fn reference_snapshot_source_summary_line(s: &ReferenceSnapshotSourceSummary) -> String {
    format!(
        "Reference snapshot source: {}; evidence class={}; coverage={}; columns={}; redistribution={}; checksum=0x{:016x}; {}; time scale={}; TDB reference epoch {}",
        s.source,
        s.evidence_class,
        s.coverage,
        s.columns,
        s.redistribution,
        s.checksum,
        s.frame_treatment,
        s.time_scale,
        format_instant(s.reference_epoch),
    )
}

/// Compact release-facing body-window summary line. Verbatim copy of
/// `ReferenceSnapshotSourceWindow::summary_line`
/// (reference_summary/reference_snapshot/core/general_b.rs:1781).
pub(crate) fn reference_snapshot_source_window_line(s: &ReferenceSnapshotSourceWindow) -> String {
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

/// Compact release-facing summary line for the checked-in reference snapshot
/// source coverage. Verbatim copy of
/// `ReferenceSnapshotSourceWindowSummary::summary_line`
/// (reference_summary/reference_snapshot/core/general_b.rs:1824), with the
/// nested `ReferenceSnapshotSourceWindow::summary_line` call rewired to the
/// local `reference_snapshot_source_window_line` (same-file struct, per the
/// recipe).
pub(crate) fn reference_snapshot_source_window_summary_line(
    s: &ReferenceSnapshotSourceWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(reference_snapshot_source_window_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Reference snapshot source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Verbatim copy of jpl's `pub(crate)`
/// `format_validated_reference_snapshot_source_window_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:2023), with
/// `summary.validated_summary_line()` rewired to `match summary.validate() {
/// Ok(()) => <local render>, ... }` (`validate()` stays on the jpl struct;
/// rendering is local).
pub(crate) fn format_validated_reference_snapshot_source_window_summary_for_report(
    summary: &ReferenceSnapshotSourceWindowSummary,
) -> String {
    match summary.validate() {
        Ok(()) => reference_snapshot_source_window_summary_line(summary),
        Err(error) => format!("Reference snapshot source windows: unavailable ({error})"),
    }
}

/// Returns the release-facing 2451917 major-body boundary summary string.
/// Verbatim copy of jpl's `reference_snapshot_2451917_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:62). The
/// `Reference2451917MajorBodyBoundarySummary` struct and its rendering live
/// in `boundaries/era_d.rs` (Slice D Task 9, already copied);
/// `summary.validated_summary_line()` is rewired to `match summary.validate()
/// { Ok(()) => <local render>, ... }`, calling the local
/// `reference_2451917_major_body_boundary_summary_line` (Slice D Task 13b;
/// `validate()` stays on the jpl struct, rendering is local).
pub(crate) fn reference_snapshot_2451917_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451917_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451917_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451917 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451917 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451919 major-body boundary summary string.
/// Verbatim copy of jpl's `reference_snapshot_2451919_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:123). The
/// `Reference2451919MajorBodyBoundarySummary` struct and its rendering live
/// in `boundaries/era_d.rs` (Slice D Task 9, already copied); rewired the
/// same way as above, calling the local
/// `reference_2451919_major_body_boundary_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451919_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451919_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451919_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451919 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451919 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451916 major-body interior summary string.
/// Verbatim copy of jpl's `reference_snapshot_2451916_major_body_interior_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:184). The
/// `Reference2451916MajorBodyInteriorSummary` struct and its rendering live
/// in `boundaries/era_d.rs` (Slice D Task 9, already copied); rewired the
/// same way as above, calling the local
/// `reference_2451916_major_body_interior_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451916_major_body_interior_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451916_major_body_interior_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451916_major_body_interior_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451916 major-body interior evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451916 major-body interior evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451920 major-body interior summary string.
/// Verbatim copy of jpl's `reference_snapshot_2451920_major_body_interior_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:245). The
/// `Reference2451920MajorBodyInteriorSummary` struct and its rendering live
/// in `boundaries/era_d.rs` (Slice D Task 9, already copied); rewired the
/// same way as above, calling the local
/// `reference_2451920_major_body_interior_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451920_major_body_interior_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451920_major_body_interior_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451920_major_body_interior_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451920 major-body interior evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451920 major-body interior evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing major-body boundary-day window summary string.
/// Verbatim copy of jpl's `reference_snapshot_major_body_boundary_window_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:344). The
/// `ReferenceMajorBodyBoundaryWindowSummary` struct and its rendering live in
/// `boundaries/era_d.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_major_body_boundary_window_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_major_body_boundary_window_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_major_body_boundary_window_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_major_body_boundary_window_summary_line(&summary),
            Err(error) => {
                format!("Reference major-body boundary windows: unavailable ({error})")
            }
        },
        None => "Reference major-body boundary windows: unavailable".to_string(),
    }
}

/// Returns the release-facing boundary day summary string. Verbatim copy of
/// jpl's `reference_snapshot_sparse_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:654), with
/// `summary.validated_summary_line()` rewired to `match summary.validate() {
/// Ok(()) => <local render>, ... }`.
pub(crate) fn reference_snapshot_sparse_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_sparse_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_sparse_boundary_summary_line(&summary),
            Err(error) => format!("Reference snapshot boundary day: unavailable ({error})"),
        },
        None => "Reference snapshot boundary day: unavailable".to_string(),
    }
}

/// Returns the release-facing pre-bridge boundary day summary string.
/// Verbatim copy of jpl's `reference_snapshot_pre_bridge_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:864), with
/// `summary.validated_summary_line()` rewired to `match summary.validate() {
/// Ok(()) => <local render>, ... }`.
pub(crate) fn reference_snapshot_pre_bridge_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_pre_bridge_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_pre_bridge_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference snapshot pre-bridge boundary day: unavailable ({error})")
            }
        },
        None => "Reference snapshot pre-bridge boundary day: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451914 major-body pre-bridge boundary summary
/// string. Verbatim copy of jpl's
/// `reference_snapshot_2451914_major_body_pre_bridge_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:883). Same-file
/// nested call, kept local.
pub(crate) fn reference_snapshot_2451914_major_body_pre_bridge_summary_for_report() -> String {
    reference_snapshot_pre_bridge_boundary_summary_for_report()
}

/// Returns the release-facing 2451914 major-body bridge summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451914_major_body_bridge_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:1050). It
/// delegates to `reference_snapshot_bridge_day_summary_for_report`, a free
/// renderer defined in `boundaries/era_d.rs` (Slice D Task 9, already
/// copied) — rewired to the local copy (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451914_major_body_bridge_summary_for_report() -> String {
    reference_snapshot_bridge_day_summary_for_report()
}

/// Returns the release-facing 2451915 major-body bridge summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451915_major_body_bridge_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:1061), reading
/// the `ReferenceMajorBodyBridgeSummary` (`boundaries/era_a.rs`, Task 9, not
/// yet copied) public fields directly rather than calling its rendering.
pub(crate) fn reference_snapshot_2451915_major_body_bridge_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format!(
                "Reference 2451915 major-body bridge evidence: {} exact samples at {} ({}); 2451915 major-body bridge sample",
                summary.sample_count,
                format_instant(summary.epoch),
                super::general_a::format_bodies(&summary.sample_bodies),
            ),
            Err(error) => {
                format!("Reference 2451915 major-body bridge evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451915 major-body bridge evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing dense boundary day summary string. Verbatim
/// copy of jpl's `reference_snapshot_dense_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:1276), with
/// `summary.validated_summary_line()` rewired to `match summary.validate() {
/// Ok(()) => <local render>, ... }`.
pub(crate) fn reference_snapshot_dense_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_dense_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_dense_boundary_summary_line(&summary),
            Err(error) => format!("Reference snapshot dense boundary day: unavailable ({error})"),
        },
        None => "Reference snapshot dense boundary day: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451916.5 dense boundary day summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451916_major_body_dense_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:1295), reading the
/// struct's public fields directly rather than calling its rendering.
pub(crate) fn reference_snapshot_2451916_major_body_dense_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451916_major_body_dense_boundary_summary() {
        Some(summary) => format!(
            "Reference 2451916 major-body dense boundary evidence: {} exact samples at {} ({}); dense boundary day",
            summary.sample_count,
            format_instant(summary.epoch),
            super::general_a::format_bodies(&summary.sample_bodies),
        ),
        None => "Reference 2451916 major-body dense boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451916 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451916_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:1314), reading the
/// struct's public fields directly rather than calling its rendering.
pub(crate) fn reference_snapshot_2451916_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451916_major_body_boundary_summary() {
        Some(summary) => format!(
            "Reference 2451916 major-body boundary evidence: {} exact samples at {} ({}); dense boundary day",
            summary.sample_count,
            format_instant(summary.epoch),
            super::general_a::format_bodies(&summary.sample_bodies),
        ),
        None => "Reference 2451916 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing major-body high-curvature window summary
/// string. Verbatim copy of jpl's
/// `reference_snapshot_high_curvature_window_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:1416). The
/// `ReferenceHighCurvatureWindowSummary` struct and its rendering live in
/// `boundaries/era_d.rs` (Slice D Task 9, already copied);
/// `summary.validated_summary_line()` is rewired to `match summary.validate()
/// { Ok(()) => <local render>, ... }`, calling the local
/// `reference_high_curvature_window_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_high_curvature_window_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_high_curvature_window_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_high_curvature_window_summary_line(&summary),
            Err(error) => {
                format!("Reference major-body high-curvature windows: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature windows: unavailable".to_string(),
    }
}

/// Returns the source-material summary for the checked-in reference
/// snapshot. Verbatim copy of jpl's
/// `reference_snapshot_source_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:1752), with
/// `summary.validated_summary_line()` rewired to `match summary.validate() {
/// Ok(()) => <local render>, ... }`.
pub(crate) fn reference_snapshot_source_summary_for_report() -> String {
    if let Err(error) = pleiades_jpl::reference_snapshot_manifest().validate() {
        return format!("Reference snapshot source: unavailable ({error})");
    }

    let summary = pleiades_jpl::reference_snapshot_source_summary();
    match summary.validate() {
        Ok(()) => reference_snapshot_source_summary_line(&summary),
        Err(error) => format!("Reference snapshot source: unavailable ({error})"),
    }
}

/// Returns the body-window summary for the checked-in reference snapshot.
/// Verbatim copy of jpl's `reference_snapshot_source_window_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:2033).
pub(crate) fn reference_snapshot_source_window_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_source_window_summary() {
        Some(summary) => {
            format_validated_reference_snapshot_source_window_summary_for_report(&summary)
        }
        None => "Reference snapshot source windows: unavailable".to_string(),
    }
}

/// Returns the validated body-window summary for the checked-in reference
/// snapshot. Verbatim copy of jpl's
/// `validated_reference_snapshot_source_window_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:2043), with
/// `summary.validated_summary_line()` rewired to `summary.validate()` +
/// local render.
pub(crate) fn validated_reference_snapshot_source_window_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::reference_snapshot_source_window_summary()
        .ok_or_else(|| "reference snapshot source windows unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(reference_snapshot_source_window_summary_line(&summary))
}

/// Returns the manifest summary for the checked-in reference snapshot.
/// Verbatim copy of jpl's `reference_snapshot_manifest_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_b.rs:2062), with the
/// manifest header/footprint gates called cross-crate (both already `pub`)
/// and the final manifest line delegated to Task 2's
/// `crate::posture::jpl::backend::snapshot_manifest_summary_line`. The
/// `manifest_text` `include_str!` reaches one directory over to jpl's
/// checked-in copy of the same CSV (established precedent —
/// `comparison.rs:284`, `holdout.rs:489`); the bytes read are identical
/// either way.
pub(crate) fn reference_snapshot_manifest_summary_for_report() -> String {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../pleiades-jpl/data/reference_snapshot.csv"
    ));
    if let Err(error) = pleiades_jpl::validate_snapshot_manifest_header_structure(
        manifest_text,
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.",
        Some("repository-checked regression fixtures, not a broad public corpus."),
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        return format!("Reference snapshot manifest: unavailable ({error})");
    }

    let summary = pleiades_jpl::reference_snapshot_manifest_summary();
    match summary.validate_with_expected_metadata(
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        Ok(()) => match pleiades_jpl::validate_snapshot_manifest_footprint(
            "reference snapshot",
            pleiades_jpl::snapshot_entries(),
            277,
            16,
            23,
        ) {
            Ok(()) => crate::posture::jpl::backend::snapshot_manifest_summary_line(&summary),
            Err(error) => format!("Reference snapshot manifest: unavailable ({error})"),
        },
        Err(error) => format!("Reference snapshot manifest: unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_jpl::{
        Reference2451916MajorBodyInteriorSummaryValidationError,
        Reference2451917MajorBodyBoundarySummaryValidationError,
        Reference2451919MajorBodyBoundarySummaryValidationError,
        Reference2451920MajorBodyInteriorSummaryValidationError,
        ReferenceHighCurvatureWindowSummaryValidationError,
        ReferenceSnapshotDenseBoundarySummaryValidationError,
        ReferenceSnapshotSourceSummaryValidationError,
        ReferenceSnapshotSourceWindowSummaryValidationError,
        ReferenceSnapshotSparseBoundarySummaryValidationError,
        SnapshotManifestSummaryValidationError,
    };

    #[test]
    fn reference_snapshot_pre_bridge_boundary_summary_reports_the_pre_bridge_day() {
        let summary = pleiades_jpl::reference_snapshot_pre_bridge_boundary_summary()
            .expect("reference snapshot pre-bridge boundary summary should exist");
        assert_eq!(summary.sample_count, 15);
        assert_eq!(summary.sample_bodies.len(), 15);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_914.5);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference snapshot pre-bridge boundary day: 15 exact samples at JD 2451914.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); pre-bridge boundary day"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_pre_bridge_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_sparse_boundary_summary_reports_the_asteroid_only_day() {
        let summary = pleiades_jpl::reference_snapshot_sparse_boundary_summary()
            .expect("reference snapshot sparse boundary summary should exist");
        assert_eq!(summary.sample_count, 16);
        assert_eq!(summary.sample_bodies.len(), 16);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_915.5);
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
        assert_eq!(
            summary.sample_bodies[4],
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(
            summary.sample_bodies[5],
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(
            summary.sample_bodies[6],
            pleiades_backend::CelestialBody::Saturn
        );
        assert_eq!(
            summary.sample_bodies[7],
            pleiades_backend::CelestialBody::Uranus
        );
        assert_eq!(
            summary.sample_bodies[8],
            pleiades_backend::CelestialBody::Neptune
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(
            summary.sample_bodies[10],
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.sample_bodies[11],
            pleiades_backend::CelestialBody::Pallas
        );
        assert_eq!(
            summary.sample_bodies[12],
            pleiades_backend::CelestialBody::Juno
        );
        assert_eq!(
            summary.sample_bodies[13],
            pleiades_backend::CelestialBody::Vesta
        );
        assert_eq!(
            summary.sample_bodies[14],
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid", "433-Eros"
            ))
        );
        assert_eq!(
            summary.sample_bodies[15],
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid",
                "99942-Apophis"
            ))
        );
        assert!(summary.missing_bodies.is_empty());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference snapshot boundary day: 16 exact samples at JD 2451915.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_sparse_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_sparse_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_sparse_boundary_summary()
            .expect("reference snapshot sparse boundary summary should exist");
        summary.sample_bodies.swap(0, 1);

        let error = summary
            .validate()
            .expect_err("drifted sparse boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceSnapshotSparseBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                ..
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_sparse_boundary_summary_for_report(),
            pleiades_jpl::reference_snapshot_sparse_boundary_summary()
                .expect("reference snapshot sparse boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_sparse_boundary_summary_validation_rejects_missing_body_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_sparse_boundary_summary()
            .expect("reference snapshot boundary day summary should exist");
        summary
            .missing_bodies
            .push(pleiades_backend::CelestialBody::Mercury);

        let error = summary
            .validate()
            .expect_err("drifted boundary day summary should fail validation");

        assert!(error
            .to_string()
            .contains("reference snapshot boundary day"));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn reference_snapshot_dense_boundary_summary_reports_the_dense_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_dense_boundary_summary()
            .expect("reference snapshot dense boundary summary should exist");
        assert_eq!(summary.sample_count, 15);
        assert_eq!(summary.sample_bodies.len(), 15);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_916.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[14],
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid", "433-Eros"
            ))
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference snapshot dense boundary day: 15 exact samples at JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_dense_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_dense_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_dense_boundary_summary()
            .expect("reference snapshot dense boundary summary should exist");
        summary.sample_bodies.swap(0, 1);

        let error = summary
            .validate()
            .expect_err("drifted dense boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceSnapshotDenseBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                ..
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_dense_boundary_summary_for_report(),
            pleiades_jpl::reference_snapshot_dense_boundary_summary()
                .expect("reference snapshot dense boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2451917_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451917_major_body_boundary_summary()
            .expect("reference 2451917 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_917.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference 2451917 major-body boundary evidence: 10 exact samples at JD 2451917.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-08 boundary sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_2451917_major_body_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2451917_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451917_major_body_boundary_summary()
            .expect("reference 2451917 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451917 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451917MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_2451917_major_body_boundary_summary_for_report(),
            pleiades_jpl::reference_snapshot_2451917_major_body_boundary_summary()
                .expect("reference 2451917 major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2451916_major_body_interior_summary_reports_the_interior_day() {
        let summary = pleiades_jpl::reference_snapshot_2451916_major_body_interior_summary()
            .expect("reference 2451916 major-body interior summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_916.0);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference 2451916 major-body interior evidence: 10 exact samples at JD 2451916.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-06 interior reference sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_2451916_major_body_interior_summary_for_report(),
            summary.summary_line()
        );
        assert!(crate::posture::jpl::reference_snapshot_summary_for_report()
            .contains(summary.summary_line().as_str()));
    }

    #[test]
    fn reference_snapshot_2451916_major_body_boundary_summary_aliases_the_dense_boundary_day() {
        let dense_summary =
            pleiades_jpl::reference_snapshot_2451916_major_body_dense_boundary_summary()
                .expect("reference 2451916 major-body dense boundary summary should exist");
        let boundary_summary =
            pleiades_jpl::reference_snapshot_2451916_major_body_boundary_summary()
                .expect("reference 2451916 major-body boundary summary should exist");
        assert_eq!(boundary_summary, dense_summary);
        assert_eq!(boundary_summary.validate(), Ok(()));
        assert_eq!(
            boundary_summary.validated_summary_line(),
            Ok(boundary_summary.summary_line())
        );
        assert_eq!(
            boundary_summary.summary_line(),
            dense_summary.summary_line()
        );
        assert_eq!(
            reference_snapshot_2451916_major_body_dense_boundary_summary_for_report(),
            "Reference 2451916 major-body dense boundary evidence: 15 exact samples at JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        );
        assert_eq!(
            reference_snapshot_2451916_major_body_boundary_summary_for_report(),
            "Reference 2451916 major-body boundary evidence: 15 exact samples at JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        );
    }

    #[test]
    fn reference_snapshot_2451916_major_body_interior_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451916_major_body_interior_summary()
            .expect("reference 2451916 major-body interior summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451916 major-body interior summary should fail validation");

        assert!(matches!(
            error,
            Reference2451916MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_2451916_major_body_interior_summary_for_report(),
            pleiades_jpl::reference_snapshot_2451916_major_body_interior_summary()
                .expect("reference 2451916 major-body interior summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2451919_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451919_major_body_boundary_summary()
            .expect("reference 2451919 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_919.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference 2451919 major-body boundary evidence: 10 exact samples at JD 2451919.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-10 boundary sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_2451919_major_body_boundary_summary_for_report(),
            summary.summary_line()
        );
        assert!(crate::posture::jpl::reference_snapshot_summary_for_report()
            .contains(summary.summary_line().as_str()));
    }

    #[test]
    fn reference_snapshot_2451919_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451919_major_body_boundary_summary()
            .expect("reference 2451919 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451919 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451919MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_2451919_major_body_boundary_summary_for_report(),
            pleiades_jpl::reference_snapshot_2451919_major_body_boundary_summary()
                .expect("reference 2451919 major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2451920_major_body_interior_summary_reports_the_interior_day() {
        let summary = pleiades_jpl::reference_snapshot_2451920_major_body_interior_summary()
            .expect("reference 2451920 major-body interior summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_920.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference 2451920 major-body interior evidence: 10 exact samples at JD 2451920.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-13 interior reference sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_2451920_major_body_interior_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2451920_major_body_interior_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451920_major_body_interior_summary()
            .expect("reference 2451920 major-body interior summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451920 major-body interior summary should fail validation");

        assert!(matches!(
            error,
            Reference2451920MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_2451920_major_body_interior_summary_for_report(),
            pleiades_jpl::reference_snapshot_2451920_major_body_interior_summary()
                .expect("reference 2451920 major-body interior summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_window_summary_reports_the_expected_windows() {
        let summary = pleiades_jpl::reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist");
        assert_eq!(summary.sample_count, 50);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_916.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(
            summary.windows[0].body,
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(summary.windows[0].sample_count, 5);
        assert_eq!(summary.windows[0].epoch_count, 5);
        assert_eq!(
            summary.windows[9].body,
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(summary.windows[9].sample_count, 5);
        assert_eq!(summary.windows[9].epoch_count, 5);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference major-body high-curvature windows: 50 source-backed samples across 10 bodies and 5 epochs (JD 2451911.5 (TDB)..JD 2451916.5 (TDB)); windows: Sun: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Moon: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Mercury: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Venus: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Saturn: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Uranus: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Neptune: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Pluto: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Mars: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Jupiter: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB)"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_high_curvature_window_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_window_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted high-curvature window summary should fail validation");

        assert!(matches!(
            error,
            ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_high_curvature_window_summary_for_report(),
            pleiades_jpl::reference_snapshot_high_curvature_window_summary()
                .expect("reference high-curvature window summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_window_summary_validation_rejects_window_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist");
        summary.windows[0].body = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted high-curvature window summary should fail validation");

        assert!(matches!(
            error,
            ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync { field: "windows" }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_high_curvature_window_summary_for_report(),
            pleiades_jpl::reference_snapshot_high_curvature_window_summary()
                .expect("reference high-curvature window summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_source_window_summary_validation_rejects_sample_body_order_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");
        summary.sample_bodies.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn reference_snapshot_source_window_summary_validated_report_matches_summary_line() {
        let summary = pleiades_jpl::reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");

        assert_eq!(
            validated_reference_snapshot_source_window_summary_for_report().unwrap(),
            summary.summary_line()
        );
        assert_eq!(
            reference_snapshot_source_window_summary_for_report(),
            summary.summary_line()
        );
        assert!(
            format_validated_reference_snapshot_source_window_summary_for_report(&summary)
                .contains("Reference snapshot source windows: ")
        );
    }

    #[test]
    fn reference_snapshot_source_window_summary_validated_report_falls_back_on_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");
        summary.windows.swap(0, 1);

        assert!(
            format_validated_reference_snapshot_source_window_summary_for_report(&summary)
                .starts_with("Reference snapshot source windows: unavailable (")
        );
    }

    #[test]
    fn reference_snapshot_source_window_summary_reports_the_current_boundary_windows() {
        let summary = pleiades_jpl::reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");

        assert_eq!(summary.sample_count, 277);
        assert_eq!(summary.sample_bodies.len(), 16);
        assert_eq!(summary.epoch_count, 23);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_snapshot_source_window_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            summary.windows[0].body,
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(summary.windows[0].sample_count, 17);
        assert_eq!(summary.windows[0].epoch_count, 17);
        assert_eq!(
            summary.windows[0].earliest_epoch.julian_day.days(),
            2_378_498.5
        );
        assert_eq!(
            summary.windows[0].latest_epoch.julian_day.days(),
            2_634_167.0
        );
        assert_eq!(
            summary.windows[5].body,
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid",
                "99942-Apophis"
            ))
        );
        assert_eq!(summary.windows[5].sample_count, 10);
        assert_eq!(summary.windows[5].epoch_count, 10);
        assert_eq!(
            summary.windows[5].earliest_epoch.julian_day.days(),
            2_378_498.5
        );
        assert_eq!(
            summary.windows[5].latest_epoch.julian_day.days(),
            2_634_167.0
        );
        assert_eq!(
            summary.windows[9].body,
            pleiades_backend::CelestialBody::Venus
        );
        assert_eq!(summary.windows[9].sample_count, 20);
        assert_eq!(summary.windows[9].epoch_count, 20);
        assert_eq!(
            summary.windows[9].earliest_epoch.julian_day.days(),
            2_415_020.5
        );
        assert_eq!(
            summary.windows[9].latest_epoch.julian_day.days(),
            2_453_000.5
        );
        assert_eq!(
            summary.windows[15].body,
            pleiades_backend::CelestialBody::Uranus
        );
        assert_eq!(summary.windows[15].sample_count, 17);
        assert_eq!(summary.windows[15].epoch_count, 17);
        assert_eq!(
            summary.windows[15].earliest_epoch.julian_day.days(),
            2_451_545.0
        );
        assert_eq!(
            summary.windows[15].latest_epoch.julian_day.days(),
            2_453_000.5
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            reference_snapshot_source_window_summary_for_report()
        );
    }

    #[test]
    fn reference_snapshot_source_window_summary_validation_rejects_window_order_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");
        summary.windows.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn reference_snapshot_source_summary_validation_reports_blank_fields() {
        let blank_source = ReferenceSnapshotSourceSummary {
            source: " ".to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: "coverage".to_string(),
            columns: pleiades_jpl::REFERENCE_SNAPSHOT_COLUMNS.to_string(),
            redistribution: pleiades_jpl::REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: "geocentric ecliptic J2000".to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            blank_source.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankSource)
        );

        let blank_coverage = ReferenceSnapshotSourceSummary {
            source: pleiades_jpl::REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: "\n".to_string(),
            columns: pleiades_jpl::REFERENCE_SNAPSHOT_COLUMNS.to_string(),
            redistribution: pleiades_jpl::REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: pleiades_jpl::REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            blank_coverage.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankCoverage)
        );

        let padded_coverage = ReferenceSnapshotSourceSummary {
            source: pleiades_jpl::REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: " coverage ".to_string(),
            columns: pleiades_jpl::REFERENCE_SNAPSHOT_COLUMNS.to_string(),
            redistribution: pleiades_jpl::REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: pleiades_jpl::REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            padded_coverage.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                }
            )
        );

        let multiline_source = ReferenceSnapshotSourceSummary {
            source: "source\nline".to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: pleiades_jpl::REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            columns: pleiades_jpl::REFERENCE_SNAPSHOT_COLUMNS.to_string(),
            redistribution: pleiades_jpl::REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: pleiades_jpl::REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            multiline_source.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                }
            )
        );

        let blank_columns = ReferenceSnapshotSourceSummary {
            source: pleiades_jpl::REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: pleiades_jpl::REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            columns: "\t".to_string(),
            redistribution: pleiades_jpl::REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: pleiades_jpl::REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            blank_columns.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankColumns)
        );

        let blank_redistribution = ReferenceSnapshotSourceSummary {
            source: pleiades_jpl::REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: pleiades_jpl::REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            columns: pleiades_jpl::REFERENCE_SNAPSHOT_COLUMNS.to_string(),
            redistribution: "\n".to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: pleiades_jpl::REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            blank_redistribution.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankRedistribution)
        );

        let blank_frame_treatment = ReferenceSnapshotSourceSummary {
            source: pleiades_jpl::REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: pleiades_jpl::REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            columns: pleiades_jpl::REFERENCE_SNAPSHOT_COLUMNS.to_string(),
            redistribution: pleiades_jpl::REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: "\n".to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            blank_frame_treatment.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankFrameTreatment)
        );

        let padded_frame_treatment = ReferenceSnapshotSourceSummary {
            source: pleiades_jpl::REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            evidence_class: pleiades_jpl::REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
            coverage: pleiades_jpl::REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            columns: pleiades_jpl::REFERENCE_SNAPSHOT_COLUMNS.to_string(),
            redistribution: pleiades_jpl::REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
            checksum: pleiades_jpl::reference_snapshot_source_checksum(),
            frame_treatment: " geocentric ecliptic J2000 ".to_string(),
            time_scale: pleiades_jpl::REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
            reference_epoch: pleiades_jpl::reference_instant(),
        };
        assert_eq!(
            padded_frame_treatment.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "frame_treatment",
                }
            )
        );
    }

    #[test]
    fn reference_snapshot_manifest_summary_rejects_metadata_drift() {
        let summary = pleiades_jpl::reference_snapshot_manifest_summary();
        let error = summary
            .validate_with_expected_metadata(
                "wrong title",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
                "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.",
                &["epoch_jd", "body", "x_km", "y_km", "z_km"],
            )
            .expect_err("reference snapshot manifest summary should reject title drift");

        assert!(matches!(
            error,
            SnapshotManifestSummaryValidationError::MetadataMismatch { field: "title", .. }
        ));
    }
}
