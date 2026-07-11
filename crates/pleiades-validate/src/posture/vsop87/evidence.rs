//! VSOP87 canonical-evidence report prose relocated from
//! `pleiades-vsop87::source_docs::evidence` (report-surface relocation
//! program, Slice B). Rendering only — the functional crate keeps the
//! structured data and their constructors.
//!
//! Three of the moved formatters (`format_canonical_epoch_evidence_summary`,
//! `format_canonical_equatorial_evidence_summary`,
//! `format_source_body_evidence_summary`) could not be deleted from
//! `pleiades-vsop87` outright: the retained inherent methods
//! `Vsop87CanonicalEvidenceSummary::summary_line()`,
//! `Vsop87CanonicalEquatorialEvidenceSummary::summary_line()`, and
//! `Vsop87SourceBodyEvidenceSummary::summary_line()` delegated to them and
//! are exercised throughout `pleiades-vsop87`'s own retained tests.
//! `pleiades-vsop87` cannot depend on `pleiades-validate`, so the rendering
//! logic was inlined into those inherent methods (unchanged format strings)
//! and this module carries a byte-identical copy for the release-facing
//! free-function path. `format_canonical_equatorial_body_class_evidence_summary`
//! renders each entry via the retained *public*
//! `Vsop87CanonicalEquatorialBodyClassEvidenceSummary::summary_line()` method
//! (mirroring the `format_source_body_class_evidence_summary` pattern in the
//! `batch_parity` module), so no local copy of the crate-private per-entry
//! rendering helper is needed here. `format_celestial_bodies` is likewise a
//! byte-identical local copy of the crate-private helper of the same name in
//! `pleiades-vsop87`.

use pleiades_types::CelestialBody;

use pleiades_vsop87::{
    canonical_epoch_equatorial_body_class_evidence_summary,
    canonical_epoch_equatorial_evidence_summary, canonical_epoch_evidence_summary,
    canonical_epoch_outlier_summary, source_body_evidence_summary,
    Vsop87CanonicalEquatorialBodyClassEvidenceSummary, Vsop87CanonicalEquatorialEvidenceSummary,
    Vsop87CanonicalEvidenceSummary, Vsop87SourceBodyEvidenceSummary,
};

const CANONICAL_EVIDENCE_SUMMARY_LABEL: &str = "VSOP87 canonical J2000 source-backed evidence";
const CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL: &str =
    "VSOP87 canonical J2000 equatorial companion evidence";
const CANONICAL_OUTLIER_NOTE_LABEL: &str = "VSOP87 canonical J2000 interim outliers";

fn format_celestial_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Formats the canonical VSOP87 J2000 evidence summary for reporting.
pub(crate) fn format_canonical_epoch_evidence_summary(
    summary: &Vsop87CanonicalEvidenceSummary,
) -> String {
    format!(
        "VSOP87 canonical J2000 source-backed evidence: {} samples, bodies: {}, status {}, mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, out-of-limit samples {}, max Δlon={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δlat={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δdist={:.12} AU (limit {:.12} AU, margin {:+.12} AU; {}; {}; {})",
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        if summary.within_interim_limits {
            "within interim limits"
        } else {
            "outside interim limits"
        },
        summary.mean_longitude_delta_deg,
        summary.median_longitude_delta_deg,
        summary.percentile_longitude_delta_deg,
        summary.rms_longitude_delta_deg,
        summary.mean_latitude_delta_deg,
        summary.median_latitude_delta_deg,
        summary.percentile_latitude_delta_deg,
        summary.rms_latitude_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.out_of_limit_count,
        summary.max_longitude_delta_deg,
        summary.max_longitude_delta_limit_deg,
        summary.max_longitude_delta_limit_deg - summary.max_longitude_delta_deg,
        summary.max_longitude_delta_body,
        summary.max_longitude_delta_source_kind,
        summary.max_longitude_delta_source_file,
        summary.max_latitude_delta_deg,
        summary.max_latitude_delta_limit_deg,
        summary.max_latitude_delta_limit_deg - summary.max_latitude_delta_deg,
        summary.max_latitude_delta_body,
        summary.max_latitude_delta_source_kind,
        summary.max_latitude_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_limit_au,
        summary.max_distance_delta_limit_au - summary.max_distance_delta_au,
        summary.max_distance_delta_body,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}

pub(crate) fn format_validated_canonical_epoch_evidence_summary_for_report(
    summary: &Vsop87CanonicalEvidenceSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("{CANONICAL_EVIDENCE_SUMMARY_LABEL}: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical VSOP87 J2000 evidence summary string.
pub(crate) fn canonical_epoch_evidence_summary_for_report() -> String {
    match canonical_epoch_evidence_summary() {
        Some(summary) => format_validated_canonical_epoch_evidence_summary_for_report(&summary),
        None => format!("{CANONICAL_EVIDENCE_SUMMARY_LABEL}: unavailable"),
    }
}

/// Returns a concise note describing any canonical J2000 bodies outside the
/// current interim limits.
pub(crate) fn canonical_epoch_outlier_note_for_report() -> String {
    match canonical_epoch_outlier_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(line) => line,
            Err(error) => format!("{CANONICAL_OUTLIER_NOTE_LABEL}: unavailable ({error})"),
        },
        None => format!("{CANONICAL_OUTLIER_NOTE_LABEL}: unavailable"),
    }
}

fn format_validated_canonical_epoch_equatorial_evidence_summary_for_report(
    summary: &Vsop87CanonicalEquatorialEvidenceSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("{CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL}: unavailable ({error})")
        }
    }
}

