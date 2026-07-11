//! VSOP87 batch-parity and source-backed body-class evidence report prose
//! relocated from `pleiades-vsop87::source_docs::batch_parity`
//! (report-surface relocation program, Slice B). Rendering only — the
//! functional crate keeps the structured data and their constructors.
//!
//! None of these renderers hit the retained-caller trap seen in the
//! `documentation`/`evidence` modules: every moved batch-parity summary's
//! `summary_line()` inherent method already has its own self-contained
//! `format!` body (not a delegation to a moved free function), and
//! `Vsop87SourceBodyClassEvidenceSummary::summary_line()` delegates to a
//! retained crate-private `format_source_body_class_evidence_entry` helper
//! that is distinct from the moved `format_source_body_class_evidence_summary`.
//! `format_source_body_class_evidence_summary` itself only calls the
//! retained *public* `Vsop87SourceBodyClassEvidenceSummary::summary_line`
//! method, so no private-helper duplication was needed here.

use pleiades_vsop87::{
    canonical_j1900_batch_parity_summary, canonical_j2000_batch_parity_summary,
    canonical_mixed_time_scale_batch_parity_summary, source_body_class_evidence_summary,
    supported_body_canonical_batch_parity_summary, supported_body_j1900_ecliptic_batch_parity_summary,
    supported_body_j1900_equatorial_batch_parity_summary,
    supported_body_j2000_ecliptic_batch_parity_summary,
    supported_body_j2000_equatorial_batch_parity_summary, Vsop87CanonicalJ1900BatchParitySummary,
    Vsop87CanonicalJ2000BatchParitySummary, Vsop87CanonicalMixedTimeScaleBatchParitySummary,
    Vsop87SourceBodyClassEvidenceSummary, Vsop87SupportedBodyCanonicalBatchParitySummary,
    Vsop87SupportedBodyJ1900EclipticBatchParitySummary,
    Vsop87SupportedBodyJ1900EquatorialBatchParitySummary,
    Vsop87SupportedBodyJ2000EclipticBatchParitySummary,
    Vsop87SupportedBodyJ2000EquatorialBatchParitySummary,
};

pub(crate) fn format_validated_canonical_j2000_batch_parity_summary_for_report(
    summary: &Vsop87CanonicalJ2000BatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 canonical J2000 batch parity: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical J2000 batch-path regression summary string.
pub(crate) fn canonical_j2000_batch_parity_summary_for_report() -> String {
    match canonical_j2000_batch_parity_summary() {
        Some(summary) => format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
        None => "VSOP87 canonical J2000 batch parity: unavailable".to_string(),
    }
}

pub(crate) fn format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report(
    summary: &Vsop87CanonicalMixedTimeScaleBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 canonical mixed TT/TDB batch parity: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical mixed TT/TDB batch-path regression summary string.
pub(crate) fn canonical_mixed_time_scale_batch_parity_summary_for_report() -> String {
    match canonical_mixed_time_scale_batch_parity_summary() {
        Some(summary) => {
            format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 canonical mixed TT/TDB batch parity: unavailable".to_string(),
    }
}

pub(crate) fn format_validated_canonical_j1900_batch_parity_summary_for_report(
    summary: &Vsop87CanonicalJ1900BatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 canonical J1900 batch parity: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical J1900 batch-path regression summary string.
pub(crate) fn canonical_j1900_batch_parity_summary_for_report() -> String {
    match canonical_j1900_batch_parity_summary() {
        Some(summary) => format_validated_canonical_j1900_batch_parity_summary_for_report(&summary),
        None => "VSOP87 canonical J1900 batch parity: unavailable".to_string(),
    }
}

pub(crate) fn format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ2000EclipticBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J2000 ecliptic batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J2000 ecliptic batch-path regression summary string.
pub(crate) fn supported_body_j2000_ecliptic_batch_parity_summary_for_report() -> String {
    match supported_body_j2000_ecliptic_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 supported-body J2000 ecliptic batch parity: unavailable".to_string(),
    }
}

pub(crate) fn format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ2000EquatorialBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J2000 equatorial batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J2000 equatorial batch-path regression summary string.
pub(crate) fn supported_body_j2000_equatorial_batch_parity_summary_for_report() -> String {
    match supported_body_j2000_equatorial_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report(
                &summary,
            )
        }
        None => "VSOP87 supported-body J2000 equatorial batch parity: unavailable".to_string(),
    }
}

pub(crate) fn format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ1900EclipticBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J1900 ecliptic batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J1900 ecliptic batch-path regression summary string.
pub(crate) fn supported_body_j1900_ecliptic_batch_parity_summary_for_report() -> String {
    match supported_body_j1900_ecliptic_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 supported-body J1900 ecliptic batch parity: unavailable".to_string(),
    }
}

