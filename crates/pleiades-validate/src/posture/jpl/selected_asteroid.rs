//! Relocated selected-asteroid renderers copied from
//! `pleiades-jpl::reference_summary::selected_asteroid` (report-surface
//! relocation program, Slice D). Rendering only — the functional crate keeps
//! the structured evidence structs, their `*_details()` constructors,
//! `validate()`/`label()` methods, and all release-gate data; jpl's own
//! rendering stays in place until the Task 14 contract sweep.
//!
//! Also copies the two data-file selected-asteroid renderers
//! (`selected_asteroid_source_2451917_summary_for_report` from
//! `pleiades-jpl::data::selected_asteroid_2001`,
//! `selected_asteroid_source_2378498_summary_for_report` from
//! `pleiades-jpl::data::selected_asteroid_2378498`). Their
//! `SelectedAsteroidSource2451917Summary`/`SelectedAsteroidSource2378498Summary`
//! structs and `*_summary()` constructors stay in jpl `data/` permanently —
//! they are checked-in data, not rendering — and are promoted from
//! module-private to crate-root-reexported (Slice D Task 11) so this file can
//! name them.

use pleiades_jpl::{
    SelectedAsteroidBatchParitySummary, SelectedAsteroidBoundarySummary,
    SelectedAsteroidBridgeSummary, SelectedAsteroidDenseBoundarySummary,
    SelectedAsteroidSource2378498Summary, SelectedAsteroidSource2451917Summary,
    SelectedAsteroidSource2453000Summary, SelectedAsteroidSource2500000Summary,
    SelectedAsteroidSource2634167Summary, SelectedAsteroidSourceRequestCorpusSummary,
    SelectedAsteroidSourceSummary, SelectedAsteroidSourceWindow,
    SelectedAsteroidSourceWindowSummary, SelectedAsteroidTerminalBoundarySummary,
};
use pleiades_types::CoordinateFrame;

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
/// — already reproduced identically in the sibling `comparison.rs`,
/// `holdout.rs`, `jpl_posture.rs`, and `reference_asteroid.rs` posture
/// modules.
fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Compact release-facing summary line for one selected-asteroid body window.
/// Verbatim copy of `SelectedAsteroidSourceWindow::summary_line`
/// (reference_summary/selected_asteroid.rs:136). Named `..._window_line`
/// (not `..._window_summary_line`) to avoid colliding with the aggregate
/// `SelectedAsteroidSourceWindowSummary` render fn below, mirroring the
/// `reference_snapshot_source_window_line` / `reference_snapshot_source_window_summary_line`
/// split used for the analogous struct pair in
/// `reference_snapshot/core/general_b.rs`.
pub(crate) fn selected_asteroid_source_window_line(s: &SelectedAsteroidSourceWindow) -> String {
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

/// Compact release-facing summary line for the expanded selected-asteroid
/// source coverage. Verbatim copy of
/// `SelectedAsteroidSourceSummary::summary_line`
/// (reference_summary/selected_asteroid.rs:171).
pub(crate) fn selected_asteroid_source_summary_line(s: &SelectedAsteroidSourceSummary) -> String {
    format!(
        "Selected asteroid source evidence: {} source-backed samples across {} bodies and {} epochs ({}..{}); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the selected-asteroid source
/// windows. Verbatim copy of
/// `SelectedAsteroidSourceWindowSummary::summary_line`
/// (reference_summary/selected_asteroid.rs:283), with the nested
/// `SelectedAsteroidSourceWindow::summary_line` call rewired to the local
/// `selected_asteroid_source_window_line` (same-file struct, per the recipe).
pub(crate) fn selected_asteroid_source_window_summary_line(
    s: &SelectedAsteroidSourceWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(selected_asteroid_source_window_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Selected asteroid source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Compact release-facing summary line for the selected-asteroid source
/// request corpus. Verbatim copy of
/// `SelectedAsteroidSourceRequestCorpusSummary::summary_line`
/// (reference_summary/selected_asteroid.rs:599).
pub(crate) fn selected_asteroid_source_request_corpus_summary_line(
    s: &SelectedAsteroidSourceRequestCorpusSummary,
) -> String {
    format!(
        "Selected asteroid source request corpus: {} requests (frame={}; time scale={}; zodiac mode={}; apparentness={}; observerless) across {} bodies and {} epochs ({}..{}); bodies: {}",
        s.request_count,
        s.frame,
        s.time_scale,
        s.zodiac_mode,
        s.apparentness,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(&s.bodies),
    )
}

/// Compact release-facing summary line for the selected-asteroid 2003-12-27
/// source evidence. Verbatim copy of
/// `SelectedAsteroidSource2453000Summary::summary_line`
/// (reference_summary/selected_asteroid.rs:927).
pub(crate) fn selected_asteroid_source_2453000_summary_line(
    s: &SelectedAsteroidSource2453000Summary,
) -> String {
    format!(
        "Reference selected-asteroid 2003-12-27 source evidence: {} exact samples at {} ({}); 2003-12-27 source sample",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the selected-asteroid 2500000
/// source evidence. Verbatim copy of
/// `SelectedAsteroidSource2500000Summary::summary_line`
/// (reference_summary/selected_asteroid.rs:1139).
pub(crate) fn selected_asteroid_source_2500000_summary_line(
    s: &SelectedAsteroidSource2500000Summary,
) -> String {
    format!(
        "Reference selected-asteroid 2500000 source evidence: {} exact samples at {} ({}); 2500000 source sample",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the selected-asteroid 2634167
/// source evidence. Verbatim copy of
/// `SelectedAsteroidSource2634167Summary::summary_line`
/// (reference_summary/selected_asteroid.rs:1351).
pub(crate) fn selected_asteroid_source_2634167_summary_line(
    s: &SelectedAsteroidSource2634167Summary,
) -> String {
    format!(
        "Reference selected-asteroid 2634167 source evidence: {} exact samples at {} ({}); 2634167 source sample",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the selected-asteroid bridge-day
/// evidence. Verbatim copy of `SelectedAsteroidBridgeSummary::summary_line`
/// (reference_summary/selected_asteroid.rs:1563).
pub(crate) fn selected_asteroid_bridge_summary_line(s: &SelectedAsteroidBridgeSummary) -> String {
    format!(
        "Selected asteroid bridge evidence: {} exact samples at {} ({}); bridge sample across the asteroid-only gap",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the dense selected-asteroid
/// boundary day. Verbatim copy of
/// `SelectedAsteroidDenseBoundarySummary::summary_line`
/// (reference_summary/selected_asteroid.rs:1758).
pub(crate) fn selected_asteroid_dense_boundary_summary_line(
    s: &SelectedAsteroidDenseBoundarySummary,
) -> String {
    format!(
        "Selected asteroid dense boundary evidence: {} exact samples at {} ({}); dense boundary day",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the selected-asteroid boundary-day
/// evidence. Verbatim copy of `SelectedAsteroidBoundarySummary::summary_line`
/// (reference_summary/selected_asteroid.rs:1963).
pub(crate) fn selected_asteroid_boundary_summary_line(
    s: &SelectedAsteroidBoundarySummary,
) -> String {
    let epochs = match s.epochs.as_slice() {
        [] => String::from("(no epochs)"),
        [epoch] => format_instant(*epoch),
        [first, .., last] => format!("{}..{}", format_instant(*first), format_instant(*last)),
    };
    format!(
        "Selected asteroid boundary evidence: {} exact samples across {} epochs at {} ({})",
        s.sample_count,
        s.epochs.len(),
        epochs,
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the terminal selected-asteroid
/// boundary day. Verbatim copy of
/// `SelectedAsteroidTerminalBoundarySummary::summary_line`
/// (reference_summary/selected_asteroid.rs:2189).
pub(crate) fn selected_asteroid_terminal_boundary_summary_line(
    s: &SelectedAsteroidTerminalBoundarySummary,
) -> String {
    format!(
        "Reference selected-asteroid terminal boundary evidence: {} exact samples at {} ({}); 2500-01-01 terminal boundary sample",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the mixed-frame selected-asteroid
/// batch parity slice. Verbatim copy of
/// `SelectedAsteroidBatchParitySummary::summary_line`
/// (reference_summary/selected_asteroid.rs:2416).
pub(crate) fn selected_asteroid_batch_parity_summary_line(
    s: &SelectedAsteroidBatchParitySummary,
) -> String {
    let parity = if s.parity_preserved {
        "preserved"
    } else {
        "not preserved"
    };

    format!(
        "Selected asteroid batch parity: {} requests across {} bodies at {} ({}); frame mix: {} ecliptic, {} equatorial; batch/single parity {}",
        s.request_count,
        s.sample_bodies.len(),
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
        s.ecliptic_count,
        s.equatorial_count,
        parity,
    )
}

/// Compact release-facing summary line for the selected-asteroid 2001-01-08
/// source evidence. Verbatim copy of
/// `SelectedAsteroidSource2451917Summary::summary_line`
/// (data/selected_asteroid_2001.rs:95). The struct is checked-in data and
/// stays in jpl `data/` permanently; only its rendering moves.
pub(crate) fn selected_asteroid_source_2451917_summary_line(
    s: &SelectedAsteroidSource2451917Summary,
) -> String {
    format!(
        "Reference selected-asteroid 2001-01-08 source evidence: {} exact samples at {} ({}); 2001-01-08 source sample",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the selected-asteroid 2378498.5
/// source evidence. Verbatim copy of
/// `SelectedAsteroidSource2378498Summary::summary_line`
/// (data/selected_asteroid_2378498.rs:95). The struct is checked-in data and
/// stays in jpl `data/` permanently; only its rendering moves.
pub(crate) fn selected_asteroid_source_2378498_summary_line(
    s: &SelectedAsteroidSource2378498Summary,
) -> String {
    format!(
        "Reference selected-asteroid 2378498.5 source evidence: {} exact samples at {} ({}); 2378498.5 source sample",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Returns the release-facing expanded selected-asteroid source coverage
/// summary string. Verbatim copy of jpl's
/// `selected_asteroid_source_evidence_summary_for_report`
/// (reference_summary/selected_asteroid.rs:501), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local).
pub fn selected_asteroid_source_evidence_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_source_evidence_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_summary_line(&summary),
            Err(error) => format!("Selected asteroid source evidence: unavailable ({error})"),
        },
        None => "Selected asteroid source evidence: unavailable".to_string(),
    }
}

/// Returns the validated release-facing expanded selected-asteroid source
/// coverage summary string. Verbatim copy of jpl's
/// `validated_selected_asteroid_source_evidence_summary_for_report`
/// (reference_summary/selected_asteroid.rs:512).
pub(crate) fn validated_selected_asteroid_source_evidence_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::selected_asteroid_source_evidence_summary()
        .ok_or_else(|| "selected asteroid source evidence unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(selected_asteroid_source_summary_line(&summary))
}

/// Returns the release-facing selected-asteroid source-window summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_source_window_summary_for_report`
/// (reference_summary/selected_asteroid.rs:521).
pub fn selected_asteroid_source_window_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_source_window_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_window_summary_line(&summary),
            Err(error) => format!("Selected asteroid source windows: unavailable ({error})"),
        },
        None => "Selected asteroid source windows: unavailable".to_string(),
    }
}

/// Returns the validated release-facing selected-asteroid source-window
/// summary string. Verbatim copy of jpl's
/// `validated_selected_asteroid_source_window_summary_for_report`
/// (reference_summary/selected_asteroid.rs:532).
pub(crate) fn validated_selected_asteroid_source_window_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::selected_asteroid_source_window_summary()
        .ok_or_else(|| "selected asteroid source windows unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(selected_asteroid_source_window_summary_line(&summary))
}

/// Formats the selected-asteroid source request corpus for release-facing
/// reporting. Verbatim copy of jpl's
/// `format_selected_asteroid_source_request_corpus_summary`
/// (reference_summary/selected_asteroid.rs:778), with `summary.summary_line()`
/// (Display-equivalent) rewired to the local render fn. Not one of the 16
/// enumerated `_for_report` renderers, but copied for completeness since Task
/// 11 is the last renderer-copy family task and this file's entire rendering
/// surface must be reproducible before the Task 14 contract sweep.
pub(crate) fn format_selected_asteroid_source_request_corpus_summary(
    summary: &SelectedAsteroidSourceRequestCorpusSummary,
) -> String {
    selected_asteroid_source_request_corpus_summary_line(summary)
}

/// Returns the release-facing selected-asteroid source request corpus
/// summary string for the requested frame. Verbatim copy of jpl's
/// `selected_asteroid_source_request_corpus_summary_for_frame`
/// (reference_summary/selected_asteroid.rs:785). Not one of the 16
/// `_for_report`-suffixed renderers (it is the same-file helper the four
/// `..._for_report`/`..._equatorial_summary_for_report` fns below delegate
/// to), copied because it renders.
pub(crate) fn selected_asteroid_source_request_corpus_summary_for_frame(
    frame: CoordinateFrame,
) -> String {
    match pleiades_jpl::selected_asteroid_source_request_corpus_summary(frame) {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_request_corpus_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid source request corpus: unavailable ({error})")
            }
        },
        None => "Selected asteroid source request corpus: unavailable".to_string(),
    }
}

/// Returns the validated release-facing selected-asteroid source request
/// corpus summary string for the requested frame. Verbatim copy of jpl's
/// `validated_selected_asteroid_source_request_corpus_summary_for_frame`
/// (reference_summary/selected_asteroid.rs:798).
pub(crate) fn validated_selected_asteroid_source_request_corpus_summary_for_frame(
    frame: CoordinateFrame,
) -> Result<String, String> {
    let summary = pleiades_jpl::selected_asteroid_source_request_corpus_summary(frame)
        .ok_or_else(|| "selected asteroid source request corpus unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(selected_asteroid_source_request_corpus_summary_line(
        &summary,
    ))
}

/// Returns the release-facing selected-asteroid source request corpus
/// summary string. Verbatim copy of jpl's
/// `selected_asteroid_source_request_corpus_summary_for_report`
/// (reference_summary/selected_asteroid.rs:809), with the cross-frame call
/// rewired to the local `selected_asteroid_source_request_corpus_summary_for_frame`
/// (same-file helper).
pub fn selected_asteroid_source_request_corpus_summary_for_report() -> String {
    selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Ecliptic)
}

/// Returns the validated release-facing selected-asteroid source request
/// corpus summary string. Verbatim copy of jpl's
/// `validated_selected_asteroid_source_request_corpus_summary_for_report`
/// (reference_summary/selected_asteroid.rs:814).
pub(crate) fn validated_selected_asteroid_source_request_corpus_summary_for_report(
) -> Result<String, String> {
    validated_selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Ecliptic)
}

/// Returns the release-facing equatorial selected-asteroid source request
/// corpus summary string. Verbatim copy of jpl's
/// `selected_asteroid_source_request_corpus_equatorial_summary_for_report`
/// (reference_summary/selected_asteroid.rs:820).
pub fn selected_asteroid_source_request_corpus_equatorial_summary_for_report() -> String {
    selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Equatorial)
}

/// Returns the validated release-facing equatorial selected-asteroid source
/// request corpus summary string. Verbatim copy of jpl's
/// `validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report`
/// (reference_summary/selected_asteroid.rs:825).
pub(crate) fn validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report(
) -> Result<String, String> {
    validated_selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Equatorial)
}

/// Returns the release-facing selected-asteroid 2003-12-27 source summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_source_2453000_summary_for_report`
/// (reference_summary/selected_asteroid.rs:1030).
pub fn selected_asteroid_source_2453000_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_source_2453000_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_2453000_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid 2003-12-27 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2003-12-27 source evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing selected-asteroid 2500000 source summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_source_2500000_summary_for_report`
/// (reference_summary/selected_asteroid.rs:1242).
pub fn selected_asteroid_source_2500000_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_source_2500000_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_2500000_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid 2500000 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2500000 source evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing selected-asteroid 2634167 source summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_source_2634167_summary_for_report`
/// (reference_summary/selected_asteroid.rs:1454).
pub fn selected_asteroid_source_2634167_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_source_2634167_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_2634167_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid 2634167 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2634167 source evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing selected-asteroid bridge-day summary string.
/// Verbatim copy of jpl's `selected_asteroid_bridge_summary_for_report`
/// (reference_summary/selected_asteroid.rs:1651).
pub fn selected_asteroid_bridge_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_bridge_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_bridge_summary_line(&summary),
            Err(error) => format!("Selected asteroid bridge evidence: unavailable ({error})"),
        },
        None => "Selected asteroid bridge evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing dense selected-asteroid boundary summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_dense_boundary_summary_for_report`
/// (reference_summary/selected_asteroid.rs:1847).
pub fn selected_asteroid_dense_boundary_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_dense_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_dense_boundary_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid dense boundary evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid dense boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing selected-asteroid boundary-day summary string.
/// Verbatim copy of jpl's `selected_asteroid_boundary_summary_for_report`
/// (reference_summary/selected_asteroid.rs:2081).
pub fn selected_asteroid_boundary_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_boundary_summary_line(&summary),
            Err(error) => format!("Selected asteroid boundary evidence: unavailable ({error})"),
        },
        None => "Selected asteroid boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing terminal selected-asteroid boundary summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_terminal_boundary_summary_for_report`
/// (reference_summary/selected_asteroid.rs:2293).
pub fn selected_asteroid_terminal_boundary_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_terminal_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_terminal_boundary_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid terminal boundary evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid terminal boundary evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing selected-asteroid batch-parity summary string.
/// Verbatim copy of jpl's `selected_asteroid_batch_parity_summary_for_report`
/// (reference_summary/selected_asteroid.rs:2626).
pub fn selected_asteroid_batch_parity_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_batch_parity_summary_line(&summary),
            Err(error) => format!("Selected asteroid batch parity: unavailable ({error})"),
        },
        None => "Selected asteroid batch parity: unavailable".to_string(),
    }
}

/// Release-facing posture line: the curated asteroids form a constrained body
/// class advertised over the 1900–2100 asteroid window. Tier A bodies are
/// sourced from pinned-kernel + per-object SPK (reproducible); Tier B is the
/// Horizons-sourced constrained set (empty after slice-3 promotion).
/// Kept distinct from release-grade major-body claims. Verbatim copy of jpl's
/// `selected_asteroid_constrained_class_report`
/// (reference_summary/selected_asteroid.rs:2641). Not one of the 16
/// `_for_report` renderers (different suffix), copied for completeness — see
/// module doc.
pub(crate) fn selected_asteroid_constrained_class_report() -> String {
    use pleiades_jpl::spk::asteroid_roster::{tier_a_bodies, tier_b_bodies};
    use pleiades_jpl::spk::corpus_spec::{AST_RANGE_END_JD, AST_RANGE_START_JD};
    format!(
        "Curated asteroids are a constrained class advertised over JD {:.1}\u{2013}{:.1} (1900\u{2013}2100 CE): \
         {} Tier A pinned-kernel + per-object SPK (reproducible) + {} Tier B Horizons-constrained bodies; \
         excluded from release-grade major-body claims.",
        AST_RANGE_START_JD,
        AST_RANGE_END_JD,
        tier_a_bodies().len(),
        tier_b_bodies().len(),
    )
}

/// Returns the release-facing selected-asteroid 2001-01-08 source summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_source_2451917_summary_for_report`
/// (data/selected_asteroid_2001.rs:198).
pub fn selected_asteroid_source_2451917_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_source_2451917_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_2451917_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid 2001-01-08 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2001-01-08 source evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing selected-asteroid 2378498.5 source summary
/// string. Verbatim copy of jpl's
/// `selected_asteroid_source_2378498_summary_for_report`
/// (data/selected_asteroid_2378498.rs:198).
pub fn selected_asteroid_source_2378498_summary_for_report() -> String {
    match pleiades_jpl::selected_asteroid_source_2378498_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => selected_asteroid_source_2378498_summary_line(&summary),
            Err(error) => {
                format!("Selected asteroid 2378498.5 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2378498.5 source evidence: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_asteroid_source_evidence_summary_reports_the_expanded_coverage() {
        let summary = pleiades_jpl::selected_asteroid_source_evidence_summary()
            .expect("selected asteroid source evidence summary should exist");
        assert_eq!(
            selected_asteroid_source_summary_line(&summary),
            "Selected asteroid source evidence: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"
        );
        assert_eq!(
            selected_asteroid_source_summary_line(&summary),
            selected_asteroid_source_evidence_summary_for_report()
        );
        assert_eq!(
            validated_selected_asteroid_source_evidence_summary_for_report(),
            Ok(selected_asteroid_source_summary_line(&summary))
        );
    }

    #[test]
    fn selected_asteroid_source_window_summary_reports_the_body_windows() {
        let summary = pleiades_jpl::selected_asteroid_source_window_summary()
            .expect("selected asteroid source window summary should exist");
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(summary.sample_count, 95);
        assert_eq!(summary.epoch_count, 17);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_source_window_summary_line(&summary),
            "Selected asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"
        );
        assert_eq!(
            selected_asteroid_source_window_summary_line(&summary),
            selected_asteroid_source_window_summary_for_report()
        );
        assert_eq!(
            validated_selected_asteroid_source_window_summary_for_report(),
            Ok(selected_asteroid_source_window_summary_line(&summary))
        );
    }

    #[test]
    fn selected_asteroid_source_request_corpus_summary_reports_the_frame_specific_request_slice() {
        let summary = pleiades_jpl::selected_asteroid_source_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )
        .expect("selected asteroid source request corpus summary should exist");
        assert_eq!(summary.request_count, 95);
        assert_eq!(summary.body_count, 6);
        assert_eq!(summary.epoch_count, 17);
        assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_source_request_corpus_summary_for_report(),
            selected_asteroid_source_request_corpus_summary_line(&summary)
        );
        assert_eq!(
            validated_selected_asteroid_source_request_corpus_summary_for_report(),
            Ok(selected_asteroid_source_request_corpus_summary_line(
                &summary
            ))
        );
        assert_eq!(
            selected_asteroid_source_request_corpus_equatorial_summary_for_report(),
            selected_asteroid_source_request_corpus_summary_line(
                &pleiades_jpl::selected_asteroid_source_request_corpus_summary(
                    CoordinateFrame::Equatorial
                )
                .expect("selected asteroid source request corpus equatorial summary should exist")
            )
        );
        assert_eq!(
            validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report(),
            Ok(selected_asteroid_source_request_corpus_summary_line(
                &pleiades_jpl::selected_asteroid_source_request_corpus_summary(
                    CoordinateFrame::Equatorial
                )
                .expect("selected asteroid source request corpus equatorial summary should exist")
            ))
        );
        assert!(
            selected_asteroid_source_request_corpus_summary_line(&summary)
                .contains("observerless) across 6 bodies and 17 epochs")
        );
        assert_eq!(
            format_selected_asteroid_source_request_corpus_summary(&summary),
            selected_asteroid_source_request_corpus_summary_line(&summary)
        );
    }

    #[test]
    fn selected_asteroid_source_2453000_summary_reports_the_2003_source_slice() {
        let summary = pleiades_jpl::selected_asteroid_source_2453000_summary()
            .expect("selected asteroid 2003-12-27 source summary should exist");
        assert_eq!(summary.sample_count, 6);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_source_2453000_summary_line(&summary),
            "Reference selected-asteroid 2003-12-27 source evidence: 6 exact samples at JD 2453000.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2003-12-27 source sample"
        );
        assert_eq!(
            selected_asteroid_source_2453000_summary_line(&summary),
            selected_asteroid_source_2453000_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_source_2500000_summary_reports_the_late_boundary_slice() {
        let summary = pleiades_jpl::selected_asteroid_source_2500000_summary()
            .expect("selected asteroid 2500000 source summary should exist");
        assert_eq!(summary.sample_count, 6);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_source_2500000_summary_line(&summary),
            "Reference selected-asteroid 2500000 source evidence: 6 exact samples at JD 2500000.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2500000 source sample"
        );
        assert_eq!(
            selected_asteroid_source_2500000_summary_line(&summary),
            selected_asteroid_source_2500000_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_source_2634167_summary_reports_the_outer_boundary_slice() {
        let summary = pleiades_jpl::selected_asteroid_source_2634167_summary()
            .expect("selected asteroid 2634167 source summary should exist");
        assert_eq!(summary.sample_count, 6);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_source_2634167_summary_line(&summary),
            "Reference selected-asteroid 2634167 source evidence: 6 exact samples at JD 2634167.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2634167 source sample"
        );
        assert_eq!(
            selected_asteroid_source_2634167_summary_line(&summary),
            selected_asteroid_source_2634167_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_boundary_summary_reports_the_boundary_days() {
        let summary = pleiades_jpl::selected_asteroid_boundary_summary()
            .expect("selected asteroid boundary summary should exist");
        assert_eq!(summary.sample_count, 23);
        assert_eq!(summary.epochs.len(), 4);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_boundary_summary_line(&summary),
            "Selected asteroid boundary evidence: 23 exact samples across 4 epochs at JD 2451914.5 (TDB)..JD 2451919.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        );
        assert_eq!(
            selected_asteroid_boundary_summary_line(&summary),
            selected_asteroid_boundary_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_bridge_summary_reports_the_bridge_day() {
        let summary = pleiades_jpl::selected_asteroid_bridge_summary()
            .expect("selected asteroid bridge summary should exist");
        assert_eq!(summary.sample_count, 6);
        assert_eq!(
            summary.sample_bodies,
            pleiades_jpl::reference_asteroids().to_vec()
        );
        assert_eq!(summary.epoch.julian_day.days(), 2_451_915.0);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_bridge_summary_line(&summary),
            "Selected asteroid bridge evidence: 6 exact samples at JD 2451915.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); bridge sample across the asteroid-only gap"
        );
        assert_eq!(
            selected_asteroid_bridge_summary_for_report(),
            selected_asteroid_bridge_summary_line(&summary)
        );
    }

    #[test]
    fn selected_asteroid_dense_boundary_summary_reports_the_dense_boundary_day() {
        let summary = pleiades_jpl::selected_asteroid_dense_boundary_summary()
            .expect("selected asteroid dense boundary summary should exist");
        assert_eq!(summary.sample_count, 5);
        assert_eq!(
            summary.sample_bodies,
            pleiades_jpl::reference_asteroids().to_vec()
        );
        assert_eq!(summary.epoch.julian_day.days(), 2_451_916.5);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_dense_boundary_summary_line(&summary),
            "Selected asteroid dense boundary evidence: 5 exact samples at JD 2451916.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); dense boundary day"
        );
        assert_eq!(
            selected_asteroid_dense_boundary_summary_for_report(),
            selected_asteroid_dense_boundary_summary_line(&summary)
        );
    }

    #[test]
    fn selected_asteroid_terminal_boundary_summary_reports_the_terminal_boundary_day() {
        let summary = pleiades_jpl::selected_asteroid_terminal_boundary_summary()
            .expect("selected asteroid terminal boundary summary should exist");
        assert_eq!(summary.sample_count, 6);
        assert_eq!(
            summary.sample_bodies,
            pleiades_jpl::reference_asteroids().to_vec()
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            selected_asteroid_terminal_boundary_summary_line(&summary),
            "Reference selected-asteroid terminal boundary evidence: 6 exact samples at JD 2500000.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2500-01-01 terminal boundary sample"
        );
        assert_eq!(
            selected_asteroid_terminal_boundary_summary_line(&summary),
            selected_asteroid_terminal_boundary_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_batch_parity_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::selected_asteroid_batch_parity_summary()
            .expect("selected asteroid batch parity summary should exist");
        assert_eq!(summary.request_count, 6);
        assert_eq!(
            summary.sample_bodies,
            pleiades_jpl::reference_asteroids().to_vec()
        );
        assert_eq!(summary.ecliptic_count, 3);
        assert_eq!(summary.equatorial_count, 3);
        assert!(summary.parity_preserved);
        summary
            .validate()
            .expect("selected asteroid batch parity summary should validate");
        assert_eq!(
            selected_asteroid_batch_parity_summary_line(&summary),
            "Selected asteroid batch parity: 6 requests across 6 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); frame mix: 3 ecliptic, 3 equatorial; batch/single parity preserved"
        );
        assert_eq!(
            selected_asteroid_batch_parity_summary_line(&summary),
            selected_asteroid_batch_parity_summary_for_report()
        );
    }

    #[test]
    fn constrained_class_report_states_window_and_tiers() {
        let r = selected_asteroid_constrained_class_report();
        assert!(r.contains("1900\u{2013}2100"));
        assert!(r.contains("constrained class"));
        assert!(r.contains("Tier A"));
        assert!(r.contains("Tier B"));
    }

    #[test]
    fn selected_asteroid_source_2451917_summary_reports_the_2001_source_slice() {
        let summary = pleiades_jpl::selected_asteroid_source_2451917_summary()
            .expect("selected asteroid 2001-01-08 source summary should exist");
        assert!(summary.sample_count > 0);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_917.5);
        assert_eq!(summary.validate(), Ok(()));
        assert!(selected_asteroid_source_2451917_summary_line(&summary)
            .contains("Reference selected-asteroid 2001-01-08 source evidence:"));
        assert!(
            selected_asteroid_source_2451917_summary_line(&summary).contains("JD 2451917.5 (TDB)")
        );
        assert_eq!(
            selected_asteroid_source_2451917_summary_line(&summary),
            selected_asteroid_source_2451917_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_source_2378498_summary_reports_the_bridge_slice() {
        let summary = pleiades_jpl::selected_asteroid_source_2378498_summary()
            .expect("selected asteroid 2378498.5 source summary should exist");
        assert_eq!(summary.sample_count, 6);
        assert_eq!(summary.epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.validate(), Ok(()));
        assert!(selected_asteroid_source_2378498_summary_line(&summary)
            .contains("Reference selected-asteroid 2378498.5 source evidence:"));
        assert!(
            selected_asteroid_source_2378498_summary_line(&summary).contains("JD 2378498.5 (TDB)")
        );
        assert_eq!(
            selected_asteroid_source_2378498_summary_line(&summary),
            selected_asteroid_source_2378498_summary_for_report()
        );
    }
}