/// Returns the release-facing canonical VSOP87 equatorial companion evidence
/// summary string.
pub(crate) fn canonical_epoch_equatorial_evidence_summary_for_report() -> String {
    match canonical_epoch_equatorial_evidence_summary() {
        Some(summary) => {
            format_validated_canonical_epoch_equatorial_evidence_summary_for_report(&summary)
        }
        None => format!("{CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL}: unavailable"),
    }
}

/// Formats the canonical VSOP87 equatorial body-class evidence for reporting.
///
/// Each entry renders via the retained public
/// `Vsop87CanonicalEquatorialBodyClassEvidenceSummary::summary_line()`
/// method rather than a local copy of the crate-private per-entry helper
/// (mirrors `batch_parity::format_source_body_class_evidence_summary`).
pub(crate) fn format_canonical_equatorial_body_class_evidence_summary(
    summaries: &[Vsop87CanonicalEquatorialBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable".to_string();
    }

    let rendered = summaries
        .iter()
        .map(Vsop87CanonicalEquatorialBodyClassEvidenceSummary::summary_line)
        .collect::<Vec<_>>()
        .join(" | ");

    format!("VSOP87 canonical J2000 equatorial body-class envelopes: {rendered}")
}

fn format_validated_canonical_equatorial_body_class_evidence_summary_for_report(
    summaries: &[Vsop87CanonicalEquatorialBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable".to_string();
    }

    if let Some(error) = summaries
        .iter()
        .find_map(|summary| summary.validate().err())
    {
        return format!(
            "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable ({error})"
        );
    }

    format_canonical_equatorial_body_class_evidence_summary(summaries)
}

/// Returns the release-facing equatorial body-class evidence summary string.
pub(crate) fn canonical_epoch_equatorial_body_class_evidence_summary_for_report() -> String {
    match canonical_epoch_equatorial_body_class_evidence_summary() {
        Some(summary) => {
            format_validated_canonical_equatorial_body_class_evidence_summary_for_report(&summary)
        }
        None => "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable".to_string(),
    }
}

/// Formats the canonical VSOP87 J2000 equatorial companion summary for reporting.
pub(crate) fn format_canonical_equatorial_evidence_summary(
    summary: &Vsop87CanonicalEquatorialEvidenceSummary,
) -> String {
    format!(
        "VSOP87 canonical J2000 equatorial companion evidence: {} samples, bodies: {}, mean Δra={:.12}°, median Δra={:.12}°, p95 Δra={:.12}°, rms Δra={:.12}°, mean Δdec={:.12}°, median Δdec={:.12}°, p95 Δdec={:.12}°, rms Δdec={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, max Δra={:.12}° ({}; {}; {}), max Δdec={:.12}° ({}; {}; {}), max Δdist={:.12} AU ({}; {}; {})",
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        summary.mean_right_ascension_delta_deg,
        summary.median_right_ascension_delta_deg,
        summary.percentile_right_ascension_delta_deg,
        summary.rms_right_ascension_delta_deg,
        summary.mean_declination_delta_deg,
        summary.median_declination_delta_deg,
        summary.percentile_declination_delta_deg,
        summary.rms_declination_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.max_right_ascension_delta_deg,
        summary.max_right_ascension_delta_body,
        summary.max_right_ascension_delta_source_kind,
        summary.max_right_ascension_delta_source_file,
        summary.max_declination_delta_deg,
        summary.max_declination_delta_body,
        summary.max_declination_delta_source_kind,
        summary.max_declination_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_body,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}

/// Formats the current VSOP87 body-evidence envelope for reporting.
pub(crate) fn format_source_body_evidence_summary(
    summary: &Vsop87SourceBodyEvidenceSummary,
) -> String {
    let outside_note = if summary.outside_interim_limit_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.outside_interim_limit_bodies)
    };

    let bodies = format_celestial_bodies(&summary.sample_bodies);

    if summary.generated_binary_count == 0 && summary.truncated_count == 0 {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    } else if summary.generated_binary_count > 0 && summary.truncated_count == 0 {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            summary.generated_binary_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    } else if summary.generated_binary_count == 0 {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} truncated slice), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            summary.truncated_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    } else {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary, {} truncated slice), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            summary.generated_binary_count,
            summary.truncated_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    }
}

