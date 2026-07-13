//! Relocated comparison-snapshot renderers copied from
//! `pleiades-jpl::reference_summary::comparison` (report-surface relocation
//! program, Slice D). Rendering only — the functional crate keeps the
//! structured evidence structs, their `*_details()` constructors,
//! `validate()`/`label()` methods, and all release-gate data; jpl's own
//! rendering stays in place until the Task 14 contract sweep.

use pleiades_jpl::{
    ComparisonSnapshotBatchParitySummary, ComparisonSnapshotBatchParitySummaryValidationError,
    ComparisonSnapshotBodyClassCoverageSummary,
    ComparisonSnapshotBodyClassCoverageSummaryValidationError, ComparisonSnapshotSourceSummary,
    ComparisonSnapshotSourceWindow, ComparisonSnapshotSourceWindowSummary,
    ComparisonSnapshotSummary, SnapshotManifest,
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

/// Compact release-facing body-class summary line for the comparison
/// snapshot. Verbatim copy of
/// `ComparisonSnapshotBodyClassCoverageSummary::summary_line`
/// (reference_summary/comparison.rs:202), with the private `body_count()`
/// helper (comparison.rs:220) inlined as `s.bodies.len()` and the nested
/// `ComparisonSnapshotSourceWindow::summary_line` call rewired to the local
/// `comparison_snapshot_source_window_line`.
pub(crate) fn comparison_snapshot_body_class_coverage_summary_line(
    s: &ComparisonSnapshotBodyClassCoverageSummary,
) -> String {
    let windows = s
        .windows
        .iter()
        .map(comparison_snapshot_source_window_line)
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Comparison snapshot body-class coverage: {} rows across {} bodies and {} epochs; bodies: {}; windows: {}",
        s.row_count,
        s.bodies.len(),
        s.epoch_count,
        format_bodies(&s.bodies),
        windows,
    )
}

/// Compact release-facing summary line for the comparison snapshot
/// provenance. Verbatim copy of `ComparisonSnapshotSourceSummary::summary_line`
/// (reference_summary/comparison.rs:478).
pub(crate) fn comparison_snapshot_source_summary_line(
    s: &ComparisonSnapshotSourceSummary,
) -> String {
    format!(
        "Comparison snapshot source: {}; coverage={}; redistribution={}; columns={}; checksum=0x{:016x}",
        s.source, s.coverage, s.redistribution, s.columns, s.checksum
    )
}

