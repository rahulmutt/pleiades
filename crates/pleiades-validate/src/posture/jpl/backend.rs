//! Relocated backend-struct renderers (InterpolationQualitySample,
//! SnapshotManifestSummary) copied from `pleiades-jpl::backend` (Slice D).

use pleiades_jpl::{InterpolationQualitySample, SnapshotManifestSummary};

/// Compact release-facing summary line for one interpolation-quality sample.
/// Verbatim copy of `InterpolationQualitySample::summary_line` (backend.rs:114).
pub(crate) fn interpolation_quality_sample_summary_line(s: &InterpolationQualitySample) -> String {
    format!(
        "{} at {}: {} interpolation, bracket span {:.1} d, |Δlon|={:.12}°, |Δlat|={:.12}°, |Δdist|={:.12} AU",
        s.body,
        s.epoch.summary_line(), // Instant::summary_line (pleiades-time) — NOT moved, stays
        s.interpolation_kind.label(),
        s.bracket_span_days,
        s.longitude_error_deg,
        s.latitude_error_deg,
        s.distance_error_au,
    )
}

/// Compact release-facing summary line for a manifest summary wrapper.
///
/// Verbatim copy of the rendering reached by
/// `SnapshotManifestSummary::summary_line` (backend.rs:798), which delegates
/// to `SnapshotManifest::summary_line_with_defaults` (backend.rs:549). The
/// source/coverage derivations call the `pub` data accessors
/// `SnapshotManifest::source_or`/`coverage_or` (backend.rs:433/438) directly —
/// exactly as jpl's `summary_line_with_defaults` does (backend.rs:562-563);
/// those accessors stay in jpl. The `columns` logic (`pub(crate)`
/// `columns_summary`, not callable cross-crate) and the title/redistribution
/// trim logic (matching the private `trimmed_or` helper) are inlined here
/// reading the struct's public fields directly. Does NOT copy
/// `validate()`/`validated_summary_line()` gate logic — that stays in jpl.
pub(crate) fn snapshot_manifest_summary_line(s: &SnapshotManifestSummary) -> String {
    let manifest = &s.manifest;
    let label = s.label;
    let source_fallback = s.source_fallback;
    let coverage_fallback = s.coverage_fallback;

    let title = manifest
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or("unknown");
    let source = manifest.source_or(source_fallback);
    let coverage = manifest.coverage_or(coverage_fallback);
    let columns = if manifest.columns.is_empty() {
        "none".to_string()
    } else {
        manifest.columns.join(", ")
    };
    let mut text =
        format!("{label}: {title}; source={source}; coverage={coverage}; columns={columns}");
    if let Some(redistribution) = manifest
        .redistribution
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        text.push_str("; redistribution=");
        text.push_str(redistribution);
    }
    text
}

#[cfg(test)]
mod golden {
    use pleiades_jpl::{
        independent_holdout_snapshot_manifest, interpolation_quality_sample_list,
        reference_snapshot_manifest, SnapshotManifestSummary,
    };

    // jpl's inherent renderer is still present through the contract sweep
    // (Task 14); this fails closed on any drift in the validate copy. Task 14
    // replaces `before` with the captured literal when the jpl method is
    // deleted.
    #[test]
    fn interpolation_quality_sample_lines_byte_identical() {
        for sample in interpolation_quality_sample_list() {
            let before = sample.summary_line(); // jpl inherent method (still present)
            let after = super::interpolation_quality_sample_summary_line(sample);
            assert_eq!(before, after);
        }
    }

    #[test]
    fn snapshot_manifest_summary_lines_byte_identical() {
        let summaries = [
            SnapshotManifestSummary {
                label: "Reference snapshot",
                manifest: reference_snapshot_manifest().clone(),
                source_fallback: "unknown",
                coverage_fallback: "unknown",
            },
            SnapshotManifestSummary {
                label: "Independent hold-out snapshot",
                manifest: independent_holdout_snapshot_manifest().clone(),
                source_fallback: "unknown",
                coverage_fallback: "unknown",
            },
        ];

        for summary in &summaries {
            let before = summary.summary_line(); // jpl inherent method (still present)
            let after = super::snapshot_manifest_summary_line(summary);
            assert_eq!(before, after);
        }
    }
}
