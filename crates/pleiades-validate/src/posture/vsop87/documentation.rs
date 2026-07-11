//! VSOP87 source-documentation and source-documentation-health report prose
//! relocated from `pleiades-vsop87::source_docs::documentation`
//! (report-surface relocation program, Slice B). Rendering only — the
//! functional crate keeps the structured data and their constructors.
//!
//! `format_source_documentation_summary` and
//! `format_source_documentation_health_summary` could not be deleted from
//! `pleiades-vsop87` outright: the retained inherent methods
//! `Vsop87SourceDocumentationSummary::summary_line()` and
//! `Vsop87SourceDocumentationHealthSummary::summary_line()` (plus their
//! `Display` impls) delegated to them, and those methods are exercised
//! throughout `pleiades-vsop87`'s own retained tests. `pleiades-vsop87`
//! cannot depend on `pleiades-validate`, so the rendering logic was inlined
//! into those inherent methods (unchanged format strings) and this module
//! carries a byte-identical copy for the release-facing free-function path,
//! along with private helpers (`format_celestial_bodies`, `format_bodies`,
//! `format_source_files`, `format_issue_labels`) mirroring the
//! crate-private helpers of the same name in `pleiades-vsop87` that remain
//! inaccessible across the crate boundary.

use pleiades_types::CelestialBody;

use pleiades_vsop87::{
    source_documentation_health_summary, source_documentation_summary,
    Vsop87SourceDocumentationHealthSummary, Vsop87SourceDocumentationSummary,
};

fn format_celestial_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_bodies(bodies: &[CelestialBody]) -> String {
    if bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(bodies)
    }
}

fn format_source_files(source_files: &[&'static str]) -> String {
    if source_files.is_empty() {
        "none".to_string()
    } else {
        source_files.join(", ")
    }
}

fn format_issue_labels<T: std::fmt::Display>(issues: &[T]) -> String {
    if issues.is_empty() {
        "none".to_string()
    } else {
        issues
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Formats the current VSOP87 source-documentation catalog for reporting.
pub(crate) fn format_source_documentation_summary(
    summary: &Vsop87SourceDocumentationSummary,
) -> String {
    let source_backed_bodies = if summary.source_backed_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.source_backed_bodies)
    };
    let fallback_bodies = if summary.fallback_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.fallback_bodies)
    };
    let source_files = if summary.source_files.is_empty() {
        "none".to_string()
    } else {
        summary.source_files.join(", ")
    };
    let date_ranges = if summary.date_ranges.is_empty() {
        "none".to_string()
    } else {
        summary.date_ranges.join("; ")
    };
    let generated_binary_bodies = if summary.generated_binary_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.generated_binary_bodies)
    };
    let vendored_full_file_bodies = if summary.vendored_full_file_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.vendored_full_file_bodies)
    };
    let truncated_bodies = if summary.truncated_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.truncated_bodies)
    };
    let fallback_profile_label = if summary.fallback_profile_count == 1 {
        "approximate fallback mean-element body profile"
    } else {
        "approximate fallback mean-element body profiles"
    };

    format!(
        "VSOP87 source documentation: {} source specs, {} source-backed body profiles, {} {} ({}); source-backed bodies: {}; source files: {}; source-backed breakdown: {} generated binary bodies ({}), {} vendored full-file bodies ({}), {} truncated slice bodies ({}); date ranges: {}",
        summary.source_specification_count,
        summary.source_backed_profile_count,
        summary.fallback_profile_count,
        fallback_profile_label,
        fallback_bodies,
        source_backed_bodies,
        source_files,
        summary.generated_binary_profile_count,
        generated_binary_bodies,
        summary.vendored_full_file_profile_count,
        vendored_full_file_bodies,
        summary.truncated_profile_count,
        truncated_bodies,
        date_ranges,
    )
}

/// Formats the current VSOP87 source-documentation health check for reporting.
pub(crate) fn format_source_documentation_health_summary(
    summary: &Vsop87SourceDocumentationHealthSummary,
) -> String {
    let issues = if summary.issues.is_empty() {
        String::new()
    } else {
        format!("; issues: {}", format_issue_labels(&summary.issues))
    };

    format!(
        "VSOP87 source documentation health: {} ({} source specs, {} source files, {} source-backed profiles, {} body profiles; {} generated binary profiles ({}), {} vendored full-file profiles ({}), {} truncated profiles ({}), {} approximate fallback profiles ({}); source files: {}; source-backed order: {}; source-backed partition order: {}; fallback order: {}; documented fields: {}){}",
        if summary.consistent { "ok" } else { "needs attention" },
        summary.source_specification_count,
        summary.source_file_count,
        summary.source_backed_profile_count,
        summary.body_profile_count,
        summary.generated_binary_profile_count,
        format_bodies(&summary.generated_binary_bodies),
        summary.vendored_full_file_profile_count,
        format_bodies(&summary.vendored_full_file_bodies),
        summary.truncated_profile_count,
        format_bodies(&summary.truncated_bodies),
        summary.fallback_profile_count,
        format_bodies(&summary.fallback_bodies),
        format_source_files(&summary.source_files),
        format_bodies(&summary.source_backed_bodies),
        format_bodies(&summary.source_backed_partition_bodies),
        format_bodies(&summary.fallback_bodies),
        if summary.documentation_consistent {
            "variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range"
        } else {
            "needs attention"
        },
        issues,
    )
}

