//! Relocated jpl-posture renderers copied from
//! `pleiades-jpl::reference_summary::jpl_posture` (report-surface relocation
//! program, Slice D). Rendering only — the functional crate keeps the
//! structured evidence structs, their `*_details()` constructors,
//! `validate()`/`label()` methods, and all release-gate data; jpl's own
//! rendering for these renderers was deleted in the Task 14b contract
//! sweep (aside from the six-function Option-A island documented in
//! `CHANGELOG.md`).
//!
//! This file is the AGGREGATOR: several of its renderers compose many other
//! files' renderers. Every call to a renderer defined in another jpl module
//! (`comparison_*`, `holdout_*`/`independent_holdout_*`/`reference_holdout_*`,
//! `reference_snapshot_*`, `reference_asteroid_*`, `production_generation_*`,
//! `selected_asteroid_*`) was originally left as `pleiades_jpl::<name>()`
//! uniformly, even for modules already copied elsewhere under
//! `posture/jpl/` — this was byte-identical (validate→jpl is allowed) and
//! Task 13's self-contain pass flipped the whole set to the local
//! equivalents at once. Only same-file (this file's own) calls are wired to
//! the local re-homed functions below.

use pleiades_jpl::{
    InterpolationQualitySampleRequestCorpusSummary, JplInterpolationBodyClassErrorEnvelopeSummary,
    JplInterpolationPostureSummary, JplInterpolationQualityKindCoverage,
    JplInterpolationQualitySourceSummary, JplInterpolationQualitySummary, JplProvenanceOnlySummary,
    JplSnapshotBatchErrorTaxonomySummary, JplSnapshotEvidenceClassificationSummary,
    JplSnapshotRequestPolicy, JplSourceCorpusContractSummary, JplSourcePostureSummary,
};

// Slice D Task 13: reach the other jpl-render submodules' free functions
// (`comparison_*`, `holdout_*`, `production_generation_*`, `reference_*`,
// `selected_asteroid_*`) without a `pleiades_jpl::` detour. See
// `posture/jpl/mod.rs` for the glob re-exports this resolves through.
#[allow(unused_imports)]
use crate::posture::jpl::*;

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Reproduced from jpl's private (`pub(crate)`, not callable cross-crate)
/// `format_bodies` helper
/// (`reference_summary/reference_snapshot/core/general_a.rs:509`).
fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Reproduced from jpl's private `join_display` generic helper
/// (`reference_summary/reference_snapshot/core/general_a.rs:500`), backing
/// the four `format_*` wrappers below
/// (`reference_summary/reference_snapshot/core/general_a.rs:514-528`).
fn join_display<T: std::fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Reproduced from jpl's private `format_coordinate_frames`
/// (`reference_summary/reference_snapshot/core/general_a.rs:514`).
fn format_coordinate_frames(frames: &[pleiades_types::CoordinateFrame]) -> String {
    join_display(frames)
}

/// Reproduced from jpl's private `format_time_scales`
/// (`reference_summary/reference_snapshot/core/general_a.rs:518`).
fn format_time_scales(time_scales: &[pleiades_types::TimeScale]) -> String {
    join_display(time_scales)
}

/// Reproduced from jpl's private `format_zodiac_modes`
/// (`reference_summary/reference_snapshot/core/general_a.rs:522`).
fn format_zodiac_modes(zodiac_modes: &[pleiades_types::ZodiacMode]) -> String {
    join_display(zodiac_modes)
}

/// Reproduced from jpl's private `format_apparentness_modes`
/// (`reference_summary/reference_snapshot/core/general_a.rs:526`).
fn format_apparentness_modes(modes: &[pleiades_types::Apparentness]) -> String {
    join_display(modes)
}

/// Reproduced from jpl's private `strip_report_prefix`
/// (`reference_summary/reference_snapshot/core/general_a.rs:447`).
fn strip_report_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

// ---------------------------------------------------------------------------
// Re-homed inherent `summary_line` renderers (one free fn per evidence struct).
// ---------------------------------------------------------------------------

/// Verbatim copy of `JplSnapshotEvidenceClassificationSummary::summary_line`
/// (reference_summary/jpl_posture.rs:73).
pub(crate) fn jpl_snapshot_evidence_classification_summary_line(
    s: &JplSnapshotEvidenceClassificationSummary,
) -> String {
    s.text.to_string()
}

/// Verbatim copy of `JplSourcePostureSummary::summary_line`
/// (reference_summary/jpl_posture.rs:156).
pub(crate) fn jpl_source_posture_summary_line(s: &JplSourcePostureSummary) -> String {
    s.text.to_string()
}

/// Verbatim copy of `JplProvenanceOnlySummary::summary_line`
/// (reference_summary/jpl_posture.rs:232).
pub(crate) fn jpl_provenance_only_summary_line(s: &JplProvenanceOnlySummary) -> String {
    s.text.to_string()
}

/// Verbatim copy of `JplSourceCorpusContractSummary::summary_line`
/// (reference_summary/jpl_posture.rs:324), with the nested
/// `JplSnapshotEvidenceClassificationSummary::summary_line`/
/// `JplSourcePostureSummary::summary_line` calls rewired to the local
/// `jpl_snapshot_evidence_classification_summary_line`/
/// `jpl_source_posture_summary_line` (same-file evidence structs). The
/// remaining nested calls (`reference_summary`, `boundary_summary`,
/// `source_windows`, `source_revision`, `boundary_request_corpus_*`) are on
/// evidence structs owned by OTHER jpl modules; Task 13 rewired them to the
/// local re-homed renderers (`reference_snapshot_source_summary_line`,
/// `independent_holdout_source_summary_summary_line`,
/// `production_generation_snapshot_window_summary_line`,
/// `production_generation_source_revision_summary_line`,
/// `production_generation_boundary_request_corpus_summary_line`).
pub(crate) fn jpl_source_corpus_contract_summary_line(
    s: &JplSourceCorpusContractSummary,
) -> String {
    format!(
        "JPL source corpus contract: {}; {}; reference={}; hold-out={}; source windows={}; source revision={}; boundary request corpora: ecliptic={}; equatorial={}",
        jpl_snapshot_evidence_classification_summary_line(&s.evidence_classification),
        jpl_source_posture_summary_line(&s.source_posture),
        reference_snapshot_source_summary_line(&s.reference_summary),
        independent_holdout_source_summary_summary_line(&s.boundary_summary),
        strip_report_prefix(
            &production_generation_snapshot_window_summary_line(&s.source_windows),
            "Production generation source windows: ",
        ),
        production_generation_source_revision_summary_line(&s.source_revision),
        strip_report_prefix(
            &production_generation_boundary_request_corpus_summary_line(
                &s.boundary_request_corpus_ecliptic
            ),
            "Production generation boundary request corpus: ",
        ),
        strip_report_prefix(
            &production_generation_boundary_request_corpus_summary_line(
                &s.boundary_request_corpus_equatorial
            ),
            "Production generation boundary request corpus: ",
        ),
    )
}