/// Compact release-facing summary line for a single comparison-snapshot
/// body window. Verbatim copy of `ComparisonSnapshotSourceWindow::summary_line`
/// (reference_summary/comparison.rs:587).
pub(crate) fn comparison_snapshot_source_window_line(s: &ComparisonSnapshotSourceWindow) -> String {
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

/// Compact release-facing summary line for the comparison snapshot source
/// windows. Verbatim copy of
/// `ComparisonSnapshotSourceWindowSummary::summary_line`
/// (reference_summary/comparison.rs:631), with the nested
/// `ComparisonSnapshotSourceWindow::summary_line` call rewired to the local
/// `comparison_snapshot_source_window_line`.
pub(crate) fn comparison_snapshot_source_window_summary_line(
    s: &ComparisonSnapshotSourceWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(comparison_snapshot_source_window_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Comparison snapshot source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Compact release-facing summary line for the comparison snapshot coverage.
/// Verbatim copy of `ComparisonSnapshotSummary::summary_line`
/// (reference_summary/comparison.rs:955).
pub(crate) fn comparison_snapshot_summary_line(s: &ComparisonSnapshotSummary) -> String {
    format!(
        "Comparison snapshot coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
        s.row_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(&s.bodies),
    )
}

/// Compact release-facing summary line for the comparison snapshot batch
/// parity evidence. Verbatim copy of
/// `ComparisonSnapshotBatchParitySummary::summary_line`
/// (reference_summary/comparison.rs:1205).
pub(crate) fn comparison_snapshot_batch_parity_summary_line(
    s: &ComparisonSnapshotBatchParitySummary,
) -> String {
    format!(
        "JPL comparison snapshot batch parity: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; frame mix: {} ecliptic, {} equatorial; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
        s.snapshot.row_count,
        s.snapshot.body_count,
        s.snapshot.epoch_count,
        format_instant(s.snapshot.earliest_epoch),
        format_instant(s.snapshot.latest_epoch),
        format_bodies(&s.snapshot.bodies),
        s.ecliptic_request_count,
        s.equatorial_request_count,
        s.exact_count,
        s.interpolated_count,
        s.approximate_count,
        s.unknown_count,
    )
}

/// Returns the release-facing body-class coverage summary string for the
/// comparison snapshot. Verbatim copy of jpl's
/// `comparison_snapshot_body_class_coverage_summary_for_report`
/// (reference_summary/comparison.rs:303).
pub(crate) fn comparison_snapshot_body_class_coverage_summary_for_report() -> String {
    match pleiades_jpl::comparison_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => comparison_snapshot_body_class_coverage_summary_line(&summary),
            Err(error) => format!("Comparison snapshot body-class coverage: unavailable ({error})"),
        },
        None => "Comparison snapshot body-class coverage: unavailable".to_string(),
    }
}

/// Returns the validated release-facing body-class coverage summary string
/// for the comparison snapshot. Verbatim copy of jpl's
/// `validated_comparison_snapshot_body_class_coverage_summary_for_report`
/// (reference_summary/comparison.rs:314).
pub(crate) fn validated_comparison_snapshot_body_class_coverage_summary_for_report(
) -> Result<String, String> {
    let summary =
        pleiades_jpl::comparison_snapshot_body_class_coverage_summary().ok_or_else(|| {
            ComparisonSnapshotBodyClassCoverageSummaryValidationError::Unavailable.to_string()
        })?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(comparison_snapshot_body_class_coverage_summary_line(
        &summary,
    ))
}

/// Formats the validated source/material summary for the comparison
/// snapshot. Reproduced from jpl's private (`pub(crate)`, not callable
/// cross-crate) `format_validated_comparison_snapshot_source_summary_for_report`
/// (reference_summary/comparison.rs:530) — not itself one of the 11 `_for_report`
/// renderers, but exercised directly by the copied report test, and needed by
/// `comparison_snapshot_source_summary_for_report` below.
pub(crate) fn format_validated_comparison_snapshot_source_summary_for_report(
    summary: &ComparisonSnapshotSourceSummary,
    manifest: &SnapshotManifest,
) -> String {
    if let Err(error) = manifest.validate() {
        return format!("Comparison snapshot source: unavailable ({error})");
    }

    match summary.validate() {
        Ok(()) => comparison_snapshot_source_summary_line(summary),
        Err(error) => format!("Comparison snapshot source: unavailable ({error})"),
    }
}

/// Returns the source/material summary for the comparison snapshot used by
/// validation. Verbatim copy of jpl's
/// `comparison_snapshot_source_summary_for_report`
/// (reference_summary/comparison.rs:545).
pub(crate) fn comparison_snapshot_source_summary_for_report() -> String {
    format_validated_comparison_snapshot_source_summary_for_report(
        &pleiades_jpl::comparison_snapshot_source_summary(),
        pleiades_jpl::comparison_snapshot_manifest(),
    )
}

/// Returns the validated source/material summary for the comparison
/// snapshot. Verbatim copy of jpl's
/// `validated_comparison_snapshot_source_summary_for_report`
/// (reference_summary/comparison.rs:553).
pub(crate) fn validated_comparison_snapshot_source_summary_for_report() -> Result<String, String> {
    let manifest = pleiades_jpl::comparison_snapshot_manifest();
    manifest.validate().map_err(|error| error.to_string())?;
    let summary = pleiades_jpl::comparison_snapshot_source_summary();
    summary.validate().map_err(|error| error.to_string())?;
    Ok(comparison_snapshot_source_summary_line(&summary))
}

/// Returns the validated source-window summary for the comparison snapshot.
/// Verbatim copy of jpl's
/// `validated_comparison_snapshot_source_window_summary_for_report`
/// (reference_summary/comparison.rs:562).
pub(crate) fn validated_comparison_snapshot_source_window_summary_for_report(
) -> Result<String, String> {
    match pleiades_jpl::comparison_snapshot_source_window_summary() {
        Some(summary) => {
            summary.validate().map_err(|error| error.to_string())?;
            Ok(comparison_snapshot_source_window_summary_line(&summary))
        }
        None => Err("comparison snapshot source windows unavailable".to_string()),
    }
}

/// Returns the body-window summary for the comparison snapshot. Verbatim
/// copy of jpl's `comparison_snapshot_source_window_summary_for_report`
/// (reference_summary/comparison.rs:825).
pub(crate) fn comparison_snapshot_source_window_summary_for_report() -> String {
    match validated_comparison_snapshot_source_window_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) if error == "comparison snapshot source windows unavailable" => {
            "Comparison snapshot source windows: unavailable".to_string()
        }
        Err(error) => format!("Comparison snapshot source windows: unavailable ({error})"),
    }
}

