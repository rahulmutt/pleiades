//! VSOP87 source-manifest report prose relocated from
//! `pleiades-vsop87::source_docs::request_corpus` (report-surface
//! relocation program, Slice B). Rendering only — the functional crate
//! keeps the structured data and their constructors.
//!
//! `pleiades-vsop87`'s own maintainer-facing `regenerate-vsop87b-tables`
//! binary called `format_source_manifest_summary` directly; it was
//! repointed to call the retained `Vsop87SourceManifestSummary::summary_line()`
//! inherent method instead (the function's entire body was that one-line
//! delegation, so the rendered output is unchanged).

use pleiades_vsop87::{source_manifest, source_manifest_summary, Vsop87SourceManifestSummary};

/// Formats a VSOP87 source-manifest summary for release-facing reporting.
pub(crate) fn format_source_manifest_summary(summary: &Vsop87SourceManifestSummary<'_>) -> String {
    summary.summary_line()
}

/// Returns the release-facing source-manifest summary for the current source catalog.
pub(crate) fn source_manifest_summary_for_report() -> String {
    let manifest = source_manifest();
    let summary = source_manifest_summary(&manifest);

    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source manifest: unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_manifest_summary_for_report_matches_the_backend_formatter() {
        let manifest = source_manifest();
        let summary = source_manifest_summary(&manifest);

        summary
            .validate()
            .expect("source manifest summary should match the catalog");
        let rendered = source_manifest_summary_for_report();

        assert_eq!(rendered, summary.to_string());
        assert_eq!(rendered, format_source_manifest_summary(&summary));
        assert_eq!(
            rendered,
            "VSOP87 source manifest: 8 entries (Sun / VSOP87B.ear, Mercury / VSOP87B.mer, Venus / VSOP87B.ven, Mars / VSOP87B.mar, Jupiter / VSOP87B.jup, Saturn / VSOP87B.sat, Uranus / VSOP87B.ura, Neptune / VSOP87B.nep)"
        );
    }
}
