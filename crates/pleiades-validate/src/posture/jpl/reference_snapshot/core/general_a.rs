//! Relocated reference-snapshot core general_a renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::core::general_a`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()`/`*_summary()` constructors, `validate()`/`label()` methods,
//! and all release-gate data; jpl's own rendering stays in place until the
//! Task 14 contract sweep.
//!
//! `ReferenceSnapshotSummary` is the only evidence struct defined in this
//! file; its inherent `summary_line`/`validated_summary_line` and its
//! `Display` impl are re-homed below as `reference_snapshot_summary_line`
//! (`Display` itself cannot be re-implemented here — it is a foreign trait on
//! a foreign type from validate's perspective — so the free-fn equivalent is
//! the re-homed rendering surface). The other 15 free `*_for_report`
//! renderers copied here (lunar boundary, high-curvature, major-body
//! boundary/bridge, Mars/Jupiter boundary, and the dated
//! 2451545/2451910-2451915/2451917-bridge/2451918/2453000 aliases) all read
//! `Option<Struct>` values whose *own* struct definitions and inherent
//! rendering (`summary_line`/`validated_summary_line`/`Display`) live in
//! `reference_summary/reference_snapshot/boundaries/{era_a,era_b,era_c}.rs`
//! (Slice D Task 9, already copied) — those `.validated_summary_line()`
//! calls are now rewired (Slice D Task 13b) to `match summary.validate() {
//! Ok(()) => <local render>, ... }`, calling each struct's local free
//! renderer directly (`validate()` stays on the jpl struct; rendering is
//! local). The `*_summary()` constructors backing all 16 renderers (data
//! accessors, not rendering — even the ones textually defined in this same
//! jpl source file) are never duplicated into validate; every call to one is
//! qualified `pleiades_jpl::<name>()`.
//!
//! Every OTHER-FILE renderer call inside
//! `reference_snapshot_summary_for_report`'s aggregation array (whether
//! already copied to validate at the time of the family task —
//! parity/coverage/evidence/reference_asteroid — or copied later —
//! general_b/boundaries/selected_asteroid) is now rewired to the local copy
//! (Slice D Task 13b), reached via the crate-wide
//! `use crate::posture::jpl::*;` glob import below (see `posture/jpl/mod.rs`
//! for the re-exports this resolves through); only jpl's own
//! `*_summary()`/`*_details()` data accessors stay `pleiades_jpl::`.
//!
//! This file is also the canonical home of jpl's format-helper cluster
//! (`format_reference_snapshot_summary`, `strip_report_prefix`,
//! `format_validated_source_summary_for_report`, `join_display`, `format_bodies`,
//! `format_coordinate_frames`, `format_time_scales`, `format_zodiac_modes`,
//! `format_apparentness_modes`); those are copied verbatim below as
//! `pub(crate) fn`s so they can later serve as the consolidation target for
//! the private per-module reproductions already living in the sibling
//! posture modules (`comparison.rs`, `holdout.rs`, `jpl_posture.rs`,
//! `reference_asteroid.rs`, and this module's own `coverage.rs`/`evidence.rs`/
//! `parity.rs` siblings) — that consolidation is deferred, not part of this
//! task. `checksum64` (general_a.rs:452) is intentionally NOT part of this
//! cluster (not a rendering helper) and is not copied here.

use ::core::fmt;

#[allow(unused_imports)]
use pleiades_jpl::{ReferenceSnapshotSummary, ReferenceSnapshotSummaryValidationError};

// Slice D Task 13b: reach every other posture/jpl submodule's local
// renderers (`parity`/`coverage`/`evidence`/`general_b`/`boundaries::era_*`/
// `selected_asteroid`/`reference_asteroid`) without a `pleiades_jpl::`
// detour, self-containing this file's `reference_snapshot_summary_for_report`
// aggregation array and its 15 sibling `Option<Struct>`-unwrapping renderers.
// See `posture/jpl/mod.rs` for the glob re-exports this resolves through.
#[allow(unused_imports)]
use crate::posture::jpl::*;

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Compact release-facing summary line for the checked-in reference snapshot
/// coverage. Verbatim copy of `ReferenceSnapshotSummary::summary_line`
/// (reference_summary/reference_snapshot/core/general_a.rs:338).
pub(crate) fn reference_snapshot_summary_line(s: &ReferenceSnapshotSummary) -> String {
    format!(
        "Reference snapshot coverage: {} rows across {} bodies and {} epochs ({} asteroid rows; {}..{}); bodies: {}",
        s.row_count,
        s.body_count,
        s.epoch_count,
        s.asteroid_row_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(s.bodies),
    )
}