/// Verbatim copy of `JplSnapshotRequestPolicy::summary_line`
/// (reference_summary/jpl_posture.rs:583).
pub(crate) fn jpl_snapshot_request_policy_summary_line(s: &JplSnapshotRequestPolicy) -> String {
    format!(
        "frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}",
        format_coordinate_frames(s.supported_frames),
        format_time_scales(s.supported_time_scales),
        format_zodiac_modes(s.supported_zodiac_modes),
        format_apparentness_modes(s.supported_apparentness),
        s.supports_topocentric_observer,
    )
}

/// Verbatim copy of `JplSnapshotBatchErrorTaxonomySummary::summary_line`
/// (reference_summary/jpl_posture.rs:695).
pub(crate) fn jpl_snapshot_batch_error_taxonomy_summary_line(
    s: &JplSnapshotBatchErrorTaxonomySummary,
) -> String {
    format!(
        "JPL batch error taxonomy: supported body {}; unsupported body {} -> {}; out-of-range {} -> {}",
        s.supported_request_body,
        s.unsupported_request_body,
        s.unsupported_error_kind,
        s.out_of_range_request_body,
        s.out_of_range_error_kind,
    )
}

/// Verbatim copy of
/// `InterpolationQualitySampleRequestCorpusSummary::summary_line`
/// (reference_summary/jpl_posture.rs:994).
pub(crate) fn interpolation_quality_sample_request_corpus_summary_line(
    s: &InterpolationQualitySampleRequestCorpusSummary,
) -> String {
    format!(
        "Interpolation-quality sample request corpus: {} requests (frame={}; time scale={}; zodiac mode={}; apparentness={}; observerless) across {} bodies and {} epochs ({}..{}); bodies: {}",
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

/// Verbatim copy of `JplInterpolationQualitySummary::summary_line`
/// (reference_summary/jpl_posture.rs:1266).
pub(crate) fn jpl_interpolation_quality_summary_line(s: &JplInterpolationQualitySummary) -> String {
    fn format_body_epoch_suffix(body: &str, epoch: pleiades_types::Instant) -> String {
        if body.is_empty() {
            String::new()
        } else {
            format!(" ({body} @ {})", format_instant(epoch))
        }
    }

    format!(
        "JPL interpolation quality: {} samples across {} bodies and {} epochs ({} cubic, {} quadratic, {} linear), epoch window {} → {}; leave-one-out runtime interpolation evidence with worst-case bodies named, max bracket span={:.1} d{}; mean bracket span={:.1} d; median bracket span={:.1} d; p95 bracket span={:.1} d; max Δlon={:.12}°{}; mean Δlon={:.12}°; median Δlon={:.12}°; p95 Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}°{}; mean Δlat={:.12}°; median Δlat={:.12}°; p95 Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU{}; mean Δdist={:.12} AU; median Δdist={:.12} AU; p95 Δdist={:.12} AU; rms Δdist={:.12} AU; transparency evidence only, not a production tolerance envelope",
        s.sample_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        s.cubic_sample_count,
        s.quadratic_sample_count,
        s.linear_sample_count,
        s.max_bracket_span_days,
        format_body_epoch_suffix(&s.max_bracket_span_body, s.max_bracket_span_epoch),
        s.mean_bracket_span_days,
        s.median_bracket_span_days,
        s.percentile_bracket_span_days,
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

/// Verbatim copy of `JplInterpolationPostureSummary::summary_line`
/// (reference_summary/jpl_posture.rs:1339).
pub(crate) fn jpl_interpolation_posture_summary_line(s: &JplInterpolationPostureSummary) -> String {
    format!(
        "JPL interpolation posture: source={}; detail={}; envelope={}",
        s.source, s.detail, s.envelope
    )
}

/// Verbatim copy of `JplInterpolationQualityKindCoverage::summary_line`
/// (reference_summary/jpl_posture.rs:1870).
pub(crate) fn jpl_interpolation_quality_kind_coverage_line(
    s: &JplInterpolationQualityKindCoverage,
) -> String {
    let bodies = if s.bodies.is_empty() {
        "none".to_string()
    } else {
        s.bodies.join(", ")
    };

    format!(
        "JPL interpolation quality kind coverage: {} samples across {} bodies [{}] ({} cubic bodies, {} quadratic bodies, {} linear bodies)",
        s.sample_count,
        s.body_count,
        bodies,
        s.cubic_body_count,
        s.quadratic_body_count,
        s.linear_body_count,
    )
}

/// Verbatim copy of `JplInterpolationQualitySourceSummary::summary_line`
/// (reference_summary/jpl_posture.rs:2014).
pub(crate) fn jpl_interpolation_quality_source_summary_line(
    s: &JplInterpolationQualitySourceSummary,
) -> String {
    format!(
        "JPL interpolation quality source: {}; derivation={}; coverage: {} samples across {} bodies and {} epochs",
        s.source, s.derivation, s.sample_count, s.body_count, s.epoch_count,
    )
}

/// Verbatim copy of `JplInterpolationBodyClassErrorEnvelopeSummary::summary_line`
/// (reference_summary/jpl_posture.rs:2222, private in jpl).
pub(crate) fn jpl_interpolation_body_class_error_envelope_summary_line(
    s: &JplInterpolationBodyClassErrorEnvelopeSummary,
) -> String {
    format!(
        "JPL interpolation body-class error envelope: {}: {} samples across {} bodies [{}] and {} epochs ({} → {}); max Δlon={:.12}° ({} @ {}); mean Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}° ({} @ {}); mean Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU ({} @ {}); mean Δdist={:.12} AU; rms Δdist={:.12} AU",
        s.class,
        s.sample_count,
        s.body_count,
        s.bodies.join(", "),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        s.max_longitude_error_deg,
        s.max_longitude_error_body,
        format_instant(s.max_longitude_error_epoch),
        s.mean_longitude_error_deg,
        s.rms_longitude_error_deg,
        s.max_latitude_error_deg,
        s.max_latitude_error_body,
        format_instant(s.max_latitude_error_epoch),
        s.mean_latitude_error_deg,
        s.rms_latitude_error_deg,
        s.max_distance_error_au,
        s.max_distance_error_body,
        format_instant(s.max_distance_error_epoch),
        s.mean_distance_error_au,
        s.rms_distance_error_au,
    )
}

// ---------------------------------------------------------------------------
// The 16 free `*_for_report` renderers, copied verbatim (validate()/gates stay
// in jpl and are called cross-crate; rendering is local).
// ---------------------------------------------------------------------------

/// Returns the schema shared by the checked-in snapshot fixtures after
/// validating the manifests. Verbatim copy of jpl's
/// `checked_in_snapshot_schema_summary_for_report`
/// (reference_summary/jpl_posture.rs:29).
/// `validated_checked_in_snapshot_schema_summary` is a validation gate (not
/// rendering); it was promoted from `pub(crate)` to `pub` in jpl (Slice D
/// Task 6) so it can be called cross-crate instead of reproduced.
pub(crate) fn checked_in_snapshot_schema_summary_for_report() -> String {
    match pleiades_jpl::validated_checked_in_snapshot_schema_summary() {
        Ok(schema) => format!("Checked-in snapshot schema: {schema}"),
        Err(error) => format!("Checked-in snapshot schema: unavailable ({error})"),
    }
}

/// Returns the validated schema shared by the checked-in snapshot fixtures.
/// Verbatim copy of jpl's
/// `validated_checked_in_snapshot_schema_summary_for_report`
/// (reference_summary/jpl_posture.rs:37).
pub(crate) fn validated_checked_in_snapshot_schema_summary_for_report() -> Result<String, String> {
    pleiades_jpl::validated_checked_in_snapshot_schema_summary().map(str::to_string)
}

/// Returns the validated evidence-classification line used by validation and
/// release reports. Verbatim copy of jpl's
/// `jpl_snapshot_evidence_classification_summary_for_report`
/// (reference_summary/jpl_posture.rs:116).
pub(crate) fn jpl_snapshot_evidence_classification_summary_for_report() -> String {
    let summary = pleiades_jpl::jpl_snapshot_evidence_classification_summary_details();
    match summary.validate() {
        Ok(()) => jpl_snapshot_evidence_classification_summary_line(&summary),
        Err(error) => format!("JPL evidence classification: unavailable ({error})"),
    }
}

/// Returns the validated source-posture line used by validation and release
/// reports. Verbatim copy of jpl's `jpl_source_posture_summary_for_report`
/// (reference_summary/jpl_posture.rs:192).
pub fn jpl_source_posture_summary_for_report() -> String {
    let summary = pleiades_jpl::jpl_source_posture_summary_details();
    match summary.validate() {
        Ok(()) => jpl_source_posture_summary_line(&summary),
        Err(error) => format!("JPL source posture: unavailable ({error})"),
    }
}

/// Returns the validated provenance-only line used by validation and release
/// reports. Verbatim copy of jpl's `jpl_provenance_only_summary_for_report`
/// (reference_summary/jpl_posture.rs:270).
pub fn jpl_provenance_only_summary_for_report() -> String {
    let summary = pleiades_jpl::jpl_provenance_only_summary_details();
    match summary.validate() {
        Ok(()) => jpl_provenance_only_summary_line(&summary),
        Err(error) => format!("JPL provenance-only evidence: unavailable ({error})"),
    }
}

/// Returns the validated source-corpus contract line used by validation and
/// release reports. Verbatim copy of jpl's
/// `jpl_source_corpus_contract_summary_for_report`
/// (reference_summary/jpl_posture.rs:467).
pub fn jpl_source_corpus_contract_summary_for_report() -> String {
    let summary = pleiades_jpl::jpl_source_corpus_contract_summary_details();
    match summary.validate() {
        Ok(()) => jpl_source_corpus_contract_summary_line(&summary),
        Err(error) => format!("JPL source corpus contract: unavailable ({error})"),
    }
}

/// Returns the combined snapshot evidence summary used by validation and
/// release reports. Verbatim copy of jpl's
/// `jpl_snapshot_evidence_summary_for_report`
/// (reference_summary/jpl_posture.rs:476). The three local calls
/// (`jpl_snapshot_evidence_classification_summary_for_report`,
/// `jpl_source_posture_summary_for_report`,
/// `jpl_provenance_only_summary_for_report`) resolve to this file's own
/// copies above; every other renderer in the list lives in a different jpl
/// module and is called as `pleiades_jpl::<name>()` (see the module doc for
/// the aggregator wiring rule).
pub(crate) fn jpl_snapshot_evidence_summary_for_report() -> String {
    [
        jpl_snapshot_evidence_classification_summary_for_report(),
        jpl_source_posture_summary_for_report(),
        jpl_provenance_only_summary_for_report(),
        reference_snapshot_summary_for_report(),
        reference_snapshot_2451910_major_body_boundary_summary_for_report(),
        reference_snapshot_2451911_major_body_boundary_summary_for_report(),
        reference_snapshot_2451912_major_body_boundary_summary_for_report(),
        reference_snapshot_2451913_major_body_boundary_summary_for_report(),
        reference_snapshot_2451914_major_body_boundary_summary_for_report(),
        reference_snapshot_2451915_major_body_boundary_summary_for_report(),
        reference_snapshot_bridge_day_summary_for_report(),
        reference_snapshot_2451914_major_body_bridge_day_summary_for_report(),
        reference_snapshot_2451914_major_body_bridge_summary_for_report(),
        reference_snapshot_2451916_major_body_interior_summary_for_report(),
        reference_snapshot_2451916_major_body_dense_boundary_summary_for_report(),
        reference_snapshot_2451917_major_body_boundary_summary_for_report(),
        reference_snapshot_2451917_major_body_bridge_summary_for_report(),
        reference_snapshot_mars_jupiter_boundary_summary_for_report(),
        reference_snapshot_2451918_major_body_boundary_summary_for_report(),
        reference_snapshot_2451919_major_body_boundary_summary_for_report(),
        reference_snapshot_2451920_major_body_interior_summary_for_report(),
        reference_snapshot_body_class_coverage_summary_for_report(),
        reference_snapshot_equatorial_parity_summary_for_report(),
        reference_snapshot_batch_parity_summary_for_report(),
        production_generation_snapshot_summary_for_report(),
        production_generation_source_summary_for_report(),
        reference_snapshot_source_summary_for_report(),
        reference_snapshot_source_window_summary_for_report(),
        reference_snapshot_boundary_epoch_coverage_summary_for_report(),
        reference_snapshot_sparse_boundary_summary_for_report(),
        reference_snapshot_major_body_boundary_summary_for_report(),
        reference_holdout_overlap_summary_for_report(),
        independent_holdout_high_curvature_summary_for_report(),
        reference_snapshot_manifest_summary_for_report(),
        production_generation_boundary_source_summary_for_report(),
        production_generation_boundary_window_summary_for_report(),
        production_generation_boundary_body_class_coverage_summary_for_report(),
        production_generation_boundary_request_corpus_summary_for_report(),
        production_generation_boundary_request_corpus_equatorial_summary_for_report(),
        reference_asteroid_evidence_summary_for_report(),
        reference_asteroid_equatorial_evidence_summary_for_report(),
        reference_asteroid_source_window_summary_for_report(),
        selected_asteroid_source_2451917_summary_for_report(),
        selected_asteroid_source_2453000_summary_for_report(),
        selected_asteroid_boundary_summary_for_report(),
        selected_asteroid_bridge_summary_for_report(),
        selected_asteroid_dense_boundary_summary_for_report(),
        selected_asteroid_terminal_boundary_summary_for_report(),
        comparison_snapshot_summary_for_report(),
        comparison_snapshot_body_class_coverage_summary_for_report(),
        comparison_snapshot_source_summary_for_report(),
        comparison_snapshot_source_window_summary_for_report(),
        comparison_snapshot_manifest_summary_for_report(),
        independent_holdout_snapshot_summary_for_report(),
        independent_holdout_snapshot_equatorial_parity_summary_for_report(),
        independent_holdout_snapshot_batch_parity_summary_for_report(),
        independent_holdout_source_summary_for_report(),
        independent_holdout_snapshot_source_window_summary_for_report(),
        independent_holdout_snapshot_quarter_day_boundary_summary_for_report(),
        independent_holdout_manifest_summary_for_report(),
        jpl_independent_holdout_summary_for_report(),
    ]
    .join(" | ")
}

/// Returns the release-facing JPL snapshot request policy summary string.
/// Verbatim copy of jpl's `jpl_snapshot_request_policy_summary_for_report`
/// (reference_summary/jpl_posture.rs:647).
pub(crate) fn jpl_snapshot_request_policy_summary_for_report() -> String {
    let policy = pleiades_jpl::jpl_snapshot_request_policy();
    match policy.validate() {
        Ok(()) => jpl_snapshot_request_policy_summary_line(&policy),
        Err(error) => format!("JPL snapshot request policy: unavailable ({error})"),
    }
}

/// Returns the release-facing batch error-taxonomy summary for the current
/// JPL snapshot backend. Verbatim copy of jpl's
/// `jpl_snapshot_batch_error_taxonomy_summary_for_report`
/// (reference_summary/jpl_posture.rs:858).
pub(crate) fn jpl_snapshot_batch_error_taxonomy_summary_for_report() -> String {
    match pleiades_jpl::jpl_snapshot_batch_error_taxonomy_summary() {
        Ok(summary) => match summary.validate() {
            Ok(()) => jpl_snapshot_batch_error_taxonomy_summary_line(&summary),
            Err(error) => format!("JPL batch error taxonomy: unavailable ({error})"),
        },
        Err(error) => format!("JPL batch error taxonomy: unavailable ({error})"),
    }
}

/// Returns the release-facing frame-treatment summary for the current JPL
/// snapshot backend. Verbatim copy of jpl's `frame_treatment_summary_for_report`
/// (reference_summary/jpl_posture.rs:885). `FrameTreatmentSummary` is a
/// `pleiades_backend` type, not one of jpl's own evidence structs, so its
/// `validated_summary_line()` stays as a direct call — only jpl's own
/// rendering moves.
pub(crate) fn frame_treatment_summary_for_report() -> String {
    let summary = pleiades_jpl::frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("JPL frame treatment unavailable ({error})"),
    }
}

/// Returns the release-facing interpolation-quality sample request corpus
/// summary string. Verbatim copy of jpl's
/// `interpolation_quality_sample_request_corpus_summary_for_report`
/// (reference_summary/jpl_posture.rs:1177).
pub fn interpolation_quality_sample_request_corpus_summary_for_report() -> String {
    match pleiades_jpl::interpolation_quality_sample_request_corpus_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => interpolation_quality_sample_request_corpus_summary_line(&summary),
            Err(error) => {
                format!("Interpolation-quality sample request corpus: unavailable ({error})")
            }
        },
        None => "Interpolation-quality sample request corpus: unavailable".to_string(),
    }
}

/// Returns the release-facing interpolation posture summary string. Verbatim
/// copy of jpl's `jpl_interpolation_posture_summary_for_report`
/// (reference_summary/jpl_posture.rs:1554).
pub(crate) fn jpl_interpolation_posture_summary_for_report() -> String {
    match pleiades_jpl::jpl_interpolation_posture_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => jpl_interpolation_posture_summary_line(&summary),
            Err(error) => format!("JPL interpolation posture: unavailable ({error})"),
        },
        None => "JPL interpolation posture: unavailable".to_string(),
    }
}