fn format_validated_source_body_evidence_summary_for_report(
    summary: &Vsop87SourceBodyEvidenceSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("VSOP87 source-backed body evidence: unavailable ({error})"),
    }
}

/// Returns the release-facing source-body evidence summary string.
pub(crate) fn source_body_evidence_summary_for_report() -> String {
    match source_body_evidence_summary() {
        Some(summary) => format_validated_source_body_evidence_summary_for_report(&summary),
        None => "VSOP87 source-backed body evidence: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_source_body_evidence_summary_lists_the_source_backed_body_order() {
        let summary = source_body_evidence_summary().expect("summary should exist");
        assert!(format_source_body_evidence_summary(&summary).contains(
            "source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
        ));
    }

    #[test]
    fn source_body_evidence_summary_for_report_matches_the_backend_formatter() {
        let summary = source_body_evidence_summary().expect("summary should exist");
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            source_body_evidence_summary_for_report(),
            format_source_body_evidence_summary(&summary)
        );
        assert_eq!(
            summary.summary_line(),
            source_body_evidence_summary_for_report()
        );
        assert_eq!(
            summary.to_string(),
            source_body_evidence_summary_for_report()
        );
    }

    #[test]
    fn canonical_epoch_equatorial_body_class_evidence_summary_for_report_matches_the_backend_formatter(
    ) {
        let summary = canonical_epoch_equatorial_body_class_evidence_summary()
            .expect("summary should exist");
        let rendered = canonical_epoch_equatorial_body_class_evidence_summary_for_report();

        // Real cross-check: recompute the expected string by calling the
        // retained public `Vsop87CanonicalEquatorialBodyClassEvidenceSummary
        // ::summary_line()` method directly on each entry (independently of
        // this module's own `format_canonical_equatorial_body_class_evidence_summary`
        // wrapper under test), guarding against silent drift between the
        // release-facing renderer and the vsop87-owned per-entry rendering.
        let expected_from_retained_summary_line = format!(
            "VSOP87 canonical J2000 equatorial body-class envelopes: {}",
            summary
                .iter()
                .map(Vsop87CanonicalEquatorialBodyClassEvidenceSummary::summary_line)
                .collect::<Vec<_>>()
                .join(" | ")
        );
        assert_eq!(rendered, expected_from_retained_summary_line);
        assert_eq!(
            rendered,
            format_canonical_equatorial_body_class_evidence_summary(&summary)
        );
        assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
        assert!(rendered.contains("median Δra="));
        assert!(rendered.contains("p95 Δra="));
        assert!(rendered.contains("median Δdec="));
        assert!(rendered.contains("p95 Δdec="));
        assert!(rendered.contains("median Δdist="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered.contains(
            "Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
        ));
    }

    #[test]
    fn canonical_epoch_evidence_summary_for_report_matches_the_backend_formatter() {
        let summary = canonical_epoch_evidence_summary().expect("summary should exist");
        let rendered = canonical_epoch_evidence_summary_for_report();

        assert_eq!(rendered, format_canonical_epoch_evidence_summary(&summary));
        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered
            .contains("bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    }

    #[test]
    fn canonical_evidence_outlier_note_for_report_reports_the_current_interim_status() {
        let summary = canonical_epoch_outlier_summary().expect("outlier summary should exist");

        assert_eq!(
            canonical_epoch_outlier_note_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn canonical_epoch_equatorial_evidence_summary_for_report_matches_the_backend_formatter() {
        let summary = canonical_epoch_equatorial_evidence_summary()
            .expect("equatorial summary should exist");
        let rendered = canonical_epoch_equatorial_evidence_summary_for_report();

        assert_eq!(
            rendered,
            format_canonical_equatorial_evidence_summary(&summary)
        );
        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("p95 Δra="));
        assert!(rendered.contains("p95 Δdec="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered
            .contains("bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    }
}
