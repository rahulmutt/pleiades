//! Fast structural claim audit (no corpus access).
//!
//! This module audits the canonical release backends for two structural
//! invariants that can be checked without loading a kernel or touching the
//! comparison corpus:
//!
//! 1. **Tier/evidence consistency** — a `ReleaseGrade` claim must be backed
//!    by `ArtifactValidated` or `CorpusValidated{..}` evidence.
//! 2. **Unsupported-body rejection** — every body declared `Unsupported` must
//!    be rejected by the backend's own `validate_request` preflight.

use crate::claims::canonical_release_metadata;
use pleiades_backend::{BodyClaim, BodyClaimTier, ClaimEvidence};
use std::fmt;

/// Errors produced by the structural claim audit.
// These items are `pub` for Task 11 (corpus audit) which will call
// `audit_structural` and pattern-match `ClaimAuditError`.  Until then the
// items are only exercised by the in-module tests.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimAuditError {
    /// A backend declares a body in its claim list but cannot actually compute
    /// it. Reserved for Task 11; not produced by the structural audit here.
    #[allow(dead_code)] // constructed in Task 11 / reserved
    DeclaredBodyNotComputable {
        /// The backend identifier.
        backend: String,
        /// The body that is declared but not computable.
        body: String,
    },
    /// A backend's preflight (`validate_request`) accepted a request for a
    /// body that is declared `Unsupported` in the metadata.
    UnsupportedBodyAccepted {
        /// The backend identifier.
        backend: String,
        /// The body that was incorrectly accepted.
        body: String,
    },
    /// A `ReleaseGrade` claim is backed by evidence that does not meet the
    /// release bar (`ArtifactValidated` or `CorpusValidated{..}`).
    TierEvidenceMismatch {
        /// The backend identifier.
        backend: String,
        /// The body whose claim fails the tier/evidence check.
        body: String,
    },
}

impl fmt::Display for ClaimAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeclaredBodyNotComputable { backend, body } => write!(
                f,
                "backend `{backend}` declares body `{body}` but cannot compute it"
            ),
            Self::UnsupportedBodyAccepted { backend, body } => write!(
                f,
                "backend `{backend}` accepted a request for unsupported body `{body}`"
            ),
            Self::TierEvidenceMismatch { backend, body } => write!(
                f,
                "backend `{backend}` claims `{body}` as ReleaseGrade but evidence is not artifact- or corpus-validated"
            ),
        }
    }
}

impl std::error::Error for ClaimAuditError {}

/// Returns `Ok(())` when the claim's tier and evidence are consistent.
///
/// A `ReleaseGrade` claim is consistent only when evidence is
/// `ArtifactValidated` or `CorpusValidated{..}`.  Any other evidence
/// on a `ReleaseGrade` claim returns `Err(())`.  Non-`ReleaseGrade`
/// claims are always consistent and return `Ok(())`.
// Called by `audit_structural` and directly by the unit tests; exposed
// `pub` for Task 11 / future corpus audit callers.
#[allow(dead_code)]
pub fn tier_evidence_consistent(claim: &BodyClaim) -> Result<(), ()> {
    match (claim.tier, &claim.evidence) {
        (BodyClaimTier::ReleaseGrade, ClaimEvidence::ArtifactValidated)
        | (BodyClaimTier::ReleaseGrade, ClaimEvidence::CorpusValidated { .. }) => Ok(()),
        (BodyClaimTier::ReleaseGrade, _) => Err(()),
        _ => Ok(()),
    }
}

/// Runs the structural audit over all canonical release backends.
///
/// Checks performed (no corpus or kernel access):
/// - Tier/evidence consistency: every `ReleaseGrade` claim must be backed by
///   `ArtifactValidated` or `CorpusValidated` evidence.
/// - Unsupported-body rejection: every body declared `Unsupported` must be
///   rejected by the backend's own `validate_request` preflight.
///
/// Returns `Ok(())` when all backends pass.  Returns `Err(errors)` with the
/// full list of violations when any check fails.
// Exposed `pub` for Task 11 / future corpus audit callers.
#[allow(dead_code)]
pub fn audit_structural() -> Result<(), Vec<ClaimAuditError>> {
    let mut errors = Vec::new();

    for meta in canonical_release_metadata() {
        for claim in &meta.body_claims {
            // Check 1: tier/evidence consistency.
            if tier_evidence_consistent(claim).is_err() {
                errors.push(ClaimAuditError::TierEvidenceMismatch {
                    backend: meta.id.as_str().to_string(),
                    body: claim.body.to_string(),
                });
            }

            // Check 2: Unsupported-tier bodies must be rejected by preflight.
            if claim.tier == BodyClaimTier::Unsupported {
                let req = crate::claims::sample_request_for(&claim.body);
                if meta.validate_request(&req).is_ok() {
                    errors.push(ClaimAuditError::UnsupportedBodyAccepted {
                        backend: meta.id.as_str().to_string(),
                        body: claim.body.to_string(),
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_audit_passes_for_canonical_backends() {
        assert!(audit_structural().is_ok());
    }

    #[test]
    fn structural_audit_flags_release_grade_without_corpus_evidence() {
        use pleiades_backend::{AccuracyClass, BodyClaim, CelestialBody, ClaimEvidence};

        // A ReleaseGrade claim with ClaimEvidence::None must be rejected.
        let bad = BodyClaim::release_grade(
            CelestialBody::Mars,
            AccuracyClass::High,
            ClaimEvidence::None,
        );
        assert!(tier_evidence_consistent(&bad).is_err());

        // A ReleaseGrade claim with ClaimEvidence::AlgorithmicModel must be rejected.
        let bad2 = BodyClaim::release_grade(
            CelestialBody::Mars,
            AccuracyClass::High,
            ClaimEvidence::AlgorithmicModel,
        );
        assert!(tier_evidence_consistent(&bad2).is_err());
    }
}
