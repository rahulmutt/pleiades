use crate::claims::{BodyClaim, BodyClaimTier};
use crate::identity::BackendId;
use crate::metadata::BackendMetadata;
use pleiades_types::CelestialBody;

/// A derived, cross-backend view of body claims for release reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct ReleasePosture {
    /// Flat (backend, claim) entries, ordered deterministically.
    pub entries: Vec<(BackendId, BodyClaim)>,
}

impl ReleasePosture {
    /// Aggregates claims across the given backend metadata (the caller chooses the set).
    pub fn from_backends(metas: &[&BackendMetadata]) -> Self {
        let mut entries: Vec<(BackendId, BodyClaim)> = Vec::new();
        for meta in metas {
            for claim in &meta.body_claims {
                entries.push((meta.id.clone(), claim.clone()));
            }
        }
        entries.sort_by(|a, b| {
            a.0.as_str()
                .cmp(b.0.as_str())
                .then_with(|| a.1.body.to_string().cmp(&b.1.body.to_string()))
        });
        Self { entries }
    }

    /// Returns the `(backend, body)` pairs claimed `ReleaseGrade`.
    pub fn release_grade(&self) -> Vec<(BackendId, CelestialBody)> {
        self.entries
            .iter()
            .filter(|(_, c)| c.tier == BodyClaimTier::ReleaseGrade)
            .map(|(id, c)| (id.clone(), c.body.clone()))
            .collect()
    }

    /// Returns the entries at a given tier.
    pub fn claims_for_tier(&self, tier: BodyClaimTier) -> Vec<(BackendId, BodyClaim)> {
        self.entries
            .iter()
            .filter(|(_, c)| c.tier == tier)
            .cloned()
            .collect()
    }

    /// Renders a deterministic one-line summary grouped by tier.
    pub fn summary_line(&self) -> String {
        let render = |tier: BodyClaimTier| -> String {
            self.entries
                .iter()
                .filter(|(_, c)| c.tier == tier)
                .map(|(id, c)| format!("{}@{}", c.body, id))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "ReleaseGrade: [{}]; Constrained: [{}]; Approximate: [{}]; Unsupported: [{}]",
            render(BodyClaimTier::ReleaseGrade),
            render(BodyClaimTier::Constrained),
            render(BodyClaimTier::Approximate),
            render(BodyClaimTier::Unsupported),
        )
    }
}

#[cfg(test)]
#[path = "release_posture_tests.rs"]
mod tests;
