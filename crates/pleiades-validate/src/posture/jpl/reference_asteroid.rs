//! Relocated reference-asteroid renderers copied from
//! `pleiades-jpl::reference_summary::reference_asteroid` (report-surface
//! relocation program, Slice D). Rendering only — the functional crate keeps
//! the structured evidence structs, their `*_details()` constructors,
//! `validate()`/`label()` methods, and all release-gate data; jpl's own
//! rendering stays in place until the Task 14 contract sweep.

use pleiades_jpl::{
    ReferenceAsteroidEquatorialEvidenceSummary, ReferenceAsteroidEvidenceSummary,
    ReferenceAsteroidSourceWindowSummary, ReferenceSnapshotSourceWindow,
};

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling
/// `comparison.rs`, `holdout.rs`, and `jpl_posture.rs` posture modules.
fn format_instant(instant: pleiades_types::Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Reproduced from jpl's `pub(crate)` `format_bodies`
/// (`reference_summary/reference_snapshot/core/general_a.rs:502`), which is
/// not callable cross-crate. Per-module duplicate accepted (Slice D expand)
/// — already reproduced identically in the sibling `comparison.rs`,
/// `holdout.rs`, and `jpl_posture.rs` posture modules.
fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Compact release-facing summary line for the exact asteroid evidence
/// slice. Verbatim copy of `ReferenceAsteroidEvidenceSummary::summary_line`
/// (reference_summary/reference_asteroid.rs:184).
pub(crate) fn reference_asteroid_evidence_summary_line(
    s: &ReferenceAsteroidEvidenceSummary,
) -> String {
    format!(
        "Selected asteroid evidence: {} exact J2000 samples at {} ({})",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
    )
}

/// Compact release-facing summary line for the equatorial asteroid evidence
/// slice. Verbatim copy of
/// `ReferenceAsteroidEquatorialEvidenceSummary::summary_line`
/// (reference_summary/reference_asteroid.rs:345).
pub(crate) fn reference_asteroid_equatorial_evidence_summary_line(
    s: &ReferenceAsteroidEquatorialEvidenceSummary,
) -> String {
    format!(
        "Selected asteroid equatorial evidence: {} exact J2000 samples at {} ({}) using a {}",
        s.sample_count,
        format_instant(s.epoch),
        format_bodies(&s.sample_bodies),
        s.transform_note,
    )
}

/// Compact release-facing summary line for the reference asteroid source
/// coverage. Verbatim copy of
/// `ReferenceAsteroidSourceWindowSummary::summary_line`
/// (reference_summary/reference_asteroid.rs:888). The nested
/// `ReferenceSnapshotSourceWindow::summary_line` call is left as a jpl
/// method call — that struct's rendering has not moved to validate yet
/// (Task 8's `reference_snapshot/core/general_b.rs` slice); validate→jpl is
/// allowed and byte-identical here.
pub(crate) fn reference_asteroid_source_window_summary_line(
    s: &ReferenceAsteroidSourceWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(ReferenceSnapshotSourceWindow::summary_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Reference asteroid source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Returns the release-facing exact asteroid evidence summary string.
/// Verbatim copy of jpl's `reference_asteroid_evidence_summary_for_report`
/// (reference_summary/reference_asteroid.rs:820), with the
/// `validate_reference_asteroid_evidence` gate promoted to `pub` (Slice D
/// Task 7) and called instead of reproduced, and
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local).
pub fn reference_asteroid_evidence_summary_for_report() -> String {
    let evidence = pleiades_jpl::reference_asteroid_evidence();
    match pleiades_jpl::validate_reference_asteroid_evidence(evidence) {
        Ok(()) => match pleiades_jpl::reference_asteroid_evidence_summary() {
            Some(summary) => match summary.validate() {
                Ok(()) => reference_asteroid_evidence_summary_line(&summary),
                Err(error) => format!("Selected asteroid evidence: unavailable ({error})"),
            },
            None => "Selected asteroid evidence: unavailable".to_string(),
        },
        Err(error) => format!("Selected asteroid evidence: unavailable ({error})"),
    }
}

/// Returns the release-facing equatorial asteroid evidence summary string.
/// Verbatim copy of jpl's
/// `reference_asteroid_equatorial_evidence_summary_for_report`
/// (reference_summary/reference_asteroid.rs:853), with the
/// `validate_reference_asteroid_equatorial_evidence` gate promoted to `pub`
/// (Slice D Task 7) and called instead of reproduced, and
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }`.
pub fn reference_asteroid_equatorial_evidence_summary_for_report() -> String {
    let evidence = pleiades_jpl::reference_asteroid_equatorial_evidence();
    match pleiades_jpl::validate_reference_asteroid_equatorial_evidence(evidence) {
        Ok(()) => match pleiades_jpl::reference_asteroid_equatorial_evidence_summary() {
            Some(summary) => match summary.validate() {
                Ok(()) => reference_asteroid_equatorial_evidence_summary_line(&summary),
                Err(error) => {
                    format!("Selected asteroid equatorial evidence: unavailable ({error})")
                }
            },
            None => "Selected asteroid equatorial evidence: unavailable".to_string(),
        },
        Err(error) => format!("Selected asteroid equatorial evidence: unavailable ({error})"),
    }
}

/// Returns the validated release-facing reference asteroid source window
/// summary string. Verbatim copy of jpl's
/// `validated_reference_asteroid_source_window_summary_for_report`
/// (reference_summary/reference_asteroid.rs:1053), with
/// `summary.validated_summary_line()` rewired to
/// `{ summary.validate().map_err(|e| e.to_string())?; Ok(<local render>) }`
/// (`validate()` stays on the jpl struct; rendering is local).
pub(crate) fn validated_reference_asteroid_source_window_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::reference_asteroid_source_window_summary()
        .ok_or_else(|| "reference asteroid source windows unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(reference_asteroid_source_window_summary_line(&summary))
}

/// Returns the release-facing reference asteroid source window summary
/// string. Verbatim copy of jpl's
/// `reference_asteroid_source_window_summary_for_report`
/// (reference_summary/reference_asteroid.rs:1062).
pub fn reference_asteroid_source_window_summary_for_report() -> String {
    match validated_reference_asteroid_source_window_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) if error == "reference asteroid source windows unavailable" => {
            "Reference asteroid source windows: unavailable".to_string()
        }
        Err(error) => format!("Reference asteroid source windows: unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_asteroid_evidence_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::reference_asteroid_evidence_summary()
            .expect("reference asteroid evidence summary should exist");
        summary
            .validate()
            .expect("reference asteroid evidence summary should validate");
        assert_eq!(
            reference_asteroid_evidence_summary_line(&summary),
            "Selected asteroid evidence: 6 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        );
        assert_eq!(
            reference_asteroid_evidence_summary_for_report(),
            reference_asteroid_evidence_summary_line(&summary)
        );
    }

    #[test]
    fn reference_asteroid_source_window_summary_reports_the_expanded_coverage() {
        let summary = pleiades_jpl::reference_asteroid_source_window_summary()
            .expect("reference asteroid source window summary should exist");
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(summary.sample_count, 95);
        assert_eq!(summary.epoch_count, 17);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            reference_asteroid_source_window_summary_line(&summary),
            "Reference asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"
        );
        assert_eq!(
            reference_asteroid_source_window_summary_line(&summary),
            reference_asteroid_source_window_summary_for_report()
        );
        assert_eq!(
            validated_reference_asteroid_source_window_summary_for_report(),
            Ok(reference_asteroid_source_window_summary_line(&summary))
        );
    }

    #[test]
    fn reference_asteroid_source_window_summary_validation_rejects_custom_body_drift() {
        let mut summary = pleiades_jpl::reference_asteroid_source_window_summary()
            .expect("reference asteroid source window summary should exist");
        summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies"
                }
            )
        ));
    }

    #[test]
    fn reference_asteroid_source_window_summary_validation_rejects_sample_body_order_drift() {
        let mut summary = pleiades_jpl::reference_asteroid_source_window_summary()
            .expect("reference asteroid source window summary should exist");
        summary.sample_bodies.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies"
                }
            )
        ));
    }

    #[test]
    fn reference_asteroid_source_window_summary_validation_rejects_window_order_drift() {
        let mut summary = pleiades_jpl::reference_asteroid_source_window_summary()
            .expect("reference asteroid source window summary should exist");
        summary.windows.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows"
                }
            )
        ));
    }

    #[test]
    fn reference_asteroid_evidence_summary_validation_rejects_body_order_drift() {
        let mut summary = pleiades_jpl::reference_asteroid_evidence_summary()
            .expect("reference asteroid evidence summary should exist");
        summary.sample_bodies.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceAsteroidEvidenceSummaryValidationError::BodyOrderMismatch {
                    index: 0,
                    ..
                }
            )
        ));
    }

    #[test]
    fn reference_asteroid_evidence_validation_rejects_body_order_drift() {
        let mut evidence = pleiades_jpl::reference_asteroid_evidence().to_vec();
        evidence.swap(0, 1);

        assert!(matches!(
            pleiades_jpl::validate_reference_asteroid_evidence(&evidence),
            Err(
                pleiades_jpl::ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch {
                    index: 0,
                    ..
                }
            )
        ));
    }

    #[test]
    fn reference_asteroid_equatorial_evidence_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::reference_asteroid_equatorial_evidence_summary()
            .expect("reference asteroid equatorial evidence summary should exist");
        summary
            .validate()
            .expect("reference asteroid equatorial evidence summary should validate");
        assert_eq!(
            reference_asteroid_equatorial_evidence_summary_line(&summary),
            "Selected asteroid equatorial evidence: 6 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) using a mean-obliquity equatorial transform"
        );
        assert_eq!(
            reference_asteroid_equatorial_evidence_summary_for_report(),
            reference_asteroid_equatorial_evidence_summary_line(&summary)
        );
    }

    #[test]
    fn reference_asteroid_equatorial_evidence_summary_validation_rejects_transform_drift() {
        let mut summary = pleiades_jpl::reference_asteroid_equatorial_evidence_summary()
            .expect("reference asteroid equatorial evidence summary should exist");
        summary.transform_note = "broken transform";

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ReferenceAsteroidEquatorialEvidenceSummaryValidationError::TransformNoteMismatch {
                    expected: "mean-obliquity equatorial transform",
                    found: "broken transform",
                }
            )
        ));
    }

    #[test]
    fn reference_asteroid_equatorial_evidence_validation_rejects_transform_drift() {
        let mut evidence = pleiades_jpl::reference_asteroid_equatorial_evidence().to_vec();
        let shifted_right_ascension = pleiades_types::Angle::from_degrees(
            evidence[0].equatorial.right_ascension.degrees() + 0.01,
        );
        evidence[0].equatorial = pleiades_types::EquatorialCoordinates::new(
            shifted_right_ascension,
            evidence[0].equatorial.declination,
            evidence[0].equatorial.distance_au,
        );

        assert!(matches!(
            pleiades_jpl::validate_reference_asteroid_equatorial_evidence(&evidence),
            Err(
                pleiades_jpl::ReferenceAsteroidEquatorialEvidenceValidationError::RightAscensionMismatch {
                    index: 0,
                    ..
                }
            )
        ));
    }
}