/// Returns the release-facing interpolation-kind coverage summary string.
/// Verbatim copy of jpl's `jpl_interpolation_quality_kind_coverage_for_report`
/// (reference_summary/jpl_posture.rs:1958).
pub(crate) fn jpl_interpolation_quality_kind_coverage_for_report() -> String {
    match pleiades_jpl::jpl_interpolation_quality_kind_coverage() {
        Some(coverage) => match coverage.validate() {
            Ok(()) => jpl_interpolation_quality_kind_coverage_line(&coverage),
            Err(_) => "JPL interpolation quality kind coverage: unavailable".to_string(),
        },
        None => "JPL interpolation quality kind coverage: unavailable".to_string(),
    }
}

/// Returns the release-facing interpolation-quality provenance summary
/// string. Verbatim copy of jpl's
/// `jpl_interpolation_quality_source_summary_for_report`
/// (reference_summary/jpl_posture.rs:2104).
pub(crate) fn jpl_interpolation_quality_source_summary_for_report() -> String {
    match pleiades_jpl::jpl_interpolation_quality_source_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => jpl_interpolation_quality_source_summary_line(&summary),
            Err(error) => format!("JPL interpolation quality source: unavailable ({error})"),
        },
        None => "JPL interpolation quality source: unavailable".to_string(),
    }
}