/// Returns the manifest summary for the comparison snapshot used by
/// validation. Verbatim copy of jpl's
/// `comparison_snapshot_manifest_summary_for_report`
/// (reference_summary/comparison.rs:847).
pub(crate) fn comparison_snapshot_manifest_summary_for_report() -> String {
    match validated_comparison_snapshot_manifest_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Comparison snapshot manifest: unavailable ({error})"),
    }
}

/// Structured validation error reproduced from jpl's private (`pub(crate)`,
/// not callable cross-crate) `SnapshotManifestHeaderStructureError`
/// (backend.rs:1008), needed only to reproduce
/// `validate_snapshot_manifest_header_structure` below.
#[derive(Clone, Debug, Eq, PartialEq)]
enum SnapshotManifestHeaderStructureError {
    /// The manifest comment block contained an unexpected number of non-empty lines.
    CommentCountMismatch {
        /// Number of header comments expected for this manifest shape.
        expected: usize,
        /// Number of header comments actually present.
        found: usize,
    },
    /// A specific manifest comment line drifted from the canonical header structure.
    CommentMismatch {
        /// Zero-based position of the drifted comment line.
        index: usize,
        /// Logical field the comment line corresponds to.
        field: &'static str,
        /// Value expected at this position.
        expected: String,
        /// Value actually found at this position.
        found: String,
    },
}

impl core::fmt::Display for SnapshotManifestHeaderStructureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CommentCountMismatch { expected, found } => write!(
                f,
                "unexpected manifest comment count: expected {expected}, found {found}"
            ),
            Self::CommentMismatch {
                index,
                field,
                expected,
                found,
            } => write!(
                f,
                "manifest comment {index} ({field}) mismatch: expected {expected} but found {found}"
            ),
        }
    }
}

/// Reproduced from jpl's private (`pub(crate)`, not callable cross-crate)
/// `validate_snapshot_manifest_header_structure` (backend.rs:1042).
fn validate_snapshot_manifest_header_structure(
    source: &str,
    expected_title: &str,
    expected_source: &str,
    expected_coverage: &str,
    expected_redistribution: Option<&str>,
    expected_columns: &[&str],
) -> Result<(), SnapshotManifestHeaderStructureError> {
    let mut expected_comments = vec![
        expected_title.to_string(),
        format!("Source: {expected_source}"),
        format!("Coverage: {expected_coverage}"),
    ];
    if let Some(expected_redistribution) = expected_redistribution {
        expected_comments.push(format!("Redistribution: {expected_redistribution}"));
    }
    expected_comments.push(format!("Columns: {}", expected_columns.join(",")));
    let comments = source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let comment = trimmed.strip_prefix('#')?.trim();
            if comment.is_empty() {
                None
            } else {
                Some(comment.to_string())
            }
        })
        .collect::<Vec<_>>();

    if comments.len() != expected_comments.len() {
        return Err(SnapshotManifestHeaderStructureError::CommentCountMismatch {
            expected: expected_comments.len(),
            found: comments.len(),
        });
    }

    for (index, (found, expected)) in comments.iter().zip(expected_comments.iter()).enumerate() {
        if found != expected {
            return Err(SnapshotManifestHeaderStructureError::CommentMismatch {
                index,
                field: match index {
                    0 => "title",
                    1 => "source",
                    2 => "coverage",
                    3 => "redistribution",
                    _ => "columns",
                },
                expected: expected.clone(),
                found: found.clone(),
            });
        }
    }

    Ok(())
}