pub(crate) fn format_validated_source_documentation_health_summary_for_report(
    summary: &Vsop87SourceDocumentationHealthSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source documentation health: unavailable ({error})"),
    }
}

pub(crate) fn format_validated_source_documentation_summary_for_report(
    summary: &Vsop87SourceDocumentationSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("VSOP87 source documentation: unavailable ({error})"),
    }
}

/// Returns the release-facing summary string for the current VSOP87 source-documentation catalog.
///
/// The compact provenance line is rendered only after the catalog-health gate
/// confirms that the source/file/body partitioning still matches the current
/// canonical VSOP87 inputs.
pub(crate) fn source_documentation_summary_for_report() -> String {
    format_validated_source_documentation_summary_for_report(&source_documentation_summary())
}

/// Returns the release-facing source-documentation health string.
pub(crate) fn source_documentation_health_summary_for_report() -> String {
    format_validated_source_documentation_health_summary_for_report(
        &source_documentation_health_summary(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use pleiades_vsop87::Vsop87SourceDocumentationHealthIssue;

    #[test]
    fn source_documentation_summary_for_report_matches_the_backend_formatter() {
        let summary = source_documentation_summary();
        let rendered = source_documentation_summary_for_report();

        assert_eq!(summary.validated_summary_line().unwrap(), rendered);
        assert_eq!(rendered, format_source_documentation_summary(&summary));
        assert_eq!(summary.summary_line(), rendered);
        assert_eq!(summary.to_string(), rendered);
        assert!(rendered.contains("source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
        assert!(rendered.contains("source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"));
    }

    #[test]
    fn format_validated_source_documentation_summary_for_report_reports_drifted_fields() {
        let mut summary = source_documentation_summary();
        summary.source_specification_count += 1;

        assert_eq!(
            format_validated_source_documentation_summary_for_report(&summary),
            "VSOP87 source documentation: unavailable (the VSOP87 source documentation summary field `source_specification_count` is out of sync with the current source catalog)"
        );
    }

    #[test]
    fn source_documentation_health_summary_for_report_matches_the_backend_formatter() {
        let summary = source_documentation_health_summary();

        assert_eq!(
            source_documentation_health_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            format_source_documentation_health_summary(&summary),
            "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
        );
        assert_eq!(
            source_documentation_health_summary_for_report(),
            format_source_documentation_health_summary(&summary)
        );
    }

    #[test]
    fn format_source_documentation_health_summary_renders_issues_when_inconsistent() {
        let summary = Vsop87SourceDocumentationHealthSummary {
            consistent: false,
            documentation_consistent: false,
            issues: vec![
                Vsop87SourceDocumentationHealthIssue::SourceSpecificationFileCountMismatch,
                Vsop87SourceDocumentationHealthIssue::DocumentedFieldMismatch,
            ],
            source_specification_count: 1,
            source_file_count: 2,
            source_files: vec!["VSOP87B.ear"],
            source_backed_profile_count: 1,
            source_backed_bodies: vec![CelestialBody::Sun],
            source_backed_partition_bodies: vec![CelestialBody::Sun],
            generated_binary_bodies: vec![CelestialBody::Sun],
            vendored_full_file_bodies: vec![],
            truncated_bodies: vec![],
            body_profile_count: 2,
            generated_binary_profile_count: 1,
            vendored_full_file_profile_count: 0,
            truncated_profile_count: 0,
            fallback_profile_count: 1,
            fallback_bodies: vec![CelestialBody::Pluto],
        };

        assert_eq!(
            format_source_documentation_health_summary(&summary),
            "VSOP87 source documentation health: needs attention (1 source specs, 2 source files, 1 source-backed profiles, 2 body profiles; 1 generated binary profiles (Sun), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear; source-backed order: Sun; source-backed partition order: Sun; fallback order: Pluto; documented fields: needs attention); issues: source specification/file count mismatch, documented field mismatch"
        );
        assert_eq!(
            format_validated_source_documentation_health_summary_for_report(&summary),
            "VSOP87 source documentation health: unavailable (VSOP87 source documentation health: needs attention (1 source specs, 2 source files, 1 source-backed profiles, 2 body profiles; 1 generated binary profiles (Sun), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear; source-backed order: Sun; source-backed partition order: Sun; fallback order: Pluto; documented fields: needs attention); issues: source specification/file count mismatch, documented field mismatch)"
        );
    }
}