pub(crate) fn format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ1900EquatorialBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J1900 equatorial batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J1900 equatorial batch-path regression summary string.
pub(crate) fn supported_body_j1900_equatorial_batch_parity_summary_for_report() -> String {
    match supported_body_j1900_equatorial_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report(
                &summary,
            )
        }
        None => "VSOP87 supported-body J1900 equatorial batch parity: unavailable".to_string(),
    }
}

pub(crate) fn format_validated_supported_body_canonical_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyCanonicalBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body canonical batch matrix: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body canonical batch matrix summary string.
pub(crate) fn supported_body_canonical_batch_parity_summary_for_report() -> String {
    match supported_body_canonical_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_canonical_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 supported-body canonical batch matrix: unavailable".to_string(),
    }
}

/// Formats the canonical VSOP87 body-class evidence for reporting.
pub(crate) fn format_source_body_class_evidence_summary(
    summaries: &[Vsop87SourceBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 source-backed body-class envelopes: unavailable".to_string();
    }

    let rendered = summaries
        .iter()
        .map(Vsop87SourceBodyClassEvidenceSummary::summary_line)
        .collect::<Vec<_>>()
        .join(" | ");

    format!("VSOP87 source-backed body-class envelopes: {rendered}")
}

pub(crate) fn format_validated_source_body_class_evidence_summary_for_report(
    summaries: &[Vsop87SourceBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 source-backed body-class envelopes: unavailable".to_string();
    }

    let mut rendered = Vec::with_capacity(summaries.len());
    for summary in summaries {
        match summary.validated_summary_line() {
            Ok(line) => rendered.push(line),
            Err(error) => {
                return format!("VSOP87 source-backed body-class envelopes: unavailable ({error})");
            }
        }
    }

    format!(
        "VSOP87 source-backed body-class envelopes: {}",
        rendered.join(" | ")
    )
}

