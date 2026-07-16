//! Relocated reference-snapshot core evidence renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::core::evidence`
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()` constructors, `validate()`/`label()` methods, and all
//! release-gate data; jpl's own rendering stays in place until the Task 14
//! contract sweep.

use pleiades_jpl::{
    ReferenceSnapshotExactJ2000BodyClassCoverageSummary, ReferenceSnapshotExactJ2000EvidenceSummary,
};

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling posture
/// modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Reproduced from jpl's `pub(crate)` `format_bodies`
/// (`reference_summary/reference_snapshot/core/general_a.rs:510`), which is
/// not callable cross-crate. Per-module duplicate accepted (Slice D expand)
/// — already reproduced identically in the sibling posture modules.
fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Compact release-facing summary line for the exact J2000 reference
/// snapshot slice. Verbatim copy of
/// `ReferenceSnapshotExactJ2000EvidenceSummary::summary_line`
/// (reference_summary/reference_snapshot/core/evidence.rs:93).
pub(crate) fn reference_snapshot_exact_j2000_evidence_summary_line(
    s: &ReferenceSnapshotExactJ2000EvidenceSummary,
) -> String {
    format!(
        "Reference snapshot exact J2000 evidence: {} exact J2000 samples at {} ({})",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Returns the validated release-facing reference snapshot exact J2000
/// evidence summary string. Verbatim copy of jpl's
/// `validated_reference_snapshot_exact_j2000_evidence_summary_for_report`
/// (reference_summary/reference_snapshot/core/evidence.rs:196), with
/// `summary.validated_summary_line()` rewired to
/// `{ summary.validate().map_err(|e| e.to_string())?; Ok(<local render>) }`
/// (`validate()` stays on the jpl struct; rendering is local).
pub(crate) fn validated_reference_snapshot_exact_j2000_evidence_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::reference_snapshot_exact_j2000_evidence_summary()
        .ok_or_else(|| "reference snapshot exact J2000 evidence unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(reference_snapshot_exact_j2000_evidence_summary_line(
        &summary,
    ))
}

/// Returns the release-facing reference snapshot exact J2000 evidence
/// summary string. Verbatim copy of jpl's
/// `reference_snapshot_exact_j2000_evidence_summary_for_report`
/// (reference_summary/reference_snapshot/core/evidence.rs:206).
pub fn reference_snapshot_exact_j2000_evidence_summary_for_report() -> String {
    match validated_reference_snapshot_exact_j2000_evidence_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) if error == "reference snapshot exact J2000 evidence unavailable" => {
            "Reference snapshot exact J2000 evidence: unavailable".to_string()
        }
        Err(error) => format!("Reference snapshot exact J2000 evidence: unavailable ({error})"),
    }
}

/// Compact release-facing summary line for the exact J2000 reference
/// snapshot body classes. Verbatim copy of
/// `ReferenceSnapshotExactJ2000BodyClassCoverageSummary::summary_line`
/// (reference_summary/reference_snapshot/core/evidence.rs:325).
pub(crate) fn reference_snapshot_exact_j2000_body_class_coverage_summary_line(
    s: &ReferenceSnapshotExactJ2000BodyClassCoverageSummary,
) -> String {
    format!(
        "Reference snapshot exact J2000 body-class coverage: {} major-body samples across {} bodies and 1 epoch ({}); {} selected-asteroid samples across {} bodies and 1 epoch ({})",
        s.major_body_row_count,
        s.major_bodies.len(),
        format_bodies(&s.major_bodies),
        s.asteroid_row_count,
        s.asteroid_bodies.len(),
        format_bodies(&s.asteroid_bodies),
    )
}

/// Returns the validated release-facing exact J2000 body-class coverage
/// summary string. Verbatim copy of jpl's
/// `validated_reference_snapshot_exact_j2000_body_class_coverage_summary_for_report`
/// (reference_summary/reference_snapshot/core/evidence.rs:485), with
/// `summary.validated_summary_line()` rewired to
/// `{ summary.validate().map_err(|e| e.to_string())?; Ok(<local render>) }`.
pub(crate) fn validated_reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::reference_snapshot_exact_j2000_body_class_coverage_summary()
        .ok_or_else(|| {
            "reference snapshot exact J2000 body-class coverage unavailable".to_string()
        })?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(reference_snapshot_exact_j2000_body_class_coverage_summary_line(&summary))
}

/// Returns the release-facing exact J2000 body-class coverage summary
/// string. Verbatim copy of jpl's
/// `reference_snapshot_exact_j2000_body_class_coverage_summary_for_report`
/// (reference_summary/reference_snapshot/core/evidence.rs:497).
pub(crate) fn reference_snapshot_exact_j2000_body_class_coverage_summary_for_report() -> String {
    match validated_reference_snapshot_exact_j2000_body_class_coverage_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) if error == "reference snapshot exact J2000 body-class coverage unavailable" => {
            "Reference snapshot exact J2000 body-class coverage: unavailable".to_string()
        }
        Err(error) => {
            format!("Reference snapshot exact J2000 body-class coverage: unavailable ({error})")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_snapshot_exact_j2000_evidence_reports_the_expected_slice() {
        let summary = pleiades_jpl::reference_snapshot_exact_j2000_evidence_summary()
            .expect("reference snapshot exact J2000 evidence should exist");
        summary
            .validate()
            .expect("reference snapshot exact J2000 evidence should validate");
        assert_eq!(summary.sample_count, 16);
        assert_eq!(summary.sample_bodies, pleiades_jpl::reference_bodies());
        assert_eq!(summary.epoch.julian_day.days(), 2_451_545.0);
        assert_eq!(
            reference_snapshot_exact_j2000_evidence_summary_line(&summary),
            format!(
                "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0 (TDB) ({})",
                format_bodies(pleiades_jpl::reference_bodies())
            )
        );
        assert_eq!(
            validated_reference_snapshot_exact_j2000_evidence_summary_for_report(),
            Ok(reference_snapshot_exact_j2000_evidence_summary_line(
                &summary
            ))
        );
        assert_eq!(
            reference_snapshot_exact_j2000_evidence_summary_for_report(),
            reference_snapshot_exact_j2000_evidence_summary_line(&summary)
        );
    }

    #[test]
    fn reference_snapshot_exact_j2000_body_class_coverage_reports_the_expected_slice() {
        let summary = pleiades_jpl::reference_snapshot_exact_j2000_body_class_coverage_summary()
            .expect("reference snapshot exact J2000 body-class coverage should exist");
        summary
            .validate()
            .expect("reference snapshot exact J2000 body-class coverage should validate");
        assert_eq!(summary.major_body_row_count, 10);
        assert_eq!(
            summary.major_bodies,
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Pluto,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
            ]
        );
        assert_eq!(summary.asteroid_row_count, 6);
        assert_eq!(
            summary.asteroid_bodies,
            pleiades_jpl::reference_asteroids().to_vec()
        );
        assert_eq!(summary.epoch.julian_day.days(), 2_451_545.0);
        assert_eq!(
            reference_snapshot_exact_j2000_body_class_coverage_summary_line(&summary),
            format!(
                "Reference snapshot exact J2000 body-class coverage: 10 major-body samples across 10 bodies and 1 epoch ({}); 6 selected-asteroid samples across 6 bodies and 1 epoch ({})",
                format_bodies(&summary.major_bodies),
                format_bodies(&summary.asteroid_bodies)
            )
        );
        assert_eq!(
            validated_reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(),
            Ok(reference_snapshot_exact_j2000_body_class_coverage_summary_line(&summary))
        );
        assert_eq!(
            reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(),
            reference_snapshot_exact_j2000_body_class_coverage_summary_line(&summary)
        );
    }
}
