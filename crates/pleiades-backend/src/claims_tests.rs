use crate::claims::{BodyClaim, BodyClaimTier, ClaimEvidence};
use crate::{AccuracyClass, CelestialBody};

#[test]
fn tier_rank_orders_release_grade_strongest() {
    assert!(BodyClaimTier::ReleaseGrade.rank() > BodyClaimTier::Constrained.rank());
    assert!(BodyClaimTier::Constrained.rank() > BodyClaimTier::Approximate.rank());
    assert!(BodyClaimTier::Approximate.rank() > BodyClaimTier::Unsupported.rank());
}

#[test]
fn tier_labels_are_stable() {
    assert_eq!(BodyClaimTier::ReleaseGrade.label(), "ReleaseGrade");
    assert_eq!(BodyClaimTier::Unsupported.label(), "Unsupported");
}

#[test]
fn release_grade_constructor_sets_fields() {
    let claim = BodyClaim::release_grade(
        CelestialBody::Pluto,
        AccuracyClass::High,
        ClaimEvidence::ArtifactValidated,
    );
    assert_eq!(claim.body, CelestialBody::Pluto);
    assert_eq!(claim.tier, BodyClaimTier::ReleaseGrade);
    assert_eq!(claim.accuracy, AccuracyClass::High);
    assert_eq!(claim.evidence, ClaimEvidence::ArtifactValidated);
}

#[test]
fn from_celestial_body_defaults_to_constrained() {
    let claim: BodyClaim = CelestialBody::Sun.into();
    assert_eq!(claim.tier, BodyClaimTier::Constrained);
    assert_eq!(claim.accuracy, AccuracyClass::Unknown);
    assert_eq!(claim.evidence, ClaimEvidence::None);
}

#[test]
fn summary_line_mentions_body_tier_and_evidence() {
    let claim = BodyClaim::release_grade(
        CelestialBody::Ceres,
        AccuracyClass::High,
        ClaimEvidence::CorpusValidated {
            source: "sb441-n16".to_string(),
        },
    );
    let line = claim.summary_line();
    assert!(line.contains("Ceres"));
    assert!(line.contains("ReleaseGrade"));
    assert!(line.contains("sb441-n16"));
}
