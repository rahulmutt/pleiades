use super::ReleasePosture;
use crate::test_support::metadata_with_claims;
use crate::{AccuracyClass, BackendMetadata, BodyClaim, CelestialBody, ClaimEvidence};

fn meta_with(id: &str, claims: Vec<BodyClaim>) -> BackendMetadata {
    metadata_with_claims(id, claims)
}

#[test]
fn posture_collects_release_grade_per_backend() {
    let a = meta_with(
        "packaged-data",
        vec![BodyClaim::release_grade(
            CelestialBody::Pluto,
            AccuracyClass::High,
            ClaimEvidence::ArtifactValidated,
        )],
    );
    let b = meta_with(
        "pleiades-vsop87",
        vec![BodyClaim::approximate(CelestialBody::Pluto)],
    );
    let posture = ReleasePosture::from_backends(&[&a, &b]);
    let rg = posture.release_grade();
    assert!(rg
        .iter()
        .any(|(id, body)| id.as_str() == "packaged-data" && body == &CelestialBody::Pluto));
    assert!(!rg.iter().any(|(id, _)| id.as_str() == "pleiades-vsop87"));
}

#[test]
fn summary_line_is_deterministic() {
    let a = meta_with(
        "packaged-data",
        vec![BodyClaim::release_grade(
            CelestialBody::Moon,
            AccuracyClass::High,
            ClaimEvidence::ArtifactValidated,
        )],
    );
    let p1 = ReleasePosture::from_backends(&[&a]);
    let p2 = ReleasePosture::from_backends(&[&a]);
    assert_eq!(p1.summary_line(), p2.summary_line());
    assert!(p1.summary_line().contains("Moon"));
    assert!(p1.summary_line().contains("ReleaseGrade"));
}