/// Formats the checked-in reference snapshot coverage for release-facing
/// reporting. Verbatim copy of jpl's `format_reference_snapshot_summary`
/// (reference_summary/reference_snapshot/core/general_a.rs:367), with
/// `summary.summary_line()` rewired to `reference_snapshot_summary_line(summary)`.
pub(crate) fn format_reference_snapshot_summary(summary: &ReferenceSnapshotSummary) -> String {
    reference_snapshot_summary_line(summary)
}

/// Returns the release-facing reference snapshot coverage summary string.
/// Verbatim copy of jpl's `reference_snapshot_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:372), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local) and every entry in the
/// aggregation array qualified per the other-file/local-file split described
/// in the module doc comment above.
pub fn reference_snapshot_summary_for_report() -> String {
    let summary_line = match pleiades_jpl::reference_snapshot_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_snapshot_summary_line(&summary),
            Err(error) => format!("Reference snapshot coverage: unavailable ({error})"),
        },
        None => "Reference snapshot coverage: unavailable".to_string(),
    };

    let summary_lines = [
        reference_snapshot_source_summary_for_report(),
        reference_snapshot_source_window_summary_for_report(),
        reference_snapshot_equatorial_parity_summary_for_report(),
        reference_snapshot_major_body_bridge_summary_for_report(),
        reference_snapshot_batch_parity_summary_for_report(),
        reference_snapshot_1900_selected_body_boundary_summary_for_report(),
        reference_snapshot_2415020_selected_body_boundary_summary_for_report(),
        reference_snapshot_lunar_boundary_summary_for_report(),
        reference_snapshot_high_curvature_summary_for_report(),
        reference_snapshot_high_curvature_window_summary_for_report(),
        reference_snapshot_high_curvature_epoch_coverage_summary_for_report(),
        reference_snapshot_2451545_major_body_boundary_summary_for_report(),
        reference_snapshot_major_body_boundary_summary_for_report(),
        reference_snapshot_exact_j2000_evidence_summary_for_report(),
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
    ];

    let mut report = summary_line;
    for summary in summary_lines {
        report.push('\n');
        report.push_str("  ");
        report.push_str(&summary);
    }
    report
}

/// Reproduced/re-homed verbatim from jpl's `pub(crate)` `strip_report_prefix`
/// (reference_summary/reference_snapshot/core/general_a.rs:447). See the
/// module doc comment: this is the canonical copy; sibling posture modules'
/// existing private reproductions are not consolidated in this task.
pub(crate) fn strip_report_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

/// Verbatim copy of jpl's `#[cfg(test)]` `format_validated_source_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:480).
#[cfg(test)]
pub(crate) fn format_validated_source_summary_for_report(
    label: &'static str,
    manifest: &pleiades_jpl::SnapshotManifest,
    render: impl FnOnce() -> String,
) -> String {
    match manifest.validate() {
        Ok(()) => render(),
        Err(error) => format!("{label}: unavailable ({error})"),
    }
}

/// Verbatim copy of jpl's `pub(crate)` `join_display`
/// (reference_summary/reference_snapshot/core/general_a.rs:502).
pub(crate) fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Verbatim copy of jpl's `pub(crate)` `format_bodies`
/// (reference_summary/reference_snapshot/core/general_a.rs:510). This is the
/// canonical copy referenced by this file's own renderers above.
pub(crate) fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    join_display(bodies)
}

/// Verbatim copy of jpl's `pub(crate)` `format_coordinate_frames`
/// (reference_summary/reference_snapshot/core/general_a.rs:514).
pub(crate) fn format_coordinate_frames(frames: &[pleiades_types::CoordinateFrame]) -> String {
    join_display(frames)
}

/// Verbatim copy of jpl's `pub(crate)` `format_time_scales`
/// (reference_summary/reference_snapshot/core/general_a.rs:518).
pub(crate) fn format_time_scales(time_scales: &[pleiades_types::TimeScale]) -> String {
    join_display(time_scales)
}

/// Verbatim copy of jpl's `pub(crate)` `format_zodiac_modes`
/// (reference_summary/reference_snapshot/core/general_a.rs:522).
pub(crate) fn format_zodiac_modes(zodiac_modes: &[pleiades_types::ZodiacMode]) -> String {
    join_display(zodiac_modes)
}

/// Verbatim copy of jpl's `pub(crate)` `format_apparentness_modes`
/// (reference_summary/reference_snapshot/core/general_a.rs:526).
pub(crate) fn format_apparentness_modes(modes: &[pleiades_types::Apparentness]) -> String {
    join_display(modes)
}