/// Returns the validated manifest summary for the comparison snapshot used
/// by validation. Verbatim copy of jpl's
/// `validated_comparison_snapshot_manifest_summary_for_report`
/// (reference_summary/comparison.rs:855), with the manifest line delegated
/// to Task 2's `crate::posture::jpl::backend::snapshot_manifest_summary_line`.
///
/// The `manifest_text` load can't use jpl's own
/// `env!("CARGO_MANIFEST_DIR")`-relative `include_str!` verbatim — that macro
/// resolves against *this* crate's manifest dir, not jpl's — so it reaches
/// one directory over to jpl's checked-in copy of the same file; the bytes
/// read are identical either way. `COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED`
/// is `pub(crate)` in jpl's `reference_snapshot/core/general_a.rs` (not yet
/// copied), so its literal value is reproduced inline.
pub(crate) fn validated_comparison_snapshot_manifest_summary_for_report() -> Result<String, String>
{
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../pleiades-jpl/data/j2000_snapshot.csv"
    ));
    const COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED: &str =
        "repository-checked regression fixtures, not a broad public corpus.";
    validate_snapshot_manifest_header_structure(
        manifest_text,
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
        "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
        Some(COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED),
        &["body", "x_km", "y_km", "z_km"],
    )
    .map_err(|error| error.to_string())?;

    let summary = pleiades_jpl::comparison_snapshot_manifest_summary();
    summary
        .validate_with_expected_metadata_and_redistribution(
            "JPL Horizons reference snapshot.",
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
            "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
            COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED,
            &["body", "x_km", "y_km", "z_km"],
        )
        .map_err(|error| error.to_string())?;

    Ok(crate::posture::jpl::backend::snapshot_manifest_summary_line(&summary))
}

/// Returns the release-facing comparison snapshot coverage summary string.
/// Verbatim copy of jpl's `comparison_snapshot_summary_for_report`
/// (reference_summary/comparison.rs:988).
pub(crate) fn comparison_snapshot_summary_for_report() -> String {
    match pleiades_jpl::comparison_snapshot_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => comparison_snapshot_summary_line(&summary),
            Err(error) => format!("Comparison snapshot coverage: unavailable ({error})"),
        },
        None => "Comparison snapshot coverage: unavailable".to_string(),
    }
}

/// Returns the release-facing comparison snapshot batch parity summary
/// string. Verbatim copy of jpl's
/// `comparison_snapshot_batch_parity_summary_for_report`
/// (reference_summary/comparison.rs:1246).
pub(crate) fn comparison_snapshot_batch_parity_summary_for_report() -> String {
    match pleiades_jpl::comparison_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => comparison_snapshot_batch_parity_summary_line(&summary),
            Err(error) => format!("JPL comparison snapshot batch parity: unavailable ({error})"),
        },
        None => "JPL comparison snapshot batch parity: unavailable".to_string(),
    }
}

