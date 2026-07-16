//! Tests for the reference_asteroid module.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn reference_asteroid_evidence_summary_reports_the_expected_coverage() {
    let summary = reference_asteroid_evidence_summary()
        .expect("reference asteroid evidence summary should exist");
    summary
        .validate()
        .expect("reference asteroid evidence summary should validate");
}

#[test]
fn reference_asteroid_source_window_summary_reports_the_expanded_coverage() {
    let summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    assert_eq!(summary.windows.len(), summary.sample_bodies.len());
    assert_eq!(summary.sample_count, 95);
    assert_eq!(summary.epoch_count, 17);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn reference_asteroid_source_window_summary_validation_rejects_custom_body_drift() {
    let mut summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
}

#[test]
fn reference_asteroid_source_window_summary_validation_rejects_sample_body_order_drift() {
    let mut summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
}

#[test]
fn reference_asteroid_source_window_summary_validation_rejects_window_order_drift() {
    let mut summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    summary.windows.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "windows"
            }
        )
    ));
}

#[test]
fn reference_asteroid_evidence_summary_validation_rejects_body_order_drift() {
    let mut summary = reference_asteroid_evidence_summary()
        .expect("reference asteroid evidence summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(ReferenceAsteroidEvidenceSummaryValidationError::BodyOrderMismatch { index: 0, .. })
    ));
}

#[test]
fn reference_asteroid_evidence_validation_rejects_body_order_drift() {
    let mut evidence = reference_asteroid_evidence().to_vec();
    evidence.swap(0, 1);

    assert!(matches!(
        validate_reference_asteroid_evidence(&evidence),
        Err(ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch { index: 0, .. })
    ));
}

#[test]
fn reference_asteroid_equatorial_evidence_summary_reports_the_expected_coverage() {
    let summary = reference_asteroid_equatorial_evidence_summary()
        .expect("reference asteroid equatorial evidence summary should exist");
    summary
        .validate()
        .expect("reference asteroid equatorial evidence summary should validate");
}

#[test]
fn reference_asteroid_equatorial_evidence_summary_validation_rejects_transform_drift() {
    let mut summary = reference_asteroid_equatorial_evidence_summary()
        .expect("reference asteroid equatorial evidence summary should exist");
    summary.transform_note = "broken transform";

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidEquatorialEvidenceSummaryValidationError::TransformNoteMismatch {
                expected: "mean-obliquity equatorial transform",
                found: "broken transform",
            }
        )
    ));
}

#[test]
fn reference_asteroid_equatorial_evidence_validation_rejects_transform_drift() {
    let mut evidence = reference_asteroid_equatorial_evidence().to_vec();
    let shifted_right_ascension = pleiades_types::Angle::from_degrees(
        evidence[0].equatorial.right_ascension.degrees() + 0.01,
    );
    evidence[0].equatorial = pleiades_types::EquatorialCoordinates::new(
        shifted_right_ascension,
        evidence[0].equatorial.declination,
        evidence[0].equatorial.distance_au,
    );

    assert!(matches!(
        validate_reference_asteroid_equatorial_evidence(&evidence),
        Err(
            ReferenceAsteroidEquatorialEvidenceValidationError::RightAscensionMismatch {
                index: 0,
                ..
            }
        )
    ));
}

#[test]
fn batch_query_preserves_reference_asteroid_order_and_values() {
    let backend = JplSnapshotBackend;
    let evidence = reference_asteroid_evidence();
    let requests = evidence
        .iter()
        .map(|sample| EphemerisRequest {
            body: sample.body.clone(),
            instant: sample.epoch,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve the asteroid reference order");

    assert_eq!(results.len(), evidence.len());
    for (sample, result) in evidence.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - sample.distance_au).abs()
                < 1e-12
        );
    }
}

#[test]
fn reference_asteroid_evidence_exposes_exact_j2000_samples() {
    let evidence = reference_asteroid_evidence();
    assert_eq!(evidence.len(), 6);
    assert_eq!(reference_asteroids().len(), evidence.len());
    assert!(evidence.iter().all(|sample| {
        sample.epoch.julian_day.days() == REFERENCE_EPOCH_JD
            && sample.longitude_deg.is_finite()
            && sample.latitude_deg.is_finite()
            && sample.distance_au.is_finite()
    }));
    assert_eq!(evidence[0].body, pleiades_backend::CelestialBody::Ceres);
    assert_eq!(evidence[1].body, pleiades_backend::CelestialBody::Pallas);
    assert_eq!(evidence[2].body, pleiades_backend::CelestialBody::Juno);
    assert_eq!(evidence[3].body, pleiades_backend::CelestialBody::Vesta);
    assert_eq!(
        evidence[4].body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
    assert_eq!(
        evidence[5].body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
    );
    assert!((evidence[0].longitude_deg - 184.459642854516).abs() < 1e-12);
    assert!((evidence[4].distance_au - 1.854402724550437).abs() < 1e-12);
}

#[test]
fn reference_asteroid_requests_preserve_the_exact_j2000_slice() {
    let backend = JplSnapshotBackend;
    let requests = reference_asteroid_requests(CoordinateFrame::Equatorial)
        .expect("selected asteroid requests should exist");
    let results = backend
        .positions(&requests)
        .expect("selected asteroid batch query should preserve the exact J2000 slice");

    assert_eq!(results.len(), requests.len());
    for ((sample, result), request) in reference_asteroid_evidence()
        .iter()
        .zip(results.iter())
        .zip(requests.iter())
    {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, sample.epoch);
        assert_eq!(result.frame, request.frame);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("selected asteroid batch rows should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - sample.distance_au).abs()
                < 1e-12
        );

        let equatorial = result
            .equatorial
            .expect("selected asteroid batch rows should include equatorial coordinates");
        let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
        assert_eq!(equatorial, expected_equatorial);
    }
}

#[test]
fn reference_asteroid_batch_parity_requests_preserve_the_selected_j2000_slice() {
    let backend = JplSnapshotBackend;
    let requests = reference_asteroid_batch_parity_requests()
        .expect("selected asteroid batch parity requests should exist");
    let results = backend
        .positions(&requests)
        .expect("mixed-frame selected asteroid batch query should preserve the exact slice");

    assert_eq!(results.len(), requests.len());
    for ((sample, result), request) in reference_asteroid_evidence()
        .iter()
        .zip(results.iter())
        .zip(requests.iter())
    {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, sample.epoch);
        assert_eq!(result.frame, request.frame);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("selected asteroid batch rows should include ecliptic coordinates");
        let expected = EclipticCoordinates::new(
            Longitude::from_degrees(sample.longitude_deg),
            Latitude::from_degrees(sample.latitude_deg),
            Some(sample.distance_au),
        );
        assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist")
                - expected
                    .distance_au
                    .expect("expected distance should exist"))
            .abs()
                < 1e-12
        );

        let equatorial = result
            .equatorial
            .expect("selected asteroid batch rows should include equatorial coordinates");
        assert_eq!(
            equatorial,
            ecliptic.to_equatorial(result.instant.mean_obliquity())
        );
    }
}