/// Formats the interpolation-quality summary together with the distinct-body
/// coverage and sample request corpus lines. Verbatim copy of jpl's
/// `format_jpl_interpolation_quality_summary_for_report`
/// (reference_summary/jpl_posture.rs:2116). The three trailing calls
/// (`jpl_interpolation_quality_kind_coverage_for_report`,
/// `interpolation_quality_sample_request_corpus_summary_for_report`,
/// `jpl_interpolation_body_class_error_envelopes_for_report`) are same-file
/// (jpl_posture) renderers and resolve to this file's own copies.
pub(crate) fn format_jpl_interpolation_quality_summary_for_report() -> String {
    let source_summary = match pleiades_jpl::jpl_interpolation_quality_source_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => jpl_interpolation_quality_source_summary_line(&summary),
            Err(_) => return "JPL interpolation quality: unavailable".to_string(),
        },
        None => return "JPL interpolation quality: unavailable".to_string(),
    };

    match pleiades_jpl::jpl_interpolation_quality_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => {
                let mut rendered = jpl_interpolation_quality_summary_line(&summary);
                rendered.insert_str(0, &format!("{}\n", source_summary));
                rendered.push('\n');
                rendered.push_str(&jpl_interpolation_quality_kind_coverage_for_report());
                rendered.push('\n');
                rendered
                    .push_str(&interpolation_quality_sample_request_corpus_summary_for_report());
                rendered.push('\n');
                rendered.push_str(&jpl_interpolation_body_class_error_envelopes_for_report());
                rendered
            }
            Err(_) => "JPL interpolation quality: unavailable".to_string(),
        },
        None => "JPL interpolation quality: unavailable".to_string(),
    }
}

