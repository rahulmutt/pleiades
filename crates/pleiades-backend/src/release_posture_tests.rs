use super::ReleasePosture;
use crate::test_support::metadata_with_claims;
use crate::{
    AccuracyClass, BackendMetadata, BodyClaim, BodyClaimTier, CelestialBody, ClaimEvidence,
};

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

#[test]
fn summary_line_stable_across_input_orderings() {
    let a = meta_with(
        "backend-alpha",
        vec![BodyClaim::release_grade(
            CelestialBody::Moon,
            AccuracyClass::High,
            ClaimEvidence::ArtifactValidated,
        )],
    );
    let b = meta_with(
        "backend-beta",
        vec![BodyClaim::release_grade(
            CelestialBody::Sun,
            AccuracyClass::High,
            ClaimEvidence::ArtifactValidated,
        )],
    );
    let p_ab = ReleasePosture::from_backends(&[&a, &b]);
    let p_ba = ReleasePosture::from_backends(&[&b, &a]);
    assert_eq!(
        p_ab.summary_line(),
        p_ba.summary_line(),
        "summary_line() must be identical regardless of input ordering"
    );
}

#[test]
fn tier_views_group_each_claim_under_its_own_tier() {
    let a = meta_with(
        "packaged-data",
        vec![
            BodyClaim::release_grade(
                CelestialBody::Sun,
                AccuracyClass::High,
                ClaimEvidence::ArtifactValidated,
            ),
            BodyClaim::approximate(CelestialBody::Mars),
        ],
    );
    let b = meta_with(
        "pleiades-vsop87",
        vec![
            BodyClaim::constrained(
                CelestialBody::Moon,
                AccuracyClass::Moderate,
                ClaimEvidence::AlgorithmicModel,
            ),
            BodyClaim::unsupported(CelestialBody::Pluto),
        ],
    );
    let posture = ReleasePosture::from_backends(&[&a, &b]);

    let constrained: Vec<(String, CelestialBody)> = posture
        .claims_for_tier(BodyClaimTier::Constrained)
        .into_iter()
        .map(|(id, claim)| (id.as_str().to_string(), claim.body))
        .collect();
    assert_eq!(
        constrained,
        [("pleiades-vsop87".to_string(), CelestialBody::Moon)]
    );
    assert_eq!(
        posture.summary_line(),
        "ReleaseGrade: [Sun@packaged-data]; Constrained: [Moon@pleiades-vsop87]; \
         Approximate: [Mars@packaged-data]; Unsupported: [Pluto@pleiades-vsop87]"
    );
}