/// Returns the release-facing Moon high-curvature reference window summary
/// string. Verbatim copy of jpl's `reference_snapshot_lunar_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:651). The
/// `ReferenceLunarBoundarySummary` struct and its rendering live in
/// `reference_summary/reference_snapshot/boundaries/era_a.rs` (Slice D Task
/// 9, already copied); `.validated_summary_line()` is rewired to `match
/// summary.validate() { Ok(()) => <local render>, ... }`, calling the local
/// `reference_lunar_boundary_summary_line` (Slice D Task 13b).
pub fn reference_snapshot_lunar_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_lunar_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_lunar_boundary_summary_line(&summary),
            Err(error) => format!("Reference lunar boundary evidence: unavailable ({error})"),
        },
        None => "Reference lunar boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing major-body high-curvature reference window
/// summary string. Verbatim copy of jpl's
/// `reference_snapshot_high_curvature_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:710). The
/// `ReferenceHighCurvatureSummary` struct/rendering live in
/// `boundaries/era_a.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local `reference_high_curvature_summary_line`
/// (Slice D Task 13b).
pub(crate) fn reference_snapshot_high_curvature_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_high_curvature_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_high_curvature_summary_line(&summary),
            Err(error) => {
                format!("Reference major-body high-curvature evidence: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing major-body boundary-day summary string.
/// Verbatim copy of jpl's `reference_snapshot_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:769). The
/// `ReferenceMajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_a.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local `reference_major_body_boundary_summary_line`
/// (Slice D Task 13b).
pub(crate) fn reference_snapshot_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing major-body bridge-day summary string. Verbatim
/// copy of jpl's `reference_snapshot_major_body_bridge_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:829). The
/// `ReferenceMajorBodyBridgeSummary` struct/rendering live in
/// `boundaries/era_a.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local `reference_major_body_bridge_summary_line`
/// (Slice D Task 13b).
pub fn reference_snapshot_major_body_bridge_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_major_body_bridge_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_major_body_bridge_summary_line(&summary),
            Err(error) => format!("Reference major-body bridge evidence: unavailable ({error})"),
        },
        None => "Reference major-body bridge evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing Mars/Jupiter boundary summary string. Verbatim
/// copy of jpl's `reference_snapshot_mars_jupiter_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:886). The
/// `ReferenceMarsJupiterBoundarySummary` struct/rendering live in
/// `boundaries/era_a.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local `reference_mars_jupiter_boundary_summary_line`
/// (Slice D Task 13b).
pub(crate) fn reference_snapshot_mars_jupiter_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_mars_jupiter_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_mars_jupiter_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference Mars/Jupiter boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference Mars/Jupiter boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451918 major-body boundary summary string.
/// This is a compatibility alias for the Mars/Jupiter boundary slice with
/// explicit 2451918 wording for release-facing reports. Verbatim copy of
/// jpl's `reference_snapshot_2451918_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:910). The
/// underlying `ReferenceMarsJupiterBoundarySummary` rendering lives in
/// `boundaries/era_a.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local `reference_mars_jupiter_boundary_summary_line`
/// (Slice D Task 13b).
pub fn reference_snapshot_2451918_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451918_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_mars_jupiter_boundary_summary_line(&summary).replacen(
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

/// Returns the release-facing 2453000 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2453000_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:975). The
/// `Reference2453000MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2453000_major_body_boundary_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2453000_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2453000_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2453000_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2453000 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2453000 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451545 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451545_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1036). The
/// `Reference2451545MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451545_major_body_boundary_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451545_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451545_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451545_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451545 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451545 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451910 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451910_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1097). The
/// `Reference2451910MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451910_major_body_boundary_summary_line` (Slice D Task 13b).
pub fn reference_snapshot_2451910_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451910_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451910_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451910 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451910 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451911 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451911_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1158). The
/// `Reference2451911MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451911_major_body_boundary_summary_line` (Slice D Task 13b).
pub fn reference_snapshot_2451911_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451911_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451911_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451911 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451911 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451912 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451912_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1219). The
/// `Reference2451912MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451912_major_body_boundary_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451912_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451912_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451912_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451912 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451912 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451913 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451913_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1280). The
/// `Reference2451913MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451913_major_body_boundary_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451913_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451913_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451913_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451913 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451913 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451914 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451914_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1341). The
/// `Reference2451914MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451914_major_body_boundary_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451914_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451914_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451914_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451914 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451914 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451915 major-body boundary summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451915_major_body_boundary_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1402). The
/// `Reference2451915MajorBodyBoundarySummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451915_major_body_boundary_summary_line` (Slice D Task 13b).
pub(crate) fn reference_snapshot_2451915_major_body_boundary_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451915_major_body_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451915_major_body_boundary_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451915 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451915 major-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing 2451917 major-body bridge summary string.
/// Verbatim copy of jpl's
/// `reference_snapshot_2451917_major_body_bridge_summary_for_report`
/// (reference_summary/reference_snapshot/core/general_a.rs:1463). The
/// `Reference2451917MajorBodyBridgeSummary` struct/rendering live in
/// `boundaries/era_c.rs` (Slice D Task 9, already copied); rewired the same
/// way as above, calling the local
/// `reference_2451917_major_body_bridge_summary_line` (Slice D Task 13b).
pub fn reference_snapshot_2451917_major_body_bridge_summary_for_report() -> String {
    match pleiades_jpl::reference_snapshot_2451917_major_body_bridge_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => reference_2451917_major_body_bridge_summary_line(&summary),
            Err(error) => {
                format!("Reference 2451917 major-body bridge evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451917 major-body bridge evidence: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_jpl::{
        Reference2451545MajorBodyBoundarySummaryValidationError,
        Reference2451910MajorBodyBoundarySummaryValidationError,
        Reference2451911MajorBodyBoundarySummaryValidationError,
        Reference2451912MajorBodyBoundarySummaryValidationError,
        Reference2451913MajorBodyBoundarySummaryValidationError,
        Reference2451914MajorBodyBoundarySummaryValidationError,
        Reference2451915MajorBodyBoundarySummaryValidationError,
        Reference2453000MajorBodyBoundarySummaryValidationError,
        ReferenceMajorBodyBoundarySummaryValidationError,
        ReferenceMajorBodyBridgeSummaryValidationError,
        ReferenceMarsJupiterBoundarySummaryValidationError,
    };

    // 8a's deferred test (tests.rs:86): needed `reference_snapshot_summary`/
    // `reference_snapshot_summary_for_report`, both general_a (this file).
    // This is a containment test (`report.contains(&<renderer>())`), so every
    // `_for_report` call in its assertions — this file's own siblings and the
    // other-file renderers alike — is validate-local (Slice D Task 13b),
    // reached through the module's `use crate::posture::jpl::*;` glob; a
    // validate aggregate containing its validate sub-renderer output is the
    // byte-identical check that survives Task 14's jpl-render deletion.
    #[test]
    fn reference_snapshot_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::reference_snapshot_summary()
            .expect("reference snapshot summary should exist");
        summary
            .validate()
            .expect("reference snapshot summary should validate");
        assert_eq!(summary.row_count, 277);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.bodies, pleiades_jpl::reference_bodies());
        assert_eq!(summary.epoch_count, 23);
        assert_eq!(summary.asteroid_row_count, 95);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            reference_snapshot_summary_line(&summary),
            format!(
                "Reference snapshot coverage: 277 rows across 16 bodies and 23 epochs (95 asteroid rows; JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}",
                format_bodies(pleiades_jpl::reference_bodies())
            )
        );
        let report = reference_snapshot_summary_for_report();
        assert!(report.contains(reference_snapshot_summary_line(&summary).as_str()));
        assert!(report.contains(&reference_snapshot_source_summary_for_report()));
        assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(report.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
        assert!(report.contains(&reference_asteroid_evidence_summary_for_report()));
        assert!(report.contains(&reference_asteroid_equatorial_evidence_summary_for_report()));
        assert!(report.contains(&reference_asteroid_source_window_summary_for_report()));
        assert!(report.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
        assert!(report.contains(&reference_snapshot_high_curvature_summary_for_report()));
        assert!(report.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451545_major_body_boundary_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_major_body_boundary_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451912_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451913_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451914_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_2451915_major_body_bridge_summary_for_report()));
        assert!(report.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451916_major_body_interior_summary_for_report())
        );
        assert!(report
            .contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451916_major_body_boundary_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_dense_boundary_summary_for_report()));
        assert!(report.contains(&reference_snapshot_mars_jupiter_boundary_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2453000_major_body_boundary_summary_for_report())
        );
        assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
        assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
        assert!(report.contains(&selected_asteroid_dense_boundary_summary_for_report()));
        assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
        // Migrated from jpl's whole-aggregator
        // `reference_snapshot_summary_for_report_highlights_recent_reference_slices`
        // (reference_summary/reference_snapshot/tests.rs:1713), which this test did
        // not yet have full parity with (Slice D Task 13d).
        assert!(report.contains(&selected_asteroid_source_2451917_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_2453000_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_2500000_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_2634167_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_evidence_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_window_summary_for_report()));
        assert!(report.contains(&reference_snapshot_equatorial_parity_summary_for_report()));
        assert!(report.contains(&reference_snapshot_batch_parity_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_1900_selected_body_boundary_summary_for_report())
        );
        assert!(report
            .contains(&reference_snapshot_2415020_selected_body_boundary_summary_for_report()));
        assert!(report.contains(&reference_snapshot_exact_j2000_evidence_summary_for_report()));
        assert!(report.contains(&reference_snapshot_bridge_day_summary_for_report()));
        assert!(report.contains(&reference_snapshot_2451914_bridge_day_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_sparse_boundary_summary_for_report()));
        assert!(report.contains(&reference_snapshot_pre_bridge_boundary_summary_for_report()));
        assert!(report.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_major_body_boundary_window_summary_for_report())
        );
        assert!(!report.contains("JPL independent hold-out:"));
        assert!(!report.contains("Reference/hold-out overlap:"));
    }

    #[test]
    fn reference_snapshot_lunar_boundary_summary_reports_the_expected_window() {
        let summary = pleiades_jpl::reference_snapshot_lunar_boundary_summary()
            .expect("reference lunar boundary summary should exist");
        assert_eq!(summary.sample_count, 2);
        assert_eq!(summary.epoch_count, 2);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_912.5);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_lunar_boundary_summary_line(&summary),
            "Reference lunar boundary evidence: 2 exact Moon samples at JD 2451911.5 (TDB)..JD 2451912.5 (TDB); high-curvature interpolation window"
        );
        assert_eq!(
            reference_snapshot_lunar_boundary_summary_for_report(),
            reference_lunar_boundary_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_summary_reports_the_expected_window() {
        let summary = pleiades_jpl::reference_snapshot_high_curvature_summary()
            .expect("reference high-curvature summary should exist");
        assert_eq!(summary.sample_count, 50);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.bodies.len(), 10);
        assert_eq!(summary.epoch_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_916.5);
        assert_eq!(summary.bodies[0], pleiades_backend::CelestialBody::Sun);
        assert_eq!(summary.bodies[9], pleiades_backend::CelestialBody::Jupiter);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_high_curvature_summary_line(&summary),
            "Reference major-body high-curvature evidence: 50 exact samples across 10 bodies and 5 epochs (JD 2451911.5 (TDB)..JD 2451916.5 (TDB)); bodies: Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter; high-curvature interpolation window"
        );
        assert_eq!(
            reference_snapshot_high_curvature_summary_for_report(),
            reference_high_curvature_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist");
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
        assert_eq!(
            reference_major_body_boundary_summary_line(&summary),
            "Reference major-body boundary evidence: 10 exact samples at JD 2451917.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-08 boundary sample"
        );
        assert_eq!(
            reference_snapshot_major_body_boundary_summary_for_report(),
            reference_major_body_boundary_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_major_body_bridge_summary_reports_the_bridge_day() {
        let summary = pleiades_jpl::reference_snapshot_major_body_bridge_summary()
            .expect("reference major-body bridge summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_915.0);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_major_body_bridge_summary_line(&summary),
            "Reference major-body bridge evidence: 10 exact samples at JD 2451915.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); bridge sample across the major-body boundary window"
        );
        assert_eq!(
            reference_snapshot_major_body_bridge_summary_for_report(),
            reference_major_body_bridge_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_major_body_bridge_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_major_body_bridge_summary()
            .expect("reference major-body bridge summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted major-body bridge summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMajorBodyBridgeSummaryValidationError::SampleCountMismatch {
                sample_count: 11,
                derived_sample_count: 10
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_major_body_bridge_summary_for_report(),
            reference_major_body_bridge_summary_line(
                &pleiades_jpl::reference_snapshot_major_body_bridge_summary()
                    .expect("reference major-body bridge summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                sample_count: 11,
                derived_sample_count: 10
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_major_body_boundary_summary_for_report(),
            reference_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_major_body_boundary_summary()
                    .expect("reference major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_major_body_boundary_summary_validation_rejects_body_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_major_body_boundary_summary_for_report(),
            reference_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_major_body_boundary_summary()
                    .expect("reference major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_mars_jupiter_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_mars_jupiter_boundary_summary()
            .expect("reference Mars/Jupiter boundary summary should exist");
        assert_eq!(summary.sample_count, 16);
        assert_eq!(summary.sample_bodies.len(), 16);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_918.5);
        assert_eq!(
            summary.sample_bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Pluto,
                pleiades_backend::CelestialBody::Ceres,
                pleiades_backend::CelestialBody::Pallas,
                pleiades_backend::CelestialBody::Juno,
                pleiades_backend::CelestialBody::Vesta,
                pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                    "asteroid", "433-Eros"
                )),
                pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                    "asteroid",
                    "99942-Apophis"
                )),
            ]
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_mars_jupiter_boundary_summary_line(&summary),
            "Reference Mars/Jupiter boundary evidence: 16 exact samples at JD 2451918.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2001-01-09 boundary sample"
        );
        assert_eq!(
            reference_snapshot_2451918_major_body_boundary_summary_for_report(),
            "Reference 2451918 major-body boundary evidence: 16 exact samples at JD 2451918.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2001-01-09 boundary sample"
        );
        assert_eq!(
            reference_snapshot_mars_jupiter_boundary_summary_for_report(),
            reference_mars_jupiter_boundary_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_mars_jupiter_boundary_summary_validation_rejects_body_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_mars_jupiter_boundary_summary()
            .expect("reference Mars/Jupiter boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted Mars/Jupiter boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMarsJupiterBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_mars_jupiter_boundary_summary_for_report(),
            reference_mars_jupiter_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_mars_jupiter_boundary_summary()
                    .expect("reference Mars/Jupiter boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451918_major_body_boundary_summary_alias_uses_explicit_2451918_wording()
    {
        let boundary_2451918 = reference_snapshot_2451918_major_body_boundary_summary_for_report();
        let boundary_generic = reference_snapshot_mars_jupiter_boundary_summary_for_report();
        let summary = pleiades_jpl::reference_snapshot_2451918_major_body_boundary_summary()
            .expect("reference 2451918 major-body boundary summary should exist");

        assert!(boundary_2451918.contains("Reference 2451918 major-body boundary evidence:"));
        assert!(boundary_2451918.contains("JD 2451918.5 (TDB)"));
        assert_eq!(
            boundary_2451918,
            reference_mars_jupiter_boundary_summary_line(&summary).replacen(
                "Reference Mars/Jupiter boundary evidence",
                "Reference 2451918 major-body boundary evidence",
                1
            )
        );
        assert_eq!(
            reference_mars_jupiter_boundary_summary_line(&summary),
            reference_mars_jupiter_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_mars_jupiter_boundary_summary()
                    .expect("reference Mars/Jupiter boundary summary should exist")
            )
        );
        assert_ne!(boundary_2451918, boundary_generic);
    }

    #[test]
    fn reference_snapshot_2451918_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451918_major_body_boundary_summary()
            .expect("reference 2451918 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451918 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMarsJupiterBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451918_major_body_boundary_summary_for_report(),
            reference_mars_jupiter_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451918_major_body_boundary_summary()
                    .expect("reference 2451918 major-body boundary summary should exist")
            )
            .replacen(
                "Reference Mars/Jupiter boundary evidence",
                "Reference 2451918 major-body boundary evidence",
                1
            )
        );
    }

    #[test]
    fn reference_snapshot_2451910_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451910_major_body_boundary_summary()
            .expect("reference 2451910 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_910.5);
        assert_eq!(
            summary.sample_bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Pluto,
            ]
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2451910_major_body_boundary_summary_line(&summary),
            "Reference 2451910 major-body boundary evidence: 10 exact samples at JD 2451910.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-01 boundary sample"
        );
        assert_eq!(
            reference_snapshot_2451910_major_body_boundary_summary_for_report(),
            reference_2451910_major_body_boundary_summary_line(&summary)
        );
        assert!(reference_snapshot_summary_for_report()
            .contains(reference_2451910_major_body_boundary_summary_line(&summary).as_str()));
    }

    #[test]
    fn reference_snapshot_2451910_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451910_major_body_boundary_summary()
            .expect("reference 2451910 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451910 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451910MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451910_major_body_boundary_summary_for_report(),
            reference_2451910_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451910_major_body_boundary_summary()
                    .expect("reference 2451910 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451911_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451911_major_body_boundary_summary()
            .expect("reference 2451911 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_911.5);
        assert_eq!(
            summary.sample_bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Pluto,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
            ]
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_snapshot_2451911_major_body_boundary_summary_for_report(),
            reference_2451911_major_body_boundary_summary_line(&summary)
        );
        assert!(reference_snapshot_summary_for_report()
            .contains(reference_2451911_major_body_boundary_summary_line(&summary).as_str()));
    }

    #[test]
    fn reference_snapshot_2451911_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451911_major_body_boundary_summary()
            .expect("reference 2451911 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451911 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451911MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451911_major_body_boundary_summary_for_report(),
            reference_2451911_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451911_major_body_boundary_summary()
                    .expect("reference 2451911 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451912_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451912_major_body_boundary_summary()
            .expect("reference 2451912 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_912.5);
        assert_eq!(
            summary.sample_bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Pluto,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
            ]
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2451912_major_body_boundary_summary_line(&summary),
            "Reference 2451912 major-body boundary evidence: 10 exact samples at JD 2451912.5 (TDB) (Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter); 2001-01-03 boundary sample"
        );
        assert_eq!(
            reference_snapshot_2451912_major_body_boundary_summary_for_report(),
            reference_2451912_major_body_boundary_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_2451912_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451912_major_body_boundary_summary()
            .expect("reference 2451912 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451912 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451912MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451912_major_body_boundary_summary_for_report(),
            reference_2451912_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451912_major_body_boundary_summary()
                    .expect("reference 2451912 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451913_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451913_major_body_boundary_summary()
            .expect("reference 2451913 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_913.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2451913_major_body_boundary_summary_line(&summary),
            "Reference 2451913 major-body boundary evidence: 10 exact samples at JD 2451913.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-04 boundary sample"
        );
        assert_eq!(
            reference_snapshot_2451913_major_body_boundary_summary_for_report(),
            reference_2451913_major_body_boundary_summary_line(&summary)
        );
        assert!(reference_snapshot_summary_for_report()
            .contains(reference_2451913_major_body_boundary_summary_line(&summary).as_str()));
    }

    #[test]
    fn reference_snapshot_2451913_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451913_major_body_boundary_summary()
            .expect("reference 2451913 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451913 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451913MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451913_major_body_boundary_summary_for_report(),
            reference_2451913_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451913_major_body_boundary_summary()
                    .expect("reference 2451913 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451914_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451914_major_body_boundary_summary()
            .expect("reference 2451914 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_914.5);
        assert_eq!(
            summary.sample_bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Pluto,
            ]
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2451914_major_body_boundary_summary_line(&summary),
            "Reference 2451914 major-body boundary evidence: 10 exact samples at JD 2451914.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-05 boundary sample"
        );
        assert_eq!(
            reference_snapshot_2451914_major_body_boundary_summary_for_report(),
            reference_2451914_major_body_boundary_summary_line(&summary)
        );
        assert!(reference_snapshot_summary_for_report()
            .contains(reference_2451914_major_body_boundary_summary_line(&summary).as_str()));
    }

    #[test]
    fn reference_snapshot_2451914_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451914_major_body_boundary_summary()
            .expect("reference 2451914 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451914 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451914MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451914_major_body_boundary_summary_for_report(),
            reference_2451914_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451914_major_body_boundary_summary()
                    .expect("reference 2451914 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451915_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2451915_major_body_boundary_summary()
            .expect("reference 2451915 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_915.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2451915_major_body_boundary_summary_line(&summary),
            "Reference 2451915 major-body boundary evidence: 10 exact samples at JD 2451915.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-06 boundary sample"
        );
        assert_eq!(
            reference_snapshot_2451915_major_body_boundary_summary_for_report(),
            reference_2451915_major_body_boundary_summary_line(&summary)
        );
        assert!(reference_snapshot_summary_for_report()
            .contains(reference_2451915_major_body_boundary_summary_line(&summary).as_str()));
    }

    #[test]
    fn reference_snapshot_2451915_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451915_major_body_boundary_summary()
            .expect("reference 2451915 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451915 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451915MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451915_major_body_boundary_summary_for_report(),
            reference_2451915_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451915_major_body_boundary_summary()
                    .expect("reference 2451915 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451545_major_body_boundary_summary_reports_the_j2000_reference_day() {
        let summary = pleiades_jpl::reference_snapshot_2451545_major_body_boundary_summary()
            .expect("reference 2451545 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_545.0);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Venus
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2451545_major_body_boundary_summary_line(&summary),
            "Reference 2451545 major-body boundary evidence: 10 exact samples at JD 2451545.0 (TDB) (Jupiter, Mars, Mercury, Moon, Neptune, Pluto, Saturn, Sun, Uranus, Venus); J2000 reference sample"
        );
        assert_eq!(
            reference_snapshot_2451545_major_body_boundary_summary_for_report(),
            reference_2451545_major_body_boundary_summary_line(&summary)
        );
        assert!(reference_snapshot_summary_for_report()
            .contains(reference_2451545_major_body_boundary_summary_line(&summary).as_str()));
    }

    #[test]
    fn reference_snapshot_2451545_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2451545_major_body_boundary_summary()
            .expect("reference 2451545 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2451545 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2451545MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Jupiter,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2451545_major_body_boundary_summary_for_report(),
            reference_2451545_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2451545_major_body_boundary_summary()
                    .expect("reference 2451545 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2453000_major_body_boundary_summary_reports_the_late_boundary_day() {
        let summary = pleiades_jpl::reference_snapshot_2453000_major_body_boundary_summary()
            .expect("reference 2453000 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_453_000.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2453000_major_body_boundary_summary_line(&summary),
            "Reference 2453000 major-body boundary evidence: 10 exact samples at JD 2453000.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2453000.5 boundary sample"
        );
        assert_eq!(
            reference_snapshot_2453000_major_body_boundary_summary_for_report(),
            reference_2453000_major_body_boundary_summary_line(&summary)
        );
        assert!(reference_snapshot_summary_for_report()
            .contains(reference_2453000_major_body_boundary_summary_line(&summary).as_str()));
    }

    #[test]
    fn reference_snapshot_2453000_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_2453000_major_body_boundary_summary()
            .expect("reference 2453000 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2453000 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2453000MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validate().is_err());
        assert_eq!(
            reference_snapshot_2453000_major_body_boundary_summary_for_report(),
            reference_2453000_major_body_boundary_summary_line(
                &pleiades_jpl::reference_snapshot_2453000_major_body_boundary_summary()
                    .expect("reference 2453000 major-body boundary summary should exist")
            )
        );
    }

    #[test]
    fn reference_snapshot_2451917_major_body_bridge_summary_reports_the_bridge_day() {
        let summary = pleiades_jpl::reference_snapshot_2451917_major_body_bridge_summary()
            .expect("reference 2451917 major-body bridge summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_917.0);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_2451917_major_body_bridge_summary_line(&summary),
            "Reference 2451917 major-body bridge evidence: 10 exact samples at JD 2451917.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); bridge sample across the major-body boundary window"
        );
        assert_eq!(
            reference_snapshot_2451917_major_body_bridge_summary_for_report(),
            reference_2451917_major_body_bridge_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_body_count_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_summary()
            .expect("reference snapshot summary should exist");
        summary.body_count += 1;

        let error = summary
            .validate()
            .expect_err("body-count drift should be rejected");
        assert_eq!(error.label(), "body count mismatch");
        assert!(error
            .to_string()
            .contains("body count 17 does not match body list length 16"));
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_body_order_drift() {
        let summary = pleiades_jpl::reference_snapshot_summary()
            .expect("reference snapshot summary should exist");
        let mut bodies = pleiades_jpl::reference_bodies().to_vec();
        bodies.swap(0, 1);
        let leaked_bodies: &'static [pleiades_backend::CelestialBody] =
            Box::leak(bodies.into_boxed_slice());
        let summary = ReferenceSnapshotSummary {
            bodies: leaked_bodies,
            ..summary
        };

        let error = summary
            .validate()
            .expect_err("body-order drift should be rejected");
        assert_eq!(error.label(), "body order mismatch");
        assert!(error.to_string().contains("index 0"));
        assert!(error
            .to_string()
            .contains(&pleiades_jpl::reference_bodies()[0].to_string()));
        assert!(error
            .to_string()
            .contains(&pleiades_jpl::reference_bodies()[1].to_string()));
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_row_count_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_summary()
            .expect("reference snapshot summary should exist");
        summary.row_count += 1;

        assert_eq!(
            summary.validate(),
            Err(ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_epoch_count_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_summary()
            .expect("reference snapshot summary should exist");
        summary.epoch_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotSummaryValidationError::EpochCountMismatch {
                    epoch_count: 24,
                    derived_epoch_count: 23,
                }
            )
        ));
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_asteroid_row_count_drift() {
        let mut summary = pleiades_jpl::reference_snapshot_summary()
            .expect("reference snapshot summary should exist");
        summary.asteroid_row_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                    asteroid_row_count: 96,
                    derived_asteroid_row_count: 95,
                }
            )
        ));
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_duplicate_bodies() {
        let summary = ReferenceSnapshotSummary {
            row_count: 2,
            body_count: 2,
            bodies: &[
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Sun,
            ],
            epoch_count: 1,
            asteroid_row_count: 0,
            earliest_epoch: pleiades_jpl::reference_instant(),
            latest_epoch: pleiades_jpl::reference_instant(),
        };

        assert!(matches!(
            summary.validate(),
            Err(ReferenceSnapshotSummaryValidationError::DuplicateBody {
                first_index: 0,
                second_index: 1,
                body,
            }) if body == "Sun"
        ));
    }
}