/// Returns the release-facing body-class error envelopes for the
/// interpolation-quality samples. Verbatim copy of jpl's
/// `jpl_interpolation_body_class_error_envelopes_for_report`
/// (reference_summary/jpl_posture.rs:2482).
/// `JplInterpolationBodyClassErrorEnvelopeSummary::validate` was
/// `pub(crate)` in jpl; promoted to `pub` (Slice D Task 6) so it can be
/// called cross-crate instead of reproduced (it's a validation gate, not
/// rendering).
pub(crate) fn jpl_interpolation_body_class_error_envelopes_for_report() -> String {
    match pleiades_jpl::jpl_interpolation_body_class_error_envelopes() {
        Some(summaries) => {
            let mut rendered = String::from("JPL interpolation body-class error envelopes:");
            for summary in summaries {
                match summary.validate() {
                    Ok(()) => {
                        rendered.push('\n');
                        rendered.push_str(
                            &jpl_interpolation_body_class_error_envelope_summary_line(&summary),
                        );
                    }
                    Err(error) => {
                        return format!(
                            "JPL interpolation body-class error envelopes: unavailable ({error})"
                        )
                    }
                }
            }
            rendered
        }
        None => "JPL interpolation body-class error envelopes: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::EphemerisErrorKind;
    use pleiades_jpl::{
        InterpolationQualitySampleRequestCorpusSummaryValidationError,
        JplInterpolationBodyClassErrorEnvelopeSummaryValidationError,
        JplInterpolationPostureSummaryValidationError,
        JplInterpolationQualitySourceSummaryValidationError,
        JplInterpolationQualitySummaryValidationError, JplSnapshotBatchErrorTaxonomySummary,
        JplSnapshotBatchErrorTaxonomySummaryValidationError,
        JplSnapshotRequestPolicyValidationError,
    };
    use pleiades_types::{Apparentness, CoordinateFrame, TimeScale, ZodiacMode};

    #[test]
    fn checked_in_snapshot_schema_summary_for_report_reports_the_shared_schema() {
        assert_eq!(
            checked_in_snapshot_schema_summary_for_report(),
            "Checked-in snapshot schema: epoch_jd, body, x_km, y_km, z_km"
        );
        assert_eq!(
            validated_checked_in_snapshot_schema_summary_for_report(),
            Ok("epoch_jd, body, x_km, y_km, z_km".to_string())
        );
    }

    #[test]
    fn jpl_snapshot_evidence_summary_combines_the_backend_reports() {
        let report = jpl_snapshot_evidence_summary_for_report();
        let reference_report = reference_snapshot_summary_for_report();
        let holdout_summary = jpl_independent_holdout_summary_for_report();
        let holdout_high_curvature = independent_holdout_high_curvature_summary_for_report();

        assert!(report.contains(&jpl_snapshot_evidence_classification_summary_for_report()));
        assert!(report.contains(&jpl_source_posture_summary_for_report()));
        assert!(report.contains(&jpl_provenance_only_summary_for_report()));
        assert!(report.contains(&reference_snapshot_summary_for_report()));
        assert!(report.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
        assert!(report.contains(&reference_snapshot_equatorial_parity_summary_for_report()));
        assert!(report.contains(&reference_snapshot_source_summary_for_report()));
        assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(report.contains(&reference_snapshot_major_body_boundary_summary_for_report()));
        assert!(report.contains(&reference_snapshot_mars_jupiter_boundary_summary_for_report()));
        // Migrated from jpl's whole-aggregator
        // `reference_snapshot_batch_parity_summary_reports_the_expected_coverage`
        // (reference_summary/reference_snapshot/tests.rs:2081), which asserted these
        // epoch-specific containments against `jpl_snapshot_evidence_summary_for_report`
        // and were not yet covered here (Slice D Task 13d).
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
            report.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_bridge_day_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451916_major_body_interior_summary_for_report())
        );
        assert!(report
            .contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
        assert!(
            report.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report())
        );
        assert!(
            report.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report())
        );
        assert!(report.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
        assert!(report.contains(&reference_snapshot_sparse_boundary_summary_for_report()));
        assert!(report.contains(&reference_asteroid_evidence_summary_for_report()));
        assert!(report.contains(&reference_asteroid_equatorial_evidence_summary_for_report()));
        assert!(report.contains(&reference_asteroid_source_window_summary_for_report()));
        assert!(report.contains(&reference_holdout_overlap_summary_for_report()));
        assert!(report.contains(&independent_holdout_high_curvature_summary_for_report()));
        assert!(report.contains(&reference_snapshot_manifest_summary_for_report()));
        assert!(report.contains(&production_generation_snapshot_summary_for_report()));
        assert!(report.contains(&production_generation_source_summary_for_report()));
        assert!(report.contains(&production_generation_boundary_source_summary_for_report()));
        assert!(report.contains(&production_generation_boundary_window_summary_for_report()));
        assert!(report
            .contains(&production_generation_boundary_body_class_coverage_summary_for_report()));
        assert!(
            report.contains(&production_generation_boundary_request_corpus_summary_for_report())
        );
        assert!(report.contains(
            &production_generation_boundary_request_corpus_equatorial_summary_for_report()
        ));
        assert!(report.contains(&selected_asteroid_source_2451917_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_2453000_summary_for_report()));
        // These four were dropped from this test's earlier (partial) per-file copy;
        // restored to match jpl's original `jpl_snapshot_evidence_summary_combines_the_backend_reports`
        // (reference_summary/jpl_posture/tests.rs:499) ahead of Task 14 deleting it (Slice D Task 13d).
        assert!(report.contains(&selected_asteroid_source_2500000_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_2634167_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_evidence_summary_for_report()));
        assert!(report.contains(&selected_asteroid_source_window_summary_for_report()));
        assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
        assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
        assert!(report.contains(&selected_asteroid_dense_boundary_summary_for_report()));
        assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_source_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_source_window_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_manifest_summary_for_report()));
        assert!(report.contains(&independent_holdout_snapshot_summary_for_report()));
        assert!(
            report.contains(&independent_holdout_snapshot_equatorial_parity_summary_for_report())
        );
        assert!(report.contains(&independent_holdout_snapshot_batch_parity_summary_for_report()));
        assert!(report.contains(&independent_holdout_source_summary_for_report()));
        assert!(report.contains(&independent_holdout_snapshot_source_window_summary_for_report()));
        assert!(report
            .contains(&independent_holdout_snapshot_quarter_day_boundary_summary_for_report()));
        assert!(report.contains(&independent_holdout_manifest_summary_for_report()));
        assert!(report.contains(&holdout_summary));
        assert!(report.contains(&holdout_high_curvature));
        assert!(!reference_report.contains(&holdout_summary));
        assert!(!reference_report.contains(&holdout_high_curvature));
    }

    #[test]
    fn jpl_snapshot_evidence_posture_summaries_validate_and_fail_closed() {
        let classification = pleiades_jpl::jpl_snapshot_evidence_classification_summary_details();
        let posture = pleiades_jpl::jpl_source_posture_summary_details();
        let provenance_only = pleiades_jpl::jpl_provenance_only_summary_details();
        let contract = pleiades_jpl::jpl_source_corpus_contract_summary_details();

        assert_eq!(
            jpl_snapshot_evidence_classification_summary_line(&classification),
            jpl_snapshot_evidence_classification_summary_for_report()
        );
        assert_eq!(
            jpl_source_posture_summary_line(&posture),
            jpl_source_posture_summary_for_report()
        );
        assert_eq!(
            jpl_provenance_only_summary_line(&provenance_only),
            jpl_provenance_only_summary_for_report()
        );
        assert_eq!(
            jpl_source_corpus_contract_summary_line(&contract),
            jpl_source_corpus_contract_summary_for_report()
        );
        assert!(jpl_source_corpus_contract_summary_line(&contract).contains("reference="));
        assert!(jpl_source_corpus_contract_summary_line(&contract).contains("hold-out="));
        assert!(jpl_source_corpus_contract_summary_line(&contract).contains("source windows="));
        assert!(jpl_source_corpus_contract_summary_line(&contract).contains("source revision="));
        assert!(jpl_source_corpus_contract_summary_line(&contract)
            .contains("boundary request corpora: ecliptic="));
        assert!(jpl_source_corpus_contract_summary_line(&contract).contains("equatorial="));
        assert_eq!(classification.validate(), Ok(()));
        assert_eq!(posture.validate(), Ok(()));
        assert_eq!(provenance_only.validate(), Ok(()));
        assert_eq!(contract.validate(), Ok(()));

        let drifted_classification = JplSnapshotEvidenceClassificationSummary {
            text: "JPL evidence classification: drifted",
        };
        let drifted_posture = JplSourcePostureSummary {
            text: "JPL source posture: drifted",
        };
        let drifted_provenance_only = JplProvenanceOnlySummary {
            text: "JPL provenance-only evidence: drifted",
        };
        let mut drifted_contract = pleiades_jpl::jpl_source_corpus_contract_summary_details();
        drifted_contract.source_posture = drifted_posture.clone();

        assert!(drifted_classification.validate().is_err());
        assert!(drifted_posture.validate().is_err());
        assert!(drifted_provenance_only.validate().is_err());
        assert!(drifted_contract.validate().is_err());
        assert!(drifted_classification
            .validate()
            .expect_err("drifted evidence classification should fail closed")
            .to_string()
            .contains("out of sync"));
        assert!(drifted_posture
            .validate()
            .expect_err("drifted source posture should fail closed")
            .to_string()
            .contains("out of sync"));
        assert!(drifted_provenance_only
            .validate()
            .expect_err("drifted provenance-only summary should fail closed")
            .to_string()
            .contains("out of sync"));
        assert!(drifted_contract
            .validate()
            .expect_err("drifted source corpus contract should fail closed")
            .to_string()
            .contains("out of sync"));
    }

    #[test]
    fn interpolation_quality_summary_reports_the_worst_case_labels() {
        let summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        assert_eq!(summary.sample_count, 223);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.epoch_count, 19);
        assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
        assert_eq!(
            summary.cubic_sample_count
                + summary.quadratic_sample_count
                + summary.linear_sample_count,
            summary.sample_count
        );
        assert!(summary.cubic_sample_count > 0);
        assert_eq!(summary.quadratic_sample_count, 0);
        assert_eq!(summary.linear_sample_count, 0);
        assert!(summary.mean_bracket_span_days.is_finite());
        assert!(summary.median_bracket_span_days.is_finite());
        assert!(summary.percentile_bracket_span_days.is_finite());
        assert!(!summary.max_bracket_span_body.is_empty());
        assert!(!summary.max_longitude_error_body.is_empty());
        assert!(!summary.max_latitude_error_body.is_empty());
        assert!(!summary.max_distance_error_body.is_empty());

        let rendered = jpl_interpolation_quality_summary_line(&summary);
        assert!(rendered.contains("cubic"));
        assert!(rendered.contains("quadratic"));
        assert!(rendered.contains("linear"));
        assert!(rendered.contains("223 samples across 16 bodies and 19 epochs"));
        assert!(rendered.contains("epoch window"));
        assert!(rendered.contains("mean bracket span="));
        assert!(rendered.contains("median bracket span="));
        assert!(rendered.contains("p95 bracket span="));
        assert!(rendered.contains("mean Δlon="));
        assert!(rendered.contains("median Δlon="));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("rms Δlon="));
        assert!(rendered.contains("mean Δlat="));
        assert!(rendered.contains("median Δlat="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("rms Δlat="));
        assert!(rendered.contains("mean Δdist="));
        assert!(rendered.contains("median Δdist="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered.contains("rms Δdist="));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_bracket_span_body,
            format_instant(summary.max_bracket_span_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_longitude_error_body,
            format_instant(summary.max_longitude_error_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_latitude_error_body,
            format_instant(summary.max_latitude_error_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_distance_error_body,
            format_instant(summary.max_distance_error_epoch)
        )));
        assert!(
            rendered.contains("transparency evidence only, not a production tolerance envelope")
        );
    }

    #[test]
    fn interpolation_quality_kind_coverage_reports_the_distinct_body_breakdown() {
        let coverage =
            pleiades_jpl::jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        assert_eq!(coverage.sample_count, 223);
        assert_eq!(coverage.body_count, 16);
        assert_eq!(coverage.bodies.len(), coverage.body_count);
        assert!(!coverage.bodies.is_empty());
        assert!(coverage.cubic_body_count > 0);
        assert_eq!(coverage.quadratic_body_count, 0);
        assert_eq!(coverage.linear_body_count, 0);

        let rendered = jpl_interpolation_quality_kind_coverage_line(&coverage);
        assert!(rendered.contains("JPL interpolation quality kind coverage:"));
        assert!(rendered.contains("223 samples across 16 bodies ["));
        assert!(rendered.contains(&coverage.bodies[0]));
        assert!(rendered.contains("cubic bodies"));
        assert!(rendered.contains("quadratic bodies"));
        assert!(rendered.contains("linear bodies"));
        assert_eq!(
            jpl_interpolation_quality_kind_coverage_for_report(),
            jpl_interpolation_quality_kind_coverage_line(&coverage)
        );
    }

    #[test]
    fn interpolation_quality_sample_request_corpus_reports_the_explicit_request_slice() {
        let summary = pleiades_jpl::interpolation_quality_sample_request_corpus_summary()
            .expect("sample request corpus should exist");
        assert_eq!(summary.request_count, 223);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.bodies.len(), summary.body_count);
        assert!(!summary.bodies.is_empty());
        assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
        assert_eq!(summary.time_scale, TimeScale::Tdb);
        assert_eq!(summary.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(summary.apparentness, Apparentness::Mean);
        assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            interpolation_quality_sample_request_corpus_summary_for_report(),
            interpolation_quality_sample_request_corpus_summary_line(&summary)
        );
        assert!(
            interpolation_quality_sample_request_corpus_summary_line(&summary)
                .contains("Interpolation-quality sample request corpus:")
        );
        assert!(
            interpolation_quality_sample_request_corpus_summary_line(&summary)
                .contains("observerless")
        );
    }

    #[test]
    fn interpolation_quality_sample_request_corpus_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::interpolation_quality_sample_request_corpus_summary()
            .expect("sample request corpus should exist");
        summary.request_count += 1;
        assert_eq!(
            summary.validate(),
            Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "request_count"
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_summary_for_report_combines_source_summary_summary_and_coverage() {
        let source_summary = pleiades_jpl::jpl_interpolation_quality_source_summary()
            .expect("source summary should exist");
        let summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        let coverage =
            pleiades_jpl::jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        let rendered = format_jpl_interpolation_quality_summary_for_report();

        assert!(
            rendered.contains(&jpl_interpolation_quality_source_summary_line(
                &source_summary
            ))
        );
        assert!(rendered.contains(&jpl_interpolation_quality_summary_line(&summary)));
        assert!(rendered.contains(&jpl_interpolation_quality_kind_coverage_line(&coverage)));
        assert!(
            rendered.contains(&interpolation_quality_sample_request_corpus_summary_for_report())
        );
        assert!(rendered.contains(&jpl_interpolation_body_class_error_envelopes_for_report()));
    }

    #[test]
    fn interpolation_body_class_error_envelope_summary_reports_the_expected_body_classes() {
        let summaries = pleiades_jpl::jpl_interpolation_body_class_error_envelopes()
            .expect("body-class envelopes should exist");

        assert_eq!(summaries.len(), 4);
        assert_eq!(summaries[0].class, "Luminaries");
        assert_eq!(summaries[1].class, "Major planets");
        assert_eq!(summaries[2].class, "Selected asteroids");
        assert_eq!(summaries[3].class, "Custom bodies");
        assert!(summaries.iter().all(|summary| summary.validate().is_ok()));
        assert!(jpl_interpolation_body_class_error_envelopes_for_report()
            .contains("JPL interpolation body-class error envelopes:"));
        assert!(jpl_interpolation_body_class_error_envelopes_for_report().contains("Luminaries"));
        assert!(jpl_interpolation_body_class_error_envelopes_for_report().contains("Major planets"));
    }

    #[test]
    fn interpolation_body_class_error_envelope_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::jpl_interpolation_body_class_error_envelopes()
            .expect("body-class envelopes should exist")
            .into_iter()
            .find(|summary| summary.class == "Luminaries")
            .expect("luminary envelope should exist");

        summary.mean_longitude_error_deg += 1e-12;

        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationBodyClassErrorEnvelopeSummaryValidationError::FieldOutOfSync {
                    class: "Luminaries"
                }
            )
        );
    }

    #[test]
    fn interpolation_posture_summary_reports_the_release_decision() {
        let summary =
            pleiades_jpl::jpl_interpolation_posture_summary().expect("summary should exist");
        // `JPL_INTERPOLATION_POSTURE_*` are `pub(crate)` consts in jpl's
        // not-yet-copied general_b.rs; their literal values are inlined here
        // (jpl's retained test still asserts the const equality directly).
        assert_eq!(
            summary.source,
            "leave-one-out runtime interpolation evidence derived from the checked-in reference snapshot"
        );
        assert_eq!(summary.detail, "transparency evidence only");
        assert_eq!(summary.envelope, "not a production tolerance envelope");
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            jpl_interpolation_posture_summary_for_report(),
            jpl_interpolation_posture_summary_line(&summary)
        );
        assert!(
            jpl_interpolation_posture_summary_line(&summary).contains("JPL interpolation posture:")
        );
        assert!(
            jpl_interpolation_posture_summary_line(&summary).contains("transparency evidence only")
        );
        assert!(jpl_interpolation_posture_summary_line(&summary)
            .contains("not a production tolerance envelope"));
    }

    #[test]
    fn interpolation_posture_summary_validation_rejects_drift() {
        let mut summary =
            pleiades_jpl::jpl_interpolation_posture_summary().expect("summary should exist");
        summary.detail = "runtime production tolerance".to_string();
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "detail" })
        );
    }

    #[test]
    fn interpolation_quality_source_summary_reports_the_expected_provenance() {
        let summary = pleiades_jpl::jpl_interpolation_quality_source_summary()
            .expect("source summary should exist");

        assert_eq!(
            summary.source,
            pleiades_jpl::reference_snapshot_source_summary().source
        );
        // `JPL_INTERPOLATION_QUALITY_DERIVATION` is a `pub(crate)` const in
        // jpl's not-yet-copied general_b.rs; its literal value is inlined
        // here (jpl's retained test still asserts the const equality
        // directly).
        assert_eq!(
            summary.derivation,
            "leave-one-out interpolation evidence derived from the checked-in reference snapshot"
        );
        assert_eq!(summary.sample_count, 223);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.epoch_count, 19);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            jpl_interpolation_quality_source_summary_for_report(),
            jpl_interpolation_quality_source_summary_line(&summary)
        );
    }

    #[test]
    fn interpolation_quality_summary_validated_summary_line_returns_the_rendered_line() {
        let summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        assert_eq!(summary.validate(), Ok(()));
    }

    #[test]
    fn interpolation_quality_summary_validated_summary_line_rejects_drift() {
        let mut summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        summary.mean_longitude_error_deg += 1e-12;
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn interpolation_quality_source_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::jpl_interpolation_quality_source_summary()
            .expect("source summary should exist");
        summary.epoch_count += 1;
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count"
                }
            )
        );
        assert_eq!(
            JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                field: "epoch_count"
            }
            .to_string(),
            "the JPL interpolation-quality source summary field `epoch_count` is out of sync with the current evidence"
        );
    }

    #[test]
    fn interpolation_quality_kind_coverage_validated_summary_line_rejects_drift() {
        let mut coverage =
            pleiades_jpl::jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        coverage.cubic_body_count += 1;
        assert_eq!(
            coverage.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_inconsistent_counts() {
        let mut summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        summary.sample_count = 0;
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::MissingSamples)
        );

        let mut summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        summary.cubic_sample_count += 1;
        let kind_count = summary.cubic_sample_count
            + summary.quadratic_sample_count
            + summary.linear_sample_count;
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::InterpolationKindCountMismatch {
                    sample_count: summary.sample_count,
                    kind_count,
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_non_finite_metrics() {
        let mut summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        summary.max_longitude_error_deg = f64::INFINITY;
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::MetricOutOfRange {
                    field: "max_longitude_error_deg",
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_blank_peak_bodies() {
        let mut summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        summary.max_latitude_error_body.clear();
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_latitude_error_body",
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_derived_summary_drift() {
        let mut summary =
            pleiades_jpl::jpl_interpolation_quality_summary().expect("summary should exist");
        summary.mean_longitude_error_deg += 1e-12;
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn interpolation_quality_coverage_validation_rejects_inconsistent_bodies() {
        let mut coverage =
            pleiades_jpl::jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        coverage.body_count += 1;
        assert_eq!(
            coverage.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::BodyCountMismatch {
                    body_count: coverage.body_count,
                    bodies_len: coverage.bodies.len(),
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_coverage_validation_rejects_duplicate_bodies() {
        let mut coverage =
            pleiades_jpl::jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        let duplicate = coverage.bodies[0].clone();
        coverage.bodies[1] = duplicate.clone();
        coverage.body_count = coverage.bodies.len();
        assert_eq!(
            coverage.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DuplicateBody { body: duplicate })
        );
    }

    #[test]
    fn interpolation_quality_coverage_validation_rejects_derived_summary_drift() {
        let mut coverage =
            pleiades_jpl::jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        coverage.cubic_body_count += 1;
        assert_eq!(
            coverage.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn frame_treatment_summary_documents_the_shared_mean_obliquity_transform() {
        let summary = pleiades_jpl::frame_treatment_summary_details();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            "checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"
        );
        assert_eq!(
            pleiades_jpl::frame_treatment_summary(),
            summary.summary_line()
        );
        assert_eq!(frame_treatment_summary_for_report(), summary.summary_line());
        assert!(summary.summary_line().contains("mean-obliquity transform"));
    }

    #[test]
    fn request_policy_summary_is_displayable() {
        let policy = pleiades_jpl::jpl_snapshot_request_policy();

        assert_eq!(
            jpl_snapshot_request_policy_summary_for_report(),
            jpl_snapshot_request_policy_summary_line(&policy)
        );
        assert!(jpl_snapshot_request_policy_summary_line(&policy)
            .contains("frames=Ecliptic, Equatorial"));
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn request_policy_summary_validation_rejects_stale_posture() {
        let mut policy = pleiades_jpl::jpl_snapshot_request_policy();
        policy.supports_topocentric_observer = true;

        let error = policy
            .validate()
            .expect_err("drifted JPL request-policy summaries should fail validation");

        assert_eq!(
            error,
            JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supports_topocentric_observer"
            }
        );
        assert_eq!(
            error.to_string(),
            "the JPL snapshot request-policy summary field `supports_topocentric_observer` is out of sync with the current posture"
        );
    }

    #[test]
    fn batch_error_taxonomy_request_corpus_matches_the_control_sample() {
        let requests = pleiades_jpl::jpl_snapshot_batch_error_taxonomy_request_corpus();

        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].body, pleiades_backend::CelestialBody::Ceres);
        assert_eq!(requests[1].body, pleiades_backend::CelestialBody::MeanNode);
        assert_eq!(requests[2].body, pleiades_backend::CelestialBody::Ceres);
        assert_eq!(requests[0].instant, pleiades_jpl::reference_instant());
        assert_eq!(requests[1].instant, pleiades_jpl::reference_instant());
        assert!(requests.iter().all(|request| request.observer.is_none()));
        assert!(requests
            .iter()
            .all(|request| request.frame == CoordinateFrame::Ecliptic));
        assert!(requests
            .iter()
            .all(|request| request.zodiac_mode == ZodiacMode::Tropical));
        assert!(requests
            .iter()
            .all(|request| request.apparent == Apparentness::Mean));
    }

    #[test]
    fn batch_error_taxonomy_summary_matches_current_backend() {
        let summary = pleiades_jpl::jpl_snapshot_batch_error_taxonomy_summary()
            .expect("the batch taxonomy summary should remain computable");
        assert_eq!(
            jpl_snapshot_batch_error_taxonomy_summary_line(&summary),
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        );
        assert_eq!(
            jpl_snapshot_batch_error_taxonomy_summary_for_report(),
            jpl_snapshot_batch_error_taxonomy_summary_line(&summary)
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            summary.supported_request_body,
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.unsupported_request_body,
            pleiades_backend::CelestialBody::MeanNode
        );
        assert_eq!(
            summary.unsupported_error_kind,
            EphemerisErrorKind::UnsupportedBody
        );
        assert_eq!(
            summary.out_of_range_request_body,
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.out_of_range_error_kind,
            EphemerisErrorKind::OutOfRangeInstant
        );
    }

    #[test]
    fn batch_error_taxonomy_summary_validation_rejects_drifted_fields() {
        let summary = JplSnapshotBatchErrorTaxonomySummary {
            supported_request_body: pleiades_backend::CelestialBody::Sun,
            unsupported_request_body: pleiades_backend::CelestialBody::MeanNode,
            unsupported_error_kind: EphemerisErrorKind::UnsupportedBody,
            out_of_range_request_body: pleiades_backend::CelestialBody::Ceres,
            out_of_range_error_kind: EphemerisErrorKind::OutOfRangeInstant,
        };
        assert_eq!(
            summary.validate(),
            Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "supported_request_body"
                }
            )
        );
        assert_eq!(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "supported_request_body"
            }
            .to_string(),
            "the JPL batch error-taxonomy summary field `supported_request_body` is out of sync with the current posture"
        );
    }
}