/// Returns the release-facing source-body-class evidence summary string.
pub(crate) fn source_body_class_evidence_summary_for_report() -> String {
    match source_body_class_evidence_summary() {
        Some(summary) => format_validated_source_body_class_evidence_summary_for_report(&summary),
        None => "VSOP87 source-backed body-class envelopes: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_j2000_batch_parity_summary_for_report_matches_the_backend_formatter() {
        let summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
        let rendered = canonical_j2000_batch_parity_summary_for_report();

        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("quality counts: Exact="));
        assert!(rendered.contains("batch/single parity preserved"));
    }

    #[test]
    fn canonical_j2000_batch_parity_summary_for_report_surfaces_validation_errors() {
        let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
        summary.sample_count += 1;

        assert_eq!(
            format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
            "VSOP87 canonical J2000 batch parity: unavailable (the VSOP87 canonical batch parity summary field `sample_count` is out of sync with the current canonical evidence)"
        );

        let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
        summary.sample_bodies.reverse();
        assert_eq!(
            format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
            "VSOP87 canonical J2000 batch parity: unavailable (the VSOP87 canonical batch parity summary field `sample_bodies` is out of sync with the current canonical evidence)"
        );

        let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
        summary.frame = pleiades_types::CoordinateFrame::Equatorial;
        assert_eq!(
            format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
            "VSOP87 canonical J2000 batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
        );
    }

    #[test]
    fn canonical_mixed_time_scale_batch_parity_summary_for_report_matches_the_backend_formatter() {
        let summary = canonical_mixed_time_scale_batch_parity_summary()
            .expect("mixed batch summary should exist");
        let rendered = canonical_mixed_time_scale_batch_parity_summary_for_report();

        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("TT/TDB mix"));
        assert!(rendered.contains("TT requests="));
        assert!(rendered.contains("TDB requests="));
    }

    #[test]
    fn canonical_mixed_time_scale_batch_parity_summary_for_report_surfaces_validation_errors() {
        let mut summary = canonical_mixed_time_scale_batch_parity_summary()
            .expect("mixed batch summary should exist");
        summary.tt_request_count += 1;

        assert_eq!(
            format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report(&summary),
            "VSOP87 canonical mixed TT/TDB batch parity: unavailable (the VSOP87 canonical batch parity summary field `tt_request_count` is out of sync with the current canonical evidence)"
        );
    }

    #[test]
    fn canonical_j1900_batch_parity_summary_for_report_matches_the_backend_formatter() {
        let summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
        let rendered = canonical_j1900_batch_parity_summary_for_report();

        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("JD 2415020.0 (TDB)"));
        assert!(rendered.contains("quality counts: Exact="));
        assert!(rendered.contains("batch/single parity preserved"));
    }

    #[test]
    fn canonical_j1900_batch_parity_summary_for_report_surfaces_validation_errors() {
        let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
        summary.frame = pleiades_types::CoordinateFrame::Ecliptic;

        assert_eq!(
            format_validated_canonical_j1900_batch_parity_summary_for_report(&summary),
            "VSOP87 canonical J1900 batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
        );
    }

    #[test]
    fn supported_body_j2000_ecliptic_batch_parity_summary_for_report_matches_the_backend_formatter()
    {
        let summary = supported_body_j2000_ecliptic_batch_parity_summary()
            .expect("batch summary should exist");
        let rendered = supported_body_j2000_ecliptic_batch_parity_summary_for_report();

        assert_eq!(summary.summary_line(), rendered);
        assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
        assert!(rendered.contains("batch/single parity preserved"));
        assert!(
            rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto")
        );
    }

    #[test]
    fn supported_body_j2000_ecliptic_batch_parity_summary_for_report_surfaces_validation_errors() {
        let mut summary = supported_body_j2000_ecliptic_batch_parity_summary()
            .expect("batch summary should exist");
        summary.frame = pleiades_types::CoordinateFrame::Equatorial;

        assert_eq!(
            format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report(&summary),
            "VSOP87 supported-body J2000 ecliptic batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
        );
    }

    #[test]
    fn supported_body_j2000_equatorial_batch_parity_summary_for_report_matches_the_backend_formatter(
    ) {
        let summary = supported_body_j2000_equatorial_batch_parity_summary()
            .expect("batch summary should exist");
        let rendered = supported_body_j2000_equatorial_batch_parity_summary_for_report();

        assert_eq!(summary.summary_line(), rendered);
        assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
        assert!(rendered.contains("batch/single parity preserved"));
        assert!(
            rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto")
        );
    }

    #[test]
    fn supported_body_j2000_equatorial_batch_parity_summary_for_report_surfaces_validation_errors()
    {
        let mut summary = supported_body_j2000_equatorial_batch_parity_summary()
            .expect("batch summary should exist");
        summary.frame = pleiades_types::CoordinateFrame::Ecliptic;

        assert_eq!(
            format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report(
                &summary
            ),
            "VSOP87 supported-body J2000 equatorial batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
        );
    }

    #[test]
    fn supported_body_j1900_ecliptic_batch_parity_summary_for_report_matches_the_backend_formatter()
    {
        let summary = supported_body_j1900_ecliptic_batch_parity_summary()
            .expect("batch summary should exist");
        let rendered = supported_body_j1900_ecliptic_batch_parity_summary_for_report();

        assert_eq!(summary.summary_line(), rendered);
        assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
        assert!(rendered.contains("batch/single parity preserved"));
        assert!(
            rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto")
        );
    }

    #[test]
    fn supported_body_j1900_ecliptic_batch_parity_summary_for_report_surfaces_validation_errors() {
        let mut summary = supported_body_j1900_ecliptic_batch_parity_summary()
            .expect("batch summary should exist");
        summary.frame = pleiades_types::CoordinateFrame::Equatorial;

        assert_eq!(
            format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report(&summary),
            "VSOP87 supported-body J1900 ecliptic batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
        );
    }

    #[test]
    fn supported_body_j1900_equatorial_batch_parity_summary_for_report_matches_the_backend_formatter(
    ) {
        let summary = supported_body_j1900_equatorial_batch_parity_summary()
            .expect("batch summary should exist");
        let rendered = supported_body_j1900_equatorial_batch_parity_summary_for_report();

        assert_eq!(summary.summary_line(), rendered);
        assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
        assert!(rendered.contains("batch/single parity preserved"));
        assert!(
            rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto")
        );
    }

    #[test]
    fn supported_body_j1900_equatorial_batch_parity_summary_for_report_surfaces_validation_errors()
    {
        let mut summary = supported_body_j1900_equatorial_batch_parity_summary()
            .expect("batch summary should exist");
        summary.frame = pleiades_types::CoordinateFrame::Ecliptic;

        assert_eq!(
            format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report(
                &summary
            ),
            "VSOP87 supported-body J1900 equatorial batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
        );
    }

    #[test]
    fn supported_body_canonical_batch_parity_summary_for_report_matches_the_backend_formatter() {
        let summary = supported_body_canonical_batch_parity_summary()
            .expect("supported-body canonical batch matrix should exist");
        let rendered = supported_body_canonical_batch_parity_summary_for_report();

        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("J2000 ecliptic"));
        assert!(rendered.contains("J2000 equatorial"));
        assert!(rendered.contains("J1900 ecliptic"));
        assert!(rendered.contains("J1900 equatorial"));
    }

    #[test]
    fn supported_body_canonical_batch_parity_summary_for_report_surfaces_validation_errors() {
        let mut summary = supported_body_canonical_batch_parity_summary()
            .expect("supported-body canonical batch matrix should exist");
        summary.supported_body_count += 1;

        let rendered =
            format_validated_supported_body_canonical_batch_parity_summary_for_report(&summary);

        assert!(rendered.starts_with("VSOP87 supported-body canonical batch matrix: unavailable ("));
        assert!(rendered.contains("supported_body_count"));
    }

    #[test]
    fn source_body_class_evidence_summary_for_report_matches_the_backend_formatter() {
        let summary = source_body_class_evidence_summary().expect("summary should exist");
        let rendered = source_body_class_evidence_summary_for_report();

        assert_eq!(rendered, format_source_body_class_evidence_summary(&summary));
        assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
        assert!(rendered.contains("median Δlon="));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("median Δlat="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("median Δdist="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered.contains(
            "Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
        ));
    }

    #[test]
    fn source_body_class_evidence_summary_for_report_marks_drift_as_unavailable() {
        let mut summary = source_body_class_evidence_summary().expect("summary should exist");
        summary[0].sample_count += 1;

        assert_eq!(
            format_validated_source_body_class_evidence_summary_for_report(&summary),
            "VSOP87 source-backed body-class envelopes: unavailable (the VSOP87 source-backed body-class evidence summary field `sample_count` is out of sync with the current canonical evidence)"
        );
    }
}
