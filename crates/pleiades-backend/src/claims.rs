use crate::identity::AccuracyClass;
use core::fmt;
use pleiades_types::CelestialBody;

/// The release-claim status of a single body for a single backend.
///
/// This is orthogonal to [`AccuracyClass`]: accuracy describes the numeric
/// band, the tier describes what the project promises at release.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BodyClaimTier {
    /// Production claim: validated against the reference corpus to the SP3 ceiling.
    ReleaseGrade,
    /// Source-backed but below the release ceiling, or corpus/kernel dependent.
    Constrained,
    /// Algorithmic approximation; no release claim.
    Approximate,
    /// Explicitly not supported; preflight rejects requests for it.
    Unsupported,
}

impl BodyClaimTier {
    /// Returns a stable human-readable label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::ReleaseGrade => "ReleaseGrade",
            Self::Constrained => "Constrained",
            Self::Approximate => "Approximate",
            Self::Unsupported => "Unsupported",
        }
    }

    /// Returns the merge rank: stronger tiers win on backend body collisions.
    pub const fn rank(self) -> u8 {
        match self {
            Self::ReleaseGrade => 3,
            Self::Constrained => 2,
            Self::Approximate => 1,
            Self::Unsupported => 0,
        }
    }
}

impl fmt::Display for BodyClaimTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// The evidence backing a body claim.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClaimEvidence {
    /// Validated inside the packaged artifact build against the reference corpus.
    ArtifactValidated,
    /// Validated against a named reference corpus (e.g. `de440`, `sb441-n373s`).
    CorpusValidated {
        /// The reference source identifier.
        source: String,
    },
    /// Backed only by an algorithmic model (VSOP87, compact ELP).
    AlgorithmicModel,
    /// No release-relevant evidence.
    None,
}

impl ClaimEvidence {
    /// Returns a compact human-readable label.
    pub fn label(&self) -> String {
        match self {
            Self::ArtifactValidated => "artifact-validated".to_string(),
            Self::CorpusValidated { source } => format!("corpus-validated:{source}"),
            Self::AlgorithmicModel => "algorithmic-model".to_string(),
            Self::None => "none".to_string(),
        }
    }
}

impl fmt::Display for ClaimEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}

/// A single backend's claim about a single body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BodyClaim {
    /// The body this claim describes.
    pub body: CelestialBody,
    /// The release-claim tier.
    pub tier: BodyClaimTier,
    /// The per-body accuracy class.
    pub accuracy: AccuracyClass,
    /// The evidence backing the claim.
    pub evidence: ClaimEvidence,
}

impl BodyClaim {
    /// Creates a claim from explicit parts.
    pub fn new(
        body: CelestialBody,
        tier: BodyClaimTier,
        accuracy: AccuracyClass,
        evidence: ClaimEvidence,
    ) -> Self {
        Self {
            body,
            tier,
            accuracy,
            evidence,
        }
    }

    /// Creates a `ReleaseGrade` claim.
    pub fn release_grade(
        body: CelestialBody,
        accuracy: AccuracyClass,
        evidence: ClaimEvidence,
    ) -> Self {
        Self::new(body, BodyClaimTier::ReleaseGrade, accuracy, evidence)
    }

    /// Creates a `Constrained` claim.
    pub fn constrained(
        body: CelestialBody,
        accuracy: AccuracyClass,
        evidence: ClaimEvidence,
    ) -> Self {
        Self::new(body, BodyClaimTier::Constrained, accuracy, evidence)
    }

    /// Creates an `Approximate` claim.
    pub fn approximate(body: CelestialBody) -> Self {
        Self::new(
            body,
            BodyClaimTier::Approximate,
            AccuracyClass::Approximate,
            ClaimEvidence::AlgorithmicModel,
        )
    }

    /// Creates an `Unsupported` claim (listed for explicitness; preflight rejects it).
    pub fn unsupported(body: CelestialBody) -> Self {
        Self::new(
            body,
            BodyClaimTier::Unsupported,
            AccuracyClass::Unknown,
            ClaimEvidence::None,
        )
    }

    /// Returns a compact one-line rendering.
    pub fn summary_line(&self) -> String {
        format!(
            "{} [{}; accuracy={}; evidence={}]",
            self.body, self.tier, self.accuracy, self.evidence
        )
    }
}

impl From<CelestialBody> for BodyClaim {
    fn from(body: CelestialBody) -> Self {
        Self::constrained(body, AccuracyClass::Unknown, ClaimEvidence::None)
    }
}

#[cfg(test)]
#[path = "claims_tests.rs"]
mod tests;