/// Returns the validated release-facing comparison snapshot batch parity
/// summary string. Verbatim copy of jpl's
/// `validated_comparison_snapshot_batch_parity_summary_for_report`
/// (reference_summary/comparison.rs:1257).
pub(crate) fn validated_comparison_snapshot_batch_parity_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::comparison_snapshot_batch_parity_summary().ok_or_else(|| {
        ComparisonSnapshotBatchParitySummaryValidationError::Unavailable.to_string()
    })?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(comparison_snapshot_batch_parity_summary_line(&summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_snapshot_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::comparison_snapshot_summary()
            .expect("comparison snapshot summary should exist");
        assert_eq!(summary.row_count, 162);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_415_020.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_453_000.5);
        assert_eq!(summary.bodies.as_slice(), pleiades_jpl::comparison_bodies());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Comparison snapshot coverage: 162 rows across 10 bodies and 18 epochs (JD 2415020.5 (TDB)..JD 2453000.5 (TDB)); bodies: Sun, Moon, Mercury, Venus, Jupiter, Mars, Neptune, Pluto, Saturn, Uranus"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            comparison_snapshot_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn comparison_snapshot_body_class_coverage_summary_reports_the_expected_windows() {
        let summary = pleiades_jpl::comparison_snapshot_body_class_coverage_summary()
            .expect("comparison snapshot body-class coverage summary should exist");

        assert_eq!(summary.row_count, 162);
        assert_eq!(summary.bodies.as_slice(), pleiades_jpl::comparison_bodies());
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.windows.len(), summary.bodies.len());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            comparison_snapshot_body_class_coverage_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            validated_comparison_snapshot_body_class_coverage_summary_for_report(),
            Ok(summary.summary_line())
        );
        assert!(summary
            .summary_line()
            .starts_with("Comparison snapshot body-class coverage: 162 rows across 10 bodies and 18 epochs; bodies: "));
        assert!(summary.summary_line().contains("windows: Sun:"));
    }

    #[test]
    fn comparison_snapshot_batch_parity_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::comparison_snapshot_batch_parity_summary()
            .expect("comparison snapshot batch parity summary should exist");
        assert_eq!(summary.snapshot.row_count, 162);
        assert_eq!(summary.snapshot.body_count, 10);
        assert_eq!(summary.snapshot.epoch_count, 18);
        assert_eq!(
            summary.snapshot.earliest_epoch.julian_day.days(),
            2_415_020.5
        );
        assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_453_000.5);
        assert_eq!(
            summary.snapshot.bodies.as_slice(),
            pleiades_jpl::comparison_bodies()
        );
        assert_eq!(summary.ecliptic_request_count, 81);
        assert_eq!(summary.equatorial_request_count, 81);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            format!(
                "JPL comparison snapshot batch parity: 162 rows across 10 bodies and 18 epochs (JD 2415020.5 (TDB)..JD 2453000.5 (TDB)); bodies: {}; frame mix: 81 ecliptic, 81 equatorial; quality counts: Exact=162, Interpolated=0, Approximate=0, Unknown=0; batch/single parity preserved",
                format_bodies(pleiades_jpl::comparison_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            comparison_snapshot_batch_parity_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            validated_comparison_snapshot_batch_parity_summary_for_report(),
            Ok(summary.summary_line())
        );
    }

    #[test]
    fn comparison_snapshot_manifest_parses_the_documented_header_comments() {
        let manifest = pleiades_jpl::comparison_snapshot_manifest();
        let source_summary = pleiades_jpl::comparison_snapshot_source_summary();
        assert_eq!(
            manifest.title.as_deref(),
            Some("JPL Horizons reference snapshot.")
        );
        assert_eq!(
            manifest.source.as_deref(),
            Some("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.")
        );
        assert_eq!(
            manifest.coverage.as_deref(),
            Some("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.")
        );
        assert_eq!(
            manifest.redistribution.as_deref(),
            Some("repository-checked regression fixtures, not a broad public corpus.")
        );
        assert_eq!(manifest.columns, ["body", "x_km", "y_km", "z_km"]);
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(
            source_summary.summary_line(),
            format!(
                "Comparison snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; redistribution=repository-checked regression fixtures, not a broad public corpus.; columns=body, x_km, y_km, z_km; checksum=0x{:016x}",
                source_summary.checksum
            )
        );
        assert_eq!(source_summary.to_string(), source_summary.summary_line());
        assert_eq!(source_summary.validate(), Ok(()));
        assert_eq!(
            source_summary.validated_summary_line(),
            Ok(source_summary.summary_line())
        );
        assert_eq!(
            pleiades_jpl::format_comparison_snapshot_source_summary(&source_summary),
            source_summary.summary_line()
        );
        assert_eq!(
            comparison_snapshot_source_summary_for_report(),
            source_summary.summary_line()
        );
        assert_eq!(
            validated_comparison_snapshot_source_summary_for_report(),
            Ok(source_summary.summary_line())
        );
        let source_window_summary = pleiades_jpl::comparison_snapshot_source_window_summary()
            .expect("comparison snapshot source window summary should exist");
        assert_eq!(
            source_window_summary.summary_line(),
            comparison_snapshot_source_window_summary_for_report()
        );
        assert_eq!(
            source_window_summary.to_string(),
            source_window_summary.summary_line()
        );
        assert_eq!(source_window_summary.validate(), Ok(()));
        assert_eq!(
            source_window_summary.validated_summary_line(),
            Ok(source_window_summary.summary_line())
        );
        assert_eq!(
            comparison_snapshot_source_window_summary_for_report(),
            source_window_summary.summary_line()
        );
        assert_eq!(
            pleiades_jpl::format_comparison_snapshot_source_window_summary(&source_window_summary),
            source_window_summary.summary_line()
        );
        assert_eq!(
            format_validated_comparison_snapshot_source_summary_for_report(
                &source_summary,
                manifest,
            ),
            source_summary.summary_line()
        );
        let invalid_manifest = pleiades_jpl::SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some(" ".to_string()),
            coverage: Some("coverage".to_string()),
            redistribution: None,
            columns: vec!["body".to_string()],
        };
        assert_eq!(
            format_validated_comparison_snapshot_source_summary_for_report(
                &source_summary,
                &invalid_manifest,
            ),
            "Comparison snapshot source: unavailable (missing source)"
        );
        assert_eq!(
            manifest.summary_line("Comparison snapshot manifest"),
            "Comparison snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
        assert_eq!(
            manifest.to_string(),
            "Snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
        let comparison_summary = pleiades_jpl::comparison_snapshot_manifest_summary();
        assert_eq!(
            comparison_summary.summary_line(),
            "Comparison snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
        assert_eq!(
            comparison_summary.to_string(),
            comparison_summary.summary_line()
        );
        assert_eq!(
            comparison_snapshot_manifest_summary_for_report(),
            comparison_summary.summary_line()
        );
    }

    #[test]
    fn comparison_snapshot_source_window_summary_reports_the_expected_body_windows() {
        let summary = pleiades_jpl::comparison_snapshot_source_window_summary()
            .expect("comparison snapshot source window summary should exist");
        assert_eq!(summary.sample_count, 162);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.windows.len(), 10);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.summary_line().contains("Comparison snapshot source windows: 162 source-backed samples across 10 bodies and 18 epochs"));
        assert!(summary.summary_line().contains(
            "Mars: 15 samples across 15 epochs at JD 2451545.0 (TDB)..JD 2453000.5 (TDB)"
        ));
        assert!(summary.summary_line().contains(
            "Pluto: 15 samples across 15 epochs at JD 2451545.0 (TDB)..JD 2453000.5 (TDB)"
        ));
        assert_eq!(
            comparison_snapshot_source_window_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            validated_comparison_snapshot_source_window_summary_for_report(),
            Ok(summary.summary_line())
        );
        assert_eq!(
            pleiades_jpl::format_comparison_snapshot_source_window_summary(&summary),
            summary.summary_line()
        );
    }

    #[test]
    fn comparison_snapshot_manifest_summary_uses_the_current_manifest() {
        let summary = pleiades_jpl::comparison_snapshot_manifest_summary();

        assert_eq!(
            summary.validate_with_expected_metadata(
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
                "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
                &["body", "x_km", "y_km", "z_km"],
            ),
            Ok(())
        );
        assert_eq!(
            summary.summary_line(),
            validated_comparison_snapshot_manifest_summary_for_report()
                .expect("comparison snapshot manifest summary should validate")
        );
        assert_eq!(summary.to_string(), summary.summary_line());
    }
}
